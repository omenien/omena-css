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
  variantRecipeKey,
  type VariantRecipeBindingV0,
} from "./variant-recipe-universe";

const PLUGIN_ID = "cva-recipe-domain";
const DOMAIN = "cva-recipe";
const IMPORT_SOURCES = ["class-variance-authority", "cva"] as const;

interface CvaBinding {
  readonly localName: string;
  readonly variants: ReadonlyMap<string, ReadonlySet<string>>;
}

export const cvaRecipeClassValueUniverseProviderV0 = makeVariantRecipeUniverseProviderV0({
  pluginId: PLUGIN_ID,
  domain: DOMAIN,
  importSources: IMPORT_SOURCES,
  importNames: ["cva"],
  callShape: "baseThenConfig",
});

export const cvaRecipeBinderPluginV0: BinderPluginV0 = {
  id: PLUGIN_ID,
  version: "0",
  stability: "builtIn",
  domains: ["cva-recipes"],
  importTargets: IMPORT_SOURCES,
  utilityTargets: ["cva"],
  ownsSurfaces: ["domainClassReferenceExtraction", "classValueUniverseProvider"],
  analyzeSource(input) {
    return {
      pluginId: PLUGIN_ID,
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [],
      domainClassReferences: collectCvaReferences(input.sourceFile),
      classValueUniverses: cvaRecipeClassValueUniverseProviderV0.lookup(input),
    };
  },
};

function collectCvaReferences(sourceFile: ts.SourceFile): readonly DomainClassReferenceHIR[] {
  const cvaImportNames = collectCvaImportNames(sourceFile);
  if (cvaImportNames.size === 0) return [];

  const recipes = collectCvaBindings(sourceFile);
  if (recipes.size === 0) return [];

  const references: DomainClassReferenceHIR[] = [];
  let nextId = 0;
  const allocateId = () => `domain-class-ref:${PLUGIN_ID}:${nextId++}`;

  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const recipe = recipes.get(node.expression.text);
      const args = node.arguments;
      if (recipe && args.length > 0) {
        collectCvaCallReferences(args[0]!, recipe, sourceFile, references, allocateId);
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return references;
}

function collectCvaImportNames(sourceFile: ts.SourceFile): ReadonlySet<string> {
  const names = new Set<string>();
  const importSources = new Set<string>(IMPORT_SOURCES);

  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement)) continue;
    if (!ts.isStringLiteral(statement.moduleSpecifier)) continue;
    if (!importSources.has(statement.moduleSpecifier.text)) continue;

    const namedBindings = statement.importClause?.namedBindings;
    if (!namedBindings || !ts.isNamedImports(namedBindings)) continue;

    for (const element of namedBindings.elements) {
      const importedName = element.propertyName?.text ?? element.name.text;
      if (importedName === "cva") {
        names.add(element.name.text);
      }
    }
  }

  return names;
}

function collectCvaBindings(sourceFile: ts.SourceFile): ReadonlyMap<string, CvaBinding> {
  const bindings = new Map<string, CvaBinding>();
  for (const binding of collectVariantRecipeBindings(sourceFile, {
    pluginId: PLUGIN_ID,
    domain: DOMAIN,
    importSources: IMPORT_SOURCES,
    importNames: ["cva"],
    callShape: "baseThenConfig",
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

function collectCvaCallReferences(
  arg: ts.Expression,
  recipe: CvaBinding,
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
    collectCvaVariantValueReferences(
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

function collectCvaVariantValueReferences(
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
        variantRecipeKey(recipeName, variantName, value.text),
        innerStringRange(value, sourceFile),
      ),
    );
    return;
  }

  if (ts.isConditionalExpression(value)) {
    collectCvaVariantValueReferences(
      value.whenTrue,
      recipeName,
      variantName,
      knownOptions,
      sourceFile,
      out,
      allocateId,
    );
    collectCvaVariantValueReferences(
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
        variantRecipeKey(recipeName, variantName, staticPrefix),
        rangeOfNode(value, sourceFile),
      ),
    );
  }
}
