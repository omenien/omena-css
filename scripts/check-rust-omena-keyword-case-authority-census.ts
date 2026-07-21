import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type KeywordCaseClassification =
  | "DEFECT-REACHABLE"
  | "PROTECTED-BY-PARSER"
  | "TEST-ONLY"
  | "ORACLE-DEMOTED";

interface KeywordCaseSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly idiom: string;
  readonly keyword: string;
  readonly classification: KeywordCaseClassification;
  readonly disposition: string;
  readonly classificationReason: string;
  readonly evidence: string;
}

interface KeywordCaseCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.keyword-case.authority-census";
  readonly policy: {
    readonly direction: "decrease-only";
    readonly enforced: boolean;
    readonly owningCheck: "rust/omena-keyword-case-authority-census";
    readonly packageScript: "check:rust-omena-keyword-case-authority-census";
  };
  readonly sourceCrates: readonly string[];
  readonly excludedAuthority: readonly string[];
  readonly baselineDefectReachableCount: number;
  readonly currentDefectReachableCount: number;
  readonly classificationCounts: Readonly<Record<KeywordCaseClassification, number>>;
  readonly sites: readonly KeywordCaseSite[];
  readonly siteDigest: string;
  readonly helperAuthority?: {
    readonly helper: "css_keyword";
    readonly functionKeys: readonly string[];
    readonly functionKeyDigest: string;
    readonly adHocCaseOperationCount: number;
    readonly adHocCaseOperations: readonly AdHocCaseOperation[];
  };
}

interface AdHocCaseOperation {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: "eq_ignore_ascii_case" | "to_ascii_lowercase";
}

interface SourceFunction {
  readonly name: string;
  readonly start: number;
  readonly end: number;
  readonly testOnly: boolean;
}

interface RustStringLiteral {
  readonly value: string;
  readonly start: number;
  readonly length: number;
}

