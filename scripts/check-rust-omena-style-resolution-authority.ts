import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface PlainResolverCallSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly authority: string;
  readonly callOrdinal: number;
  readonly evidence: string;
}

interface NamedExemption {
  readonly siteKey: string;
  readonly reason: string;
}

interface StyleResolutionAuthorityCensus {
  readonly schemaVersion: "0" | "1";
  readonly product: "omena-query.style-resolution-authority-census";
  readonly policy: {
    readonly direction: "decrease-only";
    readonly floor: 0;
    readonly owningCheck: "rust/omena-style-resolution-authority";
    readonly packageScript: "check:rust-omena-style-resolution-authority";
  };
  readonly sourceRoots: readonly ["rust/crates/omena-query/src", "rust/crates/omena-cli/src"];
  readonly authorities: readonly [
    "resolve_style_module_source",
    "summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs",
  ];
  readonly baselinePlainCallerCount: number;
  readonly currentPlainCallerCount: number;
  readonly namedExemptions: readonly NamedExemption[];
  readonly sites: readonly PlainResolverCallSite[];
  readonly siteDigest: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = path.join(repoRoot, "rust/omena-style-resolution-authority-census.json");
const writeMode = process.argv.includes("--write");
const injectPlainCaller = process.env.OMENA_STYLE_RESOLUTION_TEST_INJECT_PLAIN_CALLER === "1";
const injectDefaultSummary = process.env.OMENA_STYLE_RESOLUTION_TEST_INJECT_DEFAULT_SUMMARY === "1";
const namedExemptions: readonly NamedExemption[] = [
  {
    siteKey:
      "rust/crates/omena-query/src/style/cross_file_summary.rs#summarize_omena_query_workspace_cross_file_summary#summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs#0",
    reason:
      "The compatibility entry point preserves mapping-free behavior while product callers use the resolution-aware sibling.",
  },
];

const existing = readExistingCensus();
const sites = scanPlainResolverCallSites();
const baselinePlainCallerCount = Math.max(
  existing?.baselinePlainCallerCount ?? sites.length,
  namedExemptions.length,
);
const exemptionKeys = new Set(namedExemptions.map((entry) => entry.siteKey));
const unclassifiedSites = sites.filter((site) => !exemptionKeys.has(stableSiteKey(site)));

assert.equal(
  new Set(namedExemptions.map((entry) => entry.siteKey)).size,
  namedExemptions.length,
  "style resolution exemptions must use unique stable site keys",
);
for (const exemption of namedExemptions) {
  assert.ok(exemption.reason.trim().length > 0, `missing exemption reason: ${exemption.siteKey}`);
}
assert.ok(
  sites.length <= baselinePlainCallerCount,
  `plain style resolver caller count increased: baseline=${baselinePlainCallerCount} current=${sites.length}`,
);
assert.deepEqual(
  unclassifiedSites,
  [],
  "every mapping-less style resolver caller must be removed or registered with a domain reason",
);
assert.equal(
  sites.length,
  namedExemptions.length,
  "the post-sweep plain caller count must equal the named-exemption registry",
);

if (existing && !writeMode) {
  assert.deepEqual(
    namedExemptions,
    existing.namedExemptions,
    "style resolution exemptions changed without an explicit census update",
  );
  assert.ok(
    sites.length <= existing.currentPlainCallerCount,
    `plain style resolver caller count regressed: previous=${existing.currentPlainCallerCount} current=${sites.length}`,
  );
}

const census: StyleResolutionAuthorityCensus = {
  schemaVersion: "1",
  product: "omena-query.style-resolution-authority-census",
  policy: {
    direction: "decrease-only",
    floor: 0,
    owningCheck: "rust/omena-style-resolution-authority",
    packageScript: "check:rust-omena-style-resolution-authority",
  },
  sourceRoots: ["rust/crates/omena-query/src", "rust/crates/omena-cli/src"],
  authorities: [
    "resolve_style_module_source",
    "summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs",
  ],
  baselinePlainCallerCount,
  currentPlainCallerCount: sites.length,
  namedExemptions,
  sites,
  siteDigest: `sha256:${createHash("sha256").update(JSON.stringify(sites)).digest("hex")}`,
};

if (writeMode) {
  assert.equal(
    injectPlainCaller || injectDefaultSummary,
    false,
    "test injection cannot be combined with --write",
  );
  writeFileSync(censusPath, `${JSON.stringify(census, null, 2)}\n`);
  const format = spawnSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(format.status, 0, `failed to format style resolution census: ${format.stderr}`);
} else {
  assert.ok(existsSync(censusPath), "style resolution census is missing; run its update command");
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "style resolution authority census is stale",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      baselinePlainCallerCount: census.baselinePlainCallerCount,
      currentPlainCallerCount: census.currentPlainCallerCount,
      namedExemptionCount: census.namedExemptions.length,
      siteDigest: census.siteDigest,
      direction: census.policy.direction,
    },
    null,
    2,
  )}\n`,
);

function readExistingCensus(): StyleResolutionAuthorityCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as StyleResolutionAuthorityCensus;
  assert.ok(
    parsed.schemaVersion === "0" || parsed.schemaVersion === "1",
    "style resolution census schemaVersion",
  );
  assert.equal(parsed.product, "omena-query.style-resolution-authority-census", "census product");
  assert.equal(parsed.policy.direction, "decrease-only", "census direction");
  assert.equal(parsed.policy.floor, 0, "plain resolver tolerance floor");
  assert.equal(parsed.currentPlainCallerCount, parsed.sites.length, "committed caller count");
  assert.equal(
    parsed.siteDigest,
    `sha256:${createHash("sha256").update(JSON.stringify(parsed.sites)).digest("hex")}`,
    "committed caller digest",
  );
  return parsed;
}

function scanPlainResolverCallSites(): PlainResolverCallSite[] {
  const callSites: PlainResolverCallSite[] = [];
  for (const relativePath of trackedRustSources()) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (injectPlainCaller && relativePath.endsWith("/style.rs")) {
      source = `fn injected_mapping_less_query() { let _ = resolve_style_module_source("a", "b", &Default::default(), &[]); }\n${source}`;
    }
    if (injectDefaultSummary && relativePath.endsWith("/modules.rs")) {
      source = `fn injected_mapping_less_summary() { let _ = summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs(&[], &[], &[], &Default::default(),); }\n${source}`;
    }
    const code = maskRustNonCode(source);
    const authorities = [
      {
        name: "resolve_style_module_source",
        pattern: /\bresolve_style_module_source\s*\(/gu,
        isMappingless: () => true,
      },
      {
        name: "summarize_omena_query_workspace_cross_file_summary_with_resolution_inputs",
        pattern:
          /\bsummarize_omena_query_workspace_cross_file_summary_with_resolution_inputs\s*\(/gu,
        isMappingless: (index: number) => callUsesDefaultResolutionInputs(code, index),
      },
    ] as const;
    const ordinalByFunction = new Map<string, number>();
    for (const authority of authorities) {
      for (const match of code.matchAll(authority.pattern)) {
        const index = match.index;
        const prefix = code.slice(0, index);
        const previousNonWhitespace = prefix.match(/\S(?=\s*$)/u)?.[0];
        if (previousNonWhitespace === ".") continue;
        const previousWord = prefix.match(/([A-Za-z_][A-Za-z0-9_]*)\s*$/u)?.[1];
        if (previousWord === "fn" || !authority.isMappingless(index)) continue;
        const functionName = enclosingFunctionName(code, index);
        const ordinalKey = `${authority.name}#${functionName}`;
        const callOrdinal = ordinalByFunction.get(ordinalKey) ?? 0;
        ordinalByFunction.set(ordinalKey, callOrdinal + 1);
        const line = lineNumberAt(source, index);
        callSites.push({
          path: relativePath,
          line,
          function: functionName,
          authority: authority.name,
          callOrdinal,
          evidence: source.split(/\r?\n/u)[line - 1]?.trim() ?? "",
        });
      }
    }
  }
  return callSites.toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.function.localeCompare(right.function) ||
      left.authority.localeCompare(right.authority) ||
      left.callOrdinal - right.callOrdinal,
  );
}

