import { deepStrictEqual } from "node:assert";
import path from "node:path";
import { buildCheckerBoundedGate } from "../packages/cme-checker/src";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  runShadowCheckerStyleRecoveryCanonicalCandidate,
  runShadowCheckerStyleRecoveryCanonicalProducer,
  type CheckerStyleRecoveryCanonicalProducerSignalV0,
} from "./rust-shadow-shared";

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
    process.stdout.write(`== rust-checker-style-recovery-producer:${entry.label} ==\n`);
    // oxlint-disable-next-line no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    // oxlint-disable-next-line no-await-in-loop
    const canonicalCandidate = await runShadowCheckerStyleRecoveryCanonicalCandidate(snapshot);
    // oxlint-disable-next-line no-await-in-loop
    const actual = await runShadowCheckerStyleRecoveryCanonicalProducer(snapshot);

    const expected: CheckerStyleRecoveryCanonicalProducerSignalV0 = {
      schemaVersion: "0",
      inputVersion: canonicalCandidate.inputVersion,
      canonicalCandidate,
      boundedCheckerGate: buildCheckerBoundedGate("style-recovery"),
    };

    deepStrictEqual(
      actual,
      expected,
      `${entry.label}: checker style-recovery canonical producer mismatch`,
    );
    process.stdout.write(
      `findings=${actual.canonicalCandidate.summary.total} releaseGate=${actual.boundedCheckerGate.includedInRustReleaseBundle}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
