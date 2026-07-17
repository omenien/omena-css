import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

import { formatGeneratedJson } from "./generated-json";

export const COVERAGE_GAP_REPORT_PATH = "rust/crates/omena-spec-audit/data/omena-coverage-gap.json";
export const WEBREF_GRAMMAR_PATH = "rust/crates/omena-spec-audit/data/webref-grammar.json";
export const WEB_FEATURES_DATA_PATH = "node_modules/web-features/data.json";
export const COVERAGE_GAP_GENERATOR_PATH = "scripts/generate-rust-omena-coverage-gap.ts";

const VALUE_NAME_TABLES = [
  "VALUES_L4_MATH_FUNCTION_NAMES",
  "CSS_COLOR_FUNCTION_NAMES",
  "CSS_GRADIENT_FUNCTION_NAMES",
  "CSS_TRANSFORM_FUNCTION_NAMES",
  "CSS_FILTER_FUNCTION_NAMES",
  "CSS_IMAGE_FUNCTION_NAMES",
  "CSS_SHAPE_FUNCTION_NAMES",
] as const;

const FOLD_GAP_EXCLUDED_SPECIALIZED_ARMS = new Set(["var", "env", "attr"]);
export const COVERAGE_CAPABILITY_TIERS = ["T0", "T1", "T2", "T3", "T4"] as const;
export const COVERAGE_REASON_CODES = [
  "engine-recognition-not-observed",
  "engine-validation-not-observed",
  "upstream-syntax-absent",
  "forward-specification",
] as const;
const COVERAGE_GAP_CATEGORIES = [
  "atrules",
  "functions",
  "properties",
  "selectors",
  "types",
] as const;

const EXPLICIT_CSS_FOLD_SURFACES = [
  "rust/crates/omena-transform-passes/src/domains/calc.rs",
  "rust/crates/omena-transform-passes/src/domains/static_eval.rs",
  "rust/crates/omena-transform-passes/src/domains/unit_transform.rs",
  "rust/crates/omena-transform-passes/src/domains/unit_filter.rs",
  "rust/crates/omena-transform-passes/src/domains/color_lowering.rs",
  "rust/crates/omena-transform-passes/src/domains/color.rs",
] as const;

export type CoverageGapCategory = (typeof COVERAGE_GAP_CATEGORIES)[number];
export type CoverageCapabilityTier = (typeof COVERAGE_CAPABILITY_TIERS)[number];
export type CoverageReasonCode = (typeof COVERAGE_REASON_CODES)[number];
export type CoverageGapBaselineStatus = "high" | "low" | "limited" | "unknown";
export type CoverageBoundaryClassification = "in-boundary" | "forward-tier";

export interface CoverageGapBaselineRank {
  readonly status: CoverageGapBaselineStatus;
  readonly sortRank: number;
  readonly featureId: string | null;
  readonly featureName: string | null;
  readonly baselineHigh?: string;
  readonly baselineLow?: string;
}

export interface CoverageGapRow {
  readonly id: string;
  readonly category: CoverageGapCategory;
  readonly name: string;
  readonly href: string;
  readonly sourceOrdinal: number;
  readonly syntaxAvailable: boolean;
  readonly boundaryClassification: CoverageBoundaryClassification;
  readonly capabilityTier: CoverageCapabilityTier;
  readonly namedReason: CoverageReasonCode;
  readonly measurements: {
    readonly recognized: boolean;
    readonly staticallyReduced: boolean;
  };
  readonly baseline: CoverageGapBaselineRank;
}

export interface CoverageGapReport {
  readonly schemaVersion: "1";
  readonly product: "omena-spec-audit.coverage-gap";
  readonly generation: {
    readonly tool: typeof COVERAGE_GAP_GENERATOR_PATH;
    readonly mode: "registry-completeness-and-byte-identity";
  };
  readonly sources: {
    readonly webrefGrammar: typeof WEBREF_GRAMMAR_PATH;
    readonly webFeatures: typeof WEB_FEATURES_DATA_PATH;
  };
  readonly policy: {
    readonly advisory: true;
    readonly denominator: "CSS specs per CSS Snapshot 2025 boundary; Sass semantics covered on the oracle axis";
    readonly redCondition: "registry-row-or-tier-reason-drift";
    readonly capabilityTiers: readonly CoverageCapabilityTier[];
    readonly namedReasons: readonly CoverageReasonCode[];
    readonly staticReductionDoesNotImplyValidation: true;
  };
  readonly summary: {
    readonly categoryCounts: Readonly<Record<CoverageGapCategory, number>>;
    readonly tierCounts: Readonly<Record<CoverageCapabilityTier, number>>;
    readonly recognizedFunctionCount: number;
    readonly recognizedAtRuleCount: number;
    readonly cssFoldedFunctionCount: number;
    readonly lessFoldedFunctionCount: number;
    readonly rowCount: number;
  };
  readonly rows: readonly CoverageGapRow[];
}

