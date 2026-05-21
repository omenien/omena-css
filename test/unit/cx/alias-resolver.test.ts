import { describe, expect, it } from "vitest";
import * as path from "node:path";
import * as fs from "node:fs";
import * as os from "node:os";
import {
  AliasResolver,
  AliasResolverHolder,
  loadWorkspaceBundlerPathAliases,
  loadWorkspaceTsconfigPathAliases,
} from "../../../server/engine-core-ts/src/core/cx/alias-resolver";

const WORKSPACE = "/fake/ws";

describe("AliasResolver", () => {
  it("empty config returns null for every specifier", () => {
    const r = new AliasResolver(WORKSPACE, {});
    expect(r.resolve("@styles/button")).toBeNull();
    expect(r.resolve("anything")).toBeNull();
  });

  it("resolves `@styles/button` with a relative target", () => {
    const r = new AliasResolver(WORKSPACE, { "@styles": "src/styles" });
    expect(r.resolve("@styles/button")).toBe(path.resolve(WORKSPACE, "src/styles/button"));
  });

  it("passes absolute targets through untouched", () => {
    const r = new AliasResolver(WORKSPACE, { "@shared": "/Users/me/shared" });
    expect(r.resolve("@shared/x")).toBe("/Users/me/shared/x");
  });

  it("substitutes ${workspaceFolder} at construction time", () => {
    const r = new AliasResolver(WORKSPACE, { "@s": "${workspaceFolder}/src" });
    expect(r.resolve("@s/x")).toBe(path.resolve(WORKSPACE, "src/x"));
  });

  it("longest-prefix match: @styles wins over @", () => {
    const r = new AliasResolver(WORKSPACE, {
      "@": "src",
      "@styles": "src/styles",
    });
    expect(r.resolve("@styles/button")).toBe(path.resolve(WORKSPACE, "src/styles/button"));
  });

  it("longest-prefix tie-break: lexical order among same-length prefixes", () => {
    const r = new AliasResolver(WORKSPACE, { "@a": "x", "@b": "y" });
    expect(r.resolve("@a/foo")).toBe(path.resolve(WORKSPACE, "x/foo"));
    expect(r.resolve("@b/foo")).toBe(path.resolve(WORKSPACE, "y/foo"));
  });

  it("exact-prefix match (specifier === prefix)", () => {
    const r = new AliasResolver(WORKSPACE, {
      "@styles": "src/index.module.scss",
    });
    expect(r.resolve("@styles")).toBe(path.resolve(WORKSPACE, "src/index.module.scss"));
  });

  it("trailing slash in prefix is normalized", () => {
    const r = new AliasResolver(WORKSPACE, { "@styles/": "src/styles" });
    expect(r.resolve("@styles/button")).toBe(path.resolve(WORKSPACE, "src/styles/button"));
  });

  it("non-matching specifier returns null", () => {
    const r = new AliasResolver(WORKSPACE, { "@styles": "src/styles" });
    expect(r.resolve("lodash")).toBeNull();
  });

  it("longest-prefix wins over generic prefixes even when key order is broad-first", () => {
    // Config listed in {"@", "@styles"} order: insertion-order
    // matching would pick `@` for `@styles/button`, but the resolver
    // intentionally uses the most specific prefix. A bare `@/button`
    // specifier still routes through the `@` prefix after normalization.
    const r = new AliasResolver(WORKSPACE, {
      "@": "src",
      "@styles": "src/styles",
    });
    expect(r.resolve("@styles/button")).toBe(path.resolve(WORKSPACE, "src/styles/button"));
    expect(r.resolve("@/button")).toBe(path.resolve(WORKSPACE, "src/button"));
  });

  it("relative target without ${workspaceFolder} resolves against workspace root", () => {
    const r = new AliasResolver(WORKSPACE, { "@s": "src/styles" });
    // Equivalent to the commented plan behavior — workspace-relative default.
    expect(r.resolve("@s/button")).toBe(path.resolve(WORKSPACE, "src/styles/button"));
  });

  it("loads tsconfig wildcard paths and resolves them against baseUrl", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "ts-path-alias-"));
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: "./src",
            paths: {
              "$components/*": ["components/*"],
              $styles: ["styles/Button.module.scss"],
            },
          },
        },
        null,
        2,
      ),
    );

    const tsconfigPaths = loadWorkspaceTsconfigPathAliases(workspace);
    expect(tsconfigPaths).not.toBeNull();

    const resolver = new AliasResolver(workspace, {}, tsconfigPaths);
    expect(resolver.resolve("$components/Button.module.scss")).toBe(
      path.resolve(workspace, "src/components/Button.module.scss"),
    );
    expect(resolver.resolve("$styles")).toBe(
      path.resolve(workspace, "src/styles/Button.module.scss"),
    );
  });

  it("inherits tsconfig path aliases through extends using the base config directory", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "ts-extends-alias-"));
    const configDir = path.join(workspace, "configs");
    fs.mkdirSync(configDir, { recursive: true });
    fs.writeFileSync(
      path.join(configDir, "tsconfig.base.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "$shared/*": ["src/shared/*"],
            },
          },
        },
        null,
        2,
      ),
    );
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify(
        {
          extends: "./configs/tsconfig.base.json",
        },
        null,
        2,
      ),
    );

    const resolver = new AliasResolver(workspace, {}, loadWorkspaceTsconfigPathAliases(workspace));
    expect(resolver.resolve("$shared/Button.module.scss")).toBe(
      path.resolve(configDir, "src/shared/Button.module.scss"),
    );
  });

  it("uses child tsconfig path aliases instead of inherited aliases when paths is overridden", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "ts-extends-overrides-"));
    const configDir = path.join(workspace, "configs");
    fs.mkdirSync(configDir, { recursive: true });
    fs.writeFileSync(
      path.join(configDir, "tsconfig.base.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "$base/*": ["src/base/*"],
              "$shared/*": ["src/shared/*"],
            },
          },
        },
        null,
        2,
      ),
    );
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify(
        {
          extends: "./configs/tsconfig.base.json",
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "$shared/*": ["src/local-shared/*"],
            },
          },
        },
        null,
        2,
      ),
    );

    const resolver = new AliasResolver(workspace, {}, loadWorkspaceTsconfigPathAliases(workspace));
    expect(resolver.resolve("$shared/Button.module.scss")).toBe(
      path.resolve(workspace, "src/local-shared/Button.module.scss"),
    );
    expect(resolver.resolve("$base/Button.module.scss")).toBeNull();
  });

  it("falls back to jsconfig.json when tsconfig.json is absent", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "js-path-alias-"));
    fs.writeFileSync(
      path.join(workspace, "jsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            paths: {
              "$components/*": ["src/components/*"],
            },
          },
        },
        null,
        2,
      ),
    );

    const resolver = new AliasResolver(workspace, {}, loadWorkspaceTsconfigPathAliases(workspace));
    expect(resolver.resolve("$components/Button.module.scss")).toBe(
      path.resolve(workspace, "src/components/Button.module.scss"),
    );
  });

  it("uses the nearest package tsconfig when the workspace root has no tsconfig", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "monorepo-path-alias-"));
    const packageRoot = path.join(workspace, "packages/core");
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(
      path.join(packageRoot, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "$components/*": ["src/components/*"],
            },
          },
        },
        null,
        2,
      ),
    );

    const resolver = new AliasResolver(workspace, {}, loadWorkspaceTsconfigPathAliases(workspace));
    expect(loadWorkspaceTsconfigPathAliases(workspace)).toBeNull();
    expect(
      resolver.resolve(
        "$components/Button.module.scss",
        undefined,
        path.join(packageRoot, "src/App.tsx"),
      ),
    ).toBe(path.join(packageRoot, "src/components/Button.module.scss"));
  });

  it("prefers explicit settings aliases over equal tsconfig patterns", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "settings-overrides-"));
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "@styles/*": ["from-tsconfig/*"],
            },
          },
        },
        null,
        2,
      ),
    );

    const resolver = new AliasResolver(
      workspace,
      { "@styles": "from-settings" },
      loadWorkspaceTsconfigPathAliases(workspace),
    );
    expect(resolver.resolve("@styles/Button.module.scss")).toBe(
      path.resolve(workspace, "from-settings/Button.module.scss"),
    );
  });

  it("loads literal Vite resolve.alias entries into the workspace alias holder", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vite-alias-"));
    fs.writeFileSync(
      path.join(workspace, "vite.config.ts"),
      `export default defineConfig({
  resolve: {
    alias: {
      "@styles": "./src/styles",
      "@components": path.resolve(__dirname, "src/components"),
    },
  },
});
`,
    );

    const aliases = loadWorkspaceBundlerPathAliases(workspace);
    expect(aliases?.aliases).toEqual({
      "@styles": path.resolve(workspace, "src/styles"),
      "@components": path.resolve(workspace, "src/components"),
    });
    expect(aliases?.unrecognized).toEqual([]);

    const holder = new AliasResolverHolder(workspace, {});
    expect(holder.get().resolve("@styles/Button.module.scss")).toBe(
      path.resolve(workspace, "src/styles/Button.module.scss"),
    );
  });

  it("loads literal webpack resolve.alias entries and reports unsupported dynamic entries", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "webpack-alias-"));
    fs.writeFileSync(
      path.join(workspace, "webpack.config.js"),
      `module.exports = {
  resolve: {
    alias: [
      { find: "@styles", replacement: "./src/styles" },
      { find: /^@theme/, replacement: "./src/theme" },
      { find: "@dynamic", replacement: makePath() },
    ],
  },
};
`,
    );

    const aliases = loadWorkspaceBundlerPathAliases(workspace);
    expect(aliases?.aliases).toEqual({
      "@styles": path.resolve(workspace, "src/styles"),
    });
    expect(aliases?.unrecognized.map((entry) => entry.reason)).toEqual([
      "regex-alias-find",
      "dynamic-alias-replacement",
    ]);
  });

  it("marks dynamic bundler config containers without guessing aliases", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "dynamic-bundler-alias-"));
    fs.writeFileSync(
      path.join(workspace, "vite.config.ts"),
      `export default defineConfig(() => ({
  resolve: { alias: { "@styles": "./src/styles" } },
}));
`,
    );
    fs.writeFileSync(
      path.join(workspace, "webpack.config.js"),
      `module.exports = {
  resolve: makeResolveConfig(),
};
`,
    );

    const aliases = loadWorkspaceBundlerPathAliases(workspace);
    expect(aliases?.aliases).toEqual({});
    expect(aliases?.unrecognized.map((entry) => entry.reason)).toEqual([
      "dynamic-alias-container",
      "dynamic-alias-container",
    ]);
  });

  it("prioritizes settings aliases over bundler aliases and bundler aliases over tsconfig paths", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "alias-priority-"));
    fs.writeFileSync(
      path.join(workspace, "vite.config.ts"),
      `export default { resolve: { alias: { "@styles": "./from-bundler" } } };`,
    );
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "@styles/*": ["from-tsconfig/*"],
            },
          },
        },
        null,
        2,
      ),
    );
    const bundlerAliases = loadWorkspaceBundlerPathAliases(workspace);
    const tsconfigPaths = loadWorkspaceTsconfigPathAliases(workspace);

    const resolver = new AliasResolver(workspace, {}, tsconfigPaths, undefined, bundlerAliases);
    expect(resolver.resolve("@styles/Button.module.scss")).toBe(
      path.resolve(workspace, "from-bundler/Button.module.scss"),
    );

    const settingsResolver = new AliasResolver(
      workspace,
      { "@styles": "from-settings" },
      tsconfigPaths,
      undefined,
      bundlerAliases,
    );
    expect(settingsResolver.resolve("@styles/Button.module.scss")).toBe(
      path.resolve(workspace, "from-settings/Button.module.scss"),
    );
  });

  it("prefers the first existing tsconfig target when multiple candidates exist", () => {
    const resolver = new AliasResolver(
      WORKSPACE,
      {},
      {
        basePath: path.resolve(WORKSPACE, "src"),
        paths: {
          "$components/*": ["missing/*", "real/*"],
        },
      },
    );

    expect(
      resolver.resolve(
        "$components/Button.module.scss",
        (candidate) => candidate === path.resolve(WORKSPACE, "src/real/Button.module.scss"),
      ),
    ).toBe(path.resolve(WORKSPACE, "src/real/Button.module.scss"));
  });
});
