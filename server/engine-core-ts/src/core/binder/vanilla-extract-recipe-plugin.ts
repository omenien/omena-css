import ts from "typescript";
import type { Range } from "@css-module-explainer/shared";
import {
  makeDomainLiteralClassReference,
  makeDomainTemplateClassReference,
  type DomainClassReferenceHIR,
} from "../hir/source-types";
import type { BinderPluginV0 } from "./binder-plugin";

const PLUGIN_ID = "vanilla-extract-recipe-domain";
const DOMAIN = "vanilla-extract-recipe";

interface RecipeBinding {
  readonly localName: string;
  readonly variants: ReadonlyMap<string, ReadonlySet<string>>;
}

export const vanillaExtractRecipeBinderPluginV0: BinderPluginV0 = {
  id: PLUGIN_ID,
  version: "0",
  stability: "builtIn",
  domains: ["vanilla-extract-recipes"],
  importTargets: ["@vanilla-extract/recipes"],
  utilityTargets: ["recipe"],
  ownsSurfaces: ["domainClassReferenceExtraction"],
  analyzeSource(input) {
    return {
      pluginId: PLUGIN_ID,
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [],
      domainClassReferences: collectRecipeVariantReferences(input.sourceFile),
    };
  },
};

function collectRecipeVariantReferences(
  sourceFile: ts.SourceFile,
): readonly DomainClassReferenceHIR[] {
  const recipeImportNames = collectRecipeImportNames(sourceFile);
  if (recipeImportNames.size === 0) return [];

  const recipes = collectRecipeBindings(sourceFile, recipeImportNames);
  if (recipes.size === 0) return [];

  const references: DomainClassReferenceHIR[] = [];
  let nextId = 0;
  const allocateId = () => `domain-class-ref:${PLUGIN_ID}:${nextId++}`;

  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const recipe = recipes.get(node.expression.text);
      const args = node.arguments;
      if (recipe && args.length > 0) {
        collectRecipeCallReferences(args[0]!, recipe, sourceFile, references, allocateId);
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return references;
}

function collectRecipeImportNames(sourceFile: ts.SourceFile): ReadonlySet<string> {
  const names = new Set<string>();

  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement)) continue;
    if (!ts.isStringLiteral(statement.moduleSpecifier)) continue;
    if (statement.moduleSpecifier.text !== "@vanilla-extract/recipes") continue;

    const namedBindings = statement.importClause?.namedBindings;
    if (!namedBindings || !ts.isNamedImports(namedBindings)) continue;

    for (const element of namedBindings.elements) {
      const importedName = element.propertyName?.text ?? element.name.text;
      if (importedName === "recipe") {
        names.add(element.name.text);
      }
    }
  }

  return names;
}

