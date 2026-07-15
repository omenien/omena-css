import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

type Surface = "napi" | "wasm" | "cli" | "lsp";
type ErrorClass =
  | "input"
  | "workspace"
  | "resolution"
  | "analysis"
  | "transform"
  | "unsupported"
  | "internal"
  | "unknown";

interface SourceSpec {
  readonly surface: Surface;
  readonly sourcePath: string;
  readonly emitter: string;
  readonly pattern: RegExp;
  readonly testCutoff?: string;
}

interface DiscoveredSite {
  readonly siteId: string;
  readonly surface: Surface;
  readonly method: string;
  readonly emitter: string;
  readonly ordinal: number;
  readonly sourcePath: string;
  readonly line: number;
  readonly source: string;
}

interface MappingRow extends DiscoveredSite {
  readonly errorClass: ErrorClass;
}

interface ErrorMappingCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-sdk.error-mapping-census";
  readonly sources: readonly string[];
  readonly summary: {
    readonly siteCount: number;
    readonly surfaceCounts: Readonly<Record<Surface, number>>;
    readonly classCounts: Readonly<Record<ErrorClass, number>>;
  };
  readonly rows: readonly MappingRow[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-sdk-error-mapping-census.json");
const writeMode = process.argv.includes("--write");
const sourceSpecs: readonly SourceSpec[] = [
  {
    surface: "napi",
    sourcePath: "rust/crates/omena-napi/src/lib.rs",
    emitter: "napi-error",
    pattern: /napi::Error::(?:from_reason|from_status|new)/u,
    testCutoff: "#[cfg(test)]",
  },
  {
    surface: "napi",
    sourcePath: "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
    emitter: "napi-boundary-error",
    pattern: /napi::Error::(?:from_reason|from_status|new)/u,
  },
  {
    surface: "napi",
    sourcePath: "rust/crates/omena-napi/src/sdk_workspace.rs",
    emitter: "typed-sdk-error",
    pattern: /(?:OmenaError::new|napi::Error::from_reason)/u,
  },
  {
    surface: "wasm",
    sourcePath: "rust/crates/omena-wasm/src/lib.rs",
    emitter: "wasm-js-error",
    pattern: /JsValue::from_str/u,
    testCutoff: "#[cfg(test)]",
  },
  {
    surface: "wasm",
    sourcePath: "rust/crates/omena-wasm/src/sdk_workspace.rs",
    emitter: "typed-sdk-error",
    pattern: /(?:OmenaError::new|JsValue::from_str)/u,
  },
  {
    surface: "cli",
    sourcePath: "rust/crates/omena-cli/src/main.rs",
    emitter: "stderr-exit-failure",
    pattern: /eprintln!\("\{error\}"\)/u,
    testCutoff: "#[cfg(test)]",
  },
  {
    surface: "cli",
    sourcePath: "rust/crates/omena-cli/src/output.rs",
    emitter: "typed-json-envelope-error",
    pattern: /OmenaError::new/u,
    testCutoff: "#[cfg(test)]",
  },
  {
    surface: "cli",
    sourcePath: "rust/crates/omena-cli/src/sdk.rs",
    emitter: "typed-sdk-error",
    pattern: /OmenaError::new/u,
    testCutoff: "#[cfg(test)]",
  },
  {
    surface: "lsp",
    sourcePath: "rust/crates/omena-lsp-server/src/bin/omena-lsp-server.rs",
    emitter: "io-error",
    pattern: /(?:io|std::io)::Error::other/u,
  },
  {
    surface: "lsp",
    sourcePath: "rust/crates/omena-lsp-server/src/boundary.rs",
    emitter: "startup-error-capability",
    pattern: /"surfaceStartupErrors"/u,
  },
  {
    surface: "lsp",
    sourcePath: "rust/crates/omena-lsp-server/src/sdk_workflow.rs",
    emitter: "typed-sdk-error",
    pattern: /OmenaError::new/u,
    testCutoff: "#[cfg(test)]",
  },
];

const discovered = sourceSpecs.flatMap(scanSource);
if (process.env.OMENA_SDK_ERROR_MAPPING_TEST_INJECT_SITE === "1") {
  discovered.push({
    siteId: "napi:injected_boundary:error-injection:1",
    surface: "napi",
    method: "injected_boundary",
    emitter: "error-injection",
    ordinal: 1,
    sourcePath: "rust/crates/omena-napi/src/lib.rs",
    line: 0,
    source: "injected boundary error",
  });
}
assert.ok(discovered.length > 0, "error-emission census is empty");
for (const surface of ["napi", "wasm", "cli", "lsp"] as const) {
  assert.ok(
    discovered.some((site) => site.surface === surface),
    `${surface} error-emission census is empty`,
  );
}

const existing = readExistingCensus();
const existingById = new Map(existing?.rows.map((row) => [row.siteId, row]) ?? []);
assert.equal(
  existingById.size,
  existing?.rows.length ?? 0,
  "mapping table contains duplicate site ids",
);
const validClasses = new Set<ErrorClass>([
  "input",
  "workspace",
  "resolution",
  "analysis",
  "transform",
  "unsupported",
  "internal",
  "unknown",
]);
for (const row of existing?.rows ?? []) {
  assert.ok(validClasses.has(row.errorClass), `invalid OmenaError class for ${row.siteId}`);
}

if (!writeMode) {
  assert.ok(existing, "missing error mapping census; run this check with --write");
  const discoveredIds = new Set(discovered.map((site) => site.siteId));
  const missing = discovered.filter((site) => !existingById.has(site.siteId));
  const stale = existing.rows.filter((row) => !discoveredIds.has(row.siteId));
  assert.deepEqual(
    missing.map((site) => site.siteId),
    [],
    "scan-discovered error sites require an OmenaError mapping",
  );
  assert.deepEqual(
    stale.map((row) => row.siteId),
    [],
    "error mapping table contains sites that no longer exist",
  );
}

const rows = discovered.map(
  (site): MappingRow => ({
    ...site,
    errorClass: existingById.get(site.siteId)?.errorClass ?? classifySite(site),
  }),
);
const census = buildCensus(rows);
const serialized = `${JSON.stringify(census, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.equal(
    fs.readFileSync(censusPath, "utf8"),
    serialized,
    "error mapping census is stale; regenerate and review the scan-derived mapping table",
  );
}

const boundaryContract = fs.readFileSync(
  path.join(repoRoot, "contracts/engine-napi/main.tsp"),
  "utf8",
);
const errorAdapter = fs.readFileSync(
  path.join(repoRoot, "rust/crates/omena-query/src/sdk_error.rs"),
  "utf8",
);
for (const encoding of ["parse-error", "serialize-error", "unsupported-mode"]) {
  assert.ok(boundaryContract.includes(`"${encoding}"`), `missing FFI error encoding ${encoding}`);
  assert.ok(errorAdapter.includes(`"${encoding}"`), `missing OmenaError adapter for ${encoding}`);
}
assert.ok(
  errorAdapter.includes("OmenaErrorClassV0::Unknown"),
  "adapter must fail closed to Unknown",
);

process.stdout.write(
  `Omena SDK error mapping census OK: ${census.summary.siteCount} sites (${Object.entries(
    census.summary.surfaceCounts,
  )
    .map(([surface, count]) => `${surface}=${count}`)
    .join(", ")})\n`,
);

function scanSource(spec: SourceSpec): DiscoveredSite[] {
  const absolutePath = path.join(repoRoot, spec.sourcePath);
  let source = fs.readFileSync(absolutePath, "utf8");
  if (spec.testCutoff) {
    const cutoff = source.indexOf(spec.testCutoff);
    if (cutoff >= 0) source = source.slice(0, cutoff);
  }
  const lines = source.split("\n");
  const ordinals = new Map<string, number>();
  const rows: DiscoveredSite[] = [];
  let method = "module";
  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();
    const functionMatch = /^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)/u.exec(
      trimmed,
    );
    if (functionMatch) method = functionMatch[1];
    if (trimmed.startsWith("//") || !spec.pattern.test(trimmed)) continue;
    const ordinalKey = `${method}:${spec.emitter}`;
    const ordinal = (ordinals.get(ordinalKey) ?? 0) + 1;
    ordinals.set(ordinalKey, ordinal);
    rows.push({
      siteId: `${spec.surface}:${method}:${spec.emitter}:${ordinal}`,
      surface: spec.surface,
      method,
      emitter: spec.emitter,
      ordinal,
      sourcePath: spec.sourcePath,
      line: index + 1,
      source: trimmed,
    });
  }
  return rows;
}

function classifySite(site: DiscoveredSite): ErrorClass {
  const text = `${site.method} ${site.source}`.toLowerCase();
  if (site.emitter === "startup-error-capability" || site.emitter === "io-error") return "internal";
  if (site.emitter === "stderr-exit-failure") return "unknown";
  if (text.includes("unsupported") || text.includes("external_module_mode")) return "unsupported";
  if (text.includes("serialize") || text.includes("to_json") || text.includes("to_js_value")) {
    return "internal";
  }
  if (text.includes("parse") || text.includes("from_value")) return "input";
  if (text.includes("resolve") || text.includes("resolution")) return "resolution";
  if (text.includes("diagnostic") || text.includes("analy")) return "analysis";
  if (text.includes("transform") || text.includes("build")) return "transform";
  if (text.includes("read") || text.includes("workspace") || text.includes("candidate")) {
    return "workspace";
  }
  return "unknown";
}

function buildCensus(rows: readonly MappingRow[]): ErrorMappingCensus {
  const surfaceCounts = zeroSurfaceCounts();
  const classCounts = zeroClassCounts();
  for (const row of rows) {
    surfaceCounts[row.surface] += 1;
    classCounts[row.errorClass] += 1;
  }
  return {
    schemaVersion: "0",
    product: "omena-sdk.error-mapping-census",
    sources: sourceSpecs.map((spec) => spec.sourcePath),
    summary: { siteCount: rows.length, surfaceCounts, classCounts },
    rows,
  };
}

function readExistingCensus(): ErrorMappingCensus | null {
  if (!fs.existsSync(censusPath)) return null;
  return JSON.parse(fs.readFileSync(censusPath, "utf8")) as ErrorMappingCensus;
}

function zeroSurfaceCounts(): Record<Surface, number> {
  return { napi: 0, wasm: 0, cli: 0, lsp: 0 };
}

function zeroClassCounts(): Record<ErrorClass, number> {
  return {
    input: 0,
    workspace: 0,
    resolution: 0,
    analysis: 0,
    transform: 0,
    unsupported: 0,
    internal: 0,
    unknown: 0,
  };
}