export interface WebrefGrammarSnapshot {
  readonly schemaVersion: "1";
  readonly product: "omena-spec-audit.webref-grammar";
  readonly source: { readonly package: string; readonly version: string; readonly gitHead: string };
  readonly generation: { readonly tool: string };
  readonly entryCount: number;
  readonly categories: Record<
    string,
    readonly {
      readonly name: string;
      readonly href: string;
      readonly sourceOrdinal: number;
      readonly syntax: string | null;
      readonly boundary: {
        readonly classification: CoverageBoundaryClassification;
      };
    }[]
  >;
}

export interface CoverageGapEngineSources {
  readonly syntaxHelpers: string;
  readonly valueNames: string;
  readonly extension: string;
  readonly cssFoldSources: readonly string[];
  readonly nativeCss: string;
  readonly lessNumbers: string;
  readonly domainFoldSources: readonly string[];
}

export interface EngineRecognitionSurface {
  readonly functions: readonly string[];
  readonly atrules: readonly string[];
  readonly specializedArms: readonly string[];
  readonly valueNameTables: Readonly<Record<string, readonly string[]>>;
}

export interface EngineFoldSurface {
  readonly cssFunctions: readonly string[];
  readonly lessFunctions: readonly string[];
  readonly cssFunctionsFromExplicitSurfaces: readonly string[];
  readonly cssFunctionsFromDomainSweep: readonly string[];
}

export interface CoverageGapComputationInput {
  readonly grammar: WebrefGrammarSnapshot;
  readonly webFeaturesData: WebFeaturesData;
  readonly recognition: EngineRecognitionSurface;
  readonly fold: EngineFoldSurface;
}

export interface CoverageGapBuildOptions {
  readonly injectUntieredRow?: boolean;
  readonly injectFreeTextReason?: boolean;
}

interface WebFeaturesData {
  readonly features?: Record<string, WebFeatureEntry>;
}

interface WebFeatureEntry {
  readonly name?: string;
  readonly description?: string;
  readonly compat_features?: readonly string[];
  readonly status?: {
    readonly baseline?: "high" | "low" | false;
    readonly baseline_high_date?: string;
    readonly baseline_low_date?: string;
  };
}

interface IndexedWebFeature {
  readonly id: string;
  readonly entry: WebFeatureEntry;
}

interface WebFeatureIndex {
  readonly byId: ReadonlyMap<string, IndexedWebFeature>;
  readonly byCompatKey: ReadonlyMap<string, IndexedWebFeature>;
  readonly byNormalizedName: ReadonlyMap<string, IndexedWebFeature>;
}

export function buildCoverageGapReportFromRepo(
  repoRoot = process.cwd(),
  options: CoverageGapBuildOptions = {},
): CoverageGapReport {
  const sources = loadCoverageGapEngineSources(repoRoot);
  const grammar = readJson<WebrefGrammarSnapshot>(path.join(repoRoot, WEBREF_GRAMMAR_PATH));
  const webFeaturesData = readJson<WebFeaturesData>(path.join(repoRoot, WEB_FEATURES_DATA_PATH));
  const recognition = extractEngineRecognitionSurface(sources);
  const fold = extractEngineFoldSurface(sources);
  const report = injectCoverageLedgerFaults(
    buildCoverageGapReport({ grammar, webFeaturesData, recognition, fold }),
    options,
  );
  validateCoverageGapReport(report, grammar, recognition, fold);
  return report;
}

