import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import {
  buildClientStorageInitializationOptions,
  buildRustLspFileWatcherGlobs,
  buildThinClientRuntimeEndpoint,
  readClientCacheLocationSetting,
} from "../client/src/lsp-server-runtime-config";
import { readRustPackageMetadata } from "./rust-package-metadata";

interface RustOmenaLspServerBoundarySummary {
  readonly product: string;
  readonly migrationPhases: readonly {
    readonly phase: string;
    readonly exitGate: string;
  }[];
  readonly thinClientEndpoint: {
    readonly product: string;
    readonly endpointName: string;
    readonly transportContract: string;
    readonly commandOwner: string;
    readonly standalonePackage: string;
    readonly splitRepository: string;
    readonly cargoInstallCommand: string;
    readonly nodeFallbackAllowed: boolean;
    readonly fileWatcherGlobs: readonly string[];
    readonly hostResponsibilities: readonly string[];
    readonly rustResponsibilities: readonly string[];
  };
}

const rustSummary = readRustBoundarySummary();
const rustEndpoint = rustSummary.thinClientEndpoint;
const lspPackageMetadata = readRustPackageMetadata("omena-lsp-server");
const clientEndpoint = buildThinClientRuntimeEndpoint(
  {
    runtime: "omena-lsp-server",
    command: "/extension/dist/bin/darwin-arm64/omena-lsp-server",
    args: [],
  },
  "/extension",
);

assert.equal(rustSummary.product, "omena-lsp-server.boundary");
assert.equal(
  rustSummary.migrationPhases.find((phase) => phase.phase === "phase-4-thin-client")?.exitGate,
  "rust/omena-lsp-server/thin-client-boundary",
);
assert.equal(rustEndpoint.product, "omena-lsp-server.thin-client-endpoint");
assert.equal(rustEndpoint.endpointName, "omena-css.thin-client-runtime-endpoint");
assert.equal(rustEndpoint.transportContract, "LSP stdio JSON-RPC");
assert.equal(rustEndpoint.commandOwner, "dist/bin/<platform>-<arch>/omena-lsp-server");
assert.equal(rustEndpoint.standalonePackage, "omena-lsp-server");
assert.equal(rustEndpoint.splitRepository, lspPackageMetadata.repository);
assert.equal(
  rustEndpoint.cargoInstallCommand,
  `cargo install ${lspPackageMetadata.name} --version ${lspPackageMetadata.version}`,
);
assert.equal(rustEndpoint.nodeFallbackAllowed, false);
assert.deepEqual(rustEndpoint.fileWatcherGlobs, buildRustLspFileWatcherGlobs());
assert.deepEqual(clientEndpoint.fileWatcherGlobs, rustEndpoint.fileWatcherGlobs);
assert.equal(clientEndpoint.product, rustEndpoint.endpointName);
assert.equal(clientEndpoint.nodeFallbackAllowed, false);
assert.ok(rustEndpoint.hostResponsibilities.includes("resolveStandaloneRustCommand"));
assert.ok(clientEndpoint.hostResponsibilities.includes("resolveStandaloneRustCommand"));
assert.ok(rustEndpoint.hostResponsibilities.includes("buildThinClientServerOptions"));
assert.ok(clientEndpoint.hostResponsibilities.includes("buildThinClientServerOptions"));
assert.ok(rustEndpoint.hostResponsibilities.includes("prepareEditorStorageRoots"));
assert.ok(clientEndpoint.hostResponsibilities.includes("prepareEditorStorageRoots"));
assert.ok(rustEndpoint.hostResponsibilities.includes("passStorageInitializationOptions"));
assert.ok(clientEndpoint.hostResponsibilities.includes("passStorageInitializationOptions"));
assert.ok(rustEndpoint.hostResponsibilities.includes("requestServerOwnedCacheClear"));
assert.ok(clientEndpoint.hostResponsibilities.includes("requestServerOwnedCacheClear"));
assert.ok(rustEndpoint.hostResponsibilities.includes("declareStaticDocumentSelector"));
assert.ok(rustEndpoint.hostResponsibilities.includes("startLanguageClient"));
assert.ok(rustEndpoint.hostResponsibilities.includes("registerStaticFileWatchers"));
assert.ok(rustEndpoint.rustResponsibilities.includes("ownLspLifecycle"));
assert.ok(rustEndpoint.rustResponsibilities.includes("ownTsgoClientLifecycle"));
assert.ok(rustEndpoint.rustResponsibilities.includes("resolveAndClearDeclaredOwnedCachePaths"));
assert.ok(clientEndpoint.rustResponsibilities.includes("resolveAndClearDeclaredOwnedCachePaths"));
assert.ok(clientEndpoint.hostResponsibilities.includes("translateShowReferencesArguments"));
assert.ok(clientEndpoint.rustResponsibilities.includes("ownProviderExecution"));

