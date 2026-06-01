import type { ResolvedType } from "@omena/shared";
import type {
  SourceBindingGraph,
  SourceBindingGraphDeclNode,
  SourceBindingGraphExpressionNode,
} from "../core/binder/source-binding-graph";
import type {
  SourceBindingGraphSnapshotV1,
  StringTypeFactsV1,
  TypeFactTableEntryV1,
} from "./engine-v1";

export function buildSourceBindingGraphSnapshotV1(
  graph: SourceBindingGraph,
): SourceBindingGraphSnapshotV1 {
  const declNodes = new Map<string, SourceBindingGraphDeclNode>();
  const expressionNodes = new Map<string, SourceBindingGraphExpressionNode>();

  for (const node of graph.nodes) {
    if (node.kind === "decl") {
      declNodes.set(node.id, node);
      continue;
    }
    if (node.kind === "expression") {
      expressionNodes.set(node.id, node);
    }
  }

  return {
    declarations: [...declNodes.values()]
      .map(({ decl }) => ({
        id: decl.id,
        name: decl.name,
        kind: decl.kind,
      }))
      .toSorted(
        (a, b) =>
          a.id.localeCompare(b.id) || a.name.localeCompare(b.name) || a.kind.localeCompare(b.kind),
      ),
    resolutions: graph.edges
      .flatMap((edge) => {
        if (edge.kind !== "expressionUsesDecl") return [];
        const expression = expressionNodes.get(edge.from);
        const declaration = declNodes.get(edge.to);
        if (!expression || !declaration) return [];
        return [
          {
            expressionId: expression.expression.id,
            declarationId: declaration.decl.id,
          },
        ];
      })
      .toSorted(
        (a, b) =>
          a.expressionId.localeCompare(b.expressionId) ||
          (a.declarationId ?? "").localeCompare(b.declarationId ?? ""),
      ),
  };
}

export function normalizeResolvedTypeToTypeFactsV1(resolvedType: ResolvedType): StringTypeFactsV1 {
  if (resolvedType.kind === "unresolvable") {
    return { kind: "unknown" };
  }

  if (resolvedType.values.length <= 1) {
    return { kind: "exact", values: [...resolvedType.values] };
  }

  return {
    kind: "finiteSet",
    values: [...new Set(resolvedType.values)].toSorted(),
  };
}

export function createTypeFactTableEntryV1(
  filePath: string,
  expressionId: string,
  resolvedType: ResolvedType,
): TypeFactTableEntryV1 {
  return {
    filePath,
    expressionId,
    facts: normalizeResolvedTypeToTypeFactsV1(resolvedType),
  };
}
