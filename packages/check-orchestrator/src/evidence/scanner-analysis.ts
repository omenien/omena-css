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
const SURFACE_RESOLVER_EXPORTS = new Set([
  "resolveScanSurfaceForScanner",
  "resolveUnmigratedScanRootForScanner",
]);
const SURFACE_RESOLVER_MODULE_PATH =
  "packages/check-orchestrator/src/evidence/scan-surface-manifest.ts";

interface LexicalBindingIndex {
  readonly declarationFor: (identifier: tsTypes.Identifier) => tsTypes.Identifier | null;
}

interface ScannerBindings {
  readonly lexical: LexicalBindingIndex;
  readonly directEnumerationBindings: ReadonlySet<tsTypes.Identifier>;
  readonly filesystemModuleBindings: ReadonlySet<tsTypes.Identifier>;
  readonly enumerationModuleBindings: ReadonlySet<tsTypes.Identifier>;
  readonly childProcessBindings: ReadonlySet<tsTypes.Identifier>;
  readonly childProcessModuleBindings: ReadonlySet<tsTypes.Identifier>;
  readonly resolverBindings: ReadonlySet<tsTypes.Identifier>;
  readonly surfaceFactoryBindings: ReadonlySet<tsTypes.Identifier>;
  readonly surfaceObjectBindings: ReadonlySet<tsTypes.Identifier>;
  readonly typedSurfaceParameterBindings: ReadonlySet<tsTypes.Identifier>;
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
  const bindings = scannerBindings(sourceFile, scannerPath);
  const callSites: ScannerCallSite[] = [];
  const tertiaryExecFamilySites: ScannerCallSite[] = [];

