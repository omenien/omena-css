import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import { prefixClassValue, type AbstractClassValue } from "../abstract-value/class-value-domain";
import { projectAbstractValueSelectors } from "../abstract-value/selector-projection";
import type { EdgeCertainty } from "../semantic/certainty";
import { resolveSymbolExpressionValues } from "../semantic/resolve-symbol-values";
import type { ClassExpressionHIR, SymbolRefClassExpressionHIR } from "../hir/source-types";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { TypeResolver } from "../ts/type-resolver";

export interface ProjectExpressionSelectorsEnv {
  readonly typeResolver: TypeResolver;
  readonly filePath: string;
  readonly workspaceRoot: string;
  readonly sourceBinder?: SourceBinderResult;
  readonly sourceBindingGraph?: SourceBindingGraph;
  readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
  readonly resolveSymbolValues?: (
    expression: SymbolRefClassExpressionHIR,
    env: Omit<ProjectExpressionSelectorsEnv, "resolveSymbolValues">,
  ) => {
    readonly abstractValue: AbstractClassValue;
    readonly valueCertainty: EdgeCertainty;
    readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
  } | null;
}

export interface ProjectedExpressionSelectors {
  readonly selectors: readonly SelectorDeclHIR[];
  readonly abstractValue?: AbstractClassValue;
  readonly valueCertainty?: EdgeCertainty;
  readonly selectorCertainty: EdgeCertainty;
  readonly reason?: "flowLiteral" | "flowBranch" | "typeUnion";
}

export function projectExpressionSelectors(
  expression: ClassExpressionHIR,
  styleDocument: StyleDocumentHIR,
  env: ProjectExpressionSelectorsEnv,
): ProjectedExpressionSelectors {
  const baseEnv = {
    typeResolver: env.typeResolver,
    filePath: env.filePath,
    workspaceRoot: env.workspaceRoot,
    ...(env.sourceBinder ? { sourceBinder: env.sourceBinder } : {}),
    ...(env.sourceBindingGraph ? { sourceBindingGraph: env.sourceBindingGraph } : {}),
    ...(env.classValueUniverses ? { classValueUniverses: env.classValueUniverses } : {}),
  } satisfies Omit<ProjectExpressionSelectorsEnv, "resolveSymbolValues">;
  switch (expression.kind) {
    case "literal":
    case "styleAccess": {
      return {
        selectors: findCanonicalSelectors(styleDocument, expression.className),
        selectorCertainty: "exact",
      };
    }
    case "template": {
      const abstractValue = prefixClassValue(expression.staticPrefix);
      const projection = projectAbstractValueSelectors(abstractValue, styleDocument);
      return {
        selectors: projection.selectors,
        abstractValue,
        selectorCertainty: projection.certainty,
      };
    }
    case "symbolRef":
      return projectSymbolRefSelectors(expression, styleDocument, env.resolveSymbolValues, baseEnv);
    default:
      expression satisfies never;
      return {
        selectors: [],
        selectorCertainty: "possible",
      };
  }
}

function projectSymbolRefSelectors(
  expression: SymbolRefClassExpressionHIR,
  styleDocument: StyleDocumentHIR,
  resolveSymbolValues: ProjectExpressionSelectorsEnv["resolveSymbolValues"] | undefined,
  env: Omit<ProjectExpressionSelectorsEnv, "resolveSymbolValues">,
): ProjectedExpressionSelectors {
  const resolved =
    resolveSymbolValues?.(expression, env) ?? resolveSymbolExpressionValues(expression, env);
  if (!resolved) {
    return {
      selectors: [],
      selectorCertainty: "possible",
    };
  }
  const projection = projectAbstractValueSelectors(resolved.abstractValue, styleDocument, {
    ...(env.classValueUniverses ? { classValueUniverses: env.classValueUniverses } : {}),
    universeOwnerName: expression.rootName,
  });
  return {
    selectors: projection.selectors,
    abstractValue: resolved.abstractValue,
    valueCertainty: resolved.valueCertainty,
    selectorCertainty: projection.certainty,
    reason: resolved.reason,
  };
}

function findCanonicalSelectors(
  styleDocument: StyleDocumentHIR,
  viewName: string,
): readonly SelectorDeclHIR[] {
  const matches = styleDocument.selectors.filter((selector) => selector.name === viewName);
  if (matches.length === 0) return [];
  const canonicalNames = new Set(matches.map((selector) => selector.canonicalName));
  const canonicalSelectors = styleDocument.selectors.filter(
    (selector) => selector.viewKind === "canonical" && canonicalNames.has(selector.canonicalName),
  );
  return canonicalSelectors.length > 0 ? canonicalSelectors : matches;
}
