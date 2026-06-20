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
const webFeaturesDataPath = "node_modules/web-features/data.json";
const mdnBrowserCompatDataPath = "node_modules/@mdn/browser-compat-data/data.json";

interface SpecSourcePinsV0 {
  readonly refreshedAt: string;
  readonly refreshPolicy: {
    readonly maxAgeDays: number;
    readonly nextReviewDueAt: string;
  };
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

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
}

interface CompatFeatureSelectionsV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePolicy: {
    readonly caniuseResolver: {
      readonly workspaceDependency: string;
      readonly cargoPackage: string;
    };
    readonly requiredSourceQuorum: readonly string[];
  };
  readonly features: readonly CompatFeatureSelectionV0[];
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

type TomlValue = string | number | string[];
type TomlRecord = Record<string, TomlValue>;

interface ParsedTomlTablesV0 {
  readonly root: TomlRecord;
  readonly tables: readonly TomlRecord[];
}

type SourceJsonRecord = Record<string, unknown>;

const specSources = readJson<SpecSourcePinsV0>(
  "rust/crates/omena-spec-audit/data/spec-sources.json",
);
const specManifest = readJson<SpecManifestV0>(
  "rust/crates/omena-spec-audit/data/omena-spec-manifest.json",
);
const compatSelections = readJson<CompatFeatureSelectionsV0>(
  "rust/crates/omena-transform-target/data/compat-feature-selections.json",
);
const packageJson = readJson<PackageJsonV0>("package.json");
const webFeaturesData = readJson<SourceJsonRecord>(webFeaturesDataPath);
const mdnBrowserCompatData = readJson<SourceJsonRecord>(mdnBrowserCompatDataPath);
const browserThresholdData = parseTomlWithRepeatedTable(
  readText("rust/crates/omena-transform-target/data/browser-thresholds.toml"),
  "threshold",
);
const passFeatureBindingData = parseTomlWithRepeatedTable(
  readText("rust/crates/omena-transform-target/data/pass-feature-bindings.toml"),
  "binding",
);
const cargoToml = readText("rust/Cargo.toml");
const caniuseResolverCargoVersion = pinnedCargoPackageVersion(
  cargoToml,
  compatSelections.sourcePolicy.caniuseResolver.workspaceDependency,
  compatSelections.sourcePolicy.caniuseResolver.cargoPackage,
);
const compatReviewDate = compatDataReviewDate();

assert.equal(browserThresholdData.root.schema_version, "0");
assert.equal(compatSelections.schemaVersion, "0");
assert.equal(compatSelections.product, "omena-transform-target.compat-feature-selections");
assert.equal(compatSelections.sourcePolicy.caniuseResolver.workspaceDependency, "browserslist");
assert.equal(compatSelections.sourcePolicy.caniuseResolver.cargoPackage, "oxc-browserslist");
assert.deepEqual(compatSelections.sourcePolicy.requiredSourceQuorum, expectedQuorumSources);
assert.equal(browserThresholdData.root.product, "omena-transform-target.browser-thresholds");
assert.equal(passFeatureBindingData.root.schema_version, "0");
assert.equal(passFeatureBindingData.root.product, "omena-transform-target.pass-feature-bindings");
assert.equal(browserThresholdData.root.refreshed_at, specSources.refreshedAt);
assert.equal(passFeatureBindingData.root.refreshed_at, specSources.refreshedAt);
assertGeneratedResolverProvenance(browserThresholdData.root);
assertGeneratedResolverProvenance(passFeatureBindingData.root);
assert.match(specSources.refreshedAt, /^\d{4}-\d{2}-\d{2}$/u);
assert.match(specSources.refreshPolicy.nextReviewDueAt, /^\d{4}-\d{2}-\d{2}$/u);
assert.equal(
  specSources.refreshPolicy.nextReviewDueAt,
  addIsoDateDays(specSources.refreshedAt, specSources.refreshPolicy.maxAgeDays),
  "source pin review due date must match refreshedAt + maxAgeDays",
);
assertSourcePinsNotPastReviewDue(specSources, compatReviewDate);
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
assertSourcePinsDeclaredAsExactDevDependencies(specSources, packageJson);
assertFeatureSourceKeysPresentInPackages(
  compatSelections,
  webFeaturesData,
  mdnBrowserCompatData,
);
assert.equal(specManifest.schemaVersion, "0");
assert.equal(specManifest.product, "omena-spec-audit.single-source-manifest");
const manifestSourceKeys = specManifestSourceKeyIndex(specManifest);
const manifestEvidence = specManifestEvidenceIndex(specManifest);
assert.ok(
  manifestSourceKeys.has("web-features"),
  "spec manifest source coverage must include web-features",
);
assert.ok(
  manifestSourceKeys.has("mdn-browser-compat-data"),
  "spec manifest source coverage must include MDN browser compatibility data",
);
assert.match(
  cargoToml,
  /browserslist\s*=\s*\{\s*package\s*=\s*"oxc-browserslist"\s*,\s*version\s*=\s*"[^"]+"\s*\}/,
  "compat resolver source must pin oxc-browserslist in the Rust workspace",
);

