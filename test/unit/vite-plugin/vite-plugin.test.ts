import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import type * as ChildProcess from "node:child_process";
import { afterEach, describe, expect, it, vi } from "vitest";

type ViteTransformResult = null | {
  readonly code: string;
  readonly map: unknown;
};

type SourceMapV3 = {
  readonly version: 3;
  readonly file?: string;
  readonly sources: readonly string[];
  readonly sourcesContent?: readonly (string | null)[];
  readonly names: readonly string[];
  readonly mappings: string;
};

type BuildSource = {
  readonly stylePath: string;
  readonly styleSource: string;
};

type OmenaVitePlugin = {
  readonly configResolved: (config: { readonly root: string; readonly command: string }) => void;
  readonly transform: (
    this: {
      readonly warn?: (message: string) => void;
      readonly addWatchFile?: (file: string) => void;
      readonly getCombinedSourcemap?: () => SourceMapV3;
    },
    code: string,
    id: string,
  ) => Promise<ViteTransformResult>;
  readonly handleHotUpdate: (ctx: {
    readonly file: string;
    readonly modules: readonly unknown[];
    readonly server?: {
      readonly ws?: { readonly send?: (payload: unknown) => void };
      readonly moduleGraph?: {
        readonly getModuleById?: (id: string) => unknown;
        readonly invalidateModule?: (mod: unknown) => void;
      };
    };
  }) => Promise<readonly unknown[] | undefined>;
  readonly load: (
    this: {
      readonly warn?: (message: string) => void;
      readonly addWatchFile?: (file: string) => void;
    },
    id: string,
  ) => Promise<string | ViteTransformResult | null>;
  readonly resolveId: (id: string) => Promise<string | null>;
};

type OmenaPluginExports = {
  readonly MINIFY_PASS_IDS: readonly string[];
  readonly VIRTUAL_MODULE_ID: string;
  readonly classifyCssModuleExportDelta: (
    previousClassMap: Readonly<Record<string, string>> | undefined,
    nextClassMap: Readonly<Record<string, string>> | undefined,
  ) => "styleOnly" | "valueChanged" | "shapeChanged";
  readonly omenaCss: (options?: Record<string, unknown>) => OmenaVitePlugin;
};

const require = createRequire(import.meta.url);
const { MINIFY_PASS_IDS, VIRTUAL_MODULE_ID, classifyCssModuleExportDelta, omenaCss } =
  require("../../../packages/vite-plugin/index.cjs") as OmenaPluginExports;

const tempRoots: string[] = [];
const tempFiles: string[] = [];
const SOURCE_ADMISSION_INSPECTION = Symbol.for("omena-css.vite.source-admission-inspection");

type ResolvedTransformPlugin = {
  readonly name: string;
  readonly transform?:
    | ((this: unknown, code: string, id: string) => unknown)
    | { readonly handler: (this: unknown, code: string, id: string) => unknown };
};

type SourceAdmissionInspection = {
  readonly included: boolean;
  readonly provenanceClass: "disk-backed" | "virtual-with-map" | "virtual-only" | "not-included";
};

type SourceAdmissionInspectionApi = {
  readonly inspect: (input: {
    readonly code: string;
    readonly id: string;
    readonly diskSource: string;
    readonly upstreamMap: SourceMapV3 | null;
  }) => Promise<SourceAdmissionInspection>;
};

type PreChainResult = {
  readonly code: string;
  readonly upstreamMap: SourceMapV3 | null;
  readonly upstreamTransforms: readonly string[];
};

async function measureVirtualSourceAdmissionCensus(
  configFile = path.join(process.cwd(), "examples/vite.config.ts"),
) {
  const { resolveConfig } = await import("vite");
  const examplesRoot = path.join(process.cwd(), "examples");
  const config = await resolveConfig(
    { configFile, logLevel: "silent", root: examplesRoot },
    "build",
    "production",
  );
  const plugins = config.plugins as unknown as ResolvedTransformPlugin[];
  const omenaIndex = plugins.findIndex((plugin) =>
    Reflect.has(plugin as object, SOURCE_ADMISSION_INSPECTION),
  );
  if (omenaIndex < 0) throw new Error("Resolved examples config has no Omena admission inspector");
  const omenaPlugin = plugins[omenaIndex]!;
  const inspectionApi = Reflect.get(
    omenaPlugin as object,
    SOURCE_ADMISSION_INSPECTION,
  ) as SourceAdmissionInspectionApi;
  const upstreamPlugins = plugins.slice(0, omenaIndex);
  const rows = [];

  for (const relativePath of listTrackedStyleModuleInputs()) {
    const fileId = path.join(process.cwd(), relativePath);
    const diskSource = fs.readFileSync(fileId, "utf8");
    const transformed = await runResolvedPreChain(upstreamPlugins, diskSource, fileId);
    const inspection = await inspectionApi.inspect({
      code: transformed.code,
      id: fileId,
      diskSource,
      upstreamMap: transformed.upstreamMap,
    });
    rows.push({
      input: relativePath,
      population: relativePath.startsWith("examples/") ? "examples" : "real-project-corpus",
      included: inspection.included,
      provenanceClass: inspection.provenanceClass,
      admission: inspection.included
        ? inspection.provenanceClass === "disk-backed"
          ? "existing"
          : "newly-admitted"
        : "not-included",
      upstreamTransforms: transformed.upstreamTransforms,
    });
  }

  const count = (predicate: (row: (typeof rows)[number]) => boolean) =>
    rows.filter(predicate).length;
  return {
    schemaVersion: "1",
    product: "omena-vite.virtual-source-admission-census",
    package: "@omena/vite-plugin",
    countUnit: "inputs",
    semverIntent: "next-pre-1.0-minor",
    laneScope: {
      build: "resolved Vite pre-transform chain",
      serveDevRuntime: "disk file",
    },
    rows,
    totals: {
      trackedInputs: rows.length,
      included: count((row) => row.included),
      notIncluded: count((row) => !row.included),
      existing: count((row) => row.admission === "existing"),
      newlyAdmitted: count((row) => row.admission === "newly-admitted"),
      byProvenance: {
        diskBacked: count((row) => row.provenanceClass === "disk-backed"),
        virtualWithMap: count((row) => row.provenanceClass === "virtual-with-map"),
        virtualOnly: count((row) => row.provenanceClass === "virtual-only"),
      },
    },
  };
}

