import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

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
const NOT_DIFFED_CATEGORIES = ["properties", "selectors", "types"] as const;

const EXPLICIT_CSS_FOLD_SURFACES = [
  "rust/crates/omena-transform-passes/src/domains/calc.rs",
  "rust/crates/omena-transform-passes/src/domains/static_eval.rs",
  "rust/crates/omena-transform-passes/src/domains/unit_transform.rs",
  "rust/crates/omena-transform-passes/src/domains/unit_filter.rs",
  "rust/crates/omena-transform-passes/src/domains/color_lowering.rs",
  "rust/crates/omena-transform-passes/src/domains/color.rs",
] as const;

export type CoverageGapCategory = "atrules" | "functions";
export type CoverageGapTier = "recognition" | "fold";
export type CoverageGapBaselineStatus = "high" | "low" | "limited" | "unknown";
export type CoverageGapAssertion = "engine-not-shown-to-recognize-or-reduce";

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
  readonly tier: CoverageGapTier;
  readonly name: string;
  readonly assertion: CoverageGapAssertion;
  readonly baseline: CoverageGapBaselineRank;
}

export interface CoverageGapReport {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.coverage-gap";
  readonly generation: {
    readonly tool: typeof COVERAGE_GAP_GENERATOR_PATH;
    readonly mode: "advisory-byte-identity";
  };
  readonly sources: {
    readonly webrefGrammar: typeof WEBREF_GRAMMAR_PATH;
    readonly webFeatures: typeof WEB_FEATURES_DATA_PATH;
  };
  readonly policy: {
    readonly advisory: true;
    readonly redCondition: "report-byte-drift-only";
    readonly assertion: CoverageGapAssertion;
    readonly notDiffedCategories: readonly (typeof NOT_DIFFED_CATEGORIES)[number][];
    readonly foldGapScope: "css-functions-only";
    readonly excludedFoldArms: readonly string[];
    readonly lessFoldDialect: "separate-from-css-subtraction";
  };
  readonly summary: {
    readonly recognizedFunctionCount: number;
    readonly recognizedAtRuleCount: number;
    readonly cssFoldedFunctionCount: number;
    readonly lessFoldedFunctionCount: number;
    readonly recognitionGapCount: number;
    readonly foldGapCount: number;
    readonly rowCount: number;
  };
  readonly rows: readonly CoverageGapRow[];
}

