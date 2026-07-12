import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const transformIrSource = read("rust/crates/omena-transform-cst/src/transform_ir.rs");
const formatIrSource = read("rust/crates/omena-transform-print/src/doc_ir.rs");
const printBoundarySource = read("rust/crates/omena-transform-print/src/lib.rs");

const producerKinds = extractEnumVariants(transformIrSource, "IrNodeKindV0");
const manifestKinds = new Set(
  [...formatIrSource.matchAll(/node_kind:\s*"([a-z-]+)"/gu)].map((match) => match[1]),
);
const coverageBody = extractFunctionBody(formatIrSource, "coverage_strategy");
const classifiedKinds = new Set(
  [...coverageBody.matchAll(/IrNodeKindV0::([A-Z][A-Za-z0-9]*)/gu)].map((match) => match[1]),
);

assert.ok(producerKinds.length > 0, "transform IR must expose at least one node kind");
assert.deepEqual(
  [...manifestKinds].toSorted(),
  producerKinds.map(toKebabCase).toSorted(),
  "format coverage manifest must classify every transform IR producer kind",
);
assert.deepEqual(
  [...classifiedKinds].toSorted(),
  producerKinds.toSorted(),
  "format coverage strategy must contain one arm for every transform IR producer kind",
);

for (const marker of [
  "IrTransactionV0::new",
  ".replace_node_covering_span(",
  "materialize_transform_ir_printed_source",
]) {
  assert.ok(formatIrSource.includes(marker), `Doc-IR plan must retain print-seam marker ${marker}`);
}
assert.ok(
  printBoundarySource.includes("doc_ir::render_pretty_css_through_transform_ir"),
  "Pretty mode must consume the Doc-IR edit-plan path",
);
assert.ok(
  printBoundarySource.includes("rendered.generated_offset_lookup.as_deref()"),
  "Pretty mode must retain the shared generated-offset source-map seam",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-format-ir-coverage",
      producerKindCount: producerKinds.length,
      manifestKindCount: manifestKinds.size,
      classifiedKindCount: classifiedKinds.size,
      singlePrintSeam: true,
    },
    null,
    2,
  )}\n`,
);

function extractEnumVariants(source: string, enumName: string): string[] {
  const body = extractDelimitedBody(source, `pub enum ${enumName}`);
  return body
    .split("\n")
    .flatMap((line) => line.match(/^\s{4}([A-Z][A-Za-z0-9]*),?\s*$/u)?.slice(1) ?? []);
}

function extractFunctionBody(source: string, functionName: string): string {
  return extractDelimitedBody(source, `fn ${functionName}`);
}

function extractDelimitedBody(source: string, marker: string): string {
  const declarationStart = source.indexOf(marker);
  assert.notEqual(declarationStart, -1, `missing ${marker}`);
  const bodyStart = source.indexOf("{", declarationStart);
  assert.notEqual(bodyStart, -1, `missing body for ${marker}`);
  let depth = 1;
  let cursor = bodyStart + 1;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated body for ${marker}`);
  return source.slice(bodyStart + 1, cursor - 1);
}

function toKebabCase(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase();
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
