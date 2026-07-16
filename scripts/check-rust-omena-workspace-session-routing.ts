import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const routing = read("docs/workspace-session-routing.md");
const schema = read("rust/crates/omena-cli/src/config/schema.rs");
const daemon = read("rust/crates/omena-cli/src/daemon.rs");
const napi = read("rust/crates/omena-napi/src/sdk_workspace.rs");
const eslint = read("packages/eslint-plugin/lib/_shared.cjs");
const bundlerAdapter = read("packages/css-build-adapter/index.cjs");

const routes = [...routing.matchAll(/^\| `([^`]+)` \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$/gmu)].map(
  ([, consumer, owner, repeatedRoute, fallback]) => ({
    consumer,
    owner: owner.trim(),
    repeatedRoute: repeatedRoute.trim(),
    fallback: fallback.trim(),
  }),
);
assert.deepEqual(routes, [
  {
    consumer: "editor",
    owner: "`omena-lsp-server`",
    repeatedRoute: "LSP document and workspace state",
    fallback: "not applicable",
  },
  {
    consumer: "cli-one-shot",
    owner: "`omena-cli` and `omena-query`",
    repeatedRoute: "direct process execution",
    fallback: "not applicable",
  },
  {
    consumer: "cli-watch",
    owner: "`omena-query` through `omenad`",
    repeatedRoute: "loopback resident workspace session",
    fallback: "direct CLI execution",
  },
  {
    consumer: "eslint",
    owner: "`omena-query` through `@omena/napi`",
    repeatedRoute: "in-process `CachedWorkspace`",
    fallback: "direct CLI diagnostics when NAPI is unavailable",
  },
  {
    consumer: "bundler-host",
    owner: "`omena-query` through the existing bundler-host protocol",
    repeatedRoute: "NAPI or WASM adapter state",
    fallback: "adapter-selected NAPI/WASM compatibility route; no daemon hop",
  },
]);

assert.deepEqual(extractStructFields(schema, "OmenaWorkspaceSessionConfig"), [
  "enabled",
  "idle_timeout_ms",
  "request_deadline_ms",
  "max_response_bytes",
]);
for (const symbol of [
  "find_omena_config_for_path",
  "watch_session_settings",
  "if session_settings.enabled",
  "settings.idle_timeout_ms",
  "settings.limits",
  "process.detach()",
  '"watch-reconnect-sync"',
  '"directFallback"',
]) {
  assert.ok(daemon.includes(symbol), `watch routing does not consume ${symbol}`);
}
assert.ok(daemon.includes("MAX_TRANSPORT_LINE_BYTES as u64"));
assert.ok(!daemon.includes("WATCH_SESSION_LIMITS"), "watch limits must come from resolved config");

for (const symbol of ["CachedWorkspace", "workspace_session_cache_report_json"]) {
  assert.ok(napi.includes(symbol), `NAPI workspace route is missing ${symbol}`);
}
for (const symbol of ["new binding.CachedWorkspace", 'route: binding ? "napiSession" : "directCli"']) {
  assert.ok(eslint.includes(symbol), `ESLint workspace route is missing ${symbol}`);
}
for (const symbol of [
  "bundlerHostCapabilities",
  "resolveCssModuleForBundlerHostJson",
  'normalizeEngine(localNapiBinding, "napi")',
  'normalizeEngine(localWasmBinding, "wasm")',
]) {
  assert.ok(bundlerAdapter.includes(symbol), `bundler-host route is missing ${symbol}`);
}

for (const residual of [
  "multiple independent workspace roots",
  "remote or multi-machine workspace sessions",
  "authenticated or encrypted daemon transport",
  "replacing editor LSP lifecycle",
  "mandatory for CI",
]) {
  assert.ok(routing.includes(residual), `routing residual is not explicit: ${residual}`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-workspace-session-routing",
      routeCount: routes.length,
      configFieldCount: extractStructFields(schema, "OmenaWorkspaceSessionConfig").length,
      residualCount: 5,
    },
    null,
    2,
  )}\n`,
);

function extractStructFields(source: string, structName: string): string[] {
  const match = source.match(new RegExp(`struct ${structName} \\{([\\s\\S]*?)\\n\\}`, "u"));
  assert.ok(match, `missing struct ${structName}`);
  return [...match[1].matchAll(/pub\(crate\)\s+(\w+)\s*:/gu)].map((entry) => entry[1]);
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
