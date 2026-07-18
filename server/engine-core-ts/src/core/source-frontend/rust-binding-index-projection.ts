import { fileURLToPath, pathToFileURL } from "node:url";
import type {
  SourceBindingGraph,
  SourceBindingGraphEdge,
  SourceBindingGraphNode,
} from "../binder/source-binding-graph";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type { BinderDecl, BinderScope, SourceBinderResult, TextSpan } from "../binder/scope-types";
import { finiteClassValueUniverseV0 } from "../abstract-value/class-value-universe";
import {
  makeClassUtilBinding,
  makeDomainLiteralClassReference,
  makeDomainTemplateClassReference,
  makeLiteralClassExpression,
  makeSourceDocumentHIR,
  makeStyleAccessClassExpression,
  makeStyleImportBinding,
  makeSymbolRefClassExpression,
  makeTemplateClassExpression,
  type ClassExpressionHIR,
  type DomainClassReferenceHIR,
  type SourceDocumentHIR,
  type UtilityBindingHIR,
} from "../hir/source-types";
import type { SourceLanguage } from "../hir/shared-types";

export interface RustSourceFrontendByteSpanV0 {
  readonly start: number;
  readonly end: number;
}

export interface RustSourceBindingIndexV0 {
  readonly bindingScopes: readonly RustSourceBindingScopeFactV0[];
  readonly scopeParentEdges: readonly RustSourceScopeParentFactV0[];
  readonly bindingDecls: readonly RustSourceBindingDeclFactV0[];
  readonly scopeContainsDecls: readonly RustSourceScopeContainsDeclFactV0[];
  readonly styleImportBindings: readonly RustSourceBindingStyleImportFactV0[];
  readonly declaresStyleImports: readonly RustSourceDeclaresStyleImportFactV0[];
  readonly styleImportResolvesModules: readonly RustSourceStyleImportResolvesModuleFactV0[];
  readonly classExpressionNodes: readonly RustSourceClassExpressionNodeFactV0[];
  readonly expressionTargetsModules: readonly RustSourceExpressionTargetsModuleFactV0[];
  readonly classnamesBindUtilityBindings: readonly RustSourceClassnamesBindUtilityBindingFactV0[];
  readonly classUtilBindings: readonly RustSourceClassUtilityBindingFactV0[];
  readonly declaresUtilityBindings: readonly RustSourceDeclaresUtilityBindingFactV0[];
  readonly utilityUsesStyleImports: readonly RustSourceUtilityUsesStyleImportFactV0[];
  readonly styleAccessUsesStyleImports: readonly RustSourceStyleAccessUsesStyleImportFactV0[];
  readonly symbolRefUsesDecls: readonly RustSourceSymbolRefUsesDeclFactV0[];
  readonly moduleSpecifiers?: readonly RustSourceModuleSpecifierFactV0[];
}

export interface RustSourceSyntaxIndexV0 {
  readonly classValueUniverses?: readonly RustSourceClassValueUniverseEntryFactV0[];
  readonly domainClassReferences?: readonly RustSourceDomainClassReferenceFactV0[];
}

export interface RustSourceBindingScopeFactV0 {
  readonly kind: BinderScope["kind"];
  readonly byteSpan: RustSourceFrontendByteSpanV0;
}

export interface RustSourceScopeParentFactV0 {
  readonly childKind: BinderScope["kind"];
  readonly childByteSpan: RustSourceFrontendByteSpanV0;
  readonly parentKind: BinderScope["kind"];
  readonly parentByteSpan: RustSourceFrontendByteSpanV0;
}

export interface RustSourceBindingDeclFactV0 {
  readonly kind: BinderDecl["kind"];
  readonly name: string;
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly importPath?: string;
}

export interface RustSourceScopeContainsDeclFactV0 {
  readonly scopeKind: BinderScope["kind"];
  readonly scopeByteSpan: RustSourceFrontendByteSpanV0;
  readonly declKind: BinderDecl["kind"];
  readonly declName: string;
  readonly declByteSpan: RustSourceFrontendByteSpanV0;
  readonly importPath?: string;
}

export interface RustSourceBindingStyleImportFactV0 {
  readonly localName: string;
  readonly styleUri: string;
}

