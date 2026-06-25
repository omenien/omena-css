import * as path from "node:path";
import ts, { nodeText } from "../../ts-facade";

export interface BundlerPathAliases {
  readonly aliases: Readonly<Record<string, string>>;
  readonly unrecognized: readonly BundlerAliasUnrecognizedEntry[];
}

export interface BundlerAliasUnrecognizedEntry {
  readonly configPath: string;
  readonly reason: BundlerAliasUnrecognizedReason;
  readonly text: string;
}

export type BundlerAliasUnrecognizedReason =
  | "dynamic-alias-container"
  | "dynamic-alias-entry"
  | "regex-alias-find"
  | "dynamic-alias-replacement";

interface BundlerAliasExtractionSystem extends Pick<typeof ts.sys, "fileExists" | "readFile"> {}

const BUNDLER_CONFIG_NAMES = [
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.cts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cjs",
  "webpack.config.ts",
  "webpack.config.mts",
  "webpack.config.cts",
  "webpack.config.js",
  "webpack.config.mjs",
  "webpack.config.cjs",
] as const;

export function loadWorkspaceBundlerPathAliases(
  workspaceRoot: string,
  system: BundlerAliasExtractionSystem = {
    fileExists: ts.sys.fileExists,
    readFile: ts.sys.readFile,
  },
): BundlerPathAliases | null {
  const aliases: Record<string, string> = {};
  const unrecognized: BundlerAliasUnrecognizedEntry[] = [];

  for (const configPath of findWorkspaceBundlerConfigPaths(workspaceRoot, system)) {
    const sourceText = system.readFile(configPath);
    if (sourceText === undefined) continue;
    const result = extractBundlerPathAliasesFromConfig(configPath, sourceText);
    Object.assign(aliases, result.aliases);
    unrecognized.push(...result.unrecognized);
  }

  return Object.keys(aliases).length > 0 || unrecognized.length > 0
    ? { aliases, unrecognized }
    : null;
}

export function extractBundlerPathAliasesFromConfig(
  configPath: string,
  sourceText: string,
): BundlerPathAliases {
  const sourceFile = ts.createSourceFile(
    configPath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    scriptKindForConfigPath(configPath),
  );
  const topLevelObjectLiterals = collectTopLevelObjectLiterals(sourceFile);
  const aliases: Record<string, string> = {};
  const unrecognized: BundlerAliasUnrecognizedEntry[] = [];

  for (const expression of exportedConfigExpressions(sourceFile, topLevelObjectLiterals)) {
    const objectExpression = unwrapConfigExpression(expression, topLevelObjectLiterals);
    if (!objectExpression) {
      pushUnrecognized(configPath, "dynamic-alias-container", expression, unrecognized);
      continue;
    }
    const aliasExpression = getResolveAliasExpression(objectExpression, configPath, unrecognized);
    if (!aliasExpression) {
      continue;
    }
    collectAliasExpression(configPath, aliasExpression, aliases, unrecognized);
  }

  return { aliases, unrecognized };
}

function findWorkspaceBundlerConfigPaths(
  workspaceRoot: string,
  system: Pick<typeof ts.sys, "fileExists">,
): readonly string[] {
  return BUNDLER_CONFIG_NAMES.map((name) => path.join(workspaceRoot, name)).filter((configPath) =>
    system.fileExists(configPath),
  );
}

function scriptKindForConfigPath(configPath: string): ts.ScriptKind {
  if (configPath.endsWith(".ts") || configPath.endsWith(".mts") || configPath.endsWith(".cts")) {
    return ts.ScriptKind.TS;
  }
  return ts.ScriptKind.JS;
}

function collectTopLevelObjectLiterals(
  sourceFile: ts.SourceFile,
): ReadonlyMap<string, ts.ObjectLiteralExpression> {
  const literals = new Map<string, ts.ObjectLiteralExpression>();
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (!ts.isIdentifier(declaration.name) || !declaration.initializer) continue;
      const initializer = unwrapConfigExpression(declaration.initializer, literals);
      if (initializer) literals.set(declaration.name.text, initializer);
    }
  }
  return literals;
}

