import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type SiteDisposition = "migration-target" | "named-exempt";

interface RawScanSite {
  readonly path: string;
  readonly line: number;
  readonly idiom: string;
  readonly family: string;
  readonly disposition: SiteDisposition;
  readonly evidence: string;
  readonly reason?: string;
}

interface TokenCaseComparisonSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: TokenCaseOperation;
  readonly evidence: string;
}

interface NamedTokenCaseOperationSite extends TokenCaseComparisonSite {
  readonly reason: string;
}

type TokenCaseOperation =
  | "eq_ignore_ascii_case"
  | "to_ascii_lowercase"
  | "to_ascii_uppercase"
  | "to_lowercase"
  | "to_uppercase";

interface RawScanCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.syntax-authority.raw-scan-census";
  readonly policy: {
    readonly direction: "decrease-only";
    readonly enforced: true;
    readonly owningCheck: "rust/omena-syntax-authority-raw-scan-census";
    readonly packageScript: "check:rust-omena-syntax-authority-raw-scan-census";
  };
  readonly sourceRoots: readonly string[];
  readonly engineCrates: readonly string[];
  readonly excludedPaths: readonly string[];
  readonly baselineSiteCount: number;
  readonly currentSiteCount: number;
  readonly baselineNamedExemptSiteCount: number;
  readonly currentNamedExemptSiteCount: number;
  readonly sites: readonly RawScanSite[];
  readonly siteDigest: string;
  readonly tokenCaseComparison: {
    readonly policy: "helper-only";
    readonly helper: "matches_ignore_ascii_case";
    readonly adHocSiteCount: number;
    readonly sites: readonly TokenCaseComparisonSite[];
    readonly namedExemptSiteCount: number;
    readonly namedExemptSites: readonly NamedTokenCaseOperationSite[];
  };
}

interface IdiomPattern {
  readonly id: string;
  readonly expression: RegExp;
  readonly accept?: (match: RegExpMatchArray) => boolean;
}

interface NamedTokenCaseOperationRule {
  readonly path: string;
  readonly function: string;
  readonly operation: TokenCaseOperation;
  readonly evidence: string;
  readonly reason: string;
}

