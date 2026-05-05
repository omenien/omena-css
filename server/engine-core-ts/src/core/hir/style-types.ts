import type { BemSuffixInfo, ComposesRef, Range } from "@css-module-explainer/shared";
import type { HirDocumentBase, HirNodeBase } from "./shared-types";

export interface StyleDocumentHIR extends HirDocumentBase {
  readonly kind: "style";
  readonly selectors: readonly SelectorDeclHIR[];
  readonly keyframes: readonly KeyframesDeclHIR[];
  readonly animationNameRefs: readonly AnimationNameRefHIR[];
  readonly valueDecls: readonly ValueDeclHIR[];
  readonly valueImports: readonly ValueImportHIR[];
  readonly valueRefs: readonly ValueRefHIR[];
  readonly customPropertyDecls: readonly CustomPropertyDeclHIR[];
  readonly customPropertyRefs: readonly CustomPropertyRefHIR[];
  readonly sassSymbols: readonly SassSymbolOccurrenceHIR[];
  readonly sassSymbolDecls: readonly SassSymbolDeclHIR[];
  readonly sassModuleUses: readonly SassModuleUseHIR[];
  readonly sassModuleForwards: readonly SassModuleForwardHIR[];
  readonly sassModuleMemberRefs: readonly SassModuleMemberRefHIR[];
}

export type SelectorViewKind = "canonical" | "alias";

export type NestedSelectorSafety = "flat" | "bemSuffixSafe" | "nestedUnsafe";

export type SassSymbolKind = "variable" | "mixin" | "function";

export type StylePreprocessorSymbolSyntax = "sass" | "less";

export type SassSymbolRole = "reference" | "include" | "call";

export type SassSymbolResolution = "resolved" | "unresolved";

export type SassModuleUseNamespaceKind = "default" | "alias" | "wildcard";

export interface StyleAtRuleContextHIR {
  readonly name: string;
  readonly params: string;
  readonly range: Range;
}

export interface SelectorRuleContextHIR {
  readonly wrapperAtRules: readonly StyleAtRuleContextHIR[];
}

export interface SelectorDeclHIR extends HirNodeBase {
  readonly kind: "selector";
  readonly range: Range;
  readonly name: string;
  readonly canonicalName: string;
  readonly viewKind: SelectorViewKind;
  readonly fullSelector: string;
  readonly declarations: string;
  readonly ruleRange: Range;
  readonly composes: readonly ComposesRef[];
  readonly nestedSafety: NestedSelectorSafety;
  readonly bemSuffix?: BemSuffixInfo;
  readonly originalName?: string;
  readonly context?: SelectorRuleContextHIR;
}

export interface KeyframesDeclHIR extends HirNodeBase {
  readonly kind: "keyframes";
  readonly range: Range;
  readonly name: string;
  readonly ruleRange: Range;
}

export interface AnimationNameRefHIR extends HirNodeBase {
  readonly kind: "animationNameRef";
  readonly range: Range;
  readonly name: string;
  readonly property: "animation" | "animation-name";
}

export interface ValueDeclHIR extends HirNodeBase {
  readonly kind: "valueDecl";
  readonly range: Range;
  readonly name: string;
  readonly value: string;
  readonly ruleRange: Range;
}

export interface ValueImportHIR extends HirNodeBase {
  readonly kind: "valueImport";
  readonly range: Range;
  readonly name: string;
  readonly importedName: string;
  readonly importedNameRange?: Range;
  readonly from: string;
  readonly ruleRange: Range;
}

export interface ValueRefHIR extends HirNodeBase {
  readonly kind: "valueRef";
  readonly range: Range;
  readonly name: string;
  readonly source: "declaration" | "valueDecl";
}

export type CustomPropertyDeclContainerKind = "root" | "rule" | "atrule";

export interface CustomPropertyDeclAtRuleContextHIR extends StyleAtRuleContextHIR {}

