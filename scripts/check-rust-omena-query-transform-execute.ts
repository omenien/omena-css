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

const fontSupportsSummary = JSON.parse(
  fontSupportsResult.stdout,
) as TransformExecuteSummaryV0;

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

const mediaListSummary = JSON.parse(
  mediaListResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(mediaListSummary.product, "omena-query.transform-execute");
assert.equal(
  mediaListSummary.execution.outputCss,
  " .not-zero { color: lime; } .not-impossible { color: teal; }    .live { color: green; } @media screen, (width<=0px) { .unknown { color: orange; } }",
);
assert.deepEqual(mediaListSummary.execution.executedPassIds, [
  "media-static-eval",
  "print-css",
]);
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
assert.deepEqual(mediaOrSummary.execution.executedPassIds, [
  "media-static-eval",
  "print-css",
]);
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

const logicalCornerSummary = JSON.parse(
  logicalCornerResult.stdout,
) as TransformExecuteSummaryV0;

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

const nestAtRuleSummary = JSON.parse(
  nestAtRuleResult.stdout,
) as TransformExecuteSummaryV0;

assert.equal(nestAtRuleSummary.product, "omena-query.transform-execute");
assert.equal(
  nestAtRuleSummary.execution.outputCss,
  ".card { color: red; } .theme .card { color: blue; } .theme .card .title { color: green; } .card:is(:hover, :focus) { color: purple; }",
);
assert.deepEqual(nestAtRuleSummary.execution.executedPassIds, [
  "nesting-unwrap",
  "print-css",
]);
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
        ":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--pkg-brand); --bridge: var(--pkg-border); } .button { color: var(--alias); }",
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
  ":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--theme-brand); --bridge: #123456; } .button { color: var(--alias); }",
);
assert.deepEqual(designTokenAliasSummary.execution.executedPassIds, [
  "design-token-routing",
  "print-css",
]);
assert.equal(designTokenAliasSummary.execution.mutationCount, 2);

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
        "@value primary: #fff; @value gap: 8px; @value alias: primary; @value shadow: 0 0 4px primary; @value bp: 40rem; @value wide: 80rem; .button { color: alias; padding: gap gap; box-shadow: shadow; } @media screen and (min-width: bp) and (width >= wide) { .button { color: alias; } } @container card (inline-size >= wide) { .button { padding: gap; } }",
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
  "      .button { color: #fff; padding: 8px 8px; box-shadow: 0 0 4px #fff; } @media screen and (min-width: 40rem) and (width >= 80rem) { .button { color: #fff; } } @container card (inline-size >= 80rem) { .button { padding: 8px; } }",
);
assert.deepEqual(compositeValueSummary.execution.executedPassIds, [
  "value-resolution",
  "print-css",
]);
assert.equal(compositeValueSummary.execution.mutationCount, 14);

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

const colorMixAlphaSummary = JSON.parse(
  colorMixAlphaResult.stdout,
) as TransformExecuteSummaryV0;

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

const colorMixLinearSummary = JSON.parse(
  colorMixLinearResult.stdout,
) as TransformExecuteSummaryV0;

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
        '@property --registered { syntax: "<color>"; inherits: false; initial-value: red; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } :root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap) var(--also-broken; --tone: red; --tone: blue !important; --dynamic: env(theme-color); } .card { --brand: blue; color: var(--brand); margin: var(--gap); border-color: var(--tone); outline-color: var(--registered); box-shadow: 0 0 var(--gap) var(--broken; text-decoration-color: var(--dynamic); } .other { color: var(--brand); box-shadow: var(--shadow); }',
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
  '@property --registered { syntax: "<color>"; inherits: false; initial-value: red; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } :root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap) var(--also-broken; --tone: red; --tone: blue !important; --dynamic: env(theme-color); } .card { --brand: blue; color: var(--brand); margin: 2rem; border-color: var(--tone); outline-color: red; box-shadow: 0 0 2rem var(--broken; text-decoration-color: var(--dynamic); } .other { color: var(--brand); box-shadow: 0 0 2rem var(--also-broken; }',
);
assert.deepEqual(staticVarShadowSummary.execution.executedPassIds, [
  "custom-property-static-resolve",
  "print-css",
]);
assert.equal(staticVarShadowSummary.execution.mutationCount, 4);

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
assert.deepEqual(customPropertyReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyReachabilitySummary.execution.mutationCount, 1);

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
        ":root { --used: red; --ghost: blue; } .btn { animation: live 1s; } @keyframes live { to { color: var(--used); } } @keyframes ghost { to { color: var(--ghost); } }",
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

