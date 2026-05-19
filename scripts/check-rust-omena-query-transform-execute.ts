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
    readonly plannedOnlyPassIds: readonly string[];
    readonly mutationCount: number;
    readonly cssModuleEvaluation?: {
      readonly evaluator: string;
      readonly evaluatedCss: string;
    } | null;
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

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
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
  "calc-reduction",
  "comment-strip",
  "empty-rule-removal",
  "number-compression",
  "unit-normalization",
  "color-compression",
  "url-quote-strip",
  "string-quote-normalize",
  "selector-merging",
  "whitespace-strip",
  "print-css",
]);
assert.deepEqual(summary.execution.plannedOnlyPassIds, []);
assert.equal(summary.execution.mutationCount, 61);
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

const groupedSupportsResult = spawnSync(
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
      stylePath: "supports-grouped.css",
      styleSource:
        "@supports ((display: grid) or (display: -ms-grid)) and (color: red) { .grid { color: red; } } @supports ((display: -ms-grid) or (-ms-ime-align: auto)) { .dead { color: blue; } } @supports not ((display: -ms-grid) or (-ms-ime-align: auto)) { .not-grouped { color: green; } } @supports not ((display: grid) or (display: -ms-grid)) { .not-dead { color: red; } }",
      requestedPassIds: ["supports-static-eval", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(groupedSupportsResult.status, 0, groupedSupportsResult.stderr);
assert.equal(groupedSupportsResult.error, undefined);

const groupedSupportsSummary = JSON.parse(
  groupedSupportsResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(groupedSupportsSummary.product, "omena-query.transform-execute");
assert.equal(
  groupedSupportsSummary.execution.outputCss,
  ".grid { color: red; }  .not-grouped { color: green; } ",
);
assert.deepEqual(groupedSupportsSummary.execution.executedPassIds, [
  "supports-static-eval",
  "print-css",
]);
assert.equal(groupedSupportsSummary.execution.mutationCount, 4);

const fontSupportsResult = spawnSync(
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
      stylePath: "supports-font.css",
      styleSource:
        "@supports font-tech(color-COLRv1) { .color-font { color: red; } } @supports not font-format(woff2) { .not-woff2 { color: blue; } } @supports font-format(embedded-opentype) { .eot { color: green; } } @supports not font-tech(-ms-color) { .not-ms { color: purple; } } @supports font-tech(unknown-thing) { .unknown { color: orange; } }",
      requestedPassIds: ["supports-static-eval", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(fontSupportsResult.status, 0, fontSupportsResult.stderr);
assert.equal(fontSupportsResult.error, undefined);

const fontSupportsSummary = JSON.parse(fontSupportsResult.stdout) as TransformExecuteSummaryV0;

assert.equal(fontSupportsSummary.product, "omena-query.transform-execute");
assert.equal(
  fontSupportsSummary.execution.outputCss,
  ".color-font { color: red; }   .not-ms { color: purple; } @supports font-tech(unknown-thing) { .unknown { color: orange; } }",
);
assert.deepEqual(fontSupportsSummary.execution.executedPassIds, [
  "supports-static-eval",
  "print-css",
]);
assert.equal(fontSupportsSummary.execution.mutationCount, 4);

const mediaListResult = spawnSync(
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
      stylePath: "media-list.css",
      styleSource:
        "@media all and (max-width: 0px) { .dead-and { color: red; } } @media not (max-width: 0px) { .not-zero { color: lime; } } @media not all and (max-width: 0px) { .not-impossible { color: teal; } } @media (min-width: 10px) and (max-width: 5px) { .impossible { color: red; } } @media (min-height: calc(4px + 4px)) and (max-height: 5px) { .impossible-calc { color: red; } } @media not all, (height<=0px) { .dead-list { color: blue; } } @media all, screen { .live { color: green; } } @media screen, (max-width: 0px) { .unknown { color: orange; } }",
      requestedPassIds: ["media-static-eval", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(mediaListResult.status, 0, mediaListResult.stderr);
assert.equal(mediaListResult.error, undefined);

const mediaListSummary = JSON.parse(mediaListResult.stdout) as TransformExecuteSummaryV0;

assert.equal(mediaListSummary.product, "omena-query.transform-execute");
assert.equal(
  mediaListSummary.execution.outputCss,
  " .not-zero { color: lime; } .not-impossible { color: teal; }    .live { color: green; } @media screen, (width<=0px) { .unknown { color: orange; } }",
);
assert.deepEqual(mediaListSummary.execution.executedPassIds, ["media-static-eval", "print-css"]);
assert.equal(mediaListSummary.execution.mutationCount, 8);

const mediaOrResult = spawnSync(
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
      stylePath: "media-or.css",
      styleSource:
        "@media (max-width: 0px) or all { .live { color: red; } } @media (max-width: 0px) or (height<=0px) { .dead { color: blue; } } @media screen or (max-width: 0px) { .unknown { color: green; } }",
      requestedPassIds: ["media-static-eval", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(mediaOrResult.status, 0, mediaOrResult.stderr);
assert.equal(mediaOrResult.error, undefined);

const mediaOrSummary = JSON.parse(mediaOrResult.stdout) as TransformExecuteSummaryV0;

assert.equal(mediaOrSummary.product, "omena-query.transform-execute");
assert.equal(
  mediaOrSummary.execution.outputCss,
  ".live { color: red; }  @media screen or (width<=0px) { .unknown { color: green; } }",
);
assert.deepEqual(mediaOrSummary.execution.executedPassIds, ["media-static-eval", "print-css"]);
assert.equal(mediaOrSummary.execution.mutationCount, 3);

const conditionalWrapperMergeResult = spawnSync(
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
      stylePath: "conditional-wrapper-merge.css",
      styleSource:
        "@media (prefers-color-scheme: dark) { .card { color: white; } } @media (prefers-color-scheme: dark) { .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } } @supports (display: flex) { .flex .child { display: flex; } }",
      requestedPassIds: ["rule-merging", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(conditionalWrapperMergeResult.status, 0, conditionalWrapperMergeResult.stderr);
assert.equal(conditionalWrapperMergeResult.error, undefined);

const conditionalWrapperMergeSummary = JSON.parse(
  conditionalWrapperMergeResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(conditionalWrapperMergeSummary.product, "omena-query.transform-execute");
assert.equal(
  conditionalWrapperMergeSummary.execution.outputCss,
  "@media (prefers-color-scheme: dark) { .card { color: white; } .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } .flex .child { display: flex; } }",
);
assert.deepEqual(conditionalWrapperMergeSummary.execution.executedPassIds, [
  "rule-merging",
  "print-css",
]);
assert.equal(conditionalWrapperMergeSummary.execution.mutationCount, 2);

const logicalCornerResult = spawnSync(
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
      stylePath: "logical-corners.css",
      styleSource:
        ".ltr { direction: ltr; border-start-start-radius: 1px; border-start-end-radius: 2px; border-end-start-radius: 3px; border-end-end-radius: 4px; } .rtl { direction: rtl; border-start-start-radius: 5px; border-start-end-radius: 6px; border-end-start-radius: 7px; border-end-end-radius: 8px; } .unknown { border-start-start-radius: 9px; }",
      requestedPassIds: ["logical-to-physical", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(logicalCornerResult.status, 0, logicalCornerResult.stderr);
assert.equal(logicalCornerResult.error, undefined);

const logicalCornerSummary = JSON.parse(logicalCornerResult.stdout) as TransformExecuteSummaryV0;

assert.equal(logicalCornerSummary.product, "omena-query.transform-execute");
assert.equal(
  logicalCornerSummary.execution.outputCss,
  ".ltr { direction: ltr; border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-left-radius: 3px; border-bottom-right-radius: 4px; } .rtl { direction: rtl; border-top-right-radius: 5px; border-top-left-radius: 6px; border-bottom-right-radius: 7px; border-bottom-left-radius: 8px; } .unknown { border-start-start-radius: 9px; }",
);
assert.deepEqual(logicalCornerSummary.execution.executedPassIds, [
  "logical-to-physical",
  "print-css",
]);
assert.equal(logicalCornerSummary.execution.mutationCount, 8);

const verticalLogicalResult = spawnSync(
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
      stylePath: "vertical-logical.css",
      styleSource:
        ".vrl { writing-mode: vertical-rl; direction: ltr; margin-block-start: 1px; margin-block-end: 2px; margin-inline-start: 3px; margin-inline-end: 4px; block-size: 10px; inline-size: 20px; border-start-start-radius: 1px; border-end-end-radius: 2px; inset-block: 5px 6px; padding-inline: 7px 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; inset-inline-start: 9px; border-start-end-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }",
      requestedPassIds: ["logical-to-physical", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(verticalLogicalResult.status, 0, verticalLogicalResult.stderr);
assert.equal(verticalLogicalResult.error, undefined);

const verticalLogicalSummary = JSON.parse(
  verticalLogicalResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(verticalLogicalSummary.product, "omena-query.transform-execute");
assert.equal(
  verticalLogicalSummary.execution.outputCss,
  ".vrl { writing-mode: vertical-rl; direction: ltr; margin-right: 1px; margin-left: 2px; margin-top: 3px; margin-bottom: 4px; width: 10px; height: 20px; border-top-right-radius: 1px; border-bottom-left-radius: 2px; right: 5px; left: 6px; padding-top: 7px; padding-bottom: 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; bottom: 9px; border-top-left-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }",
);
assert.deepEqual(verticalLogicalSummary.execution.executedPassIds, [
  "logical-to-physical",
  "print-css",
]);
assert.equal(verticalLogicalSummary.execution.mutationCount, 12);

const nestAtRuleResult = spawnSync(
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
      stylePath: "nest-at-rule.css",
      styleSource:
        ".card { color: red; @nest .theme & { color: blue; & .title { color: green; } } @nest &:is(:hover, :focus) { color: purple; } }",
      requestedPassIds: ["nesting-unwrap", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(nestAtRuleResult.status, 0, nestAtRuleResult.stderr);
assert.equal(nestAtRuleResult.error, undefined);

const nestAtRuleSummary = JSON.parse(nestAtRuleResult.stdout) as TransformExecuteSummaryV0;

assert.equal(nestAtRuleSummary.product, "omena-query.transform-execute");
assert.equal(
  nestAtRuleSummary.execution.outputCss,
  ".card { color: red; } .theme .card { color: blue; } .theme .card .title { color: green; } .card:is(:hover, :focus) { color: purple; }",
);
assert.deepEqual(nestAtRuleSummary.execution.executedPassIds, ["nesting-unwrap", "print-css"]);
assert.equal(nestAtRuleSummary.execution.mutationCount, 1);

const contextStyleSource =
  '@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; } .button :global(.external) { color: var(--brand); } :global { .reset { color: var(--brand); } } :local(.button) { composes: base; color: var(--brand); } :local { .button { color: var(--brand); } } @media (min-width: 1px) { .button { composes: base; color: var(--brand); } }';

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
          { originalName: "external", rewrittenName: "_external_global" },
          { originalName: "reset", rewrittenName: "_reset_should_not_apply" },
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
  ":root { --brand: red; } ._button_x{  color: var(--theme-brand); } ._base_y{ color: blue; } ._button_x .external{ color: var(--theme-brand); }  .reset { color: var(--theme-brand); }  ._button_x{  color: var(--theme-brand); }  ._button_x{ color: var(--theme-brand); }  @media (min-width: 1px) { ._button_x{  color: var(--theme-brand); } }",
);
assert.deepEqual(contextSummary.execution.executedPassIds, [
  "import-inline",
  "composes-resolution",
  "css-modules-class-hashing",
  "design-token-routing",
  "print-css",
]);
assert.deepEqual(contextSummary.execution.plannedOnlyPassIds, []);
assert.equal(contextSummary.execution.mutationCount, 20);
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

const transitiveImportInlineResult = spawnSync(
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
      targetStylePath: "/tmp/App.css",
      styles: [
        {
          stylePath: "/tmp/base.css",
          styleSource: ".base { color: red; }",
        },
        {
          stylePath: "/tmp/tokens.css",
          styleSource: '@import "./base.css"; .token { color: blue; }',
        },
        {
          stylePath: "/tmp/App.css",
          styleSource: '@import "./tokens.css"; .app { color: green; }',
        },
      ],
      requestedPassIds: ["import-inline", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(transitiveImportInlineResult.status, 0, transitiveImportInlineResult.stderr);
assert.equal(transitiveImportInlineResult.error, undefined);

const transitiveImportInlineSummary = JSON.parse(
  transitiveImportInlineResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(transitiveImportInlineSummary.product, "omena-query.consumer-build-style-source");
assert.equal(
  transitiveImportInlineSummary.execution.outputCss,
  ".base { color: red; } .token { color: blue; } .app { color: green; }",
);
assert.deepEqual(transitiveImportInlineSummary.execution.executedPassIds, [
  "import-inline",
  "print-css",
]);
assert.equal(transitiveImportInlineSummary.execution.mutationCount, 1);
assert.equal(transitiveImportInlineSummary.execution.cssImportInlines.length, 1);
assert.equal(
  transitiveImportInlineSummary.execution.cssImportInlines[0]?.replacementCss,
  ".base { color: red; } .token { color: blue; }",
);

const designTokenRecoveryResult = spawnSync(
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
      stylePath: "DesignTokens.module.css",
      styleSource:
        '@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--pkg-brand); } @keyframes pulse { to { color: var(--pkg-border); } } .button { color: var(--pkg-brand); box-shadow: 0 0 var(--pkg-border) var(--broken; }',
      requestedPassIds: ["design-token-routing", "print-css"],
      transformContext: {
        designTokenRoutes: [
          { tokenName: "--pkg-brand", routedValue: "var(--theme-brand)" },
          { tokenName: "--pkg-border", routedValue: "#123456" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(designTokenRecoveryResult.status, 0, designTokenRecoveryResult.stderr);
assert.equal(designTokenRecoveryResult.error, undefined);

const designTokenRecoverySummary = JSON.parse(
  designTokenRecoveryResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  designTokenRecoverySummary.execution.outputCss,
  '@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--theme-brand); } @keyframes pulse { to { color: #123456; } } .button { color: var(--theme-brand); box-shadow: 0 0 #123456 var(--broken; }',
);
assert.deepEqual(designTokenRecoverySummary.execution.executedPassIds, [
  "design-token-routing",
  "print-css",
]);
assert.equal(designTokenRecoverySummary.execution.mutationCount, 4);

const designTokenImportantResult = spawnSync(
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
      stylePath: "DesignTokenImportant.module.css",
      styleSource:
        ".button { color: var(--pkg-brand) !important; --local: var(--pkg-border) !important; --pkg-brand: var(--pkg-brand, black) !important; }",
      requestedPassIds: ["design-token-routing", "print-css"],
      transformContext: {
        designTokenRoutes: [
          { tokenName: "--pkg-brand", routedValue: "var(--theme-brand)" },
          { tokenName: "--pkg-border", routedValue: "#123456" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(designTokenImportantResult.status, 0, designTokenImportantResult.stderr);
assert.equal(designTokenImportantResult.error, undefined);

const designTokenImportantSummary = JSON.parse(
  designTokenImportantResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  designTokenImportantSummary.execution.outputCss,
  ".button { color: var(--theme-brand)!important; --local: #123456!important; --pkg-brand: var(--pkg-brand, black) !important; }",
);
assert.deepEqual(designTokenImportantSummary.execution.executedPassIds, [
  "design-token-routing",
  "print-css",
]);
assert.equal(designTokenImportantSummary.execution.mutationCount, 2);

const designTokenAtRulePreludeResult = spawnSync(
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
      stylePath: "DesignTokenAtRules.module.css",
      styleSource:
        "@custom-media --wide (min-width: var(--pkg-breakpoint)); @container card style(--theme: var(--pkg-theme)) { .button { color: var(--pkg-brand); } } @supports (color: var(--pkg-brand)) { .button { border-color: currentColor; } } @media (min-width: var(--pkg-breakpoint)) { .button { color: red; } }",
      requestedPassIds: ["design-token-routing", "print-css"],
      transformContext: {
        designTokenRoutes: [
          { tokenName: "--pkg-theme", routedValue: "var(--theme-mode)" },
          { tokenName: "--pkg-brand", routedValue: "#123456" },
          { tokenName: "--pkg-breakpoint", routedValue: "40rem" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(designTokenAtRulePreludeResult.status, 0, designTokenAtRulePreludeResult.stderr);
assert.equal(designTokenAtRulePreludeResult.error, undefined);

const designTokenAtRulePreludeSummary = JSON.parse(
  designTokenAtRulePreludeResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  designTokenAtRulePreludeSummary.execution.outputCss,
  "@custom-media --wide (min-width: 40rem); @container card style(--theme: var(--theme-mode)) { .button { color: #123456; } } @supports (color: #123456) { .button { border-color: currentColor; } } @media (min-width: 40rem) { .button { color: red; } }",
);
assert.deepEqual(designTokenAtRulePreludeSummary.execution.executedPassIds, [
  "design-token-routing",
  "print-css",
]);
assert.equal(designTokenAtRulePreludeSummary.execution.mutationCount, 5);

const designTokenAliasResult = spawnSync(
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
      stylePath: "DesignTokenAliases.module.css",
      styleSource:
        ":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--pkg-brand); --fallback-alias: var(--pkg-brand, var(--pkg-border)); --multi-fallback: var(--pkg-brand, linear-gradient(red, var(--pkg-border)), var(--pkg-border)); --bridge: var(--pkg-border); } .button { color: var(--alias); }",
      requestedPassIds: ["design-token-routing", "print-css"],
      transformContext: {
        designTokenRoutes: [
          { tokenName: "--pkg-brand", routedValue: "var(--theme-brand)" },
          { tokenName: "--pkg-border", routedValue: "#123456" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(designTokenAliasResult.status, 0, designTokenAliasResult.stderr);
assert.equal(designTokenAliasResult.error, undefined);

const designTokenAliasSummary = JSON.parse(
  designTokenAliasResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  designTokenAliasSummary.execution.outputCss,
  ":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--theme-brand); --fallback-alias: var(--theme-brand, #123456); --multi-fallback: var(--theme-brand, linear-gradient(red, #123456), #123456); --bridge: #123456; } .button { color: var(--alias); }",
);
assert.deepEqual(designTokenAliasSummary.execution.executedPassIds, [
  "design-token-routing",
  "print-css",
]);
assert.equal(designTokenAliasSummary.execution.mutationCount, 4);

const localComposesResolutionResult = spawnSync(
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
      styleSource:
        ".button { composes: base global(reset); color: red; } .base { composes: utility; color: blue; } .utility { color: green; }",
      requestedPassIds: ["composes-resolution", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(localComposesResolutionResult.status, 0, localComposesResolutionResult.stderr);
assert.equal(localComposesResolutionResult.error, undefined);

const localComposesResolutionSummary = JSON.parse(
  localComposesResolutionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  localComposesResolutionSummary.execution.outputCss,
  ".button {  color: red; } .base {  color: blue; } .utility { color: green; }",
);
assert.deepEqual(localComposesResolutionSummary.execution.executedPassIds, [
  "composes-resolution",
  "print-css",
]);
assert.deepEqual(localComposesResolutionSummary.execution.plannedOnlyPassIds, []);
assert.equal(localComposesResolutionSummary.execution.mutationCount, 2);
assert.deepEqual(localComposesResolutionSummary.execution.cssModuleComposesExports, [
  { localClassName: "base", exportedClassNames: ["base", "utility"] },
  { localClassName: "button", exportedClassNames: ["button", "base", "reset", "utility"] },
]);

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
      styleSource:
        ".button, .card { composes: base; color: red; } :local(.button, .card) { composes: base; color: purple; } :global { .button { composes: base; color: pink; } } .base { color: blue; }",
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
  ".button, .card {  color: red; } :local(.button, .card) {  color: purple; } :global { .button { composes: base; color: pink; } } .base { color: blue; }",
);
assert.deepEqual(groupedComposesSummary.execution.executedPassIds, [
  "composes-resolution",
  "print-css",
]);
assert.equal(groupedComposesSummary.execution.mutationCount, 2);

const globalComposesHashResult = spawnSync(
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
      styleSource: ".button { composes: base global(reset); color: red; } .base { color: blue; }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
      transformContext: {
        classNameRewrites: [
          { originalName: "button", rewrittenName: "_button_x" },
          { originalName: "base", rewrittenName: "_base_y" },
          { originalName: "reset", rewrittenName: "_reset_should_not_apply" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(globalComposesHashResult.status, 0, globalComposesHashResult.stderr);
assert.equal(globalComposesHashResult.error, undefined);

const globalComposesHashSummary = JSON.parse(
  globalComposesHashResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(globalComposesHashSummary.product, "omena-query.transform-execute");
assert.equal(
  globalComposesHashSummary.execution.outputCss,
  "._button_x{ composes: _base_y reset; color: red; } ._base_y{ color: blue; }",
);
assert.deepEqual(globalComposesHashSummary.execution.executedPassIds, [
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(globalComposesHashSummary.execution.mutationCount, 3);

const scopedClassHashResult = spawnSync(
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
      stylePath: "Scoped.module.css",
      styleSource: "@scope (.card) to (:global(.footer)) { .title { color: red; } }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
      transformContext: {
        classNameRewrites: [
          { originalName: "card", rewrittenName: "_card_x" },
          { originalName: "footer", rewrittenName: "_footer_should_not_apply" },
          { originalName: "title", rewrittenName: "_title_z" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(scopedClassHashResult.status, 0, scopedClassHashResult.stderr);
assert.equal(scopedClassHashResult.error, undefined);

const scopedClassHashSummary = JSON.parse(
  scopedClassHashResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(scopedClassHashSummary.product, "omena-query.transform-execute");
assert.equal(
  scopedClassHashSummary.execution.outputCss,
  "@scope (._card_x) to (.footer) { ._title_z{ color: red; } }",
);
assert.deepEqual(scopedClassHashSummary.execution.executedPassIds, [
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(scopedClassHashSummary.execution.mutationCount, 2);

const supportsSelectorClassHashResult = spawnSync(
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
      stylePath: "Supports.module.css",
      styleSource:
        "@supports selector(.card:has(:global(.footer), .title)) { .item { color: red; } } @supports (background: paint(.card)) { .paint { color: blue; } }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
      transformContext: {
        classNameRewrites: [
          { originalName: "card", rewrittenName: "_card_x" },
          { originalName: "footer", rewrittenName: "_footer_should_not_apply" },
          { originalName: "title", rewrittenName: "_title_z" },
          { originalName: "item", rewrittenName: "_item_q" },
          { originalName: "paint", rewrittenName: "_paint_p" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(supportsSelectorClassHashResult.status, 0, supportsSelectorClassHashResult.stderr);
assert.equal(supportsSelectorClassHashResult.error, undefined);

const supportsSelectorClassHashSummary = JSON.parse(
  supportsSelectorClassHashResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(supportsSelectorClassHashSummary.product, "omena-query.transform-execute");
assert.equal(
  supportsSelectorClassHashSummary.execution.outputCss,
  "@supports selector(._card_x:has(.footer, ._title_z)) { ._item_q{ color: red; } } @supports (background: paint(.card)) { ._paint_p{ color: blue; } }",
);
assert.deepEqual(supportsSelectorClassHashSummary.execution.executedPassIds, [
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(supportsSelectorClassHashSummary.execution.mutationCount, 3);

const escapedClassHashResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Escaped.module.css",
      styleSource:
        ".foo\\:bar { color: red; } :local(.foo\\:bar) { color: blue; } :global(.foo\\:bar) .foo\\:bar { color: green; }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(escapedClassHashResult.status, 0, escapedClassHashResult.stderr);
assert.equal(escapedClassHashResult.error, undefined);

const escapedClassHashSummary = JSON.parse(escapedClassHashResult.stdout) as ConsumerBuildSummaryV0;

assert.equal(
  escapedClassHashSummary.execution.outputCss,
  "._foo_bar_0{ color: red; } ._foo_bar_0{ color: blue; } .foo\\:bar ._foo_bar_0{ color: green; }",
);
assert.deepEqual(escapedClassHashSummary.execution.executedPassIds, [
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(escapedClassHashSummary.execution.mutationCount, 3);

const hexEscapedClassHashResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "HexEscaped.module.css",
      styleSource: ".hex\\3A bar { color: red; } :local(.hex\\:bar) { color: blue; }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(hexEscapedClassHashResult.status, 0, hexEscapedClassHashResult.stderr);
assert.equal(hexEscapedClassHashResult.error, undefined);

const hexEscapedClassHashSummary = JSON.parse(
  hexEscapedClassHashResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(
  hexEscapedClassHashSummary.execution.outputCss,
  "._hex_bar_0{ color: red; } ._hex_bar_0{ color: blue; }",
);
assert.deepEqual(hexEscapedClassHashSummary.execution.executedPassIds, [
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(hexEscapedClassHashSummary.execution.mutationCount, 2);

const nonCssModuleClassHashResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.test.css",
      styleSource: ".button { color: red; } .base { color: blue; }",
      requestedPassIds: ["css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(nonCssModuleClassHashResult.status, 0, nonCssModuleClassHashResult.stderr);
assert.equal(nonCssModuleClassHashResult.error, undefined);

const nonCssModuleClassHashSummary = JSON.parse(
  nonCssModuleClassHashResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(
  nonCssModuleClassHashSummary.execution.outputCss,
  ".button { color: red; } .base { color: blue; }",
);
assert.deepEqual(nonCssModuleClassHashSummary.execution.executedPassIds, ["print-css"]);
assert.deepEqual(nonCssModuleClassHashSummary.execution.plannedOnlyPassIds, [
  "css-modules-class-hashing",
]);

const escapedClassTreeShakeResult = spawnSync(
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
      targetStylePath: "Escaped.module.css",
      styles: [
        {
          stylePath: "Escaped.module.css",
          styleSource:
            ".foo\\:bar { color: red; } .dead { color: blue; } .foo\\:bar:hover { color: green; } .dead, .foo\\:bar { color: cyan; } .hex\\3A bar { color: purple; } .hex-dead { color: black; }",
        },
      ],
      requestedPassIds: ["tree-shake-class", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["foo:bar", "hex:bar"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(escapedClassTreeShakeResult.status, 0, escapedClassTreeShakeResult.stderr);
assert.equal(escapedClassTreeShakeResult.error, undefined);

const escapedClassTreeShakeSummary = JSON.parse(
  escapedClassTreeShakeResult.stdout,
) as ConsumerBuildSummaryV0;

assert.ok(escapedClassTreeShakeSummary.execution.outputCss.includes(".foo\\:bar { color: red; }"));
assert.ok(
  escapedClassTreeShakeSummary.execution.outputCss.includes(".foo\\:bar:hover { color: green; }"),
);
assert.ok(escapedClassTreeShakeSummary.execution.outputCss.includes(".foo\\:bar { color: cyan; }"));
assert.ok(
  escapedClassTreeShakeSummary.execution.outputCss.includes(".hex\\3A bar { color: purple; }"),
);
assert.ok(!escapedClassTreeShakeSummary.execution.outputCss.includes(".dead {"));
assert.ok(!escapedClassTreeShakeSummary.execution.outputCss.includes(".dead,"));
assert.ok(!escapedClassTreeShakeSummary.execution.outputCss.includes(".hex-dead"));
assert.deepEqual(escapedClassTreeShakeSummary.execution.executedPassIds, [
  "tree-shake-class",
  "print-css",
]);
assertIncludesAll(
  escapedClassTreeShakeSummary.execution.semanticRemovals.map(
    (removal) => `${removal.passId}:${removal.name}`,
  ),
  ["tree-shake-class:dead"],
  "escaped class tree-shaking removals",
);

const localComposesTreeShakeResult = spawnSync(
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
      stylePath: "LocalComposes.module.css",
      styleSource:
        ".button { composes: base utility global(reset); color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }",
      requestedPassIds: [
        "tree-shake-class",
        "tree-shake-keyframes",
        "tree-shake-custom-property",
        "print-css",
      ],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(localComposesTreeShakeResult.status, 0, localComposesTreeShakeResult.stderr);
assert.equal(localComposesTreeShakeResult.error, undefined);

const localComposesTreeShakeSummary = JSON.parse(
  localComposesTreeShakeResult.stdout,
) as TransformExecuteSummaryV0;

assert.ok(localComposesTreeShakeSummary.execution.outputCss.includes(".button"));
assert.ok(localComposesTreeShakeSummary.execution.outputCss.includes(".base"));
assert.ok(localComposesTreeShakeSummary.execution.outputCss.includes(".utility"));
assert.ok(localComposesTreeShakeSummary.execution.outputCss.includes("@keyframes spin"));
assert.ok(localComposesTreeShakeSummary.execution.outputCss.includes("--brand: red"));
assert.ok(!localComposesTreeShakeSummary.execution.outputCss.includes(".dead"));
assert.ok(!localComposesTreeShakeSummary.execution.outputCss.includes("@keyframes ghost"));
assert.ok(!localComposesTreeShakeSummary.execution.outputCss.includes("--dead: blue"));
assertIncludesAll(
  localComposesTreeShakeSummary.execution.executedPassIds,
  ["tree-shake-class", "tree-shake-keyframes", "tree-shake-custom-property", "print-css"],
  "local composes tree-shaking executed passes",
);

const escapedKeyframeTreeShakeResult = spawnSync(
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
      targetStylePath: "EscapedKeyframes.module.css",
      styles: [
        {
          stylePath: "EscapedKeyframes.module.css",
          styleSource:
            ".btn { animation: spin\\:fast 1s ease; } .dead-ref { animation: dead 1s ease; } @keyframes spin\\:fast { to { opacity: 1; } } @keyframes hex\\3A fast { to { opacity: .5; } } @keyframes dead { to { opacity: 0; } }",
        },
      ],
      requestedPassIds: ["tree-shake-keyframes", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
        reachableKeyframeNames: ["hex:fast"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(escapedKeyframeTreeShakeResult.status, 0, escapedKeyframeTreeShakeResult.stderr);
assert.equal(escapedKeyframeTreeShakeResult.error, undefined);

const escapedKeyframeTreeShakeSummary = JSON.parse(
  escapedKeyframeTreeShakeResult.stdout,
) as ConsumerBuildSummaryV0;

assert.ok(escapedKeyframeTreeShakeSummary.execution.outputCss.includes("@keyframes spin\\:fast"));
assert.ok(escapedKeyframeTreeShakeSummary.execution.outputCss.includes("@keyframes hex\\3A fast"));
assert.ok(!escapedKeyframeTreeShakeSummary.execution.outputCss.includes("@keyframes dead"));
assert.deepEqual(escapedKeyframeTreeShakeSummary.execution.executedPassIds, [
  "tree-shake-keyframes",
  "print-css",
]);
assertIncludesAll(
  escapedKeyframeTreeShakeSummary.execution.semanticRemovals.map(
    (removal) => `${removal.passId}:${removal.name}`,
  ),
  ["tree-shake-keyframes:dead"],
  "escaped keyframe tree-shaking removals",
);

const atRulePreludeCustomPropertyTreeShakeResult = spawnSync(
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
      targetStylePath: "AtRulePrelude.module.css",
      styles: [
        {
          stylePath: "AtRulePrelude.module.css",
          styleSource:
            ":root { --gate: grid; --wide: 40rem; --dead: blue; --unreachable-width: 80rem; } @supports (display: var(--gate)) { .btn { color: red; } } @media (min-width: var(--wide)) { .btn { color: blue; } } @supports (color: var(--dead)) { .dead { color: black; } } @media (min-width: var(--unreachable-width)) { .dead { color: black; } }",
        },
      ],
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  atRulePreludeCustomPropertyTreeShakeResult.status,
  0,
  atRulePreludeCustomPropertyTreeShakeResult.stderr,
);
assert.equal(atRulePreludeCustomPropertyTreeShakeResult.error, undefined);

const atRulePreludeCustomPropertyTreeShakeSummary = JSON.parse(
  atRulePreludeCustomPropertyTreeShakeResult.stdout,
) as ConsumerBuildSummaryV0;

assert.ok(
  atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes("--gate: grid;"),
);
assert.ok(
  atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes("--wide: 40rem;"),
);
assert.ok(
  atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes(
    "@supports (display: var(--gate))",
  ),
);
assert.ok(
  atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes(
    "@media (min-width: var(--wide))",
  ),
);
assert.ok(
  !atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes("--dead: blue;"),
);
assert.ok(
  !atRulePreludeCustomPropertyTreeShakeSummary.execution.outputCss.includes(
    "--unreachable-width: 80rem;",
  ),
);
assert.deepEqual(atRulePreludeCustomPropertyTreeShakeSummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assertIncludesAll(
  atRulePreludeCustomPropertyTreeShakeSummary.execution.semanticRemovals.map(
    (removal) => `${removal.passId}:${removal.name}`,
  ),
  ["tree-shake-custom-property:--dead", "tree-shake-custom-property:--unreachable-width"],
  "at-rule prelude custom property tree-shaking removals",
);

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
      styleSource:
        ".card { accent-color: color(srgb 1 0 0 / 50%); color: color(srgb 0 0 1 / 1); text-shadow: 0 0 1px color(srgb-linear 0.5 0 0.5); }",
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
  ".card { accent-color: rgb(255 0 0 / .5); color: rgb(0 0 255); text-shadow: 0 0 1px rgb(188 0 188); }",
);
assert.deepEqual(alphaColorFunctionSummary.execution.executedPassIds, [
  "color-function-lowering",
  "print-css",
]);
assert.equal(alphaColorFunctionSummary.execution.mutationCount, 3);

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
        "@value primary: #fff; @value gap: 8px; @value alias: primary; @value shadow: 0 0 4px primary; @value bp: 40rem; @value wide: 80rem; @value width: 1px; .button { color: alias; padding: gap gap; box-shadow: shadow; } @media screen and (min-width: bp) and (width >= wide) and (bp <= width <= wide) { .button { color: alias; } } @container card (inline-size >= wide) { .button { padding: gap; } }",
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
  "       .button { color: #fff; padding: 8px 8px; box-shadow: 0 0 4px #fff; } @media screen and (min-width: 40rem) and (width >= 80rem) and (40rem <= width <= 80rem) { .button { color: #fff; } } @container card (inline-size >= 80rem) { .button { padding: 8px; } }",
);
assert.deepEqual(compositeValueSummary.execution.executedPassIds, [
  "value-resolution",
  "print-css",
]);
assert.equal(compositeValueSummary.execution.mutationCount, 17);

const importedValueResult = spawnSync(
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
      stylePath: "imported-values.module.css",
      styleSource:
        '@value primary as brand, gap, tone from "./tokens.module.css"; @custom-media --gap (min-width: gap); .button { color: brand; margin: gap; border-color: tone; } @media (min-width: gap) { .button { color: brand; } } @supports (width: gap) { .button { color: brand; } }',
      requestedPassIds: ["value-resolution", "print-css"],
      transformContext: {
        cssModuleValueResolutions: [
          { localName: "brand", resolvedValue: "#fff" },
          { localName: "gap", resolvedValue: "8px" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(importedValueResult.status, 0, importedValueResult.stderr);
assert.equal(importedValueResult.error, undefined);

const importedValueSummary = JSON.parse(importedValueResult.stdout) as TransformExecuteSummaryV0;

assert.equal(importedValueSummary.product, "omena-query.transform-execute");
assert.equal(
  importedValueSummary.execution.outputCss,
  '@value tone from "./tokens.module.css"; @custom-media --gap (min-width: 8px); .button { color: #fff; margin: 8px; border-color: tone; } @media (min-width: 8px) { .button { color: #fff; } } @supports (width: 8px) { .button { color: #fff; } }',
);
assert.deepEqual(importedValueSummary.execution.executedPassIds, ["value-resolution", "print-css"]);
assert.equal(importedValueSummary.execution.mutationCount, 8);

const transitiveImportedValueResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.css",
      styles: [
        {
          stylePath: "/tmp/base.module.css",
          styleSource: "@value primary: #fff;",
        },
        {
          stylePath: "/tmp/tokens.module.css",
          styleSource: '@value primary from "./base.module.css"; @value alias: primary;',
        },
        {
          stylePath: "/tmp/App.module.css",
          styleSource: '@value alias as brand from "./tokens.module.css"; .btn { color: brand; }',
        },
      ],
      requestedPassIds: ["value-resolution", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(transitiveImportedValueResult.status, 0, transitiveImportedValueResult.stderr);
assert.equal(transitiveImportedValueResult.error, undefined);

const transitiveImportedValueSummary = JSON.parse(
  transitiveImportedValueResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(transitiveImportedValueSummary.product, "omena-query.consumer-build-style-source");
assert.equal(transitiveImportedValueSummary.execution.outputCss, " .btn { color: #fff; }");
assert.deepEqual(transitiveImportedValueSummary.execution.executedPassIds, [
  "value-resolution",
  "print-css",
]);
assert.equal(transitiveImportedValueSummary.execution.mutationCount, 2);

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
        ".card { color: rgba(255, 0, 0, .5); box-shadow: 0 0 hsla(240, 100%, 50%, 50%); border-color: hwb(0 0% 0% / 50%); outline-color: transparent; text-shadow: 0 0 transparent; }",
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
  ".card { color: #ff000080; box-shadow: 0 0 #0000ff80; border-color: #ff000080; outline-color: #0000; text-shadow: 0 0 #0000; }",
);
assert.deepEqual(alphaColorCompressionSummary.execution.executedPassIds, [
  "color-compression",
  "print-css",
]);
assert.equal(alphaColorCompressionSummary.execution.mutationCount, 5);

const namedHexColorCompressionResult = spawnSync(
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
      stylePath: "named-hex-colors.css",
      styleSource:
        ".card { color: #ff0000; outline-color: #808080; background: #0000ff; border-color: #FFFFFF; }",
      requestedPassIds: ["color-compression", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(namedHexColorCompressionResult.status, 0, namedHexColorCompressionResult.stderr);
assert.equal(namedHexColorCompressionResult.error, undefined);

const namedHexColorCompressionSummary = JSON.parse(
  namedHexColorCompressionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(namedHexColorCompressionSummary.product, "omena-query.transform-execute");
assert.equal(
  namedHexColorCompressionSummary.execution.outputCss,
  ".card { color: red; outline-color: gray; background: #00f; border-color: #fff; }",
);
assert.deepEqual(namedHexColorCompressionSummary.execution.executedPassIds, [
  "color-compression",
  "print-css",
]);
assert.equal(namedHexColorCompressionSummary.execution.mutationCount, 4);

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
        ".card { color: color-mix(in srgb, red 25%, blue 25%); border-color: color-mix(in srgb, red 75%, blue 75%); border: 1px solid color-mix(in srgb, red, blue); box-shadow: 0 0 1px color-mix(in srgb, red, blue); column-rule: 1px solid color-mix(in srgb, red, blue); outline-color: color-mix(in srgb, red 0%, blue 0%); }",
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
  ".card { color: rgb(128 0 128 / .5); border-color: rgb(128 0 128); border: 1px solid rgb(128 0 128); box-shadow: 0 0 1px rgb(128 0 128); column-rule: 1px solid rgb(128 0 128); outline-color: color-mix(in srgb, red 0%, blue 0%); }",
);
assert.deepEqual(colorMixPercentageSummary.execution.executedPassIds, [
  "color-mix-lowering",
  "print-css",
]);
assert.equal(colorMixPercentageSummary.execution.mutationCount, 5);

const colorMixAlphaResult = spawnSync(
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
      stylePath: "color-mix-alpha.css",
      styleSource:
        ".card { color: color-mix(in srgb, 50% red, transparent 50%); background-color: color-mix(in srgb, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); outline-color: color-mix(in srgb, rgb(100% 0% 0% / .7) 20%, 60% rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb, 50% #ff000080, 50% blue); }",
      requestedPassIds: ["color-mix-lowering", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(colorMixAlphaResult.status, 0, colorMixAlphaResult.stderr);
assert.equal(colorMixAlphaResult.error, undefined);

const colorMixAlphaSummary = JSON.parse(colorMixAlphaResult.stdout) as TransformExecuteSummaryV0;

assert.equal(colorMixAlphaSummary.product, "omena-query.transform-execute");
assert.equal(
  colorMixAlphaSummary.execution.outputCss,
  ".card { color: rgb(255 0 0 / .5); background-color: rgb(137 118 0 / .325); outline-color: rgb(137 118 0 / .26); border-color: rgb(85 0 170 / .75098); }",
);
assert.deepEqual(colorMixAlphaSummary.execution.executedPassIds, [
  "color-mix-lowering",
  "print-css",
]);
assert.equal(colorMixAlphaSummary.execution.mutationCount, 4);

const colorMixLinearResult = spawnSync(
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
      stylePath: "color-mix-linear.css",
      styleSource:
        ".card { color: color-mix(in srgb-linear, red 50%, blue 50%); background-color: color-mix(in srgb-linear, 50% red, transparent 50%); outline-color: color-mix(in srgb-linear, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb-linear, 50% #ff000080, 50% blue); }",
      requestedPassIds: ["color-mix-lowering", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(colorMixLinearResult.status, 0, colorMixLinearResult.stderr);
assert.equal(colorMixLinearResult.error, undefined);

const colorMixLinearSummary = JSON.parse(colorMixLinearResult.stdout) as TransformExecuteSummaryV0;

assert.equal(colorMixLinearSummary.product, "omena-query.transform-execute");
assert.equal(
  colorMixLinearSummary.execution.outputCss,
  ".card { color: rgb(188 0 188); background-color: rgb(255 0 0 / .5); outline-color: rgb(194 181 0 / .325); border-color: rgb(156 0 213 / .75098); }",
);
assert.deepEqual(colorMixLinearSummary.execution.executedPassIds, [
  "color-mix-lowering",
  "print-css",
]);
assert.equal(colorMixLinearSummary.execution.mutationCount, 4);

const mathFunctionReductionResult = spawnSync(
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
      stylePath: "math-functions.css",
      styleSource:
        ".card { width: min(10px, 4px); height: max(1rem, 2rem); margin: min(1px, 1rem); opacity: max(.2, .5); outline-width: calc((2px * 3)); flex-basis: calc(2px * 3 * 4); inline-size: min(10px, max(2px, 4px)); line-height: clamp(.1, .5, .9); stroke-width: abs(-2px); order: sign(-10px); top: round(nearest, 10px, 3px); right: round(up, 10px, 3px); bottom: round(down, 10px, 3px); left: round(to-zero, 10px, 3px); rotate: round(nearest, 5px, 2px); margin-left: mod(10px, 3px); margin-right: rem(10px, 4px); perspective: mod(-10px, 3px); border-spacing: hypot(3px, 4px); flex-grow: hypot(3, 4); margin-bottom: hypot(3px, 4rem); }",
      requestedPassIds: ["calc-reduction", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(mathFunctionReductionResult.status, 0, mathFunctionReductionResult.stderr);
assert.equal(mathFunctionReductionResult.error, undefined);

const mathFunctionReductionSummary = JSON.parse(
  mathFunctionReductionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(mathFunctionReductionSummary.product, "omena-query.transform-execute");
assert.equal(
  mathFunctionReductionSummary.execution.outputCss,
  ".card { width: 4px; height: 2rem; margin: min(1px, 1rem); opacity: 0.5; outline-width: 6px; flex-basis: 24px; inline-size: 4px; line-height: 0.5; stroke-width: 2px; order: -1; top: 9px; right: 12px; bottom: 9px; left: 9px; rotate: round(nearest, 5px, 2px); margin-left: 1px; margin-right: 2px; perspective: mod(-10px, 3px); border-spacing: 5px; flex-grow: 5; margin-bottom: hypot(3px, 4rem); }",
);
assert.deepEqual(mathFunctionReductionSummary.execution.executedPassIds, [
  "calc-reduction",
  "print-css",
]);
assert.equal(mathFunctionReductionSummary.execution.mutationCount, 17);

const staticVarShadowResult = spawnSync(
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
      stylePath: "shadowed-custom-properties.css",
      styleSource:
        '@property --registered { syntax: "<length>"; inherits: false; initial-value: var(--gap); } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @keyframes pulse { to { color: var(--gap); } } :root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap) var(--also-broken; --tone: red; --tone: blue !important; --dynamic: env(theme-color); --css-wide: initial; } .card { --brand: blue; color: var(--brand); margin: var(--gap); border-color: var(--tone); margin-left: var(--registered); box-shadow: 0 0 var(--gap) var(--broken; text-decoration-color: var(--dynamic); } .other { color: var(--brand); box-shadow: var(--shadow); background: var(--missing, linear-gradient(var(--gap), white), var(--gap)); outline-color: var(--css-wide, blue); }',
      requestedPassIds: ["custom-property-static-resolve", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticVarShadowResult.status, 0, staticVarShadowResult.stderr);
assert.equal(staticVarShadowResult.error, undefined);

const staticVarShadowSummary = JSON.parse(
  staticVarShadowResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(staticVarShadowSummary.product, "omena-query.transform-execute");
assert.equal(
  staticVarShadowSummary.execution.outputCss,
  '@property --registered { syntax: "<length>"; inherits: false; initial-value: 2rem; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @keyframes pulse { to { color: 2rem; } } :root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap) var(--also-broken; --tone: red; --tone: blue !important; --dynamic: env(theme-color); --css-wide: initial; } .card { --brand: blue; color: var(--brand); margin: 2rem; border-color: var(--tone); margin-left: 2rem; box-shadow: 0 0 2rem var(--broken; text-decoration-color: var(--dynamic); } .other { color: var(--brand); box-shadow: 0 0 2rem var(--also-broken; background: linear-gradient(2rem, white), 2rem; outline-color: var(--css-wide, blue); }',
);
assert.deepEqual(staticVarShadowSummary.execution.executedPassIds, [
  "custom-property-static-resolve",
  "print-css",
]);
assert.equal(staticVarShadowSummary.execution.mutationCount, 7);

const staticVarPreludeResult = spawnSync(
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
      stylePath: "at-rule-prelude-custom-properties.css",
      styleSource:
        ":root { --wide: 40rem; --mode: dark; --color: red; --scope-root: .card; } @custom-media --wide (min-width: var(--wide)); @container card style(--mode: var(--mode)) { .card { color: var(--color); } } @supports (color: var(--color)) { .card { border-color: currentColor; } } @media (min-width: var(--wide)) { .card { color: var(--color); } } @scope (var(--scope-root)) { .card { color: var(--color); } }",
      requestedPassIds: ["custom-property-static-resolve", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticVarPreludeResult.status, 0, staticVarPreludeResult.stderr);
assert.equal(staticVarPreludeResult.error, undefined);

const staticVarPreludeSummary = JSON.parse(
  staticVarPreludeResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(staticVarPreludeSummary.product, "omena-query.transform-execute");
assert.equal(
  staticVarPreludeSummary.execution.outputCss,
  ":root { --wide: 40rem; --mode: dark; --color: red; --scope-root: .card; } @custom-media --wide (min-width: 40rem); @container card style(--mode: dark) { .card { color: red; } } @supports (color: red) { .card { border-color: currentColor; } } @media (min-width: 40rem) { .card { color: red; } } @scope (.card) { .card { color: red; } }",
);
assert.deepEqual(staticVarPreludeSummary.execution.executedPassIds, [
  "custom-property-static-resolve",
  "print-css",
]);
assert.equal(staticVarPreludeSummary.execution.mutationCount, 8);

const staticBranchResolutionResult = spawnSync(
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
      stylePath: "static-branch-resolution.module.css",
      styleSource:
        "@value mode: grid; @value bp: 0px; :root { --display: grid; --zero: 0px; } @supports (display: mode) { .value { color: red; } } @supports (display: var(--display)) { .var { color: blue; } } @media (max-width: bp) { .value-media { color: red; } } @media (max-width: var(--zero)) { .var-media { color: blue; } }",
      requestedPassIds: [
        "media-static-eval",
        "supports-static-eval",
        "custom-property-static-resolve",
        "value-resolution",
        "print-css",
      ],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticBranchResolutionResult.status, 0, staticBranchResolutionResult.stderr);
assert.equal(staticBranchResolutionResult.error, undefined);

const staticBranchResolutionSummary = JSON.parse(
  staticBranchResolutionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(staticBranchResolutionSummary.product, "omena-query.transform-execute");
assert.equal(
  staticBranchResolutionSummary.execution.outputCss,
  "  :root { --display: grid; --zero: 0px; } .value { color: red; } .var { color: blue; }  ",
);
assert.deepEqual(staticBranchResolutionSummary.execution.executedPassIds, [
  "value-resolution",
  "custom-property-static-resolve",
  "supports-static-eval",
  "media-static-eval",
  "print-css",
]);
assert.equal(staticBranchResolutionSummary.execution.mutationCount, 10);
assert.equal(staticBranchResolutionSummary.execution.passPlan.violatedDagEdgeCount, 0);

const scopeValueResolutionResult = spawnSync(
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
      stylePath: "scope-values.module.css",
      styleSource:
        "@value scopeRoot: .card; @value scopeLimit: #app; @value rootScope: :root; @value tone: red; @value dead: blue; @scope (scopeRoot) to (scopeLimit) { .card { color: tone; } } @scope (rootScope) { .card { border-color: tone; } }",
      requestedPassIds: ["value-resolution", "tree-shake-value", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["card"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(scopeValueResolutionResult.status, 0, scopeValueResolutionResult.stderr);
assert.equal(scopeValueResolutionResult.error, undefined);

const scopeValueResolutionSummary = JSON.parse(
  scopeValueResolutionResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(scopeValueResolutionSummary.product, "omena-query.transform-execute");
assert.equal(
  scopeValueResolutionSummary.execution.outputCss,
  "     @scope (.card) to (#app) { .card { color: red; } } @scope (:root) { .card { border-color: red; } }",
);
assert.deepEqual(scopeValueResolutionSummary.execution.executedPassIds, [
  "value-resolution",
  "tree-shake-value",
  "print-css",
]);
assert.deepEqual(scopeValueResolutionSummary.execution.plannedOnlyPassIds, []);
assert.equal(scopeValueResolutionSummary.execution.mutationCount, 10);
assert.equal(scopeValueResolutionSummary.execution.passPlan.violatedDagEdgeCount, 0);

const customPropertyReachabilityResult = spawnSync(
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
      stylePath: "custom-property-reachability.css",
      styleSource:
        ":root { --used: var(--dep); --dep: red; --ghost: blue; } .btn { color: var(--used); outline: var(--broken; } .dead { --used: var(--ghost); color: var(--ghost); } .broken { color: var(--broken; }",
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(customPropertyReachabilityResult.status, 0, customPropertyReachabilityResult.stderr);
assert.equal(customPropertyReachabilityResult.error, undefined);

const customPropertyReachabilitySummary = JSON.parse(
  customPropertyReachabilityResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(customPropertyReachabilitySummary.product, "omena-query.transform-execute");
assert.ok(customPropertyReachabilitySummary.execution.outputCss.includes("--used: var(--dep);"));
assert.ok(customPropertyReachabilitySummary.execution.outputCss.includes("--dep: red;"));
assert.ok(customPropertyReachabilitySummary.execution.outputCss.includes("outline: var(--broken;"));
assert.ok(customPropertyReachabilitySummary.execution.outputCss.includes("color: var(--broken;"));
assert.ok(!customPropertyReachabilitySummary.execution.outputCss.includes("--ghost: blue;"));
assert.ok(!customPropertyReachabilitySummary.execution.outputCss.includes("--used: var(--ghost);"));
assert.ok(customPropertyReachabilitySummary.execution.outputCss.includes("color: var(--ghost);"));
assert.deepEqual(customPropertyReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyReachabilitySummary.execution.mutationCount, 2);

const customPropertyKeyframeReachabilityResult = spawnSync(
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
      stylePath: "custom-property-keyframe-reachability.css",
      styleSource:
        ":root { --used: red; --ghost: blue; } .btn { animation: live 1s; } @keyframes live { to { color: var(--used); } } @keyframes ghost { to { --used: var(--ghost); color: var(--ghost); } }",
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  customPropertyKeyframeReachabilityResult.status,
  0,
  customPropertyKeyframeReachabilityResult.stderr,
);
assert.equal(customPropertyKeyframeReachabilityResult.error, undefined);

const customPropertyKeyframeReachabilitySummary = JSON.parse(
  customPropertyKeyframeReachabilityResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(customPropertyKeyframeReachabilitySummary.product, "omena-query.transform-execute");
assert.ok(customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("--used: red;"));
assert.ok(
  customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("@keyframes ghost"),
);
assert.ok(
  customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("color: var(--ghost);"),
);
assert.ok(
  !customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("--ghost: blue;"),
);
assert.ok(
  !customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("--used: var(--ghost);"),
);
assert.deepEqual(customPropertyKeyframeReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyKeyframeReachabilitySummary.execution.mutationCount, 2);

const customPropertyContainerStyleReachabilityResult = spawnSync(
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
      stylePath: "custom-property-container-style-reachability.css",
      styleSource:
        '@property --theme { syntax: "<custom-ident>"; inherits: true; initial-value: light; } @property --dead { syntax: "<custom-ident>"; inherits: true; initial-value: off; } :root { --theme: dark; --dead: off; } @container card style(--theme: dark) { .btn { color: white; } } @container card style(--dead: off) { .dead { color: black; } }',
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  customPropertyContainerStyleReachabilityResult.status,
  0,
  customPropertyContainerStyleReachabilityResult.stderr,
);
assert.equal(customPropertyContainerStyleReachabilityResult.error, undefined);

const customPropertyContainerStyleReachabilitySummary = JSON.parse(
  customPropertyContainerStyleReachabilityResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(
  customPropertyContainerStyleReachabilitySummary.product,
  "omena-query.transform-execute",
);
assert.ok(
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes("@property --theme"),
);
assert.ok(
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes("--theme: dark;"),
);
assert.ok(
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    "@container card style(--theme: dark)",
  ),
);
assert.ok(
  !customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes("@property --dead"),
);
assert.ok(
  !customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    ":root { --theme: dark; --dead: off;",
  ),
);
assert.deepEqual(customPropertyContainerStyleReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyContainerStyleReachabilitySummary.execution.mutationCount, 2);

const customPropertyRegistrationDependencyResult = spawnSync(
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
      stylePath: "custom-property-registration-dependency.css",
      styleSource:
        '@property --used { syntax: "<color>"; inherits: false; initial-value: var(--registered-dep); } @property --dead { syntax: "<color>"; inherits: false; initial-value: var(--dead-dep); } :root { --registered-dep: red; --dead-dep: blue; --ghost: orange; } .btn { color: var(--used); }',
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  customPropertyRegistrationDependencyResult.status,
  0,
  customPropertyRegistrationDependencyResult.stderr,
);
assert.equal(customPropertyRegistrationDependencyResult.error, undefined);

const customPropertyRegistrationDependencySummary = JSON.parse(
  customPropertyRegistrationDependencyResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(customPropertyRegistrationDependencySummary.product, "omena-query.transform-execute");
assert.ok(
  customPropertyRegistrationDependencySummary.execution.outputCss.includes("@property --used"),
);
assert.ok(
  customPropertyRegistrationDependencySummary.execution.outputCss.includes(
    "--registered-dep: red;",
  ),
);
assert.ok(
  !customPropertyRegistrationDependencySummary.execution.outputCss.includes("@property --dead"),
);
assert.ok(
  !customPropertyRegistrationDependencySummary.execution.outputCss.includes("--dead-dep: blue;"),
);
assert.ok(
  !customPropertyRegistrationDependencySummary.execution.outputCss.includes("--ghost: orange;"),
);
assert.deepEqual(customPropertyRegistrationDependencySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyRegistrationDependencySummary.execution.mutationCount, 3);

const customPropertyDescriptorReachabilityResult = spawnSync(
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
      stylePath: "custom-property-descriptor-root.css",
      styleSource:
        ':root { --font-src: url(omena.woff2); --dead: blue; } @font-face { font-family: "Omena"; src: var(--font-src); } .btn { color: red; }',
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["btn"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  customPropertyDescriptorReachabilityResult.status,
  0,
  customPropertyDescriptorReachabilityResult.stderr,
);
assert.equal(customPropertyDescriptorReachabilityResult.error, undefined);

const customPropertyDescriptorReachabilitySummary = JSON.parse(
  customPropertyDescriptorReachabilityResult.stdout,
) as TransformExecuteSummaryV0;

assert.ok(
  customPropertyDescriptorReachabilitySummary.execution.outputCss.includes(
    "--font-src: url(omena.woff2);",
  ),
);
assert.ok(
  customPropertyDescriptorReachabilitySummary.execution.outputCss.includes(
    '@font-face { font-family: "Omena"; src: var(--font-src); }',
  ),
);
assert.ok(
  !customPropertyDescriptorReachabilitySummary.execution.outputCss.includes("--dead: blue;"),
);
assertIncludesAll(
  customPropertyDescriptorReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.symbolKind}:${removal.name}`,
  ),
  ["customProperty:--dead"],
  "descriptor at-rule custom property reachability removals",
);
assert.deepEqual(customPropertyDescriptorReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyDescriptorReachabilitySummary.execution.mutationCount, 1);

const customPropertyIcssExportReachabilityResult = spawnSync(
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
      stylePath: "custom-property-icss-export.module.css",
      styleSource:
        ":root { --brand: red; --dead: blue; } :export { brand: var(--brand); dead: var(--dead); }",
      requestedPassIds: ["tree-shake-custom-property", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableCustomPropertyNames: ["--brand"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  customPropertyIcssExportReachabilityResult.status,
  0,
  customPropertyIcssExportReachabilityResult.stderr,
);
assert.equal(customPropertyIcssExportReachabilityResult.error, undefined);

const customPropertyIcssExportReachabilitySummary = JSON.parse(
  customPropertyIcssExportReachabilityResult.stdout,
) as TransformExecuteSummaryV0;

assert.ok(
  customPropertyIcssExportReachabilitySummary.execution.outputCss.includes("--brand: red;"),
);
assert.ok(
  customPropertyIcssExportReachabilitySummary.execution.outputCss.includes("brand: var(--brand);"),
);
assert.ok(
  !customPropertyIcssExportReachabilitySummary.execution.outputCss.includes("--dead: blue;"),
);
assert.ok(
  !customPropertyIcssExportReachabilitySummary.execution.outputCss.includes("dead: var(--dead);"),
);
assert.deepEqual(customPropertyIcssExportReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyIcssExportReachabilitySummary.execution.mutationCount, 2);

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
            '@value used from "./tokens.module.css"; @value deadValue from "./tokens.module.css"; @value deadBp from "./tokens.module.css"; @value localValue: used; @property --brand { syntax: "<color>"; inherits: false; initial-value: red; } @property --dead { syntax: "<color>"; inherits: false; initial-value: blue; } .button { composes: base utility; color: red; border-color: localValue; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } :global { .global-reset { animation: ghost 1s; color: deadValue; outline-color: var(--dead); } } .dead { color: black; background: deadValue; } .dead :global(.external) { color: deadValue; } @media (min-width: deadBp) { .dead { color: deadValue; } } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }',
        },
      ],
      requestedPassIds: [
        "tree-shake-class",
        "tree-shake-keyframes",
        "tree-shake-value",
        "tree-shake-custom-property",
        "empty-rule-removal",
      ],
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
  [
    "tree-shake-class",
    "tree-shake-keyframes",
    "tree-shake-value",
    "tree-shake-custom-property",
    "empty-rule-removal",
  ],
  "semantic reachability executed passes",
);
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".button"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".base"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".utility"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes(".global-reset"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@keyframes spin"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@keyframes ghost"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@value used from"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@value deadValue from"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@value localValue: used;"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@property --brand"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("@property --dead"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("--brand: red"));
assert.ok(semanticReachabilitySummary.execution.outputCss.includes("--dead: blue"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes(".dead"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes(".dead :global"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes("@value deadBp from"));
assert.ok(!semanticReachabilitySummary.execution.outputCss.includes("@media (min-width: deadBp)"));
const semanticRemovalPairs = semanticReachabilitySummary.execution.semanticRemovals.map(
  (removal) => `${removal.passId}:${removal.name}`,
);
assertIncludesAll(
  semanticRemovalPairs,
  ["tree-shake-class:dead", "tree-shake-value:deadBp"],
  "semantic reachability removals",
);
assert.equal(semanticReachabilitySummary.semanticRemovalCount, 4);
assertIncludesAll(
  semanticReachabilitySummary.readySurfaces,
  ["consumerBuildFacade", "multiSourceTransformContextProducer"],
  "semantic reachability build ready surfaces",
);

const valueKeyframeReachabilityResult = spawnSync(
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
            "@value used: red; @value dead: blue; @value ghost: green; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: ghost; } } .button { animation: pulse 1s; }",
        },
      ],
      requestedPassIds: ["tree-shake-keyframes", "tree-shake-value", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(valueKeyframeReachabilityResult.status, 0, valueKeyframeReachabilityResult.stderr);
assert.equal(valueKeyframeReachabilityResult.error, undefined);

const valueKeyframeReachabilitySummary = JSON.parse(
  valueKeyframeReachabilityResult.stdout,
) as ConsumerBuildSummaryV0;

assert.ok(valueKeyframeReachabilitySummary.execution.outputCss.includes("@value used: red;"));
assert.ok(valueKeyframeReachabilitySummary.execution.outputCss.includes("color: used;"));
assert.ok(!valueKeyframeReachabilitySummary.execution.outputCss.includes("@value dead:"));
assert.ok(!valueKeyframeReachabilitySummary.execution.outputCss.includes("@value ghost:"));
assert.ok(!valueKeyframeReachabilitySummary.execution.outputCss.includes("@keyframes ghost"));
assertIncludesAll(
  valueKeyframeReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.passId}:${removal.symbolKind}:${removal.name}`,
  ),
  [
    "tree-shake-keyframes:keyframes:ghost",
    "tree-shake-value:cssModuleValue:dead",
    "tree-shake-value:cssModuleValue:ghost",
  ],
  "value keyframe reachability removals",
);

const icssExportReachabilityResult = spawnSync(
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
      targetStylePath: "Tokens.module.css",
      styles: [
        {
          stylePath: "Tokens.module.css",
          styleSource:
            "@value primary: red; @value shadow: 0 0 primary; @value dead: blue; @value deadExpr: calc(1rem + 2px); :export { public-color: shadow; dead-public: dead; } .button { color: red; }",
        },
      ],
      requestedPassIds: ["tree-shake-value", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
        reachableValueNames: ["public-color"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(icssExportReachabilityResult.status, 0, icssExportReachabilityResult.stderr);
assert.equal(icssExportReachabilityResult.error, undefined);

const icssExportReachabilitySummary = JSON.parse(
  icssExportReachabilityResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(icssExportReachabilitySummary.product, "omena-query.consumer-build-style-source");
assert.deepEqual(icssExportReachabilitySummary.unknownPassIds, []);
assert.equal(icssExportReachabilitySummary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(icssExportReachabilitySummary.execution.passPlan.allRequestedRegistered, true);
assert.ok(icssExportReachabilitySummary.execution.outputCss.includes("@value primary: red;"));
assert.ok(
  icssExportReachabilitySummary.execution.outputCss.includes("@value shadow: 0 0 primary;"),
);
assert.ok(
  icssExportReachabilitySummary.execution.outputCss.includes(":export { public-color: shadow;"),
);
assert.ok(!icssExportReachabilitySummary.execution.outputCss.includes("@value dead:"));
assert.ok(!icssExportReachabilitySummary.execution.outputCss.includes("@value deadExpr:"));
assert.ok(!icssExportReachabilitySummary.execution.outputCss.includes("dead-public: dead"));
assertIncludesAll(
  icssExportReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.symbolKind}:${removal.name}`,
  ),
  ["cssModuleValue:dead", "cssModuleValue:deadExpr", "cssModuleIcssExport:dead-public"],
  "ICSS export reachability removals",
);

const valueAtRulePreludeReachabilityResult = spawnSync(
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
      targetStylePath: "PreludeValues.module.css",
      styles: [
        {
          stylePath: "PreludeValues.module.css",
          styleSource:
            "@value screen: 1px; @value bp: 40rem; @value theme: dark; @media screen and (min-width: bp) { .button { color: red; } } @container card style(--mode: theme) { .button { color: blue; } }",
        },
      ],
      requestedPassIds: ["tree-shake-value", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  valueAtRulePreludeReachabilityResult.status,
  0,
  valueAtRulePreludeReachabilityResult.stderr,
);
assert.equal(valueAtRulePreludeReachabilityResult.error, undefined);

const valueAtRulePreludeReachabilitySummary = JSON.parse(
  valueAtRulePreludeReachabilityResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(
  valueAtRulePreludeReachabilitySummary.product,
  "omena-query.consumer-build-style-source",
);
assert.ok(!valueAtRulePreludeReachabilitySummary.execution.outputCss.includes("@value screen:"));
assert.ok(valueAtRulePreludeReachabilitySummary.execution.outputCss.includes("@value bp: 40rem;"));
assert.ok(
  valueAtRulePreludeReachabilitySummary.execution.outputCss.includes("@value theme: dark;"),
);
assert.ok(
  valueAtRulePreludeReachabilitySummary.execution.outputCss.includes(
    "@media screen and (min-width: bp)",
  ),
);
assert.ok(
  valueAtRulePreludeReachabilitySummary.execution.outputCss.includes(
    "@container card style(--mode: theme)",
  ),
);
assertIncludesAll(
  valueAtRulePreludeReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.symbolKind}:${removal.name}`,
  ),
  ["cssModuleValue:screen"],
  "at-rule prelude value-position reachability removals",
);
assert.equal(valueAtRulePreludeReachabilitySummary.execution.mutationCount, 1);

const descriptorValueReachabilityResult = spawnSync(
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
      targetStylePath: "DescriptorValues.module.css",
      styles: [
        {
          stylePath: "DescriptorValues.module.css",
          styleSource:
            "@value face: OmenaSans; @value weight: 700; @value dead: blue; @font-face { font-family: face; font-weight: weight; src: url(omena.woff2); } .button { color: red; }",
        },
      ],
      requestedPassIds: ["tree-shake-value", "print-css"],
      transformContext: {
        closedStyleWorld: true,
        reachableClassNames: ["button"],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(descriptorValueReachabilityResult.status, 0, descriptorValueReachabilityResult.stderr);
assert.equal(descriptorValueReachabilityResult.error, undefined);

const descriptorValueReachabilitySummary = JSON.parse(
  descriptorValueReachabilityResult.stdout,
) as ConsumerBuildSummaryV0;

assert.ok(
  descriptorValueReachabilitySummary.execution.outputCss.includes("@value face: OmenaSans;"),
);
assert.ok(descriptorValueReachabilitySummary.execution.outputCss.includes("@value weight: 700;"));
assert.ok(
  descriptorValueReachabilitySummary.execution.outputCss.includes(
    "@font-face { font-family: face; font-weight: weight;",
  ),
);
assert.ok(!descriptorValueReachabilitySummary.execution.outputCss.includes("@value dead:"));
assertIncludesAll(
  descriptorValueReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.symbolKind}:${removal.name}`,
  ),
  ["cssModuleValue:dead"],
  "descriptor at-rule value reachability removals",
);
assert.equal(descriptorValueReachabilitySummary.execution.mutationCount, 1);

const staticScssEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.scss",
      styleSource: "$brand: red; $accent: $brand; .button { color: $accent; }",
      requestedPassIds: ["scss-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticScssEvaluationResult.status, 0, staticScssEvaluationResult.stderr);
assert.equal(staticScssEvaluationResult.error, undefined);

const staticScssEvaluationSummary = JSON.parse(
  staticScssEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(staticScssEvaluationSummary.product, "omena-query.consumer-build-style-source");
assert.deepEqual(staticScssEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(staticScssEvaluationSummary.execution.executedPassIds, [
  "scss-module-evaluate",
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(
  staticScssEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-scss-variable-evaluator",
);
assert.equal(
  staticScssEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss,
  "  .button { color: red; }",
);
assert.equal(staticScssEvaluationSummary.execution.outputCss, "  ._button_0{ color: red; }");

const importAwareScssEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@import "./tokens.scss"; .button { color: $brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(importAwareScssEvaluationResult.status, 0, importAwareScssEvaluationResult.stderr);
assert.equal(importAwareScssEvaluationResult.error, undefined);

const importAwareScssEvaluationSummary = JSON.parse(
  importAwareScssEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(importAwareScssEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(importAwareScssEvaluationSummary.execution.executedPassIds, [
  "import-inline",
  "scss-module-evaluate",
  "print-css",
]);
assert(importAwareScssEvaluationSummary.execution.outputCss.includes(".base { color: blue; }"));
assert(importAwareScssEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!importAwareScssEvaluationSummary.execution.outputCss.includes("@import"));
assert(!importAwareScssEvaluationSummary.execution.outputCss.includes("$brand:"));

const scssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; $gap: 8px; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./tokens" as tokens; .button { color: tokens.$brand; margin: tokens.$gap; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(scssUseEvaluationResult.status, 0, scssUseEvaluationResult.stderr);
assert.equal(scssUseEvaluationResult.error, undefined);

const scssUseEvaluationSummary = JSON.parse(
  scssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(scssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(scssUseEvaluationSummary.execution.executedPassIds, [
  "scss-module-evaluate",
  "print-css",
]);
assert(scssUseEvaluationSummary.execution.outputCss.includes(".base { color: blue; }"));
assert(
  scssUseEvaluationSummary.execution.outputCss.includes(".button { color: red; margin: 8px; }"),
);
assert(!scssUseEvaluationSummary.execution.outputCss.includes("@use"));
assert(!scssUseEvaluationSummary.execution.outputCss.includes("tokens.$"));
assert(!scssUseEvaluationSummary.execution.outputCss.includes("$brand:"));

const hyphenatedScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand-color: red; .base { color: $brand_color; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./tokens" as tokens; .button { color: tokens.$brand_color; border-color: tokens.$brand-color; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(hyphenatedScssUseEvaluationResult.status, 0, hyphenatedScssUseEvaluationResult.stderr);
assert.equal(hyphenatedScssUseEvaluationResult.error, undefined);

const hyphenatedScssUseEvaluationSummary = JSON.parse(
  hyphenatedScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(hyphenatedScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(hyphenatedScssUseEvaluationSummary.execution.outputCss.includes(".base { color: red; }"));
assert(
  hyphenatedScssUseEvaluationSummary.execution.outputCss.includes(
    ".button { color: red; border-color: red; }",
  ),
);
assert(!hyphenatedScssUseEvaluationSummary.execution.outputCss.includes("tokens.$"));
assert(!hyphenatedScssUseEvaluationSummary.execution.outputCss.includes("$brand_color"));

const wildcardScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; $gap: 8px; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@use "./tokens" as *; .button { color: $brand; margin: $gap; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(wildcardScssUseEvaluationResult.status, 0, wildcardScssUseEvaluationResult.stderr);
assert.equal(wildcardScssUseEvaluationResult.error, undefined);

const wildcardScssUseEvaluationSummary = JSON.parse(
  wildcardScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(wildcardScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(wildcardScssUseEvaluationSummary.execution.outputCss.includes(".base { color: blue; }"));
assert(
  wildcardScssUseEvaluationSummary.execution.outputCss.includes(
    ".button { color: red; margin: 8px; }",
  ),
);
assert(!wildcardScssUseEvaluationSummary.execution.outputCss.includes("@use"));
assert(!wildcardScssUseEvaluationSummary.execution.outputCss.includes("$brand:"));

const mixedNamespaceScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/other.scss",
          styleSource: "$brand: blue;",
        },
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red;",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./other" as *; @use "./tokens" as tokens; .button { color: tokens.$brand; border-color: $brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  mixedNamespaceScssUseEvaluationResult.status,
  0,
  mixedNamespaceScssUseEvaluationResult.stderr,
);
assert.equal(mixedNamespaceScssUseEvaluationResult.error, undefined);

const mixedNamespaceScssUseEvaluationSummary = JSON.parse(
  mixedNamespaceScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(mixedNamespaceScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  mixedNamespaceScssUseEvaluationSummary.execution.outputCss.includes(
    ".button { color: red; border-color: blue; }",
  ),
);
assert(!mixedNamespaceScssUseEvaluationSummary.execution.outputCss.includes("tokens.blue"));
assert(!mixedNamespaceScssUseEvaluationSummary.execution.outputCss.includes("tokens.$"));

const duplicateScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./tokens" as a; @use "./tokens" as b; .button { color: a.$brand; border-color: b.$brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(duplicateScssUseEvaluationResult.status, 0, duplicateScssUseEvaluationResult.stderr);
assert.equal(duplicateScssUseEvaluationResult.error, undefined);

const duplicateScssUseEvaluationSummary = JSON.parse(
  duplicateScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(duplicateScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  countOccurrences(duplicateScssUseEvaluationSummary.execution.outputCss, ".base { color: red; }"),
  1,
);
assert(
  duplicateScssUseEvaluationSummary.execution.outputCss.includes(
    ".button { color: red; border-color: red; }",
  ),
);
assert(!duplicateScssUseEvaluationSummary.execution.outputCss.includes("@use"));
assert(!duplicateScssUseEvaluationSummary.execution.outputCss.includes("a.$"));
assert(!duplicateScssUseEvaluationSummary.execution.outputCss.includes("b.$"));

const forwardedScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; $gap: 8px; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/theme.scss",
          styleSource: '@forward "./tokens" show $brand;',
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@use "./theme" as theme; .button { color: theme.$brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(forwardedScssUseEvaluationResult.status, 0, forwardedScssUseEvaluationResult.stderr);
assert.equal(forwardedScssUseEvaluationResult.error, undefined);

const forwardedScssUseEvaluationSummary = JSON.parse(
  forwardedScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(forwardedScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(forwardedScssUseEvaluationSummary.execution.outputCss.includes(".base { color: blue; }"));
assert(forwardedScssUseEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!forwardedScssUseEvaluationSummary.execution.outputCss.includes("@forward"));
assert(!forwardedScssUseEvaluationSummary.execution.outputCss.includes("theme.$"));

const duplicateScssForwardEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/theme.scss",
          styleSource: '@forward "./tokens"; @forward "./tokens";',
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@use "./theme" as theme; .button { color: theme.$brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  duplicateScssForwardEvaluationResult.status,
  0,
  duplicateScssForwardEvaluationResult.stderr,
);
assert.equal(duplicateScssForwardEvaluationResult.error, undefined);

const duplicateScssForwardEvaluationSummary = JSON.parse(
  duplicateScssForwardEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(duplicateScssForwardEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  countOccurrences(
    duplicateScssForwardEvaluationSummary.execution.outputCss,
    ".base { color: red; }",
  ),
  1,
);
assert(
  duplicateScssForwardEvaluationSummary.execution.outputCss.includes(".button { color: red; }"),
);
assert(!duplicateScssForwardEvaluationSummary.execution.outputCss.includes("@forward"));
assert(!duplicateScssForwardEvaluationSummary.execution.outputCss.includes("theme.$"));

const configuredScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: blue !default; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./tokens" as tokens with ($brand: red); .button { color: tokens.$brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(configuredScssUseEvaluationResult.status, 0, configuredScssUseEvaluationResult.stderr);
assert.equal(configuredScssUseEvaluationResult.error, undefined);

const configuredScssUseEvaluationSummary = JSON.parse(
  configuredScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(configuredScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(configuredScssUseEvaluationSummary.execution.outputCss.includes(".base { color: red; }"));
assert(configuredScssUseEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!configuredScssUseEvaluationSummary.execution.outputCss.includes("@use"));
assert(!configuredScssUseEvaluationSummary.execution.outputCss.includes("tokens.$"));
assert(!configuredScssUseEvaluationSummary.execution.outputCss.includes("$brand:"));

const configuredHyphenatedScssUseEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand-color: blue !default; .base { color: $brand-color; }",
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./tokens" as tokens with ($brand_color: red); .button { color: tokens.$brand-color; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  configuredHyphenatedScssUseEvaluationResult.status,
  0,
  configuredHyphenatedScssUseEvaluationResult.stderr,
);
assert.equal(configuredHyphenatedScssUseEvaluationResult.error, undefined);

const configuredHyphenatedScssUseEvaluationSummary = JSON.parse(
  configuredHyphenatedScssUseEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(configuredHyphenatedScssUseEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  configuredHyphenatedScssUseEvaluationSummary.execution.outputCss.includes(
    ".base { color: red; }",
  ),
);
assert(
  configuredHyphenatedScssUseEvaluationSummary.execution.outputCss.includes(
    ".button { color: red; }",
  ),
);
assert(!configuredHyphenatedScssUseEvaluationSummary.execution.outputCss.includes("blue"));
assert(!configuredHyphenatedScssUseEvaluationSummary.execution.outputCss.includes("tokens.$"));

const configuredScssForwardEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: blue !default; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/theme.scss",
          styleSource: '@forward "./tokens" with ($brand: red);',
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@use "./theme" as theme; .button { color: theme.$brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  configuredScssForwardEvaluationResult.status,
  0,
  configuredScssForwardEvaluationResult.stderr,
);
assert.equal(configuredScssForwardEvaluationResult.error, undefined);

const configuredScssForwardEvaluationSummary = JSON.parse(
  configuredScssForwardEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(configuredScssForwardEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  configuredScssForwardEvaluationSummary.execution.outputCss.includes(".base { color: red; }"),
);
assert(
  configuredScssForwardEvaluationSummary.execution.outputCss.includes(".button { color: red; }"),
);
assert(!configuredScssForwardEvaluationSummary.execution.outputCss.includes("@forward"));
assert(!configuredScssForwardEvaluationSummary.execution.outputCss.includes("theme.$"));
assert(!configuredScssForwardEvaluationSummary.execution.outputCss.includes("red);"));

const prefixedScssForwardEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; $gap: 8px; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/theme.scss",
          styleSource: '@forward "./tokens" as token-* show $token-brand;',
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource: '@use "./theme" as theme; .button { color: theme.$token-brand; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  prefixedScssForwardEvaluationResult.status,
  0,
  prefixedScssForwardEvaluationResult.stderr,
);
assert.equal(prefixedScssForwardEvaluationResult.error, undefined);

const prefixedScssForwardEvaluationSummary = JSON.parse(
  prefixedScssForwardEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(prefixedScssForwardEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(prefixedScssForwardEvaluationSummary.execution.outputCss.includes(".base { color: red; }"));
assert(
  prefixedScssForwardEvaluationSummary.execution.outputCss.includes(".button { color: red; }"),
);
assert(!prefixedScssForwardEvaluationSummary.execution.outputCss.includes("@forward"));
assert(!prefixedScssForwardEvaluationSummary.execution.outputCss.includes("theme.$"));

const prefixedScssForwardHideEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.scss",
      styles: [
        {
          stylePath: "/tmp/tokens.scss",
          styleSource: "$brand: red; $gap: 8px; .base { color: $brand; }",
        },
        {
          stylePath: "/tmp/theme.scss",
          styleSource: '@forward "./tokens" as token-* hide $token-gap;',
        },
        {
          stylePath: "/tmp/App.module.scss",
          styleSource:
            '@use "./theme" as theme; .button { color: theme.$token-brand; margin: theme.$token-gap; }',
        },
      ],
      requestedPassIds: ["scss-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  prefixedScssForwardHideEvaluationResult.status,
  0,
  prefixedScssForwardHideEvaluationResult.stderr,
);
assert.equal(prefixedScssForwardHideEvaluationResult.error, undefined);

const prefixedScssForwardHideEvaluationSummary = JSON.parse(
  prefixedScssForwardHideEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(prefixedScssForwardHideEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  prefixedScssForwardHideEvaluationSummary.execution.outputCss.includes(".button { color: red;"),
);
assert(
  prefixedScssForwardHideEvaluationSummary.execution.outputCss.includes(
    "margin: theme.$token-gap;",
  ),
);
assert(!prefixedScssForwardHideEvaluationSummary.execution.outputCss.includes("@forward"));

const staticScssDefaultEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "DefaultScss.module.scss",
      styleSource: "$brand: red !default; $brand: blue; .button { color: $brand; }",
      requestedPassIds: ["scss-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticScssDefaultEvaluationResult.status, 0, staticScssDefaultEvaluationResult.stderr);
assert.equal(staticScssDefaultEvaluationResult.error, undefined);

const staticScssDefaultEvaluationSummary = JSON.parse(
  staticScssDefaultEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(staticScssDefaultEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticScssDefaultEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss.trim(),
  ".button { color: blue; }",
);
assert.equal(
  staticScssDefaultEvaluationSummary.execution.outputCss.trim(),
  "._button_0{ color: blue; }",
);

const staticScssScopedEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "ScopedScss.module.scss",
      styleSource: "$brand: blue; .card { $brand: red; color: $brand; } .other { color: $brand; }",
      requestedPassIds: ["scss-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticScssScopedEvaluationResult.status, 0, staticScssScopedEvaluationResult.stderr);
assert.equal(staticScssScopedEvaluationResult.error, undefined);

const staticScssScopedEvaluationSummary = JSON.parse(
  staticScssScopedEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(staticScssScopedEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticScssScopedEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-scss-variable-evaluator",
);
assert(staticScssScopedEvaluationSummary.execution.outputCss.includes("color: red"));
assert(staticScssScopedEvaluationSummary.execution.outputCss.includes("color: blue"));
assert(!staticScssScopedEvaluationSummary.execution.outputCss.includes("$brand:"));

const staticScssGlobalEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "GlobalScss.module.scss",
      styleSource:
        "$brand: blue; .card { $brand: red !global; color: $brand; } .other { color: $brand; }",
      requestedPassIds: ["scss-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticScssGlobalEvaluationResult.status, 0, staticScssGlobalEvaluationResult.stderr);
assert.equal(staticScssGlobalEvaluationResult.error, undefined);

const staticScssGlobalEvaluationSummary = JSON.parse(
  staticScssGlobalEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(staticScssGlobalEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(staticScssGlobalEvaluationSummary.execution.outputCss.includes("color: red"));
assert(!staticScssGlobalEvaluationSummary.execution.outputCss.includes("color: blue"));
assert(!staticScssGlobalEvaluationSummary.execution.outputCss.includes("$brand:"));

const staticLessEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.less",
      styleSource: "@brand: red; @accent: @brand; .button { color: @accent; }",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticLessEvaluationResult.status, 0, staticLessEvaluationResult.stderr);
assert.equal(staticLessEvaluationResult.error, undefined);

const staticLessEvaluationSummary = JSON.parse(
  staticLessEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(staticLessEvaluationSummary.product, "omena-query.consumer-build-style-source");
assert.deepEqual(staticLessEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(staticLessEvaluationSummary.execution.executedPassIds, [
  "less-module-evaluate",
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(
  staticLessEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-less-variable-evaluator",
);
assert.equal(
  staticLessEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss,
  "  .button { color: red; }",
);
assert.equal(staticLessEvaluationSummary.execution.outputCss, "  ._button_0{ color: red; }");

const importAwareLessEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import "./tokens.less"; .button { color: @brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(importAwareLessEvaluationResult.status, 0, importAwareLessEvaluationResult.stderr);
assert.equal(importAwareLessEvaluationResult.error, undefined);

const importAwareLessEvaluationSummary = JSON.parse(
  importAwareLessEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(importAwareLessEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(importAwareLessEvaluationSummary.execution.executedPassIds, [
  "import-inline",
  "less-module-evaluate",
  "print-css",
]);
assert(importAwareLessEvaluationSummary.execution.outputCss.includes(".base { color: blue; }"));
assert(importAwareLessEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!importAwareLessEvaluationSummary.execution.outputCss.includes("@import"));
assert(!importAwareLessEvaluationSummary.execution.outputCss.includes("@brand:"));

const duplicateLessImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: @brand; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource:
            '@import "./tokens.less"; @import (once) "./tokens.less"; .button { color: @brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  duplicateLessImportEvaluationResult.status,
  0,
  duplicateLessImportEvaluationResult.stderr,
);
assert.equal(duplicateLessImportEvaluationResult.error, undefined);

const duplicateLessImportEvaluationSummary = JSON.parse(
  duplicateLessImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(duplicateLessImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  countOccurrences(
    duplicateLessImportEvaluationSummary.execution.outputCss,
    ".base { color: red; }",
  ),
  1,
);
assert(
  duplicateLessImportEvaluationSummary.execution.outputCss.includes(".button { color: red; }"),
);
assert(!duplicateLessImportEvaluationSummary.execution.outputCss.includes("@import"));
assert(!duplicateLessImportEvaluationSummary.execution.outputCss.includes("@brand:"));

const multipleLessImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: @brand; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource:
            '@import "./tokens.less"; @import (multiple) "./tokens.less"; .button { color: @brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  multipleLessImportEvaluationResult.status,
  0,
  multipleLessImportEvaluationResult.stderr,
);
assert.equal(multipleLessImportEvaluationResult.error, undefined);

const multipleLessImportEvaluationSummary = JSON.parse(
  multipleLessImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(multipleLessImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  countOccurrences(
    multipleLessImportEvaluationSummary.execution.outputCss,
    ".base { color: red; }",
  ),
  2,
);
assert(multipleLessImportEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!multipleLessImportEvaluationSummary.execution.outputCss.includes("@import"));
assert(!multipleLessImportEvaluationSummary.execution.outputCss.includes("@brand:"));

const optionalLessImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import (optional) "./missing.less"; .button { color: red; }',
        },
      ],
      requestedPassIds: ["import-inline", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  optionalLessImportEvaluationResult.status,
  0,
  optionalLessImportEvaluationResult.stderr,
);
assert.equal(optionalLessImportEvaluationResult.error, undefined);

const optionalLessImportEvaluationSummary = JSON.parse(
  optionalLessImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(optionalLessImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(optionalLessImportEvaluationSummary.execution.outputCss.includes(".button { color: red; }"));
assert(!optionalLessImportEvaluationSummary.execution.outputCss.includes("@import"));
assert(!optionalLessImportEvaluationSummary.execution.outputCss.includes("missing.less"));

const lessReferenceImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: blue; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import (reference) "./tokens.less"; .button { color: @brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  lessReferenceImportEvaluationResult.status,
  0,
  lessReferenceImportEvaluationResult.stderr,
);
assert.equal(lessReferenceImportEvaluationResult.error, undefined);

const lessReferenceImportEvaluationSummary = JSON.parse(
  lessReferenceImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(lessReferenceImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(lessReferenceImportEvaluationSummary.execution.executedPassIds, [
  "import-inline",
  "less-module-evaluate",
  "print-css",
]);
assert(
  lessReferenceImportEvaluationSummary.execution.outputCss.includes(".button { color: red; }"),
);
assert(!lessReferenceImportEvaluationSummary.execution.outputCss.includes(".base"));
assert(!lessReferenceImportEvaluationSummary.execution.outputCss.includes("@import"));
assert(!lessReferenceImportEvaluationSummary.execution.outputCss.includes("@brand:"));

const lessInlineLiteralImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: @brand; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import (inline) "./tokens.less"; @local: blue; .button { color: @local; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  lessInlineLiteralImportEvaluationResult.status,
  0,
  lessInlineLiteralImportEvaluationResult.stderr,
);
assert.equal(lessInlineLiteralImportEvaluationResult.error, undefined);

const lessInlineLiteralImportEvaluationSummary = JSON.parse(
  lessInlineLiteralImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(lessInlineLiteralImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  lessInlineLiteralImportEvaluationSummary.execution.outputCss.includes(
    "@brand: red; .base { color: @brand; }",
  ),
);
assert(
  lessInlineLiteralImportEvaluationSummary.execution.outputCss.includes(".button { color: blue; }"),
);
assert(!lessInlineLiteralImportEvaluationSummary.execution.outputCss.includes("@local:"));
assert(!lessInlineLiteralImportEvaluationSummary.execution.outputCss.includes("@import"));

const lessCssPassthroughImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.less",
          styleSource: "@brand: red; .base { color: @brand; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import (css) "./tokens.less" screen; .button { color: blue; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  lessCssPassthroughImportEvaluationResult.status,
  0,
  lessCssPassthroughImportEvaluationResult.stderr,
);
assert.equal(lessCssPassthroughImportEvaluationResult.error, undefined);

const lessCssPassthroughImportEvaluationSummary = JSON.parse(
  lessCssPassthroughImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(lessCssPassthroughImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  lessCssPassthroughImportEvaluationSummary.execution.outputCss.includes(
    '@import "./tokens.less" screen;',
  ),
);
assert(
  lessCssPassthroughImportEvaluationSummary.execution.outputCss.includes(
    ".button { color: blue; }",
  ),
);
assert(!lessCssPassthroughImportEvaluationSummary.execution.outputCss.includes(".base"));
assert(!lessCssPassthroughImportEvaluationSummary.execution.outputCss.includes("@brand:"));

const lessForcedCssImportEvaluationResult = spawnSync(
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
      targetStylePath: "/tmp/App.module.less",
      styles: [
        {
          stylePath: "/tmp/tokens.css",
          styleSource: "@brand: green; .from-css { color: @brand; }",
        },
        {
          stylePath: "/tmp/App.module.less",
          styleSource: '@import (less) "./tokens.css"; .button { color: @brand; }',
        },
      ],
      requestedPassIds: ["import-inline", "less-module-evaluate", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  lessForcedCssImportEvaluationResult.status,
  0,
  lessForcedCssImportEvaluationResult.stderr,
);
assert.equal(lessForcedCssImportEvaluationResult.error, undefined);

const lessForcedCssImportEvaluationSummary = JSON.parse(
  lessForcedCssImportEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(lessForcedCssImportEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(
  lessForcedCssImportEvaluationSummary.execution.outputCss.includes(".from-css { color: green; }"),
);
assert(
  lessForcedCssImportEvaluationSummary.execution.outputCss.includes(".button { color: green; }"),
);
assert(!lessForcedCssImportEvaluationSummary.execution.outputCss.includes("@brand:"));
assert(!lessForcedCssImportEvaluationSummary.execution.outputCss.includes("@import"));

const staticLessForwardEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "ForwardLess.module.less",
      styleSource: ".button { color: @accent; } @accent: @brand; @brand: red;",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticLessForwardEvaluationResult.status, 0, staticLessForwardEvaluationResult.stderr);
assert.equal(staticLessForwardEvaluationResult.error, undefined);

const staticLessForwardEvaluationSummary = JSON.parse(
  staticLessForwardEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(staticLessForwardEvaluationSummary.product, "omena-query.consumer-build-style-source");
assert.deepEqual(staticLessForwardEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.deepEqual(staticLessForwardEvaluationSummary.execution.executedPassIds, [
  "less-module-evaluate",
  "css-modules-class-hashing",
  "print-css",
]);
assert.equal(
  staticLessForwardEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-less-variable-evaluator",
);
assert.equal(
  staticLessForwardEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss,
  ".button { color: red; }  ",
);
assert.equal(
  staticLessForwardEvaluationSummary.execution.outputCss.trim(),
  "._button_0{ color: red; }",
);

const staticLessLastWinsEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "LastWinsLess.module.less",
      styleSource: "@brand: red; .button { color: @brand; } @brand: blue;",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  staticLessLastWinsEvaluationResult.status,
  0,
  staticLessLastWinsEvaluationResult.stderr,
);
assert.equal(staticLessLastWinsEvaluationResult.error, undefined);

const staticLessLastWinsEvaluationSummary = JSON.parse(
  staticLessLastWinsEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(
  staticLessLastWinsEvaluationSummary.product,
  "omena-query.consumer-build-style-source",
);
assert.deepEqual(staticLessLastWinsEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticLessLastWinsEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss.trim(),
  ".button { color: blue; }",
);
assert.equal(
  staticLessLastWinsEvaluationSummary.execution.outputCss.trim(),
  "._button_0{ color: blue; }",
);

const staticLessScopedEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "ScopedLess.module.less",
      styleSource:
        "@tone: @brand; @brand: blue; .card { @brand: red; color: @tone; } .other { color: @tone; }",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(staticLessScopedEvaluationResult.status, 0, staticLessScopedEvaluationResult.stderr);
assert.equal(staticLessScopedEvaluationResult.error, undefined);

const staticLessScopedEvaluationSummary = JSON.parse(
  staticLessScopedEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(staticLessScopedEvaluationSummary.product, "omena-query.consumer-build-style-source");
assert.deepEqual(staticLessScopedEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticLessScopedEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-less-variable-evaluator",
);
assert(staticLessScopedEvaluationSummary.execution.outputCss.includes("color: red"));
assert(staticLessScopedEvaluationSummary.execution.outputCss.includes("color: blue"));
assert(!staticLessScopedEvaluationSummary.execution.outputCss.includes("@tone:"));
assert(!staticLessScopedEvaluationSummary.execution.outputCss.includes("@brand:"));

const staticLessPropertyEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "PropertyLess.module.less",
      styleSource:
        ".card { color: red; background: $color; color: blue; } .other { color: green; background: $color; }",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  staticLessPropertyEvaluationResult.status,
  0,
  staticLessPropertyEvaluationResult.stderr,
);
assert.equal(staticLessPropertyEvaluationResult.error, undefined);

const staticLessPropertyEvaluationSummary = JSON.parse(
  staticLessPropertyEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.equal(
  staticLessPropertyEvaluationSummary.product,
  "omena-query.consumer-build-style-source",
);
assert.deepEqual(staticLessPropertyEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticLessPropertyEvaluationSummary.execution.cssModuleEvaluation?.evaluator,
  "omena-query-static-less-variable-evaluator",
);
assert(staticLessPropertyEvaluationSummary.execution.outputCss.includes("background: blue"));
assert(staticLessPropertyEvaluationSummary.execution.outputCss.includes("background: green"));
assert(!staticLessPropertyEvaluationSummary.execution.outputCss.includes("$color"));

const staticLessArithmeticEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "ArithmeticLess.module.less",
      styleSource:
        "@width: 100px; @half: (@width / 2); @sum: (@half + 10px); .card { width: @half; margin: @sum; }",
      requestedPassIds: ["less-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  staticLessArithmeticEvaluationResult.status,
  0,
  staticLessArithmeticEvaluationResult.stderr,
);
assert.equal(staticLessArithmeticEvaluationResult.error, undefined);

const staticLessArithmeticEvaluationSummary = JSON.parse(
  staticLessArithmeticEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(staticLessArithmeticEvaluationSummary.execution.plannedOnlyPassIds, []);
assert(staticLessArithmeticEvaluationSummary.execution.outputCss.includes("width: 50px"));
assert(staticLessArithmeticEvaluationSummary.execution.outputCss.includes("margin: 60px"));
assert(!staticLessArithmeticEvaluationSummary.execution.outputCss.includes("@half:"));
assert(!staticLessArithmeticEvaluationSummary.execution.outputCss.includes("@sum:"));

const staticStylesheetCompositeEvaluationResult = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "engine-shadow-runner",
    "--",
    "consumer-build-style-source",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "CompositeScss.module.scss",
      styleSource: "$brand: red; $border: 1px solid $brand; .button { border: $border; }",
      requestedPassIds: ["scss-module-evaluate", "css-modules-class-hashing", "print-css"],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(
  staticStylesheetCompositeEvaluationResult.status,
  0,
  staticStylesheetCompositeEvaluationResult.stderr,
);
assert.equal(staticStylesheetCompositeEvaluationResult.error, undefined);

const staticStylesheetCompositeEvaluationSummary = JSON.parse(
  staticStylesheetCompositeEvaluationResult.stdout,
) as ConsumerBuildSummaryV0;

assert.deepEqual(staticStylesheetCompositeEvaluationSummary.execution.plannedOnlyPassIds, []);
assert.equal(
  staticStylesheetCompositeEvaluationSummary.execution.cssModuleEvaluation?.evaluatedCss.trim(),
  ".button { border: 1px solid red; }",
);
assert.equal(
  staticStylesheetCompositeEvaluationSummary.execution.outputCss.trim(),
  "._button_0{ border: 1px solid red; }",
);
assert.equal(icssExportReachabilitySummary.semanticRemovalCount, 3);

process.stdout.write(
  [
    "validated omena-query transform-execute runtime:",
    `executed=${summary.execution.executedPassIds.length}`,
    `mutations=${summary.execution.mutationCount}`,
    `unknown=${summary.unknownPassIds.length}`,
    `contextExecuted=${contextSummary.execution.executedPassIds.length}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
    `transitiveImportInlineMutations=${transitiveImportInlineSummary.execution.mutationCount}`,
    `designTokenAliasMutations=${designTokenAliasSummary.execution.mutationCount}`,
    `designTokenImportantMutations=${designTokenImportantSummary.execution.mutationCount}`,
    `groupedComposesMutations=${groupedComposesSummary.execution.mutationCount}`,
    `globalComposesHashMutations=${globalComposesHashSummary.execution.mutationCount}`,
    `scopedClassHashMutations=${scopedClassHashSummary.execution.mutationCount}`,
    `supportsSelectorClassHashMutations=${supportsSelectorClassHashSummary.execution.mutationCount}`,
    `alphaColorMutations=${alphaColorFunctionSummary.execution.mutationCount}`,
    `alphaOkColorMutations=${alphaOkColorSummary.execution.mutationCount}`,
    `compositeValueMutations=${compositeValueSummary.execution.mutationCount}`,
    `transitiveImportedValueMutations=${transitiveImportedValueSummary.execution.mutationCount}`,
    `mediaListMutations=${mediaListSummary.execution.mutationCount}`,
    `mediaOrMutations=${mediaOrSummary.execution.mutationCount}`,
    `conditionalWrapperMergeMutations=${conditionalWrapperMergeSummary.execution.mutationCount}`,
    `logicalCornerMutations=${logicalCornerSummary.execution.mutationCount}`,
    `verticalLogicalMutations=${verticalLogicalSummary.execution.mutationCount}`,
    `nestAtRuleMutations=${nestAtRuleSummary.execution.mutationCount}`,
    `alphaColorCompressionMutations=${alphaColorCompressionSummary.execution.mutationCount}`,
    `namedHexColorMutations=${namedHexColorCompressionSummary.execution.mutationCount}`,
    `colorMixPercentageMutations=${colorMixPercentageSummary.execution.mutationCount}`,
    `colorMixAlphaMutations=${colorMixAlphaSummary.execution.mutationCount}`,
    `colorMixLinearMutations=${colorMixLinearSummary.execution.mutationCount}`,
    `mathFunctionMutations=${mathFunctionReductionSummary.execution.mutationCount}`,
    `staticVarShadowMutations=${staticVarShadowSummary.execution.mutationCount}`,
    `staticVarPreludeMutations=${staticVarPreludeSummary.execution.mutationCount}`,
    `staticBranchResolutionMutations=${staticBranchResolutionSummary.execution.mutationCount}`,
    `scopeValueResolutionMutations=${scopeValueResolutionSummary.execution.mutationCount}`,
    `customPropertyReachabilityMutations=${customPropertyReachabilitySummary.execution.mutationCount}`,
    `customPropertyContainerStyleMutations=${customPropertyContainerStyleReachabilitySummary.execution.mutationCount}`,
    `customPropertyRegistrationDependencyMutations=${customPropertyRegistrationDependencySummary.execution.mutationCount}`,
    `icssExportReachabilityRemovals=${icssExportReachabilitySummary.semanticRemovalCount}`,
    `semanticRemovals=${semanticReachabilitySummary.semanticRemovalCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
