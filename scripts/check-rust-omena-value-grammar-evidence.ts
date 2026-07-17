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
const checkerSource = readFileSync(
  path.join(root, "rust/crates/omena-checker/src/lib.rs"),
  "utf8",
).split("#[cfg(test)]")[0];
const nativeCssSource = readFileSync(
  path.join(root, "rust/crates/omena-scss-eval/src/native_css.rs"),
  "utf8",
).split("#[cfg(test)]")[0];
const registeredPropertySource = readFileSync(
  path.join(root, "rust/crates/omena-abstract-value/src/registered_property.rs"),
  "utf8",
).split("#[cfg(test)]")[0];

for (const [consumer, source, requiredEntrypoints] of [
  [
    "checker",
    checkerSource,
    ["validate_registered_property_value_v0", "validate_standard_property_value_v0"],
  ],
  ["native CSS evaluator", nativeCssSource, ["validate_registered_property_value_v0"]],
] as const) {
  for (const entrypoint of requiredEntrypoints) {
    assert.ok(source.includes(entrypoint), `${consumer} must consume ${entrypoint}`);
  }
  assert.ok(
    !source.includes("registered_syntax_match("),
    `${consumer} must not bypass typed validation through the compatibility adapter`,
  );
}
assert.match(
  registeredPropertySource,
  /pub fn registered_syntax_match[\s\S]*?validate_registered_property_value_v0/,
  "the registered-property compatibility adapter must delegate to typed validation",
);
assert.match(
  registeredPropertySource,
  /pub fn standard_property_syntax_match[\s\S]*?validate_standard_property_value_v0/,
  "the standard-property compatibility adapter must delegate to typed validation",
);
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
      validationConsumers: 2,
      compatibilityAdapters: 2,
      typedTierRows: coverage.summary.tierCounts.T1,
      validatedTierRows: coverage.summary.tierCounts.T2,
      violations: 0,
    },
    null,
    2,
  )}\n`,
);
