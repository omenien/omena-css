import ts from "typescript";
import type { Range, StyleImport } from "@css-module-explainer/shared";
import {
  buildSourceBindingGraph,
  findImportBindingDeclId,
  resolveBindingAtOffset,
  type SourceBindingGraph,
} from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";
import type { ResolvedCxBinding } from "./resolved-bindings";
import {
  makeLiteralClassExpression,
  makeStyleAccessClassExpression,
  makeSymbolRefClassExpression,
  makeTemplateClassExpression,
  type ClassExpressionHIR,
} from "../hir/source-types";

/**
 * Unified source-expression producer.
 *
 * Single AST walk that emits `ClassExpressionHIR[]` covering both
 * cx() call arguments and direct `styles.x` access. Each cx() call
 * produces entries with `origin: "cxCall"`; each `styles.x`
 * access produces a `styleAccess` expression.
 */
export function parseClassExpressions(
  sourceFile: ts.SourceFile,
  bindings: readonly ResolvedCxBinding[],
  stylesBindings: ReadonlyMap<string, StyleImport>,
  binder: SourceBinderResult,
): ClassExpressionHIR[] {
  const expressions: ClassExpressionHIR[] = [];
  let nextId = 0;
  const allocateId = () => `class-expr:${nextId++}`;

  for (const binding of bindings) {
    collectCxCallExpressions(sourceFile, binding, binder, expressions, allocateId);
  }

  if (stylesBindings.size > 0) {
    collectStyleAccessExpressions(sourceFile, stylesBindings, binder, expressions, allocateId);
  }

  return expressions;
}

function collectCxCallExpressions(
  sourceFile: ts.SourceFile,
  binding: ResolvedCxBinding,
  binder: SourceBinderResult,
  out: ClassExpressionHIR[],
  allocateId: () => string,
): void {
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) && isMatchingCxCall(node, binding, sourceFile, binder)) {
      for (const arg of node.arguments) {
        extractFromArgument(arg, binding, binder, sourceFile, out, allocateId);
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
}

function isMatchingCxCall(
  call: ts.CallExpression,
  binding: ResolvedCxBinding,
  sourceFile: ts.SourceFile,
  binder: SourceBinderResult,
): boolean {
  if (!ts.isIdentifier(call.expression)) return false;
  if (call.expression.text !== binding.cxVarName) return false;
  const resolution = resolveBindingAtOffset(
    buildGraphForBinder(sourceFile, binder),
    binding.cxVarName,
    call.expression.getStart(sourceFile),
  );
  return resolution?.declId === binding.bindingDeclId;
}

function extractFromArgument(
  arg: ts.Expression,
  binding: ResolvedCxBinding,
  binder: SourceBinderResult,
  sourceFile: ts.SourceFile,
  out: ClassExpressionHIR[],
  allocateId: () => string,
): void {
  const value = unwrapTransparentExpression(arg);

  if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
    out.push(
      makeLiteralClassExpression(
        allocateId(),
        "cxCall",
        binding.scssModulePath,
        value.text,
        innerStringRange(value, sourceFile),
      ),
    );
    return;
  }

  if (ts.isObjectLiteralExpression(value)) {
    extractObjectLiteral(value, binding, binder, sourceFile, out, allocateId);
    return;
  }

  if (
    ts.isBinaryExpression(value) &&
    value.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken
  ) {
    extractFromArgument(value.right, binding, binder, sourceFile, out, allocateId);
    return;
  }

  if (ts.isConditionalExpression(value)) {
    extractFromArgument(value.whenTrue, binding, binder, sourceFile, out, allocateId);
    extractFromArgument(value.whenFalse, binding, binder, sourceFile, out, allocateId);
    return;
  }

  if (ts.isTemplateExpression(value)) {
    out.push(
      makeTemplateClassExpression(
        allocateId(),
        "cxCall",
        binding.scssModulePath,
        value.getText(sourceFile),
        value.head.text,
        rangeOfNode(value, sourceFile),
      ),
    );
    return;
  }

  if (ts.isBinaryExpression(value) && value.operatorToken.kind === ts.SyntaxKind.PlusToken) {
    const staticPrefix = extractStaticStringPrefix(value);
    if (staticPrefix.length > 0) {
      out.push(
        makeTemplateClassExpression(
          allocateId(),
          "cxCall",
          binding.scssModulePath,
          value.getText(sourceFile),
          staticPrefix,
          rangeOfNode(value, sourceFile),
        ),
      );
      return;
    }
  }

  if (ts.isPropertyAccessExpression(value) || ts.isIdentifier(value)) {
    const rawReference = ts.isIdentifier(value) ? value.text : value.getText(sourceFile);
    const [rootName, ...pathSegments] = rawReference.split(".");
    const rootBindingDeclId = resolveRootBindingDeclId(
      binder,
      rootName ?? rawReference,
      value.getStart(sourceFile),
    );
    out.push(
      makeSymbolRefClassExpression(
        allocateId(),
        "cxCall",
        binding.scssModulePath,
        rawReference,
        rootName ?? rawReference,
        pathSegments,
        rangeOfNode(value, sourceFile),
        rootBindingDeclId ?? undefined,
      ),
    );
    return;
  }

  if (ts.isArrayLiteralExpression(value)) {
    for (const el of value.elements) {
      extractFromArgument(el, binding, binder, sourceFile, out, allocateId);
    }
    return;
  }

  if (ts.isSpreadElement(value) && ts.isArrayLiteralExpression(value.expression)) {
    for (const el of value.expression.elements) {
      extractFromArgument(el, binding, binder, sourceFile, out, allocateId);
    }
  }
}

