import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { parse as postcssParse, type Root } from "postcss";
import lessSyntax from "postcss-less";
import scssSyntax from "postcss-scss";

import { formatGeneratedJson } from "./generated-json";

type Dialect = "css" | "scss" | "less";
type AdjudicationKind = "omenaMatcherDefect" | "cssTreeDefect" | "grammarSourceDivergence";

interface FarmManifestV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.oss-corpus-farm.manifest";
  readonly fixtures: readonly FarmEntryV0[];
}

interface FarmEntryV0 {
  readonly dialect: Dialect;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
  };
}

interface WebrefGrammarV0 {
  readonly categories: {
    readonly properties: readonly { readonly name: string }[];
  };
}

interface CorpusCaseV0 {
  readonly id: string;
  readonly property: string;
  readonly value: string;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly path: string;
    readonly line: number;
    readonly dialect: Dialect;
  };
  readonly adjudication?: AdjudicationKind;
  readonly reason?: string;
  readonly owner?: string;
  readonly notComparableReason?: string;
}

interface CorpusV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-abstract-value.value-grammar-real-declarations";
  readonly generatedBy: "scripts/generate-rust-omena-value-grammar-corpus.ts";
  readonly sourceManifest: "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json";
  readonly maxCaseCount: number;
  readonly scannedFileCount: number;
  readonly harvestedDeclarationCount: number;
  readonly uniqueDeclarationCount: number;
  readonly caseCount: number;
  readonly sourcePins: readonly {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
  }[];
  readonly cases: readonly CorpusCaseV0[];
}

type HarvestedDeclaration = Omit<CorpusCaseV0, "id">;

const repoRoot = process.cwd();
const manifestPath = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json",
);
const registryPath = path.join(repoRoot, "rust/crates/omena-spec-audit/data/webref-grammar.json");
const outputPath = path.join(
  repoRoot,
  "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-real-declarations.json",
);
const MAX_CASE_COUNT = 192;
const MAX_VALUE_BYTES = 512;

void main();

async function main(): Promise<void> {
  const manifest = readJson<FarmManifestV0>(manifestPath);
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.oss-corpus-farm.manifest");
  assert.ok(manifest.fixtures.length > 0);
  const standardProperties = new Set(
    readJson<WebrefGrammarV0>(registryPath).categories.properties.map((entry) => entry.name),
  );
  const previous = existsSync(outputPath) ? readJson<CorpusV0>(outputPath) : undefined;
  const previousById = new Map(previous?.cases.map((entry) => [entry.id, entry]) ?? []);
  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-value-grammar-corpus-"));
  let scannedFileCount = 0;
  let harvestedDeclarationCount = 0;
  const harvested: HarvestedDeclaration[] = [];

  try {
    for (const group of sourceGroups(manifest.fixtures)) {
      const checkoutDir = path.join(
        tempRoot,
        sha256(`${group.repository}\0${group.pin}`).slice(0, 16),
      );
      checkoutPinnedSource(group, checkoutDir);
      for (const fixture of group.fixtures) {
        for (const relativePath of selectedStyleFiles(checkoutDir, fixture)) {
          scannedFileCount += 1;
          const absolutePath = path.join(checkoutDir, relativePath);
          const source = readFileSync(absolutePath, "utf8");
          const root = parseStyleSource(source, absolutePath, fixture.dialect);
          root.walkDecls((declaration) => {
            const property = declaration.prop.trim().toLowerCase();
            const value = declaration.value.trim();
            if (!standardProperties.has(property) || value.length === 0) return;
            if (Buffer.byteLength(value) > MAX_VALUE_BYTES) return;
            harvestedDeclarationCount += 1;
            harvested.push({
              property,
              value,
              source: {
                repository: group.repository,
                pin: group.pin,
                path: relativePath,
                line: declaration.source?.start?.line ?? 0,
                dialect: fixture.dialect,
              },
            });
          });
        }
      }
    }

    const unique = deduplicateDeclarations(harvested);
    const selected = roundRobinByProperty(unique, MAX_CASE_COUNT).map((entry): CorpusCaseV0 => {
      const id = `oss-${sha256(`${entry.property}\0${entry.value}`).slice(0, 20)}`;
      const previousEntry = previousById.get(id);
      const sameTuple =
        previousEntry?.property === entry.property && previousEntry.value === entry.value;
      return {
        id,
        property: entry.property,
        value: entry.value,
        source: entry.source,
        ...(sameTuple && previousEntry?.adjudication
          ? { adjudication: previousEntry.adjudication }
          : {}),
        ...(sameTuple && previousEntry?.reason ? { reason: previousEntry.reason } : {}),
        ...(sameTuple && previousEntry?.owner ? { owner: previousEntry.owner } : {}),
        ...(sameTuple && previousEntry?.notComparableReason
          ? { notComparableReason: previousEntry.notComparableReason }
          : {}),
      };
    });
    const selectedIds = new Set(selected.map((entry) => entry.id));
    for (const entry of previous?.cases ?? []) {
      if (entry.adjudication || entry.notComparableReason) {
        assert.ok(selectedIds.has(entry.id), `reviewed corpus row disappeared: ${entry.id}`);
      }
    }

    const corpus: CorpusV0 = {
      schemaVersion: "0",
      product: "omena-abstract-value.value-grammar-real-declarations",
      generatedBy: "scripts/generate-rust-omena-value-grammar-corpus.ts",
      sourceManifest: "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json",
      maxCaseCount: MAX_CASE_COUNT,
      scannedFileCount,
      harvestedDeclarationCount,
      uniqueDeclarationCount: unique.length,
      caseCount: selected.length,
      sourcePins: sourceGroups(manifest.fixtures).map((group) => ({
        repository: group.repository,
        pin: group.pin,
        sparsePaths: [
          ...new Set(group.fixtures.flatMap((entry) => entry.source.sparsePaths)),
        ].sort(),
      })),
      cases: selected,
    };
    writeFileSync(outputPath, await formatGeneratedJson(outputPath, corpus));
    process.stdout.write(`${JSON.stringify(corpus, null, 2)}\n`);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function sourceGroups(fixtures: readonly FarmEntryV0[]) {
  const groups = new Map<
    string,
    {
      repository: string;
      pin: string;
      fixtures: FarmEntryV0[];
    }
  >();
  for (const fixture of fixtures) {
    const key = `${fixture.source.repository}\0${fixture.source.pin}`;
    const group = groups.get(key) ?? {
      repository: fixture.source.repository,
      pin: fixture.source.pin,
      fixtures: [],
    };
    group.fixtures.push(fixture);
    groups.set(key, group);
  }
  return [...groups.values()].sort((left, right) =>
    `${left.repository}\0${left.pin}`.localeCompare(`${right.repository}\0${right.pin}`, "en"),
  );
}

function checkoutPinnedSource(
  group: { repository: string; pin: string; fixtures: readonly FarmEntryV0[] },
  checkoutDir: string,
): void {
  const sha = group.pin.split("@").at(-1) ?? "";
  assert.match(sha, /^[0-9a-f]{40}$/u);
  const sparsePaths = [...new Set(group.fixtures.flatMap((entry) => entry.source.sparsePaths))];
  run("git", ["init", "-q", checkoutDir]);
  run("git", ["-C", checkoutDir, "remote", "add", "origin", group.repository]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "init", "--no-cone"]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "set", ...sparsePaths]);
  run("git", ["-C", checkoutDir, "fetch", "--depth", "1", "origin", sha]);
  run("git", ["-C", checkoutDir, "checkout", "-q", "--detach", "FETCH_HEAD"]);
  assert.equal(run("git", ["-C", checkoutDir, "rev-parse", "HEAD"]), sha);
}

