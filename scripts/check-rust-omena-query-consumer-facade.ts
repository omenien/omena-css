import { strict as assert } from "node:assert";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

type ConsumerCrate = {
  readonly crateName: string;
  readonly cratePath: string;
};

const CONSUMER_CRATES: readonly ConsumerCrate[] = [
  { crateName: "omena-cli", cratePath: "rust/crates/omena-cli" },
  { crateName: "omena-wasm", cratePath: "rust/crates/omena-wasm" },
  { crateName: "omena-napi", cratePath: "rust/crates/omena-napi" },
];

const FORBIDDEN_DIRECT_DEPENDENCIES = [
  "engine-input-producers",
  "omena-abstract-value",
  "omena-bridge",
  "omena-cascade",
  "omena-parser",
  "omena-resolver",
  "omena-semantic",
  "omena-transform-bundle",
  "omena-transform-cst",
  "omena-transform-egg",
  "omena-transform-passes",
  "omena-transform-print",
  "omena-transform-target",
] as const;

const FORBIDDEN_SOURCE_PREFIXES = [
  "engine_input_producers",
  "omena_abstract_value",
  "omena_bridge",
  "omena_cascade",
  "omena_parser",
  "omena_resolver",
  "omena_semantic",
  "omena_transform_bundle",
  "omena_transform_cst",
  "omena_transform_egg",
  "omena_transform_passes",
  "omena_transform_print",
  "omena_transform_target",
] as const;

for (const consumer of CONSUMER_CRATES) {
  const manifest = readFileSync(join(consumer.cratePath, "Cargo.toml"), "utf8");

  assert.match(
    manifest,
    /^omena-query\s*=/m,
    `${consumer.crateName} must depend on omena-query as its single analysis facade`,
  );

  for (const dependency of FORBIDDEN_DIRECT_DEPENDENCIES) {
    assert.doesNotMatch(
      manifest,
      new RegExp(`^${escapeRegExp(dependency)}\\s*=`, "m"),
      `${consumer.crateName} must not depend directly on ${dependency}; route through omena-query`,
    );
  }

  for (const sourcePath of listRustSourceFiles(join(consumer.cratePath, "src"))) {
    const source = readFileSync(sourcePath, "utf8");
    for (const prefix of FORBIDDEN_SOURCE_PREFIXES) {
      assert.doesNotMatch(
        source,
        new RegExp(`\\b(?:use\\s+${prefix}\\b|${prefix}::)`),
        `${consumer.crateName} must not call ${prefix} directly in ${sourcePath}; route through omena-query`,
      );
    }
  }
}

const boundarySource = readFileSync("rust/crates/omena-query/src/boundary.rs", "utf8");
for (const readySurface of [
  "consumerCheckFacade",
  "consumerBuildFacade",
  "consumerTransformPassListFacade",
] as const) {
  assert(
    boundarySource.includes(`"${readySurface}"`),
    `omena-query boundary must advertise ${readySurface}`,
  );
}

console.log(
  "validated omena-query consumer facade boundary:",
  CONSUMER_CRATES.map((consumer) => consumer.crateName).join(", "),
);

function listRustSourceFiles(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) {
      return listRustSourceFiles(path);
    }
    return path.endsWith(".rs") ? [path] : [];
  });
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