export function loadCoverageGapEngineSources(repoRoot: string): CoverageGapEngineSources {
  const domainDir = path.join(repoRoot, "rust/crates/omena-transform-passes/src/domains");
  return {
    syntaxHelpers: readText(path.join(repoRoot, "rust/crates/omena-parser/src/syntax_helpers.rs")),
    valueNames: readText(path.join(repoRoot, "rust/crates/omena-parser/src/value_names.rs")),
    extension: readText(path.join(repoRoot, "rust/crates/omena-parser/src/extension.rs")),
    cssFoldSources: EXPLICIT_CSS_FOLD_SURFACES.map((relativePath) =>
      readText(path.join(repoRoot, relativePath)),
    ),
    nativeCss: readText(path.join(repoRoot, "rust/crates/omena-scss-eval/src/native_css.rs")),
    lessNumbers: readText(
      path.join(repoRoot, "rust/crates/omena-scss-eval/src/static_stylesheet/less_numbers.rs"),
    ),
    domainFoldSources: readRustSourcesRecursively(domainDir),
  };
}

export function extractEngineRecognitionSurface(
  sources: Pick<CoverageGapEngineSources, "syntaxHelpers" | "valueNames" | "extension">,
): EngineRecognitionSurface {
  const valueNameTables = Object.fromEntries(
    VALUE_NAME_TABLES.map((tableName) => [
      tableName,
      extractRustStringArray(sources.valueNames, tableName),
    ]),
  );
  const specializedArms = extractSpecializedFunctionArms(sources.syntaxHelpers);
  const functions = sortedUnique(
    [...specializedArms, ...Object.values(valueNameTables).flat()].map(normalizeFunctionName),
  );
  const atrules = sortedUnique(extractAtRuleRecognitionNames(sources.extension));
  return { functions, atrules, specializedArms: sortedUnique(specializedArms), valueNameTables };
}

export function extractEngineFoldSurface(
  sources: Pick<
    CoverageGapEngineSources,
    "cssFoldSources" | "nativeCss" | "lessNumbers" | "domainFoldSources"
  >,
): EngineFoldSurface {
  const explicitCssFoldNames = sortedUnique(
    [
      ...sources.cssFoldSources.flatMap(extractStaticCssFunctionSpecNames),
      ...extractNativeCssFunctionFoldNames(sources.nativeCss),
    ].map(normalizeFunctionName),
  );
  const domainSweepNames = sortedUnique(
    sources.domainFoldSources.flatMap(extractStaticCssFunctionSpecNames).map(normalizeFunctionName),
  );
  const lessFunctions = sortedUnique(extractLessTrigFoldNames(sources.lessNumbers));
  return {
    cssFunctions: explicitCssFoldNames,
    lessFunctions,
    cssFunctionsFromExplicitSurfaces: explicitCssFoldNames,
    cssFunctionsFromDomainSweep: domainSweepNames,
  };
}

export function buildCoverageGapReport(input: CoverageGapComputationInput): CoverageGapReport {
  const recognizedFunctions = new Set(input.recognition.functions.map(normalizeFunctionName));
  const recognizedAtRules = new Set(input.recognition.atrules.map(normalizeAtRuleName));
  const cssFoldedFunctions = new Set(input.fold.cssFunctions.map(normalizeFunctionName));
  const webFeatureIndex = buildWebFeatureIndex(input.webFeaturesData);
  const rows = COVERAGE_GAP_CATEGORIES.flatMap((category) =>
    (input.grammar.categories[category] ?? []).map((entry) => {
      const normalizedName = normalizeCoverageName(category, entry.name);
      const recognized =
        category === "functions"
          ? recognizedFunctions.has(normalizedName)
          : category === "atrules"
            ? recognizedAtRules.has(normalizedName)
            : false;
      const staticallyReduced =
        category === "functions" &&
        !FOLD_GAP_EXCLUDED_SPECIALIZED_ARMS.has(normalizedName) &&
        cssFoldedFunctions.has(normalizedName);
      const capabilityTier: CoverageCapabilityTier = recognized ? "T1" : "T0";
      return buildCoverageGapRow(
        category,
        entry,
        capabilityTier,
        recognized,
        staticallyReduced,
        webFeatureIndex,
      );
    }),
  ).toSorted(compareCoverageGapRows);

  const categoryCounts = Object.fromEntries(
    COVERAGE_GAP_CATEGORIES.map((category) => [
      category,
      rows.filter((row) => row.category === category).length,
    ]),
  ) as Record<CoverageGapCategory, number>;
  const tierCounts = Object.fromEntries(
    COVERAGE_CAPABILITY_TIERS.map((tier) => [
      tier,
      rows.filter((row) => row.capabilityTier === tier).length,
    ]),
  ) as Record<CoverageCapabilityTier, number>;

  return {
    schemaVersion: "1",
    product: "omena-spec-audit.coverage-gap",
    generation: {
      tool: COVERAGE_GAP_GENERATOR_PATH,
      mode: "registry-completeness-and-byte-identity",
    },
    sources: {
      webrefGrammar: WEBREF_GRAMMAR_PATH,
      webFeatures: WEB_FEATURES_DATA_PATH,
    },
    policy: {
      advisory: true,
      denominator:
        "CSS specs per CSS Snapshot 2025 boundary; Sass semantics covered on the oracle axis",
      redCondition: "registry-row-or-tier-reason-drift",
      capabilityTiers: [...COVERAGE_CAPABILITY_TIERS],
      namedReasons: [...COVERAGE_REASON_CODES],
      staticReductionDoesNotImplyValidation: true,
    },
    summary: {
      categoryCounts,
      tierCounts,
      recognizedFunctionCount: input.recognition.functions.length,
      recognizedAtRuleCount: input.recognition.atrules.length,
      cssFoldedFunctionCount: input.fold.cssFunctions.length,
      lessFoldedFunctionCount: input.fold.lessFunctions.length,
      rowCount: rows.length,
    },
    rows,
  };
}