assert.equal(
  customPropertyKeyframeReachabilitySummary.product,
  "omena-query.transform-execute",
);
assert.ok(
  customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("--used: red;"),
);
assert.ok(
  customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("@keyframes ghost"),
);
assert.ok(
  customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("color: var(--ghost);"),
);
assert.ok(
  !customPropertyKeyframeReachabilitySummary.execution.outputCss.includes("--ghost: blue;"),
);
assert.deepEqual(customPropertyKeyframeReachabilitySummary.execution.executedPassIds, [
  "tree-shake-custom-property",
  "print-css",
]);
assert.equal(customPropertyKeyframeReachabilitySummary.execution.mutationCount, 1);

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
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    "@property --theme",
  ),
);
assert.ok(
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    "--theme: dark;",
  ),
);
assert.ok(
  customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    "@container card style(--theme: dark)",
  ),
);
assert.ok(
  !customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    "@property --dead",
  ),
);
assert.ok(
  !customPropertyContainerStyleReachabilitySummary.execution.outputCss.includes(
    ":root { --theme: dark; --dead: off;",
  ),
);
assert.deepEqual(
  customPropertyContainerStyleReachabilitySummary.execution.executedPassIds,
  ["tree-shake-custom-property", "print-css"],
);
assert.equal(customPropertyContainerStyleReachabilitySummary.execution.mutationCount, 2);

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
            '@value primary: red; @value shadow: 0 0 primary; @value dead: blue; :export { public-color: shadow; dead-public: dead; } .button { color: red; }',
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
assert.ok(icssExportReachabilitySummary.execution.outputCss.includes("@value shadow: 0 0 primary;"));
assert.ok(icssExportReachabilitySummary.execution.outputCss.includes(":export { public-color: shadow;"));
assert.ok(!icssExportReachabilitySummary.execution.outputCss.includes("@value dead:"));
assert.ok(!icssExportReachabilitySummary.execution.outputCss.includes("dead-public: dead"));
assertIncludesAll(
  icssExportReachabilitySummary.execution.semanticRemovals.map(
    (removal) => `${removal.symbolKind}:${removal.name}`,
  ),
  ["cssModuleValue:dead", "cssModuleIcssExport:dead-public"],
  "ICSS export reachability removals",
);
assert.equal(icssExportReachabilitySummary.semanticRemovalCount, 2);

process.stdout.write(
  [
    "validated omena-query transform-execute runtime:",
    `executed=${summary.execution.executedPassIds.length}`,
    `mutations=${summary.execution.mutationCount}`,
    `unknown=${summary.unknownPassIds.length}`,
    `contextExecuted=${contextSummary.execution.executedPassIds.length}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
    `designTokenAliasMutations=${designTokenAliasSummary.execution.mutationCount}`,
    `groupedComposesMutations=${groupedComposesSummary.execution.mutationCount}`,
    `globalComposesHashMutations=${globalComposesHashSummary.execution.mutationCount}`,
    `alphaColorMutations=${alphaColorFunctionSummary.execution.mutationCount}`,
    `alphaOkColorMutations=${alphaOkColorSummary.execution.mutationCount}`,
    `compositeValueMutations=${compositeValueSummary.execution.mutationCount}`,
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
    `customPropertyReachabilityMutations=${customPropertyReachabilitySummary.execution.mutationCount}`,
    `customPropertyContainerStyleMutations=${customPropertyContainerStyleReachabilitySummary.execution.mutationCount}`,
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