function collectRecipeBindings(
  sourceFile: ts.SourceFile,
  recipeImportNames: ReadonlySet<string>,
): ReadonlyMap<string, RecipeBinding> {
  const bindings = new Map<string, RecipeBinding>();

  function visit(node: ts.Node): void {
    if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
      const initializer = unwrapTransparentExpression(node.initializer);
      if (
        initializer &&
        ts.isCallExpression(initializer) &&
        ts.isIdentifier(initializer.expression) &&
        recipeImportNames.has(initializer.expression.text)
      ) {
        const variants = parseRecipeVariants(initializer.arguments[0]);
        if (variants.size > 0) {
          bindings.set(node.name.text, { localName: node.name.text, variants });
        }
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return bindings;
}

function parseRecipeVariants(node: ts.Node | undefined): ReadonlyMap<string, ReadonlySet<string>> {
  const variants = new Map<string, ReadonlySet<string>>();
  const config = unwrapTransparentExpression(node);
  if (!config || !ts.isObjectLiteralExpression(config)) return variants;

  const variantsProperty = findObjectProperty(config, "variants");
  const variantsObject = variantsProperty ? unwrapTransparentExpression(variantsProperty) : null;
  if (!variantsObject || !ts.isObjectLiteralExpression(variantsObject)) return variants;

  for (const variantProperty of variantsObject.properties) {
    if (!ts.isPropertyAssignment(variantProperty)) continue;
    const variantName = propertyNameText(variantProperty.name);
    if (!variantName) continue;
    const variantOptions = unwrapTransparentExpression(variantProperty.initializer);
    if (!variantOptions || !ts.isObjectLiteralExpression(variantOptions)) continue;

    const options = new Set<string>();
    for (const optionProperty of variantOptions.properties) {
      if (
        !ts.isPropertyAssignment(optionProperty) &&
        !ts.isShorthandPropertyAssignment(optionProperty)
      ) {
        continue;
      }
      const optionName = propertyNameText(optionProperty.name);
      if (optionName) {
        options.add(optionName);
      }
    }

    if (options.size > 0) {
      variants.set(variantName, options);
    }
  }

  return variants;
}

function collectRecipeCallReferences(
  arg: ts.Expression,
  recipe: RecipeBinding,
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
): void {
  const value = unwrapTransparentExpression(arg);
  if (!value || !ts.isObjectLiteralExpression(value)) return;

  for (const prop of value.properties) {
    if (!ts.isPropertyAssignment(prop)) continue;
    const variantName = propertyNameText(prop.name);
    if (!variantName) continue;
    const knownOptions = recipe.variants.get(variantName);
    if (!knownOptions) continue;

    collectVariantValueReferences(
      prop.initializer,
      recipe.localName,
      variantName,
      knownOptions,
      sourceFile,
      out,
      allocateId,
    );
  }
}

function collectVariantValueReferences(
  expression: ts.Expression,
  recipeName: string,
  variantName: string,
  knownOptions: ReadonlySet<string>,
  sourceFile: ts.SourceFile,
  out: DomainClassReferenceHIR[],
  allocateId: () => string,
): void {
  const value = unwrapTransparentExpression(expression);
  if (!value) return;

  if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
    if (!knownOptions.has(value.text)) return;
    out.push(
      makeDomainLiteralClassReference(
        allocateId(),
        PLUGIN_ID,
        DOMAIN,
        "classUtilityCall",
        recipeVariantKey(recipeName, variantName, value.text),
        innerStringRange(value, sourceFile),
      ),
    );
    return;
  }

  if (ts.isConditionalExpression(value)) {
    collectVariantValueReferences(
      value.whenTrue,
      recipeName,
      variantName,
      knownOptions,
      sourceFile,
      out,
      allocateId,
    );
    collectVariantValueReferences(
      value.whenFalse,
      recipeName,
      variantName,
      knownOptions,
      sourceFile,
      out,
      allocateId,
    );
    return;
  }

  if (ts.isTemplateExpression(value)) {
    const staticPrefix = value.head.text;
    if (staticPrefix.length === 0) return;
    out.push(
      makeDomainTemplateClassReference(
        allocateId(),
        PLUGIN_ID,
        DOMAIN,
        "classUtilityCall",
        value.getText(sourceFile),
        recipeVariantKey(recipeName, variantName, staticPrefix),
        rangeOfNode(value, sourceFile),
      ),
    );
  }
}

function findObjectProperty(
  object: ts.ObjectLiteralExpression,
  name: string,
): ts.Expression | null {
  for (const prop of object.properties) {
    if (!ts.isPropertyAssignment(prop)) continue;
    if (propertyNameText(prop.name) === name) {
      return prop.initializer;
    }
  }
  return null;
}

function propertyNameText(name: ts.PropertyName): string | null {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  return null;
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

function recipeVariantKey(recipeName: string, variantName: string, optionName: string): string {
  return `${recipeName}.${variantName}.${optionName}`;
}

function rangeOfNode(node: ts.Node, sourceFile: ts.SourceFile): Range {
  return rangeFromOffsets(sourceFile, node.getStart(sourceFile), node.getEnd());
}

function innerStringRange(
  node: ts.StringLiteral | ts.NoSubstitutionTemplateLiteral,
  sourceFile: ts.SourceFile,
): Range {
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
