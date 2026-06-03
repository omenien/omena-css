import type { CheckerFindingRecordV1, CheckerReportV1 } from "../../engine-core-ts/src/contracts";
import type {
  CheckerFinding,
  CheckerSeverity,
} from "../../engine-core-ts/src/core/checker/contracts";
import type {
  WorkspaceCheckCommandFilters,
  WorkspaceCheckCommandPreset,
  WorkspaceCheckResult,
} from "../../engine-host-node/src/checker-host";
import type { CheckerStyleRecoveryCanonicalProducerSignalV0 } from "./rust-style-recovery-consumer";
import type { CheckerSourceMissingCanonicalProducerSignalV0 } from "./rust-source-missing-consumer";
import type { CheckerStyleUnusedCanonicalProducerSignalV0 } from "./rust-style-unused-consumer";
import type { RustFlowAnalysisConsumerV0 } from "./rust-flow-analysis-consumer";

export type CheckerReportJsonFinding = CheckerFindingRecordV1;
const STYLE_RECOVERY_CODES = new Set([
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "missing-keyframes",
  "missing-sass-symbol",
]);
const SOURCE_MISSING_CODES = new Set([
  "missing-module",
  "missing-static-class",
  "missing-template-prefix",
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
]);
const STYLE_UNUSED_CODES = new Set(["unused-selector"]);

export interface CheckerReportJsonV1 {
  readonly schemaVersion: "1";
  readonly reportVersion: CheckerReportV1["version"];
  readonly tool: "omena/checker";
  readonly workspaceRoot: string;
  readonly filters: {
    readonly preset: WorkspaceCheckCommandPreset | null;
    readonly category: CheckerFinding["category"] | "all";
    readonly severity: CheckerSeverity | "all";
    readonly includeBundles: readonly string[];
    readonly includeCodes: readonly string[];
    readonly excludeCodes: readonly string[];
  };
  readonly sourceFiles: readonly string[];
  readonly styleFiles: readonly string[];
  readonly summary: {
    readonly warnings: number;
    readonly hints: number;
    readonly total: number;
  };
  readonly findings: readonly CheckerReportJsonFinding[];
  readonly rustStyleRecoveryCanonicalProducer?: CheckerStyleRecoveryCanonicalProducerSignalV0;
  readonly rustStyleRecoveryConsistency?: RustStyleRecoveryConsistencyV0;
  readonly rustSourceMissingCanonicalProducer?: CheckerSourceMissingCanonicalProducerSignalV0;
  readonly rustSourceMissingConsistency?: RustSourceMissingConsistencyV0;
  readonly rustStyleUnusedCanonicalProducer?: CheckerStyleUnusedCanonicalProducerSignalV0;
  readonly rustStyleUnusedConsistency?: RustStyleUnusedConsistencyV0;
  readonly rustFlowAnalysisConsumer?: RustFlowAnalysisConsumerV0;
}

export interface RustStyleRecoveryConsistencyV0 {
  readonly schemaVersion: "0";
  readonly bundle: "style-recovery";
  readonly tsFindingCount: number;
  readonly rustFindingCount: number;
  readonly countsMatch: boolean;
  readonly findingsMatch: boolean;
  readonly mismatchedCodes: readonly string[];
}

export interface RustSourceMissingConsistencyV0 {
  readonly schemaVersion: "0";
  readonly bundle: "source-missing";
  readonly tsFindingCount: number;
  readonly rustFindingCount: number;
  readonly countsMatch: boolean;
  readonly findingsMatch: boolean;
  readonly mismatchedCodes: readonly string[];
}

export interface RustStyleUnusedConsistencyV0 {
  readonly schemaVersion: "0";
  readonly bundle: "style-unused";
  readonly tsFindingCount: number;
  readonly rustFindingCount: number;
  readonly countsMatch: boolean;
  readonly findingsMatch: boolean;
  readonly mismatchedCodes: readonly string[];
}

const CHECKER_JSON_SCHEMA_VERSION = "1" as const;
const CHECKER_TOOL_NAME = "omena/checker" as const;

