import { readFileSync } from "node:fs";
import { strict as assert } from "node:assert";

const QUERY_MANIFEST_PATH = "rust/crates/omena-query/Cargo.toml";
const CORE_MANIFEST_PATH = "rust/crates/omena-query-core/Cargo.toml";
const CHECKER_ORCHESTRATOR_MANIFEST_PATH =
  "rust/crates/omena-query-checker-orchestrator/Cargo.toml";
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
const coreManifest = readFileSync(CORE_MANIFEST_PATH, "utf8");
const checkerOrchestratorManifest = readFileSync(CHECKER_ORCHESTRATOR_MANIFEST_PATH, "utf8");
const runnerManifest = readFileSync(RUNNER_MANIFEST_PATH, "utf8");
const workspaceManifest = readFileSync(WORKSPACE_MANIFEST_PATH, "utf8");

const queryDeps = dependencyNames(queryManifest);
const coreDeps = dependencyNames(coreManifest);
const checkerOrchestratorDeps = dependencyNames(checkerOrchestratorManifest);
const runnerDeps = dependencyNames(runnerManifest);
const queryInternalDeps = queryDeps.filter(
  (dependency) => dependency.startsWith("omena-") || dependency.startsWith("engine-"),
);
const queryDirectTransformDeps = TRANSFORM_CRATES.filter((dependency) =>
  queryDeps.includes(dependency),
);
const runnerTransformDeps = TRANSFORM_CRATES.filter((dependency) =>
  runnerDeps.includes(dependency),
);
const CORE_OWNED_QUERY_DEPS: readonly string[] = [
  "omena-abstract-value",
  "omena-engine-input-producers",
  "omena-incremental",
] as const;

assert.ok(
  workspaceManifest.includes('"crates/omena-query-core"'),
  "workspace must include the omena-query-core split crate",
);
assert.ok(
  workspaceManifest.includes('"crates/omena-query-checker-orchestrator"'),
  "workspace must include the omena-query-checker-orchestrator split crate",
);
assert.ok(
  workspaceManifest.includes('"crates/omena-query-transform-runner"'),
  "workspace must include the omena-query-transform-runner split crate",
);
assert.ok(
  queryDeps.includes("omena-query-core"),
  "omena-query must depend on the query-core boundary",
);
assert.ok(
  queryDeps.includes("omena-query-checker-orchestrator"),
  "omena-query must depend on the checker-orchestrator boundary",
);
assert.deepEqual(
  queryDeps.filter((dependency) => CORE_OWNED_QUERY_DEPS.includes(dependency)),
  [],
  "omena-query must route producer fragments and expression-domain runtime through omena-query-core",
);
assert.deepEqual(
  coreDeps.filter((dependency) => CORE_OWNED_QUERY_DEPS.includes(dependency)),
  [...CORE_OWNED_QUERY_DEPS],
  "omena-query-core must own producer fragment and expression-domain runtime dependencies",
);
assert.ok(
  queryDeps.includes("omena-query-transform-runner"),
  "omena-query must depend on the transform-runner boundary",
);
assert.ok(
  !queryDeps.includes("omena-checker"),
  "omena-query must not depend directly on omena-checker after checker-orchestrator split",
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
assert.deepEqual(
  checkerOrchestratorDeps.filter((dependency) => dependency === "omena-checker"),
  ["omena-checker"],
  "omena-query-checker-orchestrator must own the direct omena-checker dependency",
);
assert.ok(
  !checkerOrchestratorDeps.includes("omena-query"),
  "checker-orchestrator boundary must not depend back on omena-query",
);
assert.ok(
  queryInternalDeps.length <= 10,
  `omena-query direct internal dependency count must be <= 10 after query-core split, got ${queryInternalDeps.length}`,
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
const queryCascadeCheckerSource = readFileSync(
  `${QUERY_SRC_PATH}/style/cascade_checker.rs`,
  "utf8",
);
assert.ok(
  !/\b(?:use|pub use)\s+omena_checker\b/u.test(queryCascadeCheckerSource),
  "omena-query cascade checker must not import omena-checker directly",
);
assert.ok(
  queryCascadeCheckerSource.includes("run_omena_query_checker_cascade_gate_v0"),
  "omena-query cascade checker must route diagnostics through the checker gate",
);

process.stdout.write(
  [
    "validated m8 query fan-in:",
    `queryInternalDeps=${queryInternalDeps.length}`,
    `queryDirectCoreOwnedDeps=${
      queryDeps.filter((dependency) => CORE_OWNED_QUERY_DEPS.includes(dependency)).length
    }`,
    `queryDirectTransformDeps=${queryDirectTransformDeps.length}`,
    `queryDirectCheckerDeps=${queryDeps.includes("omena-checker") ? 1 : 0}`,
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