function selectedStyleFiles(checkoutDir: string, fixture: FarmEntryV0): string[] {
  const extension = `.${fixture.dialect}`;
  return run("git", ["-C", checkoutDir, "ls-files", "--", ...fixture.source.sparsePaths])
    .split(/\r?\n/u)
    .filter(Boolean)
    .filter((relativePath) => relativePath.toLowerCase().endsWith(extension))
    .filter((relativePath) => statSync(path.join(checkoutDir, relativePath)).isFile())
    .sort((left, right) => left.localeCompare(right, "en"));
}

function parseStyleSource(source: string, filePath: string, dialect: Dialect): Root {
  if (dialect === "scss") return scssSyntax.parse(source, { from: filePath });
  if (dialect === "less") return lessSyntax.parse(source, { from: filePath });
  return postcssParse(source, { from: filePath });
}

function deduplicateDeclarations(entries: readonly HarvestedDeclaration[]): HarvestedDeclaration[] {
  const sorted = [...entries].sort((left, right) => {
    const leftKey = `${left.property}\0${left.value}\0${left.source.repository}\0${left.source.path}\0${String(left.source.line).padStart(8, "0")}`;
    const rightKey = `${right.property}\0${right.value}\0${right.source.repository}\0${right.source.path}\0${String(right.source.line).padStart(8, "0")}`;
    return leftKey.localeCompare(rightKey, "en");
  });
  const byTuple = new Map<string, HarvestedDeclaration>();
  for (const entry of sorted) {
    const key = `${entry.property}\0${entry.value}`;
    if (!byTuple.has(key)) byTuple.set(key, entry);
  }
  return [...byTuple.values()];
}

function roundRobinByProperty(
  entries: readonly HarvestedDeclaration[],
  limit: number,
): HarvestedDeclaration[] {
  const buckets = new Map<string, HarvestedDeclaration[]>();
  for (const entry of entries) {
    const bucket = buckets.get(entry.property) ?? [];
    bucket.push(entry);
    buckets.set(entry.property, bucket);
  }
  const properties = [...buckets.keys()].sort((left, right) => left.localeCompare(right, "en"));
  const selected: HarvestedDeclaration[] = [];
  while (selected.length < limit) {
    let added = false;
    for (const property of properties) {
      const entry = buckets.get(property)?.shift();
      if (!entry) continue;
      selected.push(entry);
      added = true;
      if (selected.length === limit) break;
    }
    if (!added) break;
  }
  return selected;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function run(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\n${result.stderr || result.stdout}`,
  );
  return result.stdout.trim();
}