interface NamedClassificationRule {
  readonly path: string;
  readonly function: string;
  readonly keyword: string;
  readonly classification: Exclude<KeywordCaseClassification, "DEFECT-REACHABLE" | "TEST-ONLY">;
  readonly reason: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = path.join(repoRoot, "rust/omena-keyword-case-authority-census.json");
const writeMode = process.argv.includes("--write");
const injectRawCaseMatch = process.env.OMENA_KEYWORD_CASE_TEST_INJECT_RAW_MATCH === "1";
const injectAdHocCaseFold = process.env.OMENA_KEYWORD_CASE_TEST_INJECT_AD_HOC_FOLD === "1";
const injectHelperFunction = process.env.OMENA_KEYWORD_CASE_TEST_INJECT_HELPER_FUNCTION === "1";

const sourceCrates = [
  "omena-cascade",
  "omena-cli",
  "omena-parser",
  "omena-query",
  "omena-scss-eval",
  "omena-semantic",
  "omena-transform-cst",
  "omena-transform-passes",
] as const;

const excludedAuthority = [
  "rust/crates/omena-parser/src/extension.rs#at_rule_spec",
  "rust/crates/omena-parser/src/extension.rs#at_rule_is_vendor_prefixed_keyframes",
] as const;

const bridgedHelperAuthorityFunctionKeys = [
  "rust/crates/omena-cascade/src/proofs.rs#prove_scope_flatten_candidate",
] as const;

const namedClassificationRules: readonly NamedClassificationRule[] = [
  {
    path: "rust/crates/omena-cascade/src/proofs.rs",
    function: "value_contains_relative_color_function",
    keyword: "from",
    classification: "PROTECTED-BY-PARSER",
    reason: "The only caller passes an ASCII-lowercased declaration value.",
  },
  {
    path: "rust/crates/omena-cascade/src/proofs.rs",
    function: "prove_scope_flatten_candidate",
    keyword: ":root",
    classification: "PROTECTED-BY-PARSER",
    reason:
      "The public crate boundary canonicalizes the root selector before entering the frozen proof implementation.",
  },
  {
    path: "rust/crates/omena-query/src/style/cascade_checker/custom_property_registration.rs",
    function: "strip_query_registration_important",
    keyword: "!important",
    classification: "PROTECTED-BY-PARSER",
    reason: "The comparison is performed on the function-local lowercase compact value.",
  },
  {
    path: "rust/crates/omena-query/src/style/cascade_checker/source_scanner.rs",
    function: "query_value_has_important_suffix",
    keyword: "!important",
    classification: "PROTECTED-BY-PARSER",
    reason: "The comparison is performed on the function-local ASCII-lowercased value.",
  },
  {
    path: "rust/crates/omena-query/src/style/diagnostics/cross_file_scc.rs",
    function: "summarize_omena_query_unified_cross_file_scc_diagnostics_from_report",
    keyword: "composes",
    classification: "PROTECTED-BY-PARSER",
    reason: "The edge kind is a canonical typed report label rather than source spelling.",
  },
  {
    path: "rust/crates/omena-semantic/src/lib.rs",
    function: "summarize_style_context_index",
    keyword: "layer",
    classification: "PROTECTED-BY-PARSER",
    reason: "The compared values are canonical semantic context-kind labels.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/domains/design_token.rs",
    function: "declaration_value_without_important",
    keyword: "!important",
    classification: "PROTECTED-BY-PARSER",
    reason: "The comparison is performed on the function-local lowercase value.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/domains/shorthand.rs",
    function: "declaration_value_without_important",
    keyword: "!important",
    classification: "PROTECTED-BY-PARSER",
    reason: "The comparison is performed on the function-local lowercase value.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/runtime/semantic_preservation.rs",
    function: "at_rule_prelude_is_semantically_transparent",
    keyword: "@layer",
    classification: "PROTECTED-BY-PARSER",
    reason: "The comparison is performed on the function-local lowercase prelude.",
  },
  {
    path: "rust/crates/omena-scss-eval/src/native_css.rs",
    function: "native_css_if_function_decision_surface",
    keyword: "supports",
    classification: "PROTECTED-BY-PARSER",
    reason: "The condition kind is a canonical evaluator label rather than source spelling.",
  },
  {
    path: "rust/crates/omena-semantic/src/lib.rs",
    function: "summarize_style_context_index",
    keyword: "scope",
    classification: "PROTECTED-BY-PARSER",
    reason: "The context kind is a canonical semantic label rather than source spelling.",
  },
] as const;

const targetKeywords = new Set([
  "!important",
  "@-webkit-keyframes",
  "@at-root",
  "@keyframes",
  "@layer",
  "@scope",
  "@supports",
  "@value",
  ":root",
  "as",
  "at-root",
  "composes",
  "from",
  "important",
  "keyframes",
  "layer",
  "scope",
  "supports",
  "to",
  "value",
]);

const existing = readExistingCensus();
const sites = scanKeywordCaseSites();
const helperAuthorityFunctionKeys = scanHelperAuthorityFunctionKeys();
const adHocCaseOperations = scanAdHocCaseOperations(helperAuthorityFunctionKeys);
const currentDefectReachableCount = sites.filter(
  (site) => site.classification === "DEFECT-REACHABLE",
).length;
const baselineDefectReachableCount =
  existing?.baselineDefectReachableCount ?? currentDefectReachableCount;

assert.ok(sites.length > 0, "keyword-case census must be non-vacuous");
assert.ok(
  sites.some((site) => site.classification === "PROTECTED-BY-PARSER"),
  "keyword-case census must observe parser-protected comparisons",
);
for (const classification of ["PROTECTED-BY-PARSER", "TEST-ONLY", "ORACLE-DEMOTED"] as const) {
  assert.ok(
    sites.some((site) => site.classification === classification),
    `keyword-case census must observe ${classification}`,
  );
}
assert.ok(
  baselineDefectReachableCount > 0,
  "keyword-case census must retain a non-vacuous defect baseline",
);
assert.ok(
  currentDefectReachableCount <= baselineDefectReachableCount,
  `keyword-case defect count increased: baseline=${baselineDefectReachableCount} current=${currentDefectReachableCount}`,
);
assert.deepEqual(
  adHocCaseOperations,
  [],
  "migrated keyword consumers must not introduce local ASCII case-folding authorities",
);

if (existing) {
  const previousDefectKeys = new Set(
    existing.sites.filter((site) => site.classification === "DEFECT-REACHABLE").map(stableSiteKey),
  );
  const addedDefects = sites
    .filter((site) => site.classification === "DEFECT-REACHABLE")
    .filter((site) => !previousDefectKeys.has(stableSiteKey(site)));
  assert.ok(
    currentDefectReachableCount <= existing.currentDefectReachableCount,
    `keyword-case defect count regressed: previous=${existing.currentDefectReachableCount} current=${currentDefectReachableCount} added=${JSON.stringify(addedDefects)}`,
  );
  assert.deepEqual(
    addedDefects,
    [],
    "keyword-case census found a new defect-reachable raw comparison",
  );
  if (existing.policy.enforced) {
    assert.equal(
      currentDefectReachableCount,
      0,
      "enforced keyword-case authority cannot retain defect-reachable comparisons",
    );
  }
}

const classificationCounts = Object.fromEntries(
  (["DEFECT-REACHABLE", "PROTECTED-BY-PARSER", "TEST-ONLY", "ORACLE-DEMOTED"] as const).map(
    (classification) => [
      classification,
      sites.filter((site) => site.classification === classification).length,
    ],
  ),
) as Record<KeywordCaseClassification, number>;

const census: KeywordCaseCensus = {
  schemaVersion: "0",
  product: "omena.keyword-case.authority-census",
  policy: {
    direction: "decrease-only",
    enforced: currentDefectReachableCount === 0,
    owningCheck: "rust/omena-keyword-case-authority-census",
    packageScript: "check:rust-omena-keyword-case-authority-census",
  },
  sourceCrates,
  excludedAuthority,
  baselineDefectReachableCount,
  currentDefectReachableCount,
  classificationCounts,
  sites,
  siteDigest: `sha256:${createHash("sha256").update(JSON.stringify(sites)).digest("hex")}`,
  helperAuthority: {
    helper: "css_keyword",
    functionKeys: helperAuthorityFunctionKeys,
    functionKeyDigest: `sha256:${createHash("sha256")
      .update(JSON.stringify(helperAuthorityFunctionKeys))
      .digest("hex")}`,
    adHocCaseOperationCount: adHocCaseOperations.length,
    adHocCaseOperations,
  },
};

const expected = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  assert.equal(
    injectRawCaseMatch || injectAdHocCaseFold || injectHelperFunction,
    false,
    "test injection cannot be combined with census regeneration",
  );
  writeFileSync(censusPath, expected);
  const formatResult = spawnSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(
    formatResult.status,
    0,
    `failed to format keyword-case census: ${(formatResult.stderr ?? "").trim()}`,
  );
} else {
  assert.ok(
    existsSync(censusPath),
    "keyword-case authority census is missing; run its update command",
  );
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "keyword-case authority census is stale",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      baselineDefectReachableCount: census.baselineDefectReachableCount,
      currentDefectReachableCount: census.currentDefectReachableCount,
      classificationCounts: census.classificationCounts,
      siteDigest: census.siteDigest,
      enforced: census.policy.enforced,
    },
    null,
    2,
  )}\n`,
);

function readExistingCensus(): KeywordCaseCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as KeywordCaseCensus;
  assert.equal(parsed.schemaVersion, "0", "keyword-case census schemaVersion");
  assert.equal(parsed.product, "omena.keyword-case.authority-census", "keyword-case product");
  assert.equal(parsed.policy.direction, "decrease-only", "keyword-case policy direction");
  assert.equal(
    parsed.currentDefectReachableCount,
    parsed.sites.filter((site) => site.classification === "DEFECT-REACHABLE").length,
    "committed defect-reachable count",
  );
  assert.equal(
    parsed.siteDigest,
    `sha256:${createHash("sha256").update(JSON.stringify(parsed.sites)).digest("hex")}`,
    "committed keyword-case site digest",
  );
  if (parsed.helperAuthority?.functionKeyDigest !== undefined) {
    assert.equal(
      parsed.helperAuthority.functionKeyDigest,
      `sha256:${createHash("sha256")
        .update(JSON.stringify(parsed.helperAuthority.functionKeys))
        .digest("hex")}`,
      "committed helper-authority function-key digest",
    );
  }
  return parsed;
}

function scanKeywordCaseSites(): KeywordCaseSite[] {
  const found: KeywordCaseSite[] = [];
  for (const relativePath of trackedSources()) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectRawCaseMatch && relativePath === "rust/crates/omena-semantic/src/layer_tree.rs") {
      source = `fn injected_keyword_case_drift(text: &str) { let _ = text.strip_prefix("@layer"); }\n${source}`;
    }
    const rustSource = scanRustSource(source);
    const functions = sourceFunctions(rustSource.code);
    for (const literal of rustSource.stringLiterals) {
      const rawKeyword = literal.value;
      const keyword = canonicalKeyword(rawKeyword);
      if (!keyword) continue;
      const operation = comparisonOperation(source, literal.start, literal.length);
      if (!operation) continue;
      const sourceFunction = functionAt(functions, literal.start);
      if (isExcludedAuthority(relativePath, sourceFunction?.name)) continue;
      const classification = classifySite(
        relativePath,
        sourceFunction,
        keyword,
        operation.idiom,
        operation.context,
      );
      const classificationReason = classificationReasonFor(
        relativePath,
        sourceFunction,
        keyword,
        classification,
      );
      const line = lineNumberAt(source, literal.start);
      found.push({
        path: relativePath,
        line,
        function: sourceFunction?.name ?? "<module>",
        idiom: operation.idiom,
        keyword,
        classification,
        disposition: dispositionFor(classification),
        classificationReason,
        evidence: source.split(/\r?\n/u)[line - 1]?.trim().replace(/\s+/gu, " ") ?? "",
      });
    }
  }

  const byKey = new Map<string, KeywordCaseSite>();
  for (const site of found) {
    const key = `${site.path}:${site.line}:${site.function}:${site.idiom}:${site.keyword}`;
    byKey.set(key, site);
  }
  const ordered = [...byKey.values()].toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.function.localeCompare(right.function) ||
      left.keyword.localeCompare(right.keyword) ||
      left.idiom.localeCompare(right.idiom),
  );
  for (const site of ordered) {
    assert.ok(site.line > 0, `keyword-case site line: ${site.path}`);
    assert.ok(site.evidence.length > 0, `keyword-case site evidence: ${site.path}:${site.line}`);
  }
  return ordered;
}

function trackedSources(): string[] {
  const files = new Set<string>();
  for (const crateName of sourceCrates) {
    const result = spawnSync("git", ["ls-files", `rust/crates/${crateName}`], {
      cwd: repoRoot,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, `git ls-files failed for ${crateName}: ${result.stderr.trim()}`);
    for (const sourcePath of result.stdout.split(/\r?\n/u)) {
      if (
        sourcePath.endsWith(".rs") &&
        sourcePath.includes("/src/") &&
        !sourcePath.endsWith("_generated.rs")
      ) {
        files.add(sourcePath);
      }
    }
  }
  return [...files].toSorted();
}

function scanHelperAuthorityFunctionKeys(): string[] {
  const keys = new Set<string>(bridgedHelperAuthorityFunctionKeys);
  for (const relativePath of trackedSources()) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectHelperFunction && relativePath === "rust/crates/omena-semantic/src/layer_tree.rs") {
      source = `${source}\nfn injected_helper_consumer(text: &str) { let _ = css_keyword(text).equals("layer"); let _ = text.to_ascii_lowercase(); }\n`;
    }
    const rustSource = scanRustSource(source);
    for (const sourceFunction of sourceFunctions(rustSource.code)) {
      if (sourceFunction.testOnly) continue;
      const body = rustSource.code.slice(sourceFunction.start, sourceFunction.end);
      if (/\bcss_keyword\s*\(/u.test(body)) {
        keys.add(`${relativePath}#${sourceFunction.name}`);
      }
    }
  }
  return [...keys].toSorted();
}

