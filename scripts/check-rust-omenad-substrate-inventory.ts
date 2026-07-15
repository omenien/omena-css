import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const snapshotPath = "rust/omena-omenad-substrate-inventory.json";
const write = process.argv.includes("--write");
const injectedMissingCapability = process.argv
  .find((argument) => argument.startsWith("--inject-missing-capability="))
  ?.split("=", 2)[1];

interface CapabilityProbe {
  readonly capability: string;
  readonly disposition: "reuse" | "extract-and-reuse" | "build-adapter";
  readonly sources: ReadonlyArray<{
    readonly path: string;
    readonly symbol: string;
    readonly pattern: RegExp;
  }>;
}

interface CapabilityRow {
  readonly capability: string;
  readonly disposition: CapabilityProbe["disposition"];
  readonly evidence: ReadonlyArray<{
    readonly path: string;
    readonly symbol: string;
  }>;
}

interface InventorySnapshot {
  readonly schemaVersion: "0";
  readonly product: "omena.omenad-substrate-inventory";
  readonly capabilities: readonly CapabilityRow[];
}

const probes: readonly CapabilityProbe[] = [
  {
    capability: "workspace-snapshot-session",
    disposition: "reuse",
    sources: [
      source(
        "rust/crates/omena-query/src/sdk_workspace.rs",
        "OmenaSdkWorkspaceV0",
        /pub struct OmenaSdkWorkspaceV0/,
      ),
      source(
        "rust/crates/omena-query/src/sdk_workspace.rs",
        "ensure_snapshot",
        /fn ensure_snapshot\s*\(/,
      ),
    ],
  },
  {
    capability: "snapshot-identity",
    disposition: "reuse",
    sources: [
      source(
        "contracts/engine-sdk-workflow/main.tsp",
        "OmenaWorkspaceSnapshotIdV0",
        /model OmenaWorkspaceSnapshotIdV0/,
      ),
      source(
        "rust/crates/omena-query/src/sdk_workspace.rs",
        "OmenaWorkspaceSnapshotIdV0::from_revision",
        /OmenaWorkspaceSnapshotIdV0::from_revision/,
      ),
    ],
  },
  {
    capability: "resident-document-state",
    disposition: "extract-and-reuse",
    sources: [
      source(
        "rust/crates/omena-lsp-server/src/state.rs",
        "LspShellState",
        /pub struct LspShellState/,
      ),
      source(
        "rust/crates/omena-lsp-server/src/state.rs",
        "style_memo_host",
        /style_memo_host:\s*RefCell/,
      ),
    ],
  },
  {
    capability: "request-cancellation",
    disposition: "reuse",
    sources: [
      source(
        "rust/crates/omena-incremental/src/lib.rs",
        "IncrementalCancellationRegistryV0",
        /pub struct IncrementalCancellationRegistryV0/,
      ),
      source(
        "rust/crates/omena-lsp-server/src/message_loop.rs",
        "cancel_lsp_request",
        /fn cancel_lsp_request\s*\(/,
      ),
    ],
  },
  {
    capability: "watched-file-stream",
    disposition: "extract-and-reuse",
    sources: [
      source(
        "rust/crates/omena-lsp-server/src/message_loop.rs",
        "workspace/didChangeWatchedFiles",
        /workspace\/didChangeWatchedFiles/,
      ),
      source(
        "rust/crates/omena-lsp-server/src/state.rs",
        "watched_file_changes",
        /watched_file_changes:\s*Vec<LspWatchedFileChangeState>/,
      ),
    ],
  },
  {
    capability: "diagnostics-stream",
    disposition: "extract-and-reuse",
    sources: [
      source(
        "rust/crates/omena-lsp-server/src/diagnostics_scheduler.rs",
        "textDocument/publishDiagnostics",
        /textDocument\/publishDiagnostics/,
      ),
      source(
        "rust/crates/omena-query/src/sdk_workspace.rs",
        "execute_diagnostics",
        /pub fn execute_diagnostics\s*\(/,
      ),
    ],
  },
  {
    capability: "format-request",
    disposition: "build-adapter",
    sources: [
      source(
        "rust/crates/omena-cli/src/dispatch.rs",
        "format_sources",
        /Command::Fmt[\s\S]*?=>\s*format_sources\(/,
      ),
      source(
        "rust/crates/omena-cli/src/format.rs",
        "OmenaQueryPrettyFormatOptionsV0",
        /OmenaQueryPrettyFormatOptionsV0/,
      ),
    ],
  },
  {
    capability: "napi-workspace-session",
    disposition: "reuse",
    sources: [
      source(
        "rust/crates/omena-napi/src/sdk_workspace.rs",
        "OmenaNapiWorkspaceV0",
        /pub struct OmenaNapiWorkspaceV0/,
      ),
      source(
        "rust/crates/omena-napi/src/sdk_workspace.rs",
        "replace_style_sources_json",
        /pub fn replace_style_sources_json\s*\(/,
      ),
    ],
  },
  {
    capability: "bundler-host-protocol",
    disposition: "reuse",
    sources: [
      source(
        "contracts/engine-sdk-workflow/main.tsp",
        "OmenaBundlerHostCapabilitiesV0",
        /model OmenaBundlerHostCapabilitiesV0/,
      ),
      source(
        "rust/crates/omena-napi/src/lib.rs",
        "bundler_host_capabilities_json",
        /pub fn bundler_host_capabilities_json\s*\(/,
      ),
    ],
  },
  {
    capability: "resident-process-transport",
    disposition: "extract-and-reuse",
    sources: [
      source("rust/crates/engine-shadow-runner/src/main.rs", "run_daemon", /fn run_daemon\s*\(/),
      source(
        "server/engine-host-node/src/selected-query-backend.ts",
        "EngineShadowRunnerDaemon",
        /class EngineShadowRunnerDaemon/,
      ),
    ],
  },
  {
    capability: "config-snapshot",
    disposition: "reuse",
    sources: [
      source(
        "rust/crates/omena-cli/src/config/loader.rs",
        "LoadedOmenaConfig::config_content_digest",
        /pub\(crate\) config_content_digest:\s*Arc<str>/,
      ),
      source(
        "rust/crates/omena-cli/src/config/loader.rs",
        "ConfigCacheKey",
        /struct ConfigCacheKey/,
      ),
    ],
  },
];

const capabilities = probes
  .filter((probe) => probe.capability !== injectedMissingCapability)
  .map((probe): CapabilityRow => {
    for (const evidence of probe.sources) {
      assert.match(
        read(evidence.path),
        evidence.pattern,
        `${probe.capability}: missing ${evidence.symbol}`,
      );
    }
    return {
      capability: probe.capability,
      disposition: probe.disposition,
      evidence: probe.sources.map(({ path: sourcePath, symbol }) => ({ path: sourcePath, symbol })),
    };
  })
  .sort((left, right) => left.capability.localeCompare(right.capability));

for (const required of ["workspace-snapshot-session", "diagnostics-stream", "format-request"]) {
  assert.ok(
    capabilities.some((row) => row.capability === required),
    `required resident-workspace capability is absent: ${required}`,
  );
}

const inventory: InventorySnapshot = {
  schemaVersion: "0",
  product: "omena.omenad-substrate-inventory",
  capabilities,
};
const serialized = `${JSON.stringify(inventory, null, 2)}\n`;
if (write) {
  writeFileSync(path.join(repoRoot, snapshotPath), serialized);
} else {
  assert.equal(
    read(snapshotPath),
    serialized,
    `${snapshotPath} is stale; run this gate with --write`,
  );
}

process.stdout.write(serialized);

function source(sourcePath: string, symbol: string, pattern: RegExp) {
  return { path: sourcePath, symbol, pattern } as const;
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
