import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { afterEach, describe, expect, it } from "vitest";

type OmenaBuildState = {
  readonly cache: Map<string, unknown>;
  readonly generations: Map<string, number>;
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
    readonly summary?: {
      readonly perPassProvenance?: readonly unknown[];
      readonly sourceMapV3?: unknown;
    };
  }>;
};

const require = createRequire(import.meta.url);
const { MINIFY_PASS_IDS, createOmenaBuildState, rebuildAndCache, resolveEffectiveOptions } =
  require("../../../packages/css-build-adapter/index.cjs") as AdapterExports;

const tempRoots: string[] = [];
const EXPECTED_MINIFY_PASS_IDS = [
  "comment-strip",
  "whitespace-strip",
  "number-compression",
  "color-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "calc-reduction",
  "print-css",
] as const;

afterEach(() => {
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/css-build-adapter", () => {
  it("keeps minify pass presets aligned across JS, CLI, and NAPI consumers", () => {
    expect(MINIFY_PASS_IDS).toEqual(EXPECTED_MINIFY_PASS_IDS);

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
    const cliSource = fs.readFileSync(
      path.join(process.cwd(), "rust/crates/omena-cli/src/build.rs"),
      "utf8",
    );
    expect(
      extractRustStringArray(napiSource, /fn minify_pass_ids\(\)[\s\S]*?\[([\s\S]*?)\]/),
    ).toEqual(EXPECTED_MINIFY_PASS_IDS);
    expect(
      extractRustStringArray(
        cliSource,
        /fn append_minify_build_passes[\s\S]*?for pass_id in \[([\s\S]*?)\]/,
      ),
    ).toEqual(EXPECTED_MINIFY_PASS_IDS);
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
          execution: {
            outputCss: ".button{color:blue}",
            executedPassIds: args[2],
            outcomes: [{ passId: "planner-import-inline", status: "applied" }],
          },
          readySurfaces: ["bundleOperationFacade"],
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
      },
    });

    expect(plannerCalls).toEqual([[source, stylePath]]);
    expect(buildCalls).toEqual([]);
    expect(bundleCalls[0]?.[2]).toEqual([
      "comment-strip",
      "planner-import-inline",
      "planner-scss-evaluate",
    ]);
    expect(bundleCalls[0]?.[5]).toEqual([]);
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
    expect(declaration).toContain("readonly sourceMapV3: OmenaSourceMapV3V0");
    expect(declaration).toContain("readonly summary: OmenaBundleArtifactV0");
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
});

function extractRustStringArray(source: string, pattern: RegExp) {
  const match = pattern.exec(source);
  if (!match) throw new Error(`unable to find Rust string array with ${pattern}`);
  return [...match[1]!.matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
