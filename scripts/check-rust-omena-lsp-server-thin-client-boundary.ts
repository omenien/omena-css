import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import {
  buildRustLspFileWatcherGlobs,
  buildThinClientRuntimeEndpoint,
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
assert.ok(rustEndpoint.hostResponsibilities.includes("declareStaticDocumentSelector"));
assert.ok(rustEndpoint.hostResponsibilities.includes("startLanguageClient"));
assert.ok(rustEndpoint.hostResponsibilities.includes("registerStaticFileWatchers"));
assert.ok(rustEndpoint.rustResponsibilities.includes("ownLspLifecycle"));
assert.ok(rustEndpoint.rustResponsibilities.includes("ownTsgoClientLifecycle"));
assert.ok(clientEndpoint.hostResponsibilities.includes("translateShowReferencesArguments"));
assert.ok(clientEndpoint.rustResponsibilities.includes("ownProviderExecution"));

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