function exportedConfigExpressions(
  sourceFile: ts.SourceFile,
  topLevelObjectLiterals: ReadonlyMap<string, ts.ObjectLiteralExpression>,
): readonly ts.Expression[] {
  const expressions: ts.Expression[] = [];
  for (const statement of sourceFile.statements) {
    if (ts.isExportAssignment(statement)) {
      expressions.push(statement.expression);
      continue;
    }
    if (!ts.isExpressionStatement(statement)) continue;
    const expression = statement.expression;
    if (
      !ts.isBinaryExpression(expression) ||
      expression.operatorToken.kind !== ts.SyntaxKind.EqualsToken
    ) {
      continue;
    }
    if (isModuleExportsExpression(expression.left)) {
      expressions.push(expression.right);
    }
  }

  if (expressions.length > 0) return expressions;
  return [...topLevelObjectLiterals.values()];
}

function isModuleExportsExpression(expression: ts.Expression): boolean {
  if (!ts.isPropertyAccessExpression(expression)) return false;
  return (
    ts.isIdentifier(expression.expression) &&
    expression.expression.text === "module" &&
    expression.name.text === "exports"
  );
}

function unwrapConfigExpression(
  expression: ts.Expression,
  topLevelObjectLiterals: ReadonlyMap<string, ts.ObjectLiteralExpression>,
): ts.ObjectLiteralExpression | null {
  let current = skipParens(expression);
  if (ts.isObjectLiteralExpression(current)) return current;
  if (ts.isIdentifier(current)) return topLevelObjectLiterals.get(current.text) ?? null;
  if (ts.isSatisfiesExpression(current) || ts.isAsExpression(current)) {
    return unwrapConfigExpression(current.expression, topLevelObjectLiterals);
  }
  if (ts.isCallExpression(current) && current.arguments.length > 0) {
    const callee = skipParens(current.expression);
    if (ts.isIdentifier(callee) && callee.text === "defineConfig") {
      return unwrapConfigExpression(current.arguments[0]!, topLevelObjectLiterals);
    }
  }
  return null;
}

function skipParens(expression: ts.Expression): ts.Expression {
  let current = expression;
  while (ts.isParenthesizedExpression(current)) {
    current = current.expression;
  }
  return current;
}

function getResolveAliasExpression(
  objectExpression: ts.ObjectLiteralExpression,
  configPath: string,
  unrecognized: BundlerAliasUnrecognizedEntry[],
): ts.Expression | null {
  const resolveExpression = getObjectPropertyValue(objectExpression, "resolve");
  if (!resolveExpression) return null;
  const resolveValue = skipParens(resolveExpression);
  if (!ts.isObjectLiteralExpression(resolveValue)) {
    pushUnrecognized(configPath, "dynamic-alias-container", resolveValue, unrecognized);
    return null;
  }
  return getObjectPropertyValue(resolveValue, "alias");
}

function getObjectPropertyValue(
  objectExpression: ts.ObjectLiteralExpression,
  name: string,
): ts.Expression | null {
  for (const property of objectExpression.properties) {
    if (!ts.isPropertyAssignment(property)) continue;
    if (propertyNameText(property.name) !== name) continue;
    return property.initializer;
  }
  return null;
}

function propertyNameText(name: ts.PropertyName): string | null {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  return null;
}

function collectAliasExpression(
  configPath: string,
  expression: ts.Expression,
  aliases: Record<string, string>,
  unrecognized: BundlerAliasUnrecognizedEntry[],
): void {
  const value = skipParens(expression);
  if (ts.isObjectLiteralExpression(value)) {
    collectObjectAliasEntries(configPath, value, aliases, unrecognized);
    return;
  }
  if (ts.isArrayLiteralExpression(value)) {
    collectArrayAliasEntries(configPath, value, aliases, unrecognized);
    return;
  }
  pushUnrecognized(configPath, "dynamic-alias-container", value, unrecognized);
}

