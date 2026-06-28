import type { Range } from "@omena/shared";
import type ts from "../../ts-facade";
import { positionOfLineChar } from "../../ts-facade";
import { buildFlowBlockGraphSnapshot, type FlowBlockGraphSnapshot } from "../flow/cfg";
import { buildFlowSlice } from "../flow/flow-slice";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type {
  ClassExpressionHIR,
  DomainClassReferenceHIR,
  SourceDocumentHIR,
  StyleAccessClassExpressionHIR,
  StyleImportBindingHIR,
  SymbolRefClassExpressionHIR,
  TemplateClassExpressionHIR,
  UtilityBindingHIR,
} from "../hir/source-types";
import type { StyleDocumentHIR } from "../hir/style-types";
import { readSourceExpressionResolution } from "../query/read-source-expression-resolution";
import type { TypeResolver } from "../ts/type-resolver";

export interface CanonicalByteSpanV0 {
  readonly start: number;
  readonly end: number;
}

export interface CanonicalSourceFrontendCaptureV0 {
  readonly schemaVersion: 0;
  readonly product: "omena.source-frontend-canonical-capture";
  readonly sourcePath: string;
  readonly syntax: CanonicalSourceSyntaxCaptureV0;
  readonly bindingGraph: CanonicalSourceBindingGraphCaptureV0;
  readonly cfgSnapshot: CanonicalSourceCfgCaptureV0 | null;
}

export interface CanonicalSourceSyntaxCaptureV0 {
  readonly importedStyleBindings: readonly CanonicalImportedStyleBindingV0[];
  readonly utilityBindings: readonly UtilityBindingHIR[];
  readonly selectorReferences: readonly CanonicalSelectorReferenceV0[];
  readonly symbolReferences: readonly CanonicalSymbolReferenceV0[];
  readonly symbolSelectorReferences: readonly CanonicalSelectorReferenceV0[];
  readonly stylePropertyAccesses: readonly CanonicalStylePropertyAccessV0[];
  readonly domainClassReferences: readonly DomainClassReferenceHIR[];
}

export interface CanonicalImportedStyleBindingV0 {
  readonly binding: string;
  readonly styleUri: string;
}

export interface CanonicalSelectorReferenceV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly selectorName: string | null;
  readonly matchKind: "exact" | "prefix";
  readonly targetStyleUri: string | null;
}

export interface CanonicalStylePropertyAccessV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly selectorName: string;
  readonly targetStyleUri: string | null;
}

export interface CanonicalSymbolReferenceV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly rawReference: string;
  readonly rootName: string;
  readonly targetStyleUri: string | null;
}

export interface CanonicalSourceBindingGraphCaptureV0 {
  readonly filePath: string;
  readonly nodes: SourceBindingGraph["nodes"];
  readonly edges: SourceBindingGraph["edges"];
  readonly expressionTargetsModules: readonly CanonicalExpressionTargetsModuleV0[];
  readonly styleAccessUsesStyleImports: readonly CanonicalStyleAccessUsesStyleImportV0[];
  readonly symbolRefUsesDecls: readonly CanonicalSymbolRefUsesDeclV0[];
}

export interface CanonicalExpressionTargetsModuleV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly targetStyleUri: string;
}

export interface CanonicalStyleAccessUsesStyleImportV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly declName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

export interface CanonicalSymbolRefUsesDeclV0 {
  readonly byteSpan: CanonicalByteSpanV0;
  readonly rawReference: string;
  readonly rootName: string;
  readonly declName: string;
  readonly styleUri: string;
}

export interface CanonicalSourceCfgCaptureV0 {
  readonly variableName: string;
  readonly referenceByteOffset: number;
  readonly snapshot: FlowBlockGraphSnapshot;
}

export interface CaptureTsSourceFrontendFactsArgsV0 {
  readonly sourceFile: ts.SourceFile;
  readonly sourceBinder: SourceBinderResult;
  readonly sourceDocument: SourceDocumentHIR;
  readonly sourceBindingGraph: SourceBindingGraph;
  readonly semantic?: {
    readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
    readonly typeResolver: TypeResolver;
    readonly filePath: string;
    readonly workspaceRoot: string;
    readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
  };
  readonly cfg?: {
    readonly variableName: string;
    readonly referenceRange: Range;
  };
}

