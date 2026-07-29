import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface CargoTarget {
  readonly kind: readonly string[];
  readonly src_path: string;
}

interface CargoPackage {
  readonly name: string;
  readonly description: string | null;
  readonly readme: string | null;
  readonly targets: readonly CargoTarget[];
  readonly manifest_path: string;
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
}

interface ProductPathEntry {
  readonly crate: string;
  readonly role: string;
  readonly surface: string;
  readonly entrypoints: readonly string[];
}

interface ProductPathMatrix {
  readonly entries: readonly ProductPathEntry[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const writeMode = process.argv.includes("--write");
const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
    { cwd: repoRoot, encoding: "utf8" },
  ),
) as CargoMetadata;
const productPathMatrix = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/omena-product-path-matrix.json"), "utf8"),
) as ProductPathMatrix;
const contractReadmeCrates = new Set([
  "omena-cascade-proof",
  "omena-cross-file-summary",
  "omena-cst-typed",
  "omena-evidence-graph",
  "omena-product-hints",
  "omena-query-checker-orchestrator",
  "omena-query-core",
  "omena-query-transform-runner",
  "omena-reactive",
  "omena-refinement-trait",
  "omena-scss-eval",
  "omena-semantic",
  "omena-sif",
  "omena-streaming-ifds",
  "omena-transform-cst",
  "omena-transform-print",
  "omena-value-lattice",
  "omena-zk-audit",
]);

assert.deepEqual(
  productPathMatrix.entries.map(({ crate }) => crate).toSorted(),
  metadata.packages.map(({ name }) => name).toSorted(),
  "the crate catalog must classify every workspace package exactly once",
);

for (const crate of metadata.packages) {
  assert.ok(crate.description?.trim(), `${crate.name} must declare a package description`);
  assert.ok(crate.readme, `${crate.name} must declare package.readme`);
  const crateDir = path.dirname(crate.manifest_path);
  const readmePath = path.resolve(crateDir, crate.readme);
  assert.ok(existsSync(readmePath), `${crate.name} README does not exist: ${readmePath}`);
  const readme = readFileSync(readmePath, "utf8");
  assert.ok(readme.length >= 100, `${crate.name} README must explain the crate's role`);
  assert.ok(
    readme.toLowerCase().includes(crate.name.toLowerCase()),
    `${crate.name} README must name the crate`,
  );
  if (contractReadmeCrates.has(crate.name)) {
    for (const heading of [
      "## Role",
      "## Contract",
      "## Consumers",
      "## Boundaries",
      "## Verification",
    ]) {
      assert.ok(readme.includes(heading), `${crate.name} README must include ${heading}`);
    }
    assert.match(
      readme,
      /cargo test --manifest-path rust\/Cargo\.toml -p [a-z0-9-]+/u,
      `${crate.name} README must include a focused verification command`,
    );
  }

  for (const target of crate.targets.filter((candidate) => candidate.kind.includes("lib"))) {
    const source = readFileSync(target.src_path, "utf8");
    const head = source.split(/\r?\n/).slice(0, 12).join("\n");
    assert.ok(
      /^\/\/!|^#!\[doc/m.test(head),
      `${crate.name} library must begin with crate-level rustdoc`,
    );
  }
}

const catalogPath = path.join(repoRoot, "docs/reference/crates.md");
const catalog = renderCrateCatalog(productPathMatrix.entries, metadata.packages);
if (writeMode) {
  writeFileSync(catalogPath, catalog);
} else {
  assert.ok(existsSync(catalogPath), "docs/reference/crates.md must be generated");
  assert.equal(
    normalizeMarkdownTableLayout(readFileSync(catalogPath, "utf8")),
    normalizeMarkdownTableLayout(catalog),
    "docs/reference/crates.md is stale; regenerate the crate catalog",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "1",
      product: "rust.crate-documentation",
      crateCount: metadata.packages.length,
      readmeCount: metadata.packages.filter((crate) => crate.readme).length,
      libraryTargetCount: metadata.packages.flatMap((crate) =>
        crate.targets.filter((target) => target.kind.includes("lib")),
      ).length,
      contractReadmeCount: contractReadmeCrates.size,
      catalogEntryCount: productPathMatrix.entries.length,
      missingDocumentationCount: 0,
      mode: writeMode ? "write" : "check",
    },
    null,
    2,
  )}\n`,
);

function renderCrateCatalog(
  entries: readonly ProductPathEntry[],
  packages: readonly CargoPackage[],
): string {
  const descriptions = new Map(packages.map((crate) => [crate.name, crate.description ?? ""]));
  const rows = entries
    .toSorted((left, right) => left.crate.localeCompare(right.crate))
    .map((entry) => {
      const description = descriptions.get(entry.crate);
      assert.ok(description, `${entry.crate} is missing its catalog description`);
      return `| [\`${entry.crate}\`](../../rust/crates/${entry.crate}/README.md) | ${surfaceLabel(
        entry.surface,
      )} | \`${entry.role}\` | ${escapeTableCell(entry.entrypoints[0] ?? description)} |`;
    });
  return `---
title: Rust crate catalog
description: Generated ownership and product-reachability index for every Rust workspace crate.
kind: reference
status: stable
products: [rust, architecture]
owner: architecture
sourceOfTruth: generated
---

<!-- Generated from product code. Do not edit by hand. -->

# Rust crate catalog

The catalog classifies every workspace crate without presenting each substrate
as a standalone product. Product guides document CLI, LSP, query, NAPI, WASM,
and bundler facades; crate READMEs document lower-level contracts.

| Crate | Surface | Layer role | Responsibility |
| --- | --- | --- | --- |
${rows.join("\n")}
`;
}

function surfaceLabel(surface: string): string {
  const labels: Readonly<Record<string, string>> = {
    "check-evidence": "Check and evidence",
    "cli-runtime": "CLI runtime",
    "legacy-oracle": "Legacy oracle",
    "product-runtime": "Product runtime",
    "research-fixture": "Research and fixture",
    support: "Support",
    umbrella: "Umbrella",
  };
  return labels[surface] ?? surface;
}

function escapeTableCell(value: string): string {
  return value.replaceAll("|", "\\|").replace(/\s+/gu, " ").trim();
}

function normalizeMarkdownTableLayout(source: string): string {
  return source
    .split("\n")
    .map((line) => {
      const trimmed = line.trim();
      if (!trimmed.startsWith("|") || !trimmed.endsWith("|")) return line;

      const cells: string[] = [];
      let cell = "";
      for (let index = 1; index < trimmed.length; index += 1) {
        const character = trimmed[index];
        if (character === "|" && trimmed[index - 1] !== "\\") {
          cells.push(cell.trim());
          cell = "";
        } else {
          cell += character;
        }
      }

      const separatorRow = cells.every((value) => /^:?-{3,}:?$/u.test(value));
      return `|${cells
        .map((value) => (separatorRow ? value.replace(/^(:?)-{3,}(:?)$/u, "$1---$2") : value))
        .join("|")}|`;
    })
    .join("\n");
}
