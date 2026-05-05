import ts from "typescript";
import type { Range } from "@css-module-explainer/shared";
import { makeDomainLiteralClassReference, type DomainClassReferenceHIR } from "../hir/source-types";
import type { BinderPluginV0 } from "./binder-plugin";

const PLUGIN_ID = "vue-style-module-domain";
const DOMAIN = "vue-style-module";

interface VueStyleModuleBinding {
  readonly localName: string;
  readonly moduleName: string;
}

export const vueStyleModuleBinderPluginV0: BinderPluginV0 = {
  id: PLUGIN_ID,
  version: "0",
  stability: "builtIn",
  domains: ["vue-style-modules"],
  importTargets: ["*.vue"],
  utilityTargets: ["useCssModule"],
  ownsSurfaces: ["domainClassReferenceExtraction"],
  analyzeSource(input) {
    return {
      pluginId: PLUGIN_ID,
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [],
      domainClassReferences: collectVueStyleModuleReferences(input.sourceFile),
    };
  },
};

function collectVueStyleModuleReferences(
  sourceFile: ts.SourceFile,
): readonly DomainClassReferenceHIR[] {
  const useCssModuleNames = collectUseCssModuleImportNames(sourceFile);
  if (useCssModuleNames.size === 0) return [];

  const bindings = collectVueStyleModuleBindings(sourceFile, useCssModuleNames);
  if (bindings.size === 0) return [];

  const references: DomainClassReferenceHIR[] = [];
  let nextId = 0;
  const allocateId = () => `domain-class-ref:${PLUGIN_ID}:${nextId++}`;

  function visit(node: ts.Node): void {
    if (ts.isPropertyAccessExpression(node) && ts.isIdentifier(node.expression)) {
      const binding = bindings.get(node.expression.text);
      if (binding) {
        references.push(
          makeDomainLiteralClassReference(
            allocateId(),
            PLUGIN_ID,
            DOMAIN,
            "styleAccess",
            vueStyleModuleKey(binding.moduleName, node.name.text),
            rangeOfNode(node.name, sourceFile),
          ),
        );
      }
    }

    if (
      ts.isElementAccessExpression(node) &&
      ts.isIdentifier(node.expression) &&
      ts.isStringLiteral(node.argumentExpression)
    ) {
      const binding = bindings.get(node.expression.text);
      if (binding) {
        references.push(
          makeDomainLiteralClassReference(
            allocateId(),
            PLUGIN_ID,
            DOMAIN,
            "styleAccess",
            vueStyleModuleKey(binding.moduleName, node.argumentExpression.text),
            innerStringRange(node.argumentExpression, sourceFile),
          ),
        );
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return references;
}

function collectUseCssModuleImportNames(sourceFile: ts.SourceFile): ReadonlySet<string> {
  const names = new Set<string>();

  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement)) continue;
    if (!ts.isStringLiteral(statement.moduleSpecifier)) continue;
    if (statement.moduleSpecifier.text !== "vue") continue;

    const namedBindings = statement.importClause?.namedBindings;
    if (!namedBindings || !ts.isNamedImports(namedBindings)) continue;

    for (const element of namedBindings.elements) {
      const importedName = element.propertyName?.text ?? element.name.text;
      if (importedName === "useCssModule") {
        names.add(element.name.text);
      }
    }
  }

  return names;
}

function collectVueStyleModuleBindings(
  sourceFile: ts.SourceFile,
  useCssModuleNames: ReadonlySet<string>,
): ReadonlyMap<string, VueStyleModuleBinding> {
  const bindings = new Map<string, VueStyleModuleBinding>();

  function visit(node: ts.Node): void {
    if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
      const initializer = unwrapTransparentExpression(node.initializer);
      if (
        initializer &&
        ts.isCallExpression(initializer) &&
        ts.isIdentifier(initializer.expression) &&
        useCssModuleNames.has(initializer.expression.text)
      ) {
        bindings.set(node.name.text, {
          localName: node.name.text,
          moduleName: readModuleName(initializer.arguments[0]),
        });
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return bindings;
}

function readModuleName(arg: ts.Node | undefined): string {
  const value = unwrapTransparentExpression(arg);
  if (value && ts.isStringLiteral(value)) {
    return value.text;
  }
  return "default";
}

function unwrapTransparentExpression<T extends ts.Node | undefined>(node: T): ts.Expression | null {
  if (!node || !ts.isExpression(node)) return null;
  let current: ts.Expression = node;
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

function vueStyleModuleKey(moduleName: string, className: string): string {
  return `${moduleName}.${className}`;
}

function rangeOfNode(node: ts.Node, sourceFile: ts.SourceFile): Range {
  return rangeFromOffsets(sourceFile, node.getStart(sourceFile), node.getEnd());
}

function innerStringRange(node: ts.StringLiteral, sourceFile: ts.SourceFile): Range {
  return rangeFromOffsets(sourceFile, node.getStart(sourceFile) + 1, node.getEnd() - 1);
}

function rangeFromOffsets(
  sourceFile: ts.SourceFile,
  startOffset: number,
  endOffset: number,
): Range {
  const start = sourceFile.getLineAndCharacterOfPosition(startOffset);
  const end = sourceFile.getLineAndCharacterOfPosition(endOffset);
  return {
    start: { line: start.line, character: start.character },
    end: { line: end.line, character: end.character },
  };
}
