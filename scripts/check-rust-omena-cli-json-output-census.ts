import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface JsonOutputSite {
  readonly siteId: string;
  readonly sourcePath: string;
  readonly method: string;
  readonly ordinal: number;
  readonly product: string;
}

interface JsonOutputCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.json-output-census";
  readonly summary: {
    readonly siteCount: number;
    readonly fileCount: number;
    readonly untaggedSiteCount: number;
  };
  readonly sites: readonly JsonOutputSite[];
}

const repoRoot = process.cwd();
const sourceRoot = path.join(repoRoot, "rust/crates/omena-cli/src");
const censusPath = path.join(repoRoot, "rust/crates/omena-cli/json-output-census.json");
const writeMode = process.argv.includes("--write");
const sites = listRustSources(sourceRoot).flatMap(scanJsonOutputSites);

if (process.env.OMENA_CLI_JSON_OUTPUT_TEST_INJECT_UNTAGGED === "1") {
  sites.push({
    siteId: "rust/crates/omena-cli/src/injected.rs:injected_output:1",
    sourcePath: "rust/crates/omena-cli/src/injected.rs",
    method: "injected_output",
    ordinal: 1,
    product: "",
  });
}

assert.ok(sites.length > 0, "CLI JSON output census must be non-vacuous");
const untaggedSites = sites.filter((site) => site.product.length === 0);
assert.deepEqual(
  untaggedSites,
  [],
  "every CLI JSON output call must provide semantic product metadata",
);
assert.ok(
  sites.every((site) => site.sourcePath !== "rust/crates/omena-cli/src/output.rs"),
  "the JSON output sink definition must not be counted as a call site",
);

const orderedSites = sites.toSorted((left, right) => left.siteId.localeCompare(right.siteId));
const census: JsonOutputCensus = {
  schemaVersion: "0",
  product: "omena-cli.json-output-census",
  summary: {
    siteCount: orderedSites.length,
    fileCount: new Set(orderedSites.map((site) => site.sourcePath)).size,
    untaggedSiteCount: untaggedSites.length,
  },
  sites: orderedSites,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.ok(fs.existsSync(censusPath), "missing CLI JSON output census; run with --write");
  assert.equal(
    fs.readFileSync(censusPath, "utf8"),
    serialized,
    "CLI JSON output census is stale; regenerate and review the source-derived sites",
  );
}

process.stdout.write(
  `Omena CLI JSON output census OK: sites=${census.summary.siteCount} files=${census.summary.fileCount} untagged=${census.summary.untaggedSiteCount}\n`,
);

function scanJsonOutputSites(absolutePath: string): JsonOutputSite[] {
  const sourcePath = path.relative(repoRoot, absolutePath);
  const source = fs.readFileSync(absolutePath, "utf8");
  const matches = [...source.matchAll(/\bprint_json\s*\(/gu)];
  const methodOrdinals = new Map<string, number>();
  return matches.map((match) => {
    const index = match.index ?? 0;
    const method = enclosingMethod(source, index);
    const ordinal = (methodOrdinals.get(method) ?? 0) + 1;
    methodOrdinals.set(method, ordinal);
    const callPrefix = source.slice(index, index + 512);
    const productMatch = /^print_json\s*\(\s*CliOutputMetadataV0::new\(\s*"([^"]+)"\s*\)/u.exec(
      callPrefix,
    );
    return {
      siteId: `${sourcePath}:${method}:${ordinal}`,
      sourcePath,
      method,
      ordinal,
      product: productMatch?.[1] ?? "",
    };
  });
}

function enclosingMethod(source: string, index: number): string {
  const prefix = source.slice(0, index);
  const methods = [
    ...prefix.matchAll(/(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)/gu),
  ];
  return methods.at(-1)?.[1] ?? "module";
}

function listRustSources(directory: string): string[] {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) return listRustSources(absolute);
      return entry.isFile() && entry.name.endsWith(".rs") ? [absolute] : [];
    })
    .toSorted();
}
