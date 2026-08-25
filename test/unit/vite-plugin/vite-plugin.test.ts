import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import type * as ChildProcess from "node:child_process";
import { afterEach, describe, expect, it, vi } from "vitest";

type ViteTransformResult = null | {
  readonly code: string;
  readonly map: unknown;
};

type OmenaVitePlugin = {
  readonly configResolved: (config: { readonly root: string; readonly command: string }) => void;
  readonly transform: (
    this: { readonly warn?: (message: string) => void },
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
    this: { readonly warn?: (message: string) => void },
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

function bundlerHostMock(
  classExports:
    | Readonly<Record<string, string>>
    | ((request: {
        readonly styleSources: readonly { readonly styleSource: string }[];
      }) => Readonly<Record<string, string>>),
  valueExports: Readonly<Record<string, string>> = {},
) {
  return {
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
          "declare const styles: Readonly<Record<string, string>>;\nexport default styles;\n",
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

afterEach(() => {
  vi.restoreAllMocks();
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/vite-plugin", () => {
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
