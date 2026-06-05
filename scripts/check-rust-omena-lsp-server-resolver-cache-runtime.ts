import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
import {
  createProtocolConnection,
  DidOpenTextDocumentNotification,
  InitializedNotification,
  InitializeRequest,
  ShutdownRequest,
  type InitializeParams,
  type InitializeResult,
  type ProtocolConnection,
} from "vscode-languageserver-protocol/node";
import { StreamMessageReader, StreamMessageWriter } from "vscode-jsonrpc/node";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const DEBUG_STATE_REQUEST = "omena/rustLspState";
const ALIAS_COUNT = parsePositiveInteger(process.env.OMENA_LSP_RESOLVER_CACHE_ALIASES, 360);
const PACKAGE_COUNT = parsePositiveInteger(process.env.OMENA_LSP_RESOLVER_CACHE_PACKAGES, 160);
const REFRESH_BASELINE_SAMPLES = parsePositiveInteger(
  process.env.OMENA_LSP_RESOLVER_CACHE_REFRESH_SAMPLES,
  8,
);
const HOT_SAMPLES = parsePositiveInteger(process.env.OMENA_LSP_RESOLVER_CACHE_HOT_SAMPLES, 24);
const REQUEST_TIMEOUT_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RESOLVER_CACHE_REQUEST_TIMEOUT_MS,
  30_000,
);
const MIN_REFRESH_TO_HOT_P50_RATIO = parsePositiveNumber(
  process.env.OMENA_LSP_RESOLVER_CACHE_MIN_REFRESH_TO_HOT_P50_RATIO,
  1.2,
);
const MAX_HOT_P95_MS = parsePositiveNumber(
  process.env.OMENA_LSP_RESOLVER_CACHE_MAX_HOT_P95_MS,
  250,
);

interface TimedSeriesSummary {
  readonly count: number;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly maxMs: number;
}

interface LspStateSnapshot {
  readonly cachedWorkspaceResolutionInputCount?: number;
}

