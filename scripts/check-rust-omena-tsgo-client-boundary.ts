import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { buildTsgoTypeFactApiOptions } from "../server/engine-host-node/src/tsgo-type-fact-collector";

interface OmenaTsgoClientBoundarySummary {
  readonly schemaVersion: string;
  readonly product: string;
  readonly clientName: string;
  readonly runtimeModel: string;
  readonly workspaceProcessPolicy: {
    readonly processScope: string;
    readonly startupMode: string;
    readonly shutdownOwner: string;
    readonly maxWorkspaceProcesses: number;
    readonly defaultCheckerWorkers: number;
  };
  readonly requestPathPolicy: readonly string[];
  readonly apiMethods: readonly {
    readonly method: string;
    readonly requestGroup: string;
  }[];
  readonly typeFactContract: {
    readonly inputContract: string;
    readonly outputContract: string;
    readonly targetIdentity: readonly string[];
    readonly projectMissBehavior: string;
  };
  readonly providerCapabilities: {
    readonly providerId: string;
    readonly providerKind: string;
    readonly capabilitySurface: string;
    readonly inputContract: string;
    readonly outputContract: string;
    readonly fallbackDiscipline: string;
    readonly unknownPrecisionValueDomain: string;
    readonly downgradeProvenance: string;
  };
  readonly lifecycle: {
    readonly openProjectMethod: string;
    readonly snapshotReleaseMethod: string;
    readonly cancellationBoundary: string;
  };
  readonly recoveryPolicy: {
    readonly retryScope: string;
    readonly maxBatchAttempts: number;
    readonly recoverableErrors: readonly string[];
    readonly recoveryAction: string;
    readonly snapshotPolicy: string;
  };
  readonly readySurfaces: readonly string[];
  readonly cmeCoupledSurfaces: readonly string[];
  readonly nextDecouplingTargets: readonly string[];
}

const summary = readRustBoundarySummary();
const methodNames = summary.apiMethods.map((method) => method.method);

assert.equal(summary.schemaVersion, "0");
assert.equal(summary.product, "omena-tsgo-client.boundary");
assert.equal(summary.clientName, "omena-tsgo-client");
assert.equal(summary.runtimeModel, "longLivedWorkspaceProcess");
assert.equal(summary.workspaceProcessPolicy.processScope, "oneTsgoApiProcessPerWorkspace");
assert.equal(summary.workspaceProcessPolicy.startupMode, "backgroundWarmup");
assert.equal(summary.workspaceProcessPolicy.shutdownOwner, "omena-lsp-server");
assert.equal(summary.workspaceProcessPolicy.maxWorkspaceProcesses, 1);
assert.deepEqual(methodNames, [
  "initialize",
  "updateSnapshot",
  "getDefaultProjectForFile",
  "getTypeAtPosition",
  "getTypesOfType",
  "release",
]);
assert.ok(summary.requestPathPolicy.includes("noTypeScriptCreateProgramOnRequestPath"));
assert.ok(summary.requestPathPolicy.includes("noSyncWorkspaceFallbackOnRequestPath"));
assert.ok(summary.requestPathPolicy.includes("returnUnresolvedWhenTsgoUnavailable"));
assert.ok(summary.requestPathPolicy.includes("cooperativeCancellationBeforeTsgoRequest"));
assert.deepEqual(summary.typeFactContract.targetIdentity, ["filePath", "expressionId", "position"]);
assert.equal(summary.typeFactContract.inputContract, "TsgoTypeFactRequestV0");
assert.equal(summary.typeFactContract.outputContract, "TsgoTypeFactResultEntryV0[]");
assert.match(summary.typeFactContract.projectMissBehavior, /unresolvable/u);
assert.deepEqual(summary.providerCapabilities, {
  providerId: "tsgo",
  providerKind: "type-oracle",
  capabilitySurface: "sourceBindingTypeFactResolution",
  inputContract: "TsgoTypeFactRequestV0",
  outputContract: "TsgoTypeFactResultEntryV0[]",
  fallbackDiscipline: "unknownNotGuess",
  unknownPrecisionValueDomain: "unknown",
  downgradeProvenance: "tsgo-provider.unavailable->unknown-precision",
});
for (const lspProviderField of [
  "definitionProvider",
  "hoverProvider",
  "completionProvider",
  "codeActionProvider",
  "referencesProvider",
  "codeLensProvider",
  "renameProvider",
]) {
  assert.equal(
    lspProviderField in summary.providerCapabilities,
    false,
    `type-oracle provider capability must not mirror LSP server capability field ${lspProviderField}`,
  );
}
assert.equal(summary.lifecycle.openProjectMethod, "updateSnapshot");
assert.equal(summary.lifecycle.snapshotReleaseMethod, "release");
assert.match(summary.lifecycle.cancellationBoundary, /getTypeAtPosition/u);
assert.equal(summary.recoveryPolicy.retryScope, "wholeTypeFactBatch");
assert.equal(summary.recoveryPolicy.maxBatchAttempts, 2);
assert.deepEqual(summary.recoveryPolicy.recoverableErrors, [
  "io",
  "missingResponse",
  "unexpectedResponseId",
]);
assert.equal(summary.recoveryPolicy.recoveryAction, "restartWorkspaceProcessThenReplayBatch");
assert.equal(summary.recoveryPolicy.snapshotPolicy, "discardPreviousSnapshotAndOpenFreshSnapshot");
assert.ok(summary.readySurfaces.includes("phase3SourceProviderExitGate"));
assert.ok(summary.readySurfaces.includes("persistentWorkspaceProcessPool"));
assert.ok(summary.readySurfaces.includes("jsonRpcContentLengthTransport"));
assert.ok(summary.readySurfaces.includes("jsonRpcProcessIo"));
assert.ok(summary.readySurfaces.includes("jsonRpcTypeFactProviderImplementation"));
assert.ok(summary.readySurfaces.includes("recoverableBatchRetry"));
assert.ok(summary.readySurfaces.includes("workspaceProcessRecovery"));
assert.ok(summary.readySurfaces.includes("providerCancellationTokenBoundary"));
assert.ok(summary.readySurfaces.includes("typeFactRpcClient"));
assert.ok(summary.readySurfaces.includes("typeFactResultReducer"));
assert.ok(summary.nextDecouplingTargets.includes("sourceProviderDirectRustAdapter"));
assert.ok(
  summary.cmeCoupledSurfaces.includes("server/engine-host-node/src/tsgo-type-fact-collector.ts"),
);

