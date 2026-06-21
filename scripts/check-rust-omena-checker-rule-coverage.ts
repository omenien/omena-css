import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const packageJsonSource = readFileSync("package.json", "utf8");
const runnerSource = readFileSync("rust/crates/engine-shadow-runner/src/main.rs", "utf8");

const REQUIRED_RULE_NAMES = [
  "no-unknown-dynamic-class",
  "no-imprecise-value",
  "no-impossible-selector",
  "missing-module",
  "missing-static-class",
  "missing-template-prefix",
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
  "unused-selector",
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "missing-keyframes",
  "missing-custom-property",
  "missing-sass-symbol",
  "unreachable-declaration",
  "dead-cascade-layer",
  "iacvt-prone",
  "circular-var",
  "registered-property-type-mismatch",
  "invalid-property-value",
  "unspecified-cascade-tie",
  "designer-intent-inconsistency",
  "cascade.smt-violation",
  "design-system-mdl-budget",
  "streaming-ifds-precision-parity",
  "rg-flow-relevant-operator",
  "replica-ensemble-inconsistency",
  "cascade.deep-conflict",
  "cascade.unreachable-rule",
  "categorical-cascade-evidence-inconsistency",
] as const;

const MECHANISM_RULE_NAMES = [
  "designer-intent-inconsistency",
  "cascade.smt-violation",
  "design-system-mdl-budget",
  "streaming-ifds-precision-parity",
  "rg-flow-relevant-operator",
  "replica-ensemble-inconsistency",
  "cascade.deep-conflict",
  "cascade.unreachable-rule",
  "categorical-cascade-evidence-inconsistency",
] as const;

assert.ok(
  packageJsonSource.includes("check:rust-omena-checker-rule-coverage"),
  "package.json must expose the omena-checker rule coverage gate",
);
assert.ok(
  packageJsonSource.includes("rust/omena-checker/rule-coverage"),
  "rust/omena-checker/boundary must include the rule coverage gate",
);
assert.ok(
  runnerSource.includes("omena-checker-rule-enforcement-coverage"),
  "engine-shadow-runner must expose omena-checker rule enforcement coverage",
);

const coverage = readRuleCoverage();
assert.equal(coverage.product, "omena-checker.rule-enforcement-coverage");
assert.equal(coverage.registeredRuleCount, 32);
assert.equal(coverage.mappedRuleCount, coverage.registeredRuleCount);
assert.equal(coverage.coveragePassed, true);
assert.deepEqual(coverage.missingRuleNames, []);
assert.deepEqual(coverage.extraRuleNames, []);
assert.equal(coverage.productDiagnosticGateRuleCount, 13);
assert.equal(coverage.directEvaluatorRuleCount, 10);
assert.equal(coverage.mechanismEvaluatorRuleCount, 9);

const evidenceByRule = new Map(coverage.evidence.map((entry) => [entry.ruleCodeName, entry]));
for (const ruleName of REQUIRED_RULE_NAMES) {
  assert.ok(evidenceByRule.has(ruleName), `missing enforcement evidence for ${ruleName}`);
}
for (const ruleName of MECHANISM_RULE_NAMES) {
  const evidence = evidenceByRule.get(ruleName);
  assert.ok(evidence, `missing mechanism evidence for ${ruleName}`);
  assert.equal(evidence.evidenceKind, "mechanismEvaluator");
  assert.ok(
    evidence.mechanismProducts.length > 0,
    `${ruleName} must name at least one mechanism product`,
  );
  assert.ok(evidence.productPath.length > 0, `${ruleName} must name a product path`);
  assert.ok(evidence.emitFixture.length > 0, `${ruleName} must name an emit fixture`);
  assert.ok(evidence.clearFixture.length > 0, `${ruleName} must name a clear fixture`);
}

process.stdout.write(
  [
    "validated omena-checker rule coverage:",
    `registered=${coverage.registeredRuleCount}`,
    `mapped=${coverage.mappedRuleCount}`,
    `productDiagnosticGate=${coverage.productDiagnosticGateRuleCount}`,
    `directEvaluator=${coverage.directEvaluatorRuleCount}`,
    `mechanismEvaluator=${coverage.mechanismEvaluatorRuleCount}`,
    "runner=engine-shadow-runner",
  ].join(" "),
);
process.stdout.write("\n");

function readRuleCoverage(): RuleCoverageSummary {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-rule-enforcement-coverage",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner rule coverage command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as RuleCoverageSummary;
}

interface RuleCoverageSummary {
  readonly product: string;
  readonly registeredRuleCount: number;
  readonly mappedRuleCount: number;
  readonly directEvaluatorRuleCount: number;
  readonly mechanismEvaluatorRuleCount: number;
  readonly productDiagnosticGateRuleCount: number;
  readonly missingRuleNames: readonly string[];
  readonly extraRuleNames: readonly string[];
  readonly coveragePassed: boolean;
  readonly evidence: readonly RuleCoverageEvidence[];
}

interface RuleCoverageEvidence {
  readonly ruleCodeName: string;
  readonly evidenceKind: string;
  readonly productPath: string;
  readonly emitFixture: string;
  readonly clearFixture: string;
  readonly mechanismProducts: readonly string[];
}
