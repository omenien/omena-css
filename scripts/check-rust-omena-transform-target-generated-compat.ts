import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const expectedQuorumSources = ["caniuse", "web-features", "mdn-bcd"] as const;
const expectedBrowsers = [
  "chrome",
  "edge",
  "firefox",
  "safari",
  "ios_saf",
  "opera",
  "op_mob",
  "and_chr",
  "and_ff",
  "samsung",
  "android",
] as const;

interface SpecSourcePinsV0 {
  readonly refreshedAt: string;
  readonly generatedDataReviewGate: {
    readonly humanReviewRequired: boolean;
    readonly changedGeneratedDataRequiresReview: boolean;
    readonly autoMergeAllowed: boolean;
  };
  readonly sources: readonly {
    readonly name: string;
    readonly package: string;
    readonly version: string;
  }[];
}

type TomlValue = string | number | string[];
type TomlRecord = Record<string, TomlValue>;

interface ParsedTomlTablesV0 {
  readonly root: TomlRecord;
  readonly tables: readonly TomlRecord[];
}

const specSources = readJson<SpecSourcePinsV0>(
  "rust/crates/omena-spec-audit/data/spec-sources.json",
);
const browserThresholdData = parseTomlWithRepeatedTable(
  readText("rust/crates/omena-transform-target/data/browser-thresholds.toml"),
  "threshold",
);
const passFeatureBindingData = parseTomlWithRepeatedTable(
  readText("rust/crates/omena-transform-target/data/pass-feature-bindings.toml"),
  "binding",
);
const cargoToml = readText("rust/Cargo.toml");

assert.equal(browserThresholdData.root.schema_version, "0");
assert.equal(browserThresholdData.root.product, "omena-transform-target.browser-thresholds");
assert.equal(passFeatureBindingData.root.schema_version, "0");
assert.equal(passFeatureBindingData.root.product, "omena-transform-target.pass-feature-bindings");
assert.equal(browserThresholdData.root.refreshed_at, specSources.refreshedAt);
assert.equal(passFeatureBindingData.root.refreshed_at, specSources.refreshedAt);
assert.equal(
  browserThresholdData.root.quorum_min_sources,
  expectedQuorumSources.length,
  "generated compat data must require all three source families",
);
assert.equal(specSources.generatedDataReviewGate.humanReviewRequired, true);
assert.equal(specSources.generatedDataReviewGate.changedGeneratedDataRequiresReview, true);
assert.equal(specSources.generatedDataReviewGate.autoMergeAllowed, false);

const sourceNames = new Set(specSources.sources.map((source) => source.name));
assert.ok(sourceNames.has("web-features"), "compat source pins must include web-features");
assert.ok(
  sourceNames.has("mdn-browser-compat-data"),
  "compat source pins must include MDN browser compatibility data",
);
assert.match(
  cargoToml,
  /browserslist\s*=\s*\{\s*package\s*=\s*"oxc-browserslist"\s*,\s*version\s*=\s*"[^"]+"\s*\}/,
  "compat resolver source must pin oxc-browserslist in the Rust workspace",
);

const thresholdsByTable = new Map<string, TomlRecord[]>();
for (const threshold of browserThresholdData.tables) {
  assertString(threshold.table, "threshold.table");
  assertString(threshold.browser, "threshold.browser");
  assertString(threshold.caniuse_key, "threshold.caniuse_key");
  assertNumber(threshold.min_major, "threshold.min_major");
  assertNumber(threshold.min_minor, "threshold.min_minor");
  assert.deepEqual(
    threshold.source_quorum,
    expectedQuorumSources,
    `threshold ${threshold.table}/${threshold.browser} must carry measured three-source quorum`,
  );
  assert.equal(
    threshold.last_verified,
    browserThresholdData.root.refreshed_at,
    `threshold ${threshold.table}/${threshold.browser} must use the current compat refresh stamp`,
  );
  assert.ok(
    expectedBrowsers.includes(threshold.browser as (typeof expectedBrowsers)[number]),
    `unexpected browser threshold row ${threshold.browser}`,
  );
  pushMapValue(thresholdsByTable, threshold.table, threshold);
}

