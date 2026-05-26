import { readFileSync } from "node:fs";
import { strict as assert } from "node:assert";

const QUERY_MANIFEST_PATH = "rust/crates/omena-query/Cargo.toml";
const RUNNER_MANIFEST_PATH = "rust/crates/omena-query-transform-runner/Cargo.toml";
const WORKSPACE_MANIFEST_PATH = "rust/Cargo.toml";
const QUERY_SRC_PATH = "rust/crates/omena-query/src";
const TRANSFORM_CRATES = [
  "omena-transform-bundle",
  "omena-transform-cst",
  "omena-transform-egg",
  "omena-transform-passes",
  "omena-transform-print",
  "omena-transform-target",
] as const;

const queryManifest = readFileSync(QUERY_MANIFEST_PATH, "utf8");
const runnerManifest = readFileSync(RUNNER_MANIFEST_PATH, "utf8");
const workspaceManifest = readFileSync(WORKSPACE_MANIFEST_PATH, "utf8");

const queryDeps = dependencyNames(queryManifest);
const runnerDeps = dependencyNames(runnerManifest);
const queryInternalDeps = queryDeps.filter(
  (dependency) => dependency.startsWith("omena-") || dependency.startsWith("engine-"),
);
const queryDirectTransformDeps = TRANSFORM_CRATES.filter((dependency) =>
  queryDeps.includes(dependency),
);
const runnerTransformDeps = TRANSFORM_CRATES.filter((dependency) => runnerDeps.includes(dependency));

assert.ok(
  workspaceManifest.includes('"crates/omena-query-transform-runner"'),
  "workspace must include the omena-query-transform-runner split crate",
);
assert.ok(
  queryDeps.includes("omena-query-transform-runner"),
  "omena-query must depend on the transform-runner boundary",
);
assert.deepEqual(
  queryDirectTransformDeps,
  [],
  "omena-query must not depend directly on transform-family crates after H2 split",
);
assert.deepEqual(
  runnerTransformDeps,
  [...TRANSFORM_CRATES],
  "omena-query-transform-runner must own the collapsed transform-family dependencies",
);
assert.ok(
  !runnerDeps.includes("omena-query"),
  "transform-runner boundary must not depend back on omena-query",
);
assert.ok(
  queryInternalDeps.length <= 12,
  `omena-query direct internal dependency count must be <= 12 after transform-runner split, got ${queryInternalDeps.length}`,
);

const querySource = readFileSync(`${QUERY_SRC_PATH}/lib.rs`, "utf8");
assert.ok(
  !/\b(?:use|pub use)\s+omena_transform_/u.test(querySource),
  "omena-query lib.rs must not import transform-family crates directly",
);
assert.ok(
  querySource.includes("omena_query_transform_runner"),
  "omena-query lib.rs must route transform-family imports through omena-query-transform-runner",
);

process.stdout.write(
  [
    "validated m8 query fan-in:",
    `queryInternalDeps=${queryInternalDeps.length}`,
    `queryDirectTransformDeps=${queryDirectTransformDeps.length}`,
    `runnerCollapsedTransformDeps=${runnerTransformDeps.length}`,
  ].join(" "),
);
process.stdout.write("\n");

function dependencyNames(manifest: string): string[] {
  const dependenciesSection = manifest.match(/\[dependencies\]\n(?<body>[\s\S]*?)(?:\n\[|$)/u)
    ?.groups?.body;
  assert.ok(dependenciesSection, "manifest must contain [dependencies]");
  return [...dependenciesSection.matchAll(/^([A-Za-z0-9_-]+)\s*=/gmu)]
    .map((match) => match[1])
    .filter((dependency): dependency is string => dependency !== undefined)
    .toSorted();
}
