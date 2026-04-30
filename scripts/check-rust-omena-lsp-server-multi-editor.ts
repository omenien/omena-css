import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

interface RustOmenaLspServerBoundarySummary {
  readonly product: string;
  readonly thinClientEndpoint: {
    readonly commandOwner: string;
    readonly standalonePackage: string;
    readonly splitRepository: string;
    readonly cargoInstallCommand: string;
    readonly nodeFallbackAllowed: boolean;
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
const endpoint = rustSummary.thinClientEndpoint;
const distribution = rustSummary.multiEditorDistribution;

assert.equal(rustSummary.product, "omena-lsp-server.boundary");
assert.equal(distribution.product, "omena-lsp-server.multi-editor-distribution");
assert.equal(distribution.owner, "omena-lsp-server/distribution");
assert.equal(distribution.distributionModel, "standaloneRustLspServerWithThinEditorHosts");
assert.deepEqual(distribution.supportedEditors.toSorted(), ["neovim", "vscode", "zed"]);
assert.deepEqual(distribution.installSurfaces.toSorted(), [
  "cargoInstallOmenaLspServer",
  "repoLocalDistBin",
  "vsixBundledDistBinary",
]);
assert.ok(distribution.documentation.includes("client/src/extension.ts"));
assert.ok(distribution.documentation.includes("docs/clients/neovim.md"));
assert.ok(distribution.documentation.includes("docs/clients/zed.md"));
assert.ok(distribution.endpointPolicy.includes("standaloneRustServerIsPrimaryMultiEditorEndpoint"));
assert.ok(distribution.endpointPolicy.includes("nodeLspServerIsNotPrimaryEndpoint"));
assert.ok(distribution.endpointPolicy.includes("editorClientsDoNotImplementProviderSemantics"));
assert.ok(!rustSummary.nextDecouplingTargets.includes("multiEditorDistribution"));
assert.equal(endpoint.nodeFallbackAllowed, false);

const vscodeHost = readFileSync("client/src/extension.ts", "utf8");
assert.match(vscodeHost, /buildThinClientServerOptions/u);
assert.match(vscodeHost, /LanguageClient/u);
assert.doesNotMatch(
  vscodeHost,
  /dist\/server\/server\.js/u,
  "VS Code host must not keep the Node LSP server as the primary runtime",
);

for (const [label, docPath, editorSpecificPattern] of [
  ["neovim", "docs/clients/neovim.md", /vim\.lsp\.config/u],
  ["zed", "docs/clients/zed.md", /language_servers/u],
] as const) {
  const doc = readFileSync(docPath, "utf8");

  assert.match(doc, /omena-lsp-server/u, `${label}: must document the Rust LSP binary`);
  assert.match(
    doc,
    new RegExp(escapeRegExp(endpoint.commandOwner), "u"),
    `${label}: must document the repo-local Rust endpoint`,
  );
  assert.match(
    doc,
    new RegExp(escapeRegExp(endpoint.cargoInstallCommand), "u"),
    `${label}: must document crates.io install`,
  );
  assert.match(
    doc,
    new RegExp(escapeRegExp(endpoint.splitRepository), "u"),
    `${label}: must document the split repository`,
  );
  assert.match(doc, editorSpecificPattern, `${label}: must document editor configuration`);
  assert.doesNotMatch(
    doc,
    /node.+dist\/server\/server\.js/su,
    `${label}: should not keep the Node LSP server as the primary multi-editor endpoint`,
  );
}

process.stdout.write(
  [
    "validated omena-lsp-server multi-editor distribution:",
    `clients=${distribution.supportedEditors.join(",")}`,
    `package=${endpoint.standalonePackage}`,
    `endpoint=${endpoint.commandOwner}`,
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

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
