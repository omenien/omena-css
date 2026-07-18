import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

type PrecisionVariant = "Exact" | "Conservative" | "Heuristic" | "Unknown";
type PrecisionSiteDisposition =
  | "derivationRule"
  | "productionOverride"
  | "threshold"
  | "consumer"
  | "test"
  | "documentation"
  | "evidenceLiteral";
type LexicalContext = "code" | "stringLiteral" | "comment";

interface PrecisionVariantCounts {
  readonly Exact: number;
  readonly Conservative: number;
  readonly Heuristic: number;
  readonly Unknown: number;
}

interface ObservedPrecisionSite {
  readonly sourcePath: string;
  readonly owner: string;
  readonly lexicalContext: LexicalContext;
  readonly testOnly: boolean;
  readonly variantCounts: PrecisionVariantCounts;
  readonly occurrenceCount: number;
}

interface PrecisionSiteDispositionRecord extends ObservedPrecisionSite {
  readonly disposition: PrecisionSiteDisposition;
  readonly authority: string;
}

interface FactPrecisionCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.fact-precision-production-census";
  readonly rawReferenceCount: number;
  readonly productionSiteCount: number;
  readonly productionOverrideCount: number;
  readonly classifiedSites: readonly PrecisionSiteDispositionRecord[];
  readonly unclassifiedSites: readonly ObservedPrecisionSite[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-fact-precision-production-census.json");
const writeMode = process.argv.includes("--write");
const dumpObserved = process.argv.includes("--dump-observed");
const precisionPattern = /FactPrecision::(Exact|Conservative|Heuristic|Unknown)/gu;

const expectedSites: readonly PrecisionSiteDispositionRecord[] = [
  site(
    "rust/crates/omena-abstract-value/src/domain.rs",
    "fn fact_precision_from_class_value_with_witness",
    { Exact: 2, Conservative: 2, Heuristic: 1, Unknown: 1 },
    "derivationRule",
    "the exhaustive class-value lattice adapter derives rank from representation and bound witness",
  ),
  site(
    "rust/crates/omena-bridge/src/style_intelligence.rs",
    "const CSS_MODULES_METADATA",
    { Exact: 1 },
    "productionOverride",
    "the built-in CSS Modules provider observes source-bound imports and class references directly",
  ),
  site(
    "rust/crates/omena-bridge/src/style_intelligence.rs",
    "const CVA_RECIPE_METADATA",
    { Exact: 1 },
    "productionOverride",
    "the built-in CVA provider exposes only statically extracted recipe facts",
  ),
  site(
    "rust/crates/omena-bridge/src/style_intelligence.rs",
    "const UTILITY_DOMAIN_METADATA",
    { Unknown: 1 },
    "productionOverride",
    "the utility-domain provider is not precision-backed until its external configuration is resolved",
  ),
  site(
    "rust/crates/omena-bridge/src/style_intelligence.rs",
    "const VANILLA_EXTRACT_METADATA",
    { Exact: 1 },
    "productionOverride",
    "the built-in vanilla-extract provider exposes only statically extracted recipe facts",
  ),
  site(
    "rust/crates/omena-bridge/src/style_intelligence.rs",
    "const VUE_STYLE_MODULE_METADATA",
    { Exact: 1 },
    "productionOverride",
    "the built-in Vue provider exposes source-bound useCssModule references",
  ),
  site(
    "rust/crates/omena-bridge/src/utility_intelligence/mod.rs",
    "fn default",
    { Unknown: 1 },
    "productionOverride",
    "an unloaded utility configuration has no validated class-universe evidence",
  ),
  site(
    "rust/crates/omena-checker/src/fix_safety.rs",
    "fn compute_fix_safety",
    { Exact: 1, Conservative: 1, Heuristic: 1, Unknown: 1 },
    "consumer",
    "fix safety exhaustively consumes every precision rank without producing a new fact rank",
  ),
  site(
    "rust/crates/omena-cli/src/migrate/mod.rs",
    "fn conservative_workspace_safety",
    { Conservative: 1 },
    "productionOverride",
    "the migration policy deliberately supplies a sound but non-exact workspace reference",
  ),
  site(
    "rust/crates/omena-cli/src/migrate/mod.rs",
    "fn exact_workspace_safety",
    { Exact: 1 },
    "productionOverride",
    "the migration policy requires syntax, local semantics, and closed-world evidence together",
  ),
  site(
    "rust/crates/omena-cli/src/migrate/mod.rs",
    "fn manual_review_safety",
    { Heuristic: 1 },
    "productionOverride",
    "the migration policy marks missing local-semantic evidence for manual review",
  ),
  site(
    "rust/crates/omena-query-core/src/lib.rs",
    "const OMENA_QUERY_ANALYSIS_FACT_PRECISION_BY_VALUE_DOMAIN",
    { Exact: 2, Conservative: 2, Heuristic: 1, Unknown: 1 },
    "derivationRule",
    "the closed value-domain adapter maps every declared query analysis domain",
  ),
  site(
    "rust/crates/omena-query-core/src/lib.rs",
    "fn fact_precision_from_analysis_precision",
    { Unknown: 1 },
    "derivationRule",
    "an undeclared query analysis domain fails closed to Unknown",
  ),
  site(
    "rust/crates/omena-query-transform-runner/src/plugins/bundle_host.rs",
    "fn analyze",
    { Exact: 1 },
    "productionOverride",
    "the bundle-host analysis reports a fully observed immutable workspace snapshot",
  ),
  site(
    "rust/crates/omena-query-transform-runner/src/plugins/bundle_host.rs",
    "fn transform",
    { Exact: 1 },
    "productionOverride",
    "the bundle-host transform performs no mutation and reports its complete IR observation",
  ),
  site(
    "rust/crates/omena-query-transform-runner/src/plugins/semantic_observation.rs",
    "fn analyze",
    { Exact: 1 },
    "productionOverride",
    "the semantic-observation analysis reads the complete supplied workspace snapshot",
  ),
  site(
    "rust/crates/omena-query-transform-runner/src/plugins/semantic_observation.rs",
    "fn transform",
    { Exact: 1 },
    "productionOverride",
    "the semantic-observation transform performs no mutation and reports its complete IR observation",
  ),
  site(
    "rust/crates/omena-query/src/style/diagnostics/source_usage.rs",
    "fn summarize_omena_query_css_modules_export_usage",
    { Unknown: 1 },
    "productionOverride",
    "an unresolved import or dynamic source-usage path is explicitly skipped rather than inferred",
  ),
  site(
    "rust/crates/omena-query/src/style/transform.rs",
    "fn closed_world_bound_reachability_precision",
    { Conservative: 2 },
    "derivationRule",
    "the sealed-bundle adapter validates enumeration membership and a content-addressed witness",
  ),
  site(
    "rust/crates/omena-query/src/style/transform.rs",
    "fn closed_world_source_precision_summary",
    { Exact: 1, Conservative: 1, Heuristic: 1, Unknown: 1 },
    "consumer",
    "the source-precision summary exhaustively counts an already-derived rank",
  ),
  site(
    "rust/crates/omena-query/src/style/transform/context.rs",
    "fn derive_omena_query_transform_context_from_engine_input",
    { Unknown: 1 },
    "derivationRule",
    "a projection absent from the precision index fails closed before the bounded ceiling fold",
  ),
  site(
    "rust/crates/omena-transform-passes/src/model.rs",
    "const TRANSFORM_STRUCTURAL_DECISION_POLICIES_V0",
    { Conservative: 4 },
    "threshold",
    "the four reachability-consuming tree-shake passes require a sound over-approximation",
  ),
  site(
    "rust/crates/omena-transform-passes/src/runtime/executor.rs",
    "fn classify_transform_reachability_precision",
    { Exact: 2, Conservative: 1, Heuristic: 1, Unknown: 1 },
    "derivationRule",
    "the runtime derives observed reachability rank from bundle presence, context, and the upstream ceiling",
  ),
];

const observedSites = collectObservedSites(
  rustSourcePaths(path.join(repoRoot, "rust/crates")),
).toSorted(compareSites);

if (dumpObserved) {
  process.stdout.write(`${JSON.stringify(observedSites, null, 2)}\n`);
  process.exit(0);
}

const expectedByKey = new Map(expectedSites.map((site) => [siteKey(site), site]));
const classifiedSites = observedSites.flatMap((observed) => {
  const automatic = automaticDisposition(observed);
  if (automatic !== undefined) {
    return [{ ...observed, ...automatic }];
  }
  const expected = expectedByKey.get(siteKey(observed));
  return expected === undefined ? [] : [expected];
});
const unclassifiedSites = observedSites.filter(
  (site) => automaticDisposition(site) === undefined && !expectedByKey.has(siteKey(site)),
);

for (const observed of observedSites) {
  const expected = expectedByKey.get(siteKey(observed));
  if (expected === undefined) {
    continue;
  }
  assert.deepEqual(
    observed.variantCounts,
    expected.variantCounts,
    `FactPrecision variants changed at ${siteKey(observed)}`,
  );
  assert.equal(
    observed.occurrenceCount,
    expected.occurrenceCount,
    `FactPrecision occurrence count changed at ${siteKey(observed)}`,
  );
}

const injected = collectObservedSites([
  {
    sourcePath: "rust/crates/injected/src/fact.rs",
    source: "fn produce_fact() { let precision = FactPrecision::Exact; }",
  },
]);
assert.deepEqual(
  injected,
  [
    {
      sourcePath: "rust/crates/injected/src/fact.rs",
      owner: "fn produce_fact",
      lexicalContext: "code",
      testOnly: false,
      variantCounts: variantCounts({ Exact: 1 }),
      occurrenceCount: 1,
    },
  ],
  "the predicate must detect a newly introduced precision producer",
);
assert.equal(
  injected.some((site) => expectedByKey.has(siteKey(site))),
  false,
  "an unknown precision producer must not match the closed census",
);

assert.deepEqual(unclassifiedSites, [], "every FactPrecision reference must be classified");
assert.equal(
  observedSites.filter((site) => automaticDisposition(site) === undefined).length,
  expectedSites.length,
  "the product precision site set must remain closed",
);

const productionSites = expectedSites.filter(
  (site) => site.disposition === "derivationRule" || site.disposition === "productionOverride",
);
const productionOverrides = expectedSites.filter(
  (site) => site.disposition === "productionOverride",
);
const census: FactPrecisionCensus = {
  schemaVersion: "0",
  product: "omena.fact-precision-production-census",
  rawReferenceCount: observedSites.reduce((sum, site) => sum + site.occurrenceCount, 0),
  productionSiteCount: productionSites.length,
  productionOverrideCount: productionOverrides.length,
  classifiedSites,
  unclassifiedSites,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "FactPrecision production census is stale",
  );
}

