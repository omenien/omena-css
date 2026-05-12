import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";

interface PassPlanV0 {
  readonly product: string;
  readonly orderedPassIds: readonly string[];
  readonly violatedDagEdgeCount: number;
  readonly allRequestedRegistered: boolean;
}

interface TransformPlanSummaryV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stylePath: string;
  readonly dialect: string;
  readonly bundle: {
    readonly product: string;
    readonly bundleEdges: readonly { readonly kind: string; readonly importSource: string }[];
    readonly requiredPassIds: readonly string[];
    readonly plannedPassIds: readonly string[];
  };
  readonly target: {
    readonly product: string;
    readonly requiredPassIds: readonly string[];
    readonly blockedPassIds: readonly string[];
    readonly plannedPassIds: readonly string[];
  };
  readonly egg: {
    readonly product: string;
    readonly plannedPassIds: readonly string[];
  };
  readonly print: {
    readonly product: string;
    readonly css: string;
    readonly provenancePreserved: boolean;
    readonly cstArtifact: {
      readonly product: string;
      readonly passIds: readonly string[];
      readonly provenancePreserved: boolean;
    };
  };
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
  readonly combinedPlan: PassPlanV0;
  readonly combinedPassIds: readonly string[];
  readonly combinedViolatedDagEdgeCount: number;
  readonly readySurfaces: readonly string[];
}

