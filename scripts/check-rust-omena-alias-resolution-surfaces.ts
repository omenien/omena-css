import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixedEntry =
  "summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs";
const mappingLessEntry = "summarize_omena_query_source_diagnostics_for_workspace_file";
const surfaces = [
  "rust/crates/omena-cli/src/diagnostics.rs",
  "rust/crates/omena-napi/src/lib.rs",
  "rust/crates/omena-wasm/src/lib.rs",
  "rust/crates/omena-query/src/sdk_workspace.rs",
  "rust/crates/engine-shadow-runner/src/main.rs",
] as const;

const surfaceEvidence = surfaces.map((relativePath) => {
  const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  const fixedCallCount = countCalls(source, fixedEntry);
  const mappingLessCallCount = countCalls(source, mappingLessEntry);
  assert.ok(fixedCallCount > 0, `${relativePath} does not consume resolution inputs`);
  assert.equal(
    mappingLessCallCount,
    0,
    `${relativePath} still calls the mapping-less diagnostics entry`,
  );
  return { path: relativePath, fixedCallCount, mappingLessCallCount };
});

const wasmSources = [
  "rust/crates/omena-wasm/src/lib.rs",
  "rust/crates/omena-wasm/src/sdk_workspace.rs",
] as const;
for (const relativePath of wasmSources) {
  const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  for (const forbidden of ["std::fs", "fs::read", "fs::canonicalize", "read_to_string("]) {
    assert.equal(
      source.includes(forbidden),
      false,
      `${relativePath} performs browser-side filesystem discovery through ${forbidden}`,
    );
  }
}

const focusedTests = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "-p",
    "omena-cli",
    "-p",
    "omena-napi",
    "-p",
    "omena-wasm",
    "workspace_alias_resolution_surface",
    "--",
    "--nocapture",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
assert.equal(
  focusedTests.status,
  0,
  `alias-resolution surface tests failed:\n${focusedTests.stdout}\n${focusedTests.stderr}`,
);
const executedTestCount = [
  ...focusedTests.stdout.matchAll(/test result: ok\. (\d+) passed/gu),
].reduce((count, match) => count + Number(match[1]), 0);
assert.equal(executedTestCount, 4, "all four focused surface test fixtures must execute");

const runnerInput = {
  sourcePath: "/workspace/src/App.tsx",
  sourceSource:
    'import styles from "@styles/Card.module.css";\nexport const app = <div className={styles.card} />;',
  styles: [
    {
      stylePath: "/workspace/src/styles/Card.module.css",
      styleSource: ".card {}",
    },
  ],
  packageManifests: [],
  resolutionInputs: {
    tsconfigPathMappings: [
      {
        basePath: "/workspace",
        pattern: "@styles/*",
        targetPatterns: ["src/styles/*"],
      },
    ],
  },
};
const runner = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "source-diagnostics-for-file",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    input: JSON.stringify(runnerInput),
  },
);
assert.equal(runner.status, 0, `shadow runner failed:\n${runner.stdout}\n${runner.stderr}`);
const runnerSummary = JSON.parse(runner.stdout) as {
  diagnostics: readonly { code: string }[];
};
assert.equal(
  runnerSummary.diagnostics.some((diagnostic) => diagnostic.code === "missingModule"),
  false,
  "shadow runner dropped explicit alias-resolution inputs",
);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena.alias-resolution-surfaces",
      fixedSurfaceCount: surfaceEvidence.length,
      mappingLessSurfaceCallCount: surfaceEvidence.reduce(
        (count, surface) => count + surface.mappingLessCallCount,
        0,
      ),
      focusedTestCount: executedTestCount,
      shadowRunnerMissingModuleCount: runnerSummary.diagnostics.filter(
        (diagnostic) => diagnostic.code === "missingModule",
      ).length,
      wasmFilesystemDiscoveryCount: 0,
      surfaces: surfaceEvidence,
    },
    null,
    2,
  )}\n`,
);

function countCalls(source: string, symbol: string): number {
  return [...source.matchAll(new RegExp(`\\b${symbol}\\s*\\(`, "gu"))].length;
}
