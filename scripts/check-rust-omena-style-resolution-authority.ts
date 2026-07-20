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
  readonly callOrdinal: number;
  readonly evidence: string;
}

interface NamedExemption {
  readonly siteKey: string;
  readonly reason: string;
}

interface StyleResolutionAuthorityCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-query.style-resolution-authority-census";
  readonly policy: {
    readonly direction: "decrease-only";
    readonly floor: 0;
    readonly owningCheck: "rust/omena-style-resolution-authority";
    readonly packageScript: "check:rust-omena-style-resolution-authority";
  };
  readonly sourceRoot: "rust/crates/omena-query/src";
  readonly resolver: "resolve_style_module_source";
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
const namedExemptions: readonly NamedExemption[] = [];

const existing = readExistingCensus();
const sites = scanPlainResolverCallSites();
const baselinePlainCallerCount = existing?.baselinePlainCallerCount ?? sites.length;
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

if (existing) {
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
  schemaVersion: "0",
  product: "omena-query.style-resolution-authority-census",
  policy: {
    direction: "decrease-only",
    floor: 0,
    owningCheck: "rust/omena-style-resolution-authority",
    packageScript: "check:rust-omena-style-resolution-authority",
  },
  sourceRoot: "rust/crates/omena-query/src",
  resolver: "resolve_style_module_source",
  baselinePlainCallerCount,
  currentPlainCallerCount: sites.length,
  namedExemptions,
  sites,
  siteDigest: `sha256:${createHash("sha256").update(JSON.stringify(sites)).digest("hex")}`,
};

if (writeMode) {
  assert.equal(injectPlainCaller, false, "test injection cannot be combined with --write");
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
  assert.equal(parsed.schemaVersion, "0", "style resolution census schemaVersion");
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
    const code = maskRustNonCode(source);
    const pattern = /\bresolve_style_module_source\s*\(/gu;
    const ordinalByFunction = new Map<string, number>();
    for (const match of code.matchAll(pattern)) {
      const index = match.index;
      const prefix = code.slice(0, index);
      const previousNonWhitespace = prefix.match(/\S(?=\s*$)/u)?.[0];
      if (previousNonWhitespace === ".") continue;
      const previousWord = prefix.match(/([A-Za-z_][A-Za-z0-9_]*)\s*$/u)?.[1];
      if (previousWord === "fn") continue;
      const functionName = enclosingFunctionName(code, index);
      const callOrdinal = ordinalByFunction.get(functionName) ?? 0;
      ordinalByFunction.set(functionName, callOrdinal + 1);
      const line = lineNumberAt(source, index);
      callSites.push({
        path: relativePath,
        line,
        function: functionName,
        callOrdinal,
        evidence: source.split(/\r?\n/u)[line - 1]?.trim() ?? "",
      });
    }
  }
  return callSites.toSorted(
    (left, right) => left.path.localeCompare(right.path) || left.line - right.line,
  );
}

function trackedRustSources(): string[] {
  const result = spawnSync("git", ["ls-files", "rust/crates/omena-query/src"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `failed to enumerate omena-query sources: ${result.stderr}`);
  return result.stdout
    .split(/\r?\n/u)
    .filter((entry) => entry.endsWith(".rs"))
    .toSorted();
}

function stableSiteKey(site: PlainResolverCallSite): string {
  return `${site.path}#${site.function}#${site.callOrdinal}`;
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
