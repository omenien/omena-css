import {
  planSelectorRename,
  type RenameEditBlockReason,
  type SelectorRenamePlanResult,
  type SelectorRenameReadResult,
  type SelectorRenameTarget,
} from "../../engine-core-ts/src/core/rewrite/selector-rename";
import { readStyleSelectorRewritePolicy } from "../../engine-core-ts/src/core/rewrite/read-style-rewrite-policy";
import {
  findCustomPropertyDeclAtCursor,
  findCustomPropertyRefAtCursor,
  findSassSymbolAtCursor,
  findSassSymbolDeclAtCursor,
  findSassSymbolDeclForSymbol,
  findSassModuleMemberRefAtCursor,
  findSelectorAtCursor,
  listCustomPropertyRefs,
  listSassSymbolsForDecl,
  resolveCustomPropertyDeclTarget,
  resolveSassModuleMemberRefTarget,
  resolveSassWildcardSymbolTarget,
  readSelectorRewriteSafetySummary,
} from "../../engine-core-ts/src/core/query";
import type { ResolvedReferenceSite } from "../../engine-core-ts/src/core/query/find-references";
import type { SelectorReferenceRewritePolicy } from "../../engine-core-ts/src/core/query/read-selector-rewrite-safety";
import type {
  CustomPropertyDeclHIR,
  CustomPropertyRefHIR,
  SassSymbolDeclHIR,
  StylePreprocessorSymbolSyntax,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import type {
  PlannedTextEdit,
  TextRewritePlan,
} from "../../engine-core-ts/src/core/rewrite/text-rewrite-plan";
import {
  resolveSelectedQueryBackendKind,
  usesRustSelectorUsageBackend,
} from "./selected-query-backend";
import {
  resolveRustStyleSelectorIdentityReadModelForWorkspaceTarget,
  type StyleSelectorIdentityQueryOptions,
} from "./style-selector-identity-query";
import {
  buildSelectorReferenceRewriteSafetyFromRustGraph,
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget,
} from "./style-selector-reference-query";
import {
  buildSelectorUsageEditableDirectSitesFromRustPayload,
  resolveRustSelectorUsagePayloadForWorkspaceTarget,
  type SelectorUsageEvaluatorCandidatePayloadV0,
} from "./selector-usage-query-backend";

export interface StyleRenameQueryOptions extends StyleSelectorIdentityQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSelectorUsagePayloadForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadForWorkspaceTarget;
}

export interface SassSymbolRenameTarget {
  readonly scssPath: string;
  readonly scssUri: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly symbolSyntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolDeclHIR["symbolKind"];
  readonly name: string;
  readonly targetDecl: SassSymbolDeclHIR;
  readonly placeholder: string;
  readonly placeholderRange: SassSymbolDeclHIR["range"];
}

export type SassSymbolRenameReadResult =
  | { readonly kind: "target"; readonly target: SassSymbolRenameTarget }
  | { readonly kind: "miss" };

export interface CustomPropertyRenameTarget {
  readonly scssPath: string;
  readonly scssUri: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly name: string;
  readonly targetDecl: Pick<
    CustomPropertyDeclHIR,
    "name" | "value" | "range" | "ruleRange" | "context"
  >;
  readonly targetDeclPath: string;
  readonly placeholder: string;
  readonly placeholderRange: CustomPropertyDeclHIR["range"] | CustomPropertyRefHIR["range"];
}

export type CustomPropertyRenameReadResult =
  | { readonly kind: "target"; readonly target: CustomPropertyRenameTarget }
  | { readonly kind: "miss" };

export type StyleRenameReadResult =
  | SelectorRenameReadResult
  | SassSymbolRenameReadResult
  | CustomPropertyRenameReadResult;

export type SassSymbolRenamePlanResult =
  | { readonly kind: "plan"; readonly plan: TextRewritePlan<SassSymbolRenameTarget> }
  | { readonly kind: "blocked"; readonly reason: RenameEditBlockReason };

export type CustomPropertyRenamePlanResult =
  | { readonly kind: "plan"; readonly plan: TextRewritePlan<CustomPropertyRenameTarget> }
  | { readonly kind: "blocked"; readonly reason: RenameEditBlockReason };

export type StyleRenamePlanResult =
  | SelectorRenamePlanResult
  | SassSymbolRenamePlanResult
  | CustomPropertyRenamePlanResult;