const selectionsByTable = new Map<string, CompatFeatureSelectionV0>();
for (const feature of compatSelections.features) {
  assertString(feature.table, "selection.table");
  assertString(feature.passId, "selection.passId");
  assert.deepEqual(
    selectionPassIds(feature)[0],
    feature.passId,
    `selection ${feature.table} passIds must keep passId as the primary binding`,
  );
  assertStringArray(feature.caniuseKeys as string[], "selection.caniuseKeys");
  assert.deepEqual(
    feature.sourceQuorum,
    expectedQuorumSources,
    `selection ${feature.table} must require every generated compat source`,
  );
  assert.deepEqual(
    Object.keys(feature.sourceKeys).toSorted(),
    [...expectedQuorumSources].toSorted(),
    `selection ${feature.table} must map every generated compat source`,
  );
  for (const source of expectedQuorumSources) {
    assert.equal(
      typeof feature.sourceKeys[source],
      "string",
      `selection ${feature.table} source key ${source}`,
    );
    assert.ok(
      feature.sourceKeys[source].length > 0,
      `selection ${feature.table} source key ${source}`,
    );
  }
  assert.equal(
    feature.sourceKeys.caniuse,
    feature.caniuseKeys[0],
    `selection ${feature.table} caniuse mapping must match the pass binding key`,
  );
  assert.ok(feature.thresholds.length > 0, `selection ${feature.table} threshold rows required`);
  assertSelectionThresholdRows(feature);
  assertFeatureSourceKeyAnchored(manifestSourceKeys, feature, "web-features");
  assertFeatureSourceKeyAnchored(manifestSourceKeys, feature, "mdn-bcd");
  assertFeatureSourceKeyEvidenceAnchored(manifestEvidence, feature);
  assert.ok(!selectionsByTable.has(feature.table), `duplicate selection table ${feature.table}`);
  selectionsByTable.set(feature.table, feature);
}

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
  const selection = selectionsByTable.get(table);
  assert.ok(selection, `threshold table ${table} has no curated source-key selection`);
  const browsers = thresholds.map((threshold) => threshold.browser);
  let previousBrowserOrder = -1;
  for (const browser of browsers) {
    const browserOrder = expectedBrowsers.indexOf(browser as (typeof expectedBrowsers)[number]);
    assert.notEqual(
      browserOrder,
      -1,
      `feature table ${table} contains unknown browser row ${browser}`,
    );
    assert.ok(
      browserOrder > previousBrowserOrder,
      `feature table ${table} must retain stable browser row order without duplicates`,
    );
    previousBrowserOrder = browserOrder;
  }
  assert.equal(
    new Set(thresholds.map((threshold) => threshold.caniuse_key)).size,
    1,
    `feature table ${table} must map to a single caniuse feature key`,
  );
  assert.equal(
    thresholds[0]?.caniuse_key,
    selection.caniuseKeys[0],
    `feature table ${table} generated rows must use the curated caniuse key`,
  );
  assertThresholdRowsMatchSelection(table, thresholds, selection);
}

