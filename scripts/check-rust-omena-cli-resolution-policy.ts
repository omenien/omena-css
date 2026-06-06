import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface ResolutionPolicyStep {
  readonly order: number;
  readonly key: string;
  readonly appliesTo: string;
  readonly precedence: string;
  readonly candidateSemantics: string;
}

interface ResolutionPolicyReport {
  readonly product: string;
  readonly candidateStrategy: string;
  readonly networkAccess: string;
  readonly steps: readonly ResolutionPolicyStep[];
  readonly readySurfaces: readonly string[];
}

const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "--bin",
    "omena-cli",
    "--",
    "report",
    "resolution-policy",
    "--json",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  },
);

if (result.error) {
  throw result.error;
}

assert.equal(
  result.status,
  0,
  `omena report resolution-policy failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const report = JSON.parse(result.stdout) as ResolutionPolicyReport;
assert.equal(report.product, "omena-resolver.style-resolution-policy");
assert.equal(report.candidateStrategy, "orderedFirstExistingCandidate");
assert.equal(report.networkAccess, "neverFetch");
assert.deepEqual(
  report.steps.map((step) => step.key),
  [
    "externalUrlBoundary",
    "bundlerPathMapping",
    "tsconfigPathMapping",
    "sassPkgImporter",
    "fileRelativeOrAbsolute",
    "packageManifestSubpath",
    "nodePackageFallback",
    "sassLoadPathRoot",
  ],
);
assert.deepEqual(
  report.steps.map((step) => step.order),
  [0, 10, 20, 30, 40, 50, 60, 70],
);
assert.ok(
  report.steps.every((step) => step.appliesTo && step.precedence && step.candidateSemantics),
  "every resolution-policy step must explain appliesTo, precedence, and candidate semantics",
);
for (const surface of [
  "resolutionPolicyReport",
  "bundlerAliasBeforeTsconfig",
  "webpackFirstAliasMatch",
  "tsconfigPathMapping",
  "sassPkgImporterBoundary",
  "sassLoadPathFallback",
  "networkFetchForbidden",
]) {
  assert.ok(report.readySurfaces.includes(surface), `missing ready surface: ${surface}`);
}

console.log(
  [
    "validated omena-cli resolution policy:",
    `steps=${report.steps.length}`,
    `strategy=${report.candidateStrategy}`,
    `network=${report.networkAccess}`,
  ].join(" "),
);