function canonicalKeyword(raw: string): string | undefined {
  const normalized = raw.trim().toLowerCase();
  const withoutDelimiter = normalized.endsWith(":") ? normalized.slice(0, -1) : normalized;
  const withoutTrailingSpace = withoutDelimiter.trimEnd();
  if (!targetKeywords.has(normalized) && !targetKeywords.has(withoutTrailingSpace)) {
    return undefined;
  }
  if (withoutTrailingSpace === "@-webkit-keyframes") return "@keyframes";
  if (withoutTrailingSpace === "important") return "!important";
  return withoutTrailingSpace;
}

function comparisonOperation(
  source: string,
  literalStart: number,
  literalLength: number,
): { readonly idiom: string; readonly context: string } | undefined {
  const before = source.slice(Math.max(0, literalStart - 220), literalStart);
  const after = source.slice(literalStart + literalLength, literalStart + literalLength + 80);
  const method = before.match(
    /\.\s*(equals|eq_ignore_ascii_case|strip_prefix|strip_suffix|starts_with|ends_with|contains|find|rfind)\s*\(\s*$/u,
  );
  if (method) {
    return {
      idiom: method[1].replaceAll("_", "-"),
      context: `${before.slice(-220)}${source.slice(literalStart, literalStart + literalLength)}${after}`,
    };
  }
  const helper = before.match(
    /\b(matches_ignore_ascii_case|css_keyword|top_level_token_text_index|strip_ascii_prefix_ignore_case|static_scss_split_header_at_keyword)\s*\([^;{}]{0,220}$/u,
  );
  if (helper) {
    return {
      idiom: helper[1].replaceAll("_", "-"),
      context: `${before.slice(-220)}${source.slice(literalStart, literalStart + literalLength)}${after}`,
    };
  }
  if (/(?:==|!=)\s*(?:Some\s*\(\s*)?$/u.test(before)) {
    return {
      idiom: "direct-equality",
      context: `${before.slice(-220)}${source.slice(literalStart, literalStart + literalLength)}${after}`,
    };
  }
  if (/^\s*\)?\s*(?:==|!=)/u.test(after)) {
    return {
      idiom: "reverse-equality",
      context: `${before.slice(-220)}${source.slice(literalStart, literalStart + literalLength)}${after}`,
    };
  }
  return undefined;
}

function classifySite(
  relativePath: string,
  sourceFunction: SourceFunction | undefined,
  keyword: string,
  idiom: string,
  context: string,
): KeywordCaseClassification {
  if (isTestPath(relativePath) || sourceFunction?.testOnly) return "TEST-ONLY";
  if (relativePath === "rust/crates/omena-cascade/src/selector.rs") {
    return "ORACLE-DEMOTED";
  }
  if (
    /\bcss_keyword\s*\(/u.test(context) ||
    idiom.includes("ignore-ascii-case") ||
    [
      "css-keyword",
      "static-scss-split-header-at-keyword",
      "strip-ascii-prefix-ignore-case",
      "top-level-token-text-index",
    ].includes(idiom)
  ) {
    return "PROTECTED-BY-PARSER";
  }
  const namedRule = namedClassificationRule(relativePath, sourceFunction?.name, keyword);
  if (namedRule) return namedRule.classification;
  return "DEFECT-REACHABLE";
}

function scanAdHocCaseOperations(functionKeys: readonly string[]): AdHocCaseOperation[] {
  const keysByPath = new Map<string, Set<string>>();
  for (const key of functionKeys) {
    const separator = key.lastIndexOf("#");
    assert.ok(separator > 0, `keyword authority function key: ${key}`);
    const relativePath = key.slice(0, separator);
    const functionName = key.slice(separator + 1);
    const names = keysByPath.get(relativePath) ?? new Set<string>();
    names.add(functionName);
    keysByPath.set(relativePath, names);
  }

  const operations: AdHocCaseOperation[] = [];
  for (const [relativePath, functionNames] of keysByPath) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectAdHocCaseFold && relativePath === "rust/crates/omena-semantic/src/layer_tree.rs") {
      const anchor = "    let text = syntax_node_text(node);";
      const injected = `${anchor}\n    let _local_case_authority = text.to_ascii_lowercase();`;
      assert.ok(source.includes(anchor), "ad-hoc case-fold injection anchor");
      source = source.replace(anchor, injected);
    }
    if (injectHelperFunction && relativePath === "rust/crates/omena-semantic/src/layer_tree.rs") {
      source = `${source}\nfn injected_helper_consumer(text: &str) { let _ = css_keyword(text).equals("layer"); let _ = text.to_ascii_lowercase(); }\n`;
    }
    const rustSource = scanRustSource(source);
    for (const sourceFunction of sourceFunctions(rustSource.code)) {
      if (!functionNames.has(sourceFunction.name)) continue;
      const body = rustSource.code.slice(sourceFunction.start, sourceFunction.end);
      for (const match of body.matchAll(/\b(eq_ignore_ascii_case|to_ascii_lowercase)\s*\(/gu)) {
        operations.push({
          path: relativePath,
          line: lineNumberAt(source, sourceFunction.start + match.index),
          function: sourceFunction.name,
          operation: match[1] as AdHocCaseOperation["operation"],
        });
      }
    }
  }
  return operations.toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.function.localeCompare(right.function) ||
      left.operation.localeCompare(right.operation),
  );
}

function classificationReasonFor(
  relativePath: string,
  sourceFunction: SourceFunction | undefined,
  keyword: string,
  classification: KeywordCaseClassification,
): string {
  if (classification === "TEST-ONLY") {
    return "The comparison belongs to an assertion or fixture rather than product logic.";
  }
  if (classification === "ORACLE-DEMOTED") {
    return "The conservative selector oracle remains a named non-authoritative scanner.";
  }
  const namedRule = namedClassificationRule(relativePath, sourceFunction?.name, keyword);
  if (namedRule) return namedRule.reason;
  if (classification === "PROTECTED-BY-PARSER") {
    return "The comparison routes through the shared ASCII-insensitive keyword authority.";
  }
  return "A product path compares source-derived CSS spelling case-sensitively.";
}

function namedClassificationRule(
  relativePath: string,
  functionName: string | undefined,
  keyword: string,
): NamedClassificationRule | undefined {
  return namedClassificationRules.find(
    (rule) =>
      rule.path === relativePath && rule.function === functionName && rule.keyword === keyword,
  );
}

function dispositionFor(classification: KeywordCaseClassification): string {
  switch (classification) {
    case "DEFECT-REACHABLE":
      return "port-to-shared-helper";
    case "PROTECTED-BY-PARSER":
      return "retain-case-insensitive-witness";
    case "TEST-ONLY":
      return "retain-test-evidence";
    case "ORACLE-DEMOTED":
      return "named-exempt-conservative-oracle";
  }
}

function isExcludedAuthority(relativePath: string, functionName: string | undefined): boolean {
  return excludedAuthority.includes(
    `${relativePath}#${functionName ?? "<module>"}` as (typeof excludedAuthority)[number],
  );
}

function isTestPath(relativePath: string): boolean {
  return (
    relativePath.includes("/tests/") ||
    relativePath.endsWith("/tests.rs") ||
    relativePath.endsWith("_tests.rs")
  );
}

function sourceFunctions(source: string): SourceFunction[] {
  const matches = [...source.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)];
  return matches.map((match, index) => {
    const start = match.index;
    const end = matches[index + 1]?.index ?? source.length;
    const attributes = source.slice(Math.max(0, start - 240), start);
    return {
      name: match[1],
      start,
      end,
      testOnly: /#\s*\[\s*(?:test|cfg\s*\(\s*test\s*\))\s*\]/u.test(attributes),
    };
  });
}

