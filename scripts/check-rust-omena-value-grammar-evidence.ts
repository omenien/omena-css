import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  buildCoverageGapReportFromRepo,
  findCoverageGapRows,
  VALUE_GRAMMAR_EVIDENCE_PATH,
  type ValueGrammarEvidenceReport,
} from "./coverage-gap-report";

const root = process.cwd();
const fixturePath = "crates/omena-abstract-value/tests/fixtures/value-grammar-seeds.json";
const generated = execFileSync(
  "cargo",
  [
    "run",
    "--quiet",
    "-p",
    "omena-abstract-value",
    "--example",
    "value_grammar_evidence",
    "--",
    fixturePath,
  ],
  {
    cwd: path.join(root, "rust"),
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  },
);
const committed = readFileSync(path.join(root, VALUE_GRAMMAR_EVIDENCE_PATH), "utf8");
assert.equal(
  generated,
  committed,
  `${VALUE_GRAMMAR_EVIDENCE_PATH} is stale; regenerate it through the Rust matcher`,
);

const evidence = JSON.parse(generated) as ValueGrammarEvidenceReport;
assert.equal(evidence.caseCount, 7);
assert.equal(evidence.cases.length, evidence.caseCount);
assert.equal(evidence.allExpectationsSatisfied, true);
assert.ok(evidence.cases.every((entry) => entry.expectationSatisfied));
assert.ok(
  evidence.cases.some(
    (entry) =>
      entry.expectedValid &&
      entry.verdict === "matched" &&
      entry.typed &&
      entry.rootNodeKind === "list",
  ),
  "evidence must contain a matched typed list",
);
assert.ok(
  evidence.cases.some(
    (entry) =>
      entry.expectedValid &&
      entry.verdict === "matched" &&
      entry.typed &&
      entry.rootNodeKind === "function",
  ),
  "evidence must contain a matched typed function",
);
assert.ok(
  evidence.cases.some(
    (entry) =>
      !entry.expectedValid && entry.verdict === "unmatched" && entry.rawPreserved && !entry.typed,
  ),
  "evidence must contain a definite mismatch with byte-preserving Raw fallback",
);

const coverage = buildCoverageGapReportFromRepo(root);
assert.equal(coverage.summary.tierCounts.T1, 1);
assert.equal(coverage.summary.tierCounts.T2, 3);
assert.equal(findCoverageGapRows(coverage, "properties", "color")[0]?.capabilityTier, "T1");
for (const property of ["border-top", "font-family", "transform"]) {
  const [row] = findCoverageGapRows(coverage, "properties", property);
  assert.equal(row?.capabilityTier, "T2");
  assert.equal(row?.measurements.typedProjectionEvidence, true);
  assert.equal(row?.measurements.grammarValidationEvidence, true);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "rust.omena-value-grammar-evidence",
      cases: evidence.caseCount,
      typedTierRows: coverage.summary.tierCounts.T1,
      validatedTierRows: coverage.summary.tierCounts.T2,
      violations: 0,
    },
    null,
    2,
  )}\n`,
);