export function buildCheckerJsonReport(
  result: WorkspaceCheckResult,
  report: CheckerReportV1,
  workspaceRoot: string,
  filters: WorkspaceCheckCommandFilters,
  rustStyleRecoveryCanonicalProducer?: CheckerStyleRecoveryCanonicalProducerSignalV0,
  rustSourceMissingCanonicalProducer?: CheckerSourceMissingCanonicalProducerSignalV0,
  rustStyleUnusedCanonicalProducer?: CheckerStyleUnusedCanonicalProducerSignalV0,
  rustFlowAnalysisConsumer?: RustFlowAnalysisConsumerV0,
): CheckerReportJsonV1 {
  const rustStyleRecoveryConsistency = rustStyleRecoveryCanonicalProducer
    ? deriveRustStyleRecoveryConsistency(report, rustStyleRecoveryCanonicalProducer)
    : undefined;
  const rustSourceMissingConsistency = rustSourceMissingCanonicalProducer
    ? deriveRustSourceMissingConsistency(report, rustSourceMissingCanonicalProducer)
    : undefined;
  const rustStyleUnusedConsistency = rustStyleUnusedCanonicalProducer
    ? deriveRustStyleUnusedConsistency(report, rustStyleUnusedCanonicalProducer)
    : undefined;

  return {
    schemaVersion: CHECKER_JSON_SCHEMA_VERSION,
    reportVersion: report.version,
    tool: CHECKER_TOOL_NAME,
    workspaceRoot,
    filters: {
      preset: filters.preset,
      category: filters.category,
      severity: filters.severity,
      includeBundles: filters.includeBundles,
      includeCodes: filters.includeCodes,
      excludeCodes: filters.excludeCodes,
    },
    sourceFiles: result.sourceFiles,
    styleFiles: result.styleFiles,
    summary: report.summary,
    findings: report.findings,
    ...(rustStyleRecoveryCanonicalProducer ? { rustStyleRecoveryCanonicalProducer } : {}),
    ...(rustStyleRecoveryConsistency ? { rustStyleRecoveryConsistency } : {}),
    ...(rustSourceMissingCanonicalProducer ? { rustSourceMissingCanonicalProducer } : {}),
    ...(rustSourceMissingConsistency ? { rustSourceMissingConsistency } : {}),
    ...(rustStyleUnusedCanonicalProducer ? { rustStyleUnusedCanonicalProducer } : {}),
    ...(rustStyleUnusedConsistency ? { rustStyleUnusedConsistency } : {}),
    ...(rustFlowAnalysisConsumer ? { rustFlowAnalysisConsumer } : {}),
  };
}

function deriveRustStyleRecoveryConsistency(
  report: CheckerReportV1,
  rustStyleRecoveryCanonicalProducer: CheckerStyleRecoveryCanonicalProducerSignalV0,
): RustStyleRecoveryConsistencyV0 {
  const tsFindings = report.findings
    .filter((finding) => finding.category === "style" && STYLE_RECOVERY_CODES.has(finding.code))
    .map((finding) => {
      const result = {
        filePath: finding.filePath,
        code: finding.code,
        severity: finding.severity,
        range: finding.range,
        message: finding.message,
      };
      return Object.assign(
        result,
        finding.analysisReason ? { analysisReason: finding.analysisReason } : {},
        finding.valueCertaintyShapeLabel
          ? { valueCertaintyShapeLabel: finding.valueCertaintyShapeLabel }
          : {},
      );
    })
    .toSorted(compareStyleRecoveryFinding);

  const rustFindings = [...rustStyleRecoveryCanonicalProducer.canonicalCandidate.findings].toSorted(
    compareStyleRecoveryFinding,
  );
  const mismatchedCodes = new Set<string>();
  const allCodes = new Set([
    ...tsFindings.map((finding) => finding.code),
    ...rustFindings.map((finding) => finding.code),
  ]);

  for (const code of allCodes) {
    const tsCount = tsFindings.filter((finding) => finding.code === code).length;
    const rustCount = rustFindings.filter((finding) => finding.code === code).length;
    if (tsCount !== rustCount) {
      mismatchedCodes.add(code);
    }
  }

  return {
    schemaVersion: "0",
    bundle: "style-recovery",
    tsFindingCount: tsFindings.length,
    rustFindingCount: rustFindings.length,
    countsMatch: tsFindings.length === rustFindings.length,
    findingsMatch: JSON.stringify(tsFindings) === JSON.stringify(rustFindings),
    mismatchedCodes: [...mismatchedCodes].toSorted(),
  };
}

