import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const RUNNER_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/src/main.rs");
const RUNNER_CARGO_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/Cargo.toml");
const PACKAGE_JSON_PATH = path.join(process.cwd(), "package.json");

const runnerSource = readFileSync(RUNNER_PATH, "utf8");
const runnerCargoToml = readFileSync(RUNNER_CARGO_PATH, "utf8");
const packageJsonSource = readFileSync(PACKAGE_JSON_PATH, "utf8");
const commandBodies = extractCommandBodies(runnerSource);

const checkerMTierBody = commandBodies.get("omena-checker-m-tier-evaluations");
assert.ok(
  checkerMTierBody,
  "missing engine-shadow-runner command arm: omena-checker-m-tier-evaluations",
);
assert.ok(
  checkerMTierBody.includes("OmenaCheckerMTierEvaluationInputV0"),
  "omena-checker-m-tier-evaluations must deserialize the checker M-tier input product",
);
assert.ok(
  checkerMTierBody.includes("summarize_omena_checker_m_tier_evaluations"),
  "omena-checker-m-tier-evaluations must route through the runner-owned checker summary wrapper",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_m_tier_rules"),
  "engine-shadow-runner must call omena-checker's M-tier evaluator",
);
assert.ok(
  runnerSource.includes('"omena-checker-m-tier-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-m-tier-evaluations",
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

process.stdout.write(
  [
    "validated omena-checker runner boundary:",
    "mTierCommand=omena-checker-m-tier-evaluations",
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
