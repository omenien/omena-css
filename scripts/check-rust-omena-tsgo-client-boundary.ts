import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { buildTsgoTypeFactWorkerInvocation } from "../server/engine-host-node/src/tsgo-type-fact-collector";

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

const projectRoot = path.join("/extension", "css-module-explainer");
const platformDir = `${process.platform}-${process.arch}`;
const binaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const packagedTsgoPath = path.join(projectRoot, "dist", "bin", platformDir, binaryName);
const nodeInvocation = buildTsgoTypeFactWorkerInvocation(
  "/workspace",
  { OMENA_PROJECT_ROOT: projectRoot } as NodeJS.ProcessEnv,
  (filePath) => filePath === packagedTsgoPath,
);

assert.equal(nodeInvocation.command, process.execPath);
assert.equal(nodeInvocation.args[0], "-e");
assert.equal(nodeInvocation.cwd, "/workspace");
assert.equal(nodeInvocation.env.OMENA_TSGO_PATH, packagedTsgoPath);

process.stdout.write(
  [
    "validated omena-tsgo-client boundary:",
    `methods=${summary.apiMethods.length}`,
    `policies=${summary.requestPathPolicy.length}`,
    `runtime=${summary.runtimeModel}`,
    `nodeWorker=${nodeInvocation.command}`,
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
