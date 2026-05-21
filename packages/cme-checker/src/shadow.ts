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

export interface CmeCheckerDeriveOptionsV0<TBundle extends string = string> {
  readonly bundle: TBundle;
  readonly category: string;
  readonly codes: ReadonlySet<string>;
  readonly extraFields?: readonly string[];
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