export async function serializeCoverageGapReport(report: CoverageGapReport): Promise<string> {
  return formatGeneratedJson(COVERAGE_GAP_REPORT_PATH, report);
}

export function validateCoverageGapReport(
  report: CoverageGapReport,
  grammar: WebrefGrammarSnapshot,
  recognition: EngineRecognitionSurface,
  fold: EngineFoldSurface,
): void {
  assert.deepEqual(report.policy.capabilityTiers, [...COVERAGE_CAPABILITY_TIERS]);
  assert.deepEqual(report.policy.namedReasons, [...COVERAGE_REASON_CODES]);
  assert.equal(report.rows.length, grammar.entryCount);
  assert.equal(new Set(report.rows.map((row) => row.id)).size, report.rows.length);
  for (const category of COVERAGE_GAP_CATEGORIES) {
    const sourceRows = grammar.categories[category] ?? [];
    assert.ok(sourceRows.length > 0, `${category} must have source rows`);
    assert.equal(report.summary.categoryCounts[category], sourceRows.length);
  }
  assert.equal(
    Object.values(report.summary.tierCounts).reduce((total, count) => total + count, 0),
    report.rows.length,
  );
  for (const row of report.rows) {
    assert.ok(
      COVERAGE_CAPABILITY_TIERS.includes(row.capabilityTier),
      `${row.id} must have a registered capability tier`,
    );
    assert.ok(
      COVERAGE_REASON_CODES.includes(row.namedReason),
      `${row.id} must have a registered reason`,
    );
  }
  assertNoTimestampLikeKeys(report);
  for (const witness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
    assert.ok(recognition.functions.includes(witness), `${witness} must be recognized`);
    assert.ok(fold.cssFunctions.includes(witness), `${witness} must be CSS-folded`);
    const rows = findCoverageGapRows(report, "functions", witness);
    assert.ok(rows.length > 0, `${witness} must remain in the registry ledger`);
    assert.ok(rows.every((row) => row.capabilityTier === "T1"));
    assert.ok(rows.every((row) => row.measurements.staticallyReduced));
  }
}

export function findCoverageGapRow(
  report: CoverageGapReport,
  category: CoverageGapCategory,
  name: string,
  tier?: CoverageCapabilityTier,
): CoverageGapRow | undefined {
  const normalizedName = normalizeCoverageName(category, name);
  return report.rows.find(
    (row) =>
      row.category === category &&
      row.name === normalizedName &&
      (tier === undefined || row.capabilityTier === tier),
  );
}

export function findCoverageGapRows(
  report: CoverageGapReport,
  category: CoverageGapCategory,
  name: string,
): readonly CoverageGapRow[] {
  const normalizedName = normalizeCoverageName(category, name);
  return report.rows.filter((row) => row.category === category && row.name === normalizedName);
}

export function normalizeFunctionName(name: string): string {
  return name.trim().replace(/\(\)$/u, "").toLowerCase();
}

export function normalizeAtRuleName(name: string): string {
  return name.trim().toLowerCase();
}

