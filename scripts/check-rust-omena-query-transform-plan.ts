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
  readonly targetQuery: {
    readonly product: string;
    readonly profileId: string;
    readonly targetDataSource: string;
    readonly resolvedTargets: readonly string[];
    readonly resolutionError: string | null;
    readonly support: {
      readonly supportsLightDark: boolean;
      readonly supportsColorMix: boolean;
    };
    readonly transformPlan: {
      readonly product: string;
      readonly requiredPassIds: readonly string[];
      readonly blockedPassIds: readonly string[];
      readonly plannedPassIds: readonly string[];
    };
  } | null;
  readonly egg: {
    readonly product: string;
    readonly plannedPassIds: readonly string[];
  };
  readonly print: {
    readonly product: string;
    readonly css: string;
    readonly sourceMapSegments: readonly {
      readonly sourcePath: string;
      readonly originalStart: number;
      readonly originalEnd: number;
      readonly generatedStart: number;
      readonly generatedEnd: number;
      readonly originalStartPoint: SourceMapPointV0;
      readonly originalEndPoint: SourceMapPointV0;
      readonly generatedStartPoint: SourceMapPointV0;
      readonly generatedEndPoint: SourceMapPointV0;
      readonly passId: string;
    }[];
    readonly provenancePreserved: boolean;
    readonly cstArtifact: {
      readonly product: string;
      readonly passIds: readonly string[];
      readonly stableIrNodeCount: number;
      readonly parserErrorCount: number;
      readonly containsBogusOrTrivia: boolean;
      readonly stableIr: {
        readonly product: string;
        readonly dialect: string;
        readonly nodeCount: number;
        readonly parserErrorCount: number;
        readonly containsBogusOrTrivia: boolean;
        readonly stablePostSemanticIr: boolean;
        readonly nodes: readonly {
          readonly kind: string;
          readonly label: string;
          readonly sourceSpanStart: number;
        }[];
        readonly provenanceAnchors: readonly unknown[];
      };
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
    readonly cascadeProofObligations: {
      readonly product: string;
      readonly obligationCount: number;
      readonly acceptedCount: number;
      readonly checkedPassIds: readonly string[];
    };
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

interface SourceMapPointV0 {
  readonly byteOffset: number;
  readonly line: number;
  readonly utf8Column: number;
  readonly utf16Column: number;
}

const styleSource =
  '@use "./tokens" as tokens;\n@value primary from "./colors.module.css";\n.button { composes: reset from "./reset.module.css"; --brand: tokens.$brand; color: var(--brand); }';

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
    "import-inline",
    "scss-module-evaluate",
    "css-modules-class-hashing",
    "composes-resolution",
    "value-resolution",
  ],
  "bundle required passes",
);
assertIncludesAll(
  summary.bundle.plannedPassIds,
  [
    "import-inline",
    "scss-module-evaluate",
    "css-modules-class-hashing",
    "composes-resolution",
    "value-resolution",
  ],
  "bundle planned passes",
);

assert.equal(summary.target.product, "omena-transform-target.plan");
assert.deepEqual(summary.target.blockedPassIds, []);
assertIncludesAll(
  summary.target.plannedPassIds,
  ["vendor-prefixing", "light-dark-lowering", "nesting-unwrap"],
  "target planned passes",
);
assert.equal(summary.targetQuery, null);

assert.equal(summary.egg.product, "omena-transform-egg.plan");
assert.deepEqual(summary.egg.plannedPassIds, []);

