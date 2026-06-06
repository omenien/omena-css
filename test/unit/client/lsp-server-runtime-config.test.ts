import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  buildRustLspFileWatcherGlobs,
  buildThinClientDocumentSelector,
  buildThinClientRuntimeEndpoint,
  buildThinClientServerOptions,
  readClientLspServerRuntimeSetting,
  resolveLspServerRuntimeSelection,
  resolveOmenaLspServerPath,
} from "../../../client/src/lsp-server-runtime-config";

describe("client LSP server runtime config", () => {
  it("defaults invalid runtime settings to auto runtime selection", () => {
    expect(readClientLspServerRuntimeSetting("future")).toBe("auto");
    expect(readClientLspServerRuntimeSetting("node")).toBe("auto");
    expect(readClientLspServerRuntimeSetting(undefined)).toBe("auto");
  });

  it("does not silently fall back to the Node server for auto runtime selection without a Rust binary", () => {
    expect(() => resolveLspServerRuntimeSelection("auto", "/repo", {}, () => false)).toThrow(
      "omena.lspServerRuntime=auto requires an omena-lsp-server binary",
    );
  });

  it("resolves an explicit omena-lsp-server binary path", () => {
    const extensionRoot = path.resolve("/repo");
    const explicit = path.join(extensionRoot, "bin", "omena-lsp-server");

    expect(
      resolveOmenaLspServerPath(
        extensionRoot,
        { OMENA_LSP_SERVER_PATH: "bin/omena-lsp-server" },
        (candidate) => candidate === explicit,
      ),
    ).toBe(explicit);
  });

  it("resolves an explicit omena-lsp-server command without requiring a repo-local file", () => {
    expect(
      resolveOmenaLspServerPath(
        "/repo",
        { OMENA_LSP_SERVER_COMMAND: "omena-lsp-server" },
        () => false,
      ),
    ).toBe("omena-lsp-server");
  });

  it("throws when an explicit omena-lsp-server path is missing", () => {
    expect(() =>
      resolveOmenaLspServerPath(
        "/repo",
        { OMENA_LSP_SERVER_PATH: "missing/omena-lsp-server" },
        () => false,
      ),
    ).toThrow("OMENA_LSP_SERVER_PATH points to a missing binary");
  });

  it("selects the packaged omena-lsp-server binary when available", () => {
    const extensionRoot = path.resolve("/repo");
    const selected = resolveLspServerRuntimeSelection("auto", extensionRoot, {}, (candidate) =>
      candidate.includes(path.join("dist", "bin")),
    );

    expect(selected).toMatchObject({
      runtime: "omena-lsp-server",
      args: [],
    });
  });

  it("builds a thin client runtime endpoint for the Rust LSP runtime", () => {
    const endpoint = buildThinClientRuntimeEndpoint(
      {
        runtime: "omena-lsp-server",
        command: "/repo/dist/bin/darwin-arm64/omena-lsp-server",
        args: [],
      },
      "/repo",
    );

    expect(endpoint).toMatchObject({
      product: "omena-css.thin-client-runtime-endpoint",
      runtime: "omena-lsp-server",
      command: "/repo/dist/bin/darwin-arm64/omena-lsp-server",
      cwd: "/repo",
      nodeFallbackAllowed: false,
    });
    expect(endpoint?.fileWatcherGlobs).toEqual(buildRustLspFileWatcherGlobs());
    expect(endpoint?.hostResponsibilities).toContain("resolveStandaloneRustCommand");
    expect(endpoint?.hostResponsibilities).toContain("buildThinClientServerOptions");
    expect(endpoint?.hostResponsibilities).toContain("declareStaticDocumentSelector");
    expect(endpoint?.hostResponsibilities).toContain("startLanguageClient");
    expect(endpoint?.rustResponsibilities).toContain("ownTsgoClientLifecycle");
  });

  it("builds server options owned by the thin client host contract", () => {
    const endpoint = buildThinClientRuntimeEndpoint(
      {
        runtime: "omena-lsp-server",
        command: "/repo/dist/bin/darwin-arm64/omena-lsp-server",
        args: [],
      },
      "/repo",
    );
    const options = buildThinClientServerOptions(endpoint, {
      OMENA_TYPE_FACT_BACKEND: "tsgo",
    } as NodeJS.ProcessEnv);

    expect(options.run.command).toBe("/repo/dist/bin/darwin-arm64/omena-lsp-server");
    expect(options.run.options.cwd).toBe("/repo");
    expect(options.run.options.env.OMENA_TYPE_FACT_BACKEND).toBe("tsgo");
    expect(options.debug).toEqual(options.run);
  });

  it("builds the static Rust LSP document selector outside extension activation", () => {
    expect(buildThinClientDocumentSelector().map((item) => item.language)).toEqual([
      "typescriptreact",
      "javascriptreact",
      "typescript",
      "javascript",
      "vue",
      "html",
      "svelte",
      "astro",
      "markdown",
      "mdx",
      "scss",
      "less",
      "css",
    ]);
  });

  it("declares static file watchers for the Rust LSP runtime", () => {
    expect(buildRustLspFileWatcherGlobs()).toEqual([
      "**/*.module.{scss,css,less}",
      "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs,d.ts,vue,html,svelte,astro,md,mdx}",
      "**/tsconfig*.json",
      "**/jsconfig*.json",
      "**/package.json",
      "**/vite.config.{ts,mts,cts,js,mjs,cjs}",
      "**/webpack.config.{ts,mts,cts,js,mjs,cjs}",
    ]);
  });
});
