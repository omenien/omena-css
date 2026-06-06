import { existsSync } from "node:fs";
import path from "node:path";
import type { Executable } from "vscode-languageclient/node";
import type { DocumentSelector } from "vscode-languageserver-protocol";

export type ClientLspServerRuntimeSetting = "auto" | "omena-lsp-server";

export interface OmenaLspServerRuntimeSelection {
  readonly runtime: "omena-lsp-server";
  readonly command: string;
  readonly args: readonly string[];
}

export type LspServerRuntimeSelection = OmenaLspServerRuntimeSelection;

export interface ThinClientRuntimeEndpoint {
  readonly product: "omena-css.thin-client-runtime-endpoint";
  readonly runtime: "omena-lsp-server";
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly fileWatcherGlobs: readonly string[];
  readonly nodeFallbackAllowed: false;
  readonly hostResponsibilities: readonly string[];
  readonly rustResponsibilities: readonly string[];
}

export interface ThinClientServerOptions {
  readonly run: Executable;
  readonly debug: Executable;
}

export function buildRustLspFileWatcherGlobs(): readonly string[] {
  return [
    "**/*.module.{scss,css,less}",
    "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs,d.ts,vue,html,svelte,astro,md,mdx}",
    "**/tsconfig*.json",
    "**/jsconfig*.json",
    "**/package.json",
    "**/vite.config.{ts,mts,cts,js,mjs,cjs}",
    "**/webpack.config.{ts,mts,cts,js,mjs,cjs}",
  ];
}

export function buildThinClientServerOptions(
  endpoint: ThinClientRuntimeEndpoint,
  serverEnv: NodeJS.ProcessEnv,
): ThinClientServerOptions {
  const command: Executable = {
    command: endpoint.command,
    args: [...endpoint.args],
    options: { env: serverEnv, cwd: endpoint.cwd },
  };
  return {
    run: command,
    debug: command,
  };
}

export function buildThinClientDocumentSelector(): DocumentSelector {
  return [
    { scheme: "file", language: "typescriptreact" },
    { scheme: "file", language: "javascriptreact" },
    { scheme: "file", language: "typescript" },
    { scheme: "file", language: "javascript" },
    { scheme: "file", language: "vue" },
    { scheme: "file", language: "html" },
    { scheme: "file", language: "svelte" },
    { scheme: "file", language: "astro" },
    { scheme: "file", language: "markdown" },
    { scheme: "file", language: "mdx" },
    { scheme: "file", language: "scss" },
    { scheme: "file", language: "less" },
    { scheme: "file", language: "css" },
  ];
}

export function readClientLspServerRuntimeSetting(value: unknown): ClientLspServerRuntimeSetting {
  if (value === "omena-lsp-server") return "omena-lsp-server";
  return "auto";
}

export function resolveLspServerRuntimeSelection(
  runtime: ClientLspServerRuntimeSetting,
  extensionRoot: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (path: string) => boolean = existsSync,
): LspServerRuntimeSelection {
  const command = resolveOmenaLspServerPath(extensionRoot, env, fileExists);
  if (!command) {
    throw new Error(
      [
        `omena.lspServerRuntime=${runtime} requires an omena-lsp-server binary.`,
        "Run pnpm build, set OMENA_LSP_SERVER_PATH to an explicit binary, or set OMENA_LSP_SERVER_COMMAND to a command on PATH.",
      ].join("\n"),
    );
  }
  return { runtime: "omena-lsp-server", command, args: [] };
}

export function buildThinClientRuntimeEndpoint(
  selection: LspServerRuntimeSelection,
  extensionRoot: string,
): ThinClientRuntimeEndpoint {
  return {
    product: "omena-css.thin-client-runtime-endpoint",
    runtime: "omena-lsp-server",
    command: selection.command,
    args: [...selection.args],
    cwd: extensionRoot,
    fileWatcherGlobs: buildRustLspFileWatcherGlobs(),
    nodeFallbackAllowed: false,
    hostResponsibilities: [
      "resolvePackagedRustBinary",
      "resolveStandaloneRustCommand",
      "buildThinClientServerOptions",
      "declareStaticDocumentSelector",
      "startLanguageClient",
      "registerStaticFileWatchers",
      "translateShowReferencesArguments",
      "renderHoverTracePanel",
      "surfaceStartupErrors",
    ],
    rustResponsibilities: [
      "ownLspLifecycle",
      "ownWorkspaceState",
      "ownDiagnosticsScheduling",
      "ownProviderExecution",
      "ownTsgoClientLifecycle",
    ],
  };
}

export function resolveOmenaLspServerPath(
  extensionRoot: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (path: string) => boolean = existsSync,
): string | null {
  const explicitCommand = env.OMENA_LSP_SERVER_COMMAND?.trim();
  if (explicitCommand) return explicitCommand;

  const explicitPath = env.OMENA_LSP_SERVER_PATH?.trim();
  if (explicitPath) {
    const resolved = path.resolve(extensionRoot, explicitPath);
    if (fileExists(resolved)) return resolved;
    throw new Error(`OMENA_LSP_SERVER_PATH points to a missing binary: ${resolved}`);
  }

  const binaryName = process.platform === "win32" ? "omena-lsp-server.exe" : "omena-lsp-server";
  const candidates = [
    path.join(extensionRoot, "dist", "bin", `${process.platform}-${process.arch}`, binaryName),
    path.join(extensionRoot, "rust", "target", "release", binaryName),
    path.join(extensionRoot, "rust", "target", "debug", binaryName),
  ];
  return candidates.find(fileExists) ?? null;
}
