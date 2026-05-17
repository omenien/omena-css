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

interface ConsumerBuildSummaryV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stylePath: string;
  readonly requestedPassIds: readonly string[];
  readonly unknownPassIds: readonly string[];
  readonly semanticRemovalCount: number;
  readonly execution: {
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly semanticRemovals: readonly {
      readonly passId: string;
      readonly symbolKind: string;
      readonly name: string;
    }[];
    readonly passPlan: {
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
        "vendor-prefixing",
        "light-dark-lowering",
        "color-mix-lowering",
        "oklch-oklab-lowering",
        "color-function-lowering",
        "logical-to-physical",
        "nesting-unwrap",
        "supports-static-eval",
        "media-static-eval",
        "value-resolution",
        "custom-property-static-resolve",
        "dead-media-branch-removal",
        "dead-supports-branch-removal",
        "calc-reduction",
        "print-css",
        "unknown-transform-pass",
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
  "vendor-prefixing",
  "light-dark-lowering",
  "color-mix-lowering",
  "oklch-oklab-lowering",
  "color-function-lowering",
  "logical-to-physical",
  "nesting-unwrap",
  "supports-static-eval",
  "media-static-eval",
  "value-resolution",
  "custom-property-static-resolve",
  "dead-media-branch-removal",
  "dead-supports-branch-removal",
  "calc-reduction",
  "print-css",
  "unknown-transform-pass",
]);
assert.deepEqual(summary.unknownPassIds, ["unknown-transform-pass"]);
assert.equal(summary.execution.product, "omena-transform-passes.execution");
assert.equal(
  summary.execution.outputCss,
  '.dupe{display:block}.merge{color:red;background:#00f}.sel-a,.sel-b{border:0}.a.ready{margin:0;color:#fff;-webkit-user-select:none;-moz-user-select:none;-ms-user-select:none;user-select:none;opacity:1;background:url(img.svg);font-family:Demo;content:"/* keep */"}',
);
assert.deepEqual(summary.execution.executedPassIds, [
  "value-resolution",
  "custom-property-static-resolve",
  "dead-media-branch-removal",
  "dead-supports-branch-removal",
  "light-dark-lowering",
  "color-mix-lowering",
  "oklch-oklab-lowering",
  "color-function-lowering",
  "logical-to-physical",
  "nesting-unwrap",
  "supports-static-eval",
  "media-static-eval",
  "vendor-prefixing",
  "selector-is-where-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "calc-reduction",
  "whitespace-strip",
  "comment-strip",
  "empty-rule-removal",
  "number-compression",
  "unit-normalization",
  "color-compression",
  "url-quote-strip",
  "string-quote-normalize",
  "print-css",
]);
assert.deepEqual(summary.execution.plannedOnlyPassIds, []);
assert.equal(summary.execution.mutationCount, 65);
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
        "import-inline",
        "composes-resolution",
        "css-modules-class-hashing",
        "design-token-routing",
        "print-css",
      ],
      transformContext: {
        importInlines: [
          { importSource: "./tokens.css", replacementCss: ":root { --brand: red; }" },
        ],
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
  ":root { --brand: red; } ._button_x{  color: var(--theme-brand); } ._base_y{ color: blue; }",
);
assert.deepEqual(contextSummary.execution.executedPassIds, [
  "import-inline",
  "composes-resolution",
  "css-modules-class-hashing",
  "design-token-routing",
  "print-css",
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

const groupedComposesResult = spawnSync(
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
      styleSource: ".button, .card { composes: base; color: red; } .base { color: blue; }",
      requestedPassIds: ["composes-resolution", "print-css"],
      transformContext: {
        cssModuleComposesResolutions: [
          { localClassName: "button", exportedClassNames: ["button", "base"] },
          { localClassName: "card", exportedClassNames: ["card", "base"] },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(groupedComposesResult.status, 0, groupedComposesResult.stderr);
assert.equal(groupedComposesResult.error, undefined);

const groupedComposesSummary = JSON.parse(
  groupedComposesResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(groupedComposesSummary.product, "omena-query.transform-execute");
assert.equal(
  groupedComposesSummary.execution.outputCss,
  ".button, .card {  color: red; } .base { color: blue; }",
);
assert.deepEqual(groupedComposesSummary.execution.executedPassIds, [
  "composes-resolution",
  "print-css",
]);
assert.equal(groupedComposesSummary.execution.mutationCount, 1);

const alphaColorFunctionResult = spawnSync(
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
      stylePath: "colors.css",
      styleSource: ".card { accent-color: color(srgb 1 0 0 / 50%); color: color(srgb 0 0 1 / 1); }",
      requestedPassIds: ["color-function-lowering", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(alphaColorFunctionResult.status, 0, alphaColorFunctionResult.stderr);
assert.equal(alphaColorFunctionResult.error, undefined);

const alphaColorFunctionSummary = JSON.parse(
  alphaColorFunctionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(alphaColorFunctionSummary.product, "omena-query.transform-execute");
assert.equal(
  alphaColorFunctionSummary.execution.outputCss,
  ".card { accent-color: rgb(255 0 0 / .5); color: rgb(0 0 255); }",
);
assert.deepEqual(alphaColorFunctionSummary.execution.executedPassIds, [
  "color-function-lowering",
  "print-css",
]);
assert.equal(alphaColorFunctionSummary.execution.mutationCount, 2);

const alphaOkColorResult = spawnSync(
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
      stylePath: "ok-colors.css",
      styleSource: ".card { color: oklab(1 0 0 / 100%); accent-color: oklch(0% 0 0deg / 50%); }",
      requestedPassIds: ["oklch-oklab-lowering", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(alphaOkColorResult.status, 0, alphaOkColorResult.stderr);
assert.equal(alphaOkColorResult.error, undefined);

const alphaOkColorSummary = JSON.parse(alphaOkColorResult.stdout) as TransformExecuteSummaryV0;

assert.equal(alphaOkColorSummary.product, "omena-query.transform-execute");
assert.equal(
  alphaOkColorSummary.execution.outputCss,
  ".card { color: rgb(255 255 255); accent-color: rgb(0 0 0 / .5); }",
);
assert.deepEqual(alphaOkColorSummary.execution.executedPassIds, [
  "oklch-oklab-lowering",
  "print-css",
]);
assert.equal(alphaOkColorSummary.execution.mutationCount, 2);

const compositeValueResult = spawnSync(
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
      stylePath: "values.module.css",
      styleSource:
        "@value primary: #fff; @value gap: 8px; @value alias: primary; @value shadow: 0 0 4px primary; .button { color: alias; padding: gap gap; box-shadow: shadow; }",
      requestedPassIds: ["value-resolution", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(compositeValueResult.status, 0, compositeValueResult.stderr);
assert.equal(compositeValueResult.error, undefined);

const compositeValueSummary = JSON.parse(compositeValueResult.stdout) as TransformExecuteSummaryV0;

assert.equal(compositeValueSummary.product, "omena-query.transform-execute");
assert.equal(
  compositeValueSummary.execution.outputCss,
  "    .button { color: #fff; padding: 8px 8px; box-shadow: 0 0 4px #fff; }",
);
assert.deepEqual(compositeValueSummary.execution.executedPassIds, [
  "value-resolution",
  "print-css",
]);
assert.equal(compositeValueSummary.execution.mutationCount, 7);

const alphaColorCompressionResult = spawnSync(
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
      stylePath: "alpha-colors.css",
      styleSource:
        ".card { color: rgba(255, 0, 0, .5); box-shadow: 0 0 hsla(240, 100%, 50%, 50%); border-color: hwb(0 0% 0% / 50%); }",
      requestedPassIds: ["color-compression", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(alphaColorCompressionResult.status, 0, alphaColorCompressionResult.stderr);
assert.equal(alphaColorCompressionResult.error, undefined);

const alphaColorCompressionSummary = JSON.parse(
  alphaColorCompressionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(alphaColorCompressionSummary.product, "omena-query.transform-execute");
assert.equal(
  alphaColorCompressionSummary.execution.outputCss,
  ".card { color: #ff000080; box-shadow: 0 0 #0000ff80; border-color: #ff000080; }",
);
assert.deepEqual(alphaColorCompressionSummary.execution.executedPassIds, [
  "color-compression",
  "print-css",
]);
assert.equal(alphaColorCompressionSummary.execution.mutationCount, 3);

const colorMixPercentageResult = spawnSync(
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
      stylePath: "color-mix-percentages.css",
      styleSource:
        ".card { color: color-mix(in srgb, red 25%, blue 25%); border-color: color-mix(in srgb, red 75%, blue 75%); outline-color: color-mix(in srgb, red 0%, blue 0%); }",
      requestedPassIds: ["color-mix-lowering", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(colorMixPercentageResult.status, 0, colorMixPercentageResult.stderr);
assert.equal(colorMixPercentageResult.error, undefined);

const colorMixPercentageSummary = JSON.parse(
  colorMixPercentageResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(colorMixPercentageSummary.product, "omena-query.transform-execute");
assert.equal(
  colorMixPercentageSummary.execution.outputCss,
  ".card { color: rgb(128 0 128 / .5); border-color: rgb(128 0 128); outline-color: color-mix(in srgb, red 0%, blue 0%); }",
);
assert.deepEqual(colorMixPercentageSummary.execution.executedPassIds, [
  "color-mix-lowering",
  "print-css",
]);
assert.equal(colorMixPercentageSummary.execution.mutationCount, 2);

const semanticReachabilityResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-sources",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      targetStylePath: "Button.module.css",
      styles: [
        {
          stylePath: "Button.module.css",
          styleSource:
            ".button { composes: base utility; color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }",
        },
      ],
      requestedPassIds: ["tree-shake-class", "tree-shake-keyframes", "tree-shake-custom-property"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(semanticReachabilityResult.status, 0, semanticReachabilityResult.stderr);
assert.equal(semanticReachabilityResult.error, undefined);

const semanticReachabilitySummary = JSON.parse(
  semanticReachabilityResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(semanticReachabilitySummary.schemaVersion, "0");
assert.equal(semanticReachabilitySummary.product, "omena-query.consumer-build-style-source");
assert.equal(semanticReachabilitySummary.stylePath, "Button.module.css");
assert.deepEqual(semanticReachabilitySummary.unknownPassIds, []);
assert.equal(semanticReachabilitySummary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(semanticReachabilitySummary.execution.passPlan.allRequestedRegistered, true);
assertIncludesAll(
  semanticReachabilitySummary.execution.executedPassIds,
  ["tree-shake-class", "tree-shake-keyframes", "tree-shake-custom-property"],
  "semantic reachability executed passes",
);
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".button"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".base"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".utility"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@keyframes spin"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("--brand: red"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes(".dead"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes("@keyframes ghost"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes("--dead: blue"));
assert.deepEqual(
  semanticReachabilitySummary.execution.semanticRemovals.map((removal) => [
    removal.passId,
    removal.name,
  ]),
  [
    ["tree-shake-class", "dead"],
    ["tree-shake-keyframes", "ghost"],
    ["tree-shake-custom-property", "--dead"],
  ],
);
assert.equal(semanticReachabilitySummary.semanticRemovalCount, 3);
assertIncludesAll(
  semanticReachabilitySummary.readySurfaces,
  ["consumerBuildFacade", "multiSourceTransformContextProducer"],
  "semantic reachability build ready surfaces",
);

process.stdout.write(
  [
    "validated omena-query transform-execute runtime:",
    `executed=${summary.execution.executedPassIds.length}`,
    `mutations=${summary.execution.mutationCount}`,
    `unknown=${summary.unknownPassIds.length}`,
    `contextExecuted=${contextSummary.execution.executedPassIds.length}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
    `groupedComposesMutations=${groupedComposesSummary.execution.mutationCount}`,
    `alphaColorMutations=${alphaColorFunctionSummary.execution.mutationCount}`,
    `alphaOkColorMutations=${alphaOkColorSummary.execution.mutationCount}`,
    `compositeValueMutations=${compositeValueSummary.execution.mutationCount}`,
    `alphaColorCompressionMutations=${alphaColorCompressionSummary.execution.mutationCount}`,
    `colorMixPercentageMutations=${colorMixPercentageSummary.execution.mutationCount}`,
    `semanticRemovals=${semanticReachabilitySummary.semanticRemovalCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