assert.ok(thresholdsByTable.size > 0, "browser threshold data must include feature tables");
for (const [table, thresholds] of thresholdsByTable) {
  const browsers = thresholds.map((threshold) => threshold.browser);
  assert.deepEqual(
    browsers,
    [...expectedBrowsers],
    `feature table ${table} must retain the generated browser support matrix shape`,
  );
  assert.equal(
    new Set(thresholds.map((threshold) => threshold.caniuse_key)).size,
    1,
    `feature table ${table} must map to a single caniuse feature key`,
  );
}

const mappedTables = new Set<string>();
for (const binding of passFeatureBindingData.tables) {
  assertString(binding.pass_id, "binding.pass_id");
  assertString(binding.support_table, "binding.support_table");
  assertStringArray(binding.caniuse_keys, "binding.caniuse_keys");
  mappedTables.add(binding.support_table);

  const thresholds = thresholdsByTable.get(binding.support_table);
  assert.ok(thresholds, `binding ${binding.pass_id} maps unknown table ${binding.support_table}`);
  const thresholdKeys = new Set(thresholds.map((threshold) => threshold.caniuse_key));
  for (const key of binding.caniuse_keys) {
    assert.ok(
      thresholdKeys.has(key),
      `binding ${binding.pass_id} key ${key} is absent from ${binding.support_table}`,
    );
  }
}

assert.deepEqual(
  [...thresholdsByTable.keys()].toSorted(),
  [...mappedTables].toSorted(),
  "every generated threshold table must be routed through a pass-feature binding",
);

function readJson<T>(relativePath: string): T {
  return JSON.parse(readText(relativePath)) as T;
}

function readText(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function parseTomlWithRepeatedTable(source: string, tableName: string): ParsedTomlTablesV0 {
  const root: TomlRecord = {};
  const tables: TomlRecord[] = [];
  let current: TomlRecord | null = null;
  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.split("#")[0]?.trim() ?? "";
    if (!line) continue;
    if (line === `[[${tableName}]]`) {
      current = {};
      tables.push(current);
      continue;
    }
    const [key, value] = parseTomlAssignment(line);
    const target = current ?? root;
    target[key] = parseTomlValue(value);
  }
  return { root, tables };
}

function parseTomlAssignment(line: string): readonly [string, string] {
  const index = line.indexOf("=");
  assert.notEqual(index, -1, `invalid TOML assignment: ${line}`);
  return [line.slice(0, index).trim(), line.slice(index + 1).trim()];
}

function parseTomlValue(value: string): TomlValue {
  if (value.startsWith("[") && value.endsWith("]")) {
    const body = value.slice(1, -1).trim();
    if (!body) return [];
    return body.split(",").map((item) => parseTomlString(item.trim()));
  }
  if (value.startsWith('"') && value.endsWith('"')) return parseTomlString(value);
  const numberValue = Number(value);
  assert.ok(Number.isInteger(numberValue), `unsupported TOML value: ${value}`);
  return numberValue;
}

function parseTomlString(value: string): string {
  assert.ok(value.startsWith('"') && value.endsWith('"'), `expected TOML string: ${value}`);
  return value.slice(1, -1);
}

function pushMapValue<K, V>(map: Map<K, V[]>, key: K, value: V): void {
  const existing = map.get(key);
  if (existing) {
    existing.push(value);
    return;
  }
  map.set(key, [value]);
}

function assertString(value: TomlValue | undefined, label: string): asserts value is string {
  assert.equal(typeof value, "string", `${label} must be a string`);
}

function assertNumber(value: TomlValue | undefined, label: string): asserts value is number {
  assert.equal(typeof value, "number", `${label} must be a number`);
}

function assertStringArray(
  value: TomlValue | undefined,
  label: string,
): asserts value is string[] {
  assert.ok(Array.isArray(value), `${label} must be a string array`);
  for (const item of value) assert.equal(typeof item, "string", `${label} item must be a string`);
}
