import { deepStrictEqual, ok, strictEqual } from "node:assert";
import {
  assertCheckerCanonicalCandidateEqual,
  deriveCheckerCanonicalCandidate,
  diffCheckerCanonicalCandidate,
} from "../packages/cme-checker/src";

const snapshot = {
  input: {
    version: "fixture-input-v0",
  },
  output: {
    checkerReport: {
      version: "fixture-report-v0",
      findings: [
        {
          category: "source",
          code: "missing-static-class",
          severity: "warning",
          filePath: "src/App.tsx",
          range: range(2, 4, 2, 16),
          message: "CSS Module selector '.ghost' not found.",
          analysisReason: "fixture-source-missing",
          valueCertaintyShapeLabel: "exact",
        },
        {
          category: "style",
          code: "unused-selector",
          severity: "hint",
          filePath: "src/App.module.css",
          range: range(1, 0, 1, 6),
          message: "CSS Module selector '.card' is not referenced.",
        },
        {
          category: "source",
          code: "missing-module",
          severity: "warning",
          filePath: "src/Missing.tsx",
          range: range(1, 7, 1, 31),
          message: "CSS Module import target was not found.",
          analysisReason: "fixture-module-missing",
        },
      ],
    },
  },
} as const;

const bundle = deriveCheckerCanonicalCandidate(snapshot, {
  bundle: "source-missing",
  category: "source",
  codes: new Set(["missing-module", "missing-static-class"]),
  extraFields: ["analysisReason", "valueCertaintyShapeLabel"],
});

strictEqual(bundle.schemaVersion, "0");
strictEqual(bundle.bundle, "source-missing");
strictEqual(bundle.inputVersion, "fixture-input-v0");
strictEqual(bundle.reportVersion, "fixture-report-v0");
strictEqual(bundle.summary.warnings, 2);
strictEqual(bundle.summary.hints, 0);
strictEqual(bundle.summary.total, 2);
strictEqual(bundle.distinctFileCount, 2);
deepStrictEqual(bundle.codeCounts, {
  "missing-module": 1,
  "missing-static-class": 1,
});
deepStrictEqual(
  bundle.findings.map((finding) => finding.code),
  ["missing-static-class", "missing-module"],
);
strictEqual(bundle.findings[0]?.analysisReason, "fixture-source-missing");
strictEqual(bundle.findings[0]?.valueCertaintyShapeLabel, "exact");
const changedBundle = {
  ...bundle,
  findings: bundle.findings.map((finding, index) =>
    index === 0 ? { ...finding, message: "changed fixture message" } : finding,
  ),
};
const diffReport = diffCheckerCanonicalCandidate(changedBundle, bundle);
strictEqual(diffReport.product, "cme-checker.canonical-candidate-diff");
strictEqual(diffReport.matches, false);
strictEqual(diffReport.diffCount, 1);
strictEqual(diffReport.diffs[0]?.path, "findings[0].message");
strictEqual(diffReport.diffs[0]?.expected, "CSS Module selector '.ghost' not found.");
strictEqual(diffReport.diffs[0]?.actual, "changed fixture message");
assertCheckerCanonicalCandidateEqual(bundle, bundle, "cme-checker-boundary self-check");
let mismatchRaised = false;
try {
  assertCheckerCanonicalCandidateEqual(changedBundle, bundle, "cme-checker-boundary drift-check");
} catch (error) {
  mismatchRaised = true;
  ok(String(error).includes("findings[0].message"));
}
strictEqual(mismatchRaised, true);

process.stdout.write(
  JSON.stringify(
    {
      product: "cme-checker.boundary",
      migratedArchetypes: [
        "checker-style-recovery/canonical-candidate",
        "checker-source-missing/canonical-candidate",
        "checker-style-unused/canonical-candidate",
      ],
      package: "@omena/checker",
      diffReport: "cme-checker.canonical-candidate-diff",
      findingCount: bundle.summary.total,
      distinctFileCount: bundle.distinctFileCount,
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function range(startLine: number, startCharacter: number, endLine: number, endCharacter: number) {
  return {
    start: {
      line: startLine,
      character: startCharacter,
    },
    end: {
      line: endLine,
      character: endCharacter,
    },
  };
}