export function extractRustStringArray(source: string, constName: string): readonly string[] {
  const escapedName = constName.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const match = new RegExp(
    `const\\s+${escapedName}\\s*:\\s*&\\[&str\\]\\s*=\\s*&\\[([\\s\\S]*?)\\];`,
    "u",
  ).exec(source);
  assert.ok(match, `missing Rust string array ${constName}`);
  return sortedUnique(extractStringLiterals(match[1]));
}

export function extractSpecializedFunctionArms(source: string): readonly string[] {
  const body = extractRustFunctionBody(source, "specialized_function_kind");
  return sortedUnique([
    ...Array.from(body.matchAll(/text\.eq_ignore_ascii_case\("([^"]+)"\)/gu), (match) =>
      normalizeFunctionName(match[1]),
    ),
    ...Array.from(
      body.matchAll(/matches_ignore_ascii_case\(\s*text,\s*&\[\s*"([^"]+)"\s*\]\s*\)/gu),
      (match) => normalizeFunctionName(match[1]),
    ),
  ]);
}

export function extractAtRuleRecognitionNames(source: string): readonly string[] {
  const atRuleSpec = extractRustFunctionBody(source, "at_rule_spec");
  const scssAtRuleSpec = extractRustFunctionBody(source, "scss_at_rule_spec");
  const pageMarginAtRule = extractRustFunctionBody(source, "is_page_margin_at_rule");
  return sortedUnique(
    [
      ...extractStringLiterals(atRuleSpec),
      ...extractStringLiterals(scssAtRuleSpec),
      ...extractStringLiterals(pageMarginAtRule),
    ]
      .filter((name) => name.startsWith("@"))
      .map(normalizeAtRuleName),
  );
}

export function extractStaticCssFunctionSpecNames(source: string): readonly string[] {
  const names: string[] = [];
  for (const functionName of [
    "substitute_static_css_function_references_in_value",
    "substitute_static_css_function_references_in_value_until_stable",
    "lower_static_color_function_references_with_lexer",
  ]) {
    let searchIndex = 0;
    while (searchIndex < source.length) {
      const callIndex = source.indexOf(functionName, searchIndex);
      if (callIndex < 0) {
        break;
      }
      const tableStart = source.indexOf("&[", callIndex);
      if (tableStart < 0) {
        break;
      }
      const nextCallIndex = source.indexOf(functionName, callIndex + functionName.length);
      if (nextCallIndex >= 0 && nextCallIndex < tableStart) {
        searchIndex = nextCallIndex;
        continue;
      }
      const tableSource = extractRustBracketBody(source, tableStart + 1, "[", "]");
      names.push(...extractStaticCssFunctionSpecPairs(tableSource));
      searchIndex = tableStart + tableSource.length;
    }
  }
  return sortedUnique(names.map(normalizeFunctionName));
}

export function extractNativeCssFunctionFoldNames(source: string): readonly string[] {
  if (!source.includes("native_css_if_function_static_edits")) {
    return [];
  }
  return ["if"];
}

export function extractLessTrigFoldNames(source: string): readonly string[] {
  return sortedUnique(
    Array.from(
      source.matchAll(/parse_static_less_(sin|cos|tan|asin|acos|atan)_value/gu),
      (match) => match[1],
    ),
  );
}

export function mathRecognitionResidue(
  recognition: EngineRecognitionSurface,
  fold: EngineFoldSurface,
): readonly string[] {
  const mathNames = new Set(recognition.valueNameTables.VALUES_L4_MATH_FUNCTION_NAMES ?? []);
  const cssFolded = new Set(fold.cssFunctions);
  return sortedUnique([...mathNames].filter((name) => !cssFolded.has(name)));
}

function buildCoverageGapRow(
  category: CoverageGapCategory,
  entry: WebrefGrammarSnapshot["categories"][string][number],
  capabilityTier: CoverageCapabilityTier,
  recognized: boolean,
  staticallyReduced: boolean,
  webFeatureIndex: WebFeatureIndex,
): CoverageGapRow {
  const normalizedName = normalizeCoverageName(category, entry.name);
  return {
    id: `${category}:${entry.sourceOrdinal}:${normalizedName}`,
    category,
    name: normalizedName,
    href: entry.href,
    sourceOrdinal: entry.sourceOrdinal,
    syntaxAvailable: entry.syntax !== null,
    boundaryClassification: entry.boundary.classification,
    capabilityTier,
    namedReason: coverageReasonForRow(entry, capabilityTier),
    measurements: { recognized, staticallyReduced },
    baseline: baselineRankForFeature(category, normalizedName, webFeatureIndex),
  };
}

function coverageReasonForRow(
  entry: WebrefGrammarSnapshot["categories"][string][number],
  capabilityTier: CoverageCapabilityTier,
): CoverageReasonCode {
  if (entry.boundary.classification === "forward-tier") {
    return "forward-specification";
  }
  if (entry.syntax === null) {
    return "upstream-syntax-absent";
  }
  return capabilityTier === "T0"
    ? "engine-recognition-not-observed"
    : "engine-validation-not-observed";
}

function baselineRankForFeature(
  category: CoverageGapCategory,
  name: string,
  webFeatureIndex: WebFeatureIndex,
): CoverageGapBaselineRank {
  const candidate = findBestWebFeature(category, name, webFeatureIndex);
  if (!candidate) {
    return { status: "unknown", sortRank: 3, featureId: null, featureName: null };
  }
  const baseline = candidate.entry.status?.baseline;
  if (baseline === "high") {
    return {
      status: "high",
      sortRank: 0,
      featureId: candidate.id,
      featureName: candidate.entry.name ?? candidate.id,
      baselineHigh: candidate.entry.status?.baseline_high_date,
      baselineLow: candidate.entry.status?.baseline_low_date,
    };
  }
  if (baseline === "low") {
    return {
      status: "low",
      sortRank: 1,
      featureId: candidate.id,
      featureName: candidate.entry.name ?? candidate.id,
      baselineLow: candidate.entry.status?.baseline_low_date,
    };
  }
  return {
    status: "limited",
    sortRank: 2,
    featureId: candidate.id,
    featureName: candidate.entry.name ?? candidate.id,
  };
}

function findBestWebFeature(
  category: CoverageGapCategory,
  name: string,
  webFeatureIndex: WebFeatureIndex,
): { readonly id: string; readonly entry: WebFeatureEntry } | null {
  const bare = category === "atrules" ? name.replace(/^@/u, "") : name;
  const compatPrefix =
    category === "atrules"
      ? "css.at-rules"
      : category === "properties"
        ? "css.properties"
        : category === "selectors"
          ? "css.selectors"
          : "css.types";
  const exactCompat = `${compatPrefix}.${bare}`;
  const normalizedBare = normalizeSearchText(bare);
  return (
    webFeatureIndex.byId.get(bare) ??
    webFeatureIndex.byId.get(normalizedBare) ??
    webFeatureIndex.byCompatKey.get(exactCompat) ??
    webFeatureIndex.byNormalizedName.get(normalizedBare) ??
    null
  );
}

function buildWebFeatureIndex(webFeaturesData: WebFeaturesData): WebFeatureIndex {
  const byId = new Map<string, IndexedWebFeature>();
  const byCompatKey = new Map<string, IndexedWebFeature>();
  const byNormalizedName = new Map<string, IndexedWebFeature>();
  for (const [id, entry] of Object.entries(webFeaturesData.features ?? {})) {
    const candidate = { id, entry };
    setPreferredFeature(byId, id, candidate);
    setPreferredFeature(byId, normalizeSearchText(id), candidate);
    if (entry.name) {
      setPreferredFeature(byNormalizedName, normalizeSearchText(entry.name), candidate);
    }
    for (const compatKey of entry.compat_features ?? []) {
      setPreferredFeature(byCompatKey, compatKey.toLowerCase(), candidate);
    }
  }
  return { byId, byCompatKey, byNormalizedName };
}

function setPreferredFeature(
  index: Map<string, IndexedWebFeature>,
  key: string,
  candidate: IndexedWebFeature,
): void {
  const current = index.get(key);
  if (
    !current ||
    baselineStatusSortRank(candidate.entry) < baselineStatusSortRank(current.entry) ||
    (baselineStatusSortRank(candidate.entry) === baselineStatusSortRank(current.entry) &&
      compareStrings(candidate.id, current.id) < 0)
  ) {
    index.set(key, candidate);
  }
}

function baselineStatusSortRank(entry: WebFeatureEntry): number {
  switch (entry.status?.baseline) {
    case "high":
      return 0;
    case "low":
      return 1;
    case false:
      return 2;
    default:
      return 3;
  }
}

function compareCoverageGapRows(left: CoverageGapRow, right: CoverageGapRow): number {
  return (
    COVERAGE_GAP_CATEGORIES.indexOf(left.category) -
      COVERAGE_GAP_CATEGORIES.indexOf(right.category) ||
    left.sourceOrdinal - right.sourceOrdinal ||
    compareStrings(left.name, right.name)
  );
}

function normalizeCoverageName(category: CoverageGapCategory, name: string): string {
  if (category === "functions") {
    return normalizeFunctionName(name);
  }
  if (category === "atrules") {
    return normalizeAtRuleName(name);
  }
  return name.trim().toLowerCase();
}

function injectCoverageLedgerFaults(
  report: CoverageGapReport,
  options: CoverageGapBuildOptions,
): CoverageGapReport {
  if (!options.injectUntieredRow && !options.injectFreeTextReason) {
    return report;
  }
  const [firstRow, ...remainingRows] = report.rows;
  assert.ok(firstRow, "coverage ledger injection requires at least one row");
  return {
    ...report,
    rows: [
      {
        ...firstRow,
        capabilityTier: options.injectUntieredRow
          ? ("" as CoverageCapabilityTier)
          : firstRow.capabilityTier,
        namedReason: options.injectFreeTextReason
          ? ("later" as CoverageReasonCode)
          : firstRow.namedReason,
      },
      ...remainingRows,
    ],
  };
}

function extractStringLiterals(source: string): readonly string[] {
  return Array.from(
    source.matchAll(/"((?:\\.|[^"\\])*)"/gu),
    (match) => JSON.parse(`"${match[1]}"`) as string,
  );
}

