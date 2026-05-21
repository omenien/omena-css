export interface CmeCheckerPositionV0 {
  readonly line: number;
  readonly character: number;
}

export interface CmeCheckerRangeV0 {
  readonly start: CmeCheckerPositionV0;
  readonly end: CmeCheckerPositionV0;
}

export interface CmeCheckerFindingLikeV0 {
  readonly category: string;
  readonly code: string;
  readonly severity: string;
  readonly filePath: string;
  readonly range: CmeCheckerRangeV0;
  readonly message: string;
  readonly [key: string]: unknown;
}

export interface CmeCheckerSnapshotLikeV0 {
  readonly input: {
    readonly version: string;
  };
  readonly output: {
    readonly checkerReport: {
      readonly version: string;
      readonly findings: readonly CmeCheckerFindingLikeV0[];
    };
  };
}

export interface CmeCheckerFindingProjectionV0 {
  readonly filePath: string;
  readonly code: string;
  readonly severity: string;
  readonly range: CmeCheckerRangeV0;
  readonly message: string;
  readonly [key: string]: unknown;
}

type MutableCheckerFindingProjectionV0 = {
  -readonly [Key in keyof CmeCheckerFindingProjectionV0]: CmeCheckerFindingProjectionV0[Key];
};

export interface CmeCheckerCanonicalCandidateBundleV0<TBundle extends string = string> {
  readonly schemaVersion: "0";
  readonly inputVersion: string;
  readonly reportVersion: string;
  readonly bundle: TBundle;
  readonly distinctFileCount: number;
  readonly codeCounts: Readonly<Record<string, number>>;
  readonly summary: {
    readonly warnings: number;
    readonly hints: number;
    readonly total: number;
  };
  readonly findings: readonly CmeCheckerFindingProjectionV0[];
}

const CHECKER_BOUNDED_GATE_BY_BUNDLE = {
  "style-recovery": {
    canonicalCandidateCommand: "pnpm check:rust-checker-style-recovery-canonical-candidate",
    canonicalProducerCommand: "pnpm check:rust-checker-style-recovery-canonical-producer",
    consumerBoundaryCommand: "pnpm check:rust-checker-style-recovery-consumer-boundary",
    boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes",
    promotionReviewCommand: "pnpm check:rust-checker-promotion-review",
    promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence",
    broaderRustLaneCommand: "pnpm check:rust-lane-bundle",
    releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness",
    releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow",
    releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review",
    releaseBundleCommand: "pnpm check:rust-release-bundle",
    minimumBoundedLaneCountForRustLaneBundle: 3,
    minimumBoundedLaneCountForRustReleaseBundle: 3,
    minimumSuccessfulShadowRunsForRustReleaseBundle: 3,
    checkerBundle: "style-recovery",
    releaseGateStage: "enforced",
    includedInRustLaneBundle: true,
    includedInRustReleaseBundle: true,
  },
  "source-missing": {
    canonicalCandidateCommand: "pnpm check:rust-checker-source-missing-canonical-candidate",
    canonicalProducerCommand: "pnpm check:rust-checker-source-missing-canonical-producer",
    consumerBoundaryCommand: "pnpm check:rust-checker-source-missing-consumer-boundary",
    boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes",
    promotionReviewCommand: "pnpm check:rust-checker-promotion-review",
    promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence",
    broaderRustLaneCommand: "pnpm check:rust-lane-bundle",
    releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness",
    releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow",
    releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review",
    releaseBundleCommand: "pnpm check:rust-release-bundle",
    minimumBoundedLaneCountForRustLaneBundle: 3,
    minimumBoundedLaneCountForRustReleaseBundle: 3,
    minimumSuccessfulShadowRunsForRustReleaseBundle: 3,
    checkerBundle: "source-missing",
    releaseGateStage: "enforced",
    includedInRustLaneBundle: true,
    includedInRustReleaseBundle: true,
  },
  "style-unused": {
    canonicalCandidateCommand: "pnpm check:rust-checker-style-unused-canonical-candidate",
    canonicalProducerCommand: "pnpm check:rust-checker-style-unused-canonical-producer",
    consumerBoundaryCommand: "pnpm check:rust-checker-style-unused-consumer-boundary",
    boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes",
    promotionReviewCommand: "pnpm check:rust-checker-promotion-review",
    promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence",
    broaderRustLaneCommand: "pnpm check:rust-lane-bundle",
    releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness",
    releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow",
    releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review",
    releaseBundleCommand: "pnpm check:rust-release-bundle",
    minimumBoundedLaneCountForRustLaneBundle: 3,
    minimumBoundedLaneCountForRustReleaseBundle: 3,
    minimumSuccessfulShadowRunsForRustReleaseBundle: 3,
    checkerBundle: "style-unused",
    releaseGateStage: "enforced",
    includedInRustLaneBundle: true,
    includedInRustReleaseBundle: true,
  },
} as const;