function deriveRustSourceMissingConsistency(
  report: CheckerReportV1,
  rustSourceMissingCanonicalProducer: CheckerSourceMissingCanonicalProducerSignalV0,
): RustSourceMissingConsistencyV0 {
  const tsFindings = report.findings
    .filter((finding) => finding.category === "source" && SOURCE_MISSING_CODES.has(finding.code))
    .map((finding) => {
      const result = {
        filePath: finding.filePath,
        code: finding.code,
        severity: finding.severity,
        range: finding.range,
        message: finding.message,
      };
      return Object.assign(
        result,
        finding.analysisReason ? { analysisReason: finding.analysisReason } : {},
        finding.valueCertaintyShapeLabel
          ? { valueCertaintyShapeLabel: finding.valueCertaintyShapeLabel }
          : {},
        finding.valueDomainDerivation
          ? { valueDomainDerivation: finding.valueDomainDerivation }
          : {},
      );
    })
    .toSorted(compareStyleRecoveryFinding);

  const rustFindings = [...rustSourceMissingCanonicalProducer.canonicalCandidate.findings].toSorted(
    compareStyleRecoveryFinding,
  );
  const mismatchedCodes = new Set<string>();
  const allCodes = new Set([
    ...tsFindings.map((finding) => finding.code),
    ...rustFindings.map((finding) => finding.code),
  ]);

  for (const code of allCodes) {
    const tsCount = tsFindings.filter((finding) => finding.code === code).length;
    const rustCount = rustFindings.filter((finding) => finding.code === code).length;
    if (tsCount !== rustCount) {
      mismatchedCodes.add(code);
    }
  }

  return {
    schemaVersion: "0",
    bundle: "source-missing",
    tsFindingCount: tsFindings.length,
    rustFindingCount: rustFindings.length,
    countsMatch: tsFindings.length === rustFindings.length,
    findingsMatch: JSON.stringify(tsFindings) === JSON.stringify(rustFindings),
    mismatchedCodes: [...mismatchedCodes].toSorted(),
  };
}

function deriveRustStyleUnusedConsistency(
  report: CheckerReportV1,
  rustStyleUnusedCanonicalProducer: CheckerStyleUnusedCanonicalProducerSignalV0,
): RustStyleUnusedConsistencyV0 {
  const tsFindings = report.findings
    .filter((finding) => finding.category === "style" && STYLE_UNUSED_CODES.has(finding.code))
    .map((finding) => {
      const result = {
        filePath: finding.filePath,
        code: finding.code,
        severity: finding.severity,
        range: finding.range,
        message: finding.message,
      };
      return Object.assign(
        result,
        finding.analysisReason ? { analysisReason: finding.analysisReason } : {},
        finding.valueCertaintyShapeLabel
          ? { valueCertaintyShapeLabel: finding.valueCertaintyShapeLabel }
          : {},
      );
    })
    .toSorted(compareStyleRecoveryFinding);

  const rustFindings = [...rustStyleUnusedCanonicalProducer.canonicalCandidate.findings].toSorted(
    compareStyleRecoveryFinding,
  );
  const mismatchedCodes = new Set<string>();
  const allCodes = new Set([
    ...tsFindings.map((finding) => finding.code),
    ...rustFindings.map((finding) => finding.code),
  ]);

  for (const code of allCodes) {
    const tsCount = tsFindings.filter((finding) => finding.code === code).length;
    const rustCount = rustFindings.filter((finding) => finding.code === code).length;
    if (tsCount !== rustCount) {
      mismatchedCodes.add(code);
    }
  }

  return {
    schemaVersion: "0",
    bundle: "style-unused",
    tsFindingCount: tsFindings.length,
    rustFindingCount: rustFindings.length,
    countsMatch: tsFindings.length === rustFindings.length,
    findingsMatch: JSON.stringify(tsFindings) === JSON.stringify(rustFindings),
    mismatchedCodes: [...mismatchedCodes].toSorted(),
  };
}

function compareStyleRecoveryFinding(
  left: {
    readonly filePath: string;
    readonly code: string;
    readonly severity: string;
    readonly range: {
      readonly start: {
        readonly line: number;
        readonly character: number;
      };
      readonly end: {
        readonly line: number;
        readonly character: number;
      };
    };
    readonly message: string;
    readonly analysisReason?: string;
    readonly valueCertaintyShapeLabel?: string;
  },
  right: {
    readonly filePath: string;
    readonly code: string;
    readonly severity: string;
    readonly range: {
      readonly start: {
        readonly line: number;
        readonly character: number;
      };
      readonly end: {
        readonly line: number;
        readonly character: number;
      };
    };
    readonly message: string;
    readonly analysisReason?: string;
    readonly valueCertaintyShapeLabel?: string;
  },
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
    (left.analysisReason ?? "").localeCompare(right.analysisReason ?? "") ||
    (left.valueCertaintyShapeLabel ?? "").localeCompare(right.valueCertaintyShapeLabel ?? "")
  );
}
