import { buildAffectedCheckPlan, type AffectedCheckPlan } from "../affected";
import type {
  EvidenceScannerManifestRowV0,
  EvidenceScanSurfaceManifestV0,
} from "./scan-surface-manifest";
import { scanSurfaceMatchesPath, type MigratedScanSurfaceDeclaration } from "./scan-surface";
import type {
  EvidenceArtifactRowV0,
  EvidenceWriterRegistryV0,
  NotPreviewableInputV0,
} from "./writer-registry";

export interface BuildEvidenceAffectedPlanInput {
  readonly changedPaths: readonly string[];
  readonly scanManifest: EvidenceScanSurfaceManifestV0;
  readonly writerRegistry: EvidenceWriterRegistryV0;
  readonly expectedScannerPaths: readonly string[];
  readonly knownGateIds: readonly string[];
  readonly forceNarrowSufficiencyForTest?: boolean;
}

export interface EvidenceAffectedPlanV0 {
  readonly schemaVersion: "0";
  readonly changedPaths: readonly string[];
  readonly pathPlane: AffectedCheckPlan;
  readonly evidenceRefreshSufficientAlone: boolean;
  readonly fallbackToFullEvidence: boolean;
  readonly fallbackReasons: readonly string[];
  readonly affectedScannerPaths: readonly string[];
  readonly excludedScannerPaths: readonly string[];
  readonly affectedArtifactPaths: readonly string[];
  readonly writerOrder: readonly string[];
  readonly gateIds: readonly string[];
  readonly notRefreshed: readonly {
    readonly artifactPath: string;
    readonly classification: "W3" | "W4";
    readonly instruction: readonly string[] | string;
  }[];
  readonly notPreviewableInputs: readonly NotPreviewableInputV0[];
  readonly commitThenRefresh: readonly {
    readonly artifactPath: string;
    readonly steps: readonly string[];
  }[];
}

const EVIDENCE_GLOBAL_INPUTS = new Set([
  "rust/evidence-scan-surfaces.json",
  "rust/evidence-writer-registry.json",
  "packages/check-orchestrator/src/affected.ts",
  "packages/check-orchestrator/src/evidence/scan-surface.ts",
  "packages/check-orchestrator/src/evidence/scan-surface-manifest.ts",
  "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
]);

export function buildEvidenceAffectedPlan(
  input: BuildEvidenceAffectedPlanInput,
): EvidenceAffectedPlanV0 {
  const changedPaths = [...new Set(input.changedPaths.map(normalizePath).filter(Boolean))]
    .filter((candidate) => !candidate.startsWith(".personal_docs/"))
    .toSorted();
  const pathPlane = buildAffectedCheckPlan(changedPaths);
  const manifestRows = input.scanManifest.scanners.filter((row) => row.disposition !== "RETIRED");
  const knownScannerPaths = new Set(manifestRows.map((row) => row.scannerPath));
  const missingScannerPaths = input.expectedScannerPaths
    .filter((scannerPath) => !knownScannerPaths.has(scannerPath))
    .toSorted();
  const knownGateIds = new Set(input.knownGateIds);
  const declaredGateIds = [
    ...manifestRows.flatMap((row) => ("gateIds" in row ? row.gateIds : [])),
    ...input.writerRegistry.artifacts.flatMap((row) => [
      ...row.writerGateIds,
      ...row.consumerGateIds,
    ]),
  ];
  const unknownGateIds = [...new Set(declaredGateIds)]
    .filter((gateId) => !knownGateIds.has(gateId))
    .toSorted();
  const globalSurfaceChange = changedPaths.some(
    (candidate) =>
      EVIDENCE_GLOBAL_INPUTS.has(candidate) ||
      candidate.startsWith("scripts/surfaces/") ||
      candidate.startsWith("packages/check-orchestrator/src/evidence/predicates/"),
  );
  const fallbackReasons = [
    ...missingScannerPaths.map(
      (scannerPath) => `detected scanner lacks a surface row: ${scannerPath}`,
    ),
    ...unknownGateIds.map((gateId) => `evidence manifest references unknown gate: ${gateId}`),
  ];
  const fallbackToFullEvidence = fallbackReasons.length > 0;

  const affectedScannerPaths = new Set<string>();
  const excludedScannerPaths = new Set<string>();
  for (const row of manifestRows) {
    const affected =
      fallbackToFullEvidence || globalSurfaceChange || scannerRowMatchesAnyPath(row, changedPaths);
    (affected ? affectedScannerPaths : excludedScannerPaths).add(row.scannerPath);
  }

  const affectedArtifactPaths = closeAffectedArtifacts(
    input.writerRegistry.artifacts,
    changedPaths,
    affectedScannerPaths,
    fallbackToFullEvidence,
  );
  const writerOrder = orderAffectedArtifacts(input.writerRegistry.artifacts, affectedArtifactPaths);
  const artifactByPath = new Map(
    input.writerRegistry.artifacts.map((row) => [row.artifactPath, row]),
  );
  const gateIds = new Set<string>();
  for (const row of manifestRows) {
    if (affectedScannerPaths.has(row.scannerPath) && "gateIds" in row) {
      for (const gateId of row.gateIds) gateIds.add(gateId);
    }
  }
  for (const artifactPath of affectedArtifactPaths) {
    const row = artifactByPath.get(artifactPath);
    if (!row) continue;
    for (const gateId of [...row.writerGateIds, ...row.consumerGateIds]) gateIds.add(gateId);
  }

  const evidenceRefreshSufficientAlone = input.forceNarrowSufficiencyForTest
    ? true
    : !pathPlane.requiresFullCi && !fallbackToFullEvidence;
  if (pathPlane.requiresFullCi && evidenceRefreshSufficientAlone) {
    throw new Error(
      "affected path plane requires FULL CI; evidence refresh must be marked insufficient-alone",
    );
  }

  const notRefreshed = writerOrder.flatMap((artifactPath) => {
    const row = artifactByPath.get(artifactPath);
    if (!row || (row.classification !== "W3" && row.classification !== "W4")) return [];
    return [
      {
        artifactPath,
        classification: row.classification,
        instruction:
          row.classification === "W3"
            ? (row.procedure ?? ["review the hand-authored artifact procedure"])
            : (row.disposition ?? "reviewer-retire-proposal"),
      },
    ];
  });
  const commitThenRefresh = input.writerRegistry.artifacts.flatMap((row) =>
    row.artifactPath === "rust/product-surface-boundary-reviews.json"
      ? [{ artifactPath: row.artifactPath, steps: row.procedure ?? [] }]
      : [],
  );

  return {
    schemaVersion: "0",
    changedPaths,
    pathPlane,
    evidenceRefreshSufficientAlone,
    fallbackToFullEvidence,
    fallbackReasons,
    affectedScannerPaths: [...affectedScannerPaths].toSorted(),
    excludedScannerPaths: [...excludedScannerPaths].toSorted(),
    affectedArtifactPaths: [...affectedArtifactPaths].toSorted(),
    writerOrder,
    gateIds: [...gateIds].toSorted(),
    notRefreshed,
    notPreviewableInputs: changedPaths.length > 0 ? input.writerRegistry.notPreviewableInputs : [],
    commitThenRefresh,
  };
}

