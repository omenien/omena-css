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
  readonly transform: (
    this: { readonly warn?: (message: string) => void },
    code: string,
    id: string,
  ) => Promise<ViteTransformResult>;
  readonly handleHotUpdate: (ctx: {
    readonly file: string;
    readonly modules: readonly unknown[];
    readonly server?: {
      readonly moduleGraph?: {
        readonly getModuleById?: (id: string) => unknown;
        readonly invalidateModule?: (mod: unknown) => void;
      };
    };
  }) => Promise<readonly unknown[] | undefined>;
  readonly load: (id: string) => Promise<string | null>;
  readonly resolveId: (id: string) => Promise<string | null>;
};

type OmenaPluginExports = {
  readonly MINIFY_PASS_IDS: readonly string[];
  readonly VIRTUAL_MODULE_ID: string;
  readonly omenaCss: (options?: Record<string, unknown>) => OmenaVitePlugin;
};

const require = createRequire(import.meta.url);
const { MINIFY_PASS_IDS, VIRTUAL_MODULE_ID, omenaCss } =
  require("../../../packages/vite-plugin/index.cjs") as OmenaPluginExports;

const tempRoots: string[] = [];

afterEach(() => {
  vi.restoreAllMocks();
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/vite-plugin", () => {
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
    const [targetPath, sourcesJson, passIds, contextJson, packageManifestsJson] = calls[0]!;
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
  });

  it("loads omena.config.json defaults while explicit options override them", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-config-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    fs.writeFileSync(stylePath, ".used { color: #ffffff; }");
    fs.writeFileSync(
      path.join(root, "omena.config.json"),
      JSON.stringify({ passes: ["url-quote-strip"], minify: true }),
    );
    const calls: unknown[][] = [];
    const engine = {
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
    expect(passIds).not.toContain("url-quote-strip");
    for (const minifyPass of MINIFY_PASS_IDS) {
      expect(passIds).toContain(minifyPass);
    }
  });

  it("invalidates changed style modules and keeps the latest rapid-edit result", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-hmr-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.scss");
    fs.writeFileSync(stylePath, ".used { color: red; }");
    const module = { id: stylePath };
    const invalidateModule = vi.fn();
    const engine = {
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
