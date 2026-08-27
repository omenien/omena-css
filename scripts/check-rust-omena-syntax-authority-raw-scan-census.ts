import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { assertRustCfgTestMaskContract, maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

assertRustCfgTestMaskContract();

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
  | "is_ascii_lowercase"
  | "is_ascii_uppercase"
  | "is_lowercase"
  | "is_uppercase"
  | "make_ascii_lowercase"
  | "make_ascii_uppercase"
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
  readonly classSelectorScanner: {
    readonly policy: "decrease-only";
    readonly baselineSiteCount: number;
    readonly currentSiteCount: number;
    readonly sites: readonly RawScanSite[];
    readonly siteDigest: string;
  };
  readonly selectorAuthority: {
    readonly policy: "single-cst-projected-authority";
    readonly ownerPath: "rust/crates/omena-syntax/src/selector.rs";
    readonly typeName: "CanonicalSelectorAst";
    readonly authorityTypeCount: 1;
    readonly parserCstProducerCallCount: 1;
    readonly reportProjectionCallCount: 1;
    readonly reportAuthorityBindingCallCount: 1;
    readonly reportStructLiteralBypassCount: 0;
    readonly reportFieldVisibility: "private";
    readonly rustAuthoritySourceScope: "all-tracked-rust";
    readonly rawNestingSourceRoots: readonly ["rust", "packages"];
    readonly rawNestingReplaceBlindSpots: readonly [
      "template-literal-ampersand-argument",
      "one-hop-replacement-argument-indirection",
      "regular-expression-ampersand-replacement",
      "split-then-join-ampersand-replacement",
    ];
    readonly rawNestingReplaceSiteCount: 0;
  };
  readonly moduleInterfaceLessScanner: {
    readonly policy: "single-named-seam";
    readonly path: "rust/crates/omena-sif/src/generator.rs";
    readonly splitter: "split_legacy_less_statements";
    readonly seam: "scan_static_less_export_statements";
    readonly directCallCount: number;
    readonly directCallFunctions: readonly string[];
    readonly knownLimitations: readonly [
      "comment-delimiters-not-isolated",
      "interpolation-braces-affect-segmentation",
      "newline-only-statements-not-segmented",
    ];
    readonly reentryCondition: "parser-facts-expose-less-mixins-and-detached-ruleset-members";
  };
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
const acceptNewlyVisibleSites = process.argv.includes("--accept-newly-visible-sites");
assert.ok(
  !acceptNewlyVisibleSites || writeMode,
  "--accept-newly-visible-sites is valid only while writing a reviewed baseline",
);
const injectRawScan = process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_SCAN === "1";
const injectTokenCaseComparison =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_COMPARE === "1";
const injectLexerCaseComparison =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LEXER_CASE_COMPARE === "1";
const injectNamedTokenCaseExemptionDrift =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_EXEMPTION_DRIFT === "1";
const injectModuleInterfaceLessScannerCall =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LESS_SCANNER_CALL === "1";
const injectClassSelectorScanner =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_CLASS_SCANNER === "1";
const injectSecondSelectorAuthority =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SECOND_SELECTOR_AUTHORITY === "1";
const injectSelectorReportStructLiteral =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SELECTOR_REPORT_STRUCT_LITERAL === "1";
const injectSelectorReportAuthoritySeverance =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_SELECTOR_REPORT_AUTHORITY_SEVERANCE === "1";
const injectRawNestingReplace =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACE === "1";
const injectRawNestingReplaceDoubleQuote =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACE_DOUBLE_QUOTE === "1";
const injectRawNestingReplacen =
  process.env.OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_NESTING_REPLACEN === "1";

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

const classSelectorScannerPatterns: readonly IdiomPattern[] = [
  {
    id: "class-dot-byte-walk",
    expression: /\bbytes\s*\[[^\]\n]+\]\s*(?:==|!=)\s*b'\.'/gu,
  },
  {
    id: "char-identifier-reader",
    expression: /\bfn\s+read_identifier\s*\(\s*chars\s*:\s*&\s*\[\s*char\s*\]/gu,
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
] as const;

const existing = readExistingCensus();
const sites = scanRawSyntaxSites();
const classSelectorScannerSites = scanClassSelectorScannerSites();
const tokenCaseOperations = scanTokenCaseOperations();
const moduleInterfaceLessScanner = scanModuleInterfaceLessScanner();
const selectorAuthority = scanSelectorAuthority();
const tokenCaseComparisonSites = tokenCaseOperations.adHocSites;
const currentNamedExemptSiteCount = sites.filter(
  (site) => site.disposition === "named-exempt",
).length;
const baselineSiteCount = acceptNewlyVisibleSites
  ? sites.length
  : (existing?.baselineSiteCount ?? sites.length);
const baselineNamedExemptSiteCount = acceptNewlyVisibleSites
  ? currentNamedExemptSiteCount
  : (existing?.baselineNamedExemptSiteCount ?? currentNamedExemptSiteCount);
const baselineClassSelectorScannerSiteCount = acceptNewlyVisibleSites
  ? classSelectorScannerSites.length
  : (existing?.classSelectorScanner?.baselineSiteCount ?? classSelectorScannerSites.length);

assert.ok(sites.length > 0, "raw syntax scan census must be non-vacuous");
assert.ok(
  sites.some((site) => site.disposition === "named-exempt"),
  "raw syntax scan census must include named exemptions",
);
assert.equal(
  selectorAuthority.authorityTypeCount,
  1,
  "selector canonicalization must have exactly one CanonicalSelectorAst authority",
);
assert.equal(
  selectorAuthority.parserCstProducerCallCount,
  1,
  "the parser must have exactly one CanonicalSelectorAst CST construction call",
);
assert.equal(
  selectorAuthority.reportProjectionCallCount,
  1,
  "SelectorCanonicalIdentityV0 must remain one projection of the canonical authority",
);
assert.equal(
  selectorAuthority.reportAuthorityBindingCallCount,
  1,
  "the selector report must bind every projected name through the CST authority",
);
assert.equal(
  selectorAuthority.reportStructLiteralBypassCount,
  0,
  "SelectorCanonicalIdentityV0 construction must not bypass its sealed constructor",
);
assert.equal(
  selectorAuthority.reportFieldVisibility,
  "private",
  "SelectorCanonicalIdentityV0 fields must remain private",
);
assert.equal(
  selectorAuthority.rawNestingReplaceSiteCount,
  0,
  "nesting expansion must not retain raw replace('&', ...) sites",
);
assert.ok(
  sites.length <= baselineSiteCount,
  `raw syntax scan count increased: baseline=${baselineSiteCount} current=${sites.length}`,
);
assert.ok(
  currentNamedExemptSiteCount <= baselineNamedExemptSiteCount,
  `named-exempt raw syntax scan count increased: baseline=${baselineNamedExemptSiteCount} current=${currentNamedExemptSiteCount}`,
);
assert.ok(
  classSelectorScannerSites.length > 0,
  "class selector scanner census must remain non-vacuous",
);
assert.ok(
  classSelectorScannerSites.length <= baselineClassSelectorScannerSiteCount,
  `class selector scanner count increased: baseline=${baselineClassSelectorScannerSiteCount} current=${classSelectorScannerSites.length}`,
);
assert.deepEqual(
  tokenCaseComparisonSites,
  [],
  "parser syntax-token case comparisons must route through matches_ignore_ascii_case",
);

if (existing && writeMode && !acceptNewlyVisibleSites) {
  const previousKeys = new Set(existing.sites.map(stableSiteKey));
  const addedSites = sites.filter((site) => !previousKeys.has(stableSiteKey(site)));
  assert.deepEqual(
    addedSites,
    [],
    "the decrease-only census cannot adopt new raw syntax scan sites during regeneration",
  );
  if (existing.classSelectorScanner !== undefined) {
    const previousClassScannerKeys = new Set(
      existing.classSelectorScanner.sites.map(stableSiteKey),
    );
    const addedClassScannerSites = classSelectorScannerSites.filter(
      (site) => !previousClassScannerKeys.has(stableSiteKey(site)),
    );
    assert.deepEqual(
      addedClassScannerSites,
      [],
      "the decrease-only census cannot adopt new class selector scanner sites during regeneration",
    );
  }
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
  classSelectorScanner: {
    policy: "decrease-only",
    baselineSiteCount: baselineClassSelectorScannerSiteCount,
    currentSiteCount: classSelectorScannerSites.length,
    sites: classSelectorScannerSites,
    siteDigest: `sha256:${createHash("sha256")
      .update(JSON.stringify(classSelectorScannerSites))
      .digest("hex")}`,
  },
  selectorAuthority,
  moduleInterfaceLessScanner,
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
      !injectNamedTokenCaseExemptionDrift &&
      !injectModuleInterfaceLessScannerCall &&
      !injectClassSelectorScanner &&
      !injectSecondSelectorAuthority &&
      !injectSelectorReportStructLiteral &&
      !injectSelectorReportAuthoritySeverance &&
      !injectRawNestingReplace &&
      !injectRawNestingReplaceDoubleQuote &&
      !injectRawNestingReplacen,
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
      classSelectorScannerSiteCount: census.classSelectorScanner.currentSiteCount,
      classSelectorScannerSiteDigest: census.classSelectorScanner.siteDigest,
      selectorAuthorityTypeCount: census.selectorAuthority.authorityTypeCount,
      selectorParserCstProducerCallCount: census.selectorAuthority.parserCstProducerCallCount,
      selectorReportProjectionCallCount: census.selectorAuthority.reportProjectionCallCount,
      selectorReportAuthorityBindingCallCount:
        census.selectorAuthority.reportAuthorityBindingCallCount,
      selectorReportStructLiteralBypassCount:
        census.selectorAuthority.reportStructLiteralBypassCount,
      selectorRustAuthoritySourceScope: census.selectorAuthority.rustAuthoritySourceScope,
      rawNestingReplaceBlindSpots: census.selectorAuthority.rawNestingReplaceBlindSpots,
      rawNestingReplaceSiteCount: census.selectorAuthority.rawNestingReplaceSiteCount,
      moduleInterfaceLessScannerCallCount: census.moduleInterfaceLessScanner.directCallCount,
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
  assert.ok(parsed.tokenCaseComparison, "committed token-case census is required");
  assert.equal(parsed.tokenCaseComparison.policy, "helper-only", "token case policy");
  assert.equal(parsed.tokenCaseComparison.helper, "matches_ignore_ascii_case", "token case helper");
  assert.equal(
    parsed.tokenCaseComparison.adHocSiteCount,
    parsed.tokenCaseComparison.sites.length,
    "token case site count",
  );
  assert.ok(
    parsed.tokenCaseComparison.namedExemptSites,
    "committed named-exempt token-case sites are required",
  );
  assert.equal(
    parsed.tokenCaseComparison.namedExemptSiteCount,
    parsed.tokenCaseComparison.namedExemptSites.length,
    "named-exempt token case operation count",
  );
  assert.ok(parsed.moduleInterfaceLessScanner, "committed Less scanner census is required");
  assert.equal(
    parsed.moduleInterfaceLessScanner.directCallCount,
    1,
    "committed module-interface Less scanner call count",
  );
  assert.deepEqual(
    parsed.moduleInterfaceLessScanner.directCallFunctions,
    ["scan_static_less_export_statements"],
    "committed module-interface Less scanner seam",
  );
  assert.ok(parsed.classSelectorScanner, "committed class-selector census is required");
  assert.equal(
    parsed.classSelectorScanner.currentSiteCount,
    parsed.classSelectorScanner.sites.length,
    "committed class selector scanner site count",
  );
  assert.equal(
    parsed.classSelectorScanner.siteDigest,
    `sha256:${createHash("sha256")
      .update(JSON.stringify(parsed.classSelectorScanner.sites))
      .digest("hex")}`,
    "committed class selector scanner site digest",
  );
  assert.ok(parsed.selectorAuthority, "committed selector-authority census is required");
  assert.equal(parsed.selectorAuthority.authorityTypeCount, 1, "committed selector authority");
  assert.equal(
    parsed.selectorAuthority.reportProjectionCallCount,
    1,
    "committed selector report projection",
  );
  assert.equal(
    parsed.selectorAuthority.reportAuthorityBindingCallCount,
    1,
    "committed selector report authority binding",
  );
  assert.equal(
    parsed.selectorAuthority.reportStructLiteralBypassCount,
    0,
    "committed selector report struct-literal bypass count",
  );
  assert.equal(
    parsed.selectorAuthority.reportFieldVisibility,
    "private",
    "committed selector report field visibility",
  );
  assert.ok(
    parsed.selectorAuthority.rawNestingReplaceBlindSpots,
    "committed raw nesting replacement blind spots are required",
  );
  assert.equal(
    parsed.selectorAuthority.rawNestingReplaceSiteCount,
    0,
    "committed raw nesting replacement count",
  );
  return parsed;
}

function scanSelectorAuthority(): RawScanCensus["selectorAuthority"] {
  const ownerPath = "rust/crates/omena-syntax/src/selector.rs" as const;
  const rustSources = trackedRustSources(".").map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectSelectorReportAuthoritySeverance) {
    const report = rustSources.find(
      ({ relativePath }) => relativePath === "rust/crates/omena-semantic/src/selector_identity.rs",
    );
    assert.ok(report, "selector report source must be in the tracked Rust closure");
    report.source = report.source.replace(
      ".canonical_class_key_for_source_span(",
      ".canonical_class_key_for_source_span_severed(",
    );
  }
  if (injectSecondSelectorAuthority) {
    rustSources.push({
      relativePath: "rust/crates/omena-query/src/injected_selector_authority.rs",
      source: "pub struct CanonicalSelectorAst;\n",
    });
  }
  const authorityTypeCount = rustSources.reduce(
    (count, { source }) =>
      count + [...source.matchAll(/\bpub\s+struct\s+CanonicalSelectorAst\b/gu)].length,
    0,
  );
  const parserCstProducerCallCount = rustSources.reduce(
    (count, { source }) =>
      count + [...source.matchAll(/\bCanonicalSelectorAst::from_cst\s*\(/gu)].length,
    0,
  );
  const reportProjectionCallCount = rustSources.reduce(
    (count, { source }) =>
      count +
      [...source.matchAll(/\bSelectorCanonicalIdentityV0::from_canonical_key\s*\(/gu)].length,
    0,
  );
  const reportAuthorityBindingCallCount = rustSources.reduce(
    (count, { source }) =>
      count + [...source.matchAll(/\.canonical_class_key_for_source_span\s*\(/gu)].length,
    0,
  );
  if (injectSelectorReportStructLiteral) {
    rustSources.push({
      relativePath: "rust/crates/omena-query/src/injected_selector_report.rs",
      source:
        'fn injected() { let _ = SelectorCanonicalIdentityV0 { canonical_id: String::new(), local_name: String::new(), identity_kind: "x", rewrite_safety: "safe", blockers: Vec::new() }; }\n',
    });
  }
  const reportStructLiteralBypassCount = rustSources.reduce(
    (count, { source }) =>
      count +
      [...source.matchAll(/\bSelectorCanonicalIdentityV0\s*\{/gu)].filter((match) => {
        const prefix = source.slice(Math.max(0, match.index - 24), match.index);
        return !/\b(?:pub\s+)?(?:struct|impl)\s*$/u.test(prefix);
      }).length,
    0,
  );
  const reportSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-semantic/src/selector_identity.rs"),
    "utf8",
  );
  const reportBody = reportSource.match(
    /\bpub\s+struct\s+SelectorCanonicalIdentityV0\s*\{(?<body>[\s\S]*?)\n\}/u,
  )?.groups?.body;
  assert.ok(reportBody, "SelectorCanonicalIdentityV0 definition must remain discoverable");
  const reportFieldVisibility = /(^|\n)\s*pub(?:\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\s*:/u.test(
    reportBody,
  )
    ? "public"
    : "private";

  const rawNestingSources = trackedProductSources(["rust", "packages"]);
  if (injectRawNestingReplace) {
    rawNestingSources.push({
      relativePath: "rust/crates/omena-query/src/injected_raw_replace.rs",
      source: "fn injected(source: &str, parent: &str) { let _ = source.replace('&', parent); }\n",
    });
  }
  if (injectRawNestingReplaceDoubleQuote) {
    rawNestingSources.push({
      relativePath: "packages/omena-query/src/injected-raw-replace.ts",
      source:
        'export const injected = (source: string, parent: string) => source.replace("&", parent);\n',
    });
  }
  if (injectRawNestingReplacen) {
    rawNestingSources.push({
      relativePath: "rust/crates/omena-query/src/injected_raw_replacen.rs",
      source:
        'fn injected(source: &str, parent: &str) { let _ = source.replacen("&", parent, 1); }\n',
    });
  }
  const rawNestingReplaceSiteCount = rawNestingSources.reduce(
    (count, { source }) =>
      count + [...source.matchAll(/\.(?:replace|replacen)\s*\(\s*(["'])&\1\s*,/gu)].length,
    0,
  );

  return {
    policy: "single-cst-projected-authority",
    ownerPath,
    typeName: "CanonicalSelectorAst",
    authorityTypeCount: authorityTypeCount as 1,
    parserCstProducerCallCount: parserCstProducerCallCount as 1,
    reportProjectionCallCount: reportProjectionCallCount as 1,
    reportAuthorityBindingCallCount: reportAuthorityBindingCallCount as 1,
    reportStructLiteralBypassCount: reportStructLiteralBypassCount as 0,
    reportFieldVisibility: reportFieldVisibility as "private",
    rustAuthoritySourceScope: "all-tracked-rust",
    rawNestingSourceRoots: ["rust", "packages"],
    rawNestingReplaceBlindSpots: [
      "template-literal-ampersand-argument",
      "one-hop-replacement-argument-indirection",
      "regular-expression-ampersand-replacement",
      "split-then-join-ampersand-replacement",
    ],
    rawNestingReplaceSiteCount: rawNestingReplaceSiteCount as 0,
  };
}

function scanModuleInterfaceLessScanner(): RawScanCensus["moduleInterfaceLessScanner"] {
  const relativePath = "rust/crates/omena-sif/src/generator.rs" as const;
  let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (injectModuleInterfaceLessScannerCall) {
    source =
      "fn injected_less_scanner_call(source: &str) { let _ = split_legacy_less_statements(source); }\n" +
      source;
  }
  const scannable = maskCommentsAndTestItems(source);
  const directCallFunctions = [...scannable.matchAll(/\bsplit_legacy_less_statements\s*\(/gu)]
    .filter((match) => {
      const line = source.split(/\r?\n/u)[lineNumberAt(source, match.index) - 1]?.trim() ?? "";
      return !line.startsWith("fn split_legacy_less_statements");
    })
    .map((match) => enclosingFunctionName(scannable, match.index));

  assert.deepEqual(
    directCallFunctions,
    ["scan_static_less_export_statements"],
    "legacy Less statement splitting must remain behind one named module-interface seam",
  );
  assert.ok(
    !scannable.includes("parse_static_sass_exports_scanner_oracle_v1"),
    "the product scanner must not retain a Sass export caller",
  );
  for (const phrase of [
    "does not isolate comment delimiters",
    "interpolation braces as block braces",
    "cannot segment newline-only",
    "parser facts expose Less mixin signatures",
    "detached-ruleset members",
  ]) {
    assert.ok(source.includes(phrase), `Less scanner seam must document: ${phrase}`);
  }

  return {
    policy: "single-named-seam",
    path: relativePath,
    splitter: "split_legacy_less_statements",
    seam: "scan_static_less_export_statements",
    directCallCount: directCallFunctions.length,
    directCallFunctions,
    knownLimitations: [
      "comment-delimiters-not-isolated",
      "interpolation-braces-affect-segmentation",
      "newline-only-statements-not-segmented",
    ],
    reentryCondition: "parser-facts-expose-less-mixins-and-detached-ruleset-members",
  };
}

function scanTokenCaseOperations(): {
  readonly adHocSites: readonly TokenCaseComparisonSite[];
  readonly namedExemptSites: readonly NamedTokenCaseOperationSite[];
} {
  const caseOperation =
    /(?<![A-Za-z0-9_])(eq_ignore_ascii_case|is_ascii_lowercase|is_ascii_uppercase|is_lowercase|is_uppercase|make_ascii_lowercase|make_ascii_uppercase|to_ascii_lowercase|to_ascii_uppercase|to_lowercase|to_uppercase)(?![A-Za-z0-9_])/gu;
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
      source = `fn injected_lexer_case_compare(text: &str) { let _ = text.chars().flat_map(char::to_uppercase).count(); }\n${source}`;
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
    const scannable = maskCommentsAndTestItems(source);
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
    const scannable = maskCommentsAndTestItems(source);
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

function scanClassSelectorScannerSites(): RawScanSite[] {
  const found: RawScanSite[] = [];
  for (const relativePath of trackedRawSyntaxSources()) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (
      injectClassSelectorScanner &&
      relativePath === "rust/crates/omena-query/src/style/cascade_checker/runtime_state.rs"
    ) {
      source =
        "fn injected_class_scanner(bytes: &[u8], index: usize) -> bool { bytes[index] == b'.' }\n" +
        source;
    }
    const scannable = maskCommentsAndTestItems(source);
    for (const pattern of classSelectorScannerPatterns) {
      pattern.expression.lastIndex = 0;
      for (const match of scannable.matchAll(pattern.expression)) {
        const line = lineNumberAt(source, match.index);
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
  return found.toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.idiom.localeCompare(right.idiom),
  );
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

function trackedProductSources(
  pathspecs: readonly string[],
): { relativePath: string; source: string }[] {
  const result = spawnSync("git", ["ls-files", "--", ...pathspecs], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-files failed: ${result.stderr.trim()}`);
  return result.stdout
    .split(/\r?\n/u)
    .filter((sourcePath) => /\.(?:rs|ts|tsx|js|mjs|cjs)$/u.test(sourcePath))
    .toSorted()
    .map((relativePath) => ({
      relativePath,
      source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
    }));
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

function maskCommentsAndTestItems(source: string): string {
  // Keep UTF-16 offsets aligned with every regex match and with the original
  // source. Code-point spreading shortens the buffer before non-BMP text and
  // can move a cfg(test) mask onto adjacent production bytes.
  const chars = source.split("");
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

  return maskRustCfgTestItems(chars.join(""));
}

function lineNumberAt(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function stableSiteKey(site: RawScanSite): string {
  return `${site.path}\u0000${site.idiom}\u0000${site.family}\u0000${site.disposition}\u0000${site.evidence}`;
}
