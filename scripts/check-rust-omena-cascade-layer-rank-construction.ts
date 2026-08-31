import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type ConstructionPartition =
  | "declaration-shape"
  | "normalizer-body"
  | "fixture-builder"
  | "unclassified";

interface ConstructionRow {
  readonly sourcePath: string;
  readonly occurrenceOrdinal: number;
  readonly partition: ConstructionPartition;
  readonly source: string;
  readonly reason: string;
}

interface ConstructionCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-cascade.layer-rank-construction-census";
  readonly sourceRoot: string;
  readonly publicSurfaceAuthority: "rust/published-crate-surface-register";
  readonly partitions: readonly {
    readonly id: ConstructionPartition;
    readonly expectedCount: number;
    readonly reason: string;
  }[];
  readonly summary: {
    readonly occurrenceCount: number;
    readonly declarationShapeCount: number;
    readonly normalizerBodyCount: number;
    readonly fixtureBuilderCount: number;
    readonly unclassifiedCount: number;
  };
  readonly rows: readonly ConstructionRow[];
  readonly limitations: readonly string[];
}

const repoRoot = process.cwd();
const sourceRoot = "rust/crates/omena-cascade/src";
const censusPath = path.join(repoRoot, "rust/omena-cascade-layer-rank-construction-census.json");
const registerPath = path.join(repoRoot, "rust/omena-published-crate-surface-register.json");
const snapshotRelativePath = "rust/crates/omena-cascade/tests/snapshots/public-api.txt";
const snapshotPath = path.join(repoRoot, snapshotRelativePath);
const fixtureSourcePaths = new Set([
  "rust/crates/omena-cascade/src/conformance.rs",
  "rust/crates/omena-cascade/src/fuzz.rs",
  "rust/crates/omena-cascade/src/tests.rs",
]);

const sourcePaths = collectRustSources(path.join(repoRoot, sourceRoot));
const rows = scanConstructionRows(sourcePaths);
const declarationRows = rows.filter((row) => row.partition === "declaration-shape");
const normalizerRows = rows.filter((row) => row.partition === "normalizer-body");
const fixtureRows = rows.filter((row) => row.partition === "fixture-builder");
const unclassifiedRows = rows.filter((row) => row.partition === "unclassified");

// A duplicate declaration or any visibility/field-shape change makes this false.
// The scanner can emit that failure because it reads every Rust source in the owning crate.
assert.equal(declarationRows.length, 1, "LayerRank must have one exact private-field declaration");
// Deleting, adding, or moving a direct constructor outside one of the four match arms makes this false.
// The scanner can emit that failure because each LayerRank tuple expression becomes a census row.
assert.equal(
  normalizerRows.length,
  4,
  "normalized_layer_rank must remain the four-arm LayerRank construction authority",
);
// Any direct constructor outside the declared partitions makes this false.
// The producer can emit such a row from every scanned source path, including newly added files.
assert.deepEqual(
  unclassifiedRows,
  [],
  "LayerRank construction contains an unclassified in-crate producer",
);

const census: ConstructionCensus = {
  schemaVersion: "0",
  product: "omena-cascade.layer-rank-construction-census",
  sourceRoot,
  publicSurfaceAuthority: "rust/published-crate-surface-register",
  partitions: [
    {
      id: "declaration-shape",
      expectedCount: declarationRows.length,
      reason:
        "The public tuple type has one private scalar field; construction remains module-owned.",
    },
    {
      id: "normalizer-body",
      expectedCount: normalizerRows.length,
      reason:
        "The four exhaustive importance and layer-presence arms are the only production constructors.",
    },
    {
      id: "fixture-builder",
      expectedCount: fixtureRows.length,
      reason:
        "Fixtures currently consume the public normalizer and need no raw-rank construction exemption.",
    },
    {
      id: "unclassified",
      expectedCount: unclassifiedRows.length,
      reason: "No unclassified direct construction is accepted.",
    },
  ],
  summary: {
    occurrenceCount: rows.length,
    declarationShapeCount: declarationRows.length,
    normalizerBodyCount: normalizerRows.length,
    fixtureBuilderCount: fixtureRows.length,
    unclassifiedCount: unclassifiedRows.length,
  },
  rows,
  limitations: [
    "This census covers direct LayerRank tuple syntax inside omena-cascade; Rust field privacy is the cross-crate construction fence.",
    "The published-crate surface register compares the generated public API with the committed snapshot and rejects additional public rank constructors.",
  ],
};

const committed = JSON.parse(readFileSync(censusPath, "utf8")) as ConstructionCensus;
// Any source-derived row, partition count, reason, or limitation drift makes this false.
// The producer can emit the drift because the entire census except policy prose is rebuilt from source.
assert.deepEqual(committed, census, "LayerRank construction census is stale");