assert.equal(summary.print.product, "omena-transform-print.artifact");
assert.equal(summary.print.css, styleSource);
assert.equal(summary.print.provenancePreserved, true);
assert.equal(
  summary.print.sourceMapSegments.length,
  summary.execution.provenanceDerivationForest.nodeCount,
);
assert.equal(summary.print.sourceMapSegments[0]?.sourcePath, "Button.module.scss");
const firstSourceMapSegment = summary.print.sourceMapSegments[0];
assert.ok(firstSourceMapSegment, "transform-plan source map must include segments");
assert.equal(
  firstSourceMapSegment.originalStartPoint.byteOffset,
  firstSourceMapSegment.originalStart,
);
assert.equal(
  firstSourceMapSegment.generatedStartPoint.byteOffset,
  firstSourceMapSegment.generatedStart,
);
const finalLine = styleSource.split("\n").at(-1) ?? "";
assert.equal(firstSourceMapSegment.originalEndPoint.byteOffset, firstSourceMapSegment.originalEnd);
assert.equal(firstSourceMapSegment.originalEndPoint.line, 2);
assert.equal(firstSourceMapSegment.originalEndPoint.utf8Column, finalLine.length);
assert.equal(firstSourceMapSegment.originalEndPoint.utf16Column, finalLine.length);
assert.equal(summary.print.cstArtifact.product, "omena-transform-cst.artifact");
assert.equal(summary.print.cstArtifact.stableIr.product, "omena-transform-cst.stable-ir");
assert.equal(summary.print.cstArtifact.stableIr.dialect, "scss");
assert.equal(summary.print.cstArtifact.parserErrorCount, 0);
assert.equal(summary.print.cstArtifact.containsBogusOrTrivia, false);
assert.equal(summary.print.cstArtifact.stableIr.parserErrorCount, 0);
assert.equal(summary.print.cstArtifact.stableIr.containsBogusOrTrivia, false);
assert.equal(summary.print.cstArtifact.stableIr.stablePostSemanticIr, true);
assert.equal(
  summary.print.cstArtifact.stableIrNodeCount,
  summary.print.cstArtifact.stableIr.provenanceAnchors.length,
);
assert.equal(
  summary.print.cstArtifact.stableIr.nodeCount,
  summary.print.cstArtifact.stableIr.nodes.length,
);
assert(
  summary.print.cstArtifact.stableIr.nodes.some(
    (node) => node.kind === "classSelector" && node.label === "button",
  ),
  "stable transform IR should include parser-owned class selector fact",
);
assert(
  summary.print.cstArtifact.stableIr.nodes.some(
    (node) => node.kind === "sassModuleEdge" && node.label === "./tokens",
  ),
  "stable transform IR should include parser-owned Sass module edge fact",
);
assert(
  summary.print.cstArtifact.stableIr.nodes.some(
    (node) => node.kind === "customPropertyDeclaration" && node.label === "--brand",
  ),
  "stable transform IR should include parser-owned custom property declaration fact",
);
assert(
  summary.print.cstArtifact.stableIr.nodes.some(
    (node) => node.kind === "customPropertyReference" && node.label === "--brand",
  ),
  "stable transform IR should include parser-owned custom property reference fact",
);
assert(
  summary.print.cstArtifact.stableIr.nodes
    .slice(1)
    .every(
      (node, index) =>
        summary.print.cstArtifact.stableIr.nodes[index]!.sourceSpanStart <= node.sourceSpanStart,
    ),
  "stable transform IR nodes should be source ordered",
);
assert.equal(summary.print.cstArtifact.provenancePreserved, true);

assert.equal(summary.execution.product, "omena-transform-passes.execution");
assert.equal(summary.execution.outputCss, styleSource);
assert.equal(summary.print.css, summary.execution.outputCss);
assert.equal(summary.execution.mutationCount, 0);
assert.equal(summary.execution.provenancePreserved, true);
assert.deepEqual(summary.execution.executedPassIds, [
  "value-resolution",
  "light-dark-lowering",
  "nesting-unwrap",
  "vendor-prefixing",
  "print-css",
]);
assertIncludesAll(
  summary.execution.plannedOnlyPassIds,
  ["import-inline", "scss-module-evaluate", "composes-resolution", "css-modules-class-hashing"],
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
    "import-inline",
    "scss-module-evaluate",
    "composes-resolution",
    "css-modules-class-hashing",
    "value-resolution",
    "light-dark-lowering",
    "vendor-prefixing",
    "nesting-unwrap",
    "print-css",
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
    "cascadeProofObligations",
    "combinedTransformPassPlan",
  ],
  "transform ready surfaces",
);
assert.equal(
  summary.execution.cascadeProofObligations.product,
  "omena-transform-passes.cascade-proof-obligations",
);

