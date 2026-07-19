import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface TransformPassSummaryV0 {
  readonly id: string;
}

interface ConsumerBuildSummaryV0 {
  readonly requestedPassIds: readonly string[];
  readonly effectivePassIds: readonly string[];
  readonly openWorldSnapshot?: {
    readonly reason: string;
  } | null;
  readonly execution: {
    readonly requestedPassIds: readonly string[];
    readonly orderedPassIds: readonly string[];
    readonly plannedOnlyPassIds: readonly string[];
    readonly outputCss: string;
    readonly decisions: readonly {
      readonly kind: string;
      readonly reason?: {
        readonly kind: string;
      };
    }[];
  };
}

function runRunner<T>(command: string, input: unknown): T {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      command,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 16 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);
  return JSON.parse(result.stdout) as T;
}

function sorted(values: readonly string[]): string[] {
  return [...values].sort();
}

function assertExecutionUsesEffectivePasses(summary: ConsumerBuildSummaryV0): void {
  assert.deepEqual(summary.execution.requestedPassIds, summary.effectivePassIds);
  assert.deepEqual(sorted(summary.execution.orderedPassIds), sorted(summary.effectivePassIds));
}

const excludedDefaultPasses = new Set([
  "native-css-static-eval",
  "layer-flatten",
  "tree-shake-class",
  "tree-shake-keyframes",
  "tree-shake-value",
  "tree-shake-custom-property",
]);
const passCatalog = runRunner<readonly TransformPassSummaryV0[]>(
  "consumer-transform-pass-list",
  {},
);
const expectedDefaultPassIds = passCatalog
  .map((pass) => pass.id)
  .filter((passId) => !excludedDefaultPasses.has(passId));

const singleSource = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "Baseline.module.css",
  styleSource: ".a { color: #ffffff; margin: 0px; } .a { color: #ffffff; margin: 0px; }",
  requestedPassIds: [],
});
assert.deepEqual(singleSource.requestedPassIds, []);
assert.deepEqual(singleSource.effectivePassIds, expectedDefaultPassIds);
assertExecutionUsesEffectivePasses(singleSource);
assert.equal(singleSource.execution.outputCss, "._a_0{color:#fff;margin:0}");
assert.equal(singleSource.openWorldSnapshot, undefined);

const multiSource = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-sources", {
  targetStylePath: "Entry.module.css",
  styles: [
    {
      stylePath: "Entry.module.css",
      styleSource: '@import "./Dep.module.css"; .entry { color: #ffffff; }',
    },
    {
      stylePath: "Dep.module.css",
      styleSource: ".dep { margin: 0px; }",
    },
  ],
  requestedPassIds: [],
});
assert.deepEqual(multiSource.requestedPassIds, []);
assert.deepEqual(multiSource.effectivePassIds, expectedDefaultPassIds);
assertExecutionUsesEffectivePasses(multiSource);
assert.equal(multiSource.openWorldSnapshot, undefined);

const explicitClosedWorld = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "Open.module.css",
  styleSource: '@import "./Missing.module.css"; .a { color: red; }',
  requestedPassIds: ["tree-shake-class"],
});
assert.deepEqual(explicitClosedWorld.requestedPassIds, ["tree-shake-class"]);
assert.deepEqual(explicitClosedWorld.effectivePassIds, ["tree-shake-class"]);
assertExecutionUsesEffectivePasses(explicitClosedWorld);
assert.deepEqual(explicitClosedWorld.execution.plannedOnlyPassIds, ["tree-shake-class"]);
assert.equal(
  explicitClosedWorld.openWorldSnapshot?.reason,
  "closed-world bundle unavailable for requested passes: tree-shake-class",
);
assert.ok(
  explicitClosedWorld.execution.decisions.some(
    (decision) => decision.kind === "blocked" && decision.reason?.kind === "precisionBelowFloor",
  ),
);

console.log(
  JSON.stringify(
    {
      product: "omena-query-effective-pass-set-gate",
      catalogPassCount: passCatalog.length,
      defaultPassCount: expectedDefaultPassIds.length,
      excludedDefaultPassIds: sorted([...excludedDefaultPasses]),
      singleSourceOutputCss: singleSource.execution.outputCss,
      multiSourcePassCount: multiSource.effectivePassIds.length,
      explicitClosedWorldDecision: "blocked",
    },
    null,
    2,
  ),
);
