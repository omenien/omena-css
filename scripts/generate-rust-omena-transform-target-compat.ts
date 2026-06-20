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
const packageJsonPath = "package.json";
const webFeaturesDataPath = "node_modules/web-features/data.json";
const mdnBrowserCompatDataPath = "node_modules/@mdn/browser-compat-data/data.json";
const browserThresholdsPath = "rust/crates/omena-transform-target/data/browser-thresholds.toml";
const passFeatureBindingsPath =
  "rust/crates/omena-transform-target/data/pass-feature-bindings.toml";
const generatorPath = "scripts/generate-rust-omena-transform-target-compat.ts";
const rustWorkspaceManifestPath = "rust/Cargo.toml";
const thresholdSourcePolicy = "mdnFullUnprefixedLowerBound";

interface CaniuseResolverProvenanceV0 {
  readonly workspaceDependency: string;
  readonly cargoPackage: string;
  readonly cargoVersion: string;
}

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

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
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

type SourceJsonRecord = Record<string, unknown>;

const specSources = readJson<SpecSourcePinsV0>(sourcePinsPath);
const specManifest = readJson<SpecManifestV0>(specManifestPath);
const selections = readJson<CompatFeatureSelectionsV0>(selectionPath);
const packageJson = readJson<PackageJsonV0>(packageJsonPath);
const webFeaturesData = readJson<SourceJsonRecord>(webFeaturesDataPath);
const mdnBrowserCompatData = readJson<SourceJsonRecord>(mdnBrowserCompatDataPath);
validateInputs(
  specSources,
  specManifest,
  selections,
  packageJson,
  webFeaturesData,
  mdnBrowserCompatData,
);
const resolverProvenance = caniuseResolverProvenance(selections);

const browserThresholdsSource = renderBrowserThresholdsToml(
  specSources,
  selections,
  resolverProvenance,
);
const passFeatureBindingsSource = renderPassFeatureBindingsToml(
  specSources,
  selections,
  resolverProvenance,
);

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
    caniuseResolver: resolverProvenance,
    refreshedAt: specSources.refreshedAt,
    nextReviewDueAt: specSources.refreshPolicy.nextReviewDueAt,
  }),
);