async function main(): Promise<void> {
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "cme-rust-omena-lsp-resolver-cache-"));
  const srcDir = path.join(workspaceRoot, "src");
  const stylesDir = path.join(srcDir, "styles");
  const sourcePath = path.join(srcDir, "App.module.scss");
  const targetPath = path.join(stylesDir, "_tokens.scss");
  const configPath = path.join(workspaceRoot, "vite.config.ts");
  const webpackConfigPath = path.join(workspaceRoot, "webpack.config.js");
  const tsconfigPath = path.join(workspaceRoot, "tsconfig.json");
  const tsconfigBasePath = path.join(workspaceRoot, "tsconfig.base.json");
  const tsconfigSourcePath = path.join(srcDir, "Tsconfig.module.scss");
  const tsconfigOldTargetPath = path.join(srcDir, "tsconfig-old", "_tokens.scss");
  const tsconfigNewTargetPath = path.join(srcDir, "tsconfig-new", "_tokens.scss");
  const webpackSourcePath = path.join(srcDir, "Webpack.module.scss");
  const webpackOldTargetPath = path.join(srcDir, "webpack-old", "_tokens.scss");
  const webpackNewTargetPath = path.join(srcDir, "webpack-new", "_tokens.scss");
  const packageSourcePath = path.join(srcDir, "Package.module.scss");
  const rootPackageImportSourcePath = path.join(srcDir, "RootPackageImport.module.scss");
  const rootPackageJsonPath = path.join(workspaceRoot, "package.json");
  const packageRoot = path.join(workspaceRoot, "node_modules", "@design", "tokens");
  const packageJsonPath = path.join(packageRoot, "package.json");
  const packageOldTargetPath = path.join(packageRoot, "old.scss");
  const packageNewTargetPath = path.join(packageRoot, "new.scss");
  const sourceUri = pathToFileURL(sourcePath).toString();
  const targetUri = pathToFileURL(targetPath).toString();
  const configUri = pathToFileURL(configPath).toString();
  const webpackConfigUri = pathToFileURL(webpackConfigPath).toString();
  const tsconfigUri = pathToFileURL(tsconfigPath).toString();
  const tsconfigBaseUri = pathToFileURL(tsconfigBasePath).toString();
  const tsconfigSourceUri = pathToFileURL(tsconfigSourcePath).toString();
  const tsconfigOldTargetUri = pathToFileURL(tsconfigOldTargetPath).toString();
  const tsconfigNewTargetUri = pathToFileURL(tsconfigNewTargetPath).toString();
  const webpackSourceUri = pathToFileURL(webpackSourcePath).toString();
  const webpackOldTargetUri = pathToFileURL(webpackOldTargetPath).toString();
  const webpackNewTargetUri = pathToFileURL(webpackNewTargetPath).toString();
  const packageSourceUri = pathToFileURL(packageSourcePath).toString();
  const rootPackageImportSourceUri = pathToFileURL(rootPackageImportSourcePath).toString();
  const rootPackageJsonUri = pathToFileURL(rootPackageJsonPath).toString();
  const packageJsonUri = pathToFileURL(packageJsonPath).toString();
  const packageOldTargetUri = pathToFileURL(packageOldTargetPath).toString();
  const packageNewTargetUri = pathToFileURL(packageNewTargetPath).toString();
  const workspaceUri = pathToFileURL(workspaceRoot).toString();
  const sourceText = `@use "@styles/tokens" as tokens;
.root { color: tokens.$brand; }
`;
  const targetText = "$brand: #123456;\n";
  const tsconfigSourceText = `@use "$styles/tokens" as tokens;
.root { color: tokens.$brand; }
`;
  const webpackSourceText = `@use "@web/tokens" as tokens;
.root { color: tokens.$brand; }
`;
  const packageSourceText = `@use "pkg:@design/tokens" as tokens;
.root { color: tokens.$brand; }
`;
  const rootPackageImportSourceText = `@use "#theme" as tokens;
.root { color: tokens.$brand; }
`;
  const definitionParams = {
    textDocument: { uri: sourceUri },
    position: positionForOffset(sourceText, sourceText.indexOf("$brand") + 1),
  };
  const tsconfigDefinitionParams = {
    textDocument: { uri: tsconfigSourceUri },
    position: positionForOffset(tsconfigSourceText, tsconfigSourceText.indexOf("$brand") + 1),
  };
  const webpackDefinitionParams = {
    textDocument: { uri: webpackSourceUri },
    position: positionForOffset(webpackSourceText, webpackSourceText.indexOf("$brand") + 1),
  };
  const packageDefinitionParams = {
    textDocument: { uri: packageSourceUri },
    position: positionForOffset(packageSourceText, packageSourceText.indexOf("$brand") + 1),
  };
  const rootPackageImportDefinitionParams = {
    textDocument: { uri: rootPackageImportSourceUri },
    position: positionForOffset(
      rootPackageImportSourceText,
      rootPackageImportSourceText.indexOf("$brand") + 1,
    ),
  };
  const invocation = resolveOmenaLspServerInvocation();

  mkdirSync(stylesDir, { recursive: true });
  mkdirSync(path.dirname(tsconfigOldTargetPath), { recursive: true });
  mkdirSync(path.dirname(tsconfigNewTargetPath), { recursive: true });
  mkdirSync(path.dirname(webpackOldTargetPath), { recursive: true });
  mkdirSync(path.dirname(webpackNewTargetPath), { recursive: true });
  mkdirSync(packageRoot, { recursive: true });
  writeFileSync(sourcePath, sourceText);
  writeFileSync(targetPath, targetText);
  writeFileSync(tsconfigSourcePath, tsconfigSourceText);
  writeFileSync(tsconfigOldTargetPath, "$brand: old;\n");
  writeFileSync(tsconfigNewTargetPath, "$brand: new;\n");
  writeFileSync(webpackSourcePath, webpackSourceText);
  writeFileSync(webpackOldTargetPath, "$brand: webpack-old;\n");
  writeFileSync(webpackNewTargetPath, "$brand: webpack-new;\n");
  writeFileSync(packageSourcePath, packageSourceText);
  writeFileSync(rootPackageImportSourcePath, rootPackageImportSourceText);
  writeFileSync(rootPackageJsonPath, buildRootPackageJson("@design/tokens/old"));
  writeFileSync(
    packageJsonPath,
    JSON.stringify({
      sass: "old.scss",
      exports: {
        ".": { sass: "./old.scss" },
        "./old": { sass: "./old.scss" },
        "./new": { sass: "./new.scss" },
      },
    }),
  );
  writeFileSync(packageOldTargetPath, "$brand: old;\n");
  writeFileSync(packageNewTargetPath, "$brand: new;\n");
  writeFileSync(configPath, buildViteConfig(ALIAS_COUNT));
  writeFileSync(webpackConfigPath, buildWebpackConfig("src/webpack-old"));
  writeFileSync(tsconfigPath, buildTsconfigExtendsBase());
  writeFileSync(tsconfigBasePath, buildTsconfig("src/tsconfig-old/*"));
  writePackageManifests(workspaceRoot, PACKAGE_COUNT);

  const child = spawn(invocation.command, [...invocation.args], {
    cwd: process.cwd(),
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stderr: string[] = [];
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  const connection: ProtocolConnection = createProtocolConnection(
    new StreamMessageReader(child.stdout),
    new StreamMessageWriter(child.stdin),
  );
  connection.listen();

  try {
    const initialized = await requestWithTimeout(
      connection.sendRequest<InitializeResult>(
        InitializeRequest.type,
        initializeParams(workspaceRoot),
      ),
      "initialize",
    );
    assert.equal(initialized.serverInfo?.name, "omena-css-rust");

    await connection.sendNotification(InitializedNotification.type, {});
    await connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: targetUri,
        languageId: "scss",
        version: 1,
        text: targetText,
      },
    });
    await connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: sourceUri,
        languageId: "scss",
        version: 1,
        text: sourceText,
      },
    });
    await Promise.all(
      (
        [
          [tsconfigOldTargetUri, "$brand: old;\n"],
          [tsconfigNewTargetUri, "$brand: new;\n"],
          [tsconfigSourceUri, tsconfigSourceText],
          [webpackOldTargetUri, "$brand: webpack-old;\n"],
          [webpackNewTargetUri, "$brand: webpack-new;\n"],
          [webpackSourceUri, webpackSourceText],
          [packageOldTargetUri, "$brand: old;\n"],
          [packageNewTargetUri, "$brand: new;\n"],
          [packageSourceUri, packageSourceText],
          [rootPackageImportSourceUri, rootPackageImportSourceText],
        ] as const
      ).map(([uri, text]) =>
        connection.sendNotification(DidOpenTextDocumentNotification.type, {
          textDocument: {
            uri,
            languageId: "scss",
            version: 1,
            text,
          },
        }),
      ),
    );

    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", definitionParams),
        "warmup definition",
      ),
      targetUri,
    );
    const state = await requestWithTimeout(
      connection.sendRequest<LspStateSnapshot>(DEBUG_STATE_REQUEST, {}),
      "debug state",
    );
    assert.ok(
      (state.cachedWorkspaceResolutionInputCount ?? 0) >= 1,
      "LSP runtime must expose cached workspace resolver inputs after initialization",
    );

    const refreshBaselineLatencies = await collectRefreshBaselineLatencies(
      connection,
      configUri,
      definitionParams,
      targetUri,
    );
    const hotLatencies = await collectHotDefinitionLatencies(
      connection,
      definitionParams,
      targetUri,
    );
    const refreshSummary = summarizeSeries(refreshBaselineLatencies);
    const hotSummary = summarizeSeries(hotLatencies);
    const ratio = refreshSummary.p50Ms / Math.max(hotSummary.p50Ms, 0.01);
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", tsconfigDefinitionParams),
        "tsconfig initial definition",
      ),
      tsconfigOldTargetUri,
    );
    writeFileSync(tsconfigPath, buildTsconfig("src/tsconfig-new/*"));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: tsconfigUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", tsconfigDefinitionParams),
        "tsconfig refreshed definition",
      ),
      tsconfigNewTargetUri,
    );
    writeFileSync(tsconfigPath, buildTsconfigExtendsBase());
    writeFileSync(tsconfigBasePath, buildTsconfig("src/tsconfig-old/*"));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: tsconfigUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", tsconfigDefinitionParams),
        "tsconfig extends reset definition",
      ),
      tsconfigOldTargetUri,
    );
    writeFileSync(tsconfigBasePath, buildTsconfig("src/tsconfig-new/*"));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: tsconfigBaseUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", tsconfigDefinitionParams),
        "tsconfig.base refreshed definition",
      ),
      tsconfigNewTargetUri,
    );
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", webpackDefinitionParams),
        "webpack initial definition",
      ),
      webpackOldTargetUri,
    );
    writeFileSync(webpackConfigPath, buildWebpackConfig("src/webpack-new"));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: webpackConfigUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", webpackDefinitionParams),
        "webpack refreshed definition",
      ),
      webpackNewTargetUri,
    );
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", packageDefinitionParams),
        "package initial definition",
      ),
      packageOldTargetUri,
    );
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", rootPackageImportDefinitionParams),
        "root package imports initial definition",
      ),
      packageOldTargetUri,
    );
    writeFileSync(packageJsonPath, JSON.stringify({ sass: "new.scss" }));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: packageJsonUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", packageDefinitionParams),
        "package refreshed definition",
      ),
      packageNewTargetUri,
    );
    writeFileSync(rootPackageJsonPath, buildRootPackageJson("@design/tokens/new"));
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: rootPackageJsonUri, type: 2 }],
    });
    assertDefinitionTarget(
      await requestWithTimeout(
        connection.sendRequest("textDocument/definition", rootPackageImportDefinitionParams),
        "root package imports refreshed definition",
      ),
      packageNewTargetUri,
    );

    if (hotSummary.p95Ms > MAX_HOT_P95_MS) {
      throw new Error(
        [
          "cached LSP resolver provider requests exceeded hot-path budget",
          `hotP95=${hotSummary.p95Ms.toFixed(2)}ms`,
          `budget=${MAX_HOT_P95_MS.toFixed(2)}ms`,
        ].join("\n"),
      );
    }
    if (ratio < MIN_REFRESH_TO_HOT_P50_RATIO) {
      throw new Error(
        [
          "cached LSP resolver requests did not beat refresh baseline",
          `refreshP50=${refreshSummary.p50Ms.toFixed(2)}ms`,
          `hotP50=${hotSummary.p50Ms.toFixed(2)}ms`,
          `ratio=${ratio.toFixed(2)}`,
          `required=${MIN_REFRESH_TO_HOT_P50_RATIO.toFixed(2)}`,
        ].join("\n"),
      );
    }

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");
    const exitCode = await waitForExit(child);
    assert.equal(exitCode, 0, `omena-lsp-server exited with ${exitCode}\n${stderr.join("")}`);

    process.stdout.write(
      [
        "omena-lsp-server resolver cache runtime ok:",
        `command=${invocation.command}`,
        `workspace=${workspaceUri}`,
        `aliases=${ALIAS_COUNT}`,
        `packages=${PACKAGE_COUNT}`,
        `cachedWorkspaceResolutionInputs=${state.cachedWorkspaceResolutionInputCount ?? 0}`,
        `refreshBaseline=${formatSummary(refreshSummary)}`,
        `hotDefinition=${formatSummary(hotSummary)}`,
        `p50Ratio=${ratio.toFixed(2)}`,
        "invalidations=package.json(root+package),tsconfig.json,tsconfig.base.json,vite.config.ts,webpack.config.js",
      ].join(" ") + "\n",
    );
  } finally {
    connection.dispose();
    if (!child.killed && child.exitCode === null) {
      child.kill();
    }
    rmSync(workspaceRoot, { force: true, recursive: true });
  }
}

