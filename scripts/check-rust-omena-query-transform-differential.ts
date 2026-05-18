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
  "media-static-eval",
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
    label: "integer-leading-zero-number",
    source: ".a { z-index: 001; opacity: 000.50; }",
  },
  {
    label: "selector-list-and-spacing",
    source: '.a , .b { color : #FFFFFF ; opacity: 1.0; background: url("x.svg"); }',
  },
  {
    label: "adjacent-duplicate-color-declarations",
    source: ".a { color: rgb(255 0 0); color: rgb(255 0 0 / 100%); background: blue; }",
  },
  {
    label: "is-where-and-shorthand",
    source:
      ".a:is(.ready) { color: #FFFFFF; margin-top: 0px; margin-right: 0px; margin-bottom: 0px; margin-left: 0px; }",
  },
  {
    label: "box-shorthand-value-compression",
    source: ".a { margin: 1px 1px 1px 1px; padding: 1px 2px 3px 2px; }",
  },
  {
    label: "flex-shorthand-compression",
    source: ".a { flex: 0 1 auto; } .b { flex: 1 1 0%; } .c { flex: 1 2 0%; }",
  },
  {
    label: "border-radius-shorthand-compression",
    source: ".a { border-radius: 1px 1px 1px 1px; }",
  },
  {
    label: "border-radius-longhand-compression",
    source:
      ".a { border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; }",
  },
  {
    label: "inset-shorthand-compression",
    source: ".a { inset: 1px 2px 1px 2px; }",
  },
  {
    label: "inset-longhand-compression",
    source: ".a { top: 1px; right: 2px; bottom: 1px; left: 2px; }",
  },
  {
    label: "list-style-shorthand-compression",
    source: ".a { list-style: disc outside none; }",
  },
  {
    label: "list-style-longhand-compression",
    source:
      ".a { list-style-type: none; list-style-position: outside; list-style-image: none; }",
  },
  {
    label: "structural-rule-merge",
    source:
      ".dupe { display: block; } .dupe { display: block; } .sel-a { border: 0; } .sel-b { border: 0; } .merge { color: red; } .merge { background: #0000FF; }",
  },
  {
    label: "rule-merge-semicolonless",
    source: ".b{color:red}.b{background:blue}",
  },
  {
    label: "comment-empty-calc",
    source: "/* head */ .calc { width: calc(1px + 2px); } .empty { } /* tail */",
  },
  {
    label: "nested-comment-empty-rules",
    source:
      ".empty { } @supports (display: grid) { .nested { } .filled { color: red; } } .outer { .inner { } } .with-comment { /* remove after comment strip */ } .filled { color: red; }",
  },
  {
    label: "keyframes-empty-frame",
    source: "@keyframes fade { 0% {} to { opacity: 1 } } .empty{}",
  },
  {
    label: "keyframes-selector-aliases",
    source: "@keyframes fade { from { opacity: 0 } 100% { opacity: 1 } 50%, TO { opacity: .5 } }",
  },
  {
    label: "media-range-normalization",
    source: "@media screen and (min-width: 1px) and (max-width: 10px) { .a { color: red; } }",
  },
  {
    label: "media-range-calc-reduction",
    source:
      "@media (min-width: calc(1px + 1px)) and (max-height: clamp(1rem, 2rem, 3rem)) { .a { color: red; } }",
  },
  {
    label: "supports-group-color-compression",
    source:
      "@supports not (display: grid) { .a { color: red; } } @supports (display: grid) or (unknown: value) { .b { color: blue; } }",
  },
  {
    label: "calc-same-unit-nested",
    source: ".a { margin: calc(2rem + 3rem); padding: calc(10px - 4px); }",
  },
  {
    label: "calc-additive-chain",
    source:
      ".a { width: calc(2px + 3px + 4px); height: calc(.5rem + .25rem + .25rem); margin: calc(10px - 3px - 2px); }",
  },
  {
    label: "calc-parenthesized-multiplicative-chain",
    source:
      ".a { width: calc((1px + 2px)); height: calc(2px * 3 * 4); margin: calc(24px / 2 / 3); }",
  },
  {
    label: "nested-min-max-functions",
    source: ".a { width: min(10px, max(2px, 4px)); height: max(1px, min(4px, 2px)); }",
  },
  {
    label: "clamp-static-value",
    source: ".a { opacity: clamp(.1, .5, .9); }",
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
    label: "border-composite-named-color",
    source: ".a { border: 1px solid black; }",
  },
  {
    label: "border-outline-zero-shorthand-lengths",
    source:
      ".a { border: 0px solid #000000; } .b { border-top: 0px solid #000000; } .c { outline: 0px solid #000000; } .d { text-decoration: underline 0px #000000; }",
  },
  {
    label: "url-zero-font-family",
    source: '.a { background: url("/icons/a.svg"); margin: 0 0 0 0%; font-family: "Inter"; }',
  },
  {
    label: "position-zero-percent-normalization",
    source:
      ".a { perspective-origin: 0% 0%; transform-origin: 0% 0%; opacity: 0%; background-position: 0% 0%; background-size: auto auto; mask-position: 0% 0%; }",
  },
  {
    label: "center-position-normalization",
    source:
      ".bg { background-position: center center; } .left { background-position: left center; } .origin { transform-origin: center top; } .mask { mask-position: bottom right; }",
  },
  {
    label: "opacity-percentage-normalization",
    source: ".a { opacity: 50%; } .b { opacity: 100%; }",
  },
  {
    label: "aspect-ratio-spacing-normalization",
    source: ".a { aspect-ratio: 16 / 9; } .b { aspect-ratio: auto 4 / 3; }",
  },
  {
    label: "shadow-zero-length-normalization",
    source:
      ".a { box-shadow: 0px 0px 0px #000; } .b { box-shadow: inset 1px 2px 0px 0px #000; } .c { text-shadow: 1px 2px 0px #000; }",
  },
  {
    label: "time-unit-shortening",
    source: ".a { transition-duration: 100ms; transition-delay: .05s; animation-delay: 0ms; }",
  },
  {
    label: "transform-zero-unit-normalization",
    source: ".a { transform: rotate(0deg) translate(0px); }",
  },
  {
    label: "transform-scale-repeat-normalization",
    source: ".a { transform: scale(1, 1) scale(2, 2); }",
  },
  {
    label: "transform-zero-axis-normalization",
    source:
      ".a { transform: translateX(0px) translateY(0px) translateZ(0px) translate(0px, 0px) perspective(0px); }",
  },
  {
    label: "font-family-list",
    source: '.fonts { font-family: "Arial", "Helvetica Neue", "system-ui", sans-serif; }',
  },
  {
    label: "font-longhand-keywords",
    source:
      ".fonts { font-weight: normal; font-stretch: normal; } .bold { font-weight: bold; font-stretch: condensed; }",
  },
  {
    label: "overflow-background-repeat-shorthand",
    source:
      ".a { background-repeat: repeat repeat; overflow-x: visible; overflow-y: visible; }",
  },
  {
    label: "alpha-hex-zero-line-height-calc",
    source:
      ".alpha { color: #ffffffff; border-color: #00000000; width: calc(2px * 3); height: calc(6px / 2); line-height: 0em; }",
  },
  {
    label: "opaque-rgba-hsla",
    source:
      ".opaque { color: rgba(255, 0, 0, 1); text-decoration-color: hsla(240, 100%, 50%, 100%); }",
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
