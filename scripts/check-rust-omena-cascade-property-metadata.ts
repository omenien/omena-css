import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { loadDerivedPropertyMetadataRows } from "./property-metadata-registry";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const generatedRustPath = "rust/crates/omena-cascade/src/property_metadata_idl_generated.rs";
const helperPath = "rust/crates/omena-cascade/src/property_metadata.rs";
const computedValuePath = "rust/crates/omena-cascade/src/computed_value.rs";
const generatorPath = "scripts/generate-engine-v2-contract-idl.ts";
const generatedTypescriptPath =
  "server/engine-core-ts/src/contracts/property-metadata-idl.generated.ts";
const callerAdjudicationPath =
  "rust/crates/omena-spec-audit/data/property-metadata-caller-adjudication.json";

interface PropertyMetadataCallerAdjudication {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.property-metadata-caller-adjudication";
  readonly policyTaxonomy: readonly string[];
  readonly entries: readonly {
    readonly sourcePath: string;
    readonly function: string;
    readonly helper: "css_property_initial_value" | "css_property_is_inherited";
    readonly callCount: number;
    readonly policy: string;
    readonly evidence: string;
  }[];
}

let generatedRustSource = read(generatedRustPath);
const helperSource = read(helperPath);
const computedValueSource = read(computedValuePath);
const generatorSource = read(generatorPath);
const generatedTypescriptSource = read(generatedTypescriptPath);
const expectedRows = loadDerivedPropertyMetadataRows(repoRoot);
const callerAdjudication = JSON.parse(
  read(callerAdjudicationPath),
) as PropertyMetadataCallerAdjudication;

if (process.argv.includes("--inject-generated-row-drift")) {
  generatedRustSource = generatedRustSource.replace(
    `property_id: "${expectedRows[0]?.propertyId}"`,
    'property_id: "injected-generated-row-drift"',
  );
}

assert.ok(
  generatorSource.includes("contracts/property-metadata/main.tsp"),
  "property metadata must use the existing contract IDL generator",
);
validateCallerAdjudication(callerAdjudication);
assert.ok(
  generatorSource.includes("loadDerivedPropertyMetadataRows(repoRoot)"),
  "property metadata rows must derive from the pinned registry snapshot",
);
assert.ok(
  !generatorSource.includes("const propertyMetadataRows = ["),
  "the generator must not keep a hand-authored property table",
);
assert.ok(
  generatedTypescriptSource.includes("export interface CssPropertyMetadataV1Json"),
  "TypeScript property metadata IDL must be generated",
);
assert.ok(
  generatedTypescriptSource.includes("boundaryClassification: string"),
  "TypeScript property metadata must carry boundary provenance",
);
assert.ok(
  generatedRustSource.includes("pub const CSS_PROPERTY_METADATA_V1"),
  "Rust property metadata DB must be generated",
);
assert.ok(
  generatedRustSource.includes('package: "@webref/css"'),
  "property metadata source must carry the Webref package pin",
);
assert.ok(
  generatedRustSource.includes("pub inherited: Option<bool>"),
  "registered property inheritance absence must be typed",
);
assert.ok(
  generatedRustSource.includes("pub initial_value: Option<&'static str>"),
  "registered property initial-value absence must be typed",
);

