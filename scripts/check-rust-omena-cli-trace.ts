import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface TraceDomainV0 {
  readonly domain: string;
  readonly product: string;
  readonly attached: boolean;
}

interface OmenaCliTraceV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly traceVersion: string;
  readonly requestedPassIds: readonly string[];
  readonly unknownPassIds: readonly string[];
  readonly domainCount: number;
  readonly domains: readonly TraceDomainV0[];
  readonly transformExecution: { readonly product?: string };
  readonly lawvereTrace: { readonly product?: string };
  readonly lawvereParallelPlan: { readonly product?: string };
  readonly variationalTrace: { readonly product?: string };
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
    "trace",
    "--",
    "--json",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 32,
  },
);

if (result.error) {
  throw result.error;
}
assert.equal(
  result.status,
  0,
  `omena trace CLI failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const trace = JSON.parse(result.stdout) as OmenaCliTraceV0;
assert.equal(trace.schemaVersion, "0");
assert.equal(trace.product, "omena-cli.trace-v0");
assert.equal(trace.traceVersion, "TraceV0");
assert.deepEqual(trace.requestedPassIds, ["color-compression", "number-compression", "print-css"]);
assert.deepEqual(trace.unknownPassIds, []);
assert.equal(trace.domainCount, 4);
assert.equal(trace.transformExecution.product, "omena-query.transform-execute");
assert.equal(trace.lawvereTrace.product, "omena-lawvere.model-trace");
assert.equal(trace.lawvereParallelPlan.product, "omena-lawvere.transform-pass-parallel-plan");
assert.equal(
  trace.variationalTrace.product,
  "omena-variational.designer-intent-belief-propagation",
);
assert.ok(trace.readySurfaces.includes("unifiedTraceV0"));
assert.ok(trace.readySurfaces.includes("lawvereModelTrace"));
assert.ok(trace.readySurfaces.includes("variationalBeliefPropagationTrace"));
assert.deepEqual(
  trace.domains.map((domain) => [domain.domain, domain.product, domain.attached]),
  [
    ["transformExecution", "omena-query.transform-execute", true],
    ["lawvereModelTrace", "omena-lawvere.model-trace", true],
    ["lawvereParallelPlanTrace", "omena-lawvere.transform-pass-parallel-plan", true],
    [
      "variationalBeliefPropagationTrace",
      "omena-variational.designer-intent-belief-propagation",
      true,
    ],
  ],
);

console.log(
  [
    "validated omena-cli trace:",
    `product=${trace.product}`,
    `domains=${trace.domains.map((domain) => domain.domain).join(",")}`,
  ].join(" "),
);