export interface CustomPropertyDeclContextHIR {
  readonly containerKind: CustomPropertyDeclContainerKind;
  readonly selectorText: string | null;
  readonly atRuleName: string | null;
  readonly atRuleParams: string | null;
  readonly wrapperAtRules: readonly CustomPropertyDeclAtRuleContextHIR[];
}

export type CustomPropertyRefContextHIR = CustomPropertyDeclContextHIR;

export interface CustomPropertyDeclHIR extends HirNodeBase {
  readonly kind: "customPropertyDecl";
  readonly range: Range;
  readonly name: string;
  readonly value: string;
  readonly ruleRange: Range;
  readonly context: CustomPropertyDeclContextHIR;
}

export interface CustomPropertyRefHIR extends HirNodeBase {
  readonly kind: "customPropertyRef";
  readonly range: Range;
  readonly name: string;
  readonly context: CustomPropertyRefContextHIR;
}

export interface SassSymbolOccurrenceHIR extends HirNodeBase {
  readonly kind: "sassSymbol";
  readonly selectorName: string;
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly role: SassSymbolRole;
  readonly resolution: SassSymbolResolution;
  readonly range: Range;
  readonly ruleRange: Range;
}

export interface SassSymbolDeclHIR extends HirNodeBase {
  readonly kind: "sassSymbolDecl";
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly range: Range;
  readonly ruleRange: Range;
}

export interface SassModuleUseHIR extends HirNodeBase {
  readonly kind: "sassModuleUse";
  readonly source: string;
  readonly namespaceKind: SassModuleUseNamespaceKind;
  readonly namespace: string | null;
  readonly range: Range;
  readonly ruleRange: Range;
}

export type SassModuleForwardVisibilityKind = "all" | "show" | "hide";

export interface SassModuleForwardMemberHIR {
  readonly name: string;
  readonly symbolKind: "variable" | null;
}

export interface SassModuleForwardHIR extends HirNodeBase {
  readonly kind: "sassModuleForward";
  readonly source: string;
  readonly prefix: string;
  readonly visibilityKind: SassModuleForwardVisibilityKind;
  readonly visibilityMembers: readonly SassModuleForwardMemberHIR[];
  readonly range: Range;
  readonly ruleRange: Range;
}

export interface SassModuleMemberRefHIR extends HirNodeBase {
  readonly kind: "sassModuleMemberRef";
  readonly selectorName: string;
  readonly namespace: string;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly role: SassSymbolRole;
  readonly range: Range;
  readonly ruleRange: Range;
}

export function makeStyleDocumentHIR(
  filePath: string,
  selectors: readonly SelectorDeclHIR[],
  keyframes: readonly KeyframesDeclHIR[] = [],
  animationNameRefs: readonly AnimationNameRefHIR[] = [],
  valueDecls: readonly ValueDeclHIR[] = [],
  valueImports: readonly ValueImportHIR[] = [],
  valueRefs: readonly ValueRefHIR[] = [],
  customPropertyDecls: readonly CustomPropertyDeclHIR[] = [],
  customPropertyRefs: readonly CustomPropertyRefHIR[] = [],
  sassSymbols: readonly SassSymbolOccurrenceHIR[] = [],
  sassSymbolDecls: readonly SassSymbolDeclHIR[] = [],
  sassModuleUses: readonly SassModuleUseHIR[] = [],
  sassModuleMemberRefs: readonly SassModuleMemberRefHIR[] = [],
  sassModuleForwards: readonly SassModuleForwardHIR[] = [],
): StyleDocumentHIR {
  return {
    kind: "style",
    filePath,
    selectors,
    keyframes,
    animationNameRefs,
    valueDecls,
    valueImports,
    valueRefs,
    customPropertyDecls,
    customPropertyRefs,
    sassSymbols,
    sassSymbolDecls,
    sassModuleUses,
    sassModuleForwards,
    sassModuleMemberRefs,
  };
}
