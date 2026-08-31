import { readFileSync } from "node:fs";
import path from "node:path";
import { compilerApi as ts } from "../../../../server/engine-core-ts/src/ts-facade";
import type tsTypes from "../../../../server/engine-core-ts/src/ts-facade";
import { resolveScanSurface, type ScanSurfaceSpec } from "./scan-surface";

export const SCANNER_DETECTION_PATTERN =
  /ls-files|readdirSync|readdir\(|opendirSync|globSync|fast-glob|globby|trackedProductionSources/u;

const LAYER_ONE_CALLEES = new Set([
  "readdirSync",
  "readdir",
  "opendirSync",
  "globSync",
  "trackedProductionSources",
]);
const EXEC_FAMILY_CALLEES = new Set([
  "exec",
  "execSync",
  "execFileSync",
  "spawnSync",
  "run",
  "git",
]);
const MODULE_SCANNER_NAMES = new Set(["fast-glob", "globby", "glob"]);
const FILESYSTEM_MODULE_NAMES = new Set(["node:fs", "fs", "node:fs/promises", "fs/promises"]);
const CHILD_PROCESS_MODULE_NAMES = new Set(["node:child_process", "child_process"]);
const SURFACE_RESOLVER_EXPORT = "resolveScanSurfaceForScanner";

interface ScannerBindings {
  readonly directEnumerationBindings: ReadonlySet<string>;
  readonly filesystemModuleBindings: ReadonlySet<string>;
  readonly enumerationModuleBindings: ReadonlySet<string>;
  readonly childProcessBindings: ReadonlySet<string>;
  readonly childProcessModuleBindings: ReadonlySet<string>;
  readonly resolverBindings: ReadonlySet<string>;
  readonly surfaceFactoryBindings: ReadonlySet<string>;
  readonly surfaceObjectBindings: ReadonlySet<string>;
  readonly typedSurfaceParameterBindings: ReadonlySet<string>;
}

export type ScannerCallSiteLayer = "layer-1" | "layer-2a" | "layer-2b" | "tertiary";

export interface ScannerCallSite {
  readonly layer: ScannerCallSiteLayer;
  readonly line: number;
  readonly text: string;
}

export interface ScannerDetection {
  readonly scannerPath: string;
  readonly rawMatchCount: number;
  readonly callSites: readonly ScannerCallSite[];
  readonly tertiaryExecFamilySites: readonly ScannerCallSite[];
}

export interface ScannerRoutingDiagnostic {
  readonly line: number;
  readonly message: string;
}

export function detectScannerFiles(
  repoRoot: string,
  detectionSurface: ScanSurfaceSpec,
): readonly ScannerDetection[] {
  const candidates = resolveScanSurface(detectionSurface, { repoRoot }).paths.filter((candidate) =>
    /\.(?:[cm]?js|ts|tsx)$/u.test(candidate),
  );
  return candidates
    .map((scannerPath) =>
      analyzeScannerSource(scannerPath, readFileSync(path.join(repoRoot, scannerPath), "utf8")),
    )
    .filter((result) => result.rawMatchCount > 0)
    .toSorted((left, right) => compareText(left.scannerPath, right.scannerPath));
}

export function analyzeScannerSource(scannerPath: string, sourceText: string): ScannerDetection {
  const sourceFile = ts.createSourceFile(
    scannerPath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    scannerPath.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const bindings = scannerBindings(sourceFile);
  const callSites: ScannerCallSite[] = [];
  const tertiaryExecFamilySites: ScannerCallSite[] = [];

  const visit = (node: tsTypes.Node): void => {
    if (ts.isCallExpression(node)) {
      const layer = enumerationCallLayer(node.expression, bindings);
      if (layer) callSites.push(siteFor(sourceFile, node, layer));
    }
    if (ts.isStringLiteral(node) && node.text === "ls-files") {
      callSites.push(siteFor(sourceFile, node, "layer-2a"));
    }
    if (
      (ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateExpression(node)) &&
      node.getText(sourceFile).includes("ls-files")
    ) {
      const enclosingCall = enclosingArgumentCall(node);
      if (enclosingCall && isExecFamilyExpression(enclosingCall.expression, bindings)) {
        tertiaryExecFamilySites.push(siteFor(sourceFile, node, "tertiary"));
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);

  return {
    scannerPath,
    rawMatchCount:
      sourceText.match(new RegExp(SCANNER_DETECTION_PATTERN.source, "gu"))?.length ?? 0,
    callSites: dedupeSites(callSites),
    tertiaryExecFamilySites: dedupeSites(tertiaryExecFamilySites),
  };
}

export function findUnroutedScannerCallSites(
  scannerPath: string,
  sourceText: string,
): readonly ScannerRoutingDiagnostic[] {
  if (
    scannerPath === "packages/check-orchestrator/src/evidence/scan-surface.ts" ||
    scannerPath === "packages/check-orchestrator/src/evidence/scanner-analysis.ts"
  ) {
    return [];
  }
  const sourceFile = ts.createSourceFile(
    scannerPath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    scannerPath.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const bindings = scannerBindings(sourceFile);
  const helperRoutes = routedTrackedProductionSourceHelpers(sourceFile, bindings);
  const diagnostics: ScannerRoutingDiagnostic[] = [];
  const visit = (node: tsTypes.Node): void => {
    if (ts.isCallExpression(node)) {
      const callee = callExpressionName(node.expression);
      const layer = enumerationCallLayer(node.expression, bindings);
      if (
        layer &&
        !isSurfaceRoutedExpression(node.expression, bindings) &&
        !(callee === "trackedProductionSources" && helperRoutes.has(callee))
      ) {
        diagnostics.push(
          routingDiagnostic(
            sourceFile,
            node,
            layer === "layer-2b"
              ? "module glob call bypasses resolveScanSurface"
              : `direct ${callee ?? "enumeration"} call`,
          ),
        );
      }
    }
    if (ts.isStringLiteral(node) && node.text === "ls-files") {
      const enclosingCall = enclosingArgumentCall(node);
      if (!enclosingCall || !isSurfaceRoutedExpression(enclosingCall.expression, bindings)) {
        diagnostics.push(
          routingDiagnostic(sourceFile, node, "git ls-files token bypasses resolveScanSurface"),
        );
      }
    }
    if (
      (ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateExpression(node)) &&
      node.getText(sourceFile).includes("ls-files")
    ) {
      const enclosingCall = enclosingArgumentCall(node);
      if (enclosingCall && isExecFamilyExpression(enclosingCall.expression, bindings)) {
        diagnostics.push(
          routingDiagnostic(sourceFile, node, "exec-family template git enumeration residue"),
        );
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return diagnostics;
}

function scannerBindings(sourceFile: tsTypes.SourceFile): ScannerBindings {
  const directEnumerationBindings = new Set<string>();
  const filesystemModuleBindings = new Set<string>();
  const enumerationModuleBindings = new Set<string>();
  const childProcessBindings = new Set<string>();
  const childProcessModuleBindings = new Set<string>();
  const resolverBindings = new Set<string>();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    const moduleName = statement.moduleSpecifier.text;
    const importClause = statement.importClause;
    if (importClause?.name) {
      if (MODULE_SCANNER_NAMES.has(moduleName)) {
        enumerationModuleBindings.add(importClause.name.text);
      }
      if (FILESYSTEM_MODULE_NAMES.has(moduleName)) {
        filesystemModuleBindings.add(importClause.name.text);
      }
      if (CHILD_PROCESS_MODULE_NAMES.has(moduleName)) {
        childProcessModuleBindings.add(importClause.name.text);
      }
    }
    const namedBindings = importClause?.namedBindings;
    if (namedBindings && ts.isNamespaceImport(namedBindings)) {
      if (MODULE_SCANNER_NAMES.has(moduleName)) {
        enumerationModuleBindings.add(namedBindings.name.text);
      }
      if (FILESYSTEM_MODULE_NAMES.has(moduleName)) {
        filesystemModuleBindings.add(namedBindings.name.text);
      }
      if (CHILD_PROCESS_MODULE_NAMES.has(moduleName)) {
        childProcessModuleBindings.add(namedBindings.name.text);
      }
    }
    if (namedBindings && ts.isNamedImports(namedBindings)) {
      for (const element of namedBindings.elements) {
        const importedName = element.propertyName?.text ?? element.name.text;
        const localName = element.name.text;
        if (FILESYSTEM_MODULE_NAMES.has(moduleName) && LAYER_ONE_CALLEES.has(importedName)) {
          directEnumerationBindings.add(localName);
        }
        if (MODULE_SCANNER_NAMES.has(moduleName)) {
          enumerationModuleBindings.add(localName);
        }
        if (CHILD_PROCESS_MODULE_NAMES.has(moduleName) && EXEC_FAMILY_CALLEES.has(importedName)) {
          childProcessBindings.add(localName);
        }
        if (isSurfaceResolverModule(moduleName) && importedName === SURFACE_RESOLVER_EXPORT) {
          resolverBindings.add(localName);
        }
      }
    }
  }

  const surfaceFactoryBindings = new Set<string>();
  let changed = true;
  while (changed) {
    changed = false;
    const inspectFactory = (node: tsTypes.Node): void => {
      if (
        ts.isFunctionDeclaration(node) &&
        node.name &&
        node.body &&
        functionReturnsSurface(node.body, resolverBindings, surfaceFactoryBindings) &&
        !surfaceFactoryBindings.has(node.name.text)
      ) {
        surfaceFactoryBindings.add(node.name.text);
        changed = true;
      }
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        (ts.isArrowFunction(node.initializer) || ts.isFunctionExpression(node.initializer)) &&
        functionLikeReturnsSurface(node.initializer, resolverBindings, surfaceFactoryBindings) &&
        !surfaceFactoryBindings.has(node.name.text)
      ) {
        surfaceFactoryBindings.add(node.name.text);
        changed = true;
      }
      ts.forEachChild(node, inspectFactory);
    };
    inspectFactory(sourceFile);
  }

  const surfaceObjectBindings = new Set<string>();
  changed = true;
  while (changed) {
    changed = false;
    const inspectSurfaceObject = (node: tsTypes.Node): void => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        expressionProducesSurface(
          node.initializer,
          resolverBindings,
          surfaceFactoryBindings,
          surfaceObjectBindings,
        ) &&
        !surfaceObjectBindings.has(node.name.text)
      ) {
        surfaceObjectBindings.add(node.name.text);
        changed = true;
      }
      ts.forEachChild(node, inspectSurfaceObject);
    };
    inspectSurfaceObject(sourceFile);
  }

  const typedSurfaceParameterBindings = new Set<string>();
  const inspectParameters = (node: tsTypes.Node): void => {
    if (ts.isParameter(node) && ts.isIdentifier(node.name) && node.type) {
      const typeText = node.type.getText(sourceFile);
      if ([...resolverBindings].some((binding) => typeText.includes(binding))) {
        typedSurfaceParameterBindings.add(node.name.text);
      }
    }
    ts.forEachChild(node, inspectParameters);
  };
  inspectParameters(sourceFile);

  return {
    directEnumerationBindings,
    filesystemModuleBindings,
    enumerationModuleBindings,
    childProcessBindings,
    childProcessModuleBindings,
    resolverBindings,
    surfaceFactoryBindings,
    surfaceObjectBindings,
    typedSurfaceParameterBindings,
  };
}

function routedTrackedProductionSourceHelpers(
  sourceFile: tsTypes.SourceFile,
  bindings: ScannerBindings,
): ReadonlySet<string> {
  const routed = new Set<string>();
  const visit = (node: tsTypes.Node): void => {
    if (
      ts.isFunctionDeclaration(node) &&
      node.name?.text === "trackedProductionSources" &&
      node.body &&
      nodeContainsRoutedEnumeration(node.body, bindings)
    ) {
      routed.add(node.name.text);
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return routed;
}

function isSurfaceRoutedExpression(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): boolean {
  const root = expressionRoot(expression);
  if (ts.isCallExpression(root)) {
    const factory = callExpressionName(root.expression);
    return (
      factory !== null &&
      (bindings.resolverBindings.has(factory) || bindings.surfaceFactoryBindings.has(factory))
    );
  }
  return (
    ts.isIdentifier(root) &&
    (bindings.surfaceObjectBindings.has(root.text) ||
      bindings.typedSurfaceParameterBindings.has(root.text))
  );
}

function enumerationCallLayer(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): Extract<ScannerCallSiteLayer, "layer-1" | "layer-2b"> | null {
  if (usesModuleAlias(expression, bindings.enumerationModuleBindings)) return "layer-2b";
  const callee = callExpressionName(expression);
  if (
    (callee && LAYER_ONE_CALLEES.has(callee)) ||
    (ts.isIdentifier(expression) && bindings.directEnumerationBindings.has(expression.text)) ||
    (callee &&
      LAYER_ONE_CALLEES.has(callee) &&
      usesModuleAlias(expression, bindings.filesystemModuleBindings))
  ) {
    return "layer-1";
  }
  return null;
}

function isExecFamilyExpression(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): boolean {
  const callee = callExpressionName(expression);
  if (ts.isIdentifier(expression) && bindings.childProcessBindings.has(expression.text))
    return true;
  if (usesModuleAlias(expression, bindings.childProcessModuleBindings)) return true;
  return callee !== null && EXEC_FAMILY_CALLEES.has(callee);
}

function isSurfaceResolverModule(moduleName: string): boolean {
  return /(?:^|\/)scan-surface-manifest(?:\.ts)?$/u.test(moduleName);
}

function functionReturnsSurface(
  body: tsTypes.Block,
  resolverBindings: ReadonlySet<string>,
  factoryBindings: ReadonlySet<string>,
): boolean {
  let returnsSurface = false;
  const visit = (node: tsTypes.Node): void => {
    if (
      node !== body &&
      (ts.isFunctionDeclaration(node) ||
        ts.isFunctionExpression(node) ||
        ts.isArrowFunction(node) ||
        ts.isMethodDeclaration(node))
    ) {
      return;
    }
    if (
      ts.isReturnStatement(node) &&
      node.expression &&
      expressionProducesSurface(node.expression, resolverBindings, factoryBindings, new Set())
    ) {
      returnsSurface = true;
      return;
    }
    if (!returnsSurface) ts.forEachChild(node, visit);
  };
  visit(body);
  return returnsSurface;
}

function functionLikeReturnsSurface(
  node: tsTypes.ArrowFunction | tsTypes.FunctionExpression,
  resolverBindings: ReadonlySet<string>,
  factoryBindings: ReadonlySet<string>,
): boolean {
  return ts.isBlock(node.body)
    ? functionReturnsSurface(node.body, resolverBindings, factoryBindings)
    : expressionProducesSurface(node.body, resolverBindings, factoryBindings, new Set());
}

function expressionProducesSurface(
  expression: tsTypes.Expression,
  resolverBindings: ReadonlySet<string>,
  factoryBindings: ReadonlySet<string>,
  surfaceBindings: ReadonlySet<string>,
): boolean {
  if (ts.isIdentifier(expression)) return surfaceBindings.has(expression.text);
  if (!ts.isCallExpression(expression)) return false;
  const factory = callExpressionName(expression.expression);
  return factory !== null && (resolverBindings.has(factory) || factoryBindings.has(factory));
}

function nodeContainsRoutedEnumeration(node: tsTypes.Node, bindings: ScannerBindings): boolean {
  let found = false;
  const visit = (candidate: tsTypes.Node): void => {
    if (
      ts.isCallExpression(candidate) &&
      enumerationCallLayer(candidate.expression, bindings) &&
      isSurfaceRoutedExpression(candidate.expression, bindings)
    ) {
      found = true;
      return;
    }
    if (ts.isStringLiteral(candidate) && candidate.text === "ls-files") {
      const enclosingCall = enclosingArgumentCall(candidate);
      if (enclosingCall && isSurfaceRoutedExpression(enclosingCall.expression, bindings)) {
        found = true;
        return;
      }
    }
    if (!found) ts.forEachChild(candidate, visit);
  };
  visit(node);
  return found;
}

function routingDiagnostic(
  sourceFile: tsTypes.SourceFile,
  node: tsTypes.Node,
  message: string,
): ScannerRoutingDiagnostic {
  return {
    line: sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1,
    message,
  };
}

function enclosingArgumentCall(node: tsTypes.Node): tsTypes.CallExpression | null {
  let current: tsTypes.Node = node;
  while (current.parent) {
    current = current.parent;
    if (ts.isCallExpression(current)) {
      return current.arguments.some(
        (argument) => node.getStart() >= argument.getStart() && node.getEnd() <= argument.getEnd(),
      )
        ? current
        : null;
    }
    if (ts.isStatement(current)) return null;
  }
  return null;
}

function callExpressionName(expression: tsTypes.LeftHandSideExpression): string | null {
  if (ts.isIdentifier(expression)) return expression.text;
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  if (
    ts.isElementAccessExpression(expression) &&
    expression.argumentExpression &&
    (ts.isStringLiteral(expression.argumentExpression) ||
      ts.isNoSubstitutionTemplateLiteral(expression.argumentExpression))
  ) {
    return expression.argumentExpression.text;
  }
  return null;
}

function expressionRoot(expression: tsTypes.LeftHandSideExpression): tsTypes.Expression {
  let current: tsTypes.Expression = expression;
  while (ts.isPropertyAccessExpression(current) || ts.isElementAccessExpression(current)) {
    current = current.expression;
  }
  return current;
}

function usesModuleAlias(
  expression: tsTypes.LeftHandSideExpression,
  aliases: ReadonlySet<string>,
): boolean {
  const root = expressionRoot(expression);
  return ts.isIdentifier(root) && aliases.has(root.text);
}

function siteFor(
  sourceFile: tsTypes.SourceFile,
  node: tsTypes.Node,
  layer: ScannerCallSiteLayer,
): ScannerCallSite {
  return {
    layer,
    line: sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1,
    text: node.getText(sourceFile).replace(/\s+/gu, " ").slice(0, 160),
  };
}

function dedupeSites(sites: readonly ScannerCallSite[]): readonly ScannerCallSite[] {
  const seen = new Set<string>();
  return sites.filter((site) => {
    const key = `${site.layer}:${site.line}:${site.text}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