const styleSource =
  '@use "./tokens" as tokens; @value primary from "./colors.module.css"; .button { composes: reset from "./reset.module.css"; color: tokens.$brand; }';

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
    "transform-plan",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.scss",
      styleSource,
      targetLabel: "legacy-webview",
      targetSupport: {
        vendorPrefixRequired: true,
        supportsLightDark: false,
        supportsColorMix: true,
        supportsOklchOklab: true,
        supportsColorFunction: true,
        supportsLogicalProperties: true,
        supportsCssNesting: false,
        supportsCssScope: true,
        supportsCascadeLayers: true,
      },
      targetOptions: {
        allowLogicalToPhysical: false,
        allowScopeFlatten: false,
        allowLayerFlatten: false,
        enableSupportsStaticEval: false,
        enableMediaStaticEval: false,
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(result.status, 0, result.stderr);
assert.equal(result.error, undefined);

const summary = JSON.parse(result.stdout) as TransformPlanSummaryV0;

assert.equal(summary.schemaVersion, "0");
assert.equal(summary.product, "omena-query.transform-plan");
assert.equal(summary.stylePath, "Button.module.scss");
assert.equal(summary.dialect, "scss");

assert.equal(summary.bundle.product, "omena-transform-bundle.source");
assertIncludesAll(
  summary.bundle.bundleEdges.map((edge) => edge.kind),
  ["sassUse", "cssModuleValueImport", "cssModuleComposesExternal"],
  "transform bundle edge kinds",
);
assertIncludesAll(
  summary.bundle.bundleEdges.map((edge) => edge.importSource),
  ["./tokens", "./colors.module.css", "./reset.module.css"],
  "transform bundle imports",
);
assertIncludesAll(
  summary.bundle.requiredPassIds,
  [
    "p26-import-inline",
    "p27-scss-module-evaluate",
    "p29-css-modules-class-hashing",
    "p30-composes-resolution",
    "p31-value-resolution",
  ],
  "bundle required passes",
);
assertIncludesAll(
  summary.bundle.plannedPassIds,
  [
    "p26-import-inline",
    "p27-scss-module-evaluate",
    "p29-css-modules-class-hashing",
    "p30-composes-resolution",
    "p31-value-resolution",
  ],
  "bundle planned passes",
);

assert.equal(summary.target.product, "omena-transform-target.plan");
assert.deepEqual(summary.target.blockedPassIds, []);
assertIncludesAll(
  summary.target.plannedPassIds,
  ["p14-vendor-prefixing", "p15-light-dark-lowering", "p20-nesting-unwrap"],
  "target planned passes",
);

assert.equal(summary.egg.product, "omena-transform-egg.plan");
assert.deepEqual(summary.egg.plannedPassIds, []);

assert.equal(summary.print.product, "omena-transform-print.artifact");
assert.equal(summary.print.css, styleSource);
assert.equal(summary.print.provenancePreserved, true);
assert.equal(summary.print.cstArtifact.product, "omena-transform-cst.artifact");
assert.equal(summary.print.cstArtifact.provenancePreserved, true);

assert.equal(summary.execution.product, "omena-transform-passes.execution");
assert.equal(summary.execution.outputCss, styleSource);
assert.equal(summary.print.css, summary.execution.outputCss);
assert.equal(summary.execution.mutationCount, 0);
assert.equal(summary.execution.provenancePreserved, true);
assert.deepEqual(summary.execution.executedPassIds, [
  "p31-value-resolution",
  "p15-light-dark-lowering",
  "p20-nesting-unwrap",
  "p14-vendor-prefixing",
  "p40-print-css",
]);
assertIncludesAll(
  summary.execution.plannedOnlyPassIds,
  [
    "p26-import-inline",
    "p27-scss-module-evaluate",
    "p30-composes-resolution",
    "p29-css-modules-class-hashing",
  ],
  "transform execution planned-only passes",
);
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

assert.equal(summary.combinedPlan.product, "omena-transform-passes.plan");
assert.equal(summary.combinedPlan.violatedDagEdgeCount, 0);
assert.equal(summary.combinedPlan.allRequestedRegistered, true);
assert.equal(summary.combinedViolatedDagEdgeCount, 0);
assertIncludesAll(
  summary.combinedPassIds,
  [
    "p26-import-inline",
    "p27-scss-module-evaluate",
    "p30-composes-resolution",
    "p29-css-modules-class-hashing",
    "p31-value-resolution",
    "p15-light-dark-lowering",
    "p14-vendor-prefixing",
    "p20-nesting-unwrap",
    "p40-print-css",
  ],
  "combined transform pass plan",
);
assert.deepEqual(summary.combinedPassIds, summary.combinedPlan.orderedPassIds);
assertIncludesAll(
  summary.print.cstArtifact.passIds,
  summary.combinedPassIds,
  "print CST pass provenance",
);
assertIncludesAll(
  summary.readySurfaces,
  [
    "transformBundlePlan",
    "transformTargetPlan",
    "transformEggPlan",
    "transformPrintArtifact",
    "transformExecutionRuntime",
    "combinedTransformPassPlan",
  ],
  "transform ready surfaces",
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
    "transform-plan",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      stylePath: "Button.module.css",
      styleSource: contextStyleSource,
      targetLabel: "modern",
      targetSupport: {
        vendorPrefixRequired: false,
        supportsLightDark: true,
        supportsColorMix: true,
        supportsOklchOklab: true,
        supportsColorFunction: true,
        supportsLogicalProperties: true,
        supportsCssNesting: true,
        supportsCssScope: true,
        supportsCascadeLayers: true,
      },
      targetOptions: {
        allowLogicalToPhysical: false,
        allowScopeFlatten: false,
        allowLayerFlatten: false,
        enableSupportsStaticEval: false,
        enableMediaStaticEval: false,
      },
      transformContext: {
        importInlines: [{ importSource: "./tokens.css", replacementCss: ":root { --brand: red; }" }],
        cssModuleComposesResolutions: [
          { localClassName: "button", exportedClassNames: ["button", "base"] },
        ],
        classNameRewrites: [
          { originalName: "button", rewrittenName: "_button_x" },
          { originalName: "base", rewrittenName: "_base_y" },
        ],
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(contextResult.status, 0, contextResult.stderr);
assert.equal(contextResult.error, undefined);

const contextSummary = JSON.parse(contextResult.stdout) as TransformPlanSummaryV0;

assert.equal(contextSummary.schemaVersion, "0");
assert.equal(contextSummary.product, "omena-query.transform-plan");
assert.equal(contextSummary.stylePath, "Button.module.css");
assert.equal(
  contextSummary.execution.outputCss,
  ':root { --brand: red; } ._button_x{  color: var(--brand); } ._base_y{ color: blue; }',
);
assert.equal(contextSummary.print.css, contextSummary.execution.outputCss);
assertIncludesAll(
  contextSummary.execution.executedPassIds,
  ["p26-import-inline", "p30-composes-resolution", "p29-css-modules-class-hashing", "p40-print-css"],
  "context transform-plan executed passes",
);
assert.deepEqual(contextSummary.execution.cssImportInlines, [
  { importSource: "./tokens.css", replacementCss: ":root { --brand: red; }" },
]);
assert.deepEqual(contextSummary.execution.cssModuleComposesExports, [
  { localClassName: "button", exportedClassNames: ["button", "base"] },
]);
assert.equal(contextSummary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(contextSummary.combinedViolatedDagEdgeCount, 0);
assert.equal(
  contextSummary.execution.provenanceDerivationForest.product,
  "omena-transform-passes.provenance-derivation-forest",
);
assert.equal(contextSummary.execution.provenanceDerivationForest.rootCount, 1);
assert.equal(
  contextSummary.execution.provenanceDerivationForest.nodeCount,
  contextSummary.execution.provenanceDerivationForest.nodes.length,
);

process.stdout.write(
  [
    "validated omena-query transform-plan runtime:",
    `edges=${summary.bundle.bundleEdges.length}`,
    `passes=${summary.combinedPassIds.length}`,
    `violatedDagEdges=${summary.combinedViolatedDagEdgeCount}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
