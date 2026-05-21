import path from "node:path";
import {
  assertCheckerCanonicalCandidateEqual,
  deriveCheckerCanonicalCandidate,
} from "../packages/cme-checker/src";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  runShadowCheckerStyleRecoveryCanonicalCandidate,
  type CheckerStyleRecoveryCanonicalCandidateBundleV0,
} from "./rust-shadow-shared";

const STYLE_RECOVERY_CODES = new Set([
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "missing-keyframes",
  "missing-sass-symbol",
]);

const REPO_ROOT = process.cwd();
const STYLELINT_SMOKE_ROOT = path.join(REPO_ROOT, "test/_fixtures/stylelint-plugin-smoke");

const STYLE_RECOVERY_CORPUS: readonly ContractParityEntry[] = [
  {
    label: "stylelint-smoke-composes-missing-module",
    workspace: {
      workspaceRoot: STYLELINT_SMOKE_ROOT,
      sourceFilePaths: [],
      styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingModule.module.css")],
    },
    filters: {
      preset: "changed-style",
      category: "style",
      severity: "all",
      includeBundles: ["style-recovery"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "stylelint-smoke-composes-missing-selector",
    workspace: {
      workspaceRoot: STYLELINT_SMOKE_ROOT,
      sourceFilePaths: [],
      styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ComposesMissingSelector.module.css")],
    },
    filters: {
      preset: "changed-style",
      category: "style",
      severity: "all",
      includeBundles: ["style-recovery"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "stylelint-smoke-value-missing-module",
    workspace: {
      workspaceRoot: STYLELINT_SMOKE_ROOT,
      sourceFilePaths: [],
      styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingModule.module.css")],
    },
    filters: {
      preset: "changed-style",
      category: "style",
      severity: "all",
      includeBundles: ["style-recovery"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "stylelint-smoke-value-missing-imported",
    workspace: {
      workspaceRoot: STYLELINT_SMOKE_ROOT,
      sourceFilePaths: [],
      styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/ValueMissingImported.module.css")],
    },
    filters: {
      preset: "changed-style",
      category: "style",
      severity: "all",
      includeBundles: ["style-recovery"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
  {
    label: "stylelint-smoke-keyframes-missing",
    workspace: {
      workspaceRoot: STYLELINT_SMOKE_ROOT,
      sourceFilePaths: [],
      styleFilePaths: [path.join(STYLELINT_SMOKE_ROOT, "src/KeyframesMissing.module.css")],
    },
    filters: {
      preset: "changed-style",
      category: "style",
      severity: "all",
      includeBundles: ["style-recovery"],
      includeCodes: [],
      excludeCodes: [],
    },
  },
] as const;

void (async () => {
  for (const entry of STYLE_RECOVERY_CORPUS) {
    process.stdout.write(`== rust-checker-style-recovery:${entry.label} ==\n`);
    // oxlint-disable-next-line no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsCheckerStyleRecoveryCanonicalCandidate(snapshot);
    // oxlint-disable-next-line no-await-in-loop
    const actual = await runShadowCheckerStyleRecoveryCanonicalCandidate(snapshot);
    assertCheckerCanonicalCandidateEqual(
      actual,
      expected,
      `${entry.label}: checker style-recovery canonical candidate mismatch`,
    );
    process.stdout.write(
      `findings=${actual.summary.total} files=${actual.distinctFileCount} codes=${JSON.stringify(actual.codeCounts)}\n\n`,
    );
  }
})();

function deriveTsCheckerStyleRecoveryCanonicalCandidate(
  snapshot: Awaited<ReturnType<typeof buildContractParitySnapshot>>,
): CheckerStyleRecoveryCanonicalCandidateBundleV0 {
  return deriveCheckerCanonicalCandidate(snapshot, {
    bundle: "style-recovery",
    category: "style",
    codes: STYLE_RECOVERY_CODES,
    extraFields: ["analysisReason", "valueCertaintyShapeLabel"],
  }) as CheckerStyleRecoveryCanonicalCandidateBundleV0;
}