  const visit = (node: tsTypes.Node): void => {
    if (ts.isCallExpression(node)) {
      const layer = enumerationCallLayer(node.expression, bindings);
      if (layer && (layer !== "layer-1" || !expressionHasLayerOneToken(node.expression))) {
        callSites.push(siteFor(sourceFile, node, layer));
      }
    }
    if (ts.isIdentifier(node) && layerOneTokenReferenceExpression(node)) {
      callSites.push(siteFor(sourceFile, node, "layer-1"));
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
  const bindings = scannerBindings(sourceFile, scannerPath);
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
    if (ts.isIdentifier(node)) {
      const referenceExpression = layerOneTokenReferenceExpression(node);
      if (
        referenceExpression &&
        !isEnumerationCalleeReference(referenceExpression, bindings) &&
        !isSurfaceRoutedExpression(referenceExpression, bindings)
      ) {
        diagnostics.push(
          routingDiagnostic(
            sourceFile,
            node,
            `direct ${node.text} token bypasses resolveScanSurface`,
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

function scannerBindings(sourceFile: tsTypes.SourceFile, scannerPath: string): ScannerBindings {
  const lexical = buildLexicalBindingIndex(sourceFile);
  const directEnumerationBindings = new Set<tsTypes.Identifier>();
  const filesystemModuleBindings = new Set<tsTypes.Identifier>();
  const enumerationModuleBindings = new Set<tsTypes.Identifier>();
  const childProcessBindings = new Set<tsTypes.Identifier>();
  const childProcessModuleBindings = new Set<tsTypes.Identifier>();
  const resolverBindings = new Set<tsTypes.Identifier>();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    const moduleName = statement.moduleSpecifier.text;
    const importClause = statement.importClause;
    if (importClause?.name) {
      if (MODULE_SCANNER_NAMES.has(moduleName)) {
        enumerationModuleBindings.add(importClause.name);
      }
      if (FILESYSTEM_MODULE_NAMES.has(moduleName)) {
        filesystemModuleBindings.add(importClause.name);
      }
      if (CHILD_PROCESS_MODULE_NAMES.has(moduleName)) {
        childProcessModuleBindings.add(importClause.name);
      }
    }
    const namedBindings = importClause?.namedBindings;
    if (namedBindings && ts.isNamespaceImport(namedBindings)) {
      if (MODULE_SCANNER_NAMES.has(moduleName)) {
        enumerationModuleBindings.add(namedBindings.name);
      }
      if (FILESYSTEM_MODULE_NAMES.has(moduleName)) {
        filesystemModuleBindings.add(namedBindings.name);
      }
      if (CHILD_PROCESS_MODULE_NAMES.has(moduleName)) {
        childProcessModuleBindings.add(namedBindings.name);
      }
    }
    if (namedBindings && ts.isNamedImports(namedBindings)) {
      for (const element of namedBindings.elements) {
        const importedName = element.propertyName?.text ?? element.name.text;
        if (FILESYSTEM_MODULE_NAMES.has(moduleName) && LAYER_ONE_CALLEES.has(importedName)) {
          directEnumerationBindings.add(element.name);
        }
        if (MODULE_SCANNER_NAMES.has(moduleName)) {
          enumerationModuleBindings.add(element.name);
        }
        if (CHILD_PROCESS_MODULE_NAMES.has(moduleName) && EXEC_FAMILY_CALLEES.has(importedName)) {
          childProcessBindings.add(element.name);
        }
        if (
          isCanonicalSurfaceResolverModule(scannerPath, moduleName) &&
          SURFACE_RESOLVER_EXPORTS.has(importedName)
        ) {
          resolverBindings.add(element.name);
        }
      }
    }
  }

  const surfaceFactoryBindings = new Set<tsTypes.Identifier>();
  let changed = true;
  while (changed) {
    changed = false;
    const inspectFactory = (node: tsTypes.Node): void => {
      if (
        ts.isFunctionDeclaration(node) &&
        node.name &&
        node.body &&
        functionReturnsSurface(node.body, resolverBindings, surfaceFactoryBindings, lexical) &&
        !surfaceFactoryBindings.has(node.name)
      ) {
        surfaceFactoryBindings.add(node.name);
        changed = true;
      }
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        (ts.isArrowFunction(node.initializer) || ts.isFunctionExpression(node.initializer)) &&
        functionLikeReturnsSurface(
          node.initializer,
          resolverBindings,
          surfaceFactoryBindings,
          lexical,
        ) &&
        !surfaceFactoryBindings.has(node.name)
      ) {
        surfaceFactoryBindings.add(node.name);
        changed = true;
      }
      ts.forEachChild(node, inspectFactory);
    };
    inspectFactory(sourceFile);
  }

  const surfaceObjectBindings = new Set<tsTypes.Identifier>();
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
          lexical,
        ) &&
        !surfaceObjectBindings.has(node.name)
      ) {
        surfaceObjectBindings.add(node.name);
        changed = true;
      }
      ts.forEachChild(node, inspectSurfaceObject);
    };
    inspectSurfaceObject(sourceFile);
  }

  const typedSurfaceParameterBindings = new Set<tsTypes.Identifier>();
  const inspectParameters = (node: tsTypes.Node): void => {
    if (
      ts.isParameter(node) &&
      ts.isIdentifier(node.name) &&
      node.type &&
      isCanonicalSurfaceParameterType(node.type, resolverBindings, lexical)
    ) {
      typedSurfaceParameterBindings.add(node.name);
    }
    ts.forEachChild(node, inspectParameters);
  };
  inspectParameters(sourceFile);

  return {
    lexical,
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

interface LexicalScope {
  readonly parent: LexicalScope | null;
  readonly bindings: ReadonlyMap<string, tsTypes.Identifier>;
}

function buildLexicalBindingIndex(sourceFile: tsTypes.SourceFile): LexicalBindingIndex {
  const resolved = new Map<tsTypes.Identifier, tsTypes.Identifier | null>();
  const declarationIdentifiers = new Set<tsTypes.Identifier>();

  const createScope = (node: tsTypes.Node, parent: LexicalScope | null): LexicalScope => {
    const bindings = collectScopeBindings(node);
    for (const declaration of bindings.values()) declarationIdentifiers.add(declaration);
    return { parent, bindings };
  };

  const rootScope = createScope(sourceFile, null);
  const visit = (node: tsTypes.Node, inheritedScope: LexicalScope): void => {
    const scope =
      node === sourceFile
        ? rootScope
        : isLexicalScopeNode(node)
          ? createScope(node, inheritedScope)
          : inheritedScope;
    if (ts.isIdentifier(node)) {
      if (declarationIdentifiers.has(node)) {
        resolved.set(node, node);
      } else {
        let current: LexicalScope | null = scope;
        let declaration: tsTypes.Identifier | null = null;
        while (current && !declaration) {
          declaration = current.bindings.get(node.text) ?? null;
          current = current.parent;
        }
        resolved.set(node, declaration);
      }
    }
    ts.forEachChild(node, (child) => visit(child, scope));
  };
  visit(sourceFile, rootScope);
  return { declarationFor: (identifier) => resolved.get(identifier) ?? null };
}

function isLexicalScopeNode(node: tsTypes.Node): boolean {
  return (
    ts.isBlock(node) ||
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isArrowFunction(node) ||
    ts.isMethodDeclaration(node) ||
    ts.isCatchClause(node) ||
    ts.isForStatement(node) ||
    ts.isForInStatement(node) ||
    ts.isForOfStatement(node)
  );
}

function collectScopeBindings(node: tsTypes.Node): ReadonlyMap<string, tsTypes.Identifier> {
  const bindings = new Map<string, tsTypes.Identifier>();
  const add = (identifier: tsTypes.Identifier | undefined): void => {
    if (identifier) bindings.set(identifier.text, identifier);
  };
  const addBindingName = (name: tsTypes.BindingName): void => {
    if (ts.isIdentifier(name)) {
      add(name);
      return;
    }
    for (const element of name.elements) {
      if (!ts.isOmittedExpression(element)) addBindingName(element.name);
    }
  };
  if (
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isArrowFunction(node) ||
    ts.isMethodDeclaration(node)
  ) {
    if ("name" in node && node.name && ts.isIdentifier(node.name)) add(node.name);
    for (const parameter of node.parameters) {
      addBindingName(parameter.name);
    }
    return bindings;
  }
  if (ts.isCatchClause(node)) {
    if (node.variableDeclaration) addBindingName(node.variableDeclaration.name);
    return bindings;
  }
  if (ts.isForStatement(node) || ts.isForInStatement(node) || ts.isForOfStatement(node)) {
    const initializer = node.initializer;
    if (initializer && ts.isVariableDeclarationList(initializer)) {
      for (const declaration of initializer.declarations) addBindingName(declaration.name);
    }
    return bindings;
  }
  if (!ts.isSourceFile(node) && !ts.isBlock(node)) return bindings;
  for (const statement of node.statements) {
    if (ts.isImportDeclaration(statement)) {
      add(statement.importClause?.name);
      const namedBindings = statement.importClause?.namedBindings;
      if (namedBindings && ts.isNamespaceImport(namedBindings)) add(namedBindings.name);
      if (namedBindings && ts.isNamedImports(namedBindings)) {
        for (const element of namedBindings.elements) add(element.name);
      }
      continue;
    }
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        addBindingName(declaration.name);
      }
      continue;
    }
    if (ts.isFunctionDeclaration(statement) && statement.name) {
      add(statement.name);
      continue;
    }
    if (statement.kind === ts.SyntaxKind.ClassDeclaration) {
      const name = (statement as tsTypes.Node & { readonly name?: tsTypes.Node }).name;
      if (name && ts.isIdentifier(name)) add(name);
    }
  }
  return bindings;
}

function isCanonicalSurfaceParameterType(
  typeNode: tsTypes.TypeNode,
  resolverBindings: ReadonlySet<tsTypes.Identifier>,
  lexical: LexicalBindingIndex,
): boolean {
  const unwrapped = ts.isParenthesizedTypeNode(typeNode) ? typeNode.type : typeNode;
  if (
    !ts.isTypeReferenceNode(unwrapped) ||
    !ts.isIdentifier(unwrapped.typeName) ||
    unwrapped.typeName.text !== "ReturnType" ||
    unwrapped.typeArguments?.length !== 1
  ) {
    return false;
  }
  const argument = unwrapped.typeArguments[0];
  if (!argument || argument.kind !== ts.SyntaxKind.TypeQuery) return false;
  let queried: tsTypes.Identifier | null = null;
  ts.forEachChild(argument, (child) => {
    if (queried === null && ts.isIdentifier(child)) queried = child;
  });
  if (queried === null) return false;
  const declaration = lexical.declarationFor(queried);
  return Boolean(declaration && resolverBindings.has(declaration));
}

function directCalleeBinding(
  expression: tsTypes.LeftHandSideExpression,
  lexical: LexicalBindingIndex,
): tsTypes.Identifier | null {
  return ts.isIdentifier(expression) ? lexical.declarationFor(expression) : null;
}

function bindingSetHasUse(
  bindings: ReadonlySet<tsTypes.Identifier>,
  identifier: tsTypes.Identifier,
  lexical: LexicalBindingIndex,
): boolean {
  const declaration = lexical.declarationFor(identifier);
  return Boolean(declaration && bindings.has(declaration));
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
    const factory = directCalleeBinding(root.expression, bindings.lexical);
    return Boolean(
      factory &&
      (bindings.resolverBindings.has(factory) || bindings.surfaceFactoryBindings.has(factory)),
    );
  }
  if (!ts.isIdentifier(root)) return false;
  const declaration = bindings.lexical.declarationFor(root);
  return Boolean(
    declaration &&
    (bindings.surfaceObjectBindings.has(declaration) ||
      bindings.typedSurfaceParameterBindings.has(declaration)),
  );
}