async function collectRefreshBaselineLatencies(
  connection: ProtocolConnection,
  configUri: string,
  definitionParams: unknown,
  targetUri: string,
): Promise<readonly number[]> {
  const latencies: number[] = [];
  for (let index = 0; index < REFRESH_BASELINE_SAMPLES; index += 1) {
    const started = performance.now();
    // oxlint-disable-next-line eslint/no-await-in-loop
    await connection.sendNotification("workspace/didChangeWatchedFiles", {
      changes: [{ uri: configUri, type: 2 }],
    });
    // oxlint-disable-next-line eslint/no-await-in-loop
    const response = await requestWithTimeout(
      connection.sendRequest("textDocument/definition", definitionParams),
      `refresh-baseline-definition:${index}`,
    );
    latencies.push(performance.now() - started);
    assertDefinitionTarget(response, targetUri);
  }
  return latencies;
}

async function collectHotDefinitionLatencies(
  connection: ProtocolConnection,
  definitionParams: unknown,
  targetUri: string,
): Promise<readonly number[]> {
  const latencies: number[] = [];
  for (let index = 0; index < HOT_SAMPLES; index += 1) {
    const started = performance.now();
    // oxlint-disable-next-line eslint/no-await-in-loop
    const response = await requestWithTimeout(
      connection.sendRequest("textDocument/definition", definitionParams),
      `hot-definition:${index}`,
    );
    latencies.push(performance.now() - started);
    assertDefinitionTarget(response, targetUri);
  }
  return latencies;
}

