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

let generatedRustSource = read(generatedRustPath);
const helperSource = read(helperPath);
const computedValueSource = read(computedValuePath);
const generatorSource = read(generatorPath);
const generatedTypescriptSource = read(generatedTypescriptPath);
const expectedRows = loadDerivedPropertyMetadataRows(repoRoot);

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
      complete: true,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}
