import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AffectedPathRuleDeclarationV0 } from "../affected.ts";
import {
  assertUnmigratedScanRoot,
  defineScanSurface,
  resolveScanSurface,
  type ResolvedScanSurface,
} from "./scan-surface.ts";
import type {
  NamedScanPredicateId,
  ScanSurfaceSpec,
  UnmigratedScanReason,
} from "./scan-surface.ts";

export const EVIDENCE_SCAN_SURFACE_MANIFEST_PATH = "rust/evidence-scan-surfaces.json";

let bootstrapSpecs: ReadonlyMap<string, ScanSurfaceSpec> | null = null;

export interface EvidenceScanSurfaceManifestV0 {
  readonly schemaVersion: "0";
  readonly generatedBy: "pnpm omena-check evidence-surfaces --write";
  readonly detector: {
    readonly fileCount: number;
    readonly patternSha256: string;
  };
  readonly resolverSha256: string;
  readonly predicateAuthority?: { readonly modulePath: string; readonly sha256: string };
  readonly predicateModules: Readonly<
    Record<NamedScanPredicateId, { readonly modulePath: string; readonly sha256: string }>
  >;
  readonly pathPlaneAuthority?: {
    readonly modulePath: string;
    readonly sha256: string;
    readonly rules: readonly AffectedPathRuleDeclarationV0[];
  };
  readonly scanners: readonly EvidenceScannerManifestRowV0[];
}

export type EvidenceScannerManifestRowV0 =
  | MigratedEvidenceScannerRowV0
  | UnmigratedEvidenceScannerRowV0
  | FalsePositiveEvidenceScannerRowV0
  | RetiredEvidenceScannerRowV0;

export interface MigratedEvidenceScannerRowV0 {
  readonly scannerPath: string;
  readonly disposition: "MIGRATED";
  readonly surfaceModulePath: string;
  readonly surfaceModuleSha256: string;
  readonly spec: ScanSurfaceSpec;
  readonly gateIds: readonly string[];
  readonly narrowingReason?: string;
  readonly renamedFrom?: string;
}

export interface UnmigratedEvidenceScannerRowV0 {
  readonly scannerPath: string;
  readonly disposition: "UNMIGRATED";
  readonly effectiveSurface: "**";
  readonly reason: UnmigratedScanReason;
  readonly surfaceModulePath: string;
  readonly surfaceModuleSha256: string;
  readonly inRepoSpec?: ScanSurfaceSpec;
  readonly gateIds: readonly string[];
}

export interface FalsePositiveEvidenceScannerRowV0 {
  readonly scannerPath: string;
  readonly disposition: "FALSE-POSITIVE";
  readonly rationale: string;
  readonly rawMatchCount: number;
  readonly gateIds: readonly string[];
}

export interface RetiredEvidenceScannerRowV0 {
  readonly scannerPath: string;
  readonly disposition: "RETIRED";
  readonly reason: string;
  readonly deletionVerifiedAtIntroduction: true;
}

export function evidenceScanSurfaceManifestPath(repoRoot: string): string {
  return path.join(repoRoot, EVIDENCE_SCAN_SURFACE_MANIFEST_PATH);
}

export function loadEvidenceScanSurfaceManifest(
  repoRoot: string,
): EvidenceScanSurfaceManifestV0 | null {
  const manifestPath = evidenceScanSurfaceManifestPath(repoRoot);
  if (!existsSync(manifestPath)) return null;
  const parsed = JSON.parse(readFileSync(manifestPath, "utf8")) as EvidenceScanSurfaceManifestV0;
  // v0 readers accept the pre-authority shape so the official writer can perform
  // the one-way migration. Freshly rendered manifests remain strict below, and
  // --check compares every generated authority digest and declaration byte.
  validateEvidenceScanSurfaceManifest(parsed, true);
  return parsed;
}

export function loadCommittedEvidenceScanSurfaceManifest(
  repoRoot: string,
  ref = "HEAD",
): EvidenceScanSurfaceManifestV0 | null {
  const shown = spawnSync("git", ["show", `${ref}:${EVIDENCE_SCAN_SURFACE_MANIFEST_PATH}`], {
    cwd: repoRoot,
    encoding: "utf8",
    shell: false,
  });
  if ((shown.status ?? 1) !== 0) return null;
  const parsed = JSON.parse(String(shown.stdout)) as EvidenceScanSurfaceManifestV0;
  validateEvidenceScanSurfaceManifest(parsed, true);
  return parsed;
}

