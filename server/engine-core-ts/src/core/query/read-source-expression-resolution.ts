import { enumerateFiniteClassValues } from "../abstract-value/class-value-domain";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type { FlowResolution } from "../flow/lattice";
import type { ClassExpressionHIR, SymbolRefClassExpressionHIR } from "../hir/source-types";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { EdgeCertainty } from "../semantic/certainty";
import type { TypeResolver } from "../ts/type-resolver";
import type ts from "typescript";
import { projectExpressionSelectors } from "./project-expression-selectors";

export interface ReadSourceExpressionResolutionEnv {
  readonly styleDocumentForPath?: (path: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly filePath: string;
  readonly workspaceRoot: string;
  readonly sourceBinder?: SourceBinderResult;
  readonly sourceBindingGraph?: SourceBindingGraph;
  readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
  readonly resolveSymbolValues?: (
    sourceFile: ts.SourceFile,
    expression: SymbolRefClassExpressionHIR,
    env: Omit<ReadSourceExpressionResolutionEnv, "styleDocumentForPath" | "resolveSymbolValues">,
  ) => FlowResolution | null;
}

export interface ReadSourceExpressionResolutionContext {
  readonly expression: ClassExpressionHIR;
  readonly sourceFile: ts.SourceFile;
  readonly styleDocument?: StyleDocumentHIR | null;
}

export interface SourceExpressionResolution {
  readonly styleDocument: StyleDocumentHIR | null;
  readonly selectors: readonly SelectorDeclHIR[];
  readonly finiteValues: readonly string[] | null;
  readonly abstractValue?: FlowResolution["abstractValue"];
  readonly valueCertainty?: EdgeCertainty;
  readonly selectorCertainty: EdgeCertainty;
  readonly reason?: FlowResolution["reason"];
}

export function readSourceExpressionResolution(
  ctx: ReadSourceExpressionResolutionContext,
  env: ReadSourceExpressionResolutionEnv,
): SourceExpressionResolution {
  const styleDocument =
    ctx.styleDocument ?? env.styleDocumentForPath?.(ctx.expression.scssModulePath) ?? null;
  if (!styleDocument) {
    return {
      styleDocument: null,
      selectors: [],
      finiteValues: null,
      selectorCertainty: "possible",
    };
  }

  const projection = projectExpressionSelectors(ctx.expression, styleDocument, ctx.sourceFile, {
    typeResolver: env.typeResolver,
    filePath: env.filePath,
    workspaceRoot: env.workspaceRoot,
    ...(env.sourceBinder ? { sourceBinder: env.sourceBinder } : {}),
    ...(env.sourceBindingGraph ? { sourceBindingGraph: env.sourceBindingGraph } : {}),
    ...(env.classValueUniverses ? { classValueUniverses: env.classValueUniverses } : {}),
    ...(env.resolveSymbolValues ? { resolveSymbolValues: env.resolveSymbolValues } : {}),
  });

  return {
    styleDocument,
    selectors: projection.selectors,
    finiteValues: projection.abstractValue
      ? enumerateFiniteClassValues(projection.abstractValue)
      : null,
    ...(projection.abstractValue ? { abstractValue: projection.abstractValue } : {}),
    ...(projection.valueCertainty ? { valueCertainty: projection.valueCertainty } : {}),
    ...(projection.reason ? { reason: projection.reason } : {}),
    selectorCertainty: projection.selectorCertainty,
  };
}