function trackedRustSources(): string[] {
  const result = spawnSync(
    "git",
    ["ls-files", "rust/crates/omena-query/src", "rust/crates/omena-cli/src"],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );
  assert.equal(result.status, 0, `failed to enumerate style-resolution sources: ${result.stderr}`);
  return result.stdout
    .split(/\r?\n/u)
    .filter((entry) => entry.endsWith(".rs"))
    .toSorted();
}

function stableSiteKey(site: PlainResolverCallSite): string {
  return `${site.path}#${site.function}#${site.authority}#${site.callOrdinal}`;
}

function callUsesDefaultResolutionInputs(code: string, callIndex: number): boolean {
  const openParen = code.indexOf("(", callIndex);
  if (openParen < 0) return false;
  const closeParen = matchingParen(code, openParen);
  if (closeParen < 0) return false;
  const argumentsList = topLevelArguments(code.slice(openParen + 1, closeParen));
  const resolutionInputs = argumentsList.at(-1)?.trim() ?? "";
  return /^(?:&\s*)?(?:(?:OmenaQueryStyleResolutionInputsV0|Default)\s*::\s*)?default\s*\(\s*\)$/u.test(
    resolutionInputs,
  );
}

function matchingParen(code: string, openParen: number): number {
  let depth = 0;
  for (let index = openParen; index < code.length; index += 1) {
    if (code[index] === "(") depth += 1;
    if (code[index] === ")") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function topLevelArguments(body: string): string[] {
  const argumentsList: string[] = [];
  let start = 0;
  let parenDepth = 0;
  let bracketDepth = 0;
  let braceDepth = 0;
  for (let index = 0; index < body.length; index += 1) {
    const character = body[index];
    if (character === "(") parenDepth += 1;
    else if (character === ")") parenDepth -= 1;
    else if (character === "[") bracketDepth += 1;
    else if (character === "]") bracketDepth -= 1;
    else if (character === "{") braceDepth += 1;
    else if (character === "}") braceDepth -= 1;
    else if (character === "," && parenDepth === 0 && bracketDepth === 0 && braceDepth === 0) {
      argumentsList.push(body.slice(start, index));
      start = index + 1;
    }
  }
  argumentsList.push(body.slice(start));
  return argumentsList.filter((argument, index) => argument.trim().length > 0 || index === 0);
}

function enclosingFunctionName(code: string, index: number): string {
  const prefix = code.slice(0, index);
  const matches = [...prefix.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/gu)];
  return matches.at(-1)?.[1] ?? "<module>";
}

function lineNumberAt(source: string, index: number): number {
  return source.slice(0, index).split("\n").length;
}

function maskRustNonCode(source: string): string {
  const chars = [...source];
  let index = 0;
  let blockDepth = 0;
  let quote: '"' | undefined;
  while (index < chars.length) {
    const current = chars[index];
    const next = chars[index + 1];
    if (blockDepth > 0) {
      if (current === "/" && next === "*") {
        chars[index] = chars[index + 1] = " ";
        blockDepth += 1;
        index += 2;
      } else if (current === "*" && next === "/") {
        chars[index] = chars[index + 1] = " ";
        blockDepth -= 1;
        index += 2;
      } else {
        if (current !== "\n") chars[index] = " ";
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (current === "\\") {
        if (current !== "\n") chars[index] = " ";
        if (next !== "\n") chars[index + 1] = " ";
        index += 2;
      } else {
        if (current !== "\n") chars[index] = " ";
        index += 1;
        if (current === quote) quote = undefined;
      }
      continue;
    }
    if (current === "/" && next === "/") {
      while (index < chars.length && chars[index] !== "\n") {
        chars[index] = " ";
        index += 1;
      }
      continue;
    }
    if (current === "/" && next === "*") {
      chars[index] = chars[index + 1] = " ";
      blockDepth = 1;
      index += 2;
      continue;
    }
    if (current === '"') {
      quote = current;
      chars[index] = " ";
    }
    index += 1;
  }
  return chars.join("");
}
