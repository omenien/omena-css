import type { StyleImport } from "@omena/shared";
import type { SourceBinderResult } from "../../server/engine-core-ts/src/core/binder/scope-types";
import type { ResolvedCxBinding } from "../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { buildSourceDocument } from "../../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";
import {
  makeLiteralClassExpression,
  makeStyleAccessClassExpression,
  makeSymbolRefClassExpression,
  makeTemplateClassExpression,
  type ClassExpressionHIR,
  type SourceDocumentHIR,
} from "../../server/engine-core-ts/src/core/hir/source-types";

type ExpressionOrigin = "cxCall" | "styleAccess";

type LiteralExpressionSpec = {
  readonly kind: "literal";
  readonly origin: ExpressionOrigin;
  readonly scssModulePath: string;
  readonly className: string;
  readonly range: ClassExpressionHIR["range"];
};

type TemplateExpressionSpec = {
  readonly kind: "template";
  readonly origin: ExpressionOrigin;
  readonly scssModulePath: string;
  readonly rawTemplate: string;
  readonly staticPrefix: string;
  readonly range: ClassExpressionHIR["range"];
};

type SymbolRefExpressionSpec = {
  readonly kind: "symbolRef";
  readonly origin: ExpressionOrigin;
  readonly scssModulePath: string;
  readonly rawReference: string;
  readonly rootName?: string;
  readonly pathSegments?: readonly string[];
  readonly rootBindingDeclId?: string;
  readonly range: ClassExpressionHIR["range"];
};

type StyleAccessExpressionSpec = {
  readonly kind: "styleAccess";
  readonly scssModulePath: string;
  readonly bindingDeclId?: string;
  readonly className: string;
  readonly accessPath?: readonly string[];
  readonly range: ClassExpressionHIR["range"];
};

export type TestClassExpressionSpec =
  | LiteralExpressionSpec
  | TemplateExpressionSpec
  | SymbolRefExpressionSpec
  | StyleAccessExpressionSpec;

export function buildClassExpressions(args: {
  readonly filePath: string;
  readonly bindings: readonly ResolvedCxBinding[];
  readonly stylesBindings?: ReadonlyMap<string, StyleImport>;
  readonly classUtilNames?: readonly string[];
  readonly expressions: readonly TestClassExpressionSpec[];
}): readonly ClassExpressionHIR[] {
  return buildSourceDocumentFixture(args).classExpressions;
}

export function buildSourceDocumentFixture(args: {
  readonly filePath: string;
  readonly bindings: readonly ResolvedCxBinding[];
  readonly stylesBindings?: ReadonlyMap<string, StyleImport>;
  readonly classUtilNames?: readonly string[];
  readonly expressions: readonly TestClassExpressionSpec[];
  readonly sourceBinder?: SourceBinderResult;
}): SourceDocumentHIR {
  return buildSourceDocument({
    filePath: args.filePath,
    cxBindings: args.bindings,
    stylesBindings: args.stylesBindings ?? new Map(),
    classUtilNames: args.classUtilNames ?? [],
    classExpressions: args.expressions.map(toClassExpression),
    sourceBinder: args.sourceBinder,
  });
}

function toClassExpression(expression: TestClassExpressionSpec, index: number): ClassExpressionHIR {
  const id = `class-expr:${index}`;
  switch (expression.kind) {
    case "literal":
      return makeLiteralClassExpression(
        id,
        expression.origin,
        expression.scssModulePath,
        expression.className,
        expression.range,
      );
    case "template":
      return makeTemplateClassExpression(
        id,
        expression.origin,
        expression.scssModulePath,
        expression.rawTemplate,
        expression.staticPrefix,
        expression.range,
      );
    case "symbolRef": {
      const rootName = expression.rootName ?? expression.rawReference.split(".")[0]!;
      const pathSegments = expression.pathSegments ?? expression.rawReference.split(".").slice(1);
      return makeSymbolRefClassExpression(
        id,
        expression.origin,
        expression.scssModulePath,
        expression.rawReference,
        rootName,
        pathSegments,
        expression.range,
        expression.rootBindingDeclId,
      );
    }
    case "styleAccess":
      return makeStyleAccessClassExpression(
        id,
        expression.scssModulePath,
        expression.bindingDeclId ?? "synthetic-style-import-decl:test",
        expression.className,
        expression.accessPath ?? [expression.className],
        expression.range,
      );
    default:
      expression satisfies never;
      return expression;
  }
}