function functionAt(
  functions: readonly SourceFunction[],
  offset: number,
): SourceFunction | undefined {
  return functions.findLast(
    (sourceFunction) => sourceFunction.start <= offset && offset < sourceFunction.end,
  );
}

function stableSiteKey(site: KeywordCaseSite): string {
  return `${site.path}:${site.function}:${site.idiom}:${site.keyword}`;
}

function lineNumberAt(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function scanRustSource(source: string): {
  readonly code: string;
  readonly stringLiterals: readonly RustStringLiteral[];
} {
  const chars = source.split("");
  const stringLiterals: RustStringLiteral[] = [];
  let index = 0;
  while (index < source.length) {
    const char = chars[index];
    const next = chars[index + 1] ?? "";
    if (char === "/" && next === "/") {
      const end = source.indexOf("\n", index + 2);
      maskSourceRange(chars, index, end === -1 ? source.length : end);
      index = end === -1 ? source.length : end;
      continue;
    }
    if (char === "/" && next === "*") {
      let depth = 1;
      let end = index + 2;
      while (end < source.length && depth > 0) {
        if (source[end] === "/" && source[end + 1] === "*") {
          depth += 1;
          end += 2;
        } else if (source[end] === "*" && source[end + 1] === "/") {
          depth -= 1;
          end += 2;
        } else {
          end += 1;
        }
      }
      maskSourceRange(chars, index, end);
      index = end;
      continue;
    }

    const rawPrefix = source.slice(index).match(/^(?:br|cr|r)(#+)?"/u);
    if (rawPrefix && rustRawStringCanStartAt(source, index)) {
      const hashCount = rawPrefix[1]?.length ?? 0;
      const quoteStart = index + rawPrefix[0].length - 1;
      const terminator = `"${"#".repeat(hashCount)}`;
      const terminatorStart = source.indexOf(terminator, quoteStart + 1);
      if (terminatorStart !== -1) {
        const end = terminatorStart + terminator.length;
        stringLiterals.push({
          value: source.slice(quoteStart + 1, terminatorStart),
          start: quoteStart,
          length: end - quoteStart,
        });
        maskSourceRange(chars, index, end);
        index = end;
        continue;
      }
    }

    if (char === '"') {
      const end = rustQuotedStringEnd(source, index);
      if (end !== undefined) {
        stringLiterals.push({
          value: source.slice(index + 1, end),
          start: index,
          length: end + 1 - index,
        });
        maskSourceRange(chars, index, end + 1);
        index = end + 1;
        continue;
      }
    }

    if (char === "'") {
      const end = rustCharacterLiteralEndsAt(source, index);
      if (end !== undefined) {
        maskSourceRange(chars, index, end + 1);
        index = end + 1;
        continue;
      }
    }
    index += 1;
  }
  return { code: chars.join(""), stringLiterals };
}

function maskSourceRange(chars: string[], start: number, end: number): void {
  for (let index = start; index < end; index += 1) {
    if (chars[index] !== "\n" && chars[index] !== "\r") chars[index] = " ";
  }
}

function rustRawStringCanStartAt(source: string, start: number): boolean {
  const previous = source[start - 1];
  return previous === undefined || !/[A-Za-z0-9_]/u.test(previous);
}

function rustQuotedStringEnd(source: string, start: number): number | undefined {
  let escaped = false;
  for (let index = start + 1; index < source.length; index += 1) {
    const char = source[index];
    if (char === "\n" && !escaped) return undefined;
    if (!escaped && char === '"') return index;
    if (!escaped && char === "\\") escaped = true;
    else escaped = false;
  }
  return undefined;
}

function rustCharacterLiteralEndsAt(source: string, start: number): number | undefined {
  const valueStart = start + 1;
  if (source[valueStart] === "\\") {
    let escaped = false;
    for (
      let index = valueStart + 1;
      index < source.length && source[index] !== "\n" && index - start <= 16;
      index += 1
    ) {
      const char = source[index];
      if (!escaped && char === "'") return index;
      if (!escaped && char === "\\") escaped = true;
      else escaped = false;
    }
    return undefined;
  }
  const codePoint = source.codePointAt(valueStart);
  if (codePoint === undefined) return undefined;
  const width = codePoint > 0xffff ? 2 : 1;
  return source[valueStart + width] === "'" ? valueStart + width : undefined;
}
