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
    readonly cssImportInlines: readonly {
      readonly importSource: string;
      readonly replacementCss: string;
    }[];
    readonly cssModuleComposesExports: readonly {
      readonly localClassName: string;
      readonly exportedClassNames: readonly string[];
    }[];
    readonly designTokenRoutes: readonly {
      readonly tokenName: string;
      readonly routedValue: string;
    }[];
    readonly provenanceDerivationForest: {
      readonly product: string;
      readonly rootCount: number;
      readonly nodeCount: number;
      readonly nodes: readonly {
        readonly nodeIndex: number;
        readonly parentIndex?: number;
        readonly passId: string;
        readonly mutationCount: number;
      }[];
    };
    readonly passPlan: {
      readonly product: string;
      readonly violatedDagEdgeCount: number;
      readonly allRequestedRegistered: boolean;
    };
  };
  readonly readySurfaces: readonly string[];
}

const styleSource =
  '.dupe { display: block; } .dupe { display: block; } .merge { color: red; } .merge { background: blue; } .sel-a { border: 0; } .sel-b { border: 0; } .empty { } .a:is(.ready) { margin-top: 0px; margin-right: 0px; margin-bottom: 0px; margin-left: 0px; color: #FFFFFF; user-select: none; opacity: 1.0; background: url("img.svg"); font-family: \'Demo\'; /* remove */ content: "/* keep */"; }';

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
        "p11-rule-merging",
        "p12-selector-merging",
        "p13-empty-rule-removal",
        "p14-vendor-prefixing",
        "p15-light-dark-lowering",
        "p16-color-mix-lowering",
        "p17-oklch-oklab-lowering",
        "p18-color-function-lowering",
        "p19-logical-to-physical",
        "p20-nesting-unwrap",
        "p23-supports-static-eval",
        "p24-media-static-eval",
        "p31-value-resolution",
        "p32-custom-property-static-resolve",
        "p37-dead-media-branch-removal",
        "p38-dead-supports-branch-removal",
        "p25-calc-reduction",
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
  "p11-rule-merging",
  "p12-selector-merging",
  "p13-empty-rule-removal",
  "p14-vendor-prefixing",
  "p15-light-dark-lowering",
  "p16-color-mix-lowering",
  "p17-oklch-oklab-lowering",
  "p18-color-function-lowering",
  "p19-logical-to-physical",
  "p20-nesting-unwrap",
  "p23-supports-static-eval",
  "p24-media-static-eval",
  "p31-value-resolution",
  "p32-custom-property-static-resolve",
  "p37-dead-media-branch-removal",
  "p38-dead-supports-branch-removal",
  "p25-calc-reduction",
  "p40-print-css",
  "p99-unknown",
]);
assert.deepEqual(summary.unknownPassIds, ["p99-unknown"]);
assert.equal(summary.execution.product, "omena-transform-passes.execution");
assert.equal(
  summary.execution.outputCss,
  '.dupe{display: block;}.merge{color: red;background: blue;}.sel-a,.sel-b{border: 0;}.a.ready{margin: 0;color: #fff;-webkit-user-select: none;user-select: none;opacity: 1;background: url(img.svg);font-family: "Demo";content: "/* keep */";}',
);
assert.deepEqual(summary.execution.executedPassIds, [
  "p31-value-resolution",
  "p32-custom-property-static-resolve",
  "p37-dead-media-branch-removal",
  "p38-dead-supports-branch-removal",
  "p15-light-dark-lowering",
  "p16-color-mix-lowering",
  "p17-oklch-oklab-lowering",
  "p18-color-function-lowering",
  "p19-logical-to-physical",
  "p20-nesting-unwrap",
  "p23-supports-static-eval",
  "p24-media-static-eval",
  "p14-vendor-prefixing",
  "p08-selector-is-where-compression",
  "p09-shorthand-combining",
  "p10-rule-deduplication",
  "p11-rule-merging",
  "p12-selector-merging",
  "p13-empty-rule-removal",
  "p25-calc-reduction",
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
assert.equal(summary.execution.mutationCount, 38);
assert.equal(summary.execution.provenancePreserved, true);
assert.equal(summary.execution.passPlan.product, "omena-transform-passes.plan");
assert.equal(summary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(summary.execution.passPlan.allRequestedRegistered, true);
assert.equal(
  summary.execution.provenanceDerivationForest.product,
  "omena-transform-passes.provenance-derivation-forest",
);
assert.equal(summary.execution.provenanceDerivationForest.rootCount, 1);
assert.equal(
  summary.execution.provenanceDerivationForest.nodeCount,
  summary.execution.provenanceDerivationForest.nodes.length,
);
assertIncludesAll(
  summary.readySurfaces,
  ["transformExecutionRuntime", "transformPassOutcomeContract"],
  "transform execute ready surfaces",
);

const contextStyleSource =
  '@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }';

const contextResult = spawnSync(
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
      styleSource: contextStyleSource,
      requestedPassIds: [
        "p26-import-inline",
        "p30-composes-resolution",
        "p29-css-modules-class-hashing",
        "p39-design-token-routing",
        "p40-print-css",
      ],
      transformContext: {
        importInlines: [{ importSource: "./tokens.css", replacementCss: ":root { --brand: red; }" }],
        cssModuleComposesResolutions: [
          { localClassName: "button", exportedClassNames: ["button", "base"] },
        ],
        classNameRewrites: [
          { originalName: "button", rewrittenName: "_button_x" },
          { originalName: "base", rewrittenName: "_base_y" },
        ],
        designTokenRoutes: [{ tokenName: "--brand", routedValue: "var(--theme-brand)" }],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(contextResult.status, 0, contextResult.stderr);
assert.equal(contextResult.error, undefined);

const contextSummary = JSON.parse(contextResult.stdout) as TransformExecuteSummaryV0;

assert.equal(contextSummary.schemaVersion, "0");
assert.equal(contextSummary.product, "omena-query.transform-execute");
assert.equal(contextSummary.stylePath, "Button.module.css");
assert.deepEqual(contextSummary.unknownPassIds, []);
assert.equal(
  contextSummary.execution.outputCss,
  ':root { --brand: red; } ._button_x{  color: var(--theme-brand); } ._base_y{ color: blue; }',
);
assert.deepEqual(contextSummary.execution.executedPassIds, [
  "p26-import-inline",
  "p30-composes-resolution",
  "p29-css-modules-class-hashing",
  "p39-design-token-routing",
  "p40-print-css",
]);
assert.deepEqual(contextSummary.execution.plannedOnlyPassIds, []);
assert.equal(contextSummary.execution.mutationCount, 5);
assert.deepEqual(contextSummary.execution.cssImportInlines, [
  { importSource: "./tokens.css", replacementCss: ":root { --brand: red; }" },
]);
assert.deepEqual(contextSummary.execution.cssModuleComposesExports, [
  { localClassName: "button", exportedClassNames: ["button", "base"] },
]);
assert.deepEqual(contextSummary.execution.designTokenRoutes, [
  { tokenName: "--brand", routedValue: "var(--theme-brand)" },
]);
assert.equal(contextSummary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(contextSummary.execution.passPlan.allRequestedRegistered, true);
assert.equal(
  contextSummary.execution.provenanceDerivationForest.product,
  "omena-transform-passes.provenance-derivation-forest",
);
assert.equal(contextSummary.execution.provenanceDerivationForest.rootCount, 1);
assert.equal(
  contextSummary.execution.provenanceDerivationForest.nodeCount,
  contextSummary.execution.executedPassIds.length,
);
assertIncludesAll(
  contextSummary.readySurfaces,
  ["transformExecutionRuntime", "transformPassOutcomeContract"],
  "transform context execute ready surfaces",
);

process.stdout.write(
  [
    "validated omena-query transform-execute runtime:",
    `executed=${summary.execution.executedPassIds.length}`,
    `mutations=${summary.execution.mutationCount}`,
    `unknown=${summary.unknownPassIds.length}`,
    `contextExecuted=${contextSummary.execution.executedPassIds.length}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