const queryTypeConstants = readFileSync(
  path.join(process.cwd(), "rust/crates/omena-query/src/types.rs"),
  "utf8",
);
assert.match(
  queryTypeConstants,
  new RegExp(
    `OMENA_QUERY_TYPE_ORACLE_UNKNOWN_VALUE_DOMAIN[^=]*=\\s*"${summary.providerCapabilities.unknownPrecisionValueDomain}"`,
    "u",
  ),
);
assert.match(
  queryTypeConstants,
  new RegExp(
    `OMENA_QUERY_TSGO_PROVIDER_UNAVAILABLE_PROVENANCE[^=]*=\\s*"${summary.providerCapabilities.downgradeProvenance.replaceAll(
      ".",
      "\\.",
    )}"`,
    "u",
  ),
);
const querySourceRefs = readFileSync(
  path.join(process.cwd(), "rust/crates/omena-query/src/style/source_refs.rs"),
  "utf8",
);
assert.match(querySourceRefs, /code:\s*"unknownClassValueDomain"/u);
assert.match(querySourceRefs, /OMENA_QUERY_TSGO_PROVIDER_UNAVAILABLE_PROVENANCE/u);

const projectRoot = path.join("/extension", "css-module-explainer");
const platformDir = `${process.platform}-${process.arch}`;
const binaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const packagedTsgoPath = path.join(projectRoot, "dist", "bin", platformDir, binaryName);
const nodeApiOptions = buildTsgoTypeFactApiOptions(
  "/workspace",
  { OMENA_PROJECT_ROOT: projectRoot } as NodeJS.ProcessEnv,
  (filePath) => filePath === packagedTsgoPath,
);

assert.equal(nodeApiOptions.cwd, "/workspace");
assert.equal(nodeApiOptions.tsserverPath, packagedTsgoPath);

process.stdout.write(
  [
    "validated omena-tsgo-client boundary:",
    `methods=${summary.apiMethods.length}`,
    `policies=${summary.requestPathPolicy.length}`,
    `runtime=${summary.runtimeModel}`,
    `nodeApiCwd=${nodeApiOptions.cwd}`,
  ].join(" "),
);
process.stdout.write("\n");

function readRustBoundarySummary(): OmenaTsgoClientBoundarySummary {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-tsgo-client",
      "--bin",
      "omena-tsgo-client-boundary",
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
      "omena-tsgo-client boundary binary failed",
      result.error ? `error=${result.error.message}` : null,
      result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );

  return JSON.parse(result.stdout) as OmenaTsgoClientBoundarySummary;
}
