import path from "node:path";
import { deriveCheckerCanonicalCandidate } from "../packages/cme-checker/src";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import type { buildContractParitySnapshot } from "./contract-parity-runtime";
import type { CheckerStyleUnusedCanonicalCandidateBundleV0 } from "./rust-shadow-shared";

const STYLE_UNUSED_CODES = new Set(["unused-selector"]);

const REPO_ROOT = process.cwd();
export const STYLE_UNUSED_WORKSPACE_ROOT = path.join(
  REPO_ROOT,
  "test/_fixtures/stylelint-plugin-smoke",
);

export const STYLE_UNUSED_ENTRY: ContractParityEntry = {
  label: "stylelint-smoke-unused-selector",
  workspace: {
    workspaceRoot: STYLE_UNUSED_WORKSPACE_ROOT,
    sourceFilePaths: [path.join(STYLE_UNUSED_WORKSPACE_ROOT, "src/App.tsx")],
    styleFilePaths: [path.join(STYLE_UNUSED_WORKSPACE_ROOT, "src/App.module.css")],
  },
  filters: {
    preset: "changed-style",
    category: "style",
    severity: "all",
    includeBundles: ["style-unused"],
    includeCodes: [],
    excludeCodes: [],
  },
};

export function deriveTsCheckerStyleUnusedCanonicalCandidate(
  snapshot: Awaited<ReturnType<typeof buildContractParitySnapshot>>,
): CheckerStyleUnusedCanonicalCandidateBundleV0 {
  return deriveCheckerCanonicalCandidate(snapshot, {
    bundle: "style-unused",
    category: "style",
    codes: STYLE_UNUSED_CODES,
    extraFields: ["analysisReason", "valueCertaintyShapeLabel"],
  }) as CheckerStyleUnusedCanonicalCandidateBundleV0;
}