const mappedTables = new Set<string>();
const mappedPassIdsByTable = new Map<string, Set<string>>();
for (const binding of passFeatureBindingData.tables) {
  assertString(binding.pass_id, "binding.pass_id");
  assertString(binding.support_table, "binding.support_table");
  assertStringArray(binding.caniuse_keys, "binding.caniuse_keys");
  mappedTables.add(binding.support_table);
  const mappedPassIds = mappedPassIdsByTable.get(binding.support_table) ?? new Set<string>();
  assert.ok(
    !mappedPassIds.has(binding.pass_id),
    `binding ${binding.pass_id} must not be duplicated for ${binding.support_table}`,
  );
  mappedPassIds.add(binding.pass_id);
  mappedPassIdsByTable.set(binding.support_table, mappedPassIds);

  const thresholds = thresholdsByTable.get(binding.support_table);
  assert.ok(thresholds, `binding ${binding.pass_id} maps unknown table ${binding.support_table}`);
  const selection = selectionsByTable.get(binding.support_table);
  assert.ok(selection, `binding ${binding.pass_id} maps table without source-key selection`);
  assert.equal(
    selectionPassIds(selection).includes(binding.pass_id),
    true,
    `binding ${binding.pass_id} must match one curated selection pass id`,
  );
  assert.deepEqual(
    binding.caniuse_keys,
    selection.caniuseKeys,
    `binding ${binding.pass_id} must keep the curated caniuse key set`,
  );
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
for (const [table, selection] of selectionsByTable) {
  assert.deepEqual(
    [...(mappedPassIdsByTable.get(table) ?? new Set<string>())].toSorted(),
    [...selectionPassIds(selection)].toSorted(),
    `selection ${table} must emit every curated pass binding`,
  );
}

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

function assertGeneratedResolverProvenance(root: TomlRecord): void {
  assert.equal(
    root.caniuse_resolver_workspace_dependency,
    compatSelections.sourcePolicy.caniuseResolver.workspaceDependency,
    "generated compat root must stamp the caniuse resolver workspace dependency",
  );
  assert.equal(
    root.caniuse_resolver_cargo_package,
    compatSelections.sourcePolicy.caniuseResolver.cargoPackage,
    "generated compat root must stamp the caniuse resolver cargo package",
  );
  assert.equal(
    root.caniuse_resolver_cargo_version,
    caniuseResolverCargoVersion,
    "generated compat root must stamp the caniuse resolver cargo version",
  );
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

function pinnedCargoPackageVersion(
  source: string,
  workspaceDependency: string,
  cargoPackage: string,
): string {
  const escapedDependency = escapeRegExp(workspaceDependency);
  const escapedPackage = escapeRegExp(cargoPackage);
  const match = source.match(
    new RegExp(
      `${escapedDependency}\\s*=\\s*\\{\\s*package\\s*=\\s*"${escapedPackage}"\\s*,\\s*version\\s*=\\s*"([^"]+)"\\s*\\}`,
      "u",
    ),
  );
  assert.ok(match?.[1], `${workspaceDependency} must pin ${cargoPackage} with an explicit version`);
  return match[1];
}

function pushMapValue<K, V>(map: Map<K, V[]>, key: K, value: V): void {
  const existing = map.get(key);
  if (existing) {
    existing.push(value);
    return;
  }
  map.set(key, [value]);
}

function specManifestSourceKeyIndex(manifest: SpecManifestV0): Map<string, Set<string>> {
  const sourceKeysByName = new Map<string, Set<string>>();
  for (const coverage of manifest.sourceCoverage) {
    assert.ok(coverage.sourceName.length > 0, "spec manifest source coverage name required");
    assert.ok(coverage.entryIds.length > 0, `spec manifest ${coverage.sourceName} entries required`);
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
  sourceKeysByManifestName: Map<string, Set<string>>,
  feature: CompatFeatureSelectionV0,
  source: "web-features" | "mdn-bcd",
): void {
  const manifestSourceName = source === "mdn-bcd" ? "mdn-browser-compat-data" : source;
  const sourceKey = feature.sourceKeys[source];
  assert.ok(
    sourceKeysByManifestName.get(manifestSourceName)?.has(sourceKey),
    `selection ${feature.table} ${source} key ${sourceKey} must be anchored in spec manifest source coverage`,
  );
}

function assertFeatureSourceKeyEvidenceAnchored(
  manifestEvidenceItems: Set<string>,
  feature: CompatFeatureSelectionV0,
): void {
  for (const [source, key] of Object.entries(feature.sourceKeys)) {
    assert.ok(
      manifestEvidenceItems.has(`compat-source-key:${source}/${key}`),
      `selection ${feature.table} ${source} key ${key} must be anchored by manifest evidence`,
    );
  }
}

function assertSelectionThresholdRows(feature: CompatFeatureSelectionV0): void {
  let previousBrowserOrder = -1;
  for (const threshold of feature.thresholds) {
    assert.equal(typeof threshold.browser, "string", `selection ${feature.table} threshold browser`);
    assert.equal(typeof threshold.minMajor, "number", `selection ${feature.table} minMajor`);
    assert.equal(typeof threshold.minMinor, "number", `selection ${feature.table} minMinor`);
    assert.ok(
      Number.isInteger(threshold.minMajor) && threshold.minMajor >= 0,
      `selection ${feature.table}/${threshold.browser} minMajor must be a non-negative integer`,
    );
    assert.ok(
      Number.isInteger(threshold.minMinor) && threshold.minMinor >= 0,
      `selection ${feature.table}/${threshold.browser} minMinor must be a non-negative integer`,
    );
    const browserOrder = expectedBrowsers.indexOf(
      threshold.browser as (typeof expectedBrowsers)[number],
    );
    assert.notEqual(
      browserOrder,
      -1,
      `selection ${feature.table} contains unknown browser row ${threshold.browser}`,
    );
    assert.ok(
      browserOrder > previousBrowserOrder,
      `selection ${feature.table} thresholds must retain stable browser row order without duplicates`,
    );
    previousBrowserOrder = browserOrder;
  }
}

function assertThresholdRowsMatchSelection(
  table: string,
  generatedThresholds: readonly TomlRecord[],
  selection: CompatFeatureSelectionV0,
): void {
  const generatedRows = generatedThresholds.map((threshold) => {
    assertString(threshold.browser, `threshold ${table}.browser`);
    assertNumber(threshold.min_major, `threshold ${table}.min_major`);
    assertNumber(threshold.min_minor, `threshold ${table}.min_minor`);
    assertString(threshold.caniuse_key, `threshold ${table}.caniuse_key`);
    return {
      browser: threshold.browser,
      minMajor: threshold.min_major,
      minMinor: threshold.min_minor,
      caniuseKey: threshold.caniuse_key,
    };
  });
  const expectedRows = selection.thresholds.map((threshold) => ({
    browser: threshold.browser,
    minMajor: threshold.minMajor,
    minMinor: threshold.minMinor,
    caniuseKey: selection.caniuseKeys[0],
  }));
  assert.deepEqual(
    generatedRows,
    expectedRows,
    `generated threshold table ${table} must exactly match the curated source mapping`,
  );
}

function addIsoDateDays(value: string, days: number): string {
  assert.ok(Number.isInteger(days) && days > 0, "maxAgeDays must be a positive integer");
  const timestamp = Date.parse(`${value}T00:00:00.000Z`);
  assert.ok(Number.isFinite(timestamp), `invalid ISO date ${value}`);
  const date = new Date(timestamp + days * 24 * 60 * 60 * 1000);
  return date.toISOString().slice(0, 10);
}

function compatDataReviewDate(): string {
  const reviewDate =
    process.env.OMENA_COMPAT_REVIEW_DATE ?? new Date().toISOString().slice(0, 10);
  assert.match(
    reviewDate,
    /^\d{4}-\d{2}-\d{2}$/u,
    "OMENA_COMPAT_REVIEW_DATE must be an ISO calendar date",
  );
  return reviewDate;
}

function assertSourcePinsNotPastReviewDue(sourcePins: SpecSourcePinsV0, reviewDate: string): void {
  assertIsoDateOrder(
    sourcePins.refreshedAt,
    sourcePins.refreshPolicy.nextReviewDueAt,
    "source pin review due date must not precede refreshedAt",
  );
  assertIsoDateOrder(
    reviewDate,
    sourcePins.refreshPolicy.nextReviewDueAt,
    `generated compat source pins are past review due date ${
      sourcePins.refreshPolicy.nextReviewDueAt
    }; refresh source pins and regenerate compat data`,
  );
}

function assertIsoDateOrder(left: string, right: string, message: string): void {
  assert.match(left, /^\d{4}-\d{2}-\d{2}$/u);
  assert.match(right, /^\d{4}-\d{2}-\d{2}$/u);
  assert.ok(left <= right, message);
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

function selectionPassIds(feature: CompatFeatureSelectionV0): readonly string[] {
  const passIds = feature.passIds ?? [feature.passId];
  assert.ok(passIds.length > 0, `selection ${feature.table} passIds is required`);
  const seen = new Set<string>();
  for (const passId of passIds) {
    assert.equal(typeof passId, "string", `selection ${feature.table} pass id`);
    assert.ok(passId.length > 0, `selection ${feature.table} pass id is required`);
    assert.ok(!seen.has(passId), `selection ${feature.table} duplicate pass id ${passId}`);
    seen.add(passId);
  }
  return passIds;
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

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
