import type { ClassExpressionHIR } from "../hir/source-types";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { TypeResolver } from "../ts/type-resolver";
import type { AnalysisEntry } from "../indexing/document-analysis-cache";
import {
  buildDynamicExpressionExplanation,
  type DynamicExpressionExplanation,
} from "./explain-expression-semantics";
import { readExpressionSemantics } from "./read-expression-semantics";

export interface ResolveRefQueryEnv {
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly filePath: string;
  readonly workspaceRoot: string;
}

export interface ResolveRefQueryContext {
  readonly expression: ClassExpressionHIR;
  readonly styleDocument: StyleDocumentHIR | null;
  readonly entry: AnalysisEntry;
}

export type DynamicHoverExplanation = DynamicExpressionExplanation;

export interface ResolveRefDetails {
  readonly selectors: readonly SelectorDeclHIR[];
  readonly dynamicExplanation: DynamicHoverExplanation | null;
}

export function resolveRefSelectors(
  ctx: ResolveRefQueryContext,
  env: ResolveRefQueryEnv,
): readonly SelectorDeclHIR[] {
  return resolveRefDetails(ctx, env).selectors;
}

export function resolveRefDetails(
  ctx: ResolveRefQueryContext,
  env: ResolveRefQueryEnv,
): ResolveRefDetails {
  const semantics = readExpressionSemantics(
    {
      expression: ctx.expression,
      styleDocument: ctx.styleDocument,
    },
    {
      styleDocumentForPath: env.styleDocumentForPath,
      typeResolver: env.typeResolver,
      filePath: env.filePath,
      workspaceRoot: env.workspaceRoot,
      sourceBinder: ctx.entry.sourceBinder,
      sourceBindingGraph: ctx.entry.sourceBindingGraph,
      classValueUniverses: ctx.entry.classValueUniverses,
    },
  );
  if (!semantics.styleDocument) {
    return { selectors: [], dynamicExplanation: null };
  }

  return {
    selectors: semantics.selectors,
    dynamicExplanation: buildDynamicHoverExplanation(ctx.expression, semantics),
  };
}

function buildDynamicHoverExplanation(
  expression: ClassExpressionHIR,
  semantics: ReturnType<typeof readExpressionSemantics>,
): DynamicHoverExplanation | null {
  return buildDynamicExpressionExplanation(expression, semantics);
}