export function captureTsSourceFrontendFactsV0(
  args: CaptureTsSourceFrontendFactsArgsV0,
): CanonicalSourceFrontendCaptureV0 {
  return {
    schemaVersion: 0,
    product: "omena.source-frontend-canonical-capture",
    sourcePath: args.sourceFile.fileName,
    syntax: canonicalSourceSyntaxCapture(args),
    bindingGraph: {
      filePath: args.sourceBindingGraph.filePath,
      nodes: args.sourceBindingGraph.nodes,
      edges: args.sourceBindingGraph.edges,
      expressionTargetsModules: canonicalExpressionTargetsModules(
        args.sourceFile,
        args.sourceBindingGraph,
      ),
      styleAccessUsesStyleImports: canonicalStyleAccessUsesStyleImports(
        args.sourceFile,
        args.sourceBindingGraph,
      ),
      symbolRefUsesDecls: canonicalSymbolRefUsesDecls(args.sourceFile, args.sourceBindingGraph),
    },
    cfgSnapshot: args.cfg ? canonicalCfgCapture(args.sourceFile, args.cfg) : null,
  };
}

export function stringifyCanonicalSourceFrontendJsonV0(value: unknown): string {
  return JSON.stringify(sortJsonValue(value));
}

function canonicalSourceSyntaxCapture(
  args: CaptureTsSourceFrontendFactsArgsV0,
): CanonicalSourceSyntaxCaptureV0 {
  return {
    importedStyleBindings: args.sourceDocument.styleImports
      .map(canonicalImportedStyleBinding)
      .toSorted(compareByStableJson),
    utilityBindings: [...args.sourceDocument.utilityBindings].toSorted(compareByStableJson),
    selectorReferences: args.sourceDocument.classExpressions
      .flatMap((expression) => canonicalSelectorReferences(args.sourceFile, expression))
      .toSorted(compareByStableJson),
    symbolReferences: args.sourceDocument.classExpressions
      .flatMap((expression) => canonicalSymbolReference(args.sourceFile, expression))
      .toSorted(compareByStableJson),
    symbolSelectorReferences: args.sourceDocument.classExpressions
      .flatMap((expression) => canonicalSymbolSelectorReferences(args, expression))
      .toSorted(compareByStableJson),
    stylePropertyAccesses: args.sourceDocument.classExpressions
      .flatMap((expression) => canonicalStylePropertyAccess(args.sourceFile, expression))
      .toSorted(compareByStableJson),
    domainClassReferences: [...args.sourceDocument.domainClassReferences].toSorted(
      compareByStableJson,
    ),
  };
}

function canonicalImportedStyleBinding(
  styleImport: StyleImportBindingHIR,
): CanonicalImportedStyleBindingV0 {
  return {
    binding: styleImport.localName,
    styleUri:
      styleImport.resolved.kind === "resolved"
        ? fileUriForAbsolutePath(styleImport.resolved.absolutePath)
        : `missing:${styleImport.resolved.specifier}`,
  };
}

function canonicalSelectorReferences(
  sourceFile: ts.SourceFile,
  expression: ClassExpressionHIR,
): readonly CanonicalSelectorReferenceV0[] {
  switch (expression.kind) {
    case "literal":
      return [
        {
          byteSpan: rangeToUtf8ByteSpan(sourceFile, expression.range),
          selectorName: expression.className,
          matchKind: "exact",
          targetStyleUri: fileUriForAbsolutePath(expression.scssModulePath),
        },
      ];
    case "template":
      return expression.staticPrefix.length === 0
        ? []
        : [
            {
              byteSpan: templatePrefixByteSpan(sourceFile, expression),
              selectorName: expression.staticPrefix,
              matchKind: "prefix",
              targetStyleUri: fileUriForAbsolutePath(expression.scssModulePath),
            },
          ];
    case "styleAccess":
    case "symbolRef":
      return [];
    default:
      expression satisfies never;
      return [];
  }
}