export function readStyleRenameTargetAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "aliasResolver"
    | "settings"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: StyleRenameQueryOptions = {},
): StyleRenameReadResult {
  const selectorResult = readStyleSelectorRenameTargetAtCursor(
    filePath,
    line,
    character,
    styleDocument,
    deps,
    options,
  );
  if (selectorResult.kind !== "miss") return selectorResult;
  const customPropertyResult = readCustomPropertyRenameTargetAtCursor(
    filePath,
    line,
    character,
    styleDocument,
    deps,
  );
  if (customPropertyResult.kind !== "miss") return customPropertyResult;
  return readSassSymbolRenameTargetAtCursor(filePath, line, character, styleDocument, deps);
}

function readStyleSelectorRenameTargetAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "aliasResolver"
    | "settings"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: StyleRenameQueryOptions,
): SelectorRenameReadResult {
  const selector = findSelectorAtCursor(styleDocument, line, character);
  if (!selector) return { kind: "miss" };

  const aliasMode = deps.settings.scss.classnameTransform;
  const rewritePolicy = readStyleSelectorRewritePolicy({
    styleDocument,
    selector,
    aliasMode,
    rejectAliasSelectorViews: true,
  });
  if (rewritePolicy.kind === "blocked") {
    return rewritePolicy;
  }

  const rustIdentity = resolveRustStyleSelectorIdentityReadModelForWorkspaceTarget(
    {
      filePath,
      styleDocument,
      canonicalName: rewritePolicy.summary.canonicalName,
    },
    deps,
    options,
  );
  if (rustIdentity?.rewriteSafety === "blocked") {
    return { kind: "blocked", reason: "unsafeSelectorShape" };
  }

  const rewriteSafety = resolveStyleRenameRewriteSafety(
    filePath,
    rewritePolicy.summary.canonicalName,
    deps,
    options,
  );
  if (rewriteSafety.hasBlockingStyleDependencyReferences) {
    return { kind: "blocked", reason: "styleDependencyReferences" };
  }
  if (rewriteSafety.hasBlockingExpandedReferences) {
    return { kind: "blocked", reason: "expandedReferences" };
  }

  const target: SelectorRenameTarget = {
    scssPath: filePath,
    scssUri: pathToFileUrl(filePath),
    styleDocument,
    selector,
    styleRewritePolicy: rewritePolicy.summary,
    placeholder: selector.name,
    placeholderRange: selector.bemSuffix?.rawTokenRange ?? selector.range,
    rewriteSafety,
    aliasMode,
  };
  return {
    kind: "target",
    target,
  };
}

function readSassSymbolRenameTargetAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  deps: Pick<ProviderDeps, "aliasResolver" | "readStyleFile" | "styleDocumentForPath">,
): SassSymbolRenameReadResult {
  const decl = findSassSymbolDeclAtCursor(styleDocument, line, character);
  if (decl) {
    return {
      kind: "target",
      target: makeSassSymbolRenameTarget(
        filePath,
        styleDocument,
        decl.syntax,
        decl.symbolKind,
        decl.name,
        decl,
        decl.range,
      ),
    };
  }

  const moduleMemberRef = findSassModuleMemberRefAtCursor(styleDocument, line, character);
  if (moduleMemberRef) {
    const target = resolveSassModuleMemberRefTarget(
      deps.styleDocumentForPath,
      filePath,
      styleDocument,
      moduleMemberRef,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    if (!target) return { kind: "miss" };
    return {
      kind: "target",
      target: makeSassSymbolRenameTarget(
        target.filePath,
        target.styleDocument,
        target.decl.syntax,
        target.decl.symbolKind,
        target.decl.name,
        target.decl,
        moduleMemberRef.range,
      ),
    };
  }

  const symbol = findSassSymbolAtCursor(styleDocument, line, character);
  if (!symbol) return { kind: "miss" };
  const targetDecl = findSassSymbolDeclForSymbol(styleDocument, symbol);
  if (!targetDecl) {
    const wildcardTarget = resolveSassWildcardSymbolTarget(
      deps.styleDocumentForPath,
      filePath,
      styleDocument,
      symbol,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    if (!wildcardTarget) return { kind: "miss" };
    return {
      kind: "target",
      target: makeSassSymbolRenameTarget(
        wildcardTarget.filePath,
        wildcardTarget.styleDocument,
        wildcardTarget.decl.syntax,
        wildcardTarget.decl.symbolKind,
        wildcardTarget.decl.name,
        wildcardTarget.decl,
        symbol.range,
      ),
    };
  }
  return {
    kind: "target",
    target: makeSassSymbolRenameTarget(
      filePath,
      styleDocument,
      targetDecl.syntax,
      symbol.symbolKind,
      symbol.name,
      targetDecl,
      symbol.range,
    ),
  };
}

function readCustomPropertyRenameTargetAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    "aliasResolver" | "readStyleFile" | "styleDependencyGraph" | "styleDocumentForPath"
  >,
): CustomPropertyRenameReadResult {
  const decl = findCustomPropertyDeclAtCursor(styleDocument, line, character);
  if (decl) {
    return {
      kind: "target",
      target: makeCustomPropertyRenameTarget(
        filePath,
        styleDocument,
        decl.name,
        decl,
        filePath,
        decl.range,
      ),
    };
  }

  const ref = findCustomPropertyRefAtCursor(styleDocument, line, character);
  if (!ref) return { kind: "miss" };

  const target = resolveCustomPropertyDeclTarget(
    deps.styleDocumentForPath,
    filePath,
    styleDocument,
    ref,
    deps.styleDependencyGraph,
    deps.aliasResolver,
    { readFile: deps.readStyleFile },
  );

  return {
    kind: "target",
    target: makeCustomPropertyRenameTarget(
      filePath,
      styleDocument,
      ref.name,
      target?.decl ?? refToDeclarationLike(ref),
      target?.filePath ?? filePath,
      ref.range,
    ),
  };
}

export function planStyleRenameAtCursor(
  filePath: string,
  line: number,
  character: number,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "aliasResolver"
    | "settings"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  newName: string,
  options: StyleRenameQueryOptions = {},
): StyleRenamePlanResult | null {
  const result = readStyleRenameTargetAtCursor(
    filePath,
    line,
    character,
    styleDocument,
    deps,
    options,
  );
  if (result.kind !== "target") return null;
  if (isCustomPropertyRenameTarget(result.target)) {
    return planCustomPropertyRename(result.target, newName, deps);
  }
  if (isSassSymbolRenameTarget(result.target)) {
    return planSassSymbolRename(result.target, newName, deps);
  }
  return planSelectorRename(result.target, newName);
}

function makeSassSymbolRenameTarget(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  symbolSyntax: StylePreprocessorSymbolSyntax | undefined,
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
  targetDecl: SassSymbolDeclHIR,
  placeholderRange: SassSymbolDeclHIR["range"],
): SassSymbolRenameTarget {
  return {
    scssPath: filePath,
    scssUri: pathToFileUrl(filePath),
    styleDocument,
    ...(symbolSyntax ? { symbolSyntax } : {}),
    symbolKind,
    name,
    targetDecl,
    placeholder: formatSassSymbolText(symbolKind, name, symbolSyntax),
    placeholderRange,
  };
}

function isSassSymbolRenameTarget(
  target: SelectorRenameTarget | SassSymbolRenameTarget,
): target is SassSymbolRenameTarget {
  return "symbolKind" in target;
}

function isCustomPropertyRenameTarget(
  target: SelectorRenameTarget | SassSymbolRenameTarget | CustomPropertyRenameTarget,
): target is CustomPropertyRenameTarget {
  return "targetDecl" in target && "targetDeclPath" in target;
}

const SASS_IDENTIFIER_RE = /^[a-zA-Z_][\w-]*$/;
const CUSTOM_PROPERTY_IDENTIFIER_RE = /^--[a-zA-Z_][\w-]*$/;

function makeCustomPropertyRenameTarget(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  name: string,
  targetDecl: Pick<CustomPropertyDeclHIR, "name" | "value" | "range" | "ruleRange" | "context">,
  targetDeclPath: string,
  placeholderRange: CustomPropertyDeclHIR["range"] | CustomPropertyRefHIR["range"],
): CustomPropertyRenameTarget {
  return {
    scssPath: filePath,
    scssUri: pathToFileUrl(filePath),
    styleDocument,
    name,
    targetDecl,
    targetDeclPath,
    placeholder: name,
    placeholderRange,
  };
}

function refToDeclarationLike(
  ref: CustomPropertyRefHIR,
): Pick<CustomPropertyDeclHIR, "name" | "value" | "range" | "ruleRange" | "context"> {
  return {
    name: ref.name,
    value: "",
    range: ref.range,
    ruleRange: ref.range,
    context: ref.context,
  };
}

function planCustomPropertyRename(
  target: CustomPropertyRenameTarget,
  newName: string,
  deps: Pick<ProviderDeps, "styleDependencyGraph" | "styleDocumentForPath">,
): CustomPropertyRenamePlanResult {
  const nextName = normalizeCustomPropertyNewName(newName);
  if (!nextName) return { kind: "blocked", reason: "invalidNewName" };

  const edits: PlannedTextEdit[] = [];
  pushCustomPropertyRenameEdit(edits, {
    uri: pathToFileUrl(target.targetDeclPath),
    range: target.targetDecl.range,
    newText: nextName,
  });

  const targetDocument = deps.styleDocumentForPath(target.targetDeclPath);
  if (targetDocument) {
    for (const decl of targetDocument.customPropertyDecls) {
      if (decl.name !== target.name) continue;
      pushCustomPropertyRenameEdit(edits, {
        uri: pathToFileUrl(target.targetDeclPath),
        range: decl.range,
        newText: nextName,
      });
    }
    for (const ref of listCustomPropertyRefs(targetDocument, target.name)) {
      pushCustomPropertyRenameEdit(edits, {
        uri: pathToFileUrl(target.targetDeclPath),
        range: ref.range,
        newText: nextName,
      });
    }
  }

  for (const decl of target.styleDocument.customPropertyDecls) {
    if (decl.name !== target.name) continue;
    pushCustomPropertyRenameEdit(edits, {
      uri: target.scssUri,
      range: decl.range,
      newText: nextName,
    });
  }
  for (const ref of listCustomPropertyRefs(target.styleDocument, target.name)) {
    pushCustomPropertyRenameEdit(edits, {
      uri: target.scssUri,
      range: ref.range,
      newText: nextName,
    });
  }

  for (const decl of deps.styleDependencyGraph.getCustomPropertyDecls(target.name)) {
    pushCustomPropertyRenameEdit(edits, {
      uri: pathToFileUrl(decl.filePath),
      range: decl.range,
      newText: nextName,
    });
  }
  for (const ref of deps.styleDependencyGraph.getCustomPropertyRefs(target.name)) {
    pushCustomPropertyRenameEdit(edits, {
      uri: pathToFileUrl(ref.filePath),
      range: ref.range,
      newText: nextName,
    });
  }

  return { kind: "plan", plan: { target, edits } };
}

function pushCustomPropertyRenameEdit(edits: PlannedTextEdit[], edit: PlannedTextEdit): void {
  pushSassSymbolRenameEdit(edits, edit);
}

function normalizeCustomPropertyNewName(newName: string): string | null {
  const trimmed = newName.trim();
  const candidate = trimmed.startsWith("--") ? trimmed : `--${trimmed}`;
  return CUSTOM_PROPERTY_IDENTIFIER_RE.test(candidate) ? candidate : null;
}

function planSassSymbolRename(
  target: SassSymbolRenameTarget,
  newName: string,
  deps: Pick<ProviderDeps, "styleDependencyGraph">,
): SassSymbolRenamePlanResult {
  const nextName = normalizeSassSymbolNewName(target.symbolKind, newName, target.symbolSyntax);
  if (!nextName) return { kind: "blocked", reason: "invalidNewName" };

  const newText = formatSassSymbolText(target.symbolKind, nextName, target.symbolSyntax);
  const edits: PlannedTextEdit[] = [];
  pushSassSymbolRenameEdit(edits, {
    uri: target.scssUri,
    range: target.targetDecl.range,
    newText,
  });
  for (const symbol of listSassSymbolsForDecl(target.styleDocument, target.targetDecl)) {
    pushSassSymbolRenameEdit(edits, {
      uri: target.scssUri,
      range: symbol.range,
      newText,
    });
  }
  for (const ref of deps.styleDependencyGraph.getIncomingSassModuleMemberRefs(
    target.scssPath,
    target.symbolKind,
    target.name,
  )) {
    pushSassSymbolRenameEdit(edits, {
      uri: pathToFileUrl(ref.filePath),
      range: ref.range,
      newText,
    });
  }

  return { kind: "plan", plan: { target, edits } };
}

function pushSassSymbolRenameEdit(edits: PlannedTextEdit[], edit: PlannedTextEdit): void {
  if (
    edits.some(
      (existing) =>
        existing.uri === edit.uri &&
        existing.range.start.line === edit.range.start.line &&
        existing.range.start.character === edit.range.start.character &&
        existing.range.end.line === edit.range.end.line &&
        existing.range.end.character === edit.range.end.character,
    )
  ) {
    return;
  }
  edits.push(edit);
}

function normalizeSassSymbolNewName(
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  newName: string,
  symbolSyntax: StylePreprocessorSymbolSyntax | undefined,
): string | null {
  const trimmed = newName.trim();
  if (symbolSyntax === "less") {
    const name = trimmed.startsWith("@") ? trimmed.slice(1) : trimmed;
    return SASS_IDENTIFIER_RE.test(name) ? name : null;
  }
  if (symbolKind === "variable") {
    const name = trimmed.startsWith("$") ? trimmed.slice(1) : trimmed;
    return SASS_IDENTIFIER_RE.test(name) ? name : null;
  }
  if (trimmed.startsWith("$") || trimmed.startsWith("@")) return null;
  return SASS_IDENTIFIER_RE.test(trimmed) ? trimmed : null;
}

function formatSassSymbolText(
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
  symbolSyntax: StylePreprocessorSymbolSyntax | undefined,
): string {
  if (symbolSyntax === "less") return `@${name}`;
  return symbolKind === "variable" ? `$${name}` : name;
}

function resolveStyleRenameRewriteSafety(
  filePath: string,
  canonicalName: string,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "settings"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: StyleRenameQueryOptions,
) {
  const base = readSelectorRewriteSafetySummary(deps, filePath, canonicalName);
  const graphReferences = resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget(
    { filePath, canonicalName },
    deps,
    options,
  );
  if (graphReferences) {
    return buildSelectorReferenceRewriteSafetyFromRustGraph(base, graphReferences);
  }

  if (!usesRustSelectorUsageBackend(resolveSelectedQueryBackendKind(options.env))) {
    return base;
  }

  const payload = (
    options.readRustSelectorUsagePayloadForWorkspaceTarget ??
    resolveRustSelectorUsagePayloadForWorkspaceTarget
  )(
    {
      workspaceRoot: deps.workspaceRoot,
      classnameTransform: deps.settings.scss.classnameTransform,
      pathAlias: deps.settings.pathAlias,
    },
    deps,
    filePath,
    canonicalName,
  );
  if (!payload) return base;

  const hasBlockingStyleDependencyReferences = payload.hasStyleDependencyReferences;
  const hasBlockingExpandedReferences = payload.hasExpandedReferences;
  const rustEditableDirectSites = buildRustEditableDirectSites(payload);
  const referenceRewritePolicy: SelectorReferenceRewritePolicy =
    hasBlockingStyleDependencyReferences
      ? "blockedByStyleDependencies"
      : hasBlockingExpandedReferences
        ? "blockedByExpandedReferences"
        : "directOnly";
  return {
    ...base,
    usage: {
      ...base.usage,
      editableDirectSites: rustEditableDirectSites ?? base.usage.editableDirectSites,
      totalReferences: payload.totalReferences,
      directReferenceCount: payload.directReferenceCount,
      hasExpandedReferences: payload.hasExpandedReferences,
      hasStyleDependencyReferences: payload.hasStyleDependencyReferences,
      hasAnyReferences: payload.hasAnyReferences,
    },
    directSites: rustEditableDirectSites ?? base.directSites,
    referenceRewritePolicy,
    hasBlockingExpandedReferences,
    hasBlockingStyleDependencyReferences,
  };
}

function buildRustEditableDirectSites(
  payload: SelectorUsageEvaluatorCandidatePayloadV0,
): readonly ResolvedReferenceSite[] | null {
  const editableDirectSites = buildSelectorUsageEditableDirectSitesFromRustPayload(payload);
  if (!editableDirectSites) return null;
  return editableDirectSites.map((site) => ({
    uri: pathToFileUrl(site.filePath),
    range: site.range,
    className: site.className,
    selectorCertainty: "exact",
    expansion: "direct",
    referenceKind: "source",
  }));
}