export function resolveScanSurfaceForScanner(
  scannerFile: string,
  repoRoot = findEvidenceRepoRoot(scannerFile),
  scanRoot = repoRoot,
): ResolvedScanSurface {
  const absoluteScannerPath = scannerFile.startsWith("file:")
    ? fileURLToPath(scannerFile)
    : path.resolve(scannerFile);
  const scannerPath = path.relative(repoRoot, absoluteScannerPath).replaceAll("\\", "/");
  const manifest = loadEvidenceScanSurfaceManifest(repoRoot);
  if (!manifest) {
    const bootstrapSpec = bootstrapSpecs?.get(scannerPath);
    if (bootstrapSpec) return resolveScanSurface(bootstrapSpec, { repoRoot: scanRoot });
    throw new Error(
      `evidence scan surface manifest is missing; run pnpm omena-check evidence-surfaces --write`,
    );
  }
  const row = manifest.scanners.find((candidate) => candidate.scannerPath === scannerPath);
  if (!row) {
    throw new Error(`scanner ${scannerPath} has no migrated scan surface`);
  }
  if (row.disposition === "MIGRATED") return resolveScanSurface(row.spec, { repoRoot: scanRoot });
  if (row.disposition !== "UNMIGRATED" || !row.inRepoSpec) {
    throw new Error(`scanner ${scannerPath} has no migrated in-repo scan surface`);
  }
  if (path.resolve(scanRoot) !== path.resolve(repoRoot)) {
    throw new Error(`scanner ${scannerPath} must use its guarded resolver outside repoRoot`);
  }
  return resolveScanSurface(row.inRepoSpec, { repoRoot });
}

export function resolveUnmigratedScanRootForScanner(
  scannerFile: string,
  reason: UnmigratedScanReason,
  repoRoot: string,
  scanRoot: string,
): ResolvedScanSurface {
  const absoluteScannerPath = scannerFile.startsWith("file:")
    ? fileURLToPath(scannerFile)
    : path.resolve(scannerFile);
  const scannerPath = path.relative(repoRoot, absoluteScannerPath).replaceAll("\\", "/");
  const manifest = loadEvidenceScanSurfaceManifest(repoRoot);
  const row = manifest?.scanners.find((candidate) => candidate.scannerPath === scannerPath);
  if (!row || row.disposition !== "UNMIGRATED" || row.reason !== reason) {
    throw new Error(`scanner ${scannerPath} has no matching guarded ${reason} scan root`);
  }
  const canonicalScanRoot = assertUnmigratedScanRoot(reason, repoRoot, scanRoot);
  return resolveScanSurface(
    defineScanSurface({
      scannerPath,
      mode: "workingTree",
      pathspecs: ["**"],
      includeUntracked: false,
      excludes: ["git-metadata"],
    }),
    { repoRoot: canonicalScanRoot },
  );
}

export function setEvidenceScanSurfaceBootstrapSpecs(
  specs: ReadonlyMap<string, ScanSurfaceSpec> | null,
): void {
  bootstrapSpecs = specs;
}

function findEvidenceRepoRoot(scannerFile: string): string {
  const absoluteScannerPath = scannerFile.startsWith("file:")
    ? fileURLToPath(scannerFile)
    : path.resolve(scannerFile);
  let directory = path.dirname(absoluteScannerPath);
  while (true) {
    const packageJsonPath = path.join(directory, "package.json");
    if (existsSync(packageJsonPath)) {
      try {
        const candidate = JSON.parse(readFileSync(packageJsonPath, "utf8")) as {
          readonly name?: string;
        };
        if (candidate.name === "omena-css") return directory;
      } catch {
        // Keep walking until the workspace root is found.
      }
    }
    const parent = path.dirname(directory);
    if (parent === directory) {
      throw new Error(`unable to locate evidence repo root from ${scannerFile}`);
    }
    directory = parent;
  }
}

