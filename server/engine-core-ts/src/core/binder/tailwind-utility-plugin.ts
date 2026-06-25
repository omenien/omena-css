import ts, { lineCharOfPosition, nodeStart, nodeEnd, nodeText } from "../../ts-facade";
import type { Range } from "@omena/shared";
import { detectClassUtilImports } from "../cx/binding-detector";
import {
  makeDomainLiteralClassReference,
  makeDomainTemplateClassReference,
  type DomainClassReferenceHIR,
} from "../hir/source-types";
import type { BinderPluginV0 } from "./binder-plugin";

const PLUGIN_ID = "tailwind-uno-utility-domain";
const DOMAIN = "utility-css";

export const tailwindUnoUtilityBinderPluginV0: BinderPluginV0 = {
  id: PLUGIN_ID,
  version: "0",
  stability: "builtIn",
  domains: ["tailwind-utilities", "unocss-utilities"],
  importTargets: [],
  utilityTargets: ["class", "className", "classnames", "clsx", "clsx/lite"],
  ownsSurfaces: ["domainClassReferenceExtraction"],
  analyzeSource(input) {
    return {
      pluginId: PLUGIN_ID,
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [],
      domainClassReferences: collectUtilityClassReferences(input.sourceFile),
      classValueUniverses: [],
    };
  },
};