export interface RustSourceDeclaresStyleImportFactV0 {
  readonly declName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

export interface RustSourceStyleImportResolvesModuleFactV0 {
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

export interface RustSourceClassExpressionNodeFactV0 {
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly kind: ClassExpressionHIR["kind"];
  readonly targetStyleUri: string;
}

export interface RustSourceExpressionTargetsModuleFactV0 {
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly targetStyleUri: string;
}

export interface RustSourceClassnamesBindUtilityBindingFactV0 {
  readonly localName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
  readonly classnamesImportName: string;
}

export interface RustSourceClassUtilityBindingFactV0 {
  readonly localName: string;
}

export interface RustSourceDeclaresUtilityBindingFactV0 {
  readonly declName: string;
  readonly utilityLocalName: string;
  readonly utilityKind: UtilityBindingHIR["kind"];
}

export interface RustSourceUtilityUsesStyleImportFactV0 {
  readonly utilityLocalName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

export interface RustSourceStyleAccessUsesStyleImportFactV0 {
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly declName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

export interface RustSourceSymbolRefUsesDeclFactV0 {
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly rawReference: string;
  readonly rootName: string;
  readonly declName: string;
  readonly styleUri: string;
}

export interface RustSourceModuleSpecifierFactV0 {
  readonly kind: "import" | "export" | "importEquals";
  readonly specifier: string;
  readonly byteSpan: RustSourceFrontendByteSpanV0;
}

export interface RustSourceClassValueUniverseEntryFactV0 {
  readonly pluginId: string;
  readonly domain: string;
  readonly ownerName: string;
  readonly classNames: readonly string[];
  readonly axes: readonly RustSourceClassValueUniverseAxisFactV0[];
  readonly byteSpan: RustSourceFrontendByteSpanV0;
}

export interface RustSourceClassValueUniverseAxisFactV0 {
  readonly axisName: string;
  readonly values: readonly string[];
}

export interface RustSourceDomainClassReferenceFactV0 {
  readonly byteSpan: RustSourceFrontendByteSpanV0;
  readonly pluginId: string;
  readonly domain: string;
  readonly ownerName: string;
  readonly axisName: string;
  readonly optionName?: string | null;
  readonly prefix?: string | null;
}

export interface ProjectRustSourceBindingIndexArgsV0 {
  readonly filePath: string;
  readonly source: string;
  readonly language: SourceLanguage;
  readonly index: RustSourceBindingIndexV0;
}

export interface ProjectedRustSourceBindingIndexV0 {
  readonly sourceBinder: SourceBinderResult;
  readonly sourceDocument: SourceDocumentHIR;
  readonly sourceBindingGraph: SourceBindingGraph;
  readonly sourceModuleSpecifiers: readonly string[];
}

export interface ProjectedRustSourceSyntaxExtrasV0 {
  readonly classValueUniverses: readonly ClassValueUniverseEntryV0[];
  readonly domainClassReferences: readonly DomainClassReferenceHIR[];
}

export function projectRustSourceBindingIndexV0(
  args: ProjectRustSourceBindingIndexArgsV0,
): ProjectedRustSourceBindingIndexV0 {
  const context = createProjectionContext(args.filePath, args.source);
  const scopes = args.index.bindingScopes.map((scope) => scopeFromFact(context, scope, args.index));
  const decls = args.index.bindingDecls.map((decl) => declFromFact(context, decl, args.index));
  const sourceBinder: SourceBinderResult = {
    filePath: args.filePath,
    scopes: scopes.toSorted(compareScopes),
    decls: decls.toSorted(compareDecls),
  };
  const sourceDocument = sourceDocumentFromIndex(args, context);
  const sourceBindingGraph = sourceBindingGraphFromIndex(args, sourceBinder, sourceDocument);
  return {
    sourceBinder,
    sourceDocument,
    sourceBindingGraph,
    sourceModuleSpecifiers: moduleSpecifiersFromIndex(args.index),
  };
}

export function projectRustSourceSyntaxExtrasV0(args: {
  readonly filePath: string;
  readonly source: string;
  readonly index: RustSourceSyntaxIndexV0;
}): ProjectedRustSourceSyntaxExtrasV0 {
  const context = createProjectionContext(args.filePath, args.source);
  return {
    classValueUniverses: (args.index.classValueUniverses ?? [])
      .map((universe) => classValueUniverseFromFact(context, universe))
      .toSorted((a, b) => a.id.localeCompare(b.id)),
    domainClassReferences: (args.index.domainClassReferences ?? [])
      .map((reference, index) => domainClassReferenceFromFact(context, reference, index))
      .toSorted((a, b) => a.id.localeCompare(b.id)),
  };
}

function sourceDocumentFromIndex(
  args: ProjectRustSourceBindingIndexArgsV0,
  context: ProjectionContext,
): SourceDocumentHIR {
  const styleImports = args.index.styleImportBindings.map((binding) => {
    const stylePath = styleUriToPath(binding.styleUri);
    const bindingDeclId =
      declIdForNameAndKind(args.index, binding.localName, "import", context) ??
      syntheticDeclId("import", binding.localName, byteSpanKey({ start: 0, end: 0 }));
    return makeStyleImportBinding(
      styleImportNodeId(binding.localName, binding.styleUri),
      binding.localName,
      bindingDeclId,
      { kind: "resolved", absolutePath: stylePath },
    );
  });
  const classnamesBindings = args.index.classnamesBindUtilityBindings.map((binding) => {
    const bindingDeclId =
      declIdForNameAndKind(args.index, binding.localName, "localVar", context) ??
      syntheticDeclId("localVar", binding.localName, byteSpanKey({ start: 0, end: 0 }));
    return {
      kind: "classnamesBind" as const,
      id: classnamesUtilityNodeId(binding.localName, binding.stylesLocalName, binding.styleUri),
      localName: binding.localName,
      stylesLocalName: binding.stylesLocalName,
      scssModulePath: styleUriToPath(binding.styleUri),
      classNamesImportName: binding.classnamesImportName,
      bindingDeclId,
    };
  });
  const classUtilBindings = args.index.classUtilBindings.map((binding) =>
    makeClassUtilBinding(
      classUtilNodeId(binding.localName),
      binding.localName,
      declIdForNameAndKind(args.index, binding.localName, "import", context) ??
        syntheticDeclId("import", binding.localName, byteSpanKey({ start: 0, end: 0 })),
    ),
  );
  return makeSourceDocumentHIR({
    filePath: args.filePath,
    language: args.language,
    styleImports,
    utilityBindings: [...classnamesBindings, ...classUtilBindings].toSorted(compareById),
    classExpressions: args.index.classExpressionNodes
      .map((expression) => classExpressionFromFact(context, expression, args.index))
      .toSorted(compareById),
  });
}

function moduleSpecifiersFromIndex(index: RustSourceBindingIndexV0): readonly string[] {
  return [...new Set((index.moduleSpecifiers ?? []).map((fact) => fact.specifier))].toSorted();
}

function classValueUniverseFromFact(
  context: ProjectionContext,
  fact: RustSourceClassValueUniverseEntryFactV0,
): ClassValueUniverseEntryV0 {
  const range = rangeFromByteSpan(fact.byteSpan, context.source);
  return {
    id: `class-value-universe:${fact.pluginId}:${fact.ownerName}:${range.start.line}:${range.start.character}`,
    pluginId: fact.pluginId,
    domain: fact.domain,
    ownerName: fact.ownerName,
    range,
    universe: finiteClassValueUniverseV0(fact.classNames),
  };
}

function domainClassReferenceFromFact(
  context: ProjectionContext,
  fact: RustSourceDomainClassReferenceFactV0,
  index: number,
): DomainClassReferenceHIR {
  const range = rangeFromByteSpan(fact.byteSpan, context.source);
  const id = `domain-class-ref:${fact.pluginId}:${index}`;
  const prefixClassName = variantReferenceKey(fact.ownerName, fact.axisName, fact.prefix ?? "");
  return fact.optionName
    ? makeDomainLiteralClassReference(
        id,
        fact.pluginId,
        fact.domain,
        "classUtilityCall",
        variantReferenceKey(fact.ownerName, fact.axisName, fact.optionName),
        range,
      )
    : makeDomainTemplateClassReference(
        id,
        fact.pluginId,
        fact.domain,
        "classUtilityCall",
        fact.prefix ?? "",
        prefixClassName,
        range,
      );
}

function variantReferenceKey(ownerName: string, axisName: string, optionName: string): string {
  return `${ownerName}.${axisName}.${optionName}`;
}

function sourceBindingGraphFromIndex(
  args: ProjectRustSourceBindingIndexArgsV0,
  sourceBinder: SourceBinderResult,
  sourceDocument: SourceDocumentHIR,
): SourceBindingGraph {
  const context = createProjectionContext(args.filePath, args.source);
  const nodes = new Map<string, SourceBindingGraphNode>();
  const edges = new Map<string, SourceBindingGraphEdge>();
  const addNode = (node: SourceBindingGraphNode): void => {
    nodes.set(node.id, node);
  };
  const addEdge = (
    from: string | undefined,
    to: string | undefined,
    kind: SourceBindingGraphEdge["kind"],
  ): void => {
    if (!from || !to) return;
    edges.set(`${from}->${to}:${kind}`, { from, to, kind });
  };

  for (const scope of sourceBinder.scopes) {
    addNode({ id: scopeNodeId(scope.id), kind: "scope", filePath: args.filePath, scope });
  }
  for (const decl of sourceBinder.decls) {
    addNode({ id: declNodeId(decl.id), kind: "decl", filePath: args.filePath, decl });
  }
  for (const styleImport of sourceDocument.styleImports) {
    addNode({
      id: styleImportNodeId(styleImport.localName, styleImportUri(styleImport.resolved)),
      kind: "styleImport",
      filePath: args.filePath,
      styleImport,
    });
  }
  for (const utilityBinding of sourceDocument.utilityBindings) {
    addNode({
      id: utilityNodeId(utilityBinding),
      kind: "utilityBinding",
      filePath: args.filePath,
      utilityBinding,
    });
  }
  for (const expression of sourceDocument.classExpressions) {
    addNode({
      id: expressionNodeId(expression.kind, expression.range, expression.scssModulePath),
      kind: "expression",
      filePath: args.filePath,
      expression,
    });
  }

  const styleModuleUris = new Set([
    ...args.index.styleImportResolvesModules.map((edge) => edge.styleUri),
    ...args.index.expressionTargetsModules.map((edge) => edge.targetStyleUri),
  ]);
  for (const styleUri of styleModuleUris) {
    addNode({
      id: styleModuleNodeId(styleUri),
      kind: "styleModule",
      filePath: args.filePath,
      scssModulePath: styleUriToPath(styleUri),
    });
  }

  for (const edge of args.index.scopeParentEdges) {
    addEdge(
      scopeNodeId(scopeId(edge.childKind, spanFromByteSpan(edge.childByteSpan, context))),
      scopeNodeId(scopeId(edge.parentKind, spanFromByteSpan(edge.parentByteSpan, context))),
      "scopeParent",
    );
  }
  for (const edge of args.index.scopeContainsDecls) {
    addEdge(
      scopeNodeId(scopeId(edge.scopeKind, spanFromByteSpan(edge.scopeByteSpan, context))),
      declNodeId(
        declId(
          edge.declKind,
          edge.declName,
          spanFromByteSpan(edge.declByteSpan, context),
          edge.importPath,
        ),
      ),
      "scopeContainsDecl",
    );
  }
  for (const edge of args.index.declaresStyleImports) {
    addEdge(
      declNodeIdForName(args.index, edge.declName, context),
      styleImportNodeId(edge.stylesLocalName, edge.styleUri),
      "declaresStyleImport",
    );
  }
  for (const edge of args.index.styleImportResolvesModules) {
    addEdge(
      styleImportNodeId(edge.stylesLocalName, edge.styleUri),
      styleModuleNodeId(edge.styleUri),
      "styleImportResolvesModule",
    );
  }
  for (const edge of args.index.declaresUtilityBindings) {
    addEdge(
      declNodeIdForName(args.index, edge.declName, context),
      edge.utilityKind === "classnamesBind"
        ? classnamesUtilityNodeIdForLocalName(args.index, edge.utilityLocalName)
        : classUtilNodeId(edge.utilityLocalName),
      "declaresUtilityBinding",
    );
  }
  for (const edge of args.index.utilityUsesStyleImports) {
    addEdge(
      classnamesUtilityNodeId(edge.utilityLocalName, edge.stylesLocalName, edge.styleUri),
      styleImportNodeId(edge.stylesLocalName, edge.styleUri),
      "utilityUsesStyleImport",
    );
  }
  for (const edge of args.index.expressionTargetsModules) {
    addEdge(
      expressionNodeIdForByteSpan(args.index, edge.byteSpan, edge.targetStyleUri, args.source),
      styleModuleNodeId(edge.targetStyleUri),
      "expressionTargetsModule",
    );
  }
  for (const edge of args.index.styleAccessUsesStyleImports) {
    addEdge(
      expressionNodeIdForByteSpan(args.index, edge.byteSpan, edge.styleUri, args.source),
      declNodeIdForName(args.index, edge.declName, context),
      "expressionUsesDecl",
    );
  }
  for (const edge of args.index.symbolRefUsesDecls) {
    addEdge(
      expressionNodeIdForByteSpan(args.index, edge.byteSpan, edge.styleUri, args.source),
      declNodeIdForName(args.index, edge.declName, context),
      "expressionUsesDecl",
    );
  }

  return {
    filePath: args.filePath,
    nodes: [...nodes.values()].toSorted((a, b) => a.id.localeCompare(b.id)),
    edges: [...edges.values()].toSorted((a, b) =>
      `${a.from}:${a.kind}:${a.to}`.localeCompare(`${b.from}:${b.kind}:${b.to}`),
    ),
  };
}

interface ProjectionContext {
  readonly filePath: string;
  readonly source: string;
}

function createProjectionContext(filePath: string, source: string): ProjectionContext {
  return { filePath, source };
}

function scopeFromFact(
  context: ProjectionContext,
  fact: RustSourceBindingScopeFactV0,
  index: RustSourceBindingIndexV0,
): BinderScope {
  const span = spanFromByteSpan(fact.byteSpan, context);
  const parentScopeId = parentScopeIdForScopeFact(index, fact, context);
  return {
    id: scopeId(fact.kind, span),
    kind: fact.kind,
    filePath: context.filePath,
    span,
    ...(parentScopeId ? { parentScopeId } : {}),
  };
}

function parentScopeIdForScopeFact(
  index: RustSourceBindingIndexV0,
  fact: RustSourceBindingScopeFactV0,
  context: ProjectionContext,
): string | null {
  const parent = index.scopeParentEdges.find(
    (edge) =>
      edge.childKind === fact.kind &&
      byteSpanKey(edge.childByteSpan) === byteSpanKey(fact.byteSpan),
  );
  return parent
    ? scopeId(parent.parentKind, spanFromByteSpan(parent.parentByteSpan, context))
    : null;
}

function declFromFact(
  context: ProjectionContext,
  fact: RustSourceBindingDeclFactV0,
  index: RustSourceBindingIndexV0,
): BinderDecl {
  const span = spanFromByteSpan(fact.byteSpan, context);
  return {
    id: declId(fact.kind, fact.name, span, fact.importPath),
    kind: fact.kind,
    scopeId: scopeIdForDecl(index, fact, context) ?? sourceFileScopeId(index, context),
    name: fact.name,
    filePath: context.filePath,
    span,
    ...(fact.importPath ? { importPath: fact.importPath } : {}),
  };
}

function classExpressionFromFact(
  context: ProjectionContext,
  fact: RustSourceClassExpressionNodeFactV0,
  index: RustSourceBindingIndexV0,
): ClassExpressionHIR {
  const range = rangeFromByteSpan(fact.byteSpan, context.source);
  const targetPath = styleUriToPath(fact.targetStyleUri);
  const text = sourceSlice(context.source, fact.byteSpan);
  switch (fact.kind) {
    case "literal":
      return makeLiteralClassExpression(
        expressionNodeId(fact.kind, range, targetPath),
        "cxCall",
        targetPath,
        text,
        range,
      );
    case "template":
      return makeTemplateClassExpression(
        expressionNodeId(fact.kind, range, targetPath),
        "cxCall",
        targetPath,
        text,
        text,
        range,
      );
    case "styleAccess": {
      const usage = index.styleAccessUsesStyleImports.find(
        (edge) => byteSpanKey(edge.byteSpan) === byteSpanKey(fact.byteSpan),
      );
      return makeStyleAccessClassExpression(
        expressionNodeId(fact.kind, range, targetPath),
        targetPath,
        declNodeIdForName(index, usage?.declName ?? "", context)?.slice("decl:".length) ?? "",
        text,
        [text],
        range,
      );
    }
    case "symbolRef": {
      const usage = index.symbolRefUsesDecls.find(
        (edge) => byteSpanKey(edge.byteSpan) === byteSpanKey(fact.byteSpan),
      );
      const rawReference = usage?.rawReference ?? text;
      const rootName = usage?.rootName ?? rawReference.split(".")[0] ?? rawReference;
      return makeSymbolRefClassExpression(
        expressionNodeId(fact.kind, range, targetPath),
        "cxCall",
        targetPath,
        rawReference,
        rootName,
        rawReference.split(".").slice(1),
        range,
        declNodeIdForName(index, usage?.declName ?? "", context)?.slice("decl:".length),
      );
    }
    default:
      fact.kind satisfies never;
      return makeLiteralClassExpression(
        expressionNodeId("literal", range, targetPath),
        "cxCall",
        targetPath,
        text,
        range,
      );
  }
}

function scopeIdForDecl(
  index: RustSourceBindingIndexV0,
  decl: RustSourceBindingDeclFactV0,
  context: ProjectionContext,
): string | null {
  const match = index.scopeContainsDecls.find(
    (edge) =>
      edge.declKind === decl.kind &&
      edge.declName === decl.name &&
      byteSpanKey(edge.declByteSpan) === byteSpanKey(decl.byteSpan) &&
      edge.importPath === decl.importPath,
  );
  return match ? scopeId(match.scopeKind, spanFromByteSpan(match.scopeByteSpan, context)) : null;
}

function sourceFileScopeId(index: RustSourceBindingIndexV0, context: ProjectionContext): string {
  const sourceFileScope = index.bindingScopes.find((scope) => scope.kind === "sourceFile");
  return sourceFileScope
    ? scopeId(sourceFileScope.kind, spanFromByteSpan(sourceFileScope.byteSpan, context))
    : "scope:sourceFile:0:0";
}

function declIdForNameAndKind(
  index: RustSourceBindingIndexV0,
  name: string,
  kind: BinderDecl["kind"],
  context: ProjectionContext,
): string | null {
  const decl = index.bindingDecls.find(
    (candidate) => candidate.name === name && candidate.kind === kind,
  );
  return decl
    ? declId(kind, decl.name, spanFromByteSpan(decl.byteSpan, context), decl.importPath)
    : null;
}

function declNodeIdForName(
  index: RustSourceBindingIndexV0,
  name: string,
  context: ProjectionContext,
): string | undefined {
  const decl = index.bindingDecls.find((candidate) => candidate.name === name);
  return decl
    ? declNodeId(
        declId(decl.kind, decl.name, spanFromByteSpan(decl.byteSpan, context), decl.importPath),
      )
    : undefined;
}

function classnamesUtilityNodeIdForLocalName(
  index: RustSourceBindingIndexV0,
  localName: string,
): string | undefined {
  const binding = index.classnamesBindUtilityBindings.find(
    (candidate) => candidate.localName === localName,
  );
  return binding
    ? classnamesUtilityNodeId(binding.localName, binding.stylesLocalName, binding.styleUri)
    : undefined;
}

function expressionNodeIdForByteSpan(
  index: RustSourceBindingIndexV0,
  byteSpan: RustSourceFrontendByteSpanV0,
  styleUri: string,
  source: string,
): string | undefined {
  const match = index.classExpressionNodes.find(
    (candidate) =>
      byteSpanKey(candidate.byteSpan) === byteSpanKey(byteSpan) &&
      candidate.targetStyleUri === styleUri,
  );
  if (!match) return undefined;
  return expressionNodeId(
    match.kind,
    rangeFromByteSpan(byteSpan, source),
    styleUriToPath(styleUri),
  );
}

function spanFromByteSpan(
  byteSpan: RustSourceFrontendByteSpanV0,
  context: ProjectionContext,
): TextSpan {
  return {
    start: utf16OffsetAtUtf8ByteOffset(context.source, byteSpan.start),
    end: utf16OffsetAtUtf8ByteOffset(context.source, byteSpan.end),
  };
}

function rangeFromByteSpan(byteSpan: RustSourceFrontendByteSpanV0, source: string) {
  return {
    start: positionAtUtf16Offset(source, utf16OffsetAtUtf8ByteOffset(source, byteSpan.start)),
    end: positionAtUtf16Offset(source, utf16OffsetAtUtf8ByteOffset(source, byteSpan.end)),
  };
}

function sourceSlice(source: string, byteSpan: RustSourceFrontendByteSpanV0): string {
  return source.slice(
    utf16OffsetAtUtf8ByteOffset(source, byteSpan.start),
    utf16OffsetAtUtf8ByteOffset(source, byteSpan.end),
  );
}

function utf16OffsetAtUtf8ByteOffset(source: string, byteOffset: number): number {
  let bytes = 0;
  for (let offset = 0; offset < source.length;) {
    if (bytes >= byteOffset) return offset;
    const codePoint = source.codePointAt(offset) ?? 0;
    const width = codePoint > 0xffff ? 2 : 1;
    const nextBytes = bytes + utf8CodePointByteLength(codePoint);
    if (nextBytes > byteOffset) return offset;
    bytes = nextBytes;
    offset += width;
  }
  return source.length;
}

function utf8CodePointByteLength(codePoint: number): number {
  if (codePoint <= 0x7f) return 1;
  if (codePoint <= 0x7ff) return 2;
  if (codePoint <= 0xffff) return 3;
  return 4;
}

function positionAtUtf16Offset(source: string, offset: number) {
  let line = 0;
  let lineStart = 0;
  for (let index = 0; index < offset; index += 1) {
    if (source.charCodeAt(index) === 10) {
      line += 1;
      lineStart = index + 1;
    }
  }
  return { line, character: offset - lineStart };
}

function styleUriToPath(styleUri: string): string {
  return styleUri.startsWith("file:") ? fileURLToPath(styleUri) : styleUri;
}

function styleImportUri(
  styleImport: SourceDocumentHIR["styleImports"][number]["resolved"],
): string {
  return styleImport.kind === "resolved"
    ? pathToFileURL(styleImport.absolutePath).href
    : pathToFileURL(styleImport.absolutePath).href;
}

function byteSpanKey(span: RustSourceFrontendByteSpanV0): string {
  return `${span.start}:${span.end}`;
}

function scopeId(kind: BinderScope["kind"], span: TextSpan): string {
  return `rust-scope:${kind}:${span.start}:${span.end}`;
}

function declId(
  kind: BinderDecl["kind"],
  name: string,
  span: TextSpan,
  importPath?: string,
): string {
  return `rust-decl:${kind}:${name}:${span.start}:${span.end}:${importPath ?? ""}`;
}

function syntheticDeclId(kind: BinderDecl["kind"], name: string, spanKey: string): string {
  return `rust-decl:${kind}:${name}:${spanKey}:synthetic`;
}

function scopeNodeId(scopeIdValue: string): string {
  return `scope:${scopeIdValue}`;
}

function declNodeId(declIdValue: string): string {
  return `decl:${declIdValue}`;
}

function styleImportNodeId(localName: string, styleUri: string): string {
  return `styleImport:rust-style-import:${localName}:${styleUri}`;
}

function classnamesUtilityNodeId(
  localName: string,
  stylesLocalName: string,
  styleUri: string,
): string {
  return `utilityBinding:rust-utility:classnamesBind:${localName}:${stylesLocalName}:${styleUri}`;
}

function classUtilNodeId(localName: string): string {
  return `utilityBinding:rust-utility:classUtil:${localName}`;
}

function utilityNodeId(binding: UtilityBindingHIR): string {
  return binding.kind === "classnamesBind"
    ? classnamesUtilityNodeId(
        binding.localName,
        binding.stylesLocalName,
        pathToFileURL(binding.scssModulePath).href,
      )
    : classUtilNodeId(binding.localName);
}

function expressionNodeId(
  kind: ClassExpressionHIR["kind"],
  range: ClassExpressionHIR["range"],
  scssModulePath: string,
): string {
  return `expression:rust-expression:${kind}:${range.start.line}:${range.start.character}:${range.end.line}:${range.end.character}:${scssModulePath}`;
}

function styleModuleNodeId(styleUri: string): string {
  return `styleModule:${styleUriToPath(styleUri)}`;
}

function compareScopes(a: BinderScope, b: BinderScope): number {
  return a.id.localeCompare(b.id);
}

function compareDecls(a: BinderDecl, b: BinderDecl): number {
  return a.id.localeCompare(b.id);
}

function compareById(a: { readonly id: string }, b: { readonly id: string }): number {
  return a.id.localeCompare(b.id);
}
