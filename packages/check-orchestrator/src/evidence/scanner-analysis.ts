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
const MODULE_SCANNER_NAMES = new Set(["fast-glob", "globby"]);

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
  const moduleAliases = new Set<string>();
  const callSites: ScannerCallSite[] = [];
  const tertiaryExecFamilySites: ScannerCallSite[] = [];

  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    if (!MODULE_SCANNER_NAMES.has(statement.moduleSpecifier.text)) continue;
    const importClause = statement.importClause;
    if (importClause?.name) moduleAliases.add(importClause.name.text);
    const bindings = importClause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings)) moduleAliases.add(bindings.name.text);
    if (bindings && ts.isNamedImports(bindings)) {
      for (const element of bindings.elements) moduleAliases.add(element.name.text);
    }
  }

  const visit = (node: tsTypes.Node): void => {
    if (ts.isCallExpression(node)) {
      const callee = callExpressionName(node.expression);
      if (callee && LAYER_ONE_CALLEES.has(callee)) {
        callSites.push(siteFor(sourceFile, node, "layer-1"));
      }
      if (usesModuleAlias(node.expression, moduleAliases)) {
        callSites.push(siteFor(sourceFile, node, "layer-2b"));
      }
    }
    if (ts.isStringLiteral(node) && node.text === "ls-files") {
      callSites.push(siteFor(sourceFile, node, "layer-2a"));
    }
    if (
      (ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateExpression(node)) &&
      node.getText(sourceFile).includes("ls-files")
    ) {
      const enclosingCall = enclosingArgumentCall(node);
      const callee = enclosingCall ? callExpressionName(enclosingCall.expression) : null;
      if (callee && EXEC_FAMILY_CALLEES.has(callee)) {
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
  const moduleAliases = scannerModuleAliases(sourceFile);
  const helperRoutes = routedTrackedProductionSourceHelpers(sourceFile);
  const diagnostics: ScannerRoutingDiagnostic[] = [];
  const visit = (node: tsTypes.Node): void => {
    if (ts.isCallExpression(node)) {
      const callee = callExpressionName(node.expression);
      if (
        callee &&
        LAYER_ONE_CALLEES.has(callee) &&
        !isSurfaceRoutedExpression(node.expression) &&
        !(callee === "trackedProductionSources" && helperRoutes.has(callee))
      ) {
        diagnostics.push(routingDiagnostic(sourceFile, node, `direct ${callee} call`));
      }
      if (usesModuleAlias(node.expression, moduleAliases)) {
        diagnostics.push(
          routingDiagnostic(sourceFile, node, "module glob call bypasses resolveScanSurface"),
        );
      }
    }
    if (ts.isStringLiteral(node) && node.text === "ls-files") {
      const enclosingCall = enclosingArgumentCall(node);
      if (!enclosingCall || !isSurfaceRoutedExpression(enclosingCall.expression)) {
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
      const callee = enclosingCall ? callExpressionName(enclosingCall.expression) : null;
      if (callee && EXEC_FAMILY_CALLEES.has(callee)) {
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

function scannerModuleAliases(sourceFile: tsTypes.SourceFile): ReadonlySet<string> {
  const aliases = new Set<string>();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    if (!MODULE_SCANNER_NAMES.has(statement.moduleSpecifier.text)) continue;
    const importClause = statement.importClause;
    if (importClause?.name) aliases.add(importClause.name.text);
    const bindings = importClause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings)) aliases.add(bindings.name.text);
    if (bindings && ts.isNamedImports(bindings)) {
      for (const element of bindings.elements) aliases.add(element.name.text);
    }
  }
  return aliases;
}

function routedTrackedProductionSourceHelpers(sourceFile: tsTypes.SourceFile): ReadonlySet<string> {
  const routed = new Set<string>();
  const visit = (node: tsTypes.Node): void => {
    if (
      ts.isFunctionDeclaration(node) &&
      node.name?.text === "trackedProductionSources" &&
      node.body &&
      node.body.getText(sourceFile).includes("evidenceScanSurface.")
    ) {
      routed.add(node.name.text);
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return routed;
}

function isSurfaceRoutedExpression(expression: tsTypes.LeftHandSideExpression): boolean {
  if (ts.isPropertyAccessExpression(expression)) {
    let root: tsTypes.Expression = expression.expression;
    while (ts.isPropertyAccessExpression(root)) root = root.expression;
    if (ts.isCallExpression(root)) {
      return callExpressionName(root.expression) === "resolveScanSurfaceForScanner";
    }
    return ts.isIdentifier(root) && /scanSurface$/iu.test(root.text);
  }
  return false;
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
  return null;
}

function usesModuleAlias(
  expression: tsTypes.LeftHandSideExpression,
  aliases: ReadonlySet<string>,
): boolean {
  if (ts.isIdentifier(expression)) return aliases.has(expression.text);
  if (ts.isPropertyAccessExpression(expression)) {
    let current: tsTypes.Expression = expression.expression;
    while (ts.isPropertyAccessExpression(current)) current = current.expression;
    return ts.isIdentifier(current) && aliases.has(current.text);
  }
  return false;
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
