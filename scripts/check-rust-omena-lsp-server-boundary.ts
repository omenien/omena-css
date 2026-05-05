import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";
import { buildServerCapabilities } from "../server/lsp-server/src/server-capabilities";

interface RustOmenaLspServerBoundarySummary {
  readonly schemaVersion: string;
  readonly product: string;
  readonly migrationStatus: string;
  readonly capabilities: {
    readonly textDocumentSync: number;
    readonly definitionProvider: boolean;
    readonly hoverProvider: boolean;
    readonly completionProvider: {
      readonly triggerCharacters: readonly string[];
      readonly resolveProvider: boolean;
    };
    readonly codeActionProvider: {
      readonly codeActionKinds: readonly string[];
      readonly resolveProvider: boolean;
    };
    readonly referencesProvider: boolean;
    readonly codeLensProvider: {
      readonly resolveProvider: boolean;
    };
    readonly renameProvider: {
      readonly prepareProvider: boolean;
    };
    readonly workspace: {
      readonly workspaceFolders: {
        readonly supported: boolean;
        readonly changeNotifications: boolean;
      };
    };
  };
  readonly handlerSurfaces: readonly {
    readonly method: string;
    readonly migrationState: string;
  }[];
  readonly migrationPhases: readonly {
    readonly phase: string;
    readonly goal: string;
    readonly exitGate: string;
  }[];
  readonly blockingWorkPolicy: readonly string[];
  readonly tsgoClientBoundary: {
    readonly product: string;
    readonly runtimeModel: string;
    readonly requestPathPolicy: readonly string[];
  };
  readonly sourceProviderAdapter: {
    readonly product: string;
    readonly candidateOwner: string;
    readonly styleDefinitionOwner: string;
    readonly typeFactOwner: string;
    readonly requestPathPolicy: readonly string[];
    readonly providerSurfaces: readonly string[];
  };
  readonly workspaceRuntimeRegistry: {
    readonly product: string;
    readonly owner: string;
    readonly folderStateOwner: string;
    readonly ownershipPolicy: readonly string[];
    readonly indexedDocumentPolicy: readonly string[];
    readonly requestPathPolicy: readonly string[];
  };
  readonly diagnosticsScheduler: {
    readonly product: string;
    readonly owner: string;
    readonly schedulingModel: string;
    readonly eventPolicy: readonly string[];
    readonly requestPathPolicy: readonly string[];
  };
  readonly queryReuse: {
    readonly product: string;
    readonly owner: string;
    readonly reuseModel: string;
    readonly cachedSurfaces: readonly string[];
    readonly invalidationPolicy: readonly string[];
    readonly requestPathPolicy: readonly string[];
  };
  readonly thinClientEndpoint: {
    readonly product: string;
    readonly standalonePackage: string;
    readonly splitRepository: string;
    readonly cargoInstallCommand: string;
  };
  readonly multiEditorDistribution: {
    readonly product: string;
    readonly owner: string;
    readonly distributionModel: string;
    readonly supportedEditors: readonly string[];
    readonly installSurfaces: readonly string[];
    readonly documentation: readonly string[];
    readonly endpointPolicy: readonly string[];
  };
  readonly nextDecouplingTargets: readonly string[];
}

const rustSummary = readRustBoundarySummary();
const nodeCapabilities = buildServerCapabilities();
const repoRoot = process.cwd();
const lspServerCargoToml = readFileSync(
  path.join(repoRoot, "rust/crates/omena-lsp-server/Cargo.toml"),
  "utf8",
);

assert.equal(rustSummary.schemaVersion, "0");
assert.equal(rustSummary.product, "omena-lsp-server.boundary");
assert.equal(rustSummary.migrationStatus, "rustStable");
assert.ok(
  !/^\s*engine-style-parser\s*=/.test(lspServerCargoToml),
  "omena-lsp-server must consume style parser facts through omena-query, not a direct engine-style-parser dependency",
);
assert.ok(
  !/^\s*omena-bridge\s*=/.test(lspServerCargoToml),
  "omena-lsp-server must consume source syntax and style URI facts through omena-query, not a direct omena-bridge dependency",
);