function enumerationCallLayer(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): Extract<ScannerCallSiteLayer, "layer-1" | "layer-2b"> | null {
  if (usesModuleAlias(expression, bindings.enumerationModuleBindings, bindings.lexical))
    return "layer-2b";
  const callee = callExpressionName(expression);
  if (
    (callee && LAYER_ONE_CALLEES.has(callee)) ||
    (ts.isIdentifier(expression) &&
      bindingSetHasUse(bindings.directEnumerationBindings, expression, bindings.lexical)) ||
    (callee &&
      LAYER_ONE_CALLEES.has(callee) &&
      usesModuleAlias(expression, bindings.filesystemModuleBindings, bindings.lexical))
  ) {
    return "layer-1";
  }
  return null;
}

function expressionHasLayerOneToken(expression: tsTypes.Node): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (ts.isIdentifier(node) && layerOneTokenReferenceExpression(node)) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(expression);
  return found;
}

function layerOneTokenReferenceExpression(
  identifier: tsTypes.Identifier,
): tsTypes.LeftHandSideExpression | null {
  if (!LAYER_ONE_CALLEES.has(identifier.text)) return null;
  const parent = identifier.parent;
  if (ts.isPropertyAccessExpression(parent) && parent.name === identifier) return parent;
  if (
    ts.isImportSpecifier(parent) ||
    parent.kind === ts.SyntaxKind.MethodSignature ||
    parent.kind === ts.SyntaxKind.PropertySignature ||
    ts.isVariableDeclaration(parent) ||
    ts.isParameter(parent) ||
    (ts.isFunctionDeclaration(parent) && parent.name === identifier) ||
    (ts.isFunctionExpression(parent) && parent.name === identifier) ||
    (ts.isMethodDeclaration(parent) && parent.name === identifier)
  ) {
    return null;
  }
  return identifier;
}