function canonicalSymbolReference(
  sourceFile: ts.SourceFile,
  expression: ClassExpressionHIR,
): readonly CanonicalSymbolReferenceV0[] {
  if (expression.kind !== "symbolRef") return [];
  const symbolRef = expression as SymbolRefClassExpressionHIR;
  return [
    {
      byteSpan: rangeToUtf8ByteSpan(sourceFile, symbolRef.range),
      rawReference: symbolRef.rawReference,
      rootName: symbolRef.rootName,
      targetStyleUri: fileUriForAbsolutePath(symbolRef.scssModulePath),
    },
  ];
}

function canonicalStylePropertyAccess(
  sourceFile: ts.SourceFile,
  expression: ClassExpressionHIR,
): readonly CanonicalStylePropertyAccessV0[] {
  if (expression.kind !== "styleAccess") return [];
  return [
    {
      byteSpan: rangeToUtf8ByteSpan(sourceFile, expression.range),
      selectorName: (expression as StyleAccessClassExpressionHIR).className,
      targetStyleUri: fileUriForAbsolutePath(expression.scssModulePath),
    },
  ];
}

function canonicalSymbolSelectorReferences(
  args: CaptureTsSourceFrontendFactsArgsV0,
  expression: ClassExpressionHIR,
): readonly CanonicalSelectorReferenceV0[] {
  if (!args.semantic || expression.kind !== "symbolRef") return [];
  const resolution = readSourceExpressionResolution(
    {
      expression,
      sourceFile: args.sourceFile,
    },
    {
      styleDocumentForPath: args.semantic.styleDocumentForPath,
      typeResolver: args.semantic.typeResolver,
      filePath: args.semantic.filePath,
      workspaceRoot: args.semantic.workspaceRoot,
      sourceBinder: args.sourceBinder,
      sourceBindingGraph: args.sourceBindingGraph,
      ...(args.semantic.classValueUniverses
        ? { classValueUniverses: args.semantic.classValueUniverses }
        : {}),
    },
  );
  const styleDocument = resolution.styleDocument;
  if (!styleDocument) return [];
  return resolution.selectors.map((selector) => ({
    byteSpan: rangeToUtf8ByteSpan(args.sourceFile, expression.range),
    selectorName: selector.name,
    matchKind: "exact",
    targetStyleUri: fileUriForAbsolutePath(styleDocument.filePath),
  }));
}

