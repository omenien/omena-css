import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  createOmenaBuildState,
  rebuildAndCache,
} = require("../packages/css-build-adapter/index.cjs");
const { omenaCss } = require("../packages/vite-plugin/index.cjs");
const { omenaPostcss } = require("../packages/postcss-plugin/index.cjs");
const postcss = require("postcss");
const scssSyntax = require("postcss-scss");

const styleSource = [
  '@use "./tokens" as tokens;',
  '@import "./base.css";',
  '@value primary from "./colors.module.css";',
  '.button { composes: reset from "./reset.module.css"; color: tokens.$brand; }',
].join("\n");

const rustBundleSummary = readRustBundleSummary("Button.module.scss", styleSource);
const rustPlannedPassIds = rustBundleSummary.plannedPassIds;

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

async function main() {
  for (const passId of [
    "import-inline",
    "scss-module-evaluate",
    "composes-resolution",
    "value-resolution",
    "css-modules-class-hashing",
  ]) {
    assert.ok(
      rustPlannedPassIds.includes(passId),
      `Rust bundle planner should include ${passId} for the Sass fixture`,
    );
  }

  await withFixture("adapter", async ({ root, stylePath }) => {
    const engine = createRecordingEngine(stylePath, rustBundleSummary);
    const state = createOmenaBuildState({ cwd: root });

    await rebuildAndCache(
      stylePath,
      styleSource,
      { cwd: root, configFile: false, engine, bundle: true },
      state,
    );

    assert.deepEqual(engine.passIdCalls, [rustPlannedPassIds]);
    assert.deepEqual(engine.plannerCalls, [[styleSource, stylePath]]);
  });

  await withFixture("vite", async ({ root, stylePath }) => {
    const engine = createRecordingEngine(stylePath, rustBundleSummary);
    const plugin = omenaCss({ cwd: root, configFile: false, engine, bundle: true });

    await plugin.transform.call({}, styleSource, stylePath);

    assert.deepEqual(engine.passIdCalls, [rustPlannedPassIds]);
    assert.deepEqual(engine.plannerCalls, [[styleSource, stylePath]]);
  });

  await withFixture("postcss", async ({ root, stylePath }) => {
    const engine = createRecordingEngine(stylePath, rustBundleSummary);

    await postcss([omenaPostcss({ cwd: root, configFile: false, engine, bundle: true })]).process(
      styleSource,
      { from: stylePath, syntax: scssSyntax },
    );

    assert.deepEqual(engine.passIdCalls, [rustPlannedPassIds]);
    assert.deepEqual(engine.plannerCalls, [[styleSource, stylePath]]);
  });

  const bundlePassIdSearch = spawnSync("git", ["grep", "-n", "BUNDLE_PASS_IDS", "--", "packages"], {
    cwd: process.cwd(),
    encoding: "utf8",
  });
  assert.equal(
    bundlePassIdSearch.status,
    1,
    `BUNDLE_PASS_IDS must not survive in package surfaces:\n${bundlePassIdSearch.stdout}`,
  );

  console.log(
    `checked bundler adapter pass authority: plannedPassIds=${rustPlannedPassIds.join(",")}`,
  );
}

function readRustBundleSummary(stylePath: string, source: string) {
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
        stylePath,
        styleSource: source,
        targetLabel: "default",
        targetSupport: {
          vendorPrefixRequired: false,
          supportsLightDark: true,
          supportsColorMix: true,
          supportsOklchOklab: true,
          supportsColorFunction: true,
          supportsRelativeColor: true,
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

  assert.equal(result.status, 0, result.stderr);
  const summary = JSON.parse(result.stdout);
  assert.equal(summary.product, "omena-query.transform-plan");
  assert.equal(summary.bundle.product, "omena-transform-bundle.source");
  return summary.bundle;
}

async function withFixture(
  label: string,
  callback: (fixture: { readonly root: string; readonly stylePath: string }) => Promise<void>,
) {
  const root = mkdtempSync(path.join(tmpdir(), `omena-bundler-pass-authority-${label}-`));
  try {
    const rawStylePath = path.join(root, "Button.module.scss");
    writeFileSync(rawStylePath, styleSource, "utf8");
    const stylePath = realpathSync.native(rawStylePath);
    await callback({ root, stylePath });
  } finally {
    rmSync(root, { force: true, recursive: true });
  }
}

function createRecordingEngine(
  stylePath: string,
  bundleSummary: { readonly plannedPassIds: string[] },
) {
  const passIdCalls: string[][] = [];
  const plannerCalls: [string, string][] = [];

  return {
    passIdCalls,
    plannerCalls,
    bundlerHostCapabilitiesJson() {
      return JSON.stringify({
        protocolVersion: "0",
        capabilities: ["semanticClassMap", "namedExports", "composesEdges"],
      });
    },
    resolveCssModuleForBundlerHostJson(requestJson: string) {
      const request = JSON.parse(requestJson) as { snapshotId: unknown; stylePath: string };
      return JSON.stringify({
        snapshotId: request.snapshotId,
        protocolVersion: "0",
        moduleId: request.stylePath,
        classMap: { button: "_button_0" },
        namedExports: { button: "_button_0" },
        typescriptDeclaration:
          "declare const styles: Readonly<Record<string, string>>;\nexport default styles;\n",
        composesEdges: [],
        diagnostics: [],
        ready: true,
      });
    },
    summarizeTransformBundleFromSourceJson(source: string, pathFromAdapter: string) {
      plannerCalls.push([source, pathFromAdapter]);
      assert.equal(source, styleSource);
      assert.equal(pathFromAdapter, stylePath);
      return JSON.stringify(bundleSummary);
    },
    buildStyleSourcesWithContextJson(_targetPath: string, _sourcesJson: string, passIds: string[]) {
      passIdCalls.push(passIds);
      return JSON.stringify({
        execution: {
          outputCss: ".button{color:blue}",
          executedPassIds: passIds,
        },
        sourceMapV3: {
          version: 3,
          sources: [stylePath],
          names: [],
          mappings: "AAAA",
        },
        readySurfaces: ["sourceMapV3Serializer"],
      });
    },
    bundleStyleSourcesWithContextJson(
      _targetPath: string,
      _sourcesJson: string,
      passIds: string[],
    ) {
      passIdCalls.push(passIds);
      return JSON.stringify({
        schemaVersion: "0",
        product: "omena-query.bundle-artifact",
        stylePath,
        outputCss: ".button{color:blue}",
        bundle: bundleSummary,
        sourceMapV3: {
          version: 3,
          sources: [stylePath],
          names: [],
          mappings: "AAAA",
        },
        codeSplitOutputs: [],
        assetRewrites: [],
        perPassProvenance: [],
        execution: { outputCss: ".button{color:blue}", executedPassIds: passIds },
        readySurfaces: ["bundleOperationFacade"],
        ...closedWorldEvidence(stylePath),
      });
    },
  };
}

function closedWorldEvidence(stylePath: string) {
  return {
    closedWorldOutcome: { status: "closed", bundle: {} },
    closedWorldDecisionParity: {
      legacyOpenDecision: false,
      typedOutcomeOpen: false,
      equivalent: true,
    },
    evidence: {
      schemaVersion: "0",
      product: "omena-query.bundle-evidence",
      stylePath,
      outcomeStatus: "closed",
      reachability: null,
      gates: [{ name: "closedWorldAdmission", passed: true }],
      blockers: [],
      interfaceHashes: [],
      sourcePrecision: null,
    },
  };
}
