import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const RUNNER_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/src/main.rs");
const RUNNER_CARGO_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/Cargo.toml");
const PACKAGE_JSON_PATH = path.join(process.cwd(), "package.json");

const runnerSource = readFileSync(RUNNER_PATH, "utf8");
const runnerCargoToml = readFileSync(RUNNER_CARGO_PATH, "utf8");
const packageJsonSource = readFileSync(PACKAGE_JSON_PATH, "utf8");
const commandBodies = extractCommandBodies(runnerSource);

const checkerMTierBody = commandBodies.get("omena-checker-m-tier-evaluations");
const checkerCascadeBody = commandBodies.get("omena-checker-cascade-evaluations");
const checkerGrnBody = commandBodies.get("omena-checker-grn-evaluations");
const checkerSmtBody = commandBodies.get("omena-checker-smt-evaluations");
assert.ok(
  checkerMTierBody,
  "missing engine-shadow-runner command arm: omena-checker-m-tier-evaluations",
);
assert.ok(
  checkerCascadeBody,
  "missing engine-shadow-runner command arm: omena-checker-cascade-evaluations",
);
assert.ok(
  checkerGrnBody,
  "missing engine-shadow-runner command arm: omena-checker-grn-evaluations",
);
assert.ok(
  checkerSmtBody,
  "missing engine-shadow-runner command arm: omena-checker-smt-evaluations",
);
assert.ok(
  checkerMTierBody.includes("OmenaCheckerMTierEvaluationInputV0"),
  "omena-checker-m-tier-evaluations must deserialize the checker M-tier input product",
);
assert.ok(
  checkerCascadeBody.includes("OmenaCheckerCascadeInputV0"),
  "omena-checker-cascade-evaluations must deserialize the checker cascade input product",
);
assert.ok(
  checkerGrnBody.includes("OmenaCheckerGrnInputV0"),
  "omena-checker-grn-evaluations must deserialize the checker GRN input product",
);
assert.ok(
  checkerSmtBody.includes("OmenaCheckerSmtInputV0"),
  "omena-checker-smt-evaluations must deserialize the checker SMT input product",
);
assert.ok(
  checkerMTierBody.includes("summarize_omena_checker_m_tier_evaluations"),
  "omena-checker-m-tier-evaluations must route through the runner-owned checker summary wrapper",
);
assert.ok(
  checkerCascadeBody.includes("summarize_omena_checker_cascade_evaluations"),
  "omena-checker-cascade-evaluations must route through the runner-owned checker cascade summary wrapper",
);
assert.ok(
  checkerGrnBody.includes("summarize_omena_checker_grn_evaluations"),
  "omena-checker-grn-evaluations must route through the runner-owned checker GRN summary wrapper",
);
assert.ok(
  checkerSmtBody.includes("summarize_omena_checker_smt_evaluations"),
  "omena-checker-smt-evaluations must route through the runner-owned checker SMT summary wrapper",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_m_tier_rules"),
  "engine-shadow-runner must call omena-checker's M-tier evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_cascade_rules"),
  "engine-shadow-runner must call omena-checker's cascade-aware evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_grn_rules"),
  "engine-shadow-runner must call omena-checker's GRN evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_smt_rules"),
  "engine-shadow-runner must call omena-checker's SMT evaluator",
);
assert.ok(
  runnerSource.includes('"omena-checker-m-tier-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-m-tier-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-cascade-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-cascade-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-grn-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-grn-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-smt-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-smt-evaluations",
);
assert.ok(
  /^\s*omena-checker\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-checker for M-tier evaluations",
);
assert.ok(
  /^\s*omena-abstract-value\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-abstract-value for M-tier value input construction",
);
assert.ok(
  packageJsonSource.includes("check:rust-omena-checker-runner-boundary"),
  "package.json must expose the checker runner boundary gate",
);
assert.ok(
  packageJsonSource.includes("rust/omena-checker/runner-boundary"),
  "rust/omena-checker/boundary must include the checker runner boundary gate",
);

const cascadeSummary = runCascadeEvaluationFixture();
assert.equal(cascadeSummary.product, "omena-checker.cascade-evaluations");
assert.equal(cascadeSummary.declarationCount, 5);
assert.equal(cascadeSummary.customPropertyCount, 3);
for (const code of [
  "unreachable-declaration",
  "dead-cascade-layer",
  "iacvt-prone",
  "circular-var",
  "unspecified-cascade-tie",
]) {
  assert.ok(
    cascadeSummary.ruleCodeNames.includes(code),
    `cascade runner output must include ${code}`,
  );
}
const grnSummary = runGrnEvaluationFixture();
assert.equal(grnSummary.product, "omena-checker.grn-evaluations");
assert.equal(grnSummary.vertexCount, 3);
for (const code of ["cascade.deep-conflict", "cascade.unreachable-rule"]) {
  assert.ok(
    grnSummary.ruleCodeNames.includes(code),
    `GRN runner output must include ${code}`,
  );
}
const smtSummary = runSmtEvaluationFixture();
assert.equal(smtSummary.product, "omena-checker.smt-evaluations");
assert.equal(smtSummary.obligationCount, 1);
assert.ok(
  smtSummary.ruleCodeNames.includes("cascade.smt-violation"),
  "SMT runner output must include cascade.smt-violation",
);

process.stdout.write(
  [
    "validated omena-checker runner boundary:",
    "mTierCommand=omena-checker-m-tier-evaluations",
    "cascadeCommand=omena-checker-cascade-evaluations",
    "grnCommand=omena-checker-grn-evaluations",
    "smtCommand=omena-checker-smt-evaluations",
    "runtime=engine-shadow-runner",
    "owner=omena-checker",
  ].join(" "),
);
process.stdout.write("\n");

function extractCommandBodies(source: string): Map<string, string> {
  const commandMatches = [...source.matchAll(/Some\("([^"]+)"\)\s*=>\s*\{/g)];
  const bodies = new Map<string, string>();

  for (const match of commandMatches) {
    const command = match[1];
    const bodyStart = match.index === undefined ? -1 : match.index + match[0].length;
    if (!command || bodyStart < 0) continue;
    bodies.set(command, readBraceBody(source, bodyStart));
  }

  return bodies;
}

function readBraceBody(source: string, bodyStart: number): string {
  let depth = 1;
  let index = bodyStart;
  while (index < source.length && depth > 0) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") depth -= 1;
    index += 1;
  }
  return source.slice(bodyStart, index - 1);
}

interface CascadeEvaluationSummary {
  readonly product: string;
  readonly declarationCount: number;
  readonly customPropertyCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface GrnEvaluationSummary {
  readonly product: string;
  readonly vertexCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface SmtEvaluationSummary {
  readonly product: string;
  readonly obligationCount: number;
  readonly ruleCodeNames: readonly string[];
}

function runCascadeEvaluationFixture(): CascadeEvaluationSummary {
  const input = {
    declarations: [
      cascadeDeclaration("base-color", ".btn", "color", "red", 1, "base", 0, false, []),
      cascadeDeclaration("override-color", ".btn", "color", "blue", 2, "overrides", 1, false, []),
      cascadeDeclaration("gap-use", ".card", "margin", "var(--gap)", 3, "components", 1, false, [
        "--gap",
      ]),
      cascadeDeclaration("tie-a", ".tie", "color", "red", 4, "utilities", 2, false, []),
      cascadeDeclaration("tie-b", ".tie", "color", "green", 5, "utilities", 2, false, []),
    ],
    customProperties: [
      { name: "--gap", dependencies: [], guaranteedInvalid: true },
      { name: "--a", dependencies: ["--b"], guaranteedInvalid: false },
      { name: "--b", dependencies: ["--a"], guaranteedInvalid: false },
    ],
  };
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
      "omena-checker-cascade-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner cascade command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as CascadeEvaluationSummary;
}

function runGrnEvaluationFixture(): GrnEvaluationSummary {
  const input = {
    vertices: [
      grnVertex("winner", ".btn", "color", "applied"),
      grnVertex("losing-eligible", ".btn", "color", "losingButEligible"),
      grnVertex("inactive-rule", ".card", "display", "inactive"),
    ],
  };
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
      "omena-checker-grn-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner GRN command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as GrnEvaluationSummary;
}

function runSmtEvaluationFixture(): SmtEvaluationSummary {
  const input = {
    obligations: [
      {
        obligationId: "bad-layer-flatten",
        l1Primitive: "layerFlattenCandidate",
        canonicalTerms: [
          "require:closed-bundle=true",
          "require:no-unlayered-rule=false",
        ],
      },
    ],
  };
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
      "omena-checker-smt-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner SMT command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as SmtEvaluationSummary;
}

function cascadeDeclaration(
  declarationId: string,
  selector: string,
  property: string,
  value: string,
  sourceOrder: number,
  layerName: string,
  layerOrder: number,
  important: boolean,
  varReferences: readonly string[],
) {
  return {
    declarationId,
    selector,
    property,
    value,
    sourceOrder,
    layerName,
    layerOrder,
    important,
    varReferences,
  };
}

function grnVertex(vertexId: string, selector: string, property: string, state: string) {
  return {
    vertexId,
    selector,
    property,
    state,
  };
}
