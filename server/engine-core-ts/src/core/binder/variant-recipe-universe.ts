import ts from "typescript";
import type { Range } from "@omena/shared";
import {
  reducedProductClassValueUniverseV0,
  type ClassValueUniverseAxisV0,
  type ClassValueUniverseConditionV0,
} from "../abstract-value/class-value-universe";
import {
  type ClassValueUniverseEntryV0,
  type ClassValueUniverseProviderV0,
} from "./class-value-universe-provider";

export interface VariantRecipeBindingV0 {
  readonly localName: string;
  readonly domain: string;
  readonly baseClassNames: readonly string[];
  readonly variants: ReadonlyMap<string, ReadonlyMap<string, readonly string[]>>;
  readonly compoundVariants: readonly VariantRecipeCompoundVariantV0[];
  readonly defaultVariants: ReadonlyMap<string, string>;
  readonly range: Range;
}

export interface VariantRecipeCompoundVariantV0 {
  readonly conditions: readonly ClassValueUniverseConditionV0[];
  readonly classNames: readonly string[];
}

export interface VariantRecipeCollectorConfigV0 {
  readonly pluginId: string;
  readonly domain: string;
  readonly importSources: readonly string[];
  readonly importNames: readonly string[];
  readonly callShape: "objectConfig" | "baseThenConfig";
}

export function makeVariantRecipeUniverseProviderV0(
  config: VariantRecipeCollectorConfigV0,
): ClassValueUniverseProviderV0 {
  return {
    pluginId: config.pluginId,
    version: "0",
    stability: "builtIn",
    lookup(args) {
      return collectVariantRecipeBindings(args.sourceFile, config).map((binding, index) =>
        recipeBindingToUniverseEntry(config.pluginId, binding, index),
      );
    },
  };
}