interface ProductPathMatrix {
  readonly schemaVersion: "0";
  readonly product: "omena-css.product-path-matrix";
  readonly entries: readonly {
    readonly crate: string;
    readonly role: string;
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = path.join(repoRoot, "rust/omena-syntax-authority-raw-scan-census.json");
const writeMode = process.argv.includes("--write");
const injectRawScan = process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_SCAN === "1";
const injectTokenCaseComparison =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_COMPARE === "1";
const injectLexerCaseComparison =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LEXER_CASE_COMPARE === "1";
const injectNamedTokenCaseExemptionDrift =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_EXEMPTION_DRIFT === "1";

const sourceRoots = ["rust/crates"] as const;
const productPathMatrix = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/omena-product-path-matrix.json"), "utf8"),
) as ProductPathMatrix;
assert.equal(productPathMatrix.schemaVersion, "0", "product-path matrix schemaVersion");
assert.equal(
  productPathMatrix.product,
  "omena-css.product-path-matrix",
  "product-path matrix product",
);
const engineCrates = productPathMatrix.entries
  .filter((entry) => entry.role === "R1" || entry.role === "R2")
  .map((entry) => entry.crate)
  .toSorted();
assert.ok(engineCrates.length > 0, "product-path matrix must identify engine crates");
assert.equal(new Set(engineCrates).size, engineCrates.length, "engine crate names must be unique");
const excludedPaths = [
  "rust/crates/omena-parser/src/bin/",
  "rust/crates/omena-parser/src/lex.rs",
  "rust/crates/omena-parser/src/facts/product_facts_authority_tests.rs",
  "rust/crates/omena-parser/src/tests.rs",
  "rust/crates/omena-parser/src/value_names.rs",
  "rust/crates/omena-syntax/",
  "rust/crates/omena-value-lattice/",
] as const;

const patterns: readonly IdiomPattern[] = [
  {
    id: "brace-find",
    expression: /\.(?:find|rfind)\s*\(\s*(?:b)?(["'])(?:\{|\})\1\s*\)/gu,
  },
  {
    id: "brace-list-search",
    expression: /\.(?:find|rfind)\s*\(\s*\[[^\]]*(?:\{|\}|;)[^\]]*\]\s*\)/gu,
  },
  {
    id: "brace-contains",
    expression:
      /\.contains\s*\(\s*(?:(?:b)?(["'])(?:\{|\}|;)\1|\[[^\]]*(?:\{|\}|;)[^\]]*\])\s*\)/gu,
  },
  {
    id: "find-next-brace-byte",
    expression: /\bfind_next_byte\s*\([^)]*(?:b)?(["'])(?:\{|\})\1[^)]*\)/gu,
  },
  {
    id: "matching-brace-helper",
    expression: /\bmatching_[A-Za-z0-9_]*brace[A-Za-z0-9_]*\s*\(/gu,
  },
  {
    id: "body-bounds-helper",
    expression: /\b[A-Za-z0-9_]*body_bounds[A-Za-z0-9_]*\s*\(/gu,
  },
  {
    id: "source-substring-gate",
    expression:
      /\b(?:source|source_text|text|canonical_text|statement|body|node_source|rule_source|segment)\.contains\s*\(\s*(?:r#+)?["'][^\n)]*["']\s*\)/gu,
    accept: (match) =>
      /(?:\{|\}|;|@|:|animation|keyframes|composes|calc\(|var\(|url\(|--)/u.test(match[0]),
  },
] as const;

const namedTokenCaseOperationRules: readonly NamedTokenCaseOperationRule[] = [
  {
    path: "rust/crates/omena-parser/src/extension.rs",
    function: "at_rule_spec",
    operation: "to_ascii_lowercase",
    evidence: "let lowered = text.to_ascii_lowercase();",
    reason: "The extension registry normalizes an at-rule name once before canonical dispatch.",
  },
  {
    path: "rust/crates/omena-parser/src/facts/at_rules.rs",
    function: "at_rule_fact_from_cst_token",
    operation: "to_ascii_lowercase",
    evidence: "source_text.to_ascii_lowercase()",
    reason: "Known CSS at-rule facts store a canonical lowercase public name.",
  },
  {
    path: "rust/crates/omena-parser/src/facts/css_modules.rs",
    function: "css_module_value_source_looks_like_style_request",
    operation: "to_ascii_lowercase",
    evidence: "let lower = source.to_ascii_lowercase();",
    reason: "A module request path is normalized for case-insensitive style-extension matching.",
  },
  {
    path: "rust/crates/omena-parser/src/public_product/syntax_index.rs",
    function: "declaration_syntax",
    operation: "to_ascii_lowercase",
    evidence: ".to_ascii_lowercase();",
    reason: "The product syntax index stores a canonical lowercase property identity.",
  },
  {
    path: "rust/crates/omena-parser/src/syntax_helpers.rs",
    function: "matches_ignore_ascii_case",
    operation: "eq_ignore_ascii_case",
    evidence: ".any(|candidate| value.eq_ignore_ascii_case(candidate))",
    reason: "This is the canonical parser token-comparison helper implementation.",
  },
] as const;

const existing = readExistingCensus();
const sites = scanRawSyntaxSites();
const tokenCaseOperations = scanTokenCaseOperations();
const tokenCaseComparisonSites = tokenCaseOperations.adHocSites;
const currentNamedExemptSiteCount = sites.filter(
  (site) => site.disposition === "named-exempt",
).length;
const baselineSiteCount = existing?.baselineSiteCount ?? sites.length;
const baselineNamedExemptSiteCount =
  existing?.baselineNamedExemptSiteCount ?? currentNamedExemptSiteCount;

assert.ok(sites.length > 0, "raw syntax scan census must be non-vacuous");
assert.ok(
  sites.some((site) => site.disposition === "named-exempt"),
  "raw syntax scan census must include named exemptions",
);
assert.ok(
  sites.length <= baselineSiteCount,
  `raw syntax scan count increased: baseline=${baselineSiteCount} current=${sites.length}`,
);
assert.ok(
  currentNamedExemptSiteCount <= baselineNamedExemptSiteCount,
  `named-exempt raw syntax scan count increased: baseline=${baselineNamedExemptSiteCount} current=${currentNamedExemptSiteCount}`,
);
assert.deepEqual(
  tokenCaseComparisonSites,
  [],
  "parser syntax-token case comparisons must route through matches_ignore_ascii_case",
);

if (existing && writeMode) {
  const previousKeys = new Set(existing.sites.map(stableSiteKey));
  const addedSites = sites.filter((site) => !previousKeys.has(stableSiteKey(site)));
  assert.deepEqual(
    addedSites,
    [],
    "the decrease-only census cannot adopt new raw syntax scan sites during regeneration",
  );
}

const census: RawScanCensus = {
  schemaVersion: "0",
  product: "omena.syntax-authority.raw-scan-census",
  policy: {
    direction: "decrease-only",
    enforced: true,
    owningCheck: "rust/omena-syntax-authority-raw-scan-census",
    packageScript: "check:rust-omena-syntax-authority-raw-scan-census",
  },
  sourceRoots,
  engineCrates,
  excludedPaths,
  baselineSiteCount,
  currentSiteCount: sites.length,
  baselineNamedExemptSiteCount,
  currentNamedExemptSiteCount,
  sites,
  siteDigest: `sha256:${createHash("sha256").update(JSON.stringify(sites)).digest("hex")}`,
  tokenCaseComparison: {
    policy: "helper-only",
    helper: "matches_ignore_ascii_case",
    adHocSiteCount: tokenCaseComparisonSites.length,
    sites: tokenCaseComparisonSites,
    namedExemptSiteCount: tokenCaseOperations.namedExemptSites.length,
    namedExemptSites: tokenCaseOperations.namedExemptSites,
  },
};

const expected = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  assert.ok(
    !injectRawScan &&
      !injectTokenCaseComparison &&
      !injectLexerCaseComparison &&
      !injectNamedTokenCaseExemptionDrift,
    "test injection cannot be combined with --write",
  );
  writeFileSync(censusPath, expected);
  const formatResult = spawnSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(
    formatResult.status,
    0,
    `failed to format generated census: ${(formatResult.stderr ?? "").trim()}`,
  );
} else {
  assert.ok(
    existsSync(censusPath),
    "syntax-authority raw scan census is missing; run the package update script",
  );
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "syntax-authority raw scan census is stale; regenerate after removing tracked raw scans",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      baselineSiteCount: census.baselineSiteCount,
      currentSiteCount: census.currentSiteCount,
      migrationTargetSiteCount: sites.length - currentNamedExemptSiteCount,
      namedExemptSiteCount: currentNamedExemptSiteCount,
      siteDigest: census.siteDigest,
      direction: census.policy.direction,
      enforced: census.policy.enforced,
      adHocTokenCaseComparisonCount: census.tokenCaseComparison.adHocSiteCount,
      namedExemptTokenCaseOperationCount: census.tokenCaseComparison.namedExemptSiteCount,
    },
    null,
    2,
  )}\n`,
);

function readExistingCensus(): RawScanCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as RawScanCensus;
  assert.equal(parsed.schemaVersion, "0", "raw scan census schemaVersion");
  assert.equal(parsed.product, "omena.syntax-authority.raw-scan-census", "raw scan product");
  assert.equal(parsed.policy.direction, "decrease-only", "raw scan direction");
  assert.equal(parsed.policy.enforced, true, "raw scan policy must be enforced");
  assert.equal(
    parsed.currentSiteCount,
    parsed.sites.length,
    "committed raw scan site count must equal its site array",
  );
  assert.equal(
    parsed.siteDigest,
    `sha256:${createHash("sha256").update(JSON.stringify(parsed.sites)).digest("hex")}`,
    "committed raw scan site digest",
  );
  if (parsed.tokenCaseComparison !== undefined) {
    assert.equal(parsed.tokenCaseComparison.policy, "helper-only", "token case policy");
    assert.equal(
      parsed.tokenCaseComparison.helper,
      "matches_ignore_ascii_case",
      "token case helper",
    );
    assert.equal(
      parsed.tokenCaseComparison.adHocSiteCount,
      parsed.tokenCaseComparison.sites.length,
      "token case site count",
    );
    if (parsed.tokenCaseComparison.namedExemptSites !== undefined) {
      assert.equal(
        parsed.tokenCaseComparison.namedExemptSiteCount,
        parsed.tokenCaseComparison.namedExemptSites.length,
        "named-exempt token case operation count",
      );
    }
  }
  return parsed;
}

function scanTokenCaseOperations(): {
  readonly adHocSites: readonly TokenCaseComparisonSite[];
  readonly namedExemptSites: readonly NamedTokenCaseOperationSite[];
} {
  const caseOperation =
    /\.\s*(eq_ignore_ascii_case|to_ascii_lowercase|to_ascii_uppercase|to_lowercase|to_uppercase)\s*\(/gu;
  const adHocSites: TokenCaseComparisonSite[] = [];
  const namedExemptSites: NamedTokenCaseOperationSite[] = [];
  const parserSources = trackedParserProductionSources();
  assert.ok(
    parserSources.includes("rust/crates/omena-parser/src/lex.rs"),
    "token case census must include the production lexer",
  );
  for (const relativePath of parserSources) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectTokenCaseComparison && relativePath === "rust/crates/omena-parser/src/facts/mod.rs") {
      source = `fn injected_case_compare(token: Token<'_>) { let alias = token.text; let _ = alias.eq_ignore_ascii_case("x"); }\n${source}`;
    }
    if (injectLexerCaseComparison && relativePath === "rust/crates/omena-parser/src/lex.rs") {
      source = `fn injected_lexer_case_compare(text: &str) { let _ = text.to_ascii_uppercase(); }\n${source}`;
    }
    if (
      injectNamedTokenCaseExemptionDrift &&
      relativePath === "rust/crates/omena-parser/src/extension.rs"
    ) {
      source = source.replace(
        "let lowered = text.to_ascii_lowercase();",
        "let duplicate = text.to_ascii_lowercase(); let lowered = duplicate.to_ascii_lowercase();",
      );
    }
    const scannable = maskCommentsAndTestTail(source);
    for (const match of scannable.matchAll(caseOperation)) {
      const line = lineNumberAt(source, match.index);
      const operation = match[1] as TokenCaseOperation;
      const functionName = enclosingFunctionName(scannable, match.index);
      const site = {
        path: relativePath,
        line,
        function: functionName,
        operation,
        evidence: source.split(/\r?\n/u)[line - 1]?.trim().replace(/\s+/gu, " ") ?? "",
      } satisfies TokenCaseComparisonSite;
      const rule = namedTokenCaseOperationRules.find(
        (candidate) =>
          candidate.path === site.path &&
          candidate.function === site.function &&
          candidate.operation === site.operation &&
          candidate.evidence === site.evidence,
      );
      if (rule) namedExemptSites.push({ ...site, reason: rule.reason });
      else adHocSites.push(site);
    }
  }
  for (const rule of namedTokenCaseOperationRules) {
    const matches = namedExemptSites.filter(
      (site) =>
        site.path === rule.path &&
        site.function === rule.function &&
        site.operation === rule.operation &&
        site.evidence === rule.evidence,
    );
    assert.equal(
      matches.length,
      1,
      `named token-case exemption must resolve exactly once: ${rule.path}#${rule.function}.${rule.operation}`,
    );
  }
  const orderSites = <T extends TokenCaseComparisonSite>(values: readonly T[]): T[] =>
    [...values].toSorted(
      (left, right) =>
        left.path.localeCompare(right.path) ||
        left.line - right.line ||
        left.operation.localeCompare(right.operation),
    );
  return {
    adHocSites: orderSites(adHocSites),
    namedExemptSites: orderSites(namedExemptSites),
  };
}

function enclosingFunctionName(source: string, offset: number): string {
  let functionName = "<module>";
  for (const match of source.slice(0, offset).matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
    functionName = match[1];
  }
  return functionName;
}

function scanRawSyntaxSites(): RawScanSite[] {
  const files = trackedRawSyntaxSources();
  const found: RawScanSite[] = [];

  for (const relativePath of files) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectRawScan && relativePath === "rust/crates/omena-parser/src/facts/mod.rs") {
      source = `fn injected_raw_scan(source: &str) { let _ = source.find('{'); }\n${source}`;
    }
    const scannable = maskCommentsAndTestTail(source);
    const occupied = new Set<string>();

    for (const pattern of patterns) {
      pattern.expression.lastIndex = 0;
      for (const match of scannable.matchAll(pattern.expression)) {
        if (pattern.accept && !pattern.accept(match)) continue;
        const start = match.index;
        const key = `${start}:${match[0].length}`;
        if (occupied.has(key)) continue;
        occupied.add(key);
        const line = lineNumberAt(source, start);
        const classification = classify(relativePath);
        found.push({
          path: relativePath,
          line,
          idiom: pattern.id,
          family: classification.family,
          disposition: classification.disposition,
          evidence: source.split(/\r?\n/u)[line - 1]?.trim().replace(/\s+/gu, " ") ?? "",
          ...(classification.reason ? { reason: classification.reason } : {}),
        });
      }
    }
  }

  const byKey = new Map<string, RawScanSite>();
  for (const site of found) {
    const key = `${site.path}:${site.line}:${site.idiom}`;
    const previous = byKey.get(key);
    if (previous) {
      assert.equal(previous.family, site.family, `raw scan family mismatch at ${key}`);
      assert.equal(
        previous.disposition,
        site.disposition,
        `raw scan disposition mismatch at ${key}`,
      );
      continue;
    }
    byKey.set(key, site);
  }
  const sites = [...byKey.values()].toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.idiom.localeCompare(right.idiom) ||
      left.evidence.localeCompare(right.evidence),
  );
  const keys = sites.map((site) => `${site.path}:${site.line}:${site.idiom}`);
  assert.equal(new Set(keys).size, keys.length, "raw scan site keys must be unique");
  for (const site of sites) {
    assert.ok(site.path.length > 0, "raw scan site path");
    assert.ok(site.line > 0, `raw scan line for ${site.path}`);
    assert.ok(site.evidence.length > 0, `raw scan evidence for ${site.path}:${site.line}`);
    if (site.disposition === "named-exempt") {
      assert.ok(site.reason?.trim(), `named exemption lacks a reason: ${site.path}:${site.line}`);
    }
  }
  return sites;
}

function trackedRustSources(pathspec: string): string[] {
  const result = spawnSync("git", ["ls-files", pathspec], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-files failed: ${result.stderr.trim()}`);
  return result.stdout
    .split(/\r?\n/u)
    .filter((sourcePath) => sourcePath.endsWith(".rs"))
    .filter((sourcePath) => sourcePath.includes("/src/"))
    .filter((sourcePath) => !sourcePath.includes("/tests/"))
    .filter((sourcePath) => !sourcePath.endsWith("/tests.rs"))
    .filter((sourcePath) => !sourcePath.includes("/src/bin/"))
    .filter((sourcePath) => !sourcePath.endsWith("_generated.rs"))
    .toSorted();
}

function trackedRawSyntaxSources(): string[] {
  return trackedRustSources("rust/crates")
    .filter((sourcePath) => {
      const crateName = sourcePath.split("/")[2];
      return engineCrates.includes(crateName);
    })
    .filter((sourcePath) => !excludedPaths.some((excluded) => sourcePath.startsWith(excluded)))
    .toSorted();
}

function trackedParserProductionSources(): string[] {
  return trackedRustSources("rust/crates/omena-parser/src")
    .filter(
      (sourcePath) =>
        sourcePath !== "rust/crates/omena-parser/src/facts/product_facts_authority_tests.rs",
    )
    .toSorted();
}

function classify(relativePath: string): {
  readonly family: string;
  readonly disposition: SiteDisposition;
  readonly reason?: string;
} {
  if (relativePath === "rust/crates/omena-parser/src/facts/mod.rs") {
    return { family: "product-facts-gates", disposition: "migration-target" };
  }
  if (relativePath === "rust/crates/omena-parser/src/public_product.rs") {
    return { family: "product-summary-blocks", disposition: "migration-target" };
  }
  if (relativePath === "rust/crates/omena-transform-cst/src/transform_ir.rs") {
    return { family: "transform-ir-ownership", disposition: "migration-target" };
  }
  if (relativePath === "rust/crates/omena-transform-passes/src/runtime/semantic_preservation.rs") {
    return { family: "semantic-preservation-observer", disposition: "migration-target" };
  }

  const namedFamilies: readonly {
    readonly prefix: string;
    readonly family: string;
    readonly reason: string;
  }[] = [
    {
      prefix: "rust/crates/omena-parser/",
      family: "parser-owned-syntax",
      reason:
        "Parser-owned token and CST construction is the syntax authority, not a parallel consumer.",
    },
    {
      prefix: "rust/crates/omena-transform-passes/src/domains/",
      family: "transform-domain",
      reason:
        "Transform-domain raw scans remain visible for a separately adjudicated consumer port.",
    },
    {
      prefix: "rust/crates/omena-transform-passes/src/helpers/",
      family: "transform-helper",
      reason:
        "Shared transform helpers remain visible until their callers consume typed CST spans.",
    },
    {
      prefix: "rust/crates/omena-transform-passes/",
      family: "transform-runtime",
      reason:
        "Transform runtime scanning outside the preservation observer remains a named follow-up.",
    },
    {
      prefix: "rust/crates/omena-query/",
      family: "query-surface",
      reason:
        "Query-layer source editing and diagnostics are outside the four authority migration families.",
    },
    {
      prefix: "rust/crates/omena-scss-eval/",
      family: "scss-evaluator",
      reason:
        "Dialect evaluation scanners remain tracked as evaluator primitives outside this consumer port.",
    },
    {
      prefix: "rust/crates/omena-sif/",
      family: "module-interface",
      reason: "Module-interface extraction remains a named consumer follow-up.",
    },
    {
      prefix: "rust/crates/omena-cascade/",
      family: "cascade-analysis",
      reason: "Cascade value and selector scanning remains owned by the cascade analysis track.",
    },
    {
      prefix: "rust/crates/omena-semantic/",
      family: "semantic-model",
      reason: "Semantic-model source scanning remains a named consumer follow-up.",
    },
  ];
  const match = namedFamilies.find(({ prefix }) => relativePath.startsWith(prefix));
  return {
    family: match?.family ?? "engine-support",
    disposition: "named-exempt",
    reason:
      match?.reason ??
      "This engine source is outside the four current migration families and remains count-frozen.",
  };
}

function maskCommentsAndTestTail(source: string): string {
  const chars = [...source];
  let inBlockComment = 0;
  let inLineComment = false;
  let inString = false;
  let stringQuote = "";
  let escaped = false;

  for (let index = 0; index < chars.length; index += 1) {
    const char = chars[index];
    const next = chars[index + 1] ?? "";
    if (inLineComment) {
      if (char === "\n") inLineComment = false;
      else chars[index] = " ";
      continue;
    }
    if (inBlockComment > 0) {
      if (char === "/" && next === "*") {
        chars[index] = chars[index + 1] = " ";
        inBlockComment += 1;
        index += 1;
      } else if (char === "*" && next === "/") {
        chars[index] = chars[index + 1] = " ";
        inBlockComment -= 1;
        index += 1;
      } else if (char !== "\n") {
        chars[index] = " ";
      }
      continue;
    }
    if (inString) {
      if (escaped) escaped = false;
      else if (char === "\\") escaped = true;
      else if (char === stringQuote) inString = false;
      continue;
    }
    if (char === "/" && next === "/") {
      chars[index] = chars[index + 1] = " ";
      inLineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      chars[index] = chars[index + 1] = " ";
      inBlockComment = 1;
      index += 1;
      continue;
    }
    if (char === '"') {
      inString = true;
      stringQuote = char;
      continue;
    }
    if (char === "'" && chars[index + 2] === "'") {
      index += 2;
    }
  }

  let masked = chars.join("");
  const testModule = masked.match(/#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z0-9_]+\s*\{/u);
  if (testModule?.index !== undefined) {
    masked = `${masked.slice(0, testModule.index)}${masked
      .slice(testModule.index)
      .replace(/[^\n]/gu, " ")}`;
  }
  return masked;
}

function lineNumberAt(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function stableSiteKey(site: RawScanSite): string {
  return `${site.path}\u0000${site.idiom}\u0000${site.family}\u0000${site.disposition}\u0000${site.evidence}`;
}