function collectUtilityClassReferences(
  sourceFile: ts.SourceFile,
): readonly DomainClassReferenceHIR[] {
  const references: DomainClassReferenceHIR[] = [];
  let nextId = 0;
  const allocateId = () => `domain-class-ref:${PLUGIN_ID}:${nextId++}`;
  const classUtilNames = new Set(detectClassUtilImports(sourceFile));
  const callsCollectedFromClassAttributes = new WeakSet<ts.CallExpression>();

  function visit(node: ts.Node): void {
    if (
      ts.isJsxAttribute(node) &&
      ts.isIdentifier(node.name) &&
      isClassAttributeName(node.name.text)
    ) {
      collectFromJsxAttribute(
        node,
        sourceFile,
        references,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
    }

    if (
      ts.isCallExpression(node) &&
      !callsCollectedFromClassAttributes.has(node) &&
      isKnownClassUtilityCall(node, classUtilNames)
    ) {
      for (const arg of node.arguments) {
        collectFromExpression(
          arg,
          "classUtilityCall",
          sourceFile,
          references,
          allocateId,
          classUtilNames,
          callsCollectedFromClassAttributes,
        );
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return references;
}

function collectFromJsxAttribute(
  attribute: ts.JsxAttribute,
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
  classUtilNames: ReadonlySet<string>,
  callsCollectedFromClassAttributes: WeakSet<ts.CallExpression>,
): void {
  const initializer = attribute.initializer;
  if (!initializer) return;

  if (ts.isStringLiteral(initializer)) {
    collectLiteralTokens(
      initializer.text,
      nodeStart(initializer, sourceFile) + 1,
      "jsxClassAttribute",
      sourceFile,
      out,
      allocateId,
    );
    return;
  }

  if (ts.isJsxExpression(initializer) && initializer.expression) {
    collectFromExpression(
      initializer.expression,
      "jsxClassAttribute",
      sourceFile,
      out,
      allocateId,
      classUtilNames,
      callsCollectedFromClassAttributes,
    );
  }
}

function collectFromExpression(
  expression: ts.Expression,
  origin: DomainClassReferenceHIR["origin"],
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
  classUtilNames: ReadonlySet<string>,
  callsCollectedFromClassAttributes: WeakSet<ts.CallExpression>,
): void {
  const value = unwrapTransparentExpression(expression);

  if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
    collectLiteralTokens(
      value.text,
      nodeStart(value, sourceFile) + 1,
      origin,
      sourceFile,
      out,
      allocateId,
    );
    return;
  }

  if (ts.isTemplateExpression(value)) {
    collectTemplatePrefix(value, origin, sourceFile, out, allocateId);
    return;
  }

  if (ts.isBinaryExpression(value)) {
    if (value.operatorToken.kind === ts.SyntaxKind.PlusToken) {
      const staticPrefix = extractStaticStringPrefix(value);
      if (staticPrefix.length > 0) {
        out.push(
          makeDomainTemplateClassReference(
            allocateId(),
            PLUGIN_ID,
            DOMAIN,
            origin,
            nodeText(value, sourceFile),
            staticPrefix,
            rangeOfNode(value, sourceFile),
          ),
        );
      }
      return;
    }

    if (value.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken) {
      collectFromExpression(
        value.right,
        origin,
        sourceFile,
        out,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
      return;
    }
  }

  if (ts.isConditionalExpression(value)) {
    collectFromExpression(
      value.whenTrue,
      origin,
      sourceFile,
      out,
      allocateId,
      classUtilNames,
      callsCollectedFromClassAttributes,
    );
    collectFromExpression(
      value.whenFalse,
      origin,
      sourceFile,
      out,
      allocateId,
      classUtilNames,
      callsCollectedFromClassAttributes,
    );
    return;
  }

  if (ts.isArrayLiteralExpression(value)) {
    for (const element of value.elements) {
      collectFromExpression(
        element,
        origin,
        sourceFile,
        out,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
    }
    return;
  }

  if (ts.isObjectLiteralExpression(value)) {
    collectObjectLiteralKeys(
      value,
      origin,
      sourceFile,
      out,
      allocateId,
      classUtilNames,
      callsCollectedFromClassAttributes,
    );
    return;
  }

  if (ts.isSpreadElement(value) && ts.isArrayLiteralExpression(value.expression)) {
    for (const element of value.expression.elements) {
      collectFromExpression(
        element,
        origin,
        sourceFile,
        out,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
    }
    return;
  }

  if (ts.isCallExpression(value) && isKnownClassUtilityCall(value, classUtilNames)) {
    if (origin === "jsxClassAttribute") {
      callsCollectedFromClassAttributes.add(value);
    }
    for (const arg of value.arguments) {
      collectFromExpression(
        arg,
        origin,
        sourceFile,
        out,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
    }
  }
}

function collectObjectLiteralKeys(
  value: ts.ObjectLiteralExpression,
  origin: DomainClassReferenceHIR["origin"],
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
  classUtilNames: ReadonlySet<string>,
  callsCollectedFromClassAttributes: WeakSet<ts.CallExpression>,
): void {
  for (const prop of value.properties) {
    if (!ts.isPropertyAssignment(prop) && !ts.isShorthandPropertyAssignment(prop)) continue;
    const name = prop.name;
    if (!name) continue;

    if (ts.isIdentifier(name)) {
      out.push(
        makeDomainLiteralClassReference(
          allocateId(),
          PLUGIN_ID,
          DOMAIN,
          origin,
          name.text,
          rangeOfNode(name, sourceFile),
        ),
      );
      continue;
    }

    if (ts.isStringLiteral(name)) {
      collectLiteralTokens(
        name.text,
        nodeStart(name, sourceFile) + 1,
        origin,
        sourceFile,
        out,
        allocateId,
      );
      continue;
    }

    if (ts.isComputedPropertyName(name)) {
      collectFromExpression(
        name.expression,
        origin,
        sourceFile,
        out,
        allocateId,
        classUtilNames,
        callsCollectedFromClassAttributes,
      );
    }
  }
}

function collectLiteralTokens(
  text: string,
  textStartOffset: number,
  origin: DomainClassReferenceHIR["origin"],
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
): void {
  for (const match of text.matchAll(/\S+/g)) {
    const className = match[0];
    if (className.length === 0) continue;
    const tokenStart = textStartOffset + (match.index ?? 0);
    const tokenEnd = tokenStart + className.length;
    out.push(
      makeDomainLiteralClassReference(
        allocateId(),
        PLUGIN_ID,
        DOMAIN,
        origin,
        className,
        rangeFromOffsets(sourceFile, tokenStart, tokenEnd),
      ),
    );
  }
}

function collectTemplatePrefix(
  value: ts.TemplateExpression,
  origin: DomainClassReferenceHIR["origin"],
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
): void {
  const staticPrefix = lastNonWhitespaceToken(value.head.text);
  if (staticPrefix.length === 0) return;
  out.push(
    makeDomainTemplateClassReference(
      allocateId(),
      PLUGIN_ID,
      DOMAIN,
      origin,
      nodeText(value, sourceFile),
      staticPrefix,
      rangeOfNode(value, sourceFile),
    ),
  );
}

function isKnownClassUtilityCall(
  call: ts.CallExpression,
  classUtilNames: ReadonlySet<string>,
): boolean {
  return ts.isIdentifier(call.expression) && classUtilNames.has(call.expression.text);
}

function isClassAttributeName(name: string): boolean {
  return name === "className" || name === "class";
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
    return lastNonWhitespaceToken(value.text);
  }

  if (ts.isTemplateExpression(value)) {
    return lastNonWhitespaceToken(value.head.text);
  }

  if (ts.isBinaryExpression(value) && value.operatorToken.kind === ts.SyntaxKind.PlusToken) {
    const leftPrefix = extractStaticStringPrefix(value.left);
    if (leftPrefix.length === 0) return "";
    const rightPrefix = extractStaticStringPrefix(value.right);
    return rightPrefix.length > 0 ? leftPrefix + rightPrefix : leftPrefix;
  }

  return "";
}

function lastNonWhitespaceToken(text: string): string {
  const match = text.match(/\S+$/);
  return match?.[0] ?? "";
}

function rangeOfNode(node: ts.Node, sourceFile: ts.SourceFile): Range {
  return rangeFromOffsets(sourceFile, nodeStart(node, sourceFile), nodeEnd(node));
}

function rangeFromOffsets(
  sourceFile: ts.SourceFile,
  startOffset: number,
  endOffset: number,
): Range {
  const start = lineCharOfPosition(sourceFile, startOffset);
  const end = lineCharOfPosition(sourceFile, endOffset);
  return {
    start: { line: start.line, character: start.character },
    end: { line: end.line, character: end.character },
  };
}