export function renderEvidenceScanSurfaceManifest(manifest: EvidenceScanSurfaceManifestV0): string {
  validateEvidenceScanSurfaceManifest(manifest, false);
  return formatEvidenceJsonArtifact(manifest, EVIDENCE_SCAN_SURFACE_MANIFEST_PATH);
}

export function formatEvidenceJsonArtifact(value: unknown, artifactPath: string): string {
  const formatter = spawnSync(
    process.platform === "win32" ? "pnpm.cmd" : "pnpm",
    ["exec", "oxfmt", `--stdin-filepath=${artifactPath}`],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: `${JSON.stringify(value, null, 2)}\n`,
      shell: false,
    },
  );
  if (formatter.error) {
    throw new Error(`failed to start evidence JSON formatter: ${formatter.error.message}`);
  }
  if ((formatter.status ?? 1) !== 0) {
    throw new Error(
      String(formatter.stderr).trim() || `evidence JSON formatter exited ${formatter.status}`,
    );
  }
  return String(formatter.stdout);
}

export function sha256Text(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function validateEvidenceScanSurfaceManifest(
  manifest: EvidenceScanSurfaceManifestV0,
  allowLegacyAuthorityFields: boolean,
): void {
  if (manifest.schemaVersion !== "0") {
    throw new Error(`unsupported evidence scan surface schema ${String(manifest.schemaVersion)}`);
  }
  if (!manifest.predicateAuthority && allowLegacyAuthorityFields) {
    // The first authority-bound regeneration must still be able to replay the committed v0 shape.
  } else if (!isRepositoryModulePath(manifest.predicateAuthority?.modulePath)) {
    throw new Error("evidence predicate authority must name a repository module path");
  }
  if (!manifest.pathPlaneAuthority && allowLegacyAuthorityFields)
    return validateScannerRows(manifest, allowLegacyAuthorityFields);
  if (!isRepositoryModulePath(manifest.pathPlaneAuthority?.modulePath)) {
    throw new Error("affected path-plane authority must name a repository module path");
  }
  const ruleIds = manifest.pathPlaneAuthority.rules.map((row) => row.ruleId);
  if (
    ruleIds.length === 0 ||
    new Set(ruleIds).size !== ruleIds.length ||
    manifest.pathPlaneAuthority.rules.some((rule, index) => rule.priority !== index)
  ) {
    throw new Error("affected path-plane rules must be non-empty, unique, and priority-ordered");
  }
  for (const rule of manifest.pathPlaneAuthority.rules) {
    if (!isRepositoryModulePath(rule.ownerModulePath)) {
      throw new Error(`affected path rule ${rule.ruleId} has a non-module owner`);
    }
  }
  validateScannerRows(manifest, allowLegacyAuthorityFields);
}

function validateScannerRows(
  manifest: EvidenceScanSurfaceManifestV0,
  allowLegacyAuthorityFields: boolean,
): void {
  const paths = manifest.scanners.map((row) => row.scannerPath);
  const sorted = [...paths].toSorted();
  if (
    new Set(paths).size !== paths.length ||
    paths.some((entry, index) => entry !== sorted[index])
  ) {
    throw new Error("evidence scan surface scanner rows must be unique and sorted");
  }
  for (const row of manifest.scanners) {
    if (row.disposition === "UNMIGRATED") {
      if (row.effectiveSurface !== "**") {
        throw new Error(`unmigrated scanner ${row.scannerPath} must retain effective surface **`);
      }
      if (
        !allowLegacyAuthorityFields &&
        (!row.surfaceModulePath ||
          !/^[0-9a-f]{64}$/u.test(row.surfaceModuleSha256) ||
          (row.inRepoSpec && row.inRepoSpec.scannerPath !== row.scannerPath))
      ) {
        throw new Error(`unmigrated scanner ${row.scannerPath} lacks guarded root authority`);
      }
    }
    if (row.disposition === "MIGRATED" && row.spec.scannerPath !== row.scannerPath) {
      throw new Error(`scanner/spec path mismatch for ${row.scannerPath}`);
    }
  }
}

function isRepositoryModulePath(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    !value.startsWith("/") &&
    !value.startsWith("../") &&
    value.includes("/") &&
    path.posix.extname(value).length > 0
  );
}
