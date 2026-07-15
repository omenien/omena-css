import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

interface BuildEnvelope {
  readonly product: string;
  readonly payload: {
    readonly execution: { readonly outputCss: string };
    readonly readySurfaces: readonly string[];
    readonly postcssCompat: {
      readonly pluginId: string;
      readonly packageName: string;
      readonly pluginVersion: string;
      readonly configDigest: string;
      readonly configuredTargets: readonly string[];
      readonly inputDigest: string;
      readonly exitStatus: number;
      readonly adopted: boolean;
      readonly outputCss: string;
      readonly evidence: {
        readonly key: { readonly inputIdentity: string };
        readonly earnedVia: string;
        readonly provenance: readonly string[];
      };
      readonly semanticDiff: {
        readonly totalChangeCount: number;
        readonly understoodChangeCount: number;
        readonly passthroughChangeCount: number;
        readonly allChangesClassified: boolean;
        readonly changes: readonly unknown[];
      };
    };
    readonly postcssNativeDifferential?: {
      readonly product: string;
      readonly comparisonBasis: string;
      readonly pluginId: string;
      readonly nativeTargetQuery: string;
      readonly stage1Targets: readonly string[];
      readonly targetSetsAligned: boolean;
      readonly classification: "equivalent" | "nativeConservative" | "investigationRequired";
      readonly matchedUncoveredFeatureIds: readonly string[];
      readonly coverageBoundaryRespected: boolean;
      readonly requiresInvestigation: boolean;
      readonly semanticDiff: {
        readonly totalChangeCount: number;
        readonly understoodChangeCount: number;
        readonly passthroughChangeCount: number;
        readonly allChangesClassified: boolean;
      };
    };
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const executable = path.join(
  repoRoot,
  "rust",
  "target",
  "debug",
  `omena${process.platform === "win32" ? ".exe" : ""}`,
);
const build = spawnSync(
  "cargo",
  ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli", "--bin", "omena"],
  { cwd: repoRoot, encoding: "utf8" },
);
assert.equal(build.status, 0, build.stderr || build.stdout);

const fixture = path.join(
  repoRoot,
  "rust",
  "crates",
  "omena-cli",
  "fixtures",
  "postcss-compat",
  "input.css",
);
const run = spawnSync(executable, ["build", fixture, "--json"], {
  cwd: repoRoot,
  encoding: "utf8",
});
assert.equal(run.status, 0, run.stderr || run.stdout);
const envelope = JSON.parse(run.stdout) as BuildEnvelope;
const report = envelope.payload.postcssCompat;
const semantic = report.semanticDiff;
const observedChanges =
  process.env.OMENA_POSTCSS_COMPAT_TEST_DROP_CHANGE === "1"
    ? semantic.changes.slice(1)
    : semantic.changes;
const observedEvidence =
  process.env.OMENA_POSTCSS_COMPAT_TEST_DROP_EVIDENCE === "1" ? undefined : report.evidence;

assert.equal(envelope.product, "omena-cli.build");
assert.equal(report.pluginId, "autoprefixer-legacy-browsers");
assert.equal(report.packageName, "autoprefixer");
assert.equal(report.pluginVersion, "10.5.2");
assert.deepEqual(report.configuredTargets, ["Firefox 20", "Safari 8"]);
assert.match(report.configDigest, /^[a-f0-9]{64}$/u);
assert.match(report.inputDigest, /^[a-f0-9]{64}$/u);
assert.equal(report.exitStatus, 0);
assert.equal(report.adopted, true);
assert.ok(observedEvidence, "adopted external output must carry an invocation witness");
assert.equal(observedEvidence.earnedVia, "externalTool");
assert.ok(observedEvidence.key.inputIdentity.endsWith(report.inputDigest));
assert.ok(observedEvidence.provenance.includes("toolVersion:10.5.2"));
assert.equal(envelope.payload.execution.outputCss, report.outputCss);
assert.match(report.outputCss, /-webkit-appearance/u);
assert.match(report.outputCss, /::-moz-placeholder/u);
assert.ok(semantic.understoodChangeCount > 0);
assert.ok(semantic.passthroughChangeCount > 0);
assert.equal(
  semantic.understoodChangeCount + semantic.passthroughChangeCount,
  semantic.totalChangeCount,
);
assert.equal(observedChanges.length, semantic.totalChangeCount);
assert.equal(semantic.allChangesClassified, true);
assert.ok(envelope.payload.readySurfaces.includes("postcssCompatibilityRunner"));

const differentialRoot = mkdtempSync(path.join(repoRoot, ".omena-postcss-native-"));
try {
  const runDifferential = (name: string, source: string): BuildEnvelope => {
    const projectRoot = path.join(differentialRoot, name);
    const input = path.join(projectRoot, "input.css");
    const config = path.join(projectRoot, "omena.toml");
    const output = path.join(projectRoot, "output.css");
    mkdirSync(projectRoot, { recursive: true });
    writeFileSync(input, source);
    writeFileSync(
      config,
      '[build]\npostcssCompat = "autoprefixer-legacy-browsers"\ntargetQuery = "Firefox 20, Safari 8"\n',
    );
    const result = spawnSync(executable, ["build", input, "--output", output, "--json"], {
      cwd: repoRoot,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr || result.stdout);
    return JSON.parse(result.stdout) as BuildEnvelope;
  };

  const cases = [
    runDifferential("equivalent", ".input { appearance: none; }\n"),
    runDifferential("conservative", ".input { hyphens: auto; }\n"),
    runDifferential("investigate", "::placeholder { color: gray; }\n"),
  ];
  const classifications = cases.map((candidate) => {
    const differential = candidate.payload.postcssNativeDifferential;
    assert.ok(differential, "a target-aware compatibility build must expose its differential");
    assert.equal(differential.product, "omena-cli.postcss-native-differential");
    assert.equal(differential.comparisonBasis, "semanticObservationNotByteIdentity");
    assert.equal(differential.pluginId, "autoprefixer-legacy-browsers");
    assert.equal(differential.nativeTargetQuery, "firefox 20, safari 8");
    assert.deepEqual(differential.stage1Targets, ["Firefox 20", "Safari 8"]);
    assert.equal(differential.targetSetsAligned, true);
    assert.equal(differential.semanticDiff.allChangesClassified, true);
    assert.ok(candidate.payload.readySurfaces.includes("postcssNativeSemanticDifferential"));
    return differential.classification;
  });

  assert.deepEqual(classifications, ["equivalent", "nativeConservative", "investigationRequired"]);
  const conservative = cases[1].payload.postcssNativeDifferential;
  assert.ok(conservative);
  assert.deepEqual(conservative.matchedUncoveredFeatureIds, ["vendor-prefixing.hyphens"]);
  assert.equal(conservative.coverageBoundaryRespected, true);
  assert.equal(conservative.requiresInvestigation, false);
  const investigate = cases[2].payload.postcssNativeDifferential;
  assert.ok(investigate);
  assert.equal(investigate.coverageBoundaryRespected, false);
  assert.equal(investigate.requiresInvestigation, true);

  const composed = runDifferential(
    "composed",
    ".input { color: light-dark(red, blue); hyphens: auto; }\n",
  );
  assert.doesNotMatch(
    composed.payload.execution.outputCss,
    /light-dark\(/u,
    "Stage-1 fallback must preserve native value lowering",
  );
  assert.match(
    composed.payload.execution.outputCss,
    /-moz-hyphens/u,
    "the composed output must retain Stage-1 fallback coverage",
  );
} finally {
  rmSync(differentialRoot, { recursive: true, force: true });
}

const unknownRoot = mkdtempSync(path.join(os.tmpdir(), "omena-postcss-compat-"));
try {
  const unknownInput = path.join(unknownRoot, "input.css");
  const unknownOutput = path.join(unknownRoot, "output.css");
  writeFileSync(unknownInput, ".a { color: red; }\n");
  writeFileSync(
    path.join(unknownRoot, "omena.toml"),
    '[build]\npostcssCompat = "not-allowlisted"\n',
  );
  const rejected = spawnSync(executable, ["build", unknownInput, "--output", unknownOutput], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.notEqual(rejected.status, 0, "unknown plugin ids must fail closed");
  assert.match(rejected.stderr, /UnknownPlugin/u);
  assert.throws(() => readFileSync(unknownOutput), "a rejected run must not emit partial CSS");
} finally {
  rmSync(unknownRoot, { recursive: true, force: true });
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-cli-postcss-compat",
      pluginId: report.pluginId,
      pluginVersion: report.pluginVersion,
      totalChangeCount: semantic.totalChangeCount,
      understoodChangeCount: semantic.understoodChangeCount,
      passthroughChangeCount: semantic.passthroughChangeCount,
      nativeDifferentialClassificationCount: 3,
      witnessCount: 1,
      violations: 0,
    },
    null,
    2,
  )}\n`,
);
