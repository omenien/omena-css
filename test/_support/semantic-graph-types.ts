import type { ClassRefOrigin, Range, StyleImport } from "@omena/shared";
import type { AbstractClassValue } from "../../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import type { SourceExpressionKind } from "../../server/engine-core-ts/src/core/hir/source-types";
import type { EdgeCertainty } from "../../server/engine-core-ts/src/core/semantic/certainty";
import type { EdgeReason } from "../../server/engine-core-ts/src/core/semantic/provenance";

export type SemanticNode =
  | DocumentNode
  | StyleImportNode
  | UtilityBindingNode
  | RefNode
  | SelectorNode
  | SelectorViewNode;

export type SemanticNodeKind =
  | "document"
  | "styleImport"
  | "utilityBinding"
  | "ref"
  | "selector"
  | "selectorView";

interface SemanticNodeBase {
  readonly id: string;
  readonly kind: SemanticNodeKind;
}

export interface DocumentNode extends SemanticNodeBase {
  readonly kind: "document";
  readonly documentKind: "source" | "style";
  readonly filePath: string;
}

export interface StyleImportNode extends SemanticNodeBase {
  readonly kind: "styleImport";
  readonly filePath: string;
  readonly localName: string;
  readonly resolved: StyleImport;
  readonly range?: Range;
}

export interface UtilityBindingNode extends SemanticNodeBase {
  readonly kind: "utilityBinding";
  readonly filePath: string;
  readonly bindingKind: "classnamesBind" | "classUtil";
  readonly localName: string;
  readonly scssModulePath?: string;
  readonly stylesLocalName?: string;
  readonly classNamesImportName?: string;
  readonly bindingDeclId?: string;
}

export interface RefNode extends SemanticNodeBase {
  readonly kind: "ref";
  readonly filePath: string;
  readonly expressionKind: SourceExpressionKind;
  readonly origin: ClassRefOrigin;
  readonly scssModulePath: string;
  readonly range: Range;
  readonly className?: string;
  readonly rawTemplate?: string;
  readonly staticPrefix?: string;
  readonly rawReference?: string;
  readonly rootName?: string;
  readonly rootBindingDeclId?: string;
  readonly pathSegments?: readonly string[];
  readonly accessPath?: readonly string[];
}

export interface SelectorNode extends SemanticNodeBase {
  readonly kind: "selector";
  readonly filePath: string;
  readonly canonicalName: string;
}

export interface SelectorViewNode extends SemanticNodeBase {
  readonly kind: "selectorView";
  readonly filePath: string;
  readonly name: string;
  readonly canonicalName: string;
  readonly viewKind: "canonical" | "alias";
  readonly nestedSafety: "flat" | "bemSuffixSafe" | "nestedUnsafe";
  readonly range: Range;
  readonly ruleRange: Range;
  readonly fullSelector: string;
  readonly originalName?: string;
}

export interface SemanticEdge {
  readonly id: string;
  readonly from: string;
  readonly to: string;
  readonly reason: EdgeReason;
  readonly certainty: EdgeCertainty;
  readonly abstractValue?: AbstractClassValue;
}

export interface SemanticGraph {
  readonly nodes: readonly SemanticNode[];
  readonly edges: readonly SemanticEdge[];
}