const storageOptions = buildClientStorageInitializationOptions(
  "/editor/global",
  "/editor/workspace",
  "/editor/logs",
  readClientCacheLocationSetting("editor"),
);
assert.deepEqual(storageOptions, {
  globalStoragePath: "/editor/global",
  workspaceStoragePath: "/editor/workspace",
  logPath: "/editor/logs",
  location: "editor",
});
assert.equal(readClientCacheLocationSetting("workspace"), "workspace");
assert.equal(readClientCacheLocationSetting("global"), "global");
assert.equal(readClientCacheLocationSetting("invalid"), "editor");

const extensionSource = readFileSync("client/src/extension.ts", "utf8");
const packageManifest = JSON.parse(readFileSync("package.json", "utf8")) as {
  readonly contributes: {
    readonly commands: readonly { readonly command: string; readonly title: string }[];
    readonly configuration: {
      readonly properties: Readonly<Record<string, { readonly markdownDescription?: string }>>;
    };
  };
};
const storagePreparationIndex = extensionSource.indexOf("vscode.workspace.fs.createDirectory");
const clientStartIndex = extensionSource.indexOf("client.start()");
assert.ok(storagePreparationIndex >= 0, "extension must prepare editor storage roots");
assert.ok(
  clientStartIndex > storagePreparationIndex,
  "storage roots must exist before client.start",
);
for (const field of ["globalStorageUri", "storageUri", "logUri"]) {
  assert.ok(extensionSource.includes(field), `extension storage handoff must include ${field}`);
}
assert.ok(extensionSource.includes('const CLEAR_CACHES_REQUEST = "omena/clearCaches"'));
assert.ok(extensionSource.includes('const CLEAR_CACHES_COMMAND = "omena.clearCaches"'));
assert.ok(extensionSource.includes("sendRequest<OmenaCacheClearReport>(CLEAR_CACHES_REQUEST"));
assert.ok(
  extensionSource.includes("target.removedPaths"),
  "the cache command must report the server-returned removed paths",
);
assert.ok(
  !extensionSource.includes('join(".cache", "omena")'),
  "the thin client must not re-derive server cache roots",
);
assert.ok(
  packageManifest.contributes.commands.some(
    (command) => command.command === "omena.clearCaches" && command.title === "Omena: Clear Caches",
  ),
  "the extension manifest must contribute the cache command",
);
const cacheLocationDocumentation =
  packageManifest.contributes.configuration.properties["omena.cache.location"]
    ?.markdownDescription ?? "";
for (const platformPath of [
  "$XDG_CACHE_HOME/omena",
  "~/Library/Caches/omena",
  "%LOCALAPPDATA%\\omena",
]) {
  assert.ok(
    cacheLocationDocumentation.includes(platformPath),
    `cache setting documentation must name ${platformPath}`,
  );
}

process.stdout.write(
  [
    "validated omena-lsp-server thin client boundary:",
    `watchers=${rustEndpoint.fileWatcherGlobs.length}`,
    `host=${rustEndpoint.hostResponsibilities.length}`,
    `rust=${rustEndpoint.rustResponsibilities.length}`,
    `fallback=${rustEndpoint.nodeFallbackAllowed}`,
  ].join(" "),
);
process.stdout.write("\n");

function readRustBoundarySummary(): RustOmenaLspServerBoundarySummary {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-lsp-server",
      "--bin",
      "omena-lsp-server-boundary",
      "--quiet",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  assert.equal(
    result.status,
    0,
    [
      "omena-lsp-server boundary binary failed",
      result.error ? `error=${result.error.message}` : null,
      result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );

  return JSON.parse(result.stdout) as RustOmenaLspServerBoundarySummary;
}