export interface WebrefGrammarSnapshot {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.webref-grammar";
  readonly source: { readonly package: string; readonly version: string; readonly gitHead: string };
  readonly generation: { readonly tool: string };
  readonly entryCount: number;
  readonly categories: Record<
    string,
    readonly { readonly name: string; readonly syntax: string }[]
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

export function buildCoverageGapReportFromRepo(repoRoot = process.cwd()): CoverageGapReport {
  const sources = loadCoverageGapEngineSources(repoRoot);
  const grammar = readJson<WebrefGrammarSnapshot>(path.join(repoRoot, WEBREF_GRAMMAR_PATH));
  const webFeaturesData = readJson<WebFeaturesData>(path.join(repoRoot, WEB_FEATURES_DATA_PATH));
  const recognition = extractEngineRecognitionSurface(sources);
  const fold = extractEngineFoldSurface(sources);
  const report = buildCoverageGapReport({ grammar, webFeaturesData, recognition, fold });
  validateCoverageGapReport(report, recognition, fold);
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
  const specFunctions = sortedUnique(
    (input.grammar.categories.functions ?? []).map((entry) => normalizeFunctionName(entry.name)),
  );
  const specAtRules = sortedUnique(
    (input.grammar.categories.atrules ?? []).map((entry) => normalizeAtRuleName(entry.name)),
  );
  const recognizedFunctions = new Set(input.recognition.functions.map(normalizeFunctionName));
  const recognizedAtRules = new Set(input.recognition.atrules.map(normalizeAtRuleName));
  const cssFoldedFunctions = new Set(input.fold.cssFunctions.map(normalizeFunctionName));

  const recognitionRows: CoverageGapRow[] = [
    ...specAtRules
      .filter((name) => !recognizedAtRules.has(name))
      .map((name) => buildCoverageGapRow("atrules", "recognition", name, input.webFeaturesData)),
    ...specFunctions
      .filter((name) => !recognizedFunctions.has(name))
      .map((name) => buildCoverageGapRow("functions", "recognition", name, input.webFeaturesData)),
  ];

  const foldRows = input.recognition.functions
    .map(normalizeFunctionName)
    .filter((name) => !FOLD_GAP_EXCLUDED_SPECIALIZED_ARMS.has(name))
    .filter((name) => !cssFoldedFunctions.has(name))
    .map((name) => buildCoverageGapRow("functions", "fold", name, input.webFeaturesData));

  const rows = [...recognitionRows, ...foldRows].toSorted(compareCoverageGapRows);

  return {
    schemaVersion: "0",
    product: "omena-spec-audit.coverage-gap",
    generation: {
      tool: COVERAGE_GAP_GENERATOR_PATH,
      mode: "advisory-byte-identity",
    },
    sources: {
      webrefGrammar: WEBREF_GRAMMAR_PATH,
      webFeatures: WEB_FEATURES_DATA_PATH,
    },
    policy: {
      advisory: true,
      redCondition: "report-byte-drift-only",
      assertion: "engine-not-shown-to-recognize-or-reduce",
      notDiffedCategories: [...NOT_DIFFED_CATEGORIES],
      foldGapScope: "css-functions-only",
      excludedFoldArms: [...FOLD_GAP_EXCLUDED_SPECIALIZED_ARMS].toSorted(compareStrings),
      lessFoldDialect: "separate-from-css-subtraction",
    },
    summary: {
      recognizedFunctionCount: input.recognition.functions.length,
      recognizedAtRuleCount: input.recognition.atrules.length,
      cssFoldedFunctionCount: input.fold.cssFunctions.length,
      lessFoldedFunctionCount: input.fold.lessFunctions.length,
      recognitionGapCount: recognitionRows.length,
      foldGapCount: foldRows.length,
      rowCount: rows.length,
    },
    rows,
  };
}

export function serializeCoverageGapReport(report: CoverageGapReport): string {
  return `${JSON.stringify(report, null, 2)
    .replace(
      /"notDiffedCategories": \[\n\s+"properties",\n\s+"selectors",\n\s+"types"\n\s+\]/u,
      '"notDiffedCategories": ["properties", "selectors", "types"]',
    )
    .replace(
      /"excludedFoldArms": \[\n\s+"attr",\n\s+"env",\n\s+"var"\n\s+\]/u,
      '"excludedFoldArms": ["attr", "env", "var"]',
    )}\n`;
}

export function validateCoverageGapReport(
  report: CoverageGapReport,
  recognition: EngineRecognitionSurface,
  fold: EngineFoldSurface,
): void {
  assert.deepEqual(report.policy.notDiffedCategories, [...NOT_DIFFED_CATEGORIES]);
  assertNoTimestampLikeKeys(report);
  for (const witness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
    assert.ok(recognition.functions.includes(witness), `${witness} must be recognized`);
    assert.ok(fold.cssFunctions.includes(witness), `${witness} must be CSS-folded`);
    assert.equal(findCoverageGapRow(report, "functions", witness, "fold"), undefined);
    assert.equal(findCoverageGapRow(report, "functions", witness, "recognition"), undefined);
  }
  for (const excluded of FOLD_GAP_EXCLUDED_SPECIALIZED_ARMS) {
    assert.equal(findCoverageGapRow(report, "functions", excluded, "fold"), undefined);
  }
}

export function findCoverageGapRow(
  report: CoverageGapReport,
  category: CoverageGapCategory,
  name: string,
  tier?: CoverageGapTier,
): CoverageGapRow | undefined {
  const normalizedName =
    category === "functions" ? normalizeFunctionName(name) : normalizeAtRuleName(name);
  return report.rows.find(
    (row) =>
      row.category === category &&
      row.name === normalizedName &&
      (tier === undefined || row.tier === tier),
  );
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
  tier: CoverageGapTier,
  name: string,
  webFeaturesData: WebFeaturesData,
): CoverageGapRow {
  const normalizedName =
    category === "functions" ? normalizeFunctionName(name) : normalizeAtRuleName(name);
  return {
    id: `${category}:${tier}:${normalizedName}`,
    category,
    tier,
    name: normalizedName,
    assertion: "engine-not-shown-to-recognize-or-reduce",
    baseline: baselineRankForFeature(category, normalizedName, webFeaturesData),
  };
}

function baselineRankForFeature(
  category: CoverageGapCategory,
  name: string,
  webFeaturesData: WebFeaturesData,
): CoverageGapBaselineRank {
  const candidate = findBestWebFeature(category, name, webFeaturesData);
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
  webFeaturesData: WebFeaturesData,
): { readonly id: string; readonly entry: WebFeatureEntry } | null {
  const features = webFeaturesData.features ?? {};
  const bare = category === "atrules" ? name.replace(/^@/u, "") : name;
  const exactCompat = category === "atrules" ? `css.at-rules.${bare}` : `css.types.${bare}`;
  const normalizedBare = normalizeSearchText(bare);

  const direct = features[bare] ?? features[normalizedBare];
  if (direct) {
    return { id: features[bare] ? bare : normalizedBare, entry: direct };
  }

  const matches: { id: string; entry: WebFeatureEntry; score: number }[] = [];
  for (const [id, entry] of Object.entries(features)) {
    const compatFeatures = (entry.compat_features ?? []).map((compatKey) =>
      compatKey.toLowerCase(),
    );
    const cssCompatFeatures = compatFeatures.filter((compatKey) =>
      category === "atrules"
        ? compatKey.startsWith("css.at-rules.")
        : compatKey.startsWith("css.types."),
    );
    let score = -1;
    if (cssCompatFeatures.some((compatKey) => compatKey === exactCompat)) {
      score = 0;
    } else if (
      cssCompatFeatures.some(
        (compatKey) => compatKey.endsWith(`.${bare}`) || compatKey.includes(`.${bare}.`),
      )
    ) {
      score = 1;
    } else if (normalizeSearchText(entry.name ?? "") === normalizedBare) {
      score = 2;
    } else if (normalizeSearchText(id).includes(normalizedBare)) {
      score = 3;
    }
    if (score >= 0) {
      matches.push({ id, entry, score });
    }
  }
  matches.sort(
    (left, right) =>
      left.score - right.score ||
      baselineStatusSortRank(left.entry) - baselineStatusSortRank(right.entry) ||
      compareStrings(left.id, right.id),
  );
  return matches[0] ? { id: matches[0].id, entry: matches[0].entry } : null;
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
    compareStrings(left.tier, right.tier) ||
    left.baseline.sortRank - right.baseline.sortRank ||
    compareOptionalStrings(
      left.baseline.baselineHigh ?? left.baseline.baselineLow,
      right.baseline.baselineHigh ?? right.baseline.baselineLow,
    ) ||
    compareStrings(left.category, right.category) ||
    compareStrings(left.name, right.name)
  );
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

function compareOptionalStrings(left: string | undefined, right: string | undefined): number {
  if (left === right) {
    return 0;
  }
  if (left === undefined) {
    return 1;
  }
  if (right === undefined) {
    return -1;
  }
  return compareStrings(left, right);
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
