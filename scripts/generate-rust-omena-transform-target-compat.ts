import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const sourcePinsPath = "rust/crates/omena-spec-audit/data/spec-sources.json";
const specManifestPath = "rust/crates/omena-spec-audit/data/omena-spec-manifest.json";
const selectionPath = "rust/crates/omena-transform-target/data/compat-feature-selections.json";
const browserThresholdsPath = "rust/crates/omena-transform-target/data/browser-thresholds.toml";
const passFeatureBindingsPath =
  "rust/crates/omena-transform-target/data/pass-feature-bindings.toml";
const generatorPath = "scripts/generate-rust-omena-transform-target-compat.ts";

interface SpecSourcePinsV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly refreshedAt: string;
  readonly refreshPolicy: {
    readonly maxAgeDays: number;
    readonly nextReviewDueAt: string;
  };
  readonly sources: readonly {
    readonly name: string;
    readonly package: string;
    readonly version: string;
  }[];
}

interface SpecManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourceCoverage: readonly SpecManifestSourceCoverageV0[];
  readonly entries: readonly SpecManifestEntryV0[];
}

interface SpecManifestSourceCoverageV0 {
  readonly sourceName: string;
  readonly entryIds: readonly string[];
  readonly sourceKeys: readonly string[];
}

interface SpecManifestEntryV0 {
  readonly id: string;
  readonly owner: string;
  readonly evidence?: readonly string[];
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
  readonly passIds?: readonly string[];
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
const specManifest = readJson<SpecManifestV0>(specManifestPath);
const selections = readJson<CompatFeatureSelectionsV0>(selectionPath);
validateInputs(specSources, specManifest, selections);

const browserThresholdsSource = renderBrowserThresholdsToml(specSources, selections);
const passFeatureBindingsSource = renderPassFeatureBindingsToml(specSources, selections);

if (checkOnly) {
  assert.equal(
    readText(browserThresholdsPath),
    browserThresholdsSource,
    `${browserThresholdsPath} is stale`,
  );
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
    specManifest: specManifestPath,
    selectionPath,
    generatedFiles: [browserThresholdsPath, passFeatureBindingsPath],
    featureCount: selections.features.length,
    thresholdCount: selections.features.reduce(
      (count, feature) => count + feature.thresholds.length,
      0,
    ),
    refreshedAt: specSources.refreshedAt,
    nextReviewDueAt: specSources.refreshPolicy.nextReviewDueAt,
  }),
);

