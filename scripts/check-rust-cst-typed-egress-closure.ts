import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";

const repoRoot = process.cwd();
const cratesRoot = path.join(repoRoot, "rust", "crates");
const targetCrate = "omena-cst-typed";

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function dependencySection(cargoToml: string): string {
  const match = cargoToml.match(/\[dependencies\]\n([\s\S]*?)(?:\n\[|$)/);
  return match?.[1] ?? "";
}

function shippedDependencyNames(crateName: string): string[] {
  const cargoToml = read(path.join("rust", "crates", crateName, "Cargo.toml"));
  return dependencySection(cargoToml)
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.startsWith("#"))
    .map((line) => line.split("=")[0]?.trim())
    .filter((name): name is string => Boolean(name));
}

const crateNames = readdirSync(cratesRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .toSorted();

const inboundShippedDeps = crateNames.filter((crateName) =>
  shippedDependencyNames(crateName).includes(targetCrate),
);

assert.deepEqual(
  inboundShippedDeps,
  ["omena-query"],
  `${targetCrate} must be consumed through omena-query only; inbound shipped deps: ${inboundShippedDeps.join(
    ", ",
  )}`,
);

for (const crateName of ["omena-wasm", "omena-napi"]) {
  const cargoToml = read(path.join("rust", "crates", crateName, "Cargo.toml"));
  assert.equal(
    cargoToml.includes(targetCrate),
    false,
    `${crateName} must not depend directly on ${targetCrate}`,
  );

  const source = read(path.join("rust", "crates", crateName, "src", "lib.rs"));
  assert.equal(
    source.includes("omena_cst_typed"),
    false,
    `${crateName} must not call ${targetCrate} directly`,
  );
  assert.equal(
    source.includes("parse_style_document_typed_v0"),
    true,
    `${crateName} must route parse stylesheet egress through omena-query`,
  );
}

const querySource = read("rust/crates/omena-query/src/lib.rs");
assert.equal(
  querySource.includes("pub fn parse_style_document_typed_v0"),
  true,
  "omena-query must own the parse stylesheet egress function",
);

console.log(
  JSON.stringify(
    {
      product: "rust.cst-typed-egress-closure",
      targetCrate,
      inboundShippedDeps,
      wasmDirectEdge: false,
      napiDirectEdge: false,
    },
    null,
    2,
  ),
);