const actualPropertyIds = [...generatedRustSource.matchAll(/\n\s*property_id:\s*"([^"]+)",/gu)].map(
  (match) => match[1],
);
const expectedPropertyIds = expectedRows.map((row) => row.propertyId);
assert.deepEqual(
  actualPropertyIds,
  expectedPropertyIds,
  "generated property rows differ from the pinned snapshot derivation",
);
assert.equal(
  new Set(actualPropertyIds).size,
  actualPropertyIds.length,
  "generated property ids must be unique",
);
assert.ok(
  expectedRows.some((row) => row.syntax === null),
  "the registry must retain properties whose upstream syntax is absent",
);
assert.ok(
  expectedRows.some((row) => row.inherited === null),
  "the registry must retain typed inheritance absence",
);
assert.ok(
  expectedRows.some((row) => row.overrideReason !== null),
  "compatibility overrides must remain named and countable",
);
assert.ok(
  helperSource.includes("CSS_PROPERTY_METADATA_RECORDS_V1"),
  "runtime helper must read from the generated property metadata DB",
);
assert.ok(
  computedValueSource.includes("css_property_is_inherited"),
  "computed-value inheritance reader must route through property metadata",
);
assert.ok(
  computedValueSource.includes("css_property_initial_value"),
  "computed-value initial reader must route through property metadata",
);
assert.ok(
  !computedValueSource.includes("match property {"),
  "computed-value module must not keep a second initial-value match table",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "1",
      product: "omena-cascade.property-metadata",
      rowCount: expectedRows.length,
      inheritedKnownCount: expectedRows.filter((row) => row.inherited !== null).length,
      initialValueKnownCount: expectedRows.filter((row) => row.initialValue !== null).length,
      syntaxAbsentCount: expectedRows.filter((row) => row.syntax === null).length,
      overrideCount: expectedRows.filter((row) => row.overrideReason !== null).length,
      adjudicatedCallerCount: callerAdjudication.entries.length,
      complete: true,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function validateCallerAdjudication(adjudication: PropertyMetadataCallerAdjudication): void {
  assert.equal(adjudication.schemaVersion, "0");
  assert.equal(adjudication.product, "omena-spec-audit.property-metadata-caller-adjudication");
  assert.equal(new Set(adjudication.policyTaxonomy).size, adjudication.policyTaxonomy.length);

  const actualCallers = collectPropertyMetadataCallers();
  if (process.argv.includes("--inject-unadjudicated-caller")) {
    actualCallers.push({
      sourcePath: "rust/crates/omena-cascade/src/injected_consumer.rs",
      function: "injected_consumer",
      helper: "css_property_is_inherited",
      callCount: 1,
    });
  }
  const expectedCallers = adjudication.entries.map(
    ({ sourcePath, function: functionName, helper, callCount }) => ({
      sourcePath,
      function: functionName,
      helper,
      callCount,
    }),
  );
  assert.deepEqual(
    actualCallers.toSorted(compareCallerRows),
    expectedCallers.toSorted(compareCallerRows),
    "every production property metadata caller must have an adjudication entry",
  );

  const seen = new Set<string>();
  for (const entry of adjudication.entries) {
    const key = callerKey(entry);
    assert.ok(!seen.has(key), `duplicate property metadata caller adjudication: ${key}`);
    seen.add(key);
    assert.ok(adjudication.policyTaxonomy.includes(entry.policy), `${key} uses an unknown policy`);
    assert.ok(entry.evidence.length > 0, `${key} must name its policy evidence`);
    const source = read(entry.sourcePath);
    const body = extractRustFunctionBody(source, entry.function);
    assert.equal(
      countMatches(body, new RegExp(`\\b${entry.helper}\\s*\\(`, "gu")),
      entry.callCount,
    );
    assert.ok(body.includes(entry.evidence), `${key} no longer carries ${entry.evidence}`);
  }
}

function collectPropertyMetadataCallers(): Array<{
  sourcePath: string;
  function: string;
  helper: "css_property_initial_value" | "css_property_is_inherited";
  callCount: number;
}> {
  const callCounts = new Map<
    string,
    {
      sourcePath: string;
      function: string;
      helper: "css_property_initial_value" | "css_property_is_inherited";
      callCount: number;
    }
  >();
  for (const sourcePath of rustSourcePaths(path.join(repoRoot, "rust/crates"))) {
    if (
      sourcePath.endsWith("/property_metadata.rs") ||
      sourcePath.endsWith("/tests.rs") ||
      sourcePath.includes("/tests/")
    ) {
      continue;
    }
    const relativePath = path.relative(repoRoot, sourcePath);
    const source = fs.readFileSync(sourcePath, "utf8");
    for (const match of source.matchAll(
      /\b(css_property_initial_value|css_property_is_inherited)\s*\(/gu,
    )) {
      const functionName = enclosingRustFunctionName(source, match.index ?? 0);
      assert.ok(functionName, `${relativePath} property metadata call must be inside a function`);
      const helper = match[1] as "css_property_initial_value" | "css_property_is_inherited";
      const key = `${relativePath}:${functionName}:${helper}`;
      const current = callCounts.get(key);
      callCounts.set(key, {
        sourcePath: relativePath,
        function: functionName,
        helper,
        callCount: (current?.callCount ?? 0) + 1,
      });
    }
  }
  return [...callCounts.values()];
}

function rustSourcePaths(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) return rustSourcePaths(entryPath);
    return entry.isFile() && entry.name.endsWith(".rs") ? [entryPath] : [];
  });
}

function enclosingRustFunctionName(source: string, offset: number): string | null {
  const prefix = source.slice(0, offset);
  const matches = [...prefix.matchAll(/\bfn\s+([A-Za-z0-9_]+)\s*\(/gu)];
  return matches.at(-1)?.[1] ?? null;
}

function extractRustFunctionBody(source: string, functionName: string): string {
  const signature = new RegExp(`\\bfn\\s+${functionName}\\s*\\(`, "u").exec(source);
  assert.ok(signature, `missing Rust function ${functionName}`);
  const openBrace = source.indexOf("{", signature.index);
  assert.ok(openBrace >= 0, `missing Rust body for ${functionName}`);
  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(openBrace + 1, index);
  }
  throw new Error(`unterminated Rust body for ${functionName}`);
}

function countMatches(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length;
}

function callerKey(entry: { sourcePath: string; function: string; helper: string }): string {
  return `${entry.sourcePath}:${entry.function}:${entry.helper}`;
}

function compareCallerRows(
  left: { sourcePath: string; function: string; helper: string },
  right: { sourcePath: string; function: string; helper: string },
): number {
  return callerKey(left).localeCompare(callerKey(right));
}
