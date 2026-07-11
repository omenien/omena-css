import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { listCheckerRuleCodes } from "../server/engine-core-ts/src/core/checker/checker-rule-registry";

interface RustLintEnvelope {
  readonly payload: {
    readonly tiers: readonly {
      readonly findings: readonly {
        readonly filePath: string;
        readonly ruleId: string;
      }[];
    }[];
    readonly ruleParity: {
      readonly sharedRuleIds: readonly string[];
      readonly rustOnlyRuleIds: readonly string[];
      readonly typescriptOnlyRuleIds: readonly string[];
    };
  };
}

interface TypeScriptCheckerReport {
  readonly findings: readonly {
    readonly filePath: string;
    readonly code: string;
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixtureRoot = path.join(repoRoot, "test/_fixtures/stylelint-plugin-smoke");

run("cargo", ["build", "--quiet", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli"]);
const rust = parseJson<RustLintEnvelope>(
  run(path.join(repoRoot, "rust/target/debug/omena"), [
    "lint",
    fixtureRoot,
    "--profile",
    "strict",
    "--json",
  ]),
);
const typescript = parseJson<TypeScriptCheckerReport>(
  run(process.execPath, [
    "--import",
    "tsx",
    "./scripts/check-workspace.ts",
    "--",
    fixtureRoot,
    "--format",
    "json",
    "--fail-on",
    "none",
  ]),
);

const typescriptRules = [...listCheckerRuleCodes()].toSorted();
const sharedRules = [...rust.payload.ruleParity.sharedRuleIds].toSorted();
assert.deepEqual(sharedRules, typescriptRules, "shared rule contract must match the TS registry");
assert.deepEqual(
  rust.payload.ruleParity.typescriptOnlyRuleIds,
  [],
  "every TypeScript checker rule must have a Rust counterpart",
);
assert.ok(
  rust.payload.ruleParity.rustOnlyRuleIds.length > 0,
  "Rust-only rules must remain explicit",
);

const injection = process.env.OMENA_LINT_PARITY_TEST_EXCLUDE_RULE;
const allRustFindings = rust.payload.tiers.flatMap(({ findings }) => findings);
const rustFindings = injection
  ? allRustFindings.filter(({ ruleId }) => ruleId !== injection)
  : allRustFindings;
const shared = new Set(sharedRules);
const rustProjection = findingCounts(
  rustFindings
    .filter(({ ruleId }) => shared.has(ruleId))
    .map(({ filePath, ruleId }) => ({ filePath, ruleId })),
);
const typescriptProjection = findingCounts(
  typescript.findings
    .filter(({ code }) => shared.has(code))
    .map(({ filePath, code }) => ({ filePath, ruleId: code })),
);
assert.deepEqual(
  rustProjection,
  typescriptProjection,
  "shared lint rules must agree on relative file, rule id, and finding count",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-lint-parity",
      sharedRuleCount: sharedRules.length,
      sharedFindingCount: Object.values(rustProjection).reduce((sum, count) => sum + count, 0),
      rustOnlyRuleCount: rust.payload.ruleParity.rustOnlyRuleIds.length,
      typescriptOnlyRuleCount: rust.payload.ruleParity.typescriptOnlyRuleIds.length,
    },
    null,
    2,
  )}\n`,
);

function findingCounts(
  findings: readonly { readonly filePath: string; readonly ruleId: string }[],
): Record<string, number> {
  const counts = new Map<string, number>();
  for (const finding of findings) {
    const relativePath = path.relative(fixtureRoot, finding.filePath).split(path.sep).join("/");
    const key = `${relativePath}\u0000${finding.ruleId}`;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return Object.fromEntries([...counts].toSorted(([left], [right]) => left.localeCompare(right)));
}

function run(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
}

function parseJson<T>(source: string): T {
  return JSON.parse(source) as T;
}