function canonicalExpressionTargetsModules(
  sourceFile: ts.SourceFile,
  graph: SourceBindingGraph,
): readonly CanonicalExpressionTargetsModuleV0[] {
  const nodes = new Map(graph.nodes.map((node) => [node.id, node]));
  return graph.edges
    .flatMap((edge) => {
      if (edge.kind !== "expressionTargetsModule") return [];
      const expressionNode = nodes.get(edge.from);
      const styleModuleNode = nodes.get(edge.to);
      if (expressionNode?.kind !== "expression" || styleModuleNode?.kind !== "styleModule") {
        return [];
      }
      const byteSpan = canonicalClassExpressionByteSpan(sourceFile, expressionNode.expression);
      if (!byteSpan) return [];
      return [
        {
          byteSpan,
          targetStyleUri: fileUriForAbsolutePath(styleModuleNode.scssModulePath),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function canonicalStyleAccessUsesStyleImports(
  sourceFile: ts.SourceFile,
  graph: SourceBindingGraph,
): readonly CanonicalStyleAccessUsesStyleImportV0[] {
  const nodes = new Map(graph.nodes.map((node) => [node.id, node]));
  return graph.edges
    .flatMap((edge) => {
      if (edge.kind !== "expressionUsesDecl") return [];
      const expressionNode = nodes.get(edge.from);
      const declNode = nodes.get(edge.to);
      if (
        expressionNode?.kind !== "expression" ||
        expressionNode.expression.kind !== "styleAccess" ||
        declNode?.kind !== "decl"
      ) {
        return [];
      }
      const expression = expressionNode.expression as StyleAccessClassExpressionHIR;
      return [
        {
          byteSpan: rangeToUtf8ByteSpan(sourceFile, expression.range),
          declName: declNode.decl.name,
          stylesLocalName: declNode.decl.name,
          styleUri: fileUriForAbsolutePath(expression.scssModulePath),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function canonicalSymbolRefUsesDecls(
  sourceFile: ts.SourceFile,
  graph: SourceBindingGraph,
): readonly CanonicalSymbolRefUsesDeclV0[] {
  const nodes = new Map(graph.nodes.map((node) => [node.id, node]));
  return graph.edges
    .flatMap((edge) => {
      if (edge.kind !== "expressionUsesDecl") return [];
      const expressionNode = nodes.get(edge.from);
      const declNode = nodes.get(edge.to);
      if (
        expressionNode?.kind !== "expression" ||
        expressionNode.expression.kind !== "symbolRef" ||
        declNode?.kind !== "decl"
      ) {
        return [];
      }
      const expression = expressionNode.expression as SymbolRefClassExpressionHIR;
      return [
        {
          byteSpan: rangeToUtf8ByteSpan(sourceFile, expression.range),
          rawReference: expression.rawReference,
          rootName: expression.rootName,
          declName: declNode.decl.name,
          styleUri: fileUriForAbsolutePath(expression.scssModulePath),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function canonicalClassExpressionByteSpan(
  sourceFile: ts.SourceFile,
  expression: ClassExpressionHIR,
): CanonicalByteSpanV0 | null {
  if (expression.kind === "template" && expression.staticPrefix.length === 0) {
    return null;
  }
  return expression.kind === "template"
    ? templatePrefixByteSpan(sourceFile, expression)
    : rangeToUtf8ByteSpan(sourceFile, expression.range);
}

function templatePrefixByteSpan(
  sourceFile: ts.SourceFile,
  expression: TemplateClassExpressionHIR,
): CanonicalByteSpanV0 {
  const start = positionOfLineChar(sourceFile, expression.range.start);
  const end = positionOfLineChar(sourceFile, expression.range.end);
  const sourceText = sourceFile.text.slice(start, end);
  if (sourceText.startsWith("`") && expression.rawTemplate.startsWith("`")) {
    const prefixStart = start + 1;
    return {
      start: utf8ByteOffsetAtPosition(sourceFile.text, prefixStart),
      end: utf8ByteOffsetAtPosition(sourceFile.text, prefixStart + expression.staticPrefix.length),
    };
  }
  return rangeToUtf8ByteSpan(sourceFile, expression.range);
}

function canonicalCfgCapture(
  sourceFile: ts.SourceFile,
  cfg: NonNullable<CaptureTsSourceFrontendFactsArgsV0["cfg"]>,
): CanonicalSourceCfgCaptureV0 | null {
  const slice = buildFlowSlice(sourceFile, cfg.referenceRange, cfg.variableName);
  if (!slice) return null;
  return {
    variableName: cfg.variableName,
    referenceByteOffset: utf8ByteOffsetAtPosition(sourceFile.text, slice.referencePos),
    snapshot: buildFlowBlockGraphSnapshot(slice.nodes),
  };
}

function rangeToUtf8ByteSpan(sourceFile: ts.SourceFile, range: Range): CanonicalByteSpanV0 {
  const start = positionOfLineChar(sourceFile, range.start);
  const end = positionOfLineChar(sourceFile, range.end);
  return {
    start: utf8ByteOffsetAtPosition(sourceFile.text, start),
    end: utf8ByteOffsetAtPosition(sourceFile.text, end),
  };
}

function utf8ByteOffsetAtPosition(text: string, position: number): number {
  return Buffer.byteLength(text.slice(0, position), "utf8");
}

function fileUriForAbsolutePath(absolutePath: string): string {
  return absolutePath.startsWith("file://") ? absolutePath : `file://${absolutePath}`;
}

function compareByStableJson(left: unknown, right: unknown): number {
  return stringifyCanonicalSourceFrontendJsonV0(left).localeCompare(
    stringifyCanonicalSourceFrontendJsonV0(right),
  );
}

function sortJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortJsonValue);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .toSorted(([left], [right]) => left.localeCompare(right))
        .map(([key, nested]) => [key, sortJsonValue(nested)]),
    );
  }
  return value;
}