assert.deepEqual(rustSummary.capabilities, nodeCapabilities);
assert.deepEqual(
  rustSummary.handlerSurfaces.map((surface) => surface.method).toSorted(),
  [
    "$/cancelRequest",
    "textDocument/codeAction",
    "textDocument/codeLens",
    "textDocument/completion",
    "textDocument/definition",
    "textDocument/didChange",
    "textDocument/didClose",
    "textDocument/didOpen",
    "textDocument/hover",
    "textDocument/prepareRename",
    "textDocument/publishDiagnostics",
    "textDocument/references",
    "textDocument/rename",
    "initialized",
    "workspace/didChangeConfiguration",
    "workspace/didChangeWatchedFiles",
    "workspace/didChangeWorkspaceFolders",
  ].toSorted(),
);
assert.ok(
  rustSummary.blockingWorkPolicy.includes("noFullWorkspaceProgramOnRequestPath"),
  "Rust LSP boundary must explicitly reject full workspace program work on request paths",
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("tsgoJsonRpcProviderImplementation"),
  "implemented tsgo JSON-RPC provider should not remain listed as a next target",
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("incrementalQueryReuse"),
  "implemented query reuse should not remain listed as a next target",
);
assert.equal(rustSummary.tsgoClientBoundary.product, "omena-tsgo-client.boundary");
assert.equal(rustSummary.tsgoClientBoundary.runtimeModel, "longLivedWorkspaceProcess");
assert.ok(
  rustSummary.tsgoClientBoundary.requestPathPolicy.includes("noSyncWorkspaceFallbackOnRequestPath"),
  "Rust LSP boundary must embed the phase-3 tsgo client request-path contract",
);
assert.equal(
  rustSummary.sourceProviderAdapter.product,
  "omena-lsp-server.source-provider-direct-rust-adapter",
);
assert.equal(rustSummary.sourceProviderAdapter.candidateOwner, "omena-query/sourceSyntaxIndex");
assert.equal(
  rustSummary.sourceProviderAdapter.styleDefinitionOwner,
  "omena-query/styleHoverCandidates",
);
assert.equal(rustSummary.sourceProviderAdapter.typeFactOwner, "omena-tsgo-client");
assert.ok(
  rustSummary.sourceProviderAdapter.requestPathPolicy.includes(
    "noNodeWorkspaceTypeResolverOnSourceProviderPath",
  ),
);
assert.ok(
  rustSummary.sourceProviderAdapter.requestPathPolicy.includes(
    "buildQuerySourceSyntaxIndexOnDocumentChange",
  ),
);
assert.ok(
  rustSummary.sourceProviderAdapter.requestPathPolicy.includes("dedupeTargetAwareSourceCandidates"),
);
assert.ok(
  rustSummary.sourceProviderAdapter.requestPathPolicy.includes("consumeQueryStyleHoverCandidates"),
);
assert.ok(
  rustSummary.sourceProviderAdapter.requestPathPolicy.includes("consumeQuerySassModuleSources"),
);
assert.ok(rustSummary.sourceProviderAdapter.providerSurfaces.includes("textDocument/definition"));
assertDefaultHostPathHasNoNodeWorkspaceResolver(repoRoot);
assert.equal(
  rustSummary.workspaceRuntimeRegistry.product,
  "omena-lsp-server.workspace-runtime-registry",
);
assert.equal(
  rustSummary.workspaceRuntimeRegistry.owner,
  "omena-lsp-server/runtime/workspaceRuntimeRegistry",
);
assert.ok(
  rustSummary.workspaceRuntimeRegistry.ownershipPolicy.includes("longestWorkspaceRootOwnsDocument"),
);
assert.ok(
  rustSummary.workspaceRuntimeRegistry.ownershipPolicy.includes(
    "filePathComponentBoundariesBeforeUriPrefix",
  ),
);
assert.ok(
  rustSummary.workspaceRuntimeRegistry.indexedDocumentPolicy.includes(
    "openedDocumentsRemainAuthoritative",
  ),
);
assert.ok(
  rustSummary.workspaceRuntimeRegistry.requestPathPolicy.includes(
    "noNodeWorkspaceRuntimeManagerOnRustLspPath",
  ),
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("rustWorkspaceRuntimeRegistry"),
  "implemented workspace runtime registry should not remain listed as a next target",
);
assert.equal(rustSummary.diagnosticsScheduler.product, "omena-lsp-server.diagnostics-scheduler");
assert.equal(rustSummary.diagnosticsScheduler.owner, "omena-lsp-server/diagnosticsScheduler");
assert.equal(rustSummary.diagnosticsScheduler.schedulingModel, "deterministicNotificationPlanner");
assert.ok(
  rustSummary.diagnosticsScheduler.eventPolicy.includes("refreshSourceDiagnosticsForStyleChanges"),
);
assert.ok(
  rustSummary.diagnosticsScheduler.requestPathPolicy.includes(
    "noNodeDiagnosticsSchedulerOnRustLspPath",
  ),
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("rustDiagnosticsScheduler"),
  "implemented diagnostics scheduler should not remain listed as a next target",
);
assert.equal(rustSummary.queryReuse.product, "omena-lsp-server.query-reuse");
assert.equal(rustSummary.queryReuse.owner, "omena-lsp-server/documentQueryReuse");
assert.equal(rustSummary.queryReuse.reuseModel, "documentRevisionOwnedReusableIndexes");
assert.ok(rustSummary.queryReuse.cachedSurfaces.includes("sourceSyntaxIndex"));
assert.ok(rustSummary.queryReuse.cachedSurfaces.includes("styleHoverCandidates"));
assert.ok(rustSummary.queryReuse.invalidationPolicy.includes("refreshOnDocumentContentChange"));
assert.ok(
  rustSummary.queryReuse.requestPathPolicy.includes("providerRequestsConsumeDocumentIndexes"),
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("thinVsCodeClientHost"),
  "implemented thin VS Code client host should not remain listed as a next target",
);
assert.ok(
  !rustSummary.nextDecouplingTargets.includes("multiEditorDistribution"),
  "implemented multi-editor distribution should not remain listed as a next target",
);
assert.equal(rustSummary.thinClientEndpoint.product, "omena-lsp-server.thin-client-endpoint");
assert.equal(rustSummary.thinClientEndpoint.standalonePackage, "omena-lsp-server");
assert.equal(
  rustSummary.thinClientEndpoint.splitRepository,
  "https://github.com/omenien/omena-lsp-server",
);
assert.equal(
  rustSummary.thinClientEndpoint.cargoInstallCommand,
  "cargo install omena-lsp-server --version 0.1.5",
);
assert.equal(
  rustSummary.multiEditorDistribution.product,
  "omena-lsp-server.multi-editor-distribution",
);
assert.equal(rustSummary.multiEditorDistribution.owner, "omena-lsp-server/distribution");
assert.equal(
  rustSummary.multiEditorDistribution.distributionModel,
  "standaloneRustLspServerWithThinEditorHosts",
);
assert.deepEqual(rustSummary.multiEditorDistribution.supportedEditors.toSorted(), [
  "neovim",
  "vscode",
  "zed",
]);
assert.ok(
  rustSummary.multiEditorDistribution.installSurfaces.includes("cargoInstallOmenaLspServer"),
);
assert.ok(rustSummary.multiEditorDistribution.installSurfaces.includes("repoLocalDistBin"));
assert.ok(rustSummary.multiEditorDistribution.documentation.includes("docs/clients/neovim.md"));
assert.ok(rustSummary.multiEditorDistribution.documentation.includes("docs/clients/zed.md"));
assert.ok(
  rustSummary.multiEditorDistribution.endpointPolicy.includes(
    "standaloneRustServerIsPrimaryMultiEditorEndpoint",
  ),
);
assert.ok(
  rustSummary.multiEditorDistribution.endpointPolicy.includes("nodeLspServerIsNotPrimaryEndpoint"),
);
assert.deepEqual(
  rustSummary.migrationPhases.map((phase) => phase.phase),
  [
    "phase-0-boundary",
    "phase-1-shell",
    "phase-2-style-providers",
    "phase-3-source-providers",
    "phase-4-thin-client",
  ],
);
assert.equal(
  rustSummary.migrationPhases.find((phase) => phase.phase === "phase-3-source-providers")?.exitGate,
  "rust/omena-tsgo-client/boundary",
);