function extractRustFunctionBody(source: string, functionName: string): string {
  const index = source.indexOf(`fn ${functionName}`);
  assert.ok(index >= 0, `missing Rust function ${functionName}`);
  const openBraceIndex = source.indexOf("{", index);
  assert.ok(openBraceIndex >= 0, `missing Rust function body ${functionName}`);
  let depth = 0;
  for (let current = openBraceIndex; current < source.length; current += 1) {
    const char = source[current];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(openBraceIndex + 1, current);
      }
    }
  }
  throw new Error(`unterminated Rust function ${functionName}`);
}

function extractRustBracketBody(
  source: string,
  openBracketIndex: number,
  openBracket: string,
  closeBracket: string,
): string {
  assert.equal(source[openBracketIndex], openBracket);
  let depth = 0;
  for (let current = openBracketIndex; current < source.length; current += 1) {
    const char = source[current];
    if (char === openBracket) {
      depth += 1;
    } else if (char === closeBracket) {
      depth -= 1;
      if (depth === 0) {
        return source.slice(openBracketIndex + 1, current);
      }
    }
  }
  throw new Error(`unterminated Rust bracket body at ${openBracketIndex}`);
}

function extractStaticCssFunctionSpecPairs(source: string): readonly string[] {
  return Array.from(
    source.matchAll(/"([A-Za-z][A-Za-z0-9_-]*)"\s*,\s*[A-Za-z_][A-Za-z0-9_]*/gu),
    (match) => match[1],
  );
}

