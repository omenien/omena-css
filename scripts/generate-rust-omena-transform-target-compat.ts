import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const sourcePinsPath = "rust/crates/omena-spec-audit/data/spec-sources.json";
const selectionPath = "rust/crates/omena-transform-target/data/compat-feature-selections.json";
const browserThresholdsPath = "rust/crates/omena-transform-target/data/browser-thresholds.toml";
const passFeatureBindingsPath = "rust/crates/omena-transform-target/data/pass-feature-bindings.toml";
const generatorPath = "scripts/generate-rust-omena-transform-target-compat.ts";

interface SpecSourcePinsV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly refreshedAt: string;
  readonly sources: readonly {
    readonly name: string;
    readonly package: string;
    readonly version: string;
  }[];
}

interface CompatFeatureSelectionsV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePolicy: {
    readonly sourcePinProduct: string;
    readonly refreshedAtSource: string;
    readonly caniuseResolver: {
      readonly workspaceDependency: string;
      readonly cargoPackage: string;
    };
    readonly requiredSourceQuorum: readonly string[];
  };
  readonly features: readonly CompatFeatureSelectionV0[];
}

interface CompatFeatureSelectionV0 {
  readonly table: string;
  readonly passId: string;
  readonly caniuseKeys: readonly string[];
  readonly sourceKeys: Record<string, string>;
  readonly sourceQuorum: readonly string[];
  readonly thresholds: readonly CompatBrowserThresholdSelectionV0[];
}

interface CompatBrowserThresholdSelectionV0 {
  readonly browser: string;
  readonly minMajor: number;
  readonly minMinor: number;
}

const specSources = readJson<SpecSourcePinsV0>(sourcePinsPath);
const selections = readJson<CompatFeatureSelectionsV0>(selectionPath);
validateInputs(specSources, selections);

const browserThresholdsSource = renderBrowserThresholdsToml(specSources, selections);
const passFeatureBindingsSource = renderPassFeatureBindingsToml(specSources, selections);

if (checkOnly) {
  assert.equal(readText(browserThresholdsPath), browserThresholdsSource, `${browserThresholdsPath} is stale`);
  assert.equal(
    readText(passFeatureBindingsPath),
    passFeatureBindingsSource,
    `${passFeatureBindingsPath} is stale`,
  );
} else if (writeMode) {
  writeText(browserThresholdsPath, browserThresholdsSource);
  writeText(passFeatureBindingsPath, passFeatureBindingsSource);
}

process.stdout.write(
  stableJson({
    product: "omena-transform-target.compat-generator",
    mode: checkOnly ? "check" : "write",
    sourcePins: sourcePinsPath,
    selectionPath,
    generatedFiles: [browserThresholdsPath, passFeatureBindingsPath],
    featureCount: selections.features.length,
    thresholdCount: selections.features.reduce(
      (count, feature) => count + feature.thresholds.length,
      0,
    ),
    refreshedAt: specSources.refreshedAt,
  }),
);

