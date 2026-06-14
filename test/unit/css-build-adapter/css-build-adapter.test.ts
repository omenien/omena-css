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
  readonly createOmenaBuildState: (options?: Record<string, unknown>) => OmenaBuildState;
  readonly rebuildAndCache: (
    filePath: string,
    source: string,
    options: Record<string, unknown>,
    state: OmenaBuildState,
  ) => Promise<{
    readonly code: string;
  }>;
};

const require = createRequire(import.meta.url);
const { createOmenaBuildState, rebuildAndCache } =
  require("../../../packages/css-build-adapter/index.cjs") as AdapterExports;

const tempRoots: string[] = [];

afterEach(() => {
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/css-build-adapter", () => {
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

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