function readRustSourcesRecursively(directory: string): readonly string[] {
  const sources: string[] = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      sources.push(...readRustSourcesRecursively(fullPath));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      sources.push(readText(fullPath));
    }
  }
  return sources;
}

function assertNoTimestampLikeKeys(value: unknown): void {
  const stack: unknown[] = [value];
  while (stack.length > 0) {
    const current = stack.pop();
    if (!current || typeof current !== "object") {
      continue;
    }
    if (Array.isArray(current)) {
      stack.push(...current);
      continue;
    }
    for (const [key, nestedValue] of Object.entries(current)) {
      assert.ok(
        !/(generatedAt|timestamp|last_changed|Date)$/u.test(key),
        `coverage gap report must not contain timestamp-like key ${key}`,
      );
      stack.push(nestedValue);
    }
  }
}

function sortedUnique(values: readonly string[]): string[] {
  return [...new Set(values.filter((value) => value.length > 0))].toSorted(compareStrings);
}

function normalizeSearchText(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/\(\)/gu, "")
    .replace(/^@/u, "")
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-|-$/gu, "");
}

function compareStrings(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readText(filePath)) as T;
}

function readText(filePath: string): string {
  assert.ok(existsSync(filePath), `${filePath} must exist`);
  return readFileSync(filePath, "utf8");
}