export function collectVariantRecipeBindings(
  sourceFile: ts.SourceFile,
  config: VariantRecipeCollectorConfigV0,
): readonly VariantRecipeBindingV0[] {
  const importNames = collectImportedCallNames(sourceFile, config);
  if (importNames.size === 0) return [];

  const bindings: VariantRecipeBindingV0[] = [];

  function visit(node: ts.Node): void {
    if (!ts.isVariableDeclaration(node) || !ts.isIdentifier(node.name)) {
      ts.forEachChild(node, visit);
      return;
    }
    const initializer = unwrapTransparentExpression(node.initializer);
    if (
      !initializer ||
      !ts.isCallExpression(initializer) ||
      !ts.isIdentifier(initializer.expression) ||
      !importNames.has(initializer.expression.text)
    ) {
      ts.forEachChild(node, visit);
      return;
    }

    const binding = parseVariantRecipeBinding(node.name.text, initializer, sourceFile, config);
    if (binding) {
      bindings.push(binding);
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return bindings;
}

export function variantRecipeKey(
  recipeName: string,
  variantName: string,
  optionName: string,
): string {
  return `${recipeName}.${variantName}.${optionName}`;
}

export function objectPropertyExpression(
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

export function propertyNameText(name: ts.PropertyName): string | null {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  return null;
}

export function unwrapTransparentExpression<T extends ts.Node | null | undefined>(
  node: T,
): ts.Expression | null {
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

export function rangeOfNode(node: ts.Node, sourceFile: ts.SourceFile): Range {
  return rangeFromOffsets(sourceFile, node.getStart(sourceFile), node.getEnd());
}

export function innerStringRange(
  node: ts.StringLiteral | ts.NoSubstitutionTemplateLiteral,
  sourceFile: ts.SourceFile,
): Range {
  return rangeFromOffsets(sourceFile, node.getStart(sourceFile) + 1, node.getEnd() - 1);
}

export function rangeFromOffsets(
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

function collectImportedCallNames(
  sourceFile: ts.SourceFile,
  config: VariantRecipeCollectorConfigV0,
): ReadonlySet<string> {
  const names = new Set<string>();
  const sourceSet = new Set(config.importSources);
  const importedNameSet = new Set(config.importNames);

  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement)) continue;
    if (!ts.isStringLiteral(statement.moduleSpecifier)) continue;
    if (!sourceSet.has(statement.moduleSpecifier.text)) continue;

    const namedBindings = statement.importClause?.namedBindings;
    if (!namedBindings || !ts.isNamedImports(namedBindings)) continue;

    for (const element of namedBindings.elements) {
      const importedName = element.propertyName?.text ?? element.name.text;
      if (importedNameSet.has(importedName)) {
        names.add(element.name.text);
      }
    }
  }

  return names;
}

function parseVariantRecipeBinding(
  localName: string,
  call: ts.CallExpression,
  sourceFile: ts.SourceFile,
  config: VariantRecipeCollectorConfigV0,
): VariantRecipeBindingV0 | null {
  const configArg = config.callShape === "objectConfig" ? call.arguments[0] : call.arguments[1];
  const configObject = unwrapTransparentExpression(configArg);
  if (!configObject || !ts.isObjectLiteralExpression(configObject)) return null;

  const baseClassNames = parseBaseClassNames(localName, call, configObject, config);
  const variants = parseVariants(configObject, localName);
  const compoundVariants = parseCompoundVariants(configObject, localName);
  const defaultVariants = parseDefaultVariants(configObject);
  if (
    baseClassNames.length === 0 &&
    variants.size === 0 &&
    compoundVariants.length === 0 &&
    defaultVariants.size === 0
  ) {
    return null;
  }

  return {
    localName,
    domain: config.domain,
    baseClassNames,
    variants,
    compoundVariants,
    defaultVariants,
    range: rangeOfNode(call, sourceFile),
  };
}

function parseBaseClassNames(
  localName: string,
  call: ts.CallExpression,
  configObject: ts.ObjectLiteralExpression,
  config: VariantRecipeCollectorConfigV0,
): readonly string[] {
  if (config.callShape === "baseThenConfig") {
    return classNamesFromExpression(call.arguments[0], localName);
  }
  const baseExpression = objectPropertyExpression(configObject, "base");
  return baseExpression ? classNamesFromExpression(baseExpression, localName) : [];
}

function parseVariants(
  configObject: ts.ObjectLiteralExpression,
  recipeName: string,
): ReadonlyMap<string, ReadonlyMap<string, readonly string[]>> {
  const variants = new Map<string, ReadonlyMap<string, readonly string[]>>();
  const variantsExpression = unwrapTransparentExpression(
    objectPropertyExpression(configObject, "variants"),
  );
  if (!variantsExpression || !ts.isObjectLiteralExpression(variantsExpression)) return variants;

  for (const variantProperty of variantsExpression.properties) {
    if (!ts.isPropertyAssignment(variantProperty)) continue;
    const variantName = propertyNameText(variantProperty.name);
    if (!variantName) continue;
    const optionsObject = unwrapTransparentExpression(variantProperty.initializer);
    if (!optionsObject || !ts.isObjectLiteralExpression(optionsObject)) continue;

    const options = new Map<string, readonly string[]>();
    for (const optionProperty of optionsObject.properties) {
      if (
        !ts.isPropertyAssignment(optionProperty) &&
        !ts.isShorthandPropertyAssignment(optionProperty)
      ) {
        continue;
      }
      const optionName = propertyNameText(optionProperty.name);
      if (!optionName) continue;
      const initializer = ts.isPropertyAssignment(optionProperty)
        ? optionProperty.initializer
        : undefined;
      const classNames = classNamesFromExpression(
        initializer,
        variantRecipeKey(recipeName, variantName, optionName),
      );
      options.set(optionName, classNames);
    }

    if (options.size > 0) {
      variants.set(variantName, options);
    }
  }

  return variants;
}

function parseCompoundVariants(
  configObject: ts.ObjectLiteralExpression,
  recipeName: string,
): readonly VariantRecipeCompoundVariantV0[] {
  const expression = unwrapTransparentExpression(
    objectPropertyExpression(configObject, "compoundVariants"),
  );
  if (!expression || !ts.isArrayLiteralExpression(expression)) return [];

  const compounds: VariantRecipeCompoundVariantV0[] = [];
  for (const element of expression.elements) {
    const compound = unwrapTransparentExpression(element);
    if (!compound || !ts.isObjectLiteralExpression(compound)) continue;
    const conditions = parseCompoundConditions(compound);
    const classNames = classNamesFromCompound(compound, recipeName, conditions);
    if (conditions.length > 0 || classNames.length > 0) {
      compounds.push({ conditions, classNames });
    }
  }
  return compounds;
}

function parseCompoundConditions(
  compound: ts.ObjectLiteralExpression,
): readonly ClassValueUniverseConditionV0[] {
  const nested = unwrapTransparentExpression(objectPropertyExpression(compound, "variants"));
  const conditionObject = nested && ts.isObjectLiteralExpression(nested) ? nested : compound;
  const conditions: ClassValueUniverseConditionV0[] = [];
  for (const prop of conditionObject.properties) {
    if (!ts.isPropertyAssignment(prop)) continue;
    const axisName = propertyNameText(prop.name);
    if (!axisName || axisName === "class" || axisName === "className" || axisName === "style") {
      continue;
    }
    const value = stringValue(prop.initializer);
    if (value) {
      conditions.push({ axisName, value });
    }
  }
  return conditions.toSorted((left, right) =>
    `${left.axisName}:${left.value}`.localeCompare(`${right.axisName}:${right.value}`),
  );
}

function parseDefaultVariants(
  configObject: ts.ObjectLiteralExpression,
): ReadonlyMap<string, string> {
  const defaults = new Map<string, string>();
  const expression = unwrapTransparentExpression(
    objectPropertyExpression(configObject, "defaultVariants"),
  );
  if (!expression || !ts.isObjectLiteralExpression(expression)) return defaults;
  for (const prop of expression.properties) {
    if (!ts.isPropertyAssignment(prop)) continue;
    const axisName = propertyNameText(prop.name);
    const value = stringValue(prop.initializer);
    if (axisName && value) {
      defaults.set(axisName, value);
    }
  }
  return defaults;
}

function classNamesFromCompound(
  compound: ts.ObjectLiteralExpression,
  recipeName: string,
  conditions: readonly ClassValueUniverseConditionV0[],
): readonly string[] {
  const styleExpression = objectPropertyExpression(compound, "style");
  return [
    ...classNamesFromExpression(objectPropertyExpression(compound, "class")),
    ...classNamesFromExpression(objectPropertyExpression(compound, "className")),
    ...(styleExpression
      ? classNamesFromExpression(styleExpression, compoundClassNameFallback(recipeName, conditions))
      : []),
  ];
}

function compoundClassNameFallback(
  recipeName: string,
  conditions: readonly ClassValueUniverseConditionV0[],
): string | undefined {
  if (conditions.length === 0) return undefined;
  return `${recipeName}.compound.${conditions
    .map((condition) => `${condition.axisName}.${condition.value}`)
    .join(".")}`;
}

function classNamesFromExpression(
  expression: ts.Node | undefined | null,
  fallback?: string,
): readonly string[] {
  const value = unwrapTransparentExpression(expression ?? undefined);
  if (!value) return fallback ? [fallback] : [];
  if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
    return splitClassNames(value.text, fallback);
  }
  if (ts.isArrayLiteralExpression(value)) {
    const values = value.elements.flatMap((element) => classNamesFromExpression(element));
    return values.length > 0 ? values : fallback ? [fallback] : [];
  }
  return fallback ? [fallback] : [];
}

function splitClassNames(value: string, fallback?: string): readonly string[] {
  const classNames = value.split(/\s+/).filter((part) => part.length > 0);
  return classNames.length > 0 ? classNames : fallback ? [fallback] : [];
}

function stringValue(expression: ts.Expression): string | null {
  const value = unwrapTransparentExpression(expression);
  return value && (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value))
    ? value.text
    : null;
}

function recipeBindingToUniverseEntry(
  pluginId: string,
  binding: VariantRecipeBindingV0,
  index: number,
): ClassValueUniverseEntryV0 {
  return {
    id: `class-value-universe:${pluginId}:${index}`,
    pluginId,
    domain: binding.domain,
    ownerName: binding.localName,
    range: binding.range,
    universe: reducedProductClassValueUniverseV0({
      baseClassNames: binding.baseClassNames,
      axes: [
        ...Array.from(binding.variants.entries(), ([axisName, options]) =>
          recipeVariantAxis(axisName, options, binding.defaultVariants.get(axisName)),
        ),
        { axisName: "slots", values: [], role: "slot", reserved: true },
      ],
      compoundVariants: binding.compoundVariants,
    }),
  };
}

function recipeVariantAxis(
  axisName: string,
  options: ReadonlyMap<string, readonly string[]>,
  defaultValue: string | undefined,
): ClassValueUniverseAxisV0 {
  return {
    axisName,
    values: Array.from(options.entries(), ([name, classNames]) => ({
      name,
      classNames,
    })).toSorted((left, right) => left.name.localeCompare(right.name)),
    ...(defaultValue ? { defaultValue } : {}),
    role: "variant",
  };
}