function scannerRowMatchesAnyPath(
  row: Exclude<EvidenceScannerManifestRowV0, { readonly disposition: "RETIRED" }>,
  changedPaths: readonly string[],
): boolean {
  return changedPaths.some((changedPath) => {
    if (changedPath === row.scannerPath) return true;
    if (row.disposition === "UNMIGRATED") return true;
    if (row.disposition === "FALSE-POSITIVE") return false;
    if (changedPath === row.surfaceModulePath) return true;
    return scanSurfaceMatchesPath(
      (row as unknown as MigratedScanSurfaceDeclaration).spec,
      changedPath,
    );
  });
}

function closeAffectedArtifacts(
  artifacts: readonly EvidenceArtifactRowV0[],
  changedPaths: readonly string[],
  affectedScannerPaths: ReadonlySet<string>,
  full: boolean,
): ReadonlySet<string> {
  const affected = new Set<string>();
  for (const row of artifacts) {
    if (
      full ||
      changedPaths.includes(row.artifactPath) ||
      row.writerScripts.some((writerScript) => changedPaths.includes(writerScript)) ||
      row.inputPaths.some((inputPath) => changedPaths.includes(inputPath)) ||
      row.inputScannerPaths.some((scannerPath) => affectedScannerPaths.has(scannerPath))
    ) {
      affected.add(row.artifactPath);
    }
  }
  let changed = true;
  while (changed) {
    changed = false;
    for (const row of artifacts) {
      const writerCommandKey = row.writeCommand ? JSON.stringify(row.writeCommand) : null;
      const sharesAffectedWriter =
        writerCommandKey !== null &&
        artifacts.some(
          (candidate) =>
            affected.has(candidate.artifactPath) &&
            candidate.writeCommand !== undefined &&
            JSON.stringify(candidate.writeCommand) === writerCommandKey,
        );
      if (
        !affected.has(row.artifactPath) &&
        (row.inputArtifactPaths.some((dependency) => affected.has(dependency)) ||
          sharesAffectedWriter)
      ) {
        affected.add(row.artifactPath);
        changed = true;
      }
    }
  }
  return affected;
}

export function orderAffectedArtifacts(
  artifacts: readonly EvidenceArtifactRowV0[],
  affectedPaths: ReadonlySet<string>,
): readonly string[] {
  const byPath = new Map(artifacts.map((row) => [row.artifactPath, row]));
  const permanent = new Set<string>();
  const temporary = new Set<string>();
  const ordered: string[] = [];
  const visit = (artifactPath: string): void => {
    if (permanent.has(artifactPath)) return;
    const row = byPath.get(artifactPath);
    if (!row) throw new Error(`artifact DAG references unknown node: ${artifactPath}`);
    if (temporary.has(artifactPath)) {
      if (row.writerNodeKind === "self-ratchet") return;
      throw new Error(`evidence artifact DAG cycle: ${artifactPath}`);
    }
    temporary.add(artifactPath);
    for (const dependency of row.inputArtifactPaths) {
      if (dependency === artifactPath && row.writerNodeKind === "self-ratchet") continue;
      if (affectedPaths.has(dependency)) visit(dependency);
    }
    temporary.delete(artifactPath);
    permanent.add(artifactPath);
    ordered.push(artifactPath);
  };
  for (const artifactPath of [...affectedPaths].toSorted()) visit(artifactPath);
  return ordered;
}

function normalizePath(value: string): string {
  return value.trim().replaceAll("\\", "/").replace(/^\.\//u, "");
}
