import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";

interface TransformExecuteSummaryV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stylePath: string;
  readonly requestedPassIds: readonly string[];
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly product: string;
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly plannedOnlyPassIds: readonly string[];
    readonly mutationCount: number;
    readonly provenancePreserved: boolean;
    readonly passPlan: {
      readonly product: string;
      readonly violatedDagEdgeCount: number;
      readonly allRequestedRegistered: boolean;
    };
  };
  readonly readySurfaces: readonly string[];
}

const styleSource =
  '.dupe { display: block; } .dupe { display: block; } .empty { } .a:is(.ready) { margin-top: 0px; margin-right: 0px; margin-bottom: 0px; margin-left: 0px; color: #FFFFFF; opacity: 1.0; background: url("img.svg"); font-family: \'Demo\'; /* remove */ content: "/* keep */"; }';

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
    "transform-execute",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.css",
      styleSource,
      requestedPassIds: [
        "p01-whitespace-strip",
        "p02-comment-strip",
        "p03-number-compression",
        "p04-unit-normalization",
        "p05-color-compression",
        "p06-url-quote-strip",
        "p07-string-quote-normalize",
        "p08-selector-is-where-compression",
        "p09-shorthand-combining",
        "p10-rule-deduplication",
        "p13-empty-rule-removal",
        "p40-print-css",
        "p99-unknown",
      ],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(result.status, 0, result.stderr);
assert.equal(result.error, undefined);

const summary = JSON.parse(result.stdout) as TransformExecuteSummaryV0;

assert.equal(summary.schemaVersion, "0");
assert.equal(summary.product, "omena-query.transform-execute");
assert.equal(summary.stylePath, "Button.module.css");
assert.deepEqual(summary.requestedPassIds, [
  "p01-whitespace-strip",
  "p02-comment-strip",
  "p03-number-compression",
  "p04-unit-normalization",
  "p05-color-compression",
  "p06-url-quote-strip",
  "p07-string-quote-normalize",
  "p08-selector-is-where-compression",
  "p09-shorthand-combining",
  "p10-rule-deduplication",
  "p13-empty-rule-removal",
  "p40-print-css",
  "p99-unknown",
]);
assert.deepEqual(summary.unknownPassIds, ["p99-unknown"]);
assert.equal(summary.execution.product, "omena-transform-passes.execution");
assert.equal(
  summary.execution.outputCss,
  '.dupe{display: block;}.a.ready{margin: 0;color: #fff;opacity: 1;background: url(img.svg);font-family: "Demo";content: "/* keep */";}',
);
assert.deepEqual(summary.execution.executedPassIds, [
  "p08-selector-is-where-compression",
  "p09-shorthand-combining",
  "p10-rule-deduplication",
  "p13-empty-rule-removal",
  "p01-whitespace-strip",
  "p02-comment-strip",
  "p03-number-compression",
  "p04-unit-normalization",
  "p05-color-compression",
  "p06-url-quote-strip",
  "p07-string-quote-normalize",
  "p40-print-css",
]);
assert.deepEqual(summary.execution.plannedOnlyPassIds, []);
assert.equal(summary.execution.mutationCount, 23);
assert.equal(summary.execution.provenancePreserved, true);
assert.equal(summary.execution.passPlan.product, "omena-transform-passes.plan");
assert.equal(summary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(summary.execution.passPlan.allRequestedRegistered, true);
assertIncludesAll(
  summary.readySurfaces,
  ["transformExecutionRuntime", "transformPassOutcomeContract"],
  "transform execute ready surfaces",
);

process.stdout.write(
  [
    "validated omena-query transform-execute runtime:",
    `executed=${summary.execution.executedPassIds.length}`,
    `mutations=${summary.execution.mutationCount}`,
    `unknown=${summary.unknownPassIds.length}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
