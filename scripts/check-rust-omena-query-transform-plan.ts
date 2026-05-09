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
assert.deepEqual(summary.execution.executedPassIds, ["p40-print-css"]);
assertIncludesAll(
  summary.execution.plannedOnlyPassIds,
  [
    "p26-import-inline",
    "p27-scss-module-evaluate",
    "p30-composes-resolution",
    "p29-css-modules-class-hashing",
    "p31-value-resolution",
    "p15-light-dark-lowering",
    "p14-vendor-prefixing",
    "p20-nesting-unwrap",
  ],
  "transform execution planned-only passes",
);
assert.equal(summary.execution.passPlan.product, "omena-transform-passes.plan");
assert.equal(summary.execution.passPlan.violatedDagEdgeCount, 0);
assert.equal(summary.execution.passPlan.allRequestedRegistered, true);

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

process.stdout.write(
  [
    "validated omena-query transform-plan runtime:",
    `edges=${summary.bundle.bundleEdges.length}`,
    `passes=${summary.combinedPassIds.length}`,
    `violatedDagEdges=${summary.combinedViolatedDagEdgeCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
