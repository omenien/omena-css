import type { BemSuffixInfo, Range } from "@css-module-explainer/shared";
import type { ClassExpressionHIR } from "../hir/source-types";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { ReferenceQueryEnv, ResolvedReferenceSite } from "../query/find-references";
import { findSelectorAtCursor } from "../query/find-style-selector";
import {
  readSelectorRewriteSafetySummary,
  type SelectorRewriteSafetySummary,
} from "../query/read-selector-rewrite-safety";
import type { PlannedTextEdit, TextRewritePlan } from "./text-rewrite-plan";
import { type ClassnameTransformMode, transformClassname } from "../scss/classname-transform";
import { pathToFileUrl } from "../util/text-utils";
import type { Settings } from "../../settings";
import {
  readStyleSelectorRewritePolicy,
  type StyleSelectorRewritePolicySummary,
} from "./read-style-rewrite-policy";

const IDENTIFIER_RE = /^[a-zA-Z_][\w-]*$/;

export type RenameBlockReason =
  | "dynamicExpression"
  | "unsafeSelectorShape"
  | "interpolatedSelector"
  | "aliasViewBlocked"
  | "expandedReferences"
  | "styleDependencyReferences";

export type RenameEditBlockReason =
  | "invalidNewName"
  | "crossParentBemRename"
  | "noopBemRename"
  | "emptyBemSuffixRename";

export interface SelectorRenamePlannerEnv extends ReferenceQueryEnv {
  readonly settings: Pick<Settings, "scss">;
}

export interface SelectorRenameTarget {
  readonly scssPath: string;
  readonly scssUri: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly selector: SelectorDeclHIR;
  readonly styleRewritePolicy: StyleSelectorRewritePolicySummary;
  readonly placeholder: string;
  readonly placeholderRange: Range;
  readonly rewriteSafety: SelectorRewriteSafetySummary;
  readonly styleDependencySites?: readonly ResolvedReferenceSite[];
  readonly aliasMode: ClassnameTransformMode;
}

export type SelectorRenamePlan = TextRewritePlan<SelectorRenameTarget>;

export type SelectorRenamePlanResult =
  | { readonly kind: "plan"; readonly plan: SelectorRenamePlan }
  | { readonly kind: "blocked"; readonly reason: RenameEditBlockReason };

export type SelectorRenameReadResult =
  | { readonly kind: "target"; readonly target: SelectorRenameTarget }
  | { readonly kind: "blocked"; readonly reason: RenameBlockReason }
  | { readonly kind: "miss" };

export function readStyleSelectorRenameTargetAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  env: SelectorRenamePlannerEnv,
): SelectorRenameReadResult {
  const selector = findSelectorAtCursor(styleDocument, line, character);
  if (!selector) return { kind: "miss" };
  return finalizeSelectorRenameTarget(
    {
      scssPath: filePath,
      styleDocument,
      selector,
      placeholder: selector.name,
      placeholderRange: selector.bemSuffix?.rawTokenRange ?? selector.range,
      rejectAliasSelectorViews: true,
    },
    env,
  );
}

export function readExpressionRenameTarget(
  expression: ClassExpressionHIR,
  styleDocument: StyleDocumentHIR,
  env: SelectorRenamePlannerEnv,
): SelectorRenameReadResult {
  if (expression.kind === "template" || expression.kind === "symbolRef") {
    return { kind: "blocked", reason: "dynamicExpression" };
  }
  if (expression.kind !== "literal" && expression.kind !== "styleAccess") {
    return { kind: "miss" };
  }
  const selector =
    styleDocument.selectors.find(
      (candidate): candidate is SelectorDeclHIR => candidate.name === expression.className,
    ) ?? null;
  if (!selector) return { kind: "miss" };
  return finalizeSelectorRenameTarget(
    {
      scssPath: expression.scssModulePath,
      styleDocument,
      selector,
      placeholder: expression.className,
      placeholderRange: expression.range,
      rejectAliasSelectorViews: false,
    },
    env,
  );
}

export function planSelectorRename(
  target: SelectorRenameTarget,
  newName: string,
): SelectorRenamePlanResult {
  if (!IDENTIFIER_RE.test(newName)) {
    return { kind: "blocked", reason: "invalidNewName" };
  }

  const scssEdit =
    target.styleRewritePolicy.rewriteShape === "bemSuffix" && target.styleRewritePolicy.bemSuffix
      ? buildBemSuffixEdit(
          target.scssUri,
          target.styleRewritePolicy.bemSuffix,
          target.styleRewritePolicy.canonicalSelector.name,
          newName,
        )
      : {
          uri: target.scssUri,
          range: target.styleRewritePolicy.canonicalSelector.range,
          newText: newName,
        };
  if ("kind" in scssEdit) return scssEdit;

  const edits: PlannedTextEdit[] = [scssEdit];
  for (const site of target.rewriteSafety.directSites) {
    const written = site.className;
    const newText =
      written === target.styleRewritePolicy.canonicalName
        ? newName
        : (pickAliasForm(target.aliasMode, newName) ?? newName);
    edits.push({
      uri: site.uri,
      range: site.range,
      newText,
    });
  }
  for (const site of target.styleDependencySites ?? []) {
    edits.push({
      uri: site.uri,
      range: site.range,
      newText: newName,
    });
  }

  return { kind: "plan", plan: { target, edits } };
}

