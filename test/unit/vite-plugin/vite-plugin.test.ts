import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { afterEach, describe, expect, it } from "vitest";

type ViteTransformResult = null | {
  readonly code: string;
  readonly map: unknown;
};

type OmenaVitePlugin = {
  readonly transform: (
    this: { readonly warn?: (message: string) => void },
    code: string,
    id: string,
  ) => ViteTransformResult;
};

const require = createRequire(import.meta.url);
const { omenaCss } = require("../../../packages/vite-plugin/index.cjs") as {
  readonly omenaCss: (options?: Record<string, unknown>) => OmenaVitePlugin;
};

const tempRoots: string[] = [];

afterEach(() => {
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/vite-plugin", () => {
  it("forwards bundle and tree-shake build options to omena build", () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const sourcePath = path.join(root, "tokens.module.css");
    const manifestPath = path.join(root, "package.json");
    const cliPath = path.join(root, "fake-omena-cli.cjs");
    const argsPath = path.join(root, "args.json");
    const source = ".used { color: blue; }";
    fs.writeFileSync(stylePath, source);
    fs.writeFileSync(sourcePath, ".token { color: red; }");
    fs.writeFileSync(manifestPath, "{}");
    fs.writeFileSync(
      cliPath,
      `#!/usr/bin/env node
const fs = require("node:fs");
const args = process.argv.slice(2);
fs.writeFileSync(process.env.OMENA_VITE_PLUGIN_ARGS_FILE, JSON.stringify(args));
process.stdout.write(JSON.stringify({
  execution: { outputCss: ".used{color:blue}" },
  sourceMapV3: { version: 3, sources: ["Button.module.css"], names: [], mappings: "" }
}));
`,
    );
    fs.chmodSync(cliPath, 0o755);

    const previousArgsFile = process.env.OMENA_VITE_PLUGIN_ARGS_FILE;
    process.env.OMENA_VITE_PLUGIN_ARGS_FILE = argsPath;
    try {
      const plugin = omenaCss({
        omenaBin: cliPath,
        passes: ["comment-strip"],
        treeShake: true,
        bundle: true,
        closedStyleWorld: true,
        sources: [sourcePath],
        packageManifests: [manifestPath],
      });

      const result = plugin.transform.call({}, source, stylePath);

      expect(result).toEqual({
        code: ".used{color:blue}",
        map: { version: 3, sources: ["Button.module.css"], names: [], mappings: "" },
      });
    } finally {
      if (previousArgsFile === undefined) {
        delete process.env.OMENA_VITE_PLUGIN_ARGS_FILE;
      } else {
        process.env.OMENA_VITE_PLUGIN_ARGS_FILE = previousArgsFile;
      }
    }

    const args = JSON.parse(fs.readFileSync(argsPath, "utf8")) as string[];
    expect(args).toContain("build");
    expect(args).toContain(stylePath);
    expect(args).toContain("--pass");
    expect(args).toContain("comment-strip");
    expect(args).toContain("--closed-style-world");
    expect(args).toContain("--tree-shake");
    expect(args).toContain("--bundle");
    expect(args).toContain("--source");
    expect(args).toContain(sourcePath);
    expect(args).toContain("--package-manifest");
    expect(args).toContain(manifestPath);
    expect(args).toContain("--json");
    expect(args).toContain("--source-map");
  });
});
