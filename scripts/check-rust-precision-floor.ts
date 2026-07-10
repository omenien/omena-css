import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function rustSources(relativeDirectory: string): string[] {
  const directory = path.join(repoRoot, relativeDirectory);
  return fs
    .readdirSync(directory, { recursive: true, encoding: "utf8" })
    .filter((entry) => entry.endsWith(".rs"))
    .map((entry) => fs.readFileSync(path.join(directory, entry), "utf8"));
}

function blockBody(source: string, marker: string): string {
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `missing ${marker}`);
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `missing body for ${marker}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated body for ${marker}`);
}

function topLevelEnumVariants(source: string, enumName: string): string[] {
  const body = blockBody(source, `pub enum ${enumName}`);
  const variants: string[] = [];
  let depth = 0;
  for (const line of body.split("\n")) {
    const trimmed = line.trim();
    if (depth === 0) {
      const match = trimmed.match(/^([A-Z][A-Za-z0-9]*)\b/u);
      if (match?.[1]) variants.push(match[1]);
    }
    depth += [...line].filter((char) => char === "{").length;
    depth -= [...line].filter((char) => char === "}").length;
  }
  return [...new Set(variants)];
}

const abstractTypes = read("rust/crates/omena-abstract-value/src/types.rs");
const abstractDomain = read("rust/crates/omena-abstract-value/src/domain.rs");
const queryCore = read("rust/crates/omena-query-core/src/lib.rs");
const queryTypes = read("rust/crates/omena-query/src/types.rs");

const factPrecisionVariants = topLevelEnumVariants(abstractTypes, "FactPrecision");
assert.deepEqual(factPrecisionVariants, ["Exact", "Conservative", "Heuristic", "Unknown"]);

const classValueVariants = topLevelEnumVariants(abstractTypes, "AbstractClassValueV0");
const classValueAdapter = blockBody(abstractDomain, "pub fn fact_precision_from_class_value");
const mappedClassValueVariants = [
  ...new Set(
    [...classValueAdapter.matchAll(/AbstractClassValueV0::([A-Z][A-Za-z0-9]*)/gu)].map(
      (match) => match[1],
    ),
  ),
].toSorted();
assert.deepEqual(mappedClassValueVariants, classValueVariants.toSorted());
assert.ok(!/(^|[^\w])_\s*=>/u.test(classValueAdapter), "class-value adapter must not catch all");

const producerSources = [queryCore, ...rustSources("rust/crates/omena-query/src")];
const producerValueDomains = new Set<string>();
for (const source of producerSources) {
  for (const match of source.matchAll(/value_domain:\s*"([^"]+)"/gu)) {
    if (match[1]) producerValueDomains.add(match[1]);
  }
  for (const match of source.matchAll(/source_diagnostic_precision\(\s*"([^"]+)"/gu)) {
    if (match[1]) producerValueDomains.add(match[1]);
  }
  if (source.includes("OMENA_QUERY_TYPE_ORACLE_UNKNOWN_VALUE_DOMAIN")) {
    producerValueDomains.add("unknown");
  }
}

const analysisAdapter = blockBody(queryCore, "pub fn fact_precision_from_analysis_precision");
for (const valueDomain of producerValueDomains) {
  assert.ok(
    analysisAdapter.includes(`"${valueDomain}"`),
    `query precision producer is not mapped: ${valueDomain}`,
  );
}
assert.ok(
  analysisAdapter.includes("FactPrecision::Unknown"),
  "open string precision inputs must fail closed to Unknown",
);
assert.ok(
  queryTypes.includes("pub fn fact_precision_from_evidence_analysis_precision"),
  "evidence precision must reuse the query-side precision adapter",
);

const factPrecisionDeclarations = rustSources("rust/crates").reduce(
  (count, source) => count + [...source.matchAll(/pub enum FactPrecision\s*\{/gu)].length,
  0,
);
assert.equal(factPrecisionDeclarations, 1, "FactPrecision must have one authority");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-transform.precision-floor",
      factPrecisionVariants,
      classValueVariantCount: classValueVariants.length,
      mappedClassValueVariantCount: mappedClassValueVariants.length,
      queryPrecisionValueDomains: [...producerValueDomains].toSorted(),
      complete: true,
    },
    null,
    2,
  )}\n`,
);
