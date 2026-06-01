import type { BemSuffixInfo } from "@omena/shared";
import type { ClassnameTransformMode } from "../scss/classname-transform";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import { findCanonicalSelector } from "../query/find-style-selector";
import type { RenameBlockReason } from "./selector-rename";

export type StyleSelectorRewriteShape = "direct" | "bemSuffix";

export interface StyleSelectorRewritePolicySummary {
  readonly selector: SelectorDeclHIR;
  readonly canonicalSelector: SelectorDeclHIR;
  readonly canonicalName: string;
  readonly rewriteShape: StyleSelectorRewriteShape;
  readonly bemSuffix: BemSuffixInfo | null;
}

export type StyleSelectorRewritePolicyReadResult =
  | { readonly kind: "policy"; readonly summary: StyleSelectorRewritePolicySummary }
  | { readonly kind: "blocked"; readonly reason: RenameBlockReason };

export interface ReadStyleSelectorRewritePolicyArgs {
  readonly styleDocument: StyleDocumentHIR;
  readonly selector: SelectorDeclHIR;
  readonly aliasMode: ClassnameTransformMode;
  readonly rejectAliasSelectorViews: boolean;
}

export function readStyleSelectorRewritePolicy(
  args: ReadStyleSelectorRewritePolicyArgs,
): StyleSelectorRewritePolicyReadResult {
  const canonicalSelector = findCanonicalSelector(args.styleDocument, args.selector);
  const shape = rewriteShapeForSelector(canonicalSelector);
  if (shape.kind === "blocked") {
    return { kind: "blocked", reason: shape.reason };
  }

  if (
    args.rejectAliasSelectorViews &&
    (args.aliasMode === "camelCaseOnly" || args.aliasMode === "dashesOnly") &&
    args.selector.viewKind === "alias"
  ) {
    return { kind: "blocked", reason: "aliasViewBlocked" };
  }

  return {
    kind: "policy",
    summary: {
      selector: args.selector,
      canonicalSelector,
      canonicalName: canonicalSelector.canonicalName,
      rewriteShape: shape.rewriteShape,
      bemSuffix: shape.bemSuffix,
    },
  };
}

function rewriteShapeForSelector(selector: SelectorDeclHIR):
  | {
      readonly kind: "policy";
      readonly rewriteShape: StyleSelectorRewriteShape;
      readonly bemSuffix: BemSuffixInfo | null;
    }
  | { readonly kind: "blocked"; readonly reason: RenameBlockReason } {
  if (selector.nestedSafety === "flat") {
    return {
      kind: "policy",
      rewriteShape: "direct",
      bemSuffix: null,
    };
  }
  if (selector.nestedSafety !== "bemSuffixSafe" || !selector.bemSuffix) {
    return { kind: "blocked", reason: "unsafeSelectorShape" };
  }
  if (selector.bemSuffix.rawToken.includes("#{")) {
    return { kind: "blocked", reason: "interpolatedSelector" };
  }
  return {
    kind: "policy",
    rewriteShape: "bemSuffix",
    bemSuffix: selector.bemSuffix,
  };
}
