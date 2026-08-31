import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { compilerApi as ts } from "../../../../server/engine-core-ts/src/ts-facade";
import type tsTypes from "../../../../server/engine-core-ts/src/ts-facade";
import type { CheckManifest } from "../manifest/types";
import { SCAN_SURFACE_PREDICATE_MODULE_PATHS } from "./predicates/index";
import {
  detectScannerFiles,
  findUnroutedScannerCallSites,
  SCANNER_DETECTION_PATTERN,
} from "./scanner-analysis";
import {
  loadCommittedEvidenceScanSurfaceManifest,
  setEvidenceScanSurfaceBootstrapSpecs,
  sha256Text,
  type EvidenceScannerManifestRowV0,
  type EvidenceScanSurfaceManifestV0,
  type MigratedEvidenceScannerRowV0,
} from "./scan-surface-manifest";
import {
  defineScanSurface,
  resolveScanSurface,
  type NamedScanPredicateId,
  type ScanSurfaceDeclaration,
  type ScanSurfaceSpec,
} from "./scan-surface";

const SURFACE_MODULE_IMPORT = "../../packages/check-orchestrator/src/evidence/scan-surface";
const RESOLVER_MODULE_PATH = "packages/check-orchestrator/src/evidence/scan-surface.ts";
const SURFACE_MODULES_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
  mode: "index",
  pathspecs: ["scripts/surfaces/*.surface.ts"],
  includeUntracked: true,
  excludes: [],
} satisfies ScanSurfaceSpec);

interface LoadedSurfaceDeclaration {
  readonly modulePath: string;
  readonly moduleSha256: string;
  readonly declaration: ScanSurfaceDeclaration;
}