export type CmeCheckerBundleV0 = keyof typeof CHECKER_BOUNDED_GATE_BY_BUNDLE;

export type CmeCheckerBoundedGateV0<TBundle extends CmeCheckerBundleV0 = CmeCheckerBundleV0> =
  (typeof CHECKER_BOUNDED_GATE_BY_BUNDLE)[TBundle];

export interface CmeCheckerDeriveOptionsV0<TBundle extends string = string> {
  readonly bundle: TBundle;
  readonly category: string;
  readonly codes: ReadonlySet<string>;
  readonly extraFields?: readonly string[];
}

export function buildCheckerBoundedGate<TBundle extends CmeCheckerBundleV0>(
  bundle: TBundle,
): CmeCheckerBoundedGateV0<TBundle> {
  return CHECKER_BOUNDED_GATE_BY_BUNDLE[bundle];
}

export function deriveCheckerCanonicalCandidate<TBundle extends string>(
  snapshot: CmeCheckerSnapshotLikeV0,
  options: CmeCheckerDeriveOptionsV0<TBundle>,
): CmeCheckerCanonicalCandidateBundleV0<TBundle> {
  const findings = snapshot.output.checkerReport.findings
    .filter((finding) => finding.category === options.category && options.codes.has(finding.code))
    .map((finding) => projectFinding(finding, options.extraFields ?? []))
    .toSorted(compareCheckerFindings);
  const codeCounts = Object.fromEntries(
    [...options.codes]
      .map((code) => [code, findings.filter((finding) => finding.code === code).length] as const)
      .filter(([, count]) => count > 0),
  );

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    reportVersion: snapshot.output.checkerReport.version,
    bundle: options.bundle,
    distinctFileCount: new Set(findings.map((finding) => finding.filePath)).size,
    codeCounts,
    summary: {
      warnings: findings.filter((finding) => finding.severity === "warning").length,
      hints: findings.filter((finding) => finding.severity === "hint").length,
      total: findings.length,
    },
    findings,
  };
}

export function compareCheckerFindings(
  left: CmeCheckerFindingProjectionV0,
  right: CmeCheckerFindingProjectionV0,
): number {
  return (
    left.filePath.localeCompare(right.filePath) ||
    left.code.localeCompare(right.code) ||
    left.severity.localeCompare(right.severity) ||
    left.range.start.line - right.range.start.line ||
    left.range.start.character - right.range.start.character ||
    left.range.end.line - right.range.end.line ||
    left.range.end.character - right.range.end.character ||
    left.message.localeCompare(right.message) ||
    stringField(left, "analysisReason").localeCompare(stringField(right, "analysisReason")) ||
    stringField(left, "valueCertaintyShapeLabel").localeCompare(
      stringField(right, "valueCertaintyShapeLabel"),
    )
  );
}

function projectFinding(
  finding: CmeCheckerFindingLikeV0,
  extraFields: readonly string[],
): CmeCheckerFindingProjectionV0 {
  const projected: MutableCheckerFindingProjectionV0 = {
    filePath: finding.filePath,
    code: finding.code,
    severity: finding.severity,
    range: finding.range,
    message: finding.message,
  };

  for (const field of extraFields) {
    const value = finding[field];
    if (value !== undefined) {
      projected[field] = value;
    }
  }

  return projected;
}

function stringField(finding: CmeCheckerFindingProjectionV0, field: string): string {
  const value = finding[field];
  return typeof value === "string" ? value : "";
}