async function runResolvedPreChain(
  plugins: readonly ResolvedTransformPlugin[],
  diskSource: string,
  fileId: string,
): Promise<PreChainResult> {
  let code = diskSource;
  let upstreamMap: SourceMapV3 | null = null;
  const upstreamTransforms: string[] = [];
  for (const plugin of plugins) {
    const hook =
      typeof plugin.transform === "function" ? plugin.transform : plugin.transform?.handler;
    if (!hook) continue;
    const result = (await hook.call({}, code, fileId)) as
      | string
      | { readonly code?: string; readonly map?: SourceMapV3 | null }
      | null
      | undefined;
    if (result == null) continue;
    const nextCode = typeof result === "string" ? result : (result.code ?? code);
    const nextMap = typeof result === "string" ? null : (result.map ?? null);
    if (nextCode !== code || nextMap != null) upstreamTransforms.push(plugin.name);
    code = nextCode;
    if (nextMap != null) upstreamMap = nextMap;
  }
  return { code, upstreamMap, upstreamTransforms };
}

function listTrackedStyleModuleInputs(): string[] {
  return gitOutput(["ls-files", "-z", "--", "examples", "test/_fixtures/real-project-corpus"])
    .split("\0")
    .filter((file) => /\.module\.(?:css|less|scss)$/u.test(file))
    .toSorted();
}

function writeExamplesConfigVariant(source: string): string {
  const configPath = path.join(
    process.cwd(),
    "examples",
    `.vite-admission-${process.pid}-${Math.random().toString(16).slice(2)}.config.ts`,
  );
  fs.writeFileSync(configPath, source);
  tempFiles.push(configPath);
  return configPath;
}

function readPublishedVirtualSourceAdmissionCensus(): unknown {
  const readme = fs.readFileSync(
    path.join(process.cwd(), "packages/vite-plugin/README.md"),
    "utf8",
  );
  const match = readme.match(
    /<!-- omena-vite-virtual-source-admission-census:start -->\n\n```json\n([\s\S]*?)\n```\n\n<!-- omena-vite-virtual-source-admission-census:end -->/u,
  );
  if (!match?.[1]) throw new Error("Published Vite virtual-source admission census is missing");
  return JSON.parse(match[1]);
}

