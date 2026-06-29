import type { SourceDocumentHIR } from "../hir/source-types";
import type { BinderDecl, BinderScope, SourceBinderResult } from "./scope-types";

export type SourceBindingGraphNode =
  | SourceBindingGraphScopeNode
  | SourceBindingGraphDeclNode
  | SourceBindingGraphStyleImportNode
  | SourceBindingGraphUtilityBindingNode
  | SourceBindingGraphExpressionNode
  | SourceBindingGraphStyleModuleNode;

export type SourceBindingGraphEdgeKind =
  | "scopeParent"
  | "scopeContainsDecl"
  | "declaresStyleImport"
  | "declaresUtilityBinding"
  | "utilityUsesStyleImport"
  | "expressionUsesDecl"
  | "styleImportResolvesModule"
  | "expressionTargetsModule";

interface SourceBindingGraphNodeBase {
  readonly id: string;
  readonly filePath: string;
}

export interface SourceBindingGraphScopeNode extends SourceBindingGraphNodeBase {
  readonly kind: "scope";
  readonly scope: BinderScope;
}

export interface SourceBindingGraphDeclNode extends SourceBindingGraphNodeBase {
  readonly kind: "decl";
  readonly decl: BinderDecl;
}

export interface SourceBindingGraphStyleImportNode extends SourceBindingGraphNodeBase {
  readonly kind: "styleImport";
  readonly styleImport: SourceDocumentHIR["styleImports"][number];
}

export interface SourceBindingGraphUtilityBindingNode extends SourceBindingGraphNodeBase {
  readonly kind: "utilityBinding";
  readonly utilityBinding: SourceDocumentHIR["utilityBindings"][number];
}

export interface SourceBindingGraphExpressionNode extends SourceBindingGraphNodeBase {
  readonly kind: "expression";
  readonly expression: SourceDocumentHIR["classExpressions"][number];
}

export interface SourceBindingGraphStyleModuleNode extends SourceBindingGraphNodeBase {
  readonly kind: "styleModule";
  readonly scssModulePath: string;
}

export interface SourceBindingGraphEdge {
  readonly from: string;
  readonly to: string;
  readonly kind: SourceBindingGraphEdgeKind;
}

export interface SourceBindingGraph {
  readonly filePath: string;
  readonly nodes: readonly SourceBindingGraphNode[];
  readonly edges: readonly SourceBindingGraphEdge[];
}

export function composeSourceBindingGraph(
  sourceDocument: SourceDocumentHIR,
  sourceBinder: SourceBinderResult,
): SourceBindingGraph {
  const nodes = new Map<string, SourceBindingGraphNode>();
  const edges = new Map<string, SourceBindingGraphEdge>();

  const addNode = (node: SourceBindingGraphNode): void => {
    nodes.set(node.id, node);
  };
  const addEdge = (from: string, to: string, kind: SourceBindingGraphEdgeKind): void => {
    edges.set(`${from}->${to}:${kind}`, { from, to, kind });
  };

  for (const scope of sourceBinder.scopes) {
    addNode({
      id: scopeNodeId(scope.id),
      kind: "scope",
      filePath: sourceDocument.filePath,
      scope,
    });
    if (scope.parentScopeId) {
      addEdge(scopeNodeId(scope.id), scopeNodeId(scope.parentScopeId), "scopeParent");
    }
  }

  for (const decl of sourceBinder.decls) {
    addNode({
      id: declNodeId(decl.id),
      kind: "decl",
      filePath: sourceDocument.filePath,
      decl,
    });
    addEdge(scopeNodeId(decl.scopeId), declNodeId(decl.id), "scopeContainsDecl");
  }

  for (const styleImport of sourceDocument.styleImports) {
    addNode({
      id: styleImportNodeId(styleImport.id),
      kind: "styleImport",
      filePath: sourceDocument.filePath,
      styleImport,
    });
    addEdge(
      declNodeId(styleImport.bindingDeclId),
      styleImportNodeId(styleImport.id),
      "declaresStyleImport",
    );

    if (styleImport.resolved.kind === "resolved") {
      addNode({
        id: styleModuleNodeId(styleImport.resolved.absolutePath),
        kind: "styleModule",
        filePath: sourceDocument.filePath,
        scssModulePath: styleImport.resolved.absolutePath,
      });
      addEdge(
        styleImportNodeId(styleImport.id),
        styleModuleNodeId(styleImport.resolved.absolutePath),
        "styleImportResolvesModule",
      );
    }
  }

  for (const utilityBinding of sourceDocument.utilityBindings) {
    addNode({
      id: utilityBindingNodeId(utilityBinding.id),
      kind: "utilityBinding",
      filePath: sourceDocument.filePath,
      utilityBinding,
    });
    addEdge(
      declNodeId(utilityBinding.bindingDeclId),
      utilityBindingNodeId(utilityBinding.id),
      "declaresUtilityBinding",
    );

    if (utilityBinding.kind === "classnamesBind") {
      const styleImport = sourceDocument.styleImports.find(
        (entry) => entry.localName === utilityBinding.stylesLocalName,
      );
      if (styleImport) {
        addEdge(
          utilityBindingNodeId(utilityBinding.id),
          styleImportNodeId(styleImport.id),
          "utilityUsesStyleImport",
        );
      }
    }
  }

  for (const expression of sourceDocument.classExpressions) {
    addNode({
      id: expressionNodeId(expression.id),
      kind: "expression",
      filePath: sourceDocument.filePath,
      expression,
    });
    addNode({
      id: styleModuleNodeId(expression.scssModulePath),
      kind: "styleModule",
      filePath: sourceDocument.filePath,
      scssModulePath: expression.scssModulePath,
    });
    addEdge(
      expressionNodeId(expression.id),
      styleModuleNodeId(expression.scssModulePath),
      "expressionTargetsModule",
    );

    switch (expression.kind) {
      case "styleAccess":
        addEdge(
          expressionNodeId(expression.id),
          declNodeId(expression.bindingDeclId),
          "expressionUsesDecl",
        );
        break;
      case "symbolRef":
        if (expression.rootBindingDeclId) {
          addEdge(
            expressionNodeId(expression.id),
            declNodeId(expression.rootBindingDeclId),
            "expressionUsesDecl",
          );
        }
        break;
      case "literal":
      case "template":
        break;
      default:
        expression satisfies never;
        break;
    }
  }

  return {
    filePath: sourceDocument.filePath,
    nodes: [...nodes.values()].toSorted((a, b) => a.id.localeCompare(b.id)),
    edges: [...edges.values()].toSorted((a, b) =>
      `${a.from}:${a.kind}:${a.to}`.localeCompare(`${b.from}:${b.kind}:${b.to}`),
    ),
  };
}