const cssImportPlanResult = spawnSync(
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
      stylePath: "App.css",
      styleSource: '@import "./tokens.css"; .button { color: red; }',
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
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(cssImportPlanResult.status, 0, cssImportPlanResult.stderr);
assert.equal(cssImportPlanResult.error, undefined);
const cssImportPlanSummary = JSON.parse(cssImportPlanResult.stdout) as TransformPlanSummaryV0;
assert.deepEqual(
  cssImportPlanSummary.bundle.bundleEdges.map((edge) => edge.kind),
  ["cssImport"],
);
assert.deepEqual(cssImportPlanSummary.bundle.requiredPassIds, ["import-inline"]);
assert(!cssImportPlanSummary.combinedPassIds.includes("scss-module-evaluate"));

const targetQueryResult = spawnSync(
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
      stylePath: "Legacy.module.css",
      styleSource: ".button { display: flex; color: light-dark(#000, #fff); }",
      targetQuery: "ie 11",
      targetOptions: {
        allowLogicalToPhysical: true,
        allowScopeFlatten: true,
        allowLayerFlatten: true,
        enableSupportsStaticEval: false,
        enableMediaStaticEval: false,
      },
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(targetQueryResult.status, 0, targetQueryResult.stderr);
assert.equal(targetQueryResult.error, undefined);

const targetQuerySummary = JSON.parse(targetQueryResult.stdout) as TransformPlanSummaryV0;

assert.equal(targetQuerySummary.product, "omena-query.transform-plan");
assert.equal(targetQuerySummary.targetQuery?.product, "omena-transform-target.query-plan");
assert.equal(targetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.equal(targetQuerySummary.targetQuery?.targetDataSource, "oxcBrowserslistV3+featureSubsetV0");
assert.deepEqual(targetQuerySummary.targetQuery?.resolvedTargets, ["ie 11"]);
assert.equal(targetQuerySummary.targetQuery?.resolutionError, null);
assert.deepEqual(
  targetQuerySummary.target.plannedPassIds,
  targetQuerySummary.targetQuery?.transformPlan.plannedPassIds,
);
assertIncludesAll(
  targetQuerySummary.target.plannedPassIds,
  ["vendor-prefixing", "light-dark-lowering", "color-function-lowering", "nesting-unwrap"],
  "browserslist target-query planned passes",
);

const targetCompatStyleSource =
  ".button { color: light-dark(#000, #fff); border-color: color-mix(in srgb, red, blue); }";
const chrome122TargetQueryResult = spawnSync(
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
      stylePath: "Compat.css",
      styleSource: targetCompatStyleSource,
      targetQuery: "chrome 122",
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

assert.equal(chrome122TargetQueryResult.status, 0, chrome122TargetQueryResult.stderr);
assert.equal(chrome122TargetQueryResult.error, undefined);

const chrome122TargetQuerySummary = JSON.parse(
  chrome122TargetQueryResult.stdout,
) as TransformPlanSummaryV0;

assert.equal(chrome122TargetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.deepEqual(chrome122TargetQuerySummary.targetQuery?.resolvedTargets, ["chrome 122"]);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsLightDark, false);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsColorMix, true);
assert.deepEqual(chrome122TargetQuerySummary.target.plannedPassIds, ["light-dark-lowering"]);
assert.equal(
  chrome122TargetQuerySummary.execution.outputCss,
  ".button { color: #000; border-color: color-mix(in srgb, red, blue); } @media (prefers-color-scheme: dark) { .button { color: #fff; } }",
);

const chrome123TargetQueryResult = spawnSync(
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
      stylePath: "Compat.css",
      styleSource: targetCompatStyleSource,
      targetQuery: "chrome 123",
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

assert.equal(chrome123TargetQueryResult.status, 0, chrome123TargetQueryResult.stderr);
assert.equal(chrome123TargetQueryResult.error, undefined);

const chrome123TargetQuerySummary = JSON.parse(
  chrome123TargetQueryResult.stdout,
) as TransformPlanSummaryV0;

assert.equal(chrome123TargetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.deepEqual(chrome123TargetQuerySummary.targetQuery?.resolvedTargets, ["chrome 123"]);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsLightDark, true);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsColorMix, true);
assert.deepEqual(chrome123TargetQuerySummary.target.plannedPassIds, []);
assert.equal(chrome123TargetQuerySummary.execution.outputCss, targetCompatStyleSource);

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
  ":root { --brand: red; } ._button_x{  color: var(--brand); } ._base_y{ color: blue; }",
);
assert.equal(contextSummary.print.css, contextSummary.execution.outputCss);
assert.equal(
  contextSummary.print.sourceMapSegments.length,
  contextSummary.execution.provenanceDerivationForest.nodeCount,
);
assert.ok(
  contextSummary.print.sourceMapSegments.some((segment) => segment.passId === "import-inline"),
  "context transform-plan source map must include import inline pass",
);
assert.ok(
  contextSummary.print.sourceMapSegments.some(
    (segment) => segment.generatedEnd === contextSummary.execution.outputCss.length,
  ),
  "context transform-plan source map must include final generated length",
);
assertIncludesAll(
  contextSummary.execution.executedPassIds,
  ["import-inline", "composes-resolution", "css-modules-class-hashing", "print-css"],
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
    `chrome122TargetPasses=${chrome122TargetQuerySummary.target.plannedPassIds.join(",")}`,
    `chrome123TargetPasses=${chrome123TargetQuerySummary.target.plannedPassIds.join(",") || "none"}`,
    `contextMutations=${contextSummary.execution.mutationCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
