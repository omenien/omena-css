import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const read = (relativePath: string) => fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
const pluginApi = read("rust/crates/omena-query-transform-runner/src/plugin_api.rs");
const pluginRegistry = read("rust/crates/omena-query-transform-runner/src/plugins/mod.rs");
const bundleHostPlugin = read(
  "rust/crates/omena-query-transform-runner/src/plugins/bundle_host.rs",
);
const semanticPlugin = read(
  "rust/crates/omena-query-transform-runner/src/plugins/semantic_observation.rs",
);
const adapter = read("packages/css-build-adapter/index.cjs");
const vite = read("packages/vite-plugin/index.cjs");
const bundlerProtocol = read("rust/crates/omena-query/src/bundler_host.rs");
const residualLedger = JSON.parse(read("rust/omena-bundler-host-residual-ledger.json")) as {
  readonly product: string;
  readonly entries: readonly {
    readonly id: string;
    readonly status: string;
    readonly owner: string;
    readonly reason: string;
  }[];
};

assert.match(pluginApi, /pub enum PluginKindV0\s*\{\s*Transform,\s*BundleHost,\s*\}/su);
assert.match(pluginApi, /pub kind: PluginKindV0,/u);
assert.equal(
  [...`${bundleHostPlugin}\n${semanticPlugin}`.matchAll(/kind:\s*PluginKindV0::BundleHost/gu)]
    .length,
  1,
  "exactly one built-in bundle host kind must be registered",
);
assert.ok(pluginRegistry.includes("&VITE_BUNDLE_HOST_PLUGIN"));
assert.ok(vite.includes('require("@omena/css-build-adapter")'));
assert.ok(adapter.includes("resolveCssModuleForBundlerHost"));
assert.ok(
  !vite.includes("resolveCssModuleForBundlerHost"),
  "Vite must consume the shared adapter instead of opening a second native host entry point",
);
assert.ok(!bundleHostPlugin.includes("OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0"));
assert.ok(!bundlerProtocol.includes("OMENA_PLUGIN_ABI_VERSION_V0"));
assert.equal(residualLedger.product, "omena.bundler-host-residual-ledger");
assert.deepEqual(residualLedger.entries.map(({ id }) => id).toSorted(), [
  "chunk-graph",
  "cross-chunk-css-order",
  "rspack-host",
  "source-map-passthrough-depth",
  "ssr-module-federation",
  "webpack-host",
]);
for (const entry of residualLedger.entries) {
  assert.ok(entry.status.length > 0, `${entry.id} must declare a status`);
  assert.ok(entry.owner.length > 0, `${entry.id} must declare an owner`);
  assert.ok(entry.reason.length > 0, `${entry.id} must explain why it remains residual`);
}
assert.deepEqual(
  residualLedger.entries.find(({ id }) => id === "chunk-graph"),
  {
    id: "chunk-graph",
    status: "unowned",
    owner: "unowned",
    reason:
      "No current component owns a chunk graph, so chunk-level host behavior must remain unavailable.",
  },
);

process.stdout.write(
  `${JSON.stringify({
    schemaVersion: "0",
    product: "js-bundler-host.plugin-kind",
    pluginKinds: ["transform", "bundleHost"],
    bundleHostRegistrationCount: 1,
    directJsBoundaryFileCount: 1,
    versionAuthoritiesDistinct: true,
    residualEntryCount: residualLedger.entries.length,
  })}\n`,
);