export async function buildEvidenceScanSurfaceManifest(
  repoRoot: string,
): Promise<EvidenceScanSurfaceManifestV0> {
  const declarations = await loadSurfaceDeclarations(repoRoot);
  const declarationByScanner = new Map<string, LoadedSurfaceDeclaration>();
  for (const loaded of declarations) {
    const scannerPath = scannerPathForDeclaration(loaded.declaration);
    if (declarationByScanner.has(scannerPath)) {
      throw new Error(`duplicate scan surface declaration for ${scannerPath}`);
    }
    declarationByScanner.set(scannerPath, loaded);
  }

  const analysisDeclaration = declarationByScanner.get(
    "packages/check-orchestrator/src/evidence/scanner-analysis.ts",
  );
  if (!analysisDeclaration || analysisDeclaration.declaration.disposition !== "MIGRATED") {
    throw new Error("scanner-analysis.ts must have a migrated detection surface declaration");
  }
  const detections = detectScannerFiles(repoRoot, analysisDeclaration.declaration.spec);
  const detectionByPath = new Map(detections.map((entry) => [entry.scannerPath, entry]));
  const bootstrapSpecs = new Map(
    declarations.flatMap(({ declaration }) =>
      declaration.disposition === "MIGRATED"
        ? [[declaration.spec.scannerPath, declaration.spec] as const]
        : [],
    ),
  );
  let checkManifest: CheckManifest;
  setEvidenceScanSurfaceBootstrapSpecs(bootstrapSpecs);
  try {
    const { loadCheckManifest } = await import("../manifest/index");
    checkManifest = loadCheckManifest(repoRoot);
  } finally {
    setEvidenceScanSurfaceBootstrapSpecs(null);
  }
  const oldManifest = loadCommittedEvidenceScanSurfaceManifest(repoRoot);
  const deletedPaths = currentDeletedPaths(repoRoot);
  const resolverSha256 = sha256Text(
    readFileSync(path.join(repoRoot, RESOLVER_MODULE_PATH), "utf8"),
  );
  const predicateModules = Object.fromEntries(
    Object.entries(SCAN_SURFACE_PREDICATE_MODULE_PATHS).map(([predicateId, modulePath]) => [
      predicateId,
      {
        modulePath,
        sha256: sha256Text(readFileSync(path.join(repoRoot, modulePath), "utf8")),
      },
    ]),
  ) as EvidenceScanSurfaceManifestV0["predicateModules"];

  const rows: EvidenceScannerManifestRowV0[] = [];
  for (const detection of detections) {
    const loaded = declarationByScanner.get(detection.scannerPath);
    if (!loaded) {
      throw new Error(`detected scanner has no surface declaration: ${detection.scannerPath}`);
    }
    const directGateIds = checkManifest.gates
      .filter(
        (gate) =>
          gate.command.includes(detection.scannerPath) ||
          gate.command.includes(`./${detection.scannerPath}`),
      )
      .map((gate) => gate.id);
    const gateIds = [
      ...new Set([
        ...directGateIds,
        ...(detection.scannerPath.startsWith("packages/check-orchestrator/src/")
          ? ["tooling/orchestrator-doctor", "tooling/orchestrator-inventory"]
          : []),
      ]),
    ].toSorted();
    const declaration = loaded.declaration;
    switch (declaration.disposition) {
      case "MIGRATED": {
        if (declaration.spec.scannerPath !== detection.scannerPath) {
          throw new Error(`migrated scan surface path mismatch: ${detection.scannerPath}`);
        }
        requireResolverRouting(repoRoot, detection.scannerPath);
        rows.push({
          scannerPath: detection.scannerPath,
          disposition: "MIGRATED",
          surfaceModulePath: loaded.modulePath,
          surfaceModuleSha256: loaded.moduleSha256,
          spec: declaration.spec,
          gateIds,
          ...(declaration.narrowingReason ? { narrowingReason: declaration.narrowingReason } : {}),
          ...(declaration.renamedFrom ? { renamedFrom: declaration.renamedFrom } : {}),
        });
        break;
      }
      case "UNMIGRATED": {
        const source = readFileSync(path.join(repoRoot, detection.scannerPath), "utf8");
        if (!source.includes(declaration.evidenceNeedle)) {
          throw new Error(
            `unmigrated scanner ${detection.scannerPath} lacks evidence needle ${declaration.evidenceNeedle}`,
          );
        }
        rows.push({
          scannerPath: detection.scannerPath,
          disposition: "UNMIGRATED",
          effectiveSurface: "**",
          reason: declaration.reason,
          evidenceNeedle: declaration.evidenceNeedle,
          gateIds,
        });
        break;
      }
      case "FALSE-POSITIVE": {
        if (detection.callSites.length > 0 || detection.tertiaryExecFamilySites.length > 0) {
          throw new Error(
            `FALSE-POSITIVE scanner ${detection.scannerPath} has ${detection.callSites.length} call sites`,
          );
        }
        rows.push({
          scannerPath: detection.scannerPath,
          disposition: "FALSE-POSITIVE",
          rationale: declaration.rationale,
          rawMatchCount: detection.rawMatchCount,
          gateIds,
        });
        break;
      }
      case "RETIRED":
        throw new Error(`detected scanner cannot be RETIRED: ${detection.scannerPath}`);
    }
  }

  for (const loaded of declarations) {
    if (loaded.declaration.disposition !== "RETIRED") continue;
    const scannerPath = loaded.declaration.scannerPath;
    if (detectionByPath.has(scannerPath)) {
      throw new Error(`retired scanner is still detected: ${scannerPath}`);
    }
    const existedAsRetired = oldManifest?.scanners.some(
      (row) => row.scannerPath === scannerPath && row.disposition === "RETIRED",
    );
    if (!existedAsRetired && !deletedPaths.has(scannerPath)) {
      throw new Error(
        `new RETIRED row does not name a file deleted in the current change: ${scannerPath}`,
      );
    }
    rows.push({
      scannerPath,
      disposition: "RETIRED",
      reason: loaded.declaration.reason,
      deletionVerifiedAtIntroduction: true,
    });
  }

  const declaredActivePaths = new Set(
    rows.filter((row) => row.disposition !== "RETIRED").map((row) => row.scannerPath),
  );
  for (const loaded of declarations) {
    const scannerPath = scannerPathForDeclaration(loaded.declaration);
    if (loaded.declaration.disposition !== "RETIRED" && !declaredActivePaths.has(scannerPath)) {
      throw new Error(
        `surface declaration does not correspond to a detected scanner: ${scannerPath}`,
      );
    }
  }

  const manifest: EvidenceScanSurfaceManifestV0 = {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-surfaces --write",
    detector: {
      fileCount: detections.length,
      patternSha256: sha256Text(SCANNER_DETECTION_PATTERN.source),
    },
    resolverSha256,
    predicateModules,
    scanners: rows.toSorted((left, right) => compareText(left.scannerPath, right.scannerPath)),
  };
  enforceManifestContinuity(oldManifest, manifest);
  enforceDetectorRatchet(oldManifest, manifest);
  await enforceSurfaceNarrowing(repoRoot, oldManifest, manifest);
  return manifest;
}