function collectObjectAliasEntries(
  configPath: string,
  objectExpression: ts.ObjectLiteralExpression,
  aliases: Record<string, string>,
  unrecognized: BundlerAliasUnrecognizedEntry[],
): void {
  for (const property of objectExpression.properties) {
    if (!ts.isPropertyAssignment(property)) {
      pushUnrecognized(configPath, "dynamic-alias-entry", property, unrecognized);
      continue;
    }
    const key = propertyNameText(property.name);
    if (!key) {
      pushUnrecognized(configPath, "dynamic-alias-entry", property, unrecognized);
      continue;
    }
    const replacement = staticReplacementPath(configPath, property.initializer);
    if (!replacement) {
      pushUnrecognized(configPath, "dynamic-alias-replacement", property.initializer, unrecognized);
      continue;
    }
    aliases[key] = replacement;
  }
}

function collectArrayAliasEntries(
  configPath: string,
  arrayExpression: ts.ArrayLiteralExpression,
  aliases: Record<string, string>,
  unrecognized: BundlerAliasUnrecognizedEntry[],
): void {
  for (const element of arrayExpression.elements) {
    const value = skipParens(element);
    if (!ts.isObjectLiteralExpression(value)) {
      pushUnrecognized(configPath, "dynamic-alias-entry", value, unrecognized);
      continue;
    }
    const find = getObjectPropertyValue(value, "find");
    const replacement = getObjectPropertyValue(value, "replacement");
    if (!find || !replacement) {
      pushUnrecognized(configPath, "dynamic-alias-entry", value, unrecognized);
      continue;
    }
    if (ts.isRegularExpressionLiteral(find)) {
      pushUnrecognized(configPath, "regex-alias-find", find, unrecognized);
      continue;
    }
    const key = staticString(find);
    if (!key) {
      pushUnrecognized(configPath, "dynamic-alias-entry", find, unrecognized);
      continue;
    }
    const target = staticReplacementPath(configPath, replacement);
    if (!target) {
      pushUnrecognized(configPath, "dynamic-alias-replacement", replacement, unrecognized);
      continue;
    }
    aliases[key] = target;
  }
}

function staticReplacementPath(configPath: string, expression: ts.Expression): string | null {
  const value = staticString(expression);
  if (value !== null) {
    return path.isAbsolute(value) ? value : path.resolve(path.dirname(configPath), value);
  }
  const resolvedCall = staticPathCall(configPath, expression);
  return resolvedCall;
}

function staticString(expression: ts.Expression): string | null {
  const value = skipParens(expression);
  return ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value) ? value.text : null;
}

function staticPathCall(configPath: string, expression: ts.Expression): string | null {
  const value = skipParens(expression);
  if (!ts.isCallExpression(value) || !ts.isPropertyAccessExpression(value.expression)) {
    return null;
  }
  const callee = value.expression;
  if (!ts.isIdentifier(callee.expression) || callee.expression.text !== "path") {
    return null;
  }
  if (callee.name.text !== "resolve" && callee.name.text !== "join") {
    return null;
  }
  const parts = value.arguments.map((argument) => staticPathSegment(configPath, argument));
  if (parts.some((part) => part === null)) return null;
  return path.resolve(...(parts as string[]));
}

function staticPathSegment(configPath: string, expression: ts.Expression): string | null {
  const value = skipParens(expression);
  if (ts.isIdentifier(value) && value.text === "__dirname") {
    return path.dirname(configPath);
  }
  return staticString(value);
}

function pushUnrecognized(
  configPath: string,
  reason: BundlerAliasUnrecognizedReason,
  node: ts.Node,
  unrecognized: BundlerAliasUnrecognizedEntry[],
): void {
  unrecognized.push({
    configPath,
    reason,
    text: nodeText(node),
  });
}
