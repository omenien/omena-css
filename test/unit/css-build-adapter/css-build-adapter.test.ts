import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { afterEach, describe, expect, it } from "vitest";

type OmenaBuildState = {
  readonly cache: Map<string, unknown>;
  readonly generations: Map<string, number>;
  readonly cacheMetrics: {
    readonly hits: number;
    readonly misses: number;
    readonly bypasses: number;
    readonly builds: number;
  };
};

type CacheEntry = {
  readonly output: {
    readonly code: string;
  };
};

type BuildSource = {
  readonly stylePath: string;
  readonly styleSource: string;
};

type AdapterExports = {
  readonly MINIFY_PASS_IDS: readonly string[];
  readonly createOmenaBuildState: (options?: Record<string, unknown>) => OmenaBuildState;
  readonly resolveEffectiveOptions: (
    options: Record<string, unknown>,
    state: OmenaBuildState,
  ) => Promise<Record<string, unknown>>;
  readonly rebuildAndCache: (
    filePath: string,
    source: string,
    options: Record<string, unknown>,
    state: OmenaBuildState,
  ) => Promise<{
    readonly code: string;
    readonly classExports?: Readonly<Record<string, string>>;
    readonly valueExports?: Readonly<Record<string, string>>;
    readonly namedExports?: readonly {
      readonly exportedName: string;
      readonly kind: "class" | "value";
      readonly value: string;
    }[];
    readonly moduleInterface?: { readonly diagnostics: readonly { readonly code: string }[] };
    readonly summary?: {
      readonly perPassProvenance?: readonly unknown[];
      readonly executionScope?: unknown;
      readonly sourceMapV3?: unknown;
    };
  }>;
};

const require = createRequire(import.meta.url);
const { MINIFY_PASS_IDS, createOmenaBuildState, rebuildAndCache, resolveEffectiveOptions } =
  require("../../../packages/css-build-adapter/index.cjs") as AdapterExports;

const tempRoots: string[] = [];
const SEMANTIC_MINIFY_PASS_IDS = JSON.parse(
  fs.readFileSync(
    path.join(process.cwd(), "packages/css-build-adapter/semantic-minify-pass-ids.json"),
    "utf8",
  ),
) as readonly string[];

function bundlerHostMock(
  classExports: Readonly<Record<string, string>>,
  valueExports: Readonly<Record<string, string>> = {},
) {
  return {
    buildSnapshotIdentityJson: buildSnapshotIdentityJsonMock,
    bundlerHostCapabilitiesJson: () =>
      JSON.stringify({
        protocolVersion: "0",
        capabilities: [
          "semanticClassExports",
          "typedExportNamespaces",
          "namedExports",
          "composesEdges",
        ],
      }),
    resolveCssModuleForBundlerHostJson: (requestJson: string) => {
      const request = JSON.parse(requestJson) as { snapshotId: unknown; stylePath: string };
      return JSON.stringify({
        snapshotId: request.snapshotId,
        protocolVersion: "0",
        moduleId: request.stylePath,
        classExports,
        valueExports,
        namedExports: [
          ...Object.entries(classExports).map(([exportedName, value]) => ({
            exportedName,
            kind: "class",
            value,
          })),
          ...Object.entries(valueExports).map(([exportedName, value]) => ({
            exportedName,
            kind: "value",
            value,
          })),
        ],
        typescriptDeclaration:
          "declare const styles: Readonly<Record<string, string>>;\nexport default styles;\n",
        composesEdges: [],
        diagnostics: Object.keys(classExports)
          .filter((name) => Object.hasOwn(valueExports, name))
          .map((name) => ({
            code: "exportNamespaceCollision",
            message: `${name} is exported by both namespaces`,
            sourceAnchors: [],
          })),
        ready: true,
      });
    },
  };
}