function extractObjectLiteral(
  arg: ts.ObjectLiteralExpression,
  binding: ResolvedCxBinding,
  binder: SourceBinderResult,
  sourceFile: ts.SourceFile,
  out: ClassExpressionHIR[],
  allocateId: () => string,
): void {
  for (const prop of arg.properties) {
    if (!ts.isPropertyAssignment(prop) && !ts.isShorthandPropertyAssignment(prop)) {
      continue;
    }

    const name = prop.name;
    if (!name) continue;

    if (ts.isIdentifier(name)) {
      out.push(
        makeLiteralClassExpression(
          allocateId(),
          "cxCall",
          binding.scssModulePath,
          name.text,
          rangeOfNode(name, sourceFile),
        ),
      );
    } else if (ts.isStringLiteral(name)) {
      out.push(
        makeLiteralClassExpression(
          allocateId(),
          "cxCall",
          binding.scssModulePath,
          name.text,
          innerStringRange(name, sourceFile),
        ),
      );
    } else if (ts.isComputedPropertyName(name)) {
      extractFromArgument(name.expression, binding, binder, sourceFile, out, allocateId);
    }
  }
}

function collectStyleAccessExpressions(
  sourceFile: ts.SourceFile,
  stylesBindings: ReadonlyMap<string, StyleImport>,
  binder: SourceBinderResult,
  out: ClassExpressionHIR[],
  allocateId: () => string,
): void {
  function visit(node: ts.Node): void {
    if (
      ts.isPropertyAccessExpression(node) &&
      ts.isIdentifier(node.expression) &&
      ts.isIdentifier(node.name)
    ) {
      const styleImport = stylesBindings.get(node.expression.text);
      if (styleImport) {
        const bindingDeclId = resolveStyleImportDeclId(
          binder,
          node.expression.text,
          node.expression.getStart(sourceFile),
        );
        if (!bindingDeclId) {
          ts.forEachChild(node, visit);
          return;
        }
        const propName = node.name;
        const start = sourceFile.getLineAndCharacterOfPosition(propName.getStart(sourceFile));
        const end = sourceFile.getLineAndCharacterOfPosition(propName.getEnd());
        out.push(
          makeStyleAccessClassExpression(
            allocateId(),
            styleImport.absolutePath,
            bindingDeclId,
            propName.text,
            [propName.text],
            {
              start: { line: start.line, character: start.character },
              end: { line: end.line, character: end.character },
            },
          ),
        );
      }
    }

    if (
      ts.isElementAccessExpression(node) &&
      ts.isIdentifier(node.expression) &&
      ts.isStringLiteral(node.argumentExpression)
    ) {
      const styleImport = stylesBindings.get(node.expression.text);
      if (styleImport) {
        const bindingDeclId = resolveStyleImportDeclId(
          binder,
          node.expression.text,
          node.expression.getStart(sourceFile),
        );
        if (!bindingDeclId) {
          ts.forEachChild(node, visit);
          return;
        }
        const className = node.argumentExpression.text;
        out.push(
          makeStyleAccessClassExpression(
            allocateId(),
            styleImport.absolutePath,
            bindingDeclId,
            className,
            [className],
            innerStringRange(node.argumentExpression, sourceFile),
          ),
        );
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
}

function resolveStyleImportDeclId(
  binder: SourceBinderResult,
  localName: string,
  offset: number,
): string | null {
  const graph = buildGraphForBinder(
    {
      ...sourceFileStubForBinder(binder),
    },
    binder,
  );
  const expectedDeclId = findImportBindingDeclId(graph, localName);
  if (!expectedDeclId) return null;
  const resolution = resolveBindingAtOffset(graph, localName, offset);
  return resolution?.declId === expectedDeclId ? expectedDeclId : null;
}

function resolveRootBindingDeclId(
  binder: SourceBinderResult,
  rootName: string,
  offset: number,
): string | null {
  const resolution = resolveBindingAtOffset(
    buildGraphForBinder(
      {
        ...sourceFileStubForBinder(binder),
      },
      binder,
    ),
    rootName,
    offset,
  );
  return resolution?.declId ?? null;
}

function buildGraphForBinder(
  sourceFile: ts.SourceFile,
  binder: SourceBinderResult,
): SourceBindingGraph {
  return buildSourceBindingGraph(
    {
      filePath: sourceFile.fileName,
      kind: "source",
      language: "unknown",
      styleImports: [],
      utilityBindings: [],
      classExpressions: [],
      domainClassReferences: [],
    },
    binder,
  );
}

function sourceFileStubForBinder(binder: SourceBinderResult): ts.SourceFile {
  return ts.createSourceFile(binder.filePath, "", ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
}

function unwrapTransparentExpression(expression: ts.Expression): ts.Expression {
  let current = expression;
  while (
    ts.isParenthesizedExpression(current) ||
    ts.isAsExpression(current) ||
    ts.isTypeAssertionExpression(current) ||
    ts.isNonNullExpression(current) ||
    ts.isSatisfiesExpression(current)
  ) {
    current = current.expression;
  }
  return current;
}

function extractStaticStringPrefix(expression: ts.Expression): string {
  const value = unwrapTransparentExpression(expression);

  if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
    return value.text;
  }

  if (ts.isTemplateExpression(value)) {
    return value.head.text;
  }

  if (ts.isBinaryExpression(value) && value.operatorToken.kind === ts.SyntaxKind.PlusToken) {
    const leftPrefix = extractStaticStringPrefix(value.left);
    if (leftPrefix.length === 0) return "";
    const rightPrefix = extractStaticStringPrefix(value.right);
    return rightPrefix.length > 0 ? leftPrefix + rightPrefix : leftPrefix;
  }

  return "";
}

function rangeOfNode(node: ts.Node, sourceFile: ts.SourceFile): Range {
  const start = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
  const end = sourceFile.getLineAndCharacterOfPosition(node.getEnd());
  return {
    start: { line: start.line, character: start.character },
    end: { line: end.line, character: end.character },
  };
}

function innerStringRange(
  node: ts.StringLiteral | ts.NoSubstitutionTemplateLiteral,
  sourceFile: ts.SourceFile,
): Range {
  const startPos = node.getStart(sourceFile) + 1;
  const endPos = node.getEnd() - 1;
  const start = sourceFile.getLineAndCharacterOfPosition(startPos);
  const end = sourceFile.getLineAndCharacterOfPosition(endPos);
  return {
    start: { line: start.line, character: start.character },
    end: { line: end.line, character: end.character },
  };
}