export function listStyleModulePaths(graph: SourceBindingGraph): readonly string[] {
  const paths = new Set<string>();
  for (const node of graph.nodes) {
    if (node.kind === "styleModule") {
      paths.add(node.scssModulePath);
    }
  }
  return [...paths].toSorted();
}

export interface SourceBindingGraphResolution {
  readonly refId: string;
  readonly declId: string;
  readonly depth: number;
}

export function resolveBindingAtOffset(
  graph: SourceBindingGraph,
  name: string,
  offset: number,
): SourceBindingGraphResolution | null {
  const scope = findInnermostScopeAtOffset(graph, offset);
  if (!scope) return null;

  let currentScopeId: string | undefined = scope.scope.id;
  let depth = 0;
  while (currentScopeId) {
    const match = findVisibleDeclInScope(graph, currentScopeId, name, offset);
    if (match) {
      return { refId: `offset:${offset}:${name}`, declId: match.decl.id, depth };
    }
    currentScopeId = graph.nodes.find(
      (node): node is SourceBindingGraphScopeNode =>
        node.kind === "scope" && node.scope.id === currentScopeId,
    )?.scope.parentScopeId;
    depth += 1;
  }
  return null;
}

export function getBindingDeclById(graph: SourceBindingGraph, declId: string): BinderDecl | null {
  return (
    graph.nodes.find(
      (node): node is SourceBindingGraphDeclNode => node.kind === "decl" && node.decl.id === declId,
    )?.decl ?? null
  );
}

export function findImportBindingDeclId(
  graph: SourceBindingGraph | undefined,
  localName: string,
  allowedImportPaths?: ReadonlySet<string>,
): string | null {
  if (!graph) return null;
  const match = graph.nodes.find(
    (node): node is SourceBindingGraphDeclNode =>
      node.kind === "decl" &&
      node.decl.kind === "import" &&
      node.decl.name === localName &&
      (!allowedImportPaths ||
        (node.decl.importPath !== undefined && allowedImportPaths.has(node.decl.importPath))),
  );
  return match?.decl.id ?? null;
}

export function findInnermostScopeAtOffset(
  graph: SourceBindingGraph,
  offset: number,
): SourceBindingGraphScopeNode | null {
  let winner: SourceBindingGraphScopeNode | null = null;
  for (const node of graph.nodes) {
    if (node.kind !== "scope") continue;
    if (offset < node.scope.span.start || offset > node.scope.span.end) continue;
    if (!winner) {
      winner = node;
      continue;
    }
    const winnerSize = winner.scope.span.end - winner.scope.span.start;
    const scopeSize = node.scope.span.end - node.scope.span.start;
    if (scopeSize <= winnerSize) {
      winner = node;
    }
  }
  return winner;
}

function findVisibleDeclInScope(
  graph: SourceBindingGraph,
  scopeId: string,
  name: string,
  offset: number,
): SourceBindingGraphDeclNode | null {
  const candidates = graph.nodes.filter(
    (node): node is SourceBindingGraphDeclNode =>
      node.kind === "decl" &&
      node.decl.scopeId === scopeId &&
      node.decl.name === name &&
      node.decl.span.start <= offset,
  );
  if (candidates.length === 0) return null;
  return candidates.reduce((best, current) =>
    current.decl.span.start >= best.decl.span.start ? current : best,
  );
}

function scopeNodeId(scopeId: string): string {
  return `scope:${scopeId}`;
}

function declNodeId(declId: string): string {
  return `decl:${declId}`;
}

function styleImportNodeId(styleImportId: string): string {
  return `styleImport:${styleImportId}`;
}

function utilityBindingNodeId(utilityBindingId: string): string {
  return `utilityBinding:${utilityBindingId}`;
}

function expressionNodeId(expressionId: string): string {
  return `expression:${expressionId}`;
}

function styleModuleNodeId(scssModulePath: string): string {
  return `styleModule:${scssModulePath}`;
}