async function loadSurfaceDeclarations(
  repoRoot: string,
): Promise<readonly LoadedSurfaceDeclaration[]> {
  const modulePaths = resolveScanSurface(SURFACE_MODULES_SURFACE, { repoRoot }).paths.toSorted();
  const loaded: LoadedSurfaceDeclaration[] = [];
  for (const modulePath of modulePaths) {
    const source = readFileSync(path.join(repoRoot, modulePath), "utf8");
    enforceImportClosed(modulePath, source);
    const moduleSha256 = sha256Text(source);
    const moduleUrl = `${pathToFileURL(path.join(repoRoot, modulePath)).href}?sha256=${moduleSha256}`;
    const imported = (await import(moduleUrl)) as { readonly default?: unknown };
    if (!isScanSurfaceDeclaration(imported.default)) {
      throw new Error(`surface module ${modulePath} must default-export a scan declaration`);
    }
    loaded.push({ modulePath, moduleSha256, declaration: imported.default });
  }
  return loaded;
}

function enforceImportClosed(modulePath: string, source: string): void {
  const sourceFile = ts.createSourceFile(modulePath, source, ts.ScriptTarget.Latest, true);
  const imports = sourceFile.statements.filter(ts.isImportDeclaration);
  const importDeclaration = imports[0];
  if (
    imports.length !== 1 ||
    !importDeclaration ||
    !ts.isStringLiteral(importDeclaration.moduleSpecifier) ||
    importDeclaration.moduleSpecifier.text !== SURFACE_MODULE_IMPORT
  ) {
    throw new Error(
      `surface module ${modulePath} must import only the shared scan-surface contract`,
    );
  }
  let forbiddenRuntimeImport: tsTypes.Node | null = null;
  const visit = (node: tsTypes.Node): void => {
    if (
      (ts.isCallExpression(node) &&
        (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
          (ts.isIdentifier(node.expression) && node.expression.text === "require"))) ||
      ts.isImportEqualsDeclaration(node) ||
      ts.isExportDeclaration(node)
    ) {
      forbiddenRuntimeImport = node;
      return;
    }
    if (!forbiddenRuntimeImport) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  if (forbiddenRuntimeImport) {
    throw new Error(
      `surface module ${modulePath} must be import-closed beyond the shared scan-surface contract`,
    );
  }
}

function isScanSurfaceDeclaration(value: unknown): value is ScanSurfaceDeclaration {
  if (!value || typeof value !== "object" || !("disposition" in value)) return false;
  return ["MIGRATED", "UNMIGRATED", "FALSE-POSITIVE", "RETIRED"].includes(
    String(value.disposition),
  );
}

function scannerPathForDeclaration(declaration: ScanSurfaceDeclaration): string {
  return declaration.disposition === "MIGRATED"
    ? declaration.spec.scannerPath
    : declaration.scannerPath;
}

function requireResolverRouting(repoRoot: string, scannerPath: string): void {
  const source = readFileSync(path.join(repoRoot, scannerPath), "utf8");
  if (!source.includes("resolveScanSurface")) {
    throw new Error(`migrated scanner does not route through resolveScanSurface: ${scannerPath}`);
  }
  const diagnostics = findUnroutedScannerCallSites(scannerPath, source);
  if (diagnostics.length > 0) {
    throw new Error(
      `migrated scanner has an undeclared enumeration entry point: ${scannerPath}:${diagnostics[0]?.line} ${diagnostics[0]?.message}`,
    );
  }
}

function currentDeletedPaths(repoRoot: string): ReadonlySet<string> {
  const result = spawnSync("git", ["diff", "--name-only", "--diff-filter=D", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: false,
  });
  if ((result.status ?? 1) !== 0) throw new Error(String(result.stderr).trim());
  return new Set(String(result.stdout).split(/\r?\n/u).filter(Boolean));
}

function enforceDetectorRatchet(
  oldManifest: EvidenceScanSurfaceManifestV0 | null,
  newManifest: EvidenceScanSurfaceManifestV0,
): void {
  if (!oldManifest || newManifest.detector.fileCount >= oldManifest.detector.fileCount) return;
  const oldActivePaths = oldManifest.scanners
    .filter((row) => row.disposition !== "RETIRED")
    .map((row) => row.scannerPath);
  const newPaths = new Set(newManifest.scanners.map((row) => row.scannerPath));
  const missing = oldActivePaths.filter((scannerPath) => !newPaths.has(scannerPath));
  if (missing.length === 0) return;
  throw new Error(
    `scanner detector file count decreased ${oldManifest.detector.fileCount}->${newManifest.detector.fileCount}; missing RETIRED rows: ${missing.join(", ")}`,
  );
}

function enforceManifestContinuity(
  oldManifest: EvidenceScanSurfaceManifestV0 | null,
  newManifest: EvidenceScanSurfaceManifestV0,
): void {
  if (!oldManifest) return;
  const oldActivePaths = new Set(
    oldManifest.scanners
      .filter((row) => row.disposition !== "RETIRED")
      .map((row) => row.scannerPath),
  );
  const newRowsByPath = new Map(newManifest.scanners.map((row) => [row.scannerPath, row]));
  const pairedOldPaths = new Set<string>();
  for (const row of newManifest.scanners) {
    if (row.disposition !== "MIGRATED" || !row.renamedFrom) continue;
    if (!oldActivePaths.has(row.renamedFrom)) {
      throw new Error(
        `scan surface rename ${row.scannerPath} names unknown old scanner ${row.renamedFrom}`,
      );
    }
    if (newRowsByPath.has(row.renamedFrom)) {
      throw new Error(
        `scan surface rename ${row.scannerPath} cannot pair an old path that still has a row: ${row.renamedFrom}`,
      );
    }
    if (pairedOldPaths.has(row.renamedFrom)) {
      throw new Error(`old scan surface is paired more than once: ${row.renamedFrom}`);
    }
    pairedOldPaths.add(row.renamedFrom);
  }
  for (const oldPath of oldActivePaths) {
    if (newRowsByPath.has(oldPath) || pairedOldPaths.has(oldPath)) continue;
    throw new Error(
      `old scan surface disappeared without a RETIRED row or explicit renamedFrom pair: ${oldPath}`,
    );
  }
}

async function enforceSurfaceNarrowing(
  repoRoot: string,
  oldManifest: EvidenceScanSurfaceManifestV0 | null,
  newManifest: EvidenceScanSurfaceManifestV0,
): Promise<void> {
  if (!oldManifest) return;
  const oldMigrated = new Map(
    oldManifest.scanners
      .filter((row): row is MigratedEvidenceScannerRowV0 => row.disposition === "MIGRATED")
      .map((row) => [row.scannerPath, row]),
  );
  const resolverChanged = oldManifest.resolverSha256 !== newManifest.resolverSha256;
  const historicalResolver = resolverChanged
    ? await loadHistoricalScanSurfaceResolver(repoRoot, oldManifest)
    : null;
  for (const row of newManifest.scanners) {
    if (row.disposition !== "MIGRATED") continue;
    const oldRow = oldMigrated.get(row.renamedFrom ?? row.scannerPath);
    if (!oldRow) continue;
    const relevantPredicateIds = [...new Set([...oldRow.spec.excludes, ...row.spec.excludes])];
    const predicateChanged = relevantPredicateIds.some(
      (predicateId: NamedScanPredicateId) =>
        oldManifest.predicateModules[predicateId]?.sha256 !==
        newManifest.predicateModules[predicateId]?.sha256,
    );
    const specChanged = JSON.stringify(oldRow.spec) !== JSON.stringify(row.spec);
    const moduleChanged = oldRow.surfaceModuleSha256 !== row.surfaceModuleSha256;
    if (!resolverChanged && !predicateChanged && !specChanged && !moduleChanged) continue;
    const oldPredicateOverrides = historicalResolver
      ? {}
      : await loadHistoricalPredicateOverrides(repoRoot, oldManifest, relevantPredicateIds);
    let oldPaths: readonly string[];
    try {
      oldPaths = historicalResolver
        ? historicalResolver(oldRow.spec, { repoRoot }).paths
        : resolveScanSurface(oldRow.spec, {
            repoRoot,
            excludePredicates: oldPredicateOverrides,
          }).paths;
    } catch (error) {
      throw new Error(
        `old scan surface resolution failed and requires reviewer escalation for ${row.scannerPath}: ${String(error)}`,
      );
    }
    const newPathSet = new Set(resolveScanSurface(row.spec, { repoRoot }).paths);
    const removedPaths = oldPaths.filter((candidate) => !newPathSet.has(candidate));
    if (
      removedPaths.length > 0 &&
      (!row.narrowingReason || row.narrowingReason === oldRow.narrowingReason)
    ) {
      throw new Error(
        `scan surface narrowed without a new reason for ${row.scannerPath}; first removed path ${removedPaths[0]}`,
      );
    }
  }
}

type HistoricalResolveScanSurface = typeof resolveScanSurface;

async function loadHistoricalScanSurfaceResolver(
  repoRoot: string,
  oldManifest: EvidenceScanSurfaceManifestV0,
): Promise<HistoricalResolveScanSurface> {
  const resolverSource = historicalSource(repoRoot, RESOLVER_MODULE_PATH);
  if (sha256Text(resolverSource) !== oldManifest.resolverSha256) {
    throw new Error("old scan surface resolver bytes do not match the committed manifest digest");
  }
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "omena-old-scan-surface-"));
  try {
    const predicateDirectory = path.join(temporaryRoot, "predicates");
    mkdirSync(predicateDirectory, { recursive: true });
    writeFileSync(path.join(temporaryRoot, "scan-surface.ts"), resolverSource);
    for (const sourcePath of [
      "packages/check-orchestrator/src/evidence/predicates/index.ts",
      "packages/check-orchestrator/src/evidence/predicates/types.ts",
      ...Object.values(oldManifest.predicateModules).map((entry) => entry.modulePath),
    ]) {
      writeFileSync(
        path.join(predicateDirectory, path.posix.basename(sourcePath)),
        historicalSource(repoRoot, sourcePath),
      );
    }
    const moduleUrl = `${pathToFileURL(path.join(temporaryRoot, "scan-surface.ts")).href}?sha256=${oldManifest.resolverSha256}`;
    const imported = (await import(moduleUrl)) as {
      readonly resolveScanSurface?: unknown;
    };
    if (typeof imported.resolveScanSurface !== "function") {
      throw new Error("historical scan surface module has no resolveScanSurface export");
    }
    return imported.resolveScanSurface as HistoricalResolveScanSurface;
  } catch (error) {
    throw new Error(`historical scan surface resolver load failed and escalates: ${String(error)}`);
  } finally {
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
}

