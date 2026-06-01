import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

const REPO_ROOT = process.cwd();
const RUST_MANIFEST = path.join(REPO_ROOT, "rust", "Cargo.toml");

const CRATES = [
  "omena-syntax",
  "omena-parser",
  "omena-incremental",
  "omena-semantic",
  "omena-cascade",
  "omena-diff-test",
  "omena-testkit",
  "omena-transform-cst",
  "omena-transform-passes",
  "omena-transform-bundle",
  "omena-transform-target",
  "omena-transform-print",
  "omena-transform-egg",
] as const;

const COVERAGE_THRESHOLD = Number.parseFloat(
  process.env.OMENA_CSS_RUSTDOC_COVERAGE_THRESHOLD ?? "100",
);

interface PublicItem {
  readonly crateName: string;
  readonly relativePath: string;
  readonly lineNumber: number;
  readonly signature: string;
  readonly documented: boolean;
}

function main(): void {
  runCargoDoc();

  const items = CRATES.flatMap((crateName) => collectPublicItems(crateName));
  if (items.length === 0) {
    throw new Error("No public Rust API items were discovered for omena-css rustdoc coverage.");
  }

  const documented = items.filter((item) => item.documented).length;
  const coverage = (documented / items.length) * 100;
  const missing = items.filter((item) => !item.documented);

  for (const crateName of CRATES) {
    const crateItems = items.filter((item) => item.crateName === crateName);
    const crateDocumented = crateItems.filter((item) => item.documented).length;
    const crateCoverage =
      crateItems.length === 0 ? 100 : (crateDocumented / crateItems.length) * 100;
    console.log(
      `${crateName}: ${crateDocumented}/${crateItems.length} public items documented (${crateCoverage.toFixed(
        1,
      )}%)`,
    );
  }

  console.log(
    `omena-css rustdoc coverage: ${documented}/${items.length} public items documented (${coverage.toFixed(
      1,
    )}%), threshold=${COVERAGE_THRESHOLD.toFixed(1)}%`,
  );

  if (coverage < COVERAGE_THRESHOLD) {
    const sample = missing
      .slice(0, 20)
      .map((item) => `  - ${item.relativePath}:${item.lineNumber} ${item.signature}`)
      .join("\n");
    throw new Error(
      `omena-css rustdoc coverage is below threshold.\nMissing documentation sample:\n${sample}`,
    );
  }
}

function runCargoDoc(): void {
  const args = [
    "doc",
    "--manifest-path",
    RUST_MANIFEST,
    "--no-deps",
    ...CRATES.flatMap((crateName) => ["-p", crateName]),
  ];
  const result = spawnSync("cargo", args, {
    cwd: REPO_ROOT,
    env: process.env,
    stdio: "inherit",
  });

  if (result.status !== 0) {
    throw new Error(`cargo ${args.join(" ")} failed with status ${result.status ?? "unknown"}`);
  }
}

function collectPublicItems(crateName: string): PublicItem[] {
  const srcDir = path.join(REPO_ROOT, "rust", "crates", crateName, "src");
  if (!existsSync(srcDir)) {
    throw new Error(`Missing crate source directory: ${srcDir}`);
  }

  return collectRustFiles(srcDir)
    .filter((filePath) => !filePath.endsWith(`${path.sep}tests.rs`))
    .filter((filePath) => !filePath.includes(`${path.sep}bin${path.sep}`))
    .flatMap((filePath) => collectPublicItemsFromFile(crateName, srcDir, filePath));
}

function collectRustFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const filePath = path.join(dir, entry);
    const stats = statSync(filePath);
    if (stats.isDirectory()) return collectRustFiles(filePath);
    if (entry.endsWith(".rs")) return [filePath];
    return [];
  });
}

function collectPublicItemsFromFile(
  crateName: string,
  srcDir: string,
  filePath: string,
): PublicItem[] {
  const source = readFileSync(filePath, "utf8");
  const lines = source.split(/\r?\n/);
  const relativePath = path.relative(REPO_ROOT, filePath);

  return lines.flatMap((line, index) => {
    const signature = publicSignature(line);
    if (!signature) return [];

    return [
      {
        crateName,
        relativePath,
        lineNumber: index + 1,
        signature,
        documented: hasLeadingDoc(lines, index) || hasModuleDoc(lines),
      },
    ];
  });
}

function publicSignature(line: string): string | null {
  const trimmed = line.trim();
  if (!trimmed.startsWith("pub ")) return null;
  if (trimmed.startsWith("pub use ")) return null;
  if (trimmed.startsWith("pub(crate) ")) return null;
  if (trimmed.startsWith("pub(super) ")) return null;
  if (trimmed.startsWith("pub(in ")) return null;

  if (
    /^pub\s+(async\s+)?(struct|enum|fn|trait|type|const|static|mod)\b/.test(trimmed) ||
    /^pub\s+[A-Z_][A-Z0-9_]*\s*:/.test(trimmed)
  ) {
    return trimmed.replace(/\s+/g, " ");
  }

  return null;
}

function hasLeadingDoc(lines: readonly string[], declarationIndex: number): boolean {
  for (let index = declarationIndex - 1; index >= 0; index -= 1) {
    const trimmed = lines[index]?.trim() ?? "";
    if (trimmed.length === 0) continue;
    if (trimmed.startsWith("///") || trimmed.startsWith("/**") || trimmed.startsWith("#[doc")) {
      return true;
    }
    if (trimmed.startsWith("#[")) continue;
    return false;
  }

  return false;
}

function hasModuleDoc(lines: readonly string[]): boolean {
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.length === 0) continue;
    if (trimmed.startsWith("//!") || trimmed.startsWith("/*!")) return true;
    if (trimmed.startsWith("#![")) continue;
    return false;
  }

  return false;
}

main();
