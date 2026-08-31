import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
  readonly scripts: Readonly<Record<string, string>>;
};
const ledger = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/frame-refresh-runtime-disposition.json"), "utf8"),
) as Readonly<Record<string, unknown>>;
const injectRuntime = process.argv.includes("--inject-runtime");
const injectPublicCache = process.argv.includes("--inject-public-cache");
const injectGateRetarget = process.argv.includes("--inject-gate-retarget");

const rustSources = evidenceScanSurface
  .execFileSync("git", ["ls-files", "rust/crates/*/src/*.rs", "rust/crates/*/src/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
  .trim()
  .split("\n")
  .filter(Boolean)
  .map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
if (injectRuntime) {
  rustSources.push({
    relativePath: "rust/crates/omena-lsp-server/src/injected_runtime.rs",
    source: "struct FrameAwareRefreshRuntimeV0;",
  });
}
if (injectPublicCache) {
  rustSources.push({
    relativePath: "rust/crates/omena-query/src/injected_cache.rs",
    source: "pub struct OmenaQueryStyleFrameRefreshParseCacheV0;",
  });
}

const retiredSymbols = [
  "FrameAwareRefreshRuntimeV0",
  "FrameAwareStyleModuleInputV0",
  "OmenaQueryStyleFrameRefreshParseCacheV0",
  "OmenaQueryStyleFrameRefreshFactsV0",
  "summarize_omena_query_style_frame_refresh_facts_with_reuse",
] as const;
const symbolCounts = Object.fromEntries(
  retiredSymbols.map((symbol) => [
    symbol,
    rustSources.reduce((count, file) => count + countLiteral(file.source, symbol), 0),
  ]),
);
for (const symbol of retiredSymbols) {
  assert.equal(symbolCounts[symbol], 0, `${symbol} must remain retired from all Rust sources`);
}

let latencyGate = packageJson.scripts["check:rust-m4-alpha-frame-refresh-latency"] ?? "";
if (injectGateRetarget)
  latencyGate = latencyGate.replace("fixed_workspace_comparison", "runtime_refresh");
assert.match(latencyGate, /fixed_workspace_comparison_reports_work_reduction_when_enabled/);
assert.doesNotMatch(latencyGate, /runtime_refresh|FrameAwareRefreshRuntimeV0/);

const cliSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-cli/src/postcss_compat.rs"),
  "utf8",
);
const parseErrorBody = extractFunctionBody(cliSource, "postcss_parse_error_count");
assert.match(parseErrorBody, /summarize_omena_query_omena_parser_style_facts/);
assert.match(parseErrorBody, /\.parser_error_count/);
assert.doesNotMatch(parseErrorBody, /cache|reuse/iu);

assert.equal(ledger.disposition, "retired");
assert.equal(ledger.preservedGate, "check:rust-m4-alpha-frame-refresh-latency");
assert.equal(ledger.preservedGateMeasurement, "frame-selection work reduction");
assert.equal(ledger.liveCliParsePath, "summarize_omena_query_omena_parser_style_facts");
assert.match(String(ledger.reason), /constructed only by an unregistered test/);
assert.match(String(ledger.dirtinessAfterParseDisposition), /retired with the unowned runtime/);

const semverRegister = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/omena-rust-semver-intent.json"), "utf8"),
) as {
  readonly intents: readonly {
    readonly crate: string;
    readonly expectedFailures: readonly { readonly lint: string }[];
  }[];
};
for (const crateName of ["omena-lsp-server", "omena-query"]) {
  const intent = semverRegister.intents.find((entry) => entry.crate === crateName);
  assert.ok(intent, `${crateName} requires an explicit 0.6.0 retirement intent`);
  assert.ok(
    intent.expectedFailures.some((failure) => failure.lint === "struct_missing"),
    `${crateName} must bind its removed cache/runtime structs to cargo-semver-checks`,
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-lsp-server.frame-refresh-runtime-disposition-check",
      runtimeOwnerCount: symbolCounts.FrameAwareRefreshRuntimeV0,
      publicParseCacheCount: symbolCounts.OmenaQueryStyleFrameRefreshParseCacheV0,
      preservedGate: ledger.preservedGate,
      preservedMeasurement: ledger.preservedGateMeasurement,
      liveCliParsePath: ledger.liveCliParsePath,
      dirtinessAfterParseDisposition: ledger.dirtinessAfterParseDisposition,
    },
    null,
    2,
  )}\n`,
);

function countLiteral(source: string, value: string): number {
  return source.split(value).length - 1;
}

function extractFunctionBody(source: string, functionName: string): string {
  const match = new RegExp(`\\bfn\\s+${functionName}\\b`).exec(source);
  assert.ok(match, `missing function ${functionName}`);
  const open = source.indexOf("{", match.index);
  assert.ok(open >= 0, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated body for ${functionName}`);
}