function isEnumerationCalleeReference(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): boolean {
  const parent = expression.parent;
  return (
    ts.isCallExpression(parent) &&
    parent.expression === expression &&
    enumerationCallLayer(parent.expression, bindings) === "layer-1"
  );
}

function isExecFamilyExpression(
  expression: tsTypes.LeftHandSideExpression,
  bindings: ScannerBindings,
): boolean {
  const callee = callExpressionName(expression);
  if (
    ts.isIdentifier(expression) &&
    bindingSetHasUse(bindings.childProcessBindings, expression, bindings.lexical)
  )
    return true;
  if (usesModuleAlias(expression, bindings.childProcessModuleBindings, bindings.lexical))
    return true;
  return callee !== null && EXEC_FAMILY_CALLEES.has(callee);
}

function isCanonicalSurfaceResolverModule(scannerPath: string, moduleName: string): boolean {
  if (!moduleName.startsWith(".")) return false;
  const resolved = path.posix.normalize(
    path.posix.join(path.posix.dirname(scannerPath), moduleName),
  );
  const withoutExtension = resolved.replace(/\.(?:[cm]?[jt]sx?)$/u, "");
  return withoutExtension === SURFACE_RESOLVER_MODULE_PATH.replace(/\.ts$/u, "");
}

function functionReturnsSurface(
  body: tsTypes.Block,
  resolverBindings: ReadonlySet<tsTypes.Identifier>,
  factoryBindings: ReadonlySet<tsTypes.Identifier>,
  lexical: LexicalBindingIndex,
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
      expressionProducesSurface(
        node.expression,
        resolverBindings,
        factoryBindings,
        new Set(),
        lexical,
      )
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
  node: tsTypes.FunctionLikeDeclaration,
  resolverBindings: ReadonlySet<tsTypes.Identifier>,
  factoryBindings: ReadonlySet<tsTypes.Identifier>,
  lexical: LexicalBindingIndex,
): boolean {
  const body = node.body;
  if (!body) return false;
  return ts.isBlock(body)
    ? functionReturnsSurface(body, resolverBindings, factoryBindings, lexical)
    : expressionProducesSurface(body, resolverBindings, factoryBindings, new Set(), lexical);
}

function expressionProducesSurface(
  expression: tsTypes.Expression,
  resolverBindings: ReadonlySet<tsTypes.Identifier>,
  factoryBindings: ReadonlySet<tsTypes.Identifier>,
  surfaceBindings: ReadonlySet<tsTypes.Identifier>,
  lexical: LexicalBindingIndex,
): boolean {
  if (ts.isIdentifier(expression)) {
    const declaration = lexical.declarationFor(expression);
    return Boolean(declaration && surfaceBindings.has(declaration));
  }
  if (!ts.isCallExpression(expression)) return false;
  const factory = directCalleeBinding(expression.expression, lexical);
  return Boolean(factory && (resolverBindings.has(factory) || factoryBindings.has(factory)));
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
  aliases: ReadonlySet<tsTypes.Identifier>,
  lexical: LexicalBindingIndex,
): boolean {
  const root = expressionRoot(expression);
  return ts.isIdentifier(root) && bindingSetHasUse(aliases, root, lexical);
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