process.stdout.write(
  [
    "validated omena-lsp-server boundary:",
    `handlers=${rustSummary.handlerSurfaces.length}`,
    `phases=${rustSummary.migrationPhases.length}`,
    `completionTriggers=${rustSummary.capabilities.completionProvider.triggerCharacters.length}`,
    `migration=${rustSummary.migrationStatus}`,
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

function assertDefaultHostPathHasNoNodeWorkspaceResolver(root: string): void {
  const coreTypeResolverSource = readRepoFile(
    root,
    "server/engine-core-ts/src/core/ts/type-resolver.ts",
  );
  assert.doesNotMatch(
    coreTypeResolverSource,
    /\bclass\s+WorkspaceTypeResolver\b/u,
    "engine-core-ts must keep TypeResolver as a contract only, without the legacy sync workspace resolver implementation",
  );
  assert.doesNotMatch(
    coreTypeResolverSource,
    /\bcreateProgram\b/u,
    "engine-core-ts TypeResolver contract must not expose synchronous ts.Program construction",
  );

  const typeBackendSource = readRepoFile(root, "server/engine-host-node/src/type-backend.ts");
  assert.doesNotMatch(
    typeBackendSource,
    /\bWorkspaceTypeResolver\b/u,
    "type-backend.ts must not import or construct WorkspaceTypeResolver on the default host path",
  );
  assert.doesNotMatch(
    typeBackendSource,
    /\bcreateDefaultProgram\b/u,
    "type-backend.ts must not create a synchronous TypeScript program on the default host path",
  );

  const extensionSource = readRepoFile(root, "client/src/extension.ts");
  assert.doesNotMatch(
    extensionSource,
    /\btypeFactMaxSyncProgramFiles\b|CME_TYPE_FACT_MAX_SYNC_PROGRAM_FILES/u,
    "VS Code thin client must not expose sync TypeScript program budget settings",
  );

  const typeFactConfigSource = readRepoFile(root, "client/src/type-fact-backend-config.ts");
  assert.doesNotMatch(
    typeFactConfigSource,
    /typescript-current|CME_TYPE_FACT_MAX_SYNC_PROGRAM_FILES|readTypeFactMaxSyncProgramFilesSetting/u,
    "client type-fact config must expose only tsgo-backed product modes",
  );

  const packageJson = JSON.parse(readRepoFile(root, "package.json")) as {
    contributes?: { configuration?: { properties?: Record<string, unknown> } };
  };
  const properties = packageJson.contributes?.configuration?.properties ?? {};
  assert.ok(
    !Object.hasOwn(properties, "cssModuleExplainer.typeFactMaxSyncProgramFiles"),
    "package settings must not expose sync TypeScript resolver budget",
  );
  const typeFactBackend = properties["cssModuleExplainer.typeFactBackend"] as
    | { enum?: readonly string[] }
    | undefined;
  assert.deepEqual(
    typeFactBackend?.enum,
    ["tsgo", "tsgo-workspace"],
    "package settings must expose only tsgo-backed type fact backends",
  );
}

function readRepoFile(root: string, relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}