function historicalSource(repoRoot: string, sourcePath: string): string {
  const shown = spawnSync("git", ["show", `HEAD:${sourcePath}`], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: false,
  });
  if ((shown.status ?? 1) !== 0) {
    throw new Error(`historical source is unavailable at HEAD: ${sourcePath}`);
  }
  return String(shown.stdout);
}

async function loadHistoricalPredicateOverrides(
  repoRoot: string,
  oldManifest: EvidenceScanSurfaceManifestV0,
  predicateIds: readonly NamedScanPredicateId[],
): Promise<Partial<Readonly<Record<NamedScanPredicateId, (candidate: string) => boolean>>>> {
  const overrides: Partial<Record<NamedScanPredicateId, (candidate: string) => boolean>> = {};
  for (const predicateId of predicateIds) {
    const oldModule = oldManifest.predicateModules[predicateId];
    if (!oldModule) {
      throw new Error(`old predicate ${predicateId} is absent; narrowing comparison escalates`);
    }
    const shown = spawnSync("git", ["show", `HEAD:${oldModule.modulePath}`], {
      cwd: repoRoot,
      encoding: "utf8",
      shell: false,
    });
    if ((shown.status ?? 1) !== 0) {
      const currentSource = readFileSync(path.join(repoRoot, oldModule.modulePath), "utf8");
      if (sha256Text(currentSource) !== oldModule.sha256) {
        throw new Error(
          `old predicate ${predicateId} cannot be loaded from HEAD and current bytes do not match its digest`,
        );
      }
      continue;
    }
    const source = String(shown.stdout);
    if (sha256Text(source) !== oldModule.sha256) {
      throw new Error(
        `old predicate ${predicateId} bytes do not match the committed manifest digest`,
      );
    }
    const sourceFile = ts.createSourceFile(
      oldModule.modulePath,
      source,
      ts.ScriptTarget.Latest,
      true,
    );
    const runtimeImport = sourceFile.statements.find(
      (statement) => ts.isImportDeclaration(statement) && !statement.importClause?.isTypeOnly,
    );
    if (runtimeImport) {
      throw new Error(
        `predicate ${predicateId} has a runtime import; historical behavior cannot be isolated`,
      );
    }
    const javascript = ts.transpileModule(source, {
      compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
      fileName: oldModule.modulePath,
    }).outputText;
    const moduleUrl = `data:text/javascript;base64,${Buffer.from(javascript).toString("base64")}#${oldModule.sha256}`;
    const imported = (await import(moduleUrl)) as { readonly default?: unknown };
    if (typeof imported.default !== "function") {
      throw new Error(`historical predicate ${predicateId} has no default function`);
    }
    overrides[predicateId] = imported.default as (candidate: string) => boolean;
  }
  return overrides;
}

export function assertSurfaceNarrowingReason(
  repoRoot: string,
  oldSpec: ScanSurfaceSpec,
  newSpec: ScanSurfaceSpec,
  narrowingReason?: string,
): void {
  let oldPaths: readonly string[];
  try {
    oldPaths = resolveScanSurface(oldSpec, { repoRoot }).paths;
  } catch (error) {
    throw new Error(
      `old scan surface resolution failed and requires reviewer escalation for ${newSpec.scannerPath}: ${String(error)}`,
    );
  }
  const newPathSet = new Set(resolveScanSurface(newSpec, { repoRoot }).paths);
  const removedPaths = oldPaths.filter((candidate) => !newPathSet.has(candidate));
  if (removedPaths.length > 0 && !narrowingReason) {
    throw new Error(
      `scan surface narrowed without a reason for ${newSpec.scannerPath}; first removed path ${removedPaths[0]}`,
    );
  }
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
