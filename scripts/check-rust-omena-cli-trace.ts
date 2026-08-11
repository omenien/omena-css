import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface TraceDomainV0 {
  readonly domain: string;
  readonly product: string;
  readonly attached: boolean;
}

interface TraceAttachmentV0 {
  readonly product?: string;
}

interface OmenaCliTraceV0 {
  readonly [wireKey: string]: unknown;
  readonly schemaVersion: string;
  readonly product: string;
  readonly traceVersion: string;
  readonly requestedPassIds: readonly string[];
  readonly unknownPassIds: readonly string[];
  readonly domainCount: number;
  readonly domains: readonly TraceDomainV0[];
  readonly transformExecution: TraceAttachmentV0;
  readonly variationalTrace: TraceAttachmentV0;
  readonly readySurfaces: readonly string[];
}

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0 = "lawvereTrace";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0 = "lawvereParallelPlan";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0 = "lawvereModelTrace";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0 = "lawvereParallelPlanTrace";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_MODEL_PRODUCT_V0 = "omena-lawvere.model-trace";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_PRODUCT_V0 =
  "omena-lawvere.transform-pass-parallel-plan";

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_TRANSFORM_CATALOG_WIRE_PRODUCT_BYTES_V0 =
  '["lawvereTrace","omena-lawvere.model-trace","lawvereParallelPlan","omena-lawvere.transform-pass-parallel-plan"]';

/**
 * @deprecated Compatibility owner: omena-cli maintainers. Removal condition:
 * not before 1.0, and only after downstream migration and zero in-repo
 * non-compat uses.
 */
const LEGACY_VARIATIONAL_TRACE_SURFACE_V0 = "variationalBeliefPropagationTrace";

/**
 * @deprecated Compatibility owner: omena-variational maintainers. Removal
 * condition: not before 1.0, and only after downstream migration and zero
 * in-repo non-compat uses.
 */
const LEGACY_VARIATIONAL_TRACE_PRODUCT_V0 = "omena-variational.designer-intent-belief-propagation";

const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "--features",
    "variational-trace",
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
const transformCatalogTrace = traceAttachmentAtLegacyKey(
  trace,
  LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0,
);
const transformCatalogParallelPlan = traceAttachmentAtLegacyKey(
  trace,
  LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0,
);
assert.equal(trace.schemaVersion, "0");
assert.equal(trace.product, "omena-cli.trace-v0");
assert.equal(trace.traceVersion, "TraceV0");
assert.deepEqual(trace.requestedPassIds, ["color-compression", "number-compression", "print-css"]);
assert.deepEqual(trace.unknownPassIds, []);
assert.equal(trace.domainCount, 4);
assert.equal(trace.transformExecution.product, "omena-query.transform-execute");
assert.equal(transformCatalogTrace.product, LEGACY_TRANSFORM_CATALOG_MODEL_PRODUCT_V0);
assert.equal(
  transformCatalogParallelPlan.product,
  LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_PRODUCT_V0,
);
assert.equal(
  JSON.stringify([
    LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0,
    transformCatalogTrace.product,
    LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0,
    transformCatalogParallelPlan.product,
  ]),
  LEGACY_TRANSFORM_CATALOG_WIRE_PRODUCT_BYTES_V0,
);
assert.equal(trace.variationalTrace.product, LEGACY_VARIATIONAL_TRACE_PRODUCT_V0);
assert.ok(trace.readySurfaces.includes("unifiedTraceV0"));
assert.ok(trace.readySurfaces.includes(LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0));
assert.ok(trace.readySurfaces.includes(LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0));
assert.ok(trace.readySurfaces.includes(LEGACY_VARIATIONAL_TRACE_SURFACE_V0));
assert.deepEqual(
  trace.domains.map((domain) => [domain.domain, domain.product, domain.attached]),
  [
    ["transformExecution", "omena-query.transform-execute", true],
    [LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0, LEGACY_TRANSFORM_CATALOG_MODEL_PRODUCT_V0, true],
    [
      LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0,
      LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_PRODUCT_V0,
      true,
    ],
    [LEGACY_VARIATIONAL_TRACE_SURFACE_V0, LEGACY_VARIATIONAL_TRACE_PRODUCT_V0, true],
  ],
);

console.log(
  [
    "validated omena-cli trace:",
    `product=${trace.product}`,
    `domains=${trace.domains.map((domain) => domain.domain).join(",")}`,
  ].join(" "),
);

function traceAttachmentAtLegacyKey(trace: OmenaCliTraceV0, wireKey: string): TraceAttachmentV0 {
  const value = trace[wireKey];
  assert.ok(value !== null && typeof value === "object", `missing trace attachment ${wireKey}`);
  return value as TraceAttachmentV0;
}
