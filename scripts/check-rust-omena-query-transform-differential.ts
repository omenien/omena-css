import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { transform as lightningTransform } from "lightningcss";

interface TransformExecuteSummaryV0 {
  readonly product: string;
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly product: string;
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly mutationCount: number;
    readonly provenancePreserved: boolean;
    readonly passPlan: {
      readonly violatedDagEdgeCount: number;
      readonly allRequestedRegistered: boolean;
    };
  };
}

interface DifferentialFixture {
  readonly label: string;
  readonly source: string;
}

const passIds = [
  "whitespace-strip",
  "comment-strip",
  "number-compression",
  "unit-normalization",
  "color-compression",
  "url-quote-strip",
  "string-quote-normalize",
  "selector-is-where-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "calc-reduction",
  "print-css",
] as const;

const fixtures: readonly DifferentialFixture[] = [
  {
    label: "token-value-minification",
    source:
      '.a { color: #FFFFFF; opacity: 1.0; background: url("x.svg"); width: 0.50rem; margin: 0px; }',
  },
  {
    label: "selector-list-and-spacing",
    source: '.a , .b { color : #FFFFFF ; opacity: 1.0; background: url("x.svg"); }',
  },
  {
    label: "is-where-and-shorthand",
    source:
      ".a:is(.ready) { color: #FFFFFF; margin-top: 0px; margin-right: 0px; margin-bottom: 0px; margin-left: 0px; }",
  },
  {
    label: "structural-rule-merge",
    source:
      ".dupe { display: block; } .dupe { display: block; } .sel-a { border: 0; } .sel-b { border: 0; } .merge { color: red; } .merge { background: #0000FF; }",
  },
  {
    label: "comment-empty-calc",
    source: "/* head */ .calc { width: calc(1px + 2px); } .empty { } /* tail */",
  },
  {
    label: "calc-same-unit-nested",
    source: ".a { margin: calc(2rem + 3rem); padding: calc(10px - 4px); }",
  },
  {
    label: "is-where-multi",
    source: ":is(.a) { color: #ffffff; } :where(.b) { color: #0000ff; }",
  },
  {
    label: "rule-selector-merge-with-named-color",
    source: ".a { color: red; } .b { color: red; } .a { background: blue; } .empty {}",
  },
  {
    label: "url-zero-font-family",
    source: '.a { background: url("/icons/a.svg"); margin: 0 0 0 0%; font-family: "Inter"; }',
  },
  {
    label: "font-family-list",
    source: '.fonts { font-family: "Arial", "Helvetica Neue", "system-ui", sans-serif; }',
  },
  {
    label: "alpha-hex-zero-line-height-calc",
    source: ".alpha { color: #ffffffff; border-color: #00000000; width: calc(2px * 3); height: calc(6px / 2); line-height: 0em; }",
  },
];

const reports = fixtures.map((fixture) => {
  const omena = runOmenaTransform(fixture);
  const lightning = runLightningTransform(fixture);

  assert.equal(omena.product, "omena-query.transform-execute", fixture.label);
  assert.equal(omena.execution.product, "omena-transform-passes.execution", fixture.label);
  assert.deepEqual(omena.unknownPassIds, [], fixture.label);
  assert.equal(omena.execution.passPlan.violatedDagEdgeCount, 0, fixture.label);
  assert.equal(omena.execution.passPlan.allRequestedRegistered, true, fixture.label);
  assert.equal(omena.execution.provenancePreserved, true, fixture.label);
  assert.deepEqual(
    omena.execution.outputCss,
    lightning,
    `${fixture.label} should match lightningcss minified output for the supported CSS subset`,
  );

  return {
    label: fixture.label,
    byteLength: omena.execution.outputCss.length,
    mutationCount: omena.execution.mutationCount,
    executedPassCount: omena.execution.executedPassIds.length,
  };
});

process.stdout.write(
  [
    "validated omena-query transform differential against lightningcss:",
    `fixtures=${reports.length}`,
    `bytes=${reports.reduce((sum, report) => sum + report.byteLength, 0)}`,
    `mutations=${reports.reduce((sum, report) => sum + report.mutationCount, 0)}`,
  ].join(" "),
);
process.stdout.write("\n");

function runOmenaTransform(fixture: DifferentialFixture): TransformExecuteSummaryV0 {
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
        stylePath: `${fixture.label}.css`,
        styleSource: fixture.source,
        requestedPassIds: passIds,
      }),
      maxBuffer: 8 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);

  return JSON.parse(result.stdout) as TransformExecuteSummaryV0;
}

function runLightningTransform(fixture: DifferentialFixture): string {
  const result = lightningTransform({
    filename: `${fixture.label}.css`,
    code: Buffer.from(fixture.source),
    minify: true,
  });

  return String(result.code);
}
