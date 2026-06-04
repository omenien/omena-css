import { enumerateFiniteClassValues } from "../abstract-value/class-value-domain";
import { findClosestMatch } from "../util/text-utils";
import type { TypeResolver } from "../ts/type-resolver";
import type { FlowResolution } from "../flow/lattice";
import type ts from "typescript";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type {
  ClassExpressionHIR,
  LiteralClassExpressionHIR,
  SymbolRefClassExpressionHIR,
  TemplateClassExpressionHIR,
} from "../hir/source-types";
import type { StyleDocumentHIR } from "../hir/style-types";
import {
  readExpressionSemantics,
  type ReducedClassValueDerivation,
} from "./read-expression-semantics";

export interface InvalidClassReferenceQueryEnv {
  readonly typeResolver: TypeResolver;
  readonly filePath: string;
  readonly workspaceRoot: string;
  readonly sourceBinder?: SourceBinderResult;
  readonly sourceBindingGraph?: SourceBindingGraph;
  readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
  readonly resolveSymbolValues?: (
    sourceFile: ts.SourceFile,
    expression: SymbolRefClassExpressionHIR,
    env: Omit<InvalidClassReferenceQueryEnv, "resolveSymbolValues">,
  ) => FlowResolution | null;
}

export type InvalidClassReferenceFinding =
  | {
      readonly kind: "missingStaticClass";
      readonly expression: LiteralClassExpressionHIR;
      readonly range: LiteralClassExpressionHIR["range"];
      readonly suggestion?: string;
    }
  | {
      readonly kind: "missingTemplatePrefix";
      readonly expression: TemplateClassExpressionHIR;
      readonly range: TemplateClassExpressionHIR["range"];
    }
  | {
      readonly kind: "missingResolvedClassValues";
      readonly expression: SymbolRefClassExpressionHIR;
      readonly range: SymbolRefClassExpressionHIR["range"];
      readonly missingValues: readonly string[];
      readonly abstractValue: FlowResolution["abstractValue"];
      readonly valueCertainty: "exact" | "inferred" | "possible";
      readonly selectorCertainty: "exact" | "inferred" | "possible";
      readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
      readonly valueDomainDerivation?: ReducedClassValueDerivation;
    }
  | {
      readonly kind: "missingResolvedClassDomain";
      readonly expression: SymbolRefClassExpressionHIR;
      readonly range: SymbolRefClassExpressionHIR["range"];
      readonly abstractValue: FlowResolution["abstractValue"];
      readonly valueCertainty: "exact" | "inferred" | "possible";
      readonly selectorCertainty: "exact" | "inferred" | "possible";
      readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
      readonly valueDomainDerivation?: ReducedClassValueDerivation;
    };

export function findInvalidClassReference(
  expression: ClassExpressionHIR,
  sourceFile: ts.SourceFile,
  styleDocument: StyleDocumentHIR,
  env: InvalidClassReferenceQueryEnv,
): InvalidClassReferenceFinding | null {
  if (expression.origin !== "cxCall") return null;

  switch (expression.kind) {
    case "literal": {
      if (hasSelectorNamed(styleDocument, expression.className)) return null;
      const suggestion = findClosestMatch(
        expression.className,
        styleDocument.selectors.map((selector) => selector.name),
      );
      return {
        kind: "missingStaticClass",
        expression,
        range: expression.range,
        ...(suggestion ? { suggestion } : {}),
      };
    }
    case "template": {
      if (anySelectorStartsWith(styleDocument, expression.staticPrefix)) return null;
      return {
        kind: "missingTemplatePrefix",
        expression,
        range: expression.range,
      };
    }
    case "symbolRef": {
      const semantics = readExpressionSemantics(
        {
          expression,
          sourceFile,
          styleDocument,
        },
        env,
      );
      if (!semantics.abstractValue || !semantics.reason || !semantics.valueCertainty) return null;
      const finiteValues =
        semantics.finiteValues ?? enumerateFiniteClassValues(semantics.abstractValue);
      if (!finiteValues) {
        return semantics.selectors.length === 0
          ? {
              kind: "missingResolvedClassDomain",
              expression,
              range: expression.range,
              abstractValue: semantics.abstractValue,
              valueCertainty: semantics.valueCertainty,
              selectorCertainty: semantics.selectorCertainty,
              reason: semantics.reason,
              ...(semantics.valueDomainDerivation
                ? { valueDomainDerivation: semantics.valueDomainDerivation }
                : {}),
            }
          : null;
      }
      const missingValues = finiteValues.filter((value) => !hasSelectorNamed(styleDocument, value));
      if (missingValues.length === 0) return null;
      return {
        kind: "missingResolvedClassValues",
        expression,
        range: expression.range,
        missingValues,
        abstractValue: semantics.abstractValue,
        valueCertainty: semantics.valueCertainty,
        selectorCertainty: semantics.selectorCertainty,
        reason: semantics.reason,
        ...(semantics.valueDomainDerivation
          ? { valueDomainDerivation: semantics.valueDomainDerivation }
          : {}),
      };
    }
    case "styleAccess":
      return null;
    default:
      expression satisfies never;
      return null;
  }
}

function anySelectorStartsWith(styleDocument: StyleDocumentHIR, prefix: string): boolean {
  for (const selector of styleDocument.selectors) {
    if (selector.name.startsWith(prefix)) return true;
  }
  return false;
}

function hasSelectorNamed(styleDocument: StyleDocumentHIR, name: string): boolean {
  return styleDocument.selectors.some((selector) => selector.name === name);
}