export function renameBlockReasonMessage(reason: RenameBlockReason): string {
  switch (reason) {
    case "dynamicExpression":
      return "Dynamic class expressions cannot be renamed safely.";
    case "interpolatedSelector":
      return "Selectors containing interpolation cannot be renamed safely.";
    case "unsafeSelectorShape":
      return "Only flat selectors and safe BEM suffix selectors can be renamed.";
    case "aliasViewBlocked":
      return "Alias selector views cannot be renamed under the current classnameTransform mode.";
    case "expandedReferences":
      return "Rename is blocked because inferred or expanded references would make the edit unsafe.";
    case "styleDependencyReferences":
      return "Rename is blocked because composed-style references are not rewritten automatically.";
    default:
      reason satisfies never;
      return "Rename cannot be performed safely.";
  }
}

interface FinalizeRenameTargetArgs {
  readonly scssPath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly selector: SelectorDeclHIR;
  readonly placeholder: string;
  readonly placeholderRange: Range;
  readonly rejectAliasSelectorViews: boolean;
}

function finalizeSelectorRenameTarget(
  args: FinalizeRenameTargetArgs,
  env: SelectorRenamePlannerEnv,
): SelectorRenameReadResult {
  const aliasMode = env.settings.scss.classnameTransform;
  const rewritePolicy = readStyleSelectorRewritePolicy({
    styleDocument: args.styleDocument,
    selector: args.selector,
    aliasMode,
    rejectAliasSelectorViews: args.rejectAliasSelectorViews,
  });
  if (rewritePolicy.kind === "blocked") {
    return rewritePolicy;
  }

  const rewriteSafety = readSelectorRewriteSafetySummary(
    env,
    args.scssPath,
    rewritePolicy.summary.canonicalName,
  );
  if (rewriteSafety.hasBlockingStyleDependencyReferences) {
    const styleDependencySites = collectDirectStyleDependencyRenameSites(
      env,
      args.scssPath,
      rewritePolicy.summary.canonicalName,
    );
    if (!canRewriteStyleDependencyReferences(rewriteSafety, styleDependencySites)) {
      return { kind: "blocked", reason: "styleDependencyReferences" };
    }
    return {
      kind: "target",
      target: {
        scssPath: args.scssPath,
        scssUri: pathToFileUrl(args.scssPath),
        styleDocument: args.styleDocument,
        selector: args.selector,
        styleRewritePolicy: rewritePolicy.summary,
        placeholder: args.placeholder,
        placeholderRange: args.placeholderRange,
        rewriteSafety,
        styleDependencySites,
        aliasMode,
      },
    };
  }
  if (rewriteSafety.hasBlockingExpandedReferences) {
    return { kind: "blocked", reason: "expandedReferences" };
  }

  return {
    kind: "target",
    target: {
      scssPath: args.scssPath,
      scssUri: pathToFileUrl(args.scssPath),
      styleDocument: args.styleDocument,
      selector: args.selector,
      styleRewritePolicy: rewritePolicy.summary,
      placeholder: args.placeholder,
      placeholderRange: args.placeholderRange,
      rewriteSafety,
      aliasMode,
    },
  };
}

export function collectDirectStyleDependencyRenameSites(
  deps: ReferenceQueryEnv,
  scssPath: string,
  canonicalName: string,
): readonly ResolvedReferenceSite[] {
  if (!deps.styleDependencyGraph) return [];
  return deps.styleDependencyGraph.getIncoming(scssPath, canonicalName).flatMap((incoming) =>
    incoming.range
      ? [
          {
            uri: pathToFileUrl(incoming.filePath),
            range: incoming.range,
            className: canonicalName,
            selectorCertainty: "exact" as const,
            expansion: "direct" as const,
            referenceKind: "styleDependency" as const,
          },
        ]
      : [],
  );
}

export function canRewriteStyleDependencyReferences(
  rewriteSafety: SelectorRewriteSafetySummary,
  styleDependencySites: readonly ResolvedReferenceSite[],
): boolean {
  const styleDependencyReferenceCount = rewriteSafety.usage.allSites.filter(
    (site) => site.referenceKind === "styleDependency",
  ).length;
  return (
    styleDependencyReferenceCount > 0 &&
    styleDependencySites.length === styleDependencyReferenceCount &&
    !rewriteSafety.hasBlockingExpandedReferences
  );
}

function pickAliasForm(mode: ClassnameTransformMode, newName: string): string | null {
  const forms = transformClassname(mode, newName);
  for (const form of forms) {
    if (form !== newName) return form;
  }
  return null;
}

function buildBemSuffixEdit(
  uri: string,
  bemSuffix: BemSuffixInfo,
  oldName: string,
  newName: string,
): PlannedTextEdit | { readonly kind: "blocked"; readonly reason: RenameEditBlockReason } {
  const { parentResolvedName: parent, rawToken, rawTokenRange: rawRange } = bemSuffix;

  if (!oldName.startsWith(parent)) return { kind: "blocked", reason: "crossParentBemRename" };
  if (!newName.startsWith(parent)) return { kind: "blocked", reason: "crossParentBemRename" };

  const oldSuffix = oldName.slice(parent.length);
  const newSuffix = newName.slice(parent.length);

  if (oldSuffix === newSuffix) return { kind: "blocked", reason: "noopBemRename" };
  if (newSuffix.length === 0) return { kind: "blocked", reason: "emptyBemSuffixRename" };

  const suffixOffset = rawToken.indexOf(oldSuffix);
  if (suffixOffset !== 1) return { kind: "blocked", reason: "crossParentBemRename" };

  return {
    uri,
    range: {
      start: {
        line: rawRange.start.line,
        character: rawRange.start.character + suffixOffset,
      },
      end: {
        line: rawRange.start.line,
        character: rawRange.start.character + suffixOffset + oldSuffix.length,
      },
    },
    newText: newSuffix,
  };
}