function initializeParams(workspaceRoot: string): InitializeParams {
  const workspaceUri = pathToFileURL(workspaceRoot).toString();
  return {
    processId: process.pid,
    rootUri: workspaceUri,
    workspaceFolders: [{ uri: workspaceUri, name: "cme-rust-omena-lsp-resolver-cache" }],
    capabilities: {
      workspace: {
        configuration: true,
        workspaceFolders: true,
      },
      textDocument: {
        publishDiagnostics: {},
      },
    },
  };
}

function buildViteConfig(aliasCount: number): string {
  const aliases = [
    `"@styles": "./src/styles"`,
    ...Array.from(
      { length: aliasCount },
      (_, index) => `"@unused-${index}": "./src/unused-${index}"`,
    ),
  ].join(",\n      ");
  return `export default {
  resolve: {
    alias: {
      ${aliases}
    }
  }
};
`;
}

function buildWebpackConfig(webAliasTarget: string): string {
  return `module.exports = {
  resolve: {
    alias: [
      { find: "@unused", replacement: "./src/unused" },
      { find: "@web", replacement: "./${webAliasTarget}" }
    ]
  }
};
`;
}

function buildTsconfig(stylesTarget: string): string {
  return `${JSON.stringify(
    {
      compilerOptions: {
        baseUrl: ".",
        paths: {
          "$styles/*": [stylesTarget],
        },
      },
    },
    null,
    2,
  )}\n`;
}

