import ts, { nodeText } from "../../ts-facade";
import {
  makeDomainLiteralClassReference,
  makeDomainTemplateClassReference,
  type DomainClassReferenceHIR,
} from "../hir/source-types";
import type { BinderPluginV0 } from "./binder-plugin";
import {
  collectVariantRecipeBindings,
  innerStringRange,
  makeVariantRecipeUniverseProviderV0,
  propertyNameText,
  rangeOfNode,
  unwrapTransparentExpression,
  type VariantRecipeBindingV0,
} from "./variant-recipe-universe";

const PLUGIN_ID = "vanilla-extract-recipe-domain";
const DOMAIN = "vanilla-extract-recipe";

interface RecipeBinding {
  readonly localName: string;
  readonly variants: ReadonlyMap<string, ReadonlySet<string>>;
}

export const vanillaExtractRecipeClassValueUniverseProviderV0 = makeVariantRecipeUniverseProviderV0(
  {
    pluginId: PLUGIN_ID,
    domain: DOMAIN,
    importSources: ["@vanilla-extract/recipes"],
    importNames: ["recipe"],
    callShape: "objectConfig",
  },
);

export const vanillaExtractRecipeBinderPluginV0: BinderPluginV0 = {
  id: PLUGIN_ID,
  version: "0",
  stability: "builtIn",
  domains: ["vanilla-extract-recipes"],
  importTargets: ["@vanilla-extract/recipes"],
  utilityTargets: ["recipe"],
  ownsSurfaces: ["domainClassReferenceExtraction", "classValueUniverseProvider"],
  analyzeSource(input) {
    return {
      pluginId: PLUGIN_ID,
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [],
      domainClassReferences: collectRecipeVariantReferences(input.sourceFile),
      classValueUniverses: vanillaExtractRecipeClassValueUniverseProviderV0.lookup(input),
    };
  },
};

function collectRecipeVariantReferences(
  sourceFile: ts.SourceFile,
): readonly DomainClassReferenceHIR[] {
  const recipeImportNames = collectRecipeImportNames(sourceFile);
  if (recipeImportNames.size === 0) return [];

  const recipes = collectRecipeBindings(sourceFile);
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

function collectRecipeBindings(sourceFile: ts.SourceFile): ReadonlyMap<string, RecipeBinding> {
  const bindings = new Map<string, RecipeBinding>();
  for (const binding of collectVariantRecipeBindings(sourceFile, {
    pluginId: PLUGIN_ID,
    domain: DOMAIN,
    importSources: ["@vanilla-extract/recipes"],
    importNames: ["recipe"],
    callShape: "objectConfig",
  })) {
    const variants = variantOptionsFromRecipeBinding(binding);
    if (variants.size > 0) {
      bindings.set(binding.localName, { localName: binding.localName, variants });
    }
  }
  return bindings;
}

function variantOptionsFromRecipeBinding(
  binding: VariantRecipeBindingV0,
): ReadonlyMap<string, ReadonlySet<string>> {
  const variants = new Map<string, ReadonlySet<string>>();
  for (const [variantName, options] of binding.variants) {
    variants.set(variantName, new Set(options.keys()));
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
        vanillaRecipeVariantKey(recipeName, variantName, value.text),
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
        nodeText(value, sourceFile),
        vanillaRecipeVariantKey(recipeName, variantName, staticPrefix),
        rangeOfNode(value, sourceFile),
      ),
    );
  }
}

function vanillaRecipeVariantKey(
  recipeName: string,
  variantName: string,
  optionName: string,
): string {
  return `${recipeName}.${variantName}.${optionName}`;
}
