import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";

const generatedPassFeatureBindingSource = readFileSync(
  "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
  "utf8",
);
const generatedPassFeatureBindingCount = countTomlRepeatedTables(
  generatedPassFeatureBindingSource,
  "binding",
);
const engineShadowRunnerBinary = path.resolve(
  "rust/target/debug",
  process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner",
);
const engineShadowRunnerBuild = spawnSync(
  "cargo",
  ["build", "--quiet", "--manifest-path", "rust/Cargo.toml", "-p", "engine-shadow-runner"],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(engineShadowRunnerBuild.error, undefined);
assert.equal(engineShadowRunnerBuild.status, 0, engineShadowRunnerBuild.stderr);

interface PassPlanV0 {
  readonly product: string;
  readonly orderedPassIds: readonly string[];
  readonly violatedDagEdgeCount: number;
  readonly allRequestedRegistered: boolean;
}

interface TargetDataEvidenceV0 {
  readonly product: string;
  readonly passId: string;
  readonly supportTable: string;
  readonly caniuseKeys: readonly string[];
  readonly sourceQuorum: readonly string[];
  readonly lastVerified: readonly string[];
  readonly allResolvedTargetsSupported: boolean;
  readonly resolvedTargets: readonly {
    readonly browser: string;
    readonly version: string;
    readonly supported: boolean;
    readonly matchedThreshold: {
      readonly browser: string;
      readonly minVersion: string;
      readonly caniuseKey: string;
      readonly sourceQuorum: readonly string[];
      readonly lastVerified: string;
    } | null;
  }[];
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
    readonly targetDataContractId: string;
    readonly targetDataSnapshotId: string;
    readonly targetDataEvidence: readonly TargetDataEvidenceV0[];
    readonly vendorPrefixMatrixSource: string;
    readonly resolvedTargets: readonly string[];
    readonly resolutionError: string | null;
    readonly support: {
      readonly vendorPrefixRequired: boolean;
      readonly supportsLightDark: boolean;
      readonly supportsColorMix: boolean;
      readonly supportsOklchOklab: boolean;
      readonly supportsLogicalProperties: boolean;
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
  readonly eggWitnesses: readonly {
    readonly passId: string;
    readonly sourceKind: string;
    readonly cssBefore: string;
    readonly cssAfter: string;
    readonly execution: {
      readonly product: string;
      readonly accepted: boolean;
      readonly afterMatchesCandidate: boolean;
    };
  }[];
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
      readonly blockedCount: number;
      readonly checkedPassIds: readonly string[];
      readonly obligations: readonly {
        readonly passId: string;
        readonly proofProduct: string;
        readonly accepted: boolean;
        readonly blockedReason?: string | null;
        readonly checkedObligations: readonly string[];
      }[];
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

const result = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

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

const cssImportPlanResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

assert.equal(cssImportPlanResult.status, 0, cssImportPlanResult.stderr);
assert.equal(cssImportPlanResult.error, undefined);
const cssImportPlanSummary = JSON.parse(cssImportPlanResult.stdout) as TransformPlanSummaryV0;
assert.deepEqual(
  cssImportPlanSummary.bundle.bundleEdges.map((edge) => edge.kind),
  ["cssImport"],
);
assert.deepEqual(cssImportPlanSummary.bundle.requiredPassIds, ["import-inline"]);
assert(!cssImportPlanSummary.combinedPassIds.includes("scss-module-evaluate"));

const targetQueryResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

assert.equal(targetQueryResult.status, 0, targetQueryResult.stderr);
assert.equal(targetQueryResult.error, undefined);

const targetQuerySummary = JSON.parse(targetQueryResult.stdout) as TransformPlanSummaryV0;

assert.equal(targetQuerySummary.product, "omena-query.transform-plan");
assert.equal(targetQuerySummary.targetQuery?.product, "omena-transform-target.query-plan");
assert.equal(targetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.equal(
  targetQuerySummary.targetQuery?.targetDataSource,
  "oxcBrowserslistV3+browserThresholdsTomlV0+generatedFeatureMatrixV0",
);
assert.equal(
  targetQuerySummary.targetQuery?.targetDataContractId,
  "omena-transform-target-data-v0",
);
assert.equal(
  targetQuerySummary.targetQuery?.targetDataSnapshotId,
  "omena-transform-target-data-v0:thresholds-2026-07-19:bindings-2026-07-19",
);
assert.equal(
  targetQuerySummary.targetQuery?.vendorPrefixMatrixSource,
  "curatedVendorPrefixMatrixTomlV0",
);
assert.deepEqual(targetQuerySummary.targetQuery?.resolvedTargets, ["ie 11"]);
assert.equal(targetQuerySummary.targetQuery?.resolutionError, null);
assert.equal(
  targetQuerySummary.targetQuery?.targetDataEvidence.length,
  generatedPassFeatureBindingCount,
);
const ieLightDarkEvidence = requireTargetDataEvidence(targetQuerySummary, "light_dark");
assert.equal(ieLightDarkEvidence.product, "omena-transform-target.data-evidence");
assert.equal(ieLightDarkEvidence.passId, "light-dark-lowering");
assert.deepEqual(ieLightDarkEvidence.caniuseKeys, ["css-light-dark-function"]);
assert.deepEqual(ieLightDarkEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
assert.deepEqual(ieLightDarkEvidence.lastVerified, ["2026-07-19"]);
assert.equal(ieLightDarkEvidence.allResolvedTargetsSupported, false);
const ieLightDarkTarget = requireResolvedTargetEvidence(ieLightDarkEvidence, "ie");
assert.equal(ieLightDarkTarget.version, "11");
assert.equal(ieLightDarkTarget.supported, false);
assert.equal(ieLightDarkTarget.matchedThreshold, null);
const ieStickyEvidence = requireTargetDataEvidence(targetQuerySummary, "sticky_positioning");
assert.equal(ieStickyEvidence.passId, "vendor-prefixing");
assert.deepEqual(ieStickyEvidence.caniuseKeys, ["css-sticky"]);
assert.deepEqual(ieStickyEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
assert.equal(ieStickyEvidence.allResolvedTargetsSupported, false);
const ieFlexboxStalePrefixEvidence = requireTargetDataEvidenceForPass(
  targetQuerySummary,
  "flexbox",
  "stale-prefix-removal",
);
assert.deepEqual(ieFlexboxStalePrefixEvidence.caniuseKeys, ["flexbox"]);
assert.deepEqual(ieFlexboxStalePrefixEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
const ieStickyStalePrefixEvidence = requireTargetDataEvidenceForPass(
  targetQuerySummary,
  "sticky_positioning",
  "stale-prefix-removal",
);
assert.deepEqual(ieStickyStalePrefixEvidence.caniuseKeys, ["css-sticky"]);
assert.deepEqual(ieStickyStalePrefixEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
const ieOklchEvidence = requireTargetDataEvidence(targetQuerySummary, "oklch_oklab");
assert.equal(ieOklchEvidence.passId, "oklch-oklab-lowering");
assert.deepEqual(ieOklchEvidence.caniuseKeys, ["css-lch-lab"]);
assert.deepEqual(ieOklchEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
assert.equal(ieOklchEvidence.allResolvedTargetsSupported, false);
const ieLogicalEvidence = requireTargetDataEvidence(targetQuerySummary, "logical_properties");
assert.equal(ieLogicalEvidence.passId, "logical-to-physical");
assert.deepEqual(ieLogicalEvidence.caniuseKeys, ["css-logical-props"]);
assert.deepEqual(ieLogicalEvidence.sourceQuorum, ["caniuse", "web-features", "mdn-bcd"]);
assert.equal(ieLogicalEvidence.allResolvedTargetsSupported, false);
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
const chrome122TargetQueryResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

assert.equal(chrome122TargetQueryResult.status, 0, chrome122TargetQueryResult.stderr);
assert.equal(chrome122TargetQueryResult.error, undefined);

const chrome122TargetQuerySummary = JSON.parse(
  chrome122TargetQueryResult.stdout,
) as TransformPlanSummaryV0;

assert.equal(chrome122TargetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.deepEqual(chrome122TargetQuerySummary.targetQuery?.resolvedTargets, ["chrome 122"]);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.vendorPrefixRequired, false);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsLightDark, false);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsColorMix, true);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsOklchOklab, true);
assert.equal(chrome122TargetQuerySummary.targetQuery?.support.supportsLogicalProperties, true);
const chrome122LightDarkEvidence = requireTargetDataEvidence(
  chrome122TargetQuerySummary,
  "light_dark",
);
const chrome122LightDarkTarget = requireResolvedTargetEvidence(
  chrome122LightDarkEvidence,
  "chrome",
);
assert.equal(chrome122LightDarkEvidence.allResolvedTargetsSupported, false);
assert.equal(chrome122LightDarkTarget.supported, false);
assert.equal(chrome122LightDarkTarget.matchedThreshold?.minVersion, "123.0");
assert.equal(chrome122LightDarkTarget.matchedThreshold?.caniuseKey, "css-light-dark-function");
assert.deepEqual(chrome122LightDarkTarget.matchedThreshold?.sourceQuorum, [
  "caniuse",
  "web-features",
  "mdn-bcd",
]);
assert.equal(chrome122LightDarkTarget.matchedThreshold?.lastVerified, "2026-07-19");
const chrome122ColorMixEvidence = requireTargetDataEvidence(
  chrome122TargetQuerySummary,
  "color_mix",
);
const chrome122ColorMixTarget = requireResolvedTargetEvidence(chrome122ColorMixEvidence, "chrome");
assert.equal(chrome122ColorMixEvidence.allResolvedTargetsSupported, true);
assert.equal(chrome122ColorMixTarget.supported, true);
assert.equal(chrome122ColorMixTarget.matchedThreshold?.minVersion, "111.0");
assert.equal(chrome122ColorMixTarget.matchedThreshold?.caniuseKey, "css-color-mix");
assert.deepEqual(chrome122TargetQuerySummary.target.plannedPassIds, [
  "stale-prefix-removal",
  "light-dark-lowering",
]);
assert.equal(
  chrome122TargetQuerySummary.execution.outputCss,
  ".button { color: #000; border-color: color-mix(in srgb, red, blue); } @media (prefers-color-scheme: dark) { .button { color: #fff; } }",
);

const chrome123TargetQueryResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

assert.equal(chrome123TargetQueryResult.status, 0, chrome123TargetQueryResult.stderr);
assert.equal(chrome123TargetQueryResult.error, undefined);

const chrome123TargetQuerySummary = JSON.parse(
  chrome123TargetQueryResult.stdout,
) as TransformPlanSummaryV0;

assert.equal(chrome123TargetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.deepEqual(chrome123TargetQuerySummary.targetQuery?.resolvedTargets, ["chrome 123"]);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.vendorPrefixRequired, false);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsLightDark, true);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsColorMix, true);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsOklchOklab, true);
assert.equal(chrome123TargetQuerySummary.targetQuery?.support.supportsLogicalProperties, true);
assert.deepEqual(chrome123TargetQuerySummary.target.plannedPassIds, ["stale-prefix-removal"]);
assert.equal(chrome123TargetQuerySummary.execution.outputCss, targetCompatStyleSource);

const stalePrefixTargetStyleSource =
  ".a { -webkit-user-select: none; -moz-user-select: none; user-select: none; -webkit-transform: translateX(1px) !important; transform: translateX(1px) !important; } .keep { -webkit-user-select: text; user-select: none; }";
const stalePrefixTargetQueryResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
  cwd: process.cwd(),
  encoding: "utf8",
  input: JSON.stringify({
    stylePath: "Compat.css",
    styleSource: stalePrefixTargetStyleSource,
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
});

assert.equal(stalePrefixTargetQueryResult.status, 0, stalePrefixTargetQueryResult.stderr);
assert.equal(stalePrefixTargetQueryResult.error, undefined);

const stalePrefixTargetQuerySummary = JSON.parse(
  stalePrefixTargetQueryResult.stdout,
) as TransformPlanSummaryV0;

assert.equal(stalePrefixTargetQuerySummary.targetQuery?.profileId, "browserslist-resolved");
assert.deepEqual(stalePrefixTargetQuerySummary.targetQuery?.resolvedTargets, ["chrome 123"]);
assert.deepEqual(stalePrefixTargetQuerySummary.target.plannedPassIds, ["stale-prefix-removal"]);
assert.deepEqual(stalePrefixTargetQuerySummary.combinedPassIds, [
  "stale-prefix-removal",
  "print-css",
]);
assert.deepEqual(stalePrefixTargetQuerySummary.execution.executedPassIds, [
  "stale-prefix-removal",
  "print-css",
]);
assert.equal(stalePrefixTargetQuerySummary.execution.mutationCount, 3);
assert.equal(
  stalePrefixTargetQuerySummary.execution.outputCss,
  ".a {   user-select: none;  transform: translateX(1px) !important; } .keep { -webkit-user-select: text; user-select: none; }",
);
assert.ok(!stalePrefixTargetQuerySummary.execution.outputCss.includes("-moz-user-select: none"));
assert.ok(
  !stalePrefixTargetQuerySummary.execution.outputCss.includes(
    "-webkit-transform: translateX(1px) !important",
  ),
);
assert.ok(
  stalePrefixTargetQuerySummary.execution.outputCss.includes("-webkit-user-select: text"),
  "stale-prefix-removal must keep prefixed declarations when exact-peer proof is unavailable",
);
assert.equal(
  stalePrefixTargetQuerySummary.execution.cascadeProofObligations.product,
  "omena-transform-passes.cascade-proof-obligations",
);
assert.equal(stalePrefixTargetQuerySummary.execution.cascadeProofObligations.obligationCount, 3);
assert.equal(stalePrefixTargetQuerySummary.execution.cascadeProofObligations.acceptedCount, 3);
assert.equal(stalePrefixTargetQuerySummary.execution.cascadeProofObligations.blockedCount, 0);
assert.deepEqual(stalePrefixTargetQuerySummary.execution.cascadeProofObligations.checkedPassIds, [
  "stale-prefix-removal",
]);
assert.ok(
  stalePrefixTargetQuerySummary.execution.cascadeProofObligations.obligations.every(
    (obligation) =>
      obligation.passId === "stale-prefix-removal" &&
      obligation.proofProduct === "omena-cascade.stale-prefix-removal-proof" &&
      obligation.accepted &&
      obligation.blockedReason === null &&
      obligation.checkedObligations.includes("knownVendorPrefixMapping") &&
      obligation.checkedObligations.includes("exactUnprefixedPeer") &&
      obligation.checkedObligations.includes("sameImportantFlag"),
  ),
);
assert.equal(stalePrefixTargetQuerySummary.eggWitnesses.length, 3);
assert.ok(
  stalePrefixTargetQuerySummary.eggWitnesses.every(
    (witness) =>
      witness.passId === "stale-prefix-removal" &&
      witness.sourceKind === "stalePrefixExactPeer" &&
      witness.execution.product === "omena-transform-egg.execution" &&
      witness.execution.accepted &&
      witness.execution.afterMatchesCandidate,
  ),
);

const contextStyleSource =
  '@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }';

const contextResult = spawnSync(engineShadowRunnerBinary, ["transform-plan"], {
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
});

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

function countTomlRepeatedTables(source: string, tableName: string): number {
  return source.split(/\r?\n/).filter((line) => line.trim() === `[[${tableName}]]`).length;
}

function requireTargetDataEvidence(
  planSummary: TransformPlanSummaryV0,
  supportTable: string,
): TargetDataEvidenceV0 {
  if (!planSummary.targetQuery) {
    throw new Error(`target query plan is required for ${supportTable} evidence`);
  }
  const evidence = planSummary.targetQuery.targetDataEvidence.find(
    (candidate) => candidate.supportTable === supportTable,
  );
  if (!evidence) {
    throw new Error(`target query evidence must include ${supportTable}`);
  }
  return evidence;
}

function requireTargetDataEvidenceForPass(
  planSummary: TransformPlanSummaryV0,
  supportTable: string,
  passId: string,
): TargetDataEvidenceV0 {
  if (!planSummary.targetQuery) {
    throw new Error(`target query plan is required for ${supportTable}/${passId} evidence`);
  }
  const evidence = planSummary.targetQuery.targetDataEvidence.find(
    (candidate) => candidate.supportTable === supportTable && candidate.passId === passId,
  );
  if (!evidence) {
    throw new Error(`target query evidence must include ${supportTable}/${passId}`);
  }
  return evidence;
}

function requireResolvedTargetEvidence(
  evidence: TargetDataEvidenceV0,
  browser: string,
): TargetDataEvidenceV0["resolvedTargets"][number] {
  const target = evidence.resolvedTargets.find((candidate) => candidate.browser === browser);
  if (!target) {
    throw new Error(`${evidence.supportTable} evidence must include ${browser}`);
  }
  return target;
}

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