function buildTsconfigExtendsBase(): string {
  return `${JSON.stringify(
    {
      extends: "./tsconfig.base.json",
    },
    null,
    2,
  )}\n`;
}

function buildRootPackageJson(themeTarget: string): string {
  return `${JSON.stringify(
    {
      imports: {
        "#theme": themeTarget,
      },
    },
    null,
    2,
  )}\n`;
}

function writePackageManifests(workspaceRoot: string, packageCount: number): void {
  const scopeDir = path.join(workspaceRoot, "node_modules", "@cache");
  mkdirSync(scopeDir, { recursive: true });
  for (let index = 0; index < packageCount; index += 1) {
    const packageDir = path.join(scopeDir, `pkg-${index}`);
    mkdirSync(packageDir, { recursive: true });
    writeFileSync(
      path.join(packageDir, "package.json"),
      JSON.stringify({
        name: `@cache/pkg-${index}`,
        version: "0.0.0",
        exports: {
          ".": {
            style: "./index.css",
          },
        },
      }),
    );
  }
}

function assertDefinitionTarget(response: unknown, targetUri: string): void {
  const values = Array.isArray(response) ? response : response ? [response] : [];
  assert.ok(values.length > 0, "definition response must include at least one location");
  assert.ok(
    values.some((value) => isObject(value) && value.uri === targetUri),
    `definition response must target ${targetUri}: ${JSON.stringify(response)}`,
  );
}

function positionForOffset(text: string, offset: number): { line: number; character: number } {
  assert.ok(offset >= 0, "source text must contain requested token");
  const before = text.slice(0, offset);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)!.length,
  };
}

async function requestWithTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
  let timer: NodeJS.Timeout | null = null;
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), REQUEST_TIMEOUT_MS);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

function waitForExit(child: ReturnType<typeof spawn>): Promise<number | null> {
  return new Promise((resolve) => {
    child.once("exit", (code) => resolve(code));
    child.once("close", (code) => resolve(code));
  });
}

function summarizeSeries(values: readonly number[]): TimedSeriesSummary {
  return {
    count: values.length,
    p50Ms: percentile(values, 50),
    p95Ms: percentile(values, 95),
    maxMs: Math.max(...values),
  };
}

function formatSummary(summary: TimedSeriesSummary): string {
  return `n=${summary.count},p50=${summary.p50Ms.toFixed(2)}ms,p95=${summary.p95Ms.toFixed(2)}ms,max=${summary.maxMs.toFixed(2)}ms`;
}

function percentile(values: readonly number[], p: number): number {
  const sorted = values.toSorted((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil((p / 100) * sorted.length) - 1));
  return sorted[index] ?? 0;
}

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parsePositiveNumber(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function isObject(value: unknown): value is { readonly uri?: unknown } {
  return typeof value === "object" && value !== null;
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});