function validateInputs(
  specSources: SpecSourcePinsV0,
  selections: CompatFeatureSelectionsV0,
): void {
  assert.equal(specSources.schemaVersion, "0");
  assert.equal(specSources.product, selections.sourcePolicy.sourcePinProduct);
  assert.equal(selections.schemaVersion, "0");
  assert.equal(selections.product, "omena-transform-target.compat-feature-selections");
  assert.equal(selections.sourcePolicy.refreshedAtSource, sourcePinsPath);
  assert.equal(selections.sourcePolicy.caniuseResolver.workspaceDependency, "browserslist");
  assert.equal(selections.sourcePolicy.caniuseResolver.cargoPackage, "oxc-browserslist");
  assert.deepEqual(selections.sourcePolicy.requiredSourceQuorum, [
    "caniuse",
    "web-features",
    "mdn-bcd",
  ]);

  const sourceNames = new Set(specSources.sources.map((source) => source.name));
  assert.ok(sourceNames.has("web-features"), "web-features source pin is required");
  assert.ok(sourceNames.has("mdn-browser-compat-data"), "MDN BCD source pin is required");
  assert.match(
    readText("rust/Cargo.toml"),
    /browserslist\s*=\s*\{\s*package\s*=\s*"oxc-browserslist"\s*,\s*version\s*=\s*"[^"]+"\s*\}/,
    "caniuse resolution must remain pinned through oxc-browserslist",
  );

  assert.ok(selections.features.length > 0, "at least one compat feature is required");
  const tables = new Set<string>();
  const passIds = new Set<string>();
  for (const feature of selections.features) {
    assert.match(feature.table, /^[a-z][a-z0-9_]*$/u, `invalid table ${feature.table}`);
    assert.ok(!tables.has(feature.table), `duplicate compat table ${feature.table}`);
    tables.add(feature.table);
    assert.ok(feature.passId.length > 0, `${feature.table} passId is required`);
    assert.ok(!passIds.has(feature.passId), `duplicate compat passId ${feature.passId}`);
    passIds.add(feature.passId);
    assert.ok(feature.caniuseKeys.length > 0, `${feature.table} caniuseKeys required`);
    assert.deepEqual(
      Object.keys(feature.sourceKeys).toSorted(),
      [...selections.sourcePolicy.requiredSourceQuorum].toSorted(),
      `${feature.table} must carry a cross-source feature key map`,
    );
    assert.equal(
      feature.sourceKeys.caniuse,
      feature.caniuseKeys[0],
      `${feature.table} primary caniuse key must match sourceKeys.caniuse`,
    );
    for (const source of selections.sourcePolicy.requiredSourceQuorum) {
      assert.ok(
        feature.sourceKeys[source]?.length > 0,
        `${feature.table} missing source key for ${source}`,
      );
    }
    assert.deepEqual(
      feature.sourceQuorum,
      selections.sourcePolicy.requiredSourceQuorum,
      `${feature.table} must retain full source quorum`,
    );
    assert.deepEqual(
      feature.thresholds.map((threshold) => threshold.browser),
      expectedBrowserOrder(),
      `${feature.table} must retain stable browser row order`,
    );
    for (const threshold of feature.thresholds) {
      assert.ok(Number.isInteger(threshold.minMajor), `${feature.table}/${threshold.browser} major`);
      assert.ok(Number.isInteger(threshold.minMinor), `${feature.table}/${threshold.browser} minor`);
      assert.ok(threshold.minMajor >= 0, `${feature.table}/${threshold.browser} major`);
      assert.ok(threshold.minMinor >= 0, `${feature.table}/${threshold.browser} minor`);
    }
  }
}

function renderBrowserThresholdsToml(
  specSources: SpecSourcePinsV0,
  selections: CompatFeatureSelectionsV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.browser-thresholds"',
    `refreshed_at = ${quoteToml(specSources.refreshedAt)}`,
    `quorum_min_sources = ${selections.sourcePolicy.requiredSourceQuorum.length}`,
    "",
  ];

  for (const feature of selections.features) {
    const caniuseKey = feature.caniuseKeys[0];
    assert.ok(caniuseKey, `${feature.table} needs a primary caniuse key`);
    for (const threshold of feature.thresholds) {
      lines.push(
        "[[threshold]]",
        `table = ${quoteToml(feature.table)}`,
        `browser = ${quoteToml(threshold.browser)}`,
        `min_major = ${threshold.minMajor}`,
        `min_minor = ${threshold.minMinor}`,
        `caniuse_key = ${quoteToml(caniuseKey)}`,
        `source_quorum = ${tomlStringArray(feature.sourceQuorum)}`,
        `last_verified = ${quoteToml(specSources.refreshedAt)}`,
        "",
      );
    }
  }

  return `${lines.join("\n").trimEnd()}\n`;
}

function renderPassFeatureBindingsToml(
  specSources: SpecSourcePinsV0,
  selections: CompatFeatureSelectionsV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.pass-feature-bindings"',
    `refreshed_at = ${quoteToml(specSources.refreshedAt)}`,
    "",
  ];

  for (const feature of selections.features) {
    lines.push(
      "[[binding]]",
      `pass_id = ${quoteToml(feature.passId)}`,
      `caniuse_keys = ${tomlStringArray(feature.caniuseKeys)}`,
      `support_table = ${quoteToml(feature.table)}`,
      "",
    );
  }

  return `${lines.join("\n").trimEnd()}\n`;
}

function expectedBrowserOrder(): readonly string[] {
  return [
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
  ];
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(readText(relativePath)) as T;
}

function readText(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function writeText(relativePath: string, source: string): void {
  writeFileSync(path.join(repoRoot, relativePath), source);
}

function quoteToml(value: string): string {
  return JSON.stringify(value);
}

function tomlStringArray(values: readonly string[]): string {
  return `[${values.map(quoteToml).join(", ")}]`;
}

function stableJson(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