const register = JSON.parse(readFileSync(registerPath, "utf8")) as {
  readonly rows: readonly {
    readonly crate: string;
    readonly disposition: string;
    readonly snapshot?: string;
  }[];
};
const cascadeRegisterRows = register.rows.filter((row) => row.crate === "omena-cascade");
// Removing or duplicating the registry row makes this false.
// The registry producer can emit either state when its canonical publish-train rows drift.
assert.equal(cascadeRegisterRows.length, 1, "omena-cascade must have one public-surface row");
// Weakening the crate to a non-snapshot disposition makes this false.
// The committed register can emit that state, so this is a load-bearing policy assertion.
assert.equal(
  cascadeRegisterRows[0]?.disposition,
  "snapshotGated",
  "omena-cascade must remain snapshot-gated",
);
// Pointing the row at another or missing snapshot makes this false.
// The committed register can emit either state and the public-surface gate consumes this exact path.
assert.equal(
  cascadeRegisterRows[0]?.snapshot,
  snapshotRelativePath,
  "omena-cascade must retain its canonical public API snapshot",
);

const snapshotSource = readFileSync(snapshotPath, "utf8");
const publicRankMethods = snapshotSource
  .split(/\r?\n/u)
  .filter((line) => line.includes("omena_cascade::LayerRank::"));
const publicRankProducers = snapshotSource
  .split(/\r?\n/u)
  .filter((line) => line.endsWith("-> omena_cascade::LayerRank"));
// Widening the tuple field changes the generated snapshot marker and makes this false.
// The public API generator can emit the widened marker; the sibling surface gate proves freshness.
assert.ok(
  snapshotSource.includes("pub struct omena_cascade::LayerRank(_)"),
  "LayerRank public API must expose an opaque field",
);
// Adding an associated public constructor or mutator makes this false.
// The public API generator can emit that method, and the sibling surface gate rejects stale snapshots.
assert.deepEqual(
  publicRankMethods,
  ["pub const fn omena_cascade::LayerRank::get(self) -> i32"],
  "LayerRank must expose only its scalar observer",
);
// Adding another public free function that mints LayerRank makes this false.
// The public API generator can emit that function, and the sibling surface gate rejects stale snapshots.
assert.deepEqual(
  publicRankProducers,
  [
    "pub const fn omena_cascade::normalized_layer_rank(bool, core::option::Option<omena_cascade::LayerOrdinal>) -> omena_cascade::LayerRank",
  ],
  "normalized_layer_rank must remain the only public rank producer",
);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-cascade.layer-rank-construction-check",
      occurrenceCount: rows.length,
      normalizerBodyCount: normalizerRows.length,
      fixtureBuilderCount: fixtureRows.length,
      unclassifiedCount: unclassifiedRows.length,
      publicRankProducerCount: publicRankProducers.length,
    },
    null,
    2,
  )}\n`,
);

function collectRustSources(root: string): string[] {
  const result: string[] = [];
  for (const entry of evidenceScanSurface.readdirSync(root, { withFileTypes: true })) {
    const absolutePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      result.push(...collectRustSources(absolutePath));
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".rs")) {
      result.push(absolutePath);
    }
  }
  return result.toSorted();
}

function scanConstructionRows(absolutePaths: readonly string[]): ConstructionRow[] {
  const result: ConstructionRow[] = [];
  let occurrenceOrdinal = 0;
  for (const absolutePath of absolutePaths) {
    const sourcePath = path.relative(repoRoot, absolutePath);
    const source = readFileSync(absolutePath, "utf8");
    const normalizerRange =
      sourcePath === "rust/crates/omena-cascade/src/model.rs"
        ? namedFunctionLineRange(source, "normalized_layer_rank")
        : undefined;
    for (const [lineIndex, line] of source.split(/\r?\n/u).entries()) {
      for (const _match of line.matchAll(/\bLayerRank\s*\(/gu)) {
        occurrenceOrdinal += 1;
        const trimmed = line.trim();
        let partition: ConstructionPartition = "unclassified";
        let reason = "No construction policy covers this occurrence.";
        if (trimmed === "pub struct LayerRank(i32);") {
          partition = "declaration-shape";
          reason = "The exact declaration pins a public type with a private scalar field.";
        } else if (
          normalizerRange &&
          normalizerRange.startLine <= lineIndex + 1 &&
          lineIndex + 1 <= normalizerRange.endLine
        ) {
          partition = "normalizer-body";
          reason = "This tuple construction belongs to an exhaustive normalization match arm.";
        } else if (fixtureSourcePaths.has(sourcePath)) {
          partition = "fixture-builder";
          reason = "This source is an explicitly classified in-crate fixture producer.";
        }
        result.push({
          sourcePath,
          occurrenceOrdinal,
          partition,
          source: trimmed,
          reason,
        });
      }
    }
  }
  return result;
}

function namedFunctionLineRange(
  source: string,
  functionName: string,
): { readonly startLine: number; readonly endLine: number } {
  const lines = source.split(/\r?\n/u);
  const startIndex = lines.findIndex((line) =>
    new RegExp(`\\bfn\\s+${functionName}\\b`, "u").test(line),
  );
  if (startIndex < 0) {
    throw new Error(`missing function ${functionName}`);
  }
  let depth = 0;
  let opened = false;
  for (let lineIndex = startIndex; lineIndex < lines.length; lineIndex += 1) {
    for (const character of lines[lineIndex] ?? "") {
      if (character === "{") {
        depth += 1;
        opened = true;
      } else if (character === "}") {
        depth -= 1;
      }
    }
    if (opened && depth === 0) {
      return { startLine: startIndex + 1, endLine: lineIndex + 1 };
    }
  }
  throw new Error(`unterminated function ${functionName}`);
}