function validateInputs(
  sourcePins: SpecSourcePinsV0,
  manifest: SpecManifestV0,
  featureSelections: CompatFeatureSelectionsV0,
): void {
  assert.equal(sourcePins.schemaVersion, "0");
  assert.equal(sourcePins.product, featureSelections.sourcePolicy.sourcePinProduct);
  assertIsoDate(sourcePins.refreshedAt, "sourcePins.refreshedAt");
  assert.equal(
    sourcePins.refreshPolicy.nextReviewDueAt,
    addIsoDateDays(sourcePins.refreshedAt, sourcePins.refreshPolicy.maxAgeDays),
    "source pin review due date must match refreshedAt + maxAgeDays",
  );
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-spec-audit.single-source-manifest");
  assert.equal(featureSelections.schemaVersion, "0");
  assert.equal(featureSelections.product, "omena-transform-target.compat-feature-selections");
  assert.equal(featureSelections.sourcePolicy.refreshedAtSource, sourcePinsPath);
  assert.equal(featureSelections.sourcePolicy.caniuseResolver.workspaceDependency, "browserslist");
  assert.equal(featureSelections.sourcePolicy.caniuseResolver.cargoPackage, "oxc-browserslist");
  assert.deepEqual(featureSelections.sourcePolicy.requiredSourceQuorum, [
    "caniuse",
    "web-features",
    "mdn-bcd",
  ]);

  const sourceNames = new Set(sourcePins.sources.map((source) => source.name));
  assert.ok(sourceNames.has("web-features"), "web-features source pin is required");
  assert.ok(sourceNames.has("mdn-browser-compat-data"), "MDN BCD source pin is required");
  const manifestSourceKeys = specManifestSourceKeyIndex(manifest);
  const manifestEvidence = specManifestEvidenceIndex(manifest);
  assert.ok(
    manifestSourceKeys.has("web-features"),
    "spec manifest source coverage must include web-features",
  );
  assert.ok(
    manifestSourceKeys.has("mdn-browser-compat-data"),
    "spec manifest source coverage must include MDN browser compatibility data",
  );
  assert.match(
    readText("rust/Cargo.toml"),
    /browserslist\s*=\s*\{\s*package\s*=\s*"oxc-browserslist"\s*,\s*version\s*=\s*"[^"]+"\s*\}/,
    "caniuse resolution must remain pinned through oxc-browserslist",
  );

  assert.ok(featureSelections.features.length > 0, "at least one compat feature is required");
  const tables = new Set<string>();
  for (const feature of featureSelections.features) {
    assert.match(feature.table, /^[a-z][a-z0-9_]*$/u, `invalid table ${feature.table}`);
    assert.ok(!tables.has(feature.table), `duplicate compat table ${feature.table}`);
    tables.add(feature.table);
    assert.ok(feature.passId.length > 0, `${feature.table} passId is required`);
    assert.deepEqual(
      selectionPassIds(feature)[0],
      feature.passId,
      `${feature.table} passIds must keep passId as the primary binding`,
    );
    assert.ok(feature.caniuseKeys.length > 0, `${feature.table} caniuseKeys required`);
    assert.deepEqual(
      Object.keys(feature.sourceKeys).toSorted(),
      [...featureSelections.sourcePolicy.requiredSourceQuorum].toSorted(),
      `${feature.table} must carry a cross-source feature key map`,
    );
    assert.equal(
      feature.sourceKeys.caniuse,
      feature.caniuseKeys[0],
      `${feature.table} primary caniuse key must match sourceKeys.caniuse`,
    );
    assertFeatureSourceKeyAnchored(manifestSourceKeys, feature, "web-features");
    assertFeatureSourceKeyAnchored(manifestSourceKeys, feature, "mdn-bcd");
    assertFeatureSourceKeyEvidenceAnchored(manifestEvidence, feature);
    for (const source of featureSelections.sourcePolicy.requiredSourceQuorum) {
      assert.ok(
        feature.sourceKeys[source]?.length > 0,
        `${feature.table} missing source key for ${source}`,
      );
    }
    assert.deepEqual(
      feature.sourceQuorum,
      featureSelections.sourcePolicy.requiredSourceQuorum,
      `${feature.table} must retain full source quorum`,
    );
    let previousBrowserOrder = -1;
    for (const threshold of feature.thresholds) {
      const browserOrder = expectedBrowserOrder().indexOf(threshold.browser);
      assert.notEqual(
        browserOrder,
        -1,
        `${feature.table}/${threshold.browser} must use a known browserslist browser id`,
      );
      assert.ok(
        browserOrder > previousBrowserOrder,
        `${feature.table} must retain stable browser row order without duplicates`,
      );
      previousBrowserOrder = browserOrder;
      assert.ok(
        Number.isInteger(threshold.minMajor),
        `${feature.table}/${threshold.browser} major`,
      );
      assert.ok(
        Number.isInteger(threshold.minMinor),
        `${feature.table}/${threshold.browser} minor`,
      );
      assert.ok(threshold.minMajor >= 0, `${feature.table}/${threshold.browser} major`);
      assert.ok(threshold.minMinor >= 0, `${feature.table}/${threshold.browser} minor`);
    }
  }
}

function specManifestSourceKeyIndex(manifest: SpecManifestV0): Map<string, Set<string>> {
  const sourceKeysByName = new Map<string, Set<string>>();
  for (const coverage of manifest.sourceCoverage) {
    assert.ok(coverage.sourceName.length > 0, "spec manifest source coverage name required");
    assert.ok(
      coverage.entryIds.length > 0,
      `spec manifest ${coverage.sourceName} entries required`,
    );
    assert.ok(
      coverage.sourceKeys.length > 0,
      `spec manifest ${coverage.sourceName} source keys required`,
    );
    sourceKeysByName.set(coverage.sourceName, new Set(coverage.sourceKeys));
  }
  return sourceKeysByName;
}