process.stdout.write(
  `Omena FactPrecision census OK: raw=${census.rawReferenceCount} production=${census.productionSiteCount} overrides=${census.productionOverrideCount} unclassified=0\n`,
);

function collectObservedSites(
  sources: readonly (string | { readonly sourcePath: string; readonly source: string })[],
): ObservedPrecisionSite[] {
  const grouped = new Map<
    string,
    {
      sourcePath: string;
      owner: string;
      lexicalContext: LexicalContext;
      testOnly: boolean;
      counts: Record<PrecisionVariant, number>;
    }
  >();
  for (const sourceEntry of sources) {
    const sourcePath =
      typeof sourceEntry === "string"
        ? path.relative(repoRoot, sourceEntry)
        : sourceEntry.sourcePath;
    const source =
      typeof sourceEntry === "string" ? fs.readFileSync(sourceEntry, "utf8") : sourceEntry.source;
    const lexical = analyzeRustLexicalContext(source);
    const testRanges = cfgTestModuleRanges(lexical.ownerSource);
    for (const match of source.matchAll(precisionPattern)) {
      const owner = enclosingOwner(lexical.ownerSource, match.index);
      const lexicalContext = lexical.contextAt(match.index);
      const testOnly =
        sourcePath.endsWith("/tests.rs") ||
        sourcePath.includes("/tests/") ||
        testRanges.some(([start, end]) => match.index >= start && match.index < end);
      const key = `${sourcePath}#${owner}#${lexicalContext}#${testOnly}`;
      const current = grouped.get(key) ?? {
        sourcePath,
        owner,
        lexicalContext,
        testOnly,
        counts: mutableVariantCounts(),
      };
      current.counts[match[1] as PrecisionVariant] += 1;
      grouped.set(key, current);
    }
  }
  return [...grouped.values()].map(({ sourcePath, owner, lexicalContext, testOnly, counts }) => ({
    sourcePath,
    owner,
    lexicalContext,
    testOnly,
    variantCounts: counts,
    occurrenceCount: Object.values(counts).reduce((sum, count) => sum + count, 0),
  }));
}