function buildSnapshotIdentityJsonMock(inputJson: string) {
  return JSON.stringify({
    schemaVersion: "0",
    product: "omena-query.build-snapshot-digest",
    contentHashAlgorithm: "blake3",
    digest: `blake3:test-${inputJson}`,
  });
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

afterEach(() => {
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/css-build-adapter", () => {
  it("keeps one digest-gated adapter cache identity", () => {
    const adapterSource = fs.readFileSync(
      path.join(process.cwd(), "packages/css-build-adapter/index.cjs"),
      "utf8",
    );
    const rebuildBody = adapterSource.match(
      /async function rebuildAndCache[\s\S]*?(?=\nasync function runOmenaBuild)/u,
    )?.[0];

    expect(adapterSource.match(/\bcache:\s*new Map\(/gu)).toHaveLength(1);
    expect(adapterSource.match(/state\.cache\.set\(/gu)).toHaveLength(1);
    expect(adapterSource).not.toContain("buildCacheKey");
    expect(adapterSource).not.toContain("cacheKey");
    expect(rebuildBody).toContain("cached?.buildSnapshotDigest");
    expect(rebuildBody!.indexOf("resolveBuildSnapshotIdentity")).toBeLessThan(
      rebuildBody!.indexOf("state.cache.get"),
    );

    const nativeSource = fs.readFileSync(
      path.join(process.cwd(), "rust/crates/omena-query/src/build_snapshot.rs"),
      "utf8",
    );
    expect(nativeSource).toContain("compute_omena_sif_leaf_hash_v1");
    expect(nativeSource).toContain("omena_resolver_style_identity_generation()");
    expect(nativeSource).toContain("target_data_snapshot_id: target_plan.target_data_snapshot_id");
    expect(nativeSource).toContain('env!("CARGO_PKG_VERSION")');
    expect(nativeSource).toContain("pass_plan_digest");
  });

  it("keeps public minify presets pinned to the semantic profile authority", () => {
    expect(MINIFY_PASS_IDS).toEqual(SEMANTIC_MINIFY_PASS_IDS);

    const benchmarkScript = fs.readFileSync(
      path.join(process.cwd(), "scripts/benchmark-omena-vite-productization.mjs"),
      "utf8",
    );
    expect(benchmarkScript).toContain("packages/css-build-adapter/index.cjs");
    expect(benchmarkScript).not.toContain("const MINIFY_PASS_IDS = [");

    const napiSource = fs.readFileSync(
      path.join(process.cwd(), "rust/crates/omena-napi/src/lib.rs"),
      "utf8",
    );
    const minifyPresetFunction = napiSource.match(
      /fn minify_pass_ids\(\) -> Vec<String> \{([\s\S]*?)\n\}/,
    );
    expect(minifyPresetFunction?.[1]).toContain("semantic_omena_query_minify_build_profile()");
    expect(minifyPresetFunction?.[1]).not.toContain('"print-css"');
  });

  it("derives bundle pass ids from the engine planner", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-bundle-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.scss");
    const source = '@use "./tokens";\n.button { color: tokens.$brand; }';
    const buildCalls: unknown[][] = [];
    const bundleCalls: unknown[][] = [];
    const plannerCalls: unknown[][] = [];
    const engine = {
      ...bundlerHostMock({ button: "_button_0" }),
      summarizeTransformBundleFromSourceJson: (...args: unknown[]) => {
        plannerCalls.push(args);
        return JSON.stringify({
          plannedPassIds: ["planner-import-inline", "planner-scss-evaluate"],
        });
      },
      buildStyleSourcesWithContextJson: (...args: unknown[]) => {
        buildCalls.push(args);
        return JSON.stringify({
          execution: {
            outputCss: ".button{color:blue}",
            executedPassIds: args[2],
          },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
        });
      },
      bundleStyleSourcesWithContextJson: (...args: unknown[]) => {
        bundleCalls.push(args);
        return JSON.stringify({
          schemaVersion: "0",
          product: "omena-query.bundle-artifact",
          stylePath,
          outputCss: ".button{color:blue}",
          bundle: {
            product: "omena-transform-bundle.source",
            plannedPassIds: ["planner-import-inline", "planner-scss-evaluate"],
          },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
          codeSplitOutputs: [],
          assetRewrites: [],
          perPassProvenance: [{ passId: "planner-import-inline", status: "applied" }],
          executionScope: null,
          execution: {
            outputCss: ".button{color:blue}",
            executedPassIds: args[2],
            outcomes: [{ passId: "planner-import-inline", status: "applied" }],
          },
          readySurfaces: ["bundleOperationFacade"],
          ...closedWorldEvidence(stylePath),
        });
      },
    };
    const state = createOmenaBuildState({ cwd: root });

    await expect(
      rebuildAndCache(
        stylePath,
        source,
        {
          cwd: root,
          configFile: false,
          engine,
          bundle: true,
          passes: ["comment-strip"],
        },
        state,
      ),
    ).resolves.toMatchObject({
      code: ".button{color:blue}",
      summary: {
        product: "omena-query.bundle-artifact",
        perPassProvenance: [{ passId: "planner-import-inline", status: "applied" }],
        executionScope: null,
      },
    });

    expect(plannerCalls).toEqual([[source, stylePath]]);
    expect(buildCalls).toEqual([]);
    expect(bundleCalls[0]?.[2]).toEqual([
      "comment-strip",
      "planner-import-inline",
      "planner-scss-evaluate",
    ]);
    expect(JSON.parse(bundleCalls[0]?.[3] as string)).toMatchObject({
      classNameRewrites: [{ originalName: "button", rewrittenName: "_button_0" }],
    });
    expect(bundleCalls[0]?.[5]).toEqual([stylePath]);
  });

  it("builds CSS bytes with the selected module-interface token", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-interface-token-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const source = ".button { color: red; }";
    const observedContexts: unknown[] = [];
    const engine = {
      ...bundlerHostMock({ button: "_Ab1cdE_button _Zy9xW8_base" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (
        _targetPath: string,
        _sourcesJson: string,
        _passIds: string[],
        contextJson: string,
      ) => {
        const context = JSON.parse(contextJson) as {
          classNameRewrites: readonly { rewrittenName: string }[];
        };
        observedContexts.push(context);
        return JSON.stringify({
          execution: {
            outputCss: `.${context.classNameRewrites[0]?.rewrittenName}{color:red}`,
            executedPassIds: [],
          },
        });
      },
    };

    await expect(
      rebuildAndCache(
        stylePath,
        source,
        {
          cwd: root,
          configFile: false,
          engine,
          context: {
            classNameRewrites: [{ originalName: "button", rewrittenName: "_Cd2efG_button" }],
          },
        },
        createOmenaBuildState({ cwd: root }),
      ),
    ).resolves.toMatchObject({
      code: "._Cd2efG_button{color:red}",
      classExports: { button: "_Cd2efG_button _Zy9xW8_base" },
    });
    expect(observedContexts).toEqual([
      {
        classNameRewrites: [{ originalName: "button", rewrittenName: "_Cd2efG_button" }],
      },
    ]);
  });

  it("keeps same-named class and value exports separate from class rewrites", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-export-families-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const source = ":export { button: #0af; } .button { color: red; }";
    const observedContexts: unknown[] = [];
    const engine = {
      ...bundlerHostMock({ button: "_Ab1cdE_button" }, { button: "#0af" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (
        _targetPath: string,
        _sourcesJson: string,
        _passIds: string[],
        contextJson: string,
      ) => {
        observedContexts.push(JSON.parse(contextJson));
        return JSON.stringify({ execution: { outputCss: source, executedPassIds: [] } });
      },
    };

    const output = await rebuildAndCache(
      stylePath,
      source,
      { cwd: root, configFile: false, engine },
      createOmenaBuildState({ cwd: root }),
    );

    expect(output.classExports).toEqual({ button: "_Ab1cdE_button" });
    expect(output.valueExports).toEqual({ button: "#0af" });
    expect(output.namedExports).toEqual([
      { exportedName: "button", kind: "class", value: "_Ab1cdE_button" },
      { exportedName: "button", kind: "value", value: "#0af" },
    ]);
    expect(output.moduleInterface?.diagnostics).toEqual([
      expect.objectContaining({ code: "exportNamespaceCollision" }),
    ]);
    expect(observedContexts).toEqual([
      { classNameRewrites: [{ originalName: "button", rewrittenName: "_Ab1cdE_button" }] },
    ]);
  });

  it("rejects open bundle outcomes without returning partial CSS", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-open-bundle-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const source = ".button { color: red; }";
    const engine = {
      ...bundlerHostMock({ button: "_button_0" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: () =>
        JSON.stringify({ execution: { outputCss: "", executedPassIds: [] } }),
      bundleStyleSourcesWithContextJson: () =>
        JSON.stringify({
          schemaVersion: "0",
          product: "omena-query.bundle-artifact",
          stylePath,
          outputCss: "._button_0{color:red}",
          execution: { outputCss: "._button_0{color:red}", executedPassIds: [] },
          closedWorldOutcome: {
            status: "open",
            blockers: [
              {
                kind: "missingDependency",
                sourcePath: stylePath,
                importSource: "./Missing.module.css",
              },
            ],
          },
          closedWorldDecisionParity: {
            legacyOpenDecision: true,
            typedOutcomeOpen: true,
            equivalent: true,
          },
          evidence: {
            schemaVersion: "0",
            product: "omena-query.bundle-evidence",
            stylePath,
            outcomeStatus: "open",
            gates: [{ name: "closedWorldAdmission", passed: false }],
            blockers: [
              {
                kind: "missingDependency",
                sourcePath: stylePath,
                importSource: "./Missing.module.css",
              },
            ],
          },
        }),
    };

    await expect(
      rebuildAndCache(
        stylePath,
        source,
        { cwd: root, configFile: false, engine, bundle: true },
        createOmenaBuildState({ cwd: root }),
      ),
    ).rejects.toThrow(/closed-world bundle admission failed with typed blockers/u);
  });

  it("loads TOML build sections into effective adapter options", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-config-"));
    tempRoots.push(root);
    fs.writeFileSync(
      path.join(root, "omena.config.toml"),
      `
[build]
minify = true
source-map = false

[build.target-options]
enable-media-static-eval = true
`,
    );
    const state = createOmenaBuildState({ cwd: root });

    await expect(resolveEffectiveOptions({ cwd: root }, state)).resolves.toMatchObject({
      minify: true,
      sourceMap: false,
      targetOptions: {
        enableMediaStaticEval: true,
      },
    });
  });

  it("prefers canonical unified config while preserving legacy build-table semantics", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-unified-config-"));
    tempRoots.push(root);
    fs.writeFileSync(
      path.join(root, "omena.toml"),
      `
[workspace]
roots = ["packages/*"]

[lint]
profile = "recommended"

[build]
minify = true
source-map = true
`,
    );
    fs.writeFileSync(
      path.join(root, "omena.config.json"),
      JSON.stringify({ build: { minify: false, sourceMap: false } }),
    );

    const state = createOmenaBuildState({ cwd: root });
    await expect(resolveEffectiveOptions({ cwd: root }, state)).resolves.toMatchObject({
      minify: true,
      sourceMap: true,
    });
  });

  it("loads unified JSON build tables and keeps explicit options authoritative", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-unified-json-"));
    tempRoots.push(root);
    fs.writeFileSync(
      path.join(root, "omena.config.json"),
      JSON.stringify({
        lint: { profile: "recommended" },
        build: { minify: true, sourceMap: true },
      }),
    );

    const state = createOmenaBuildState({ cwd: root });
    await expect(
      resolveEffectiveOptions({ cwd: root, minify: false }, state),
    ).resolves.toMatchObject({ minify: false, sourceMap: true });
  });

  it("exposes typed bundle artifacts in the adapter declarations", () => {
    const declaration = fs.readFileSync(
      path.join(process.cwd(), "packages/css-build-adapter/index.d.ts"),
      "utf8",
    );

    expect(declaration).toContain("export interface OmenaBundleArtifactV0");
    expect(declaration).toContain("readonly perPassProvenance");
    expect(declaration).toContain(
      "readonly executionScope: OmenaBundleExecutionScopeEvidenceV0 | null",
    );
    expect(declaration).toContain("readonly sourceMapV3: OmenaSourceMapV3V0");
    expect(declaration).toContain("readonly summary: OmenaBundleWithEvidenceV0");
    expect(declaration).not.toContain("readonly summary: Record<string, unknown>");
    expect(declaration).not.toContain("readonly map: Record<string, unknown>");
  });

  it("keeps the latest Vite watcher generation in cache when earlier builds resolve last", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.scss");
    const state = createOmenaBuildState({ cwd: root });
    const releaseRedBuild = deferred<void>();
    const completedBuilds: string[] = [];
    const engine = {
      ...bundlerHostMock({ button: "_button_0" }),
      summarizeTransformBundleFromSourceJson: () =>
        JSON.stringify({ plannedPassIds: ["class-name-rewrite"] }),
      buildStyleSourcesWithContextJson: async (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        const color = source.styleSource.includes("red") ? "red" : "blue";
        if (color === "red") await releaseRedBuild.promise;
        completedBuilds.push(color);
        return JSON.stringify({
          execution: {
            outputCss: `.button{color:${color}}`,
            executedPassIds: ["comment-strip"],
          },
          sourceMapV3: { version: 3, sources: [source.stylePath], names: [], mappings: "AAAA" },
        });
      },
    };
    const options = {
      cwd: root,
      configFile: false,
      engine,
      passes: ["comment-strip"],
    };

    const redBuild = rebuildAndCache(stylePath, ".button { color: red; }", options, state);
    const blueBuild = rebuildAndCache(stylePath, ".button { color: blue; }", options, state);
    await expect(blueBuild).resolves.toMatchObject({ code: ".button{color:blue}" });

    releaseRedBuild.resolve();
    await expect(redBuild).resolves.toMatchObject({ code: ".button{color:red}" });

    const cacheEntry = state.cache.get(stylePath) as CacheEntry | undefined;
    expect(completedBuilds).toEqual(["blue", "red"]);
    expect(state.generations.get(stylePath)).toBe(2);
    expect(cacheEntry?.output.code).toBe(".button{color:blue}");
  });

  it("invalidates a cached build when an additional style source changes at the same path and mtime", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-source-identity-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "App.module.css");
    const dependencyPath = path.join(root, "tokens.module.css");
    const source = '@import "./tokens.module.css";\n.button { color: var(--brand); }';
    const fixedTimestamp = new Date("2026-01-02T03:04:05.000Z");
    const buildInputs: BuildSource[][] = [];
    const engine = {
      buildSnapshotIdentityJson: buildSnapshotIdentityJsonMock,
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const sources = JSON.parse(sourcesJson) as BuildSource[];
        buildInputs.push(sources);
        return JSON.stringify({
          execution: {
            outputCss: sources.map(({ styleSource }) => styleSource).join("\n"),
            executedPassIds: [],
          },
        });
      },
    };
    const options = {
      cwd: root,
      configFile: false,
      engine,
      moduleInterface: false,
      sourceMap: false,
      sources: [dependencyPath],
    };
    const state = createOmenaBuildState({ cwd: root });

    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(dependencyPath, ":root { --brand: red; }");
    fs.utimesSync(dependencyPath, fixedTimestamp, fixedTimestamp);
    const first = await rebuildAndCache(stylePath, source, options, state);

    fs.writeFileSync(dependencyPath, ":root { --brand: blue; }");
    fs.utimesSync(dependencyPath, fixedTimestamp, fixedTimestamp);
    const second = await rebuildAndCache(stylePath, source, options, state);
    const unchanged = await rebuildAndCache(stylePath, source, options, state);

    expect(fs.statSync(dependencyPath).mtimeMs).toBe(fixedTimestamp.getTime());
    expect(first.code).toContain("--brand: red");
    expect(second.code).toContain("--brand: blue");
    expect(unchanged.code).toBe(second.code);
    expect(buildInputs).toHaveLength(2);
    expect(state.cacheMetrics).toMatchObject({ hits: 1, misses: 2, bypasses: 0, builds: 2 });
  });

  it("invalidates a cached build when a package manifest changes at the same path and mtime", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-manifest-identity-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "App.module.css");
    const manifestPath = path.join(root, "node_modules/@design/tokens/package.json");
    const source = '@import "@design/tokens/theme";';
    const fixedTimestamp = new Date("2026-01-02T03:04:05.000Z");
    const buildManifests: string[][] = [];
    const engine = {
      buildSnapshotIdentityJson: buildSnapshotIdentityJsonMock,
      buildStyleSourcesWithContextJson: (
        _targetPath: string,
        _sourcesJson: string,
        _passIds: string[],
        _contextJson: string,
        packageManifestsJson: string,
      ) => {
        const manifests = JSON.parse(packageManifestsJson) as Array<{
          readonly packageJsonSource: string;
        }>;
        buildManifests.push(manifests.map(({ packageJsonSource }) => packageJsonSource));
        const exportTarget = JSON.parse(manifests[0]!.packageJsonSource).exports["./theme"].style;
        return JSON.stringify({
          execution: { outputCss: exportTarget, executedPassIds: [] },
        });
      },
    };
    const options = {
      cwd: root,
      configFile: false,
      engine,
      moduleInterface: false,
      packageManifests: [manifestPath],
      sourceMap: false,
    };
    const state = createOmenaBuildState({ cwd: root });

    fs.mkdirSync(path.dirname(manifestPath), { recursive: true });
    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(
      manifestPath,
      JSON.stringify({ exports: { "./theme": { style: "./dist/red.css" } } }),
    );
    fs.utimesSync(manifestPath, fixedTimestamp, fixedTimestamp);
    const first = await rebuildAndCache(stylePath, source, options, state);

    fs.writeFileSync(
      manifestPath,
      JSON.stringify({ exports: { "./theme": { style: "./dist/blue.css" } } }),
    );
    fs.utimesSync(manifestPath, fixedTimestamp, fixedTimestamp);
    const second = await rebuildAndCache(stylePath, source, options, state);
    const unchanged = await rebuildAndCache(stylePath, source, options, state);

    expect(fs.statSync(manifestPath).mtimeMs).toBe(fixedTimestamp.getTime());
    expect(first.code).toBe("./dist/red.css");
    expect(second.code).toBe("./dist/blue.css");
    expect(unchanged.code).toBe(second.code);
    expect(buildManifests).toHaveLength(2);
    expect(state.cacheMetrics).toMatchObject({ hits: 1, misses: 2, bypasses: 0, builds: 2 });
  });

  it("binds static config bytes and reloads same-path same-mtime edits", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-config-identity-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "App.module.css");
    const configPath = path.join(root, "omena.config.json");
    const source = ".button { color: red; }";
    const fixedTimestamp = new Date("2026-01-02T03:04:05.000Z");
    let buildCount = 0;
    const engine = {
      buildSnapshotIdentityJson: buildSnapshotIdentityJsonMock,
      buildStyleSourcesWithContextJson: () => {
        buildCount += 1;
        return JSON.stringify({ execution: { outputCss: source, executedPassIds: [] } });
      },
    };
    const explicitOptions = {
      cwd: root,
      engine,
      moduleInterface: false,
      sourceMap: false,
    };
    const state = createOmenaBuildState({ cwd: root });

    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(configPath, '{"build":{"sourceMap":false}}\n');
    fs.utimesSync(configPath, fixedTimestamp, fixedTimestamp);
    const firstOptions = await resolveEffectiveOptions(explicitOptions, state);
    await rebuildAndCache(stylePath, source, firstOptions, state);

    fs.writeFileSync(configPath, '{ "build": { "sourceMap": false } }\n');
    fs.utimesSync(configPath, fixedTimestamp, fixedTimestamp);
    const secondOptions = await resolveEffectiveOptions(explicitOptions, state);
    await rebuildAndCache(stylePath, source, secondOptions, state);
    const unchangedOptions = await resolveEffectiveOptions(explicitOptions, state);
    await rebuildAndCache(stylePath, source, unchangedOptions, state);

    expect(fs.statSync(configPath).mtimeMs).toBe(fixedTimestamp.getTime());
    expect(buildCount).toBe(2);
    expect(state.cacheMetrics).toMatchObject({ hits: 1, misses: 2, bypasses: 0, builds: 2 });
  });

  it("rebuilds instead of serving a hopeful hit when the engine cannot seal an identity", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-build-adapter-fail-closed-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "App.module.css");
    const source = ".button { color: red; }";
    let buildCount = 0;
    const engine = {
      buildStyleSourcesWithContextJson: () => {
        buildCount += 1;
        return JSON.stringify({ execution: { outputCss: source, executedPassIds: [] } });
      },
    };
    const options = {
      cwd: root,
      configFile: false,
      engine,
      moduleInterface: false,
      sourceMap: false,
    };
    const state = createOmenaBuildState({ cwd: root });

    await rebuildAndCache(stylePath, source, options, state);
    await rebuildAndCache(stylePath, source, options, state);

    expect(buildCount).toBe(2);
    expect(state.cacheMetrics).toMatchObject({ hits: 0, misses: 0, bypasses: 2, builds: 2 });
    expect(state.cache.get(stylePath)).toMatchObject({
      buildSnapshotDigest: null,
      cacheBypassReason: "engineMissingBuildSnapshotIdentity",
    });
  });
});

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