function specManifestEvidenceIndex(manifest: SpecManifestV0): Set<string> {
  const evidence = new Set<string>();
  for (const entry of manifest.entries) {
    assert.ok(entry.id.length > 0, "spec manifest entry id required");
    assert.ok(entry.owner.length > 0, `spec manifest ${entry.id} owner required`);
    for (const item of entry.evidence ?? []) {
      assert.ok(item.trim().length > 0, `spec manifest ${entry.id} evidence item required`);
      evidence.add(item);
    }
  }
  return evidence;
}

function assertFeatureSourceKeyAnchored(
  manifestSourceKeys: Map<string, Set<string>>,
  feature: CompatFeatureSelectionV0,
  source: "web-features" | "mdn-bcd",
): void {
  const manifestSourceName = source === "mdn-bcd" ? "mdn-browser-compat-data" : source;
  const sourceKey = feature.sourceKeys[source];
  assert.ok(
    manifestSourceKeys.get(manifestSourceName)?.has(sourceKey),
    `${feature.table} ${source} key ${sourceKey} must be anchored in spec manifest source coverage`,
  );
}

function assertFeatureSourceKeyEvidenceAnchored(
  manifestEvidence: Set<string>,
  feature: CompatFeatureSelectionV0,
): void {
  for (const [source, key] of Object.entries(feature.sourceKeys)) {
    assert.ok(
      manifestEvidence.has(`compat-source-key:${source}/${key}`),
      `${feature.table} ${source} key ${key} must be anchored by manifest evidence`,
    );
  }
}

function assertIsoDate(value: string, label: string): void {
  assert.match(value, /^\d{4}-\d{2}-\d{2}$/u, `${label} must be an ISO date`);
}

function addIsoDateDays(value: string, days: number): string {
  assert.ok(Number.isInteger(days) && days > 0, "maxAgeDays must be a positive integer");
  const timestamp = Date.parse(`${value}T00:00:00.000Z`);
  assert.ok(Number.isFinite(timestamp), `invalid ISO date ${value}`);
  const date = new Date(timestamp + days * 24 * 60 * 60 * 1000);
  return date.toISOString().slice(0, 10);
}

function renderBrowserThresholdsToml(
  sourcePins: SpecSourcePinsV0,
  featureSelections: CompatFeatureSelectionsV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.browser-thresholds"',
    `refreshed_at = ${quoteToml(sourcePins.refreshedAt)}`,
    `quorum_min_sources = ${featureSelections.sourcePolicy.requiredSourceQuorum.length}`,
    "",
  ];

  for (const feature of featureSelections.features) {
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
        `last_verified = ${quoteToml(sourcePins.refreshedAt)}`,
        "",
      );
    }
  }

  return `${lines.join("\n").trimEnd()}\n`;
}

function renderPassFeatureBindingsToml(
  sourcePins: SpecSourcePinsV0,
  featureSelections: CompatFeatureSelectionsV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.pass-feature-bindings"',
    `refreshed_at = ${quoteToml(sourcePins.refreshedAt)}`,
    "",
  ];

  for (const feature of featureSelections.features) {
    for (const passId of selectionPassIds(feature)) {
      lines.push(
        "[[binding]]",
        `pass_id = ${quoteToml(passId)}`,
        `caniuse_keys = ${tomlStringArray(feature.caniuseKeys)}`,
        `support_table = ${quoteToml(feature.table)}`,
        "",
      );
    }
  }

  return `${lines.join("\n").trimEnd()}\n`;
}

function selectionPassIds(feature: CompatFeatureSelectionV0): readonly string[] {
  const passIds = feature.passIds ?? [feature.passId];
  assert.ok(passIds.length > 0, `${feature.table} passIds is required`);
  const seen = new Set<string>();
  for (const passId of passIds) {
    assert.equal(typeof passId, "string", `${feature.table} pass id must be a string`);
    assert.ok(passId.length > 0, `${feature.table} pass id is required`);
    assert.ok(!seen.has(passId), `${feature.table} duplicate pass id ${passId}`);
    seen.add(passId);
  }
  return passIds;
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