function cfgTestModuleRanges(source: string): readonly (readonly [number, number])[] {
  const ranges: Array<readonly [number, number]> = [];
  const pattern =
    /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+[a-zA-Z0-9_]+\s*\{/gu;
  for (const match of source.matchAll(pattern)) {
    const open = source.indexOf("{", match.index);
    if (open === -1) {
      continue;
    }
    let depth = 1;
    let cursor = open + 1;
    while (cursor < source.length && depth > 0) {
      if (source[cursor] === "{") {
        depth += 1;
      } else if (source[cursor] === "}") {
        depth -= 1;
      }
      cursor += 1;
    }
    ranges.push([match.index, cursor]);
  }
  return ranges;
}

function enclosingOwner(source: string, offset: number): string {
  const prefix = source.slice(0, offset);
  const declarations = [...prefix.matchAll(/(?<!')\b(fn|const|static)\s+([a-zA-Z0-9_]+)\b/gu)];
  const declaration = declarations.at(-1);
  return declaration === undefined ? "moduleScope" : `${declaration[1]} ${declaration[2]}`;
}

function analyzeRustLexicalContext(source: string): {
  readonly ownerSource: string;
  readonly contextAt: (offset: number) => LexicalContext;
} {
  const ownerSource = source.split("");
  const contexts = new Uint8Array(source.length);
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end === -1 ? source.length : end;
      blankRange(ownerSource, contexts, index, stop, 2);
      index = stop;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith("*/", cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      blankRange(ownerSource, contexts, index, cursor, 2);
      index = cursor;
      continue;
    }
    const rawPrefix = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (rawPrefix !== null) {
      const terminator = `"${rawPrefix[1]}`;
      const bodyStart = index + rawPrefix[0].length;
      const close = source.indexOf(terminator, bodyStart);
      const stop = close === -1 ? source.length : close + terminator.length;
      blankRange(ownerSource, contexts, index, stop, 1);
      index = stop;
      continue;
    }
    const stringPrefixLength = source.startsWith('b"', index) ? 2 : source[index] === '"' ? 1 : 0;
    if (stringPrefixLength > 0) {
      const stop = quotedLiteralEnd(source, index + stringPrefixLength, '"');
      blankRange(ownerSource, contexts, index, stop, 1);
      index = stop;
      continue;
    }
    const charLiteral = source.slice(index).match(/^(?:b)?'(?:\\.|[^\\'\r\n])'/u);
    if (charLiteral !== null) {
      const stop = index + charLiteral[0].length;
      blankRange(ownerSource, contexts, index, stop, 1);
      index = stop;
      continue;
    }
    index += 1;
  }
  return {
    ownerSource: ownerSource.join(""),
    contextAt: (offset) => {
      const context = contexts[offset];
      return context === 1 ? "stringLiteral" : context === 2 ? "comment" : "code";
    },
  };
}

function quotedLiteralEnd(source: string, bodyStart: number, quote: string): number {
  let cursor = bodyStart;
  while (cursor < source.length) {
    if (source[cursor] === "\\") {
      cursor += 2;
    } else if (source[cursor] === quote) {
      return cursor + 1;
    } else {
      cursor += 1;
    }
  }
  return source.length;
}

function blankRange(
  ownerSource: string[],
  contexts: Uint8Array,
  start: number,
  end: number,
  context: 1 | 2,
): void {
  for (let index = start; index < end; index += 1) {
    contexts[index] = context;
    if (ownerSource[index] !== "\n" && ownerSource[index] !== "\r") {
      ownerSource[index] = " ";
    }
  }
}

function mutableVariantCounts(): Record<PrecisionVariant, number> {
  return { Exact: 0, Conservative: 0, Heuristic: 0, Unknown: 0 };
}

function variantCounts(
  overrides: Partial<Record<PrecisionVariant, number>>,
): PrecisionVariantCounts {
  return { ...mutableVariantCounts(), ...overrides };
}

function site(
  sourcePath: string,
  owner: string,
  counts: Partial<Record<PrecisionVariant, number>>,
  disposition: PrecisionSiteDisposition,
  authority: string,
): PrecisionSiteDispositionRecord {
  const fullCounts = variantCounts(counts);
  return {
    sourcePath,
    owner,
    lexicalContext: "code",
    testOnly: false,
    variantCounts: fullCounts,
    occurrenceCount: Object.values(fullCounts).reduce((sum, count) => sum + count, 0),
    disposition,
    authority,
  };
}

function automaticDisposition(
  site: ObservedPrecisionSite,
): Pick<PrecisionSiteDispositionRecord, "disposition" | "authority"> | undefined {
  if (site.testOnly) {
    return {
      disposition: "test",
      authority: "test-only precision expectation or mutation fixture",
    };
  }
  if (site.lexicalContext === "comment") {
    return {
      disposition: "documentation",
      authority: "non-executable documentation example",
    };
  }
  if (site.lexicalContext === "stringLiteral") {
    return {
      disposition: "evidenceLiteral",
      authority: "non-executable source text used by a checker or fixture",
    };
  }
  return undefined;
}

function siteKey(
  site: Pick<ObservedPrecisionSite, "sourcePath" | "owner" | "lexicalContext" | "testOnly">,
): string {
  return `${site.sourcePath}#${site.owner}#${site.lexicalContext}#${site.testOnly}`;
}

function compareSites(left: ObservedPrecisionSite, right: ObservedPrecisionSite): number {
  return (
    left.sourcePath.localeCompare(right.sourcePath) ||
    left.owner.localeCompare(right.owner) ||
    left.lexicalContext.localeCompare(right.lexicalContext) ||
    Number(left.testOnly) - Number(right.testOnly)
  );
}

function rustSourcePaths(root: string): string[] {
  const paths: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      paths.push(...rustSourcePaths(entryPath));
    } else if (entry.name.endsWith(".rs")) {
      paths.push(entryPath);
    }
  }
  return paths.toSorted();
}