function validateInputs(
  sourcePins: SpecSourcePinsV0,
  manifest: SpecManifestV0,
  featureSelections: CompatFeatureSelectionsV0,
  rootPackageJson: PackageJsonV0,
  webFeaturesSourceData: SourceJsonRecord,
  mdnCompatSourceData: SourceJsonRecord,
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
  assertSourcePinsDeclaredAsExactDevDependencies(sourcePins, rootPackageJson);
  assertFeatureSourceKeysPresentInPackages(
    featureSelections,
    webFeaturesSourceData,
    mdnCompatSourceData,
  );
  assertFeatureThresholdsNotOlderThanMdnFullSupport(featureSelections, mdnCompatSourceData);
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
    readText(rustWorkspaceManifestPath),
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

function caniuseResolverProvenance(
  featureSelections: CompatFeatureSelectionsV0,
): CaniuseResolverProvenanceV0 {
  const { workspaceDependency, cargoPackage } = featureSelections.sourcePolicy.caniuseResolver;
  const cargoVersion = pinnedCargoPackageVersion(
    readText(rustWorkspaceManifestPath),
    workspaceDependency,
    cargoPackage,
  );
  return {
    workspaceDependency,
    cargoPackage,
    cargoVersion,
  };
}

function pinnedCargoPackageVersion(
  cargoToml: string,
  workspaceDependency: string,
  cargoPackage: string,
): string {
  const escapedDependency = escapeRegExp(workspaceDependency);
  const escapedPackage = escapeRegExp(cargoPackage);
  const match = cargoToml.match(
    new RegExp(
      `${escapedDependency}\\s*=\\s*\\{\\s*package\\s*=\\s*"${escapedPackage}"\\s*,\\s*version\\s*=\\s*"([^"]+)"\\s*\\}`,
      "u",
    ),
  );
  assert.ok(match?.[1], `${workspaceDependency} must pin ${cargoPackage} with an explicit version`);
  return match[1];
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

function assertSourcePinsDeclaredAsExactDevDependencies(
  sourcePins: SpecSourcePinsV0,
  rootPackageJson: PackageJsonV0,
): void {
  for (const source of sourcePins.sources) {
    assert.equal(
      rootPackageJson.devDependencies?.[source.package],
      source.version,
      `${source.package} must be pinned in root devDependencies at spec source version ${source.version}`,
    );
  }
}

function assertFeatureSourceKeysPresentInPackages(
  featureSelections: CompatFeatureSelectionsV0,
  webFeaturesSourceData: SourceJsonRecord,
  mdnCompatSourceData: SourceJsonRecord,
): void {
  const webFeatureRecords = objectProperty(webFeaturesSourceData, "features", "web-features");
  for (const feature of featureSelections.features) {
    const webFeatureKey = feature.sourceKeys["web-features"];
    const mdnCompatKey = feature.sourceKeys["mdn-bcd"];
    const caniuseKey = feature.sourceKeys.caniuse;
    const webFeature = objectProperty(
      webFeatureRecords,
      webFeatureKey,
      `web-features feature ${webFeatureKey}`,
    );
    const compatFeatures = stringArrayProperty(
      webFeature,
      "compat_features",
      `web-features feature ${webFeatureKey}.compat_features`,
    );
    assert.ok(
      compatFeatures.includes(mdnCompatKey),
      `${feature.table} web-features key ${webFeatureKey} must include MDN compat key ${mdnCompatKey}`,
    );
    const webFeatureCaniuseKeys = optionalStringArrayProperty(
      webFeature,
      "caniuse",
      `web-features feature ${webFeatureKey}.caniuse`,
    );
    if (webFeatureCaniuseKeys) {
      assert.ok(
        webFeatureCaniuseKeys.includes(caniuseKey),
        `${feature.table} web-features key ${webFeatureKey} must include caniuse key ${caniuseKey}`,
      );
    }
    const mdnCompat = dottedObjectProperty(mdnCompatSourceData, mdnCompatKey, "MDN BCD");
    objectProperty(mdnCompat, "__compat", `MDN BCD ${mdnCompatKey}`);
  }
}

function assertFeatureThresholdsNotOlderThanMdnFullSupport(
  featureSelections: CompatFeatureSelectionsV0,
  mdnCompatSourceData: SourceJsonRecord,
): void {
  for (const feature of featureSelections.features) {
    const mdnCompatKey = feature.sourceKeys["mdn-bcd"];
    const mdnCompat = dottedObjectProperty(mdnCompatSourceData, mdnCompatKey, "MDN BCD");
    const support = objectProperty(
      objectProperty(mdnCompat, "__compat", `MDN BCD ${mdnCompatKey}`),
      "support",
      `MDN BCD ${mdnCompatKey} support`,
    );
    for (const threshold of feature.thresholds) {
      const mdnBrowser = mdnBrowserForThresholdBrowser(threshold.browser);
      const mdnVersion = mdnFullUnprefixedSupportVersion(support, mdnBrowser);
      assert.ok(
        compareBrowserVersions([threshold.minMajor, threshold.minMinor], mdnVersion) >= 0,
        `${feature.table}/${threshold.browser} threshold ${threshold.minMajor}.${threshold.minMinor} must not be older than MDN full unprefixed support ${mdnVersion.join(".")}`,
      );
    }
  }
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
  provenance: CaniuseResolverProvenanceV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.browser-thresholds"',
    `refreshed_at = ${quoteToml(sourcePins.refreshedAt)}`,
    `threshold_source_policy = ${quoteToml(thresholdSourcePolicy)}`,
    `quorum_min_sources = ${featureSelections.sourcePolicy.requiredSourceQuorum.length}`,
    `caniuse_resolver_workspace_dependency = ${quoteToml(provenance.workspaceDependency)}`,
    `caniuse_resolver_cargo_package = ${quoteToml(provenance.cargoPackage)}`,
    `caniuse_resolver_cargo_version = ${quoteToml(provenance.cargoVersion)}`,
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
  provenance: CaniuseResolverProvenanceV0,
): string {
  const lines = [
    `# Generated by ${generatorPath}. Do not edit manually.`,
    `# Source selections: ${selectionPath}.`,
    'schema_version = "0"',
    'product = "omena-transform-target.pass-feature-bindings"',
    `refreshed_at = ${quoteToml(sourcePins.refreshedAt)}`,
    `caniuse_resolver_workspace_dependency = ${quoteToml(provenance.workspaceDependency)}`,
    `caniuse_resolver_cargo_package = ${quoteToml(provenance.cargoPackage)}`,
    `caniuse_resolver_cargo_version = ${quoteToml(provenance.cargoVersion)}`,
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

function dottedObjectProperty(
  source: SourceJsonRecord,
  dottedPath: string,
  label: string,
): SourceJsonRecord {
  return dottedPath.split(".").reduce<SourceJsonRecord>((current, segment, index) => {
    return objectProperty(current, segment, `${label} ${dottedPath} segment ${index + 1}`);
  }, source);
}

function objectProperty(
  source: SourceJsonRecord,
  key: string,
  label: string,
): SourceJsonRecord {
  const value = source[key];
  assert.ok(value && typeof value === "object" && !Array.isArray(value), `${label} required`);
  return value as SourceJsonRecord;
}

function stringArrayProperty(
  source: SourceJsonRecord,
  key: string,
  label: string,
): readonly string[] {
  const value = source[key];
  assert.ok(Array.isArray(value), `${label} must be a string array`);
  for (const item of value) assert.equal(typeof item, "string", `${label} item`);
  return value as string[];
}

function optionalStringArrayProperty(
  source: SourceJsonRecord,
  key: string,
  label: string,
): readonly string[] | undefined {
  if (source[key] === undefined) return undefined;
  return stringArrayProperty(source, key, label);
}

function mdnBrowserForThresholdBrowser(browser: string): string {
  const browserMap: Record<string, string> = {
    chrome: "chrome",
    edge: "edge",
    firefox: "firefox",
    safari: "safari",
    ios_saf: "safari_ios",
    opera: "opera",
    op_mob: "opera_android",
    and_chr: "chrome_android",
    and_ff: "firefox_android",
    samsung: "samsunginternet_android",
    android: "webview_android",
  };
  const mapped = browserMap[browser];
  assert.ok(mapped, `missing MDN browser mapping for ${browser}`);
  return mapped;
}

function mdnFullUnprefixedSupportVersion(
  support: SourceJsonRecord,
  browser: string,
): readonly [number, number] {
  const entries = supportEntryArray(support[browser]);
  const versions = entries
    .filter((entry) => {
      return (
        !entry.partial_implementation &&
        !entry.prefix &&
        !entry.alternative_name &&
        !entry.flags
      );
    })
    .map((entry) => parseBrowserVersion(entry.version_added))
    .filter((version): version is readonly [number, number] => version !== undefined)
    .sort(compareBrowserVersions);
  const version = versions[0];
  assert.ok(version, `MDN ${browser} full unprefixed support version required`);
  return version;
}

function supportEntryArray(value: unknown): SourceJsonRecord[] {
  if (value === undefined || value === false) return [];
  if (Array.isArray(value)) {
    return value.filter(
      (item): item is SourceJsonRecord =>
        !!item && typeof item === "object" && !Array.isArray(item),
    );
  }
  assert.ok(value && typeof value === "object", "MDN support entry must be an object");
  return [value as SourceJsonRecord];
}

function parseBrowserVersion(value: unknown): readonly [number, number] | undefined {
  if (typeof value !== "string") return undefined;
  const match = value.match(/^(\d+)(?:\.(\d+))?/u);
  if (!match) return undefined;
  return [Number(match[1]), Number(match[2] ?? 0)];
}

function compareBrowserVersions(left: readonly [number, number], right: readonly [number, number]) {
  return left[0] - right[0] || left[1] - right[1];
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

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function stableJson(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
