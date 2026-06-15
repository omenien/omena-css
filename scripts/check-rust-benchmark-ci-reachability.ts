import { strict as assert } from "node:assert";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

import {
  buildCheckPlan,
  loadCheckManifest,
  resolveGateTarget,
} from "../packages/check-orchestrator/src";

const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;

const REQUIRED_BENCHMARK_GATES = [
  "rust/benchmark/emitted-css-golden-gate",
  "rust/benchmark/headline-axis",
  "rust/benchmark/instruction-count-advisory",
  "rust/z5-parser-product-cutover",
  "rust/z5-performance-baseline-micro",
  "rust/z5-performance-baseline-macro",
  "rust/z5-performance-baseline-readiness",
] as const;

const root = process.cwd();
const manifest = loadCheckManifest(root);
const reachable = collectWorkflowReachableGateIds();
const missing = REQUIRED_BENCHMARK_GATES.filter((id) => !reachable.has(id));

assert.deepEqual(
  missing,
  [],
  `benchmark check id(s) are not reachable from any workflow: ${missing.join(", ")}`,
);

const ci = read(".github/workflows/ci.yml");
assert.ok(
  ci.includes("pnpm omena-check run rust/benchmark/emitted-css-golden-gate"),
  "CI must hard-run the emitted CSS golden gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/benchmark/headline-axis"),
  "CI must hard-run the headline-axis fidelity gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-parser-product-cutover"),
  "CI must hard-run the parser-product cutover gate",
);

const drift = read(".github/workflows/omena-css-drift.yml");
assert.ok(
  !drift.includes("continue-on-error: true"),
  "Omena CSS drift workflow must not mask benchmark readiness failures",
);

console.log(
  JSON.stringify({
    schemaVersion: "0",
    product: "rust.benchmark-ci-reachability",
    requiredBenchmarkGateCount: REQUIRED_BENCHMARK_GATES.length,
    reachableBenchmarkGateCount: REQUIRED_BENCHMARK_GATES.length - missing.length,
    requiredBenchmarkGates: REQUIRED_BENCHMARK_GATES,
  }),
);

function collectWorkflowReachableGateIds(): Set<string> {
  const workflowsDir = path.join(root, ".github", "workflows");
  const ids = new Set<string>();
  if (!existsSync(workflowsDir)) return ids;

  for (const fileName of readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;
    const workflowText = read(path.join(".github", "workflows", fileName));
    for (const match of workflowText.matchAll(OMENA_CHECK_TARGET_REF)) {
      const target = match[2];
      if (!target) continue;
      const gate = resolveGateTarget(manifest, target);
      if (!gate) {
        throw new Error(`${fileName} references unknown omena-check target: ${target}`);
      }
      for (const step of buildCheckPlan(manifest, gate).steps) {
        ids.add(step.id);
      }
    }
  }
  return ids;
}

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}