function gitOutput(args: readonly string[]): string {
  return execFileSync("git", args, {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function resolveCommit(ref: string): string {
  return gitOutput(["rev-parse", "--verify", `${ref}^{commit}`]).trim();
}

function listStyleModulesAt(ref: string): string[] {
  return gitOutput([
    "ls-tree",
    "-r",
    "--name-only",
    ref,
    "--",
    "examples",
    "test/_fixtures/real-project-corpus",
  ])
    .split("\n")
    .filter((file) => /\.module\.(?:css|less|scss)$/u.test(file))
    .toSorted();
}

function readFileAt(ref: string, relativePath: string): string {
  return gitOutput(["show", `${ref}:${relativePath}`]);
}

function evaluateCommonJs(
  source: string,
  filename: string,
  localRequire: (specifier: string) => unknown,
): unknown {
  const commonJsModule: { exports: unknown } = { exports: {} };
  const evaluate = new Function(
    "exports",
    "require",
    "module",
    "__filename",
    "__dirname",
    source,
  ) as (...args: unknown[]) => void;
  evaluate(commonJsModule.exports, localRequire, commonJsModule, filename, path.dirname(filename));
  return commonJsModule.exports;
}

function loadPluginAt(ref: string): (options?: Record<string, unknown>) => OmenaVitePlugin {
  const adapterFilename = path.join(process.cwd(), "packages/css-build-adapter/index.cjs");
  const pluginFilename = path.join(process.cwd(), "packages/vite-plugin/index.cjs");
  const semanticPassIds = JSON.parse(
    readFileAt(ref, "packages/css-build-adapter/semantic-minify-pass-ids.json"),
  );
  const adapter = evaluateCommonJs(
    readFileAt(ref, "packages/css-build-adapter/index.cjs"),
    adapterFilename,
    (specifier) => {
      if (specifier === "./semantic-minify-pass-ids.json") return semanticPassIds;
      return require(specifier);
    },
  );
  const plugin = evaluateCommonJs(
    readFileAt(ref, "packages/vite-plugin/index.cjs"),
    pluginFilename,
    (specifier) => {
      if (specifier === "@omena/css-build-adapter") return adapter;
      return require(specifier);
    },
  ) as { readonly omenaCss: (options?: Record<string, unknown>) => OmenaVitePlugin };
  return plugin.omenaCss;
}

function diskByteProbeBuildSnapshotIdentity() {
  return {
    schemaVersion: "0",
    product: "omena-query.build-snapshot-digest",
    contentHashAlgorithm: "blake3",
    digest: "blake3:test-probe-snapshot",
    targetSourceDigest: "blake3:test-probe-target-source",
  };
}

function diskByteProbeEngine(withBuildSnapshotIdentity = true) {
  const engine = {
    buildStyleSourcesWithContextJson(targetPath: string, sourcesJson: string) {
      const [source] = JSON.parse(sourcesJson) as BuildSource[];
      return JSON.stringify({
        execution: {
          outputCss: `${source!.styleSource}\n/* deterministic disk-backed probe */\n`,
          executedPassIds: [],
        },
        sourceMapV3: {
          version: 3,
          sources: [targetPath],
          names: [],
          mappings: "AAAA",
        },
      });
    },
  };
  return withBuildSnapshotIdentity
    ? { ...engine, buildSnapshotIdentity: diskByteProbeBuildSnapshotIdentity }
    : engine;
}

function byteOutputEngine(withBuildSnapshotIdentity: boolean) {
  const buildStyleSourcesWithContextJson = (_targetPath: string, sourcesJson: string) => {
    const [source] = JSON.parse(sourcesJson) as BuildSource[];
    return JSON.stringify({
      execution: {
        outputCss: source!.styleSource.replace(/\s+/gu, "").replace("}", "}\n"),
        executedPassIds: [],
      },
    });
  };
  if (!withBuildSnapshotIdentity) return { buildStyleSourcesWithContextJson };
  return {
    buildSnapshotIdentity: diskByteProbeBuildSnapshotIdentity,
    buildStyleSourcesWithContextJson,
  };
}

async function transformDiskBytes(
  createPlugin: (options?: Record<string, unknown>) => OmenaVitePlugin,
  filePath: string,
  source: string,
  withBuildSnapshotIdentity: boolean,
): Promise<Buffer> {
  const warnings: string[] = [];
  const plugin = createPlugin({
    configFile: false,
    cwd: process.cwd(),
    engine: diskByteProbeEngine(withBuildSnapshotIdentity),
    moduleInterface: false,
    passes: [],
  });
  const result = await plugin.transform.call(
    { warn: (message) => warnings.push(String(message)) },
    source,
    filePath,
  );
  expect(warnings).toEqual([]);
  expect(result).not.toBeNull();
  return Buffer.from(`${JSON.stringify({ code: result!.code, map: result!.map })}\n`, "utf8");
}

function bundlerHostMock(
  classExports:
    | Readonly<Record<string, string>>
    | ((request: {
        readonly styleSources: readonly { readonly styleSource: string }[];
      }) => Readonly<Record<string, string>>),
  valueExports: Readonly<Record<string, string>> = {},
) {
  return {
    buildSnapshotIdentity: (input: unknown) => {
      const request = input as {
        readonly targetPath?: string;
        readonly styleSources?: readonly {
          readonly stylePath: string;
          readonly styleSource: string;
        }[];
      };
      const targetSource = request.styleSources?.find(
        ({ stylePath }) => stylePath === request.targetPath,
      )?.styleSource;
      return {
        schemaVersion: "0",
        product: "omena-query.build-snapshot-digest",
        contentHashAlgorithm: "blake3",
        digest: `blake3:test-${JSON.stringify(input)}`,
        targetSourceDigest: `blake3:test-source-${JSON.stringify(targetSource)}`,
      };
    },
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
      const request = JSON.parse(requestJson) as {
        snapshotId: unknown;
        stylePath: string;
        styleSources: readonly { readonly styleSource: string }[];
      };
      const resolvedClassExports =
        typeof classExports === "function" ? classExports(request) : classExports;
      return JSON.stringify({
        snapshotId: request.snapshotId,
        protocolVersion: "0",
        moduleId: request.stylePath,
        classExports: resolvedClassExports,
        valueExports,
        namedExports: [
          ...Object.entries(resolvedClassExports).map(([exportedName, value]) => ({
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
          "export declare const classExports: Readonly<Record<string, string>>;\nexport declare const valueExports: Readonly<Record<string, string>>;\ndeclare const styles: typeof classExports;\nexport default styles;\n",
        composesEdges: [],
        diagnostics: Object.keys(resolvedClassExports)
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

async function readBuildSummary(plugin: OmenaVitePlugin) {
  const resolvedId = await plugin.resolveId(VIRTUAL_MODULE_ID);
  const loaded = await plugin.load(resolvedId!);
  if (typeof loaded !== "string" || !loaded.startsWith("export default ")) {
    throw new Error(`Expected build summary module, got ${JSON.stringify(loaded)}`);
  }
  return JSON.parse(loaded.slice("export default ".length, -2)) as Array<{
    readonly filePath: string;
    readonly sourceProvenance?: {
      readonly classification: "disk-backed" | "virtual-with-map" | "virtual-only";
      readonly diskDigest: string | null;
      readonly inputDigest: string | null;
      readonly reason: string | null;
      readonly upstreamMapPresent: boolean;
      readonly upstreamMapSources: readonly string[];
    };
  }>;
}

afterEach(() => {
  vi.restoreAllMocks();
  for (const file of tempFiles.splice(0)) {
    fs.rmSync(file, { force: true });
  }
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/vite-plugin", () => {
  it("pins the reviewed virtual-source admission census", async () => {
    const measured = await measureVirtualSourceAdmissionCensus();
    if (process.env.OMENA_VITE_PRINT_ADMISSION_CENSUS === "1") {
      process.stdout.write(`OMENA_VITE_ADMISSION_CENSUS=${JSON.stringify(measured, null, 2)}\n`);
    }
    expect(readPublishedVirtualSourceAdmissionCensus()).toEqual(measured);
  });

  it("detects a differently named upstream source rewriter by resolved behavior", async () => {
    const original = fs.readFileSync(path.join(process.cwd(), "examples/vite.config.ts"), "utf8");
    const rewriter = [
      "function alternateSourceRewriter() {",
      "  return {",
      '    name: "examples-alternate-source-rewriter",',
      '    enforce: "pre" as const,',
      "    transform(code: string, id: string) {",
      "      if (!/\\.module\\.scss(?:\\?|$)/u.test(id)) return null;",
      "      return { code: `${code}\\n:root { --alternate-source-rewriter: active; }\\n`, map: null };",
      "    },",
      "  };",
      "}",
      "",
    ].join("\n");
    const mutated = original
      .replace(
        "function assertVirtualSourceComposition() {",
        `${rewriter}function assertVirtualSourceComposition() {`,
      )
      .replace("    omenaCss({", "    alternateSourceRewriter(),\n    omenaCss({");
    expect(mutated).not.toBe(original);

    const measured = await measureVirtualSourceAdmissionCensus(writeExamplesConfigVariant(mutated));

    expect(measured).not.toEqual(readPublishedVirtualSourceAdmissionCensus());
    expect(
      measured.rows.some(({ upstreamTransforms }) =>
        upstreamTransforms.includes("examples-alternate-source-rewriter"),
      ),
    ).toBe(true);
  });

  it("keeps the admission sweep stable across pure config re-indentation", async () => {
    const original = fs.readFileSync(path.join(process.cwd(), "examples/vite.config.ts"), "utf8");
    const reindented = original.replace(
      "    upstreamVirtualSource(),",
      "          upstreamVirtualSource(),",
    );
    expect(reindented).not.toBe(original);

    const [baseline, measured] = await Promise.all([
      measureVirtualSourceAdmissionCensus(),
      measureVirtualSourceAdmissionCensus(writeExamplesConfigVariant(reindented)),
    ]);

    expect(measured).toEqual(baseline);
    expect(measured).toEqual(readPublishedVirtualSourceAdmissionCensus());
  });

  const diskByteBaseline = process.env.OMENA_VITE_DISK_BYTE_BASELINE;
  const diskByteIdentityTest = diskByteBaseline ? it : it.skip;
  diskByteIdentityTest("preserves disk-backed output bytes across git pins", async () => {
    const baseline = resolveCommit(diskByteBaseline!);
    const current = resolveCommit("HEAD");
    const baselineFiles = listStyleModulesAt(baseline);
    const currentFiles = listStyleModulesAt(current);
    const baselineSet = new Set(baselineFiles);
    const currentSet = new Set(currentFiles);
    const removed = baselineFiles.filter((file) => !currentSet.has(file));
    const added = currentFiles.filter((file) => !baselineSet.has(file));

    expect(removed).toEqual([]);
    const baselinePlugin = loadPluginAt(baseline);
    const currentPlugin = loadPluginAt(current);
    for (const withBuildSnapshotIdentity of [true, false]) {
      await Promise.all(
        baselineFiles.map(async (relativePath) => {
          const filePath = path.join(process.cwd(), relativePath);
          const source = fs.readFileSync(filePath, "utf8");
          expect(source).toBe(readFileAt(current, relativePath));
          const baselineBytes = await transformDiskBytes(
            baselinePlugin,
            filePath,
            source,
            withBuildSnapshotIdentity,
          );
          const currentBytes = await transformDiskBytes(
            currentPlugin,
            filePath,
            source,
            withBuildSnapshotIdentity,
          );
          expect(
            currentBytes.equals(baselineBytes),
            `${relativePath} engine=${withBuildSnapshotIdentity ? "build-snapshot" : "fallback"}`,
          ).toBe(true);
        }),
      );
    }
    process.stdout.write(
      `Vite disk-backed byte identity: compared=${baselineFiles.length} identical=${baselineFiles.length} engineClasses=2 added=${added.length} removed=${removed.length} baseline=${baseline} current=${current}\n`,
    );
  });

  it("classifies semantic export deltas into three hot-update decisions", () => {
    expect(classifyCssModuleExportDelta({ root: "_root_0" }, { root: "_root_0" })).toBe(
      "styleOnly",
    );
    expect(classifyCssModuleExportDelta({ root: "_root_a" }, { root: "_root_b" })).toBe(
      "valueChanged",
    );
    expect(
      classifyCssModuleExportDelta({ root: "_root_0" }, { root: "_root_0", icon: "_icon_1" }),
    ).toBe("shapeChanged");
  });

  it("builds through the in-process napi-compatible engine without spawning", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const sourcePath = path.join(root, "tokens.module.css");
    const manifestPath = path.join(root, "package.json");
    const source = ".used { color: blue; }";
    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(sourcePath, ".token { color: red; }");
    fs.writeFileSync(manifestPath, "{}");
    const childProcess = require("node:child_process") as typeof ChildProcess;
    const execFileSyncSpy = vi.spyOn(childProcess, "execFileSync");
    const calls: unknown[][] = [];
    const engine = {
      ...bundlerHostMock({ used: "_used_0" }),
      summarizeTransformBundleFromSourceJson: () =>
        JSON.stringify({
          plannedPassIds: ["import-inline", "scss-module-evaluate", "composes-resolution"],
        }),
      buildStyleSourcesWithContextJson: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          execution: { outputCss: ".used{color:blue}", executedPassIds: ["comment-strip"] },
          sourceMapV3: {
            version: 3,
            sources: [stylePath, sourcePath],
            names: [],
            mappings: "AAAA",
          },
          readySurfaces: ["sourceMapV3Serializer"],
        });
      },
      bundleStyleSourcesWithContextJson: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          schemaVersion: "0",
          product: "omena-query.bundle-artifact",
          stylePath,
          outputCss: ".used{color:blue}",
          sourceMapV3: {
            version: 3,
            sources: [stylePath, sourcePath],
            names: [],
            mappings: "AAAA",
          },
          execution: { outputCss: ".used{color:blue}", executedPassIds: ["comment-strip"] },
          ...closedWorldEvidence(stylePath),
        });
      },
    };

    const plugin = omenaCss({
      engine,
      passes: ["comment-strip"],
      treeShake: true,
      bundle: true,
      closedStyleWorld: true,
      sources: [sourcePath],
      packageManifests: [manifestPath],
      cwd: root,
      configFile: false,
    });

    const result = await plugin.transform.call({}, source, stylePath);

    expect(result).toEqual({
      code: ".used{color:blue}",
      map: { version: 3, sources: [stylePath, sourcePath], names: [], mappings: "AAAA" },
    });
    expect(execFileSyncSpy).not.toHaveBeenCalled();
    expect(calls).toHaveLength(1);
    const [
      targetPath,
      sourcesJson,
      passIds,
      contextJson,
      packageManifestsJson,
      bundleEntryStylePaths,
    ] = calls[0]!;
    expect(targetPath).toBe(stylePath);
    expect(JSON.parse(sourcesJson as string)).toEqual([
      { stylePath, styleSource: source },
      { stylePath: sourcePath, styleSource: ".token { color: red; }" },
    ]);
    expect(passIds).toEqual(
      expect.arrayContaining([
        "comment-strip",
        "tree-shake-class",
        "import-inline",
        "scss-module-evaluate",
        "composes-resolution",
      ]),
    );
    expect(JSON.parse(contextJson as string)).toMatchObject({ closedStyleWorld: true });
    expect(JSON.parse(packageManifestsJson as string)).toEqual([
      { packageJsonPath: manifestPath, packageJsonSource: "{}" },
    ]);
    expect(bundleEntryStylePaths).toEqual([stylePath]);
    expect(calls[0]).toHaveLength(6);
  });

  it("loads omena.config.json defaults while explicit options override them", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-config-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    fs.writeFileSync(stylePath, ".used { color: #ffffff; }");
    fs.writeFileSync(
      path.join(root, "omena.config.json"),
      JSON.stringify({ passes: ["import-inline"], minify: true }),
    );
    const calls: unknown[][] = [];
    const engine = {
      ...bundlerHostMock({ used: "_used_0" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          execution: { outputCss: ".used{color:#fff}", executedPassIds: [] },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
        });
      },
    };

    const plugin = omenaCss({
      cwd: root,
      engine,
      passes: ["color-compression"],
    });
    await plugin.transform.call({}, fs.readFileSync(stylePath, "utf8"), stylePath);

    const passIds = calls[0]![2] as string[];
    expect(passIds).toContain("color-compression");
    expect(passIds).not.toContain("import-inline");
    for (const minifyPass of MINIFY_PASS_IDS) {
      expect(passIds).toContain(minifyPass);
    }
  });

  it("builds disk-backed bytes when the engine cannot seal a build-snapshot identity", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-no-snapshot-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    fs.writeFileSync(stylePath, diskSource);
    const plugin = omenaCss({
      cwd: root,
      configFile: false,
      engine: byteOutputEngine(false),
      moduleInterface: false,
      sourceMap: false,
    });

    const result = await plugin.transform.call({}, diskSource, stylePath);

    expect(Buffer.from(result!.code)).toEqual(Buffer.from(".root{color:red;}\n"));
    expect(result?.map).toBeNull();
    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "disk-backed",
      diskDigest: null,
      inputDigest: null,
      reason: "engineMissingBuildSnapshotIdentity",
    });
  });

  it("builds disk-backed bytes with a dynamic omena.config.cjs", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-dynamic-config-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    fs.writeFileSync(stylePath, diskSource);
    fs.writeFileSync(path.join(root, "omena.config.cjs"), "module.exports = { passes: [] };\n");
    const plugin = omenaCss({
      cwd: root,
      engine: byteOutputEngine(true),
      moduleInterface: false,
      sourceMap: false,
    });

    const result = await plugin.transform.call({}, diskSource, stylePath);

    expect(Buffer.from(result!.code)).toEqual(Buffer.from(".root{color:red;}\n"));
    expect(result?.map).toBeNull();
    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "disk-backed",
      diskDigest: null,
      inputDigest: null,
      reason: "dynamicBuildConfigurationDependencies",
    });
  });

  it("analyzes an upstream virtual source by default and records mapped provenance", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-virtual-map-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    const marker = ":root { --upstream-virtual-marker: active; }\n";
    const virtualSource = `${marker}${diskSource}`;
    const buildInputs: string[] = [];
    fs.writeFileSync(stylePath, diskSource);
    const host = bundlerHostMock({ root: "root-token" });
    const buildSnapshotIdentity = vi.fn(host.buildSnapshotIdentity);
    const engine = {
      ...host,
      buildSnapshotIdentity,
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        buildInputs.push(source!.styleSource);
        return JSON.stringify({
          execution: {
            outputCss: `${source!.styleSource}:root { --omena-output: analyzed; }\n`,
            executedPassIds: [],
          },
        });
      },
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });
    const result = await plugin.transform.call(
      {
        getCombinedSourcemap: () => ({
          version: 3,
          file: stylePath,
          sources: [stylePath],
          sourcesContent: [diskSource],
          names: [],
          mappings: ";AAAA",
        }),
      },
      virtualSource,
      stylePath,
    );

    expect(result?.code).toContain("--upstream-virtual-marker: active");
    expect(buildInputs).toEqual([virtualSource]);
    expect(buildSnapshotIdentity).toHaveBeenCalledTimes(2);
    expect(buildSnapshotIdentity.mock.calls[1]?.[0]).toMatchObject({
      adapterEnvironment: {
        sourceProvenance: {
          classification: "virtual-with-map",
          upstreamMapPresent: true,
          upstreamMap: { mappings: ";AAAA", sourcesContent: [diskSource] },
        },
      },
    });
    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "virtual-with-map",
      upstreamMapPresent: true,
      upstreamMapSources: [stylePath],
    });
    expect(summary?.sourceProvenance?.diskDigest).toMatch(/^blake3:/u);
    expect(summary?.sourceProvenance?.inputDigest).toMatch(/^blake3:/u);
    expect(summary?.sourceProvenance?.inputDigest).not.toBe(summary?.sourceProvenance?.diskDigest);
  });

  it("analyzes a virtual-only source when no disk mapping is available", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-virtual-only-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    const virtualSource = `:root { --virtual-only-marker: active; }\n${diskSource}`;
    fs.writeFileSync(stylePath, diskSource);
    const engine = {
      ...bundlerHostMock({ root: "root-token" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        return JSON.stringify({
          execution: {
            outputCss: `${source!.styleSource}:root { --omena-output: analyzed; }\n`,
            executedPassIds: [],
          },
        });
      },
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });

    const result = await plugin.transform.call({}, virtualSource, stylePath);

    expect(result?.code).toContain("--virtual-only-marker: active");
    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "virtual-only",
      upstreamMapPresent: false,
      upstreamMapSources: [],
    });
    expect(summary?.sourceProvenance?.diskDigest).toMatch(/^blake3:/u);
  });

  it("rejects a foreign mapped source even when its sourcesContent matches disk", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-foreign-map-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const foreignPath = path.join(root, "Foreign.module.css");
    const diskSource = ".root { color: red; }\n";
    const virtualSource = `:root { --foreign-map-marker: active; }\n${diskSource}`;
    fs.writeFileSync(stylePath, diskSource);
    const plugin = omenaCss({
      cwd: root,
      engine: {
        ...bundlerHostMock({ root: "root-token" }),
        summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
        buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
          const [source] = JSON.parse(sourcesJson) as BuildSource[];
          return JSON.stringify({
            execution: { outputCss: `${source!.styleSource}/* analyzed */\n`, executedPassIds: [] },
          });
        },
      },
      configFile: false,
    });

    await plugin.transform.call(
      {
        getCombinedSourcemap: () => ({
          version: 3,
          file: stylePath,
          sources: [stylePath, foreignPath],
          sourcesContent: [diskSource, diskSource],
          names: [],
          mappings: ";ACAA",
        }),
      },
      virtualSource,
      stylePath,
    );

    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "virtual-only",
      upstreamMapPresent: false,
      upstreamMapSources: [],
    });
  });

  it("rejects an identity source map as upstream rewrite provenance", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-identity-map-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    const virtualSource = `${diskSource}/* virtual tail */\n`;
    fs.writeFileSync(stylePath, diskSource);
    const plugin = omenaCss({
      cwd: root,
      engine: {
        ...bundlerHostMock({ root: "root-token" }),
        summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
        buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
          const [source] = JSON.parse(sourcesJson) as BuildSource[];
          return JSON.stringify({
            execution: { outputCss: `${source!.styleSource}/* analyzed */\n`, executedPassIds: [] },
          });
        },
      },
      configFile: false,
    });

    await plugin.transform.call(
      {
        getCombinedSourcemap: () => ({
          version: 3,
          file: stylePath,
          sources: [stylePath],
          sourcesContent: [diskSource],
          names: [],
          mappings: "AAAA",
        }),
      },
      virtualSource,
      stylePath,
    );

    const [summary] = await readBuildSummary(plugin);
    expect(summary?.sourceProvenance).toMatchObject({
      classification: "virtual-only",
      upstreamMapPresent: false,
      upstreamMapSources: [],
    });
  });

  it("retains disk-source matching as an explicit strict opt-in", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-strict-disk-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    const virtualSource = `:root { --strict-marker: active; }\n${diskSource}`;
    const warn = vi.fn();
    const build = vi.fn(() =>
      JSON.stringify({ execution: { outputCss: virtualSource, executedPassIds: [] } }),
    );
    fs.writeFileSync(stylePath, diskSource);
    const plugin = omenaCss({
      cwd: root,
      configFile: false,
      requireDiskSource: true,
      engine: {
        ...bundlerHostMock({ root: "root-token" }),
        summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
        buildStyleSourcesWithContextJson: build,
      },
    });

    await expect(plugin.transform.call({ warn }, virtualSource, stylePath)).resolves.toBeNull();
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("strict disk-source mode"));
    expect(build).not.toHaveBeenCalled();
  });

  it("pins build composition and serve dev-runtime disk input as distinct lanes", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-lane-scope-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const diskSource = ".root { color: red; }\n";
    const virtualSource = `:root { --build-lane-marker: active; }\n${diskSource}`;
    const buildInputs: string[] = [];
    fs.writeFileSync(stylePath, diskSource);
    const engine = {
      ...bundlerHostMock({ root: "root-token" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        buildInputs.push(source!.styleSource);
        return JSON.stringify({
          execution: {
            outputCss: `${source!.styleSource}/* analyzed */\n`,
            executedPassIds: [],
          },
        });
      },
    };
    const servePlugin = omenaCss({ cwd: root, configFile: false, engine });
    servePlugin.configResolved({ root, command: "serve" });
    const runtimeId = await servePlugin.resolveId(stylePath);
    const runtimeModule = await servePlugin.load(runtimeId!);

    expect((runtimeModule as ViteTransformResult)?.code).not.toContain("--build-lane-marker");

    const buildPlugin = omenaCss({ cwd: root, configFile: false, engine });
    buildPlugin.configResolved({ root, command: "build" });
    const buildResult = await buildPlugin.transform.call(
      {
        getCombinedSourcemap: () => ({
          version: 3,
          file: stylePath,
          sources: [stylePath],
          sourcesContent: [diskSource],
          names: [],
          mappings: ";AAAA",
        }),
      },
      virtualSource,
      stylePath,
    );

    expect(buildInputs).toEqual([diskSource, virtualSource]);
    expect(buildResult?.code).toContain("--build-lane-marker");
  });

  it("pushes value changes and invalidates only the runtime dependency set on shape changes", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-export-delta-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    fs.writeFileSync(stylePath, ".used { color: red; }");
    const runtimeImporter = { id: path.join(root, "App.tsx") };
    const runtimeModule = { id: "runtime", importers: new Set([runtimeImporter]) };
    const invalidateModule = vi.fn();
    const send = vi.fn();
    const engine = {
      ...bundlerHostMock((request) => {
        const source = request.styleSources[0]?.styleSource ?? "";
        if (source.includes("shape")) return { used: "_used_blue", icon: "_icon_1" };
        return { used: source.includes("blue") ? "_used_blue" : "_used_red" };
      }),
      summarizeTransformBundleFromSourceJson: () =>
        JSON.stringify({ plannedPassIds: ["class-name-rewrite"] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as Array<{ styleSource: string }>;
        return JSON.stringify({
          execution: { outputCss: source!.styleSource, executedPassIds: ["class-name-rewrite"] },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
        });
      },
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });
    plugin.configResolved({ root, command: "serve" });
    const runtimeId = await plugin.resolveId(stylePath);
    expect(runtimeId).toBeTruthy();
    const initialModule = await plugin.load(runtimeId!);
    expect((initialModule as ViteTransformResult)?.code).toContain(
      'let used = classExports["used"]',
    );

    const ctx = {
      file: stylePath,
      modules: [],
      server: {
        ws: { send },
        moduleGraph: {
          getModuleById: () => runtimeModule,
          invalidateModule,
        },
      },
    };
    fs.writeFileSync(stylePath, ".used { color: blue; }");
    await expect(plugin.handleHotUpdate(ctx)).resolves.toEqual([]);
    expect(send).toHaveBeenLastCalledWith(
      expect.objectContaining({
        data: expect.objectContaining({
          decision: "valueChanged",
          classExports: { used: "_used_blue" },
          valueExports: {},
        }),
      }),
    );
    expect(invalidateModule).not.toHaveBeenCalled();

    fs.writeFileSync(stylePath, ".used { color: shape; } .icon { display: block; }");
    await expect(plugin.handleHotUpdate(ctx)).resolves.toEqual([runtimeModule, runtimeImporter]);
    expect(invalidateModule).toHaveBeenCalledWith(runtimeModule);
    expect(invalidateModule).toHaveBeenCalledWith(runtimeImporter);
  });

  it("invalidates the transitive runtime importer closure on shape changes", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-export-closure-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    fs.writeFileSync(stylePath, ".used { color: red; }");
    const thirdHop = { id: path.join(root, "entry.ts") };
    const secondHop = { id: path.join(root, "middle.ts"), importers: new Set([thirdHop]) };
    const firstHop = { id: path.join(root, "leaf.ts"), importers: new Set([secondHop]) };
    const runtimeModule = { id: "runtime", importers: new Set([firstHop]) };
    const invalidateModule = vi.fn();
    const engine = {
      ...bundlerHostMock((request) => {
        const source = request.styleSources[0]?.styleSource ?? "";
        return source.includes("shape")
          ? { used: "used-token", added: "added-token" }
          : { used: "used-token" };
      }),
      summarizeTransformBundleFromSourceJson: () =>
        JSON.stringify({ plannedPassIds: ["class-name-rewrite"] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        return JSON.stringify({
          execution: { outputCss: source!.styleSource, executedPassIds: ["class-name-rewrite"] },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
        });
      },
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });
    plugin.configResolved({ root, command: "serve" });
    const runtimeId = await plugin.resolveId(stylePath);
    await plugin.load(runtimeId!);
    fs.writeFileSync(stylePath, ".used { color: shape; } .added { display: block; }");

    const invalidated = await plugin.handleHotUpdate({
      file: stylePath,
      modules: [],
      server: {
        moduleGraph: {
          getModuleById: () => runtimeModule,
          invalidateModule,
        },
      },
    });

    expect(invalidated).toEqual([runtimeModule, firstHop, secondHop, thirdHop]);
    expect(invalidateModule.mock.calls.map(([module]) => module)).toEqual([
      runtimeModule,
      firstHop,
      secondHop,
      thirdHop,
    ]);
  });

  it("exposes colliding class and value exports through distinct runtime families", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-export-families-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const source = ":export { button: #0af; } .button { color: red; }";
    fs.writeFileSync(stylePath, source);
    const warn = vi.fn();
    const engine = {
      ...bundlerHostMock({ button: "_Ab1cdE_button" }, { button: "#0af" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: () =>
        JSON.stringify({ execution: { outputCss: source, executedPassIds: [] } }),
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });
    plugin.configResolved({ root, command: "serve" });

    const runtimeId = await plugin.resolveId(stylePath);
    expect(runtimeId).toBeTruthy();
    const loaded = await plugin.load.call({ warn }, runtimeId!);
    const code = (loaded as ViteTransformResult)?.code ?? "";

    expect(code).toContain('const classExports = {"button":"_Ab1cdE_button"}');
    expect(code).toContain('const valueExports = {"button":"#0af"}');
    expect(code).not.toContain("let button =");
    expect(code).toContain("export default classExports");
    expect(code).toContain("export { classExports, valueExports }");
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("exportNamespaceCollision"));
  });

  it("binds noncolliding named exports to their runtime family", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-value-family-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Theme.module.css");
    const source = ":export { themeColor: #0af; spacing: 8px; } .button { color: red; }";
    fs.writeFileSync(stylePath, source);
    const engine = {
      ...bundlerHostMock({ button: "_Ab1cdE_button" }, { spacing: "8px", themeColor: "#0af" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: () =>
        JSON.stringify({ execution: { outputCss: source, executedPassIds: [] } }),
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false });
    plugin.configResolved({ root, command: "serve" });

    const runtimeId = await plugin.resolveId(stylePath);
    expect(runtimeId).toBeTruthy();
    const loaded = await plugin.load(runtimeId!);
    const code = (loaded as ViteTransformResult)?.code ?? "";

    expect(code).toContain('let button = classExports["button"]');
    expect(code).toContain('let spacing = valueExports["spacing"]');
    expect(code).toContain('let themeColor = valueExports["themeColor"]');
    expect(code).toContain('    button = classExports["button"]');
    expect(code).toContain('    spacing = valueExports["spacing"]');
    expect(code).toContain('    themeColor = valueExports["themeColor"]');
    expect(code).toContain("export default classExports;");
    expect(code).not.toContain("export default { ...classExports, ...valueExports };");
  });

  it("watches additional sources and rebuilds their cached target on dependency edits", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-dependency-hmr-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "App.module.scss");
    const dependencyPath = path.join(root, "tokens.module.scss");
    const source = '@use "./tokens.module.scss";\n.button { color: tokens.$brand; }';
    const fixedTimestamp = new Date("2026-01-02T03:04:05.000Z");
    const addWatchFile = vi.fn();
    const send = vi.fn();
    const buildSources: BuildSource[][] = [];
    const engine = {
      ...bundlerHostMock({ button: "_button_0" }),
      summarizeTransformBundleFromSourceJson: () =>
        JSON.stringify({ plannedPassIds: ["scss-module-evaluate"] }),
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const sources = JSON.parse(sourcesJson) as BuildSource[];
        buildSources.push(sources);
        const dependency = sources.find(({ stylePath }) => stylePath === dependencyPath);
        return JSON.stringify({
          execution: {
            outputCss: `.button{color:${dependency?.styleSource.includes("blue") ? "blue" : "red"}}`,
            executedPassIds: ["scss-module-evaluate"],
          },
        });
      },
    };
    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(dependencyPath, "$brand: red;");
    fs.utimesSync(dependencyPath, fixedTimestamp, fixedTimestamp);

    const plugin = omenaCss({
      cwd: root,
      configFile: false,
      engine,
      sources: [dependencyPath],
    });
    plugin.configResolved({ root, command: "serve" });
    const runtimeId = await plugin.resolveId(stylePath);
    await plugin.load.call({ addWatchFile }, runtimeId!);
    expect(addWatchFile).toHaveBeenCalledWith(fs.realpathSync.native(dependencyPath));

    fs.writeFileSync(dependencyPath, "$brand: blue;");
    fs.utimesSync(dependencyPath, fixedTimestamp, fixedTimestamp);
    await expect(
      plugin.handleHotUpdate({
        file: dependencyPath,
        modules: [],
        server: { ws: { send }, moduleGraph: { getModuleById: () => ({ id: runtimeId }) } },
      }),
    ).resolves.toEqual([]);

    expect(buildSources).toHaveLength(2);
    expect(buildSources[1]).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ stylePath: dependencyPath, styleSource: "$brand: blue;" }),
      ]),
    );
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        event: expect.stringContaining("omena-css:update:"),
        data: expect.objectContaining({ css: ".button{color:blue}" }),
      }),
    );
  });

  it("invalidates changed style modules and keeps the latest rapid-edit result", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-hmr-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.scss");
    fs.writeFileSync(stylePath, ".used { color: red; }");
    const module = { id: stylePath };
    const invalidateModule = vi.fn();
    const engine = {
      ...bundlerHostMock({ used: "_used_0" }),
      summarizeTransformBundleFromSourceJson: () => JSON.stringify({ plannedPassIds: [] }),
      buildStyleSourcesWithContextJson: async (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as Array<{ styleSource: string }>;
        if (source.styleSource.includes("red")) {
          await new Promise((resolve) => setTimeout(resolve, 20));
        }
        return JSON.stringify({
          execution: {
            outputCss: source.styleSource
              .replace(/\s+/g, "")
              .replace("red", "built-red")
              .replace("blue", "built-blue"),
            executedPassIds: [],
          },
          sourceMapV3: { version: 3, sources: [stylePath], names: [], mappings: "AAAA" },
        });
      },
    };
    const plugin = omenaCss({ cwd: root, engine, configFile: false, requireDiskSource: false });
    const ctx = {
      file: stylePath,
      modules: [module],
      server: { moduleGraph: { invalidateModule } },
    };

    const firstUpdate = plugin.handleHotUpdate(ctx);
    fs.writeFileSync(stylePath, ".used { color: blue; }");
    const secondUpdate = plugin.handleHotUpdate(ctx);

    await Promise.all([firstUpdate, secondUpdate]);
    const transformed = await plugin.transform.call(
      {},
      fs.readFileSync(stylePath, "utf8"),
      stylePath,
    );

    expect(invalidateModule).toHaveBeenCalledWith(module);
    expect(transformed?.code).toContain("built-blue");
    expect(transformed?.code).not.toContain("built-red");
    const resolvedId = await plugin.resolveId(VIRTUAL_MODULE_ID);
    expect(resolvedId).toBe(`\0${VIRTUAL_MODULE_ID}`);
    const virtualModule = await plugin.load(resolvedId!);
    expect(virtualModule).toContain(stylePath);
  });
});
