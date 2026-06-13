import { strict as assert } from "node:assert";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

/**
 * rust/core-layer-hygiene
 *
 * Guards the structural split of the high-risk Rust entry files. Behavioral
 * compatibility is covered by crate tests and boundary gates; this check keeps
 * the facade and diagnostic-family layout from silently regressing into a
 * single-file implementation.
 */

const root = process.cwd();

const parserLibPath = "rust/crates/omena-parser/src/lib.rs";
const parserParsePath = "rust/crates/omena-parser/src/parse.rs";
const cliMainPath = "rust/crates/omena-cli/src/main.rs";
const queryDiagnosticsFilePath = "rust/crates/omena-query/src/style/diagnostics.rs";
const queryDiagnosticsDirPath = "rust/crates/omena-query/src/style/diagnostics";

const parserLib = read(parserLibPath);
const parserParse = read(parserParsePath);
const cliMain = read(cliMainPath);

assertLineBudget(parserLibPath, parserLib, 220);
assertLineBudget(cliMainPath, cliMain, 120);
assert.ok(
  !existsSync(path.join(root, queryDiagnosticsFilePath)),
  `${queryDiagnosticsFilePath} must not be recreated; use diagnostics/mod.rs and family modules`,
);
assert.ok(
  statSync(path.join(root, queryDiagnosticsDirPath)).isDirectory(),
  `${queryDiagnosticsDirPath} must remain a directory-backed module`,
);

for (const moduleName of [
  "cst",
  "extension",
  "facts",
  "language",
  "lex",
  "parse",
  "public_product",
  "recovery",
  "spans",
  "summaries",
  "syntax_helpers",
  "value_names",
]) {
  assert.ok(
    parserLib.includes(`mod ${moduleName};`),
    `${parserLibPath} must declare parser split module ${moduleName}`,
  );
}

for (const snippet of [
  "pub use parse::{",
  "collect_style_facts",
  "parse_entry_point_with_extension",
  "pub(crate) use parse::{Parser, tokenize};",
]) {
  assert.ok(parserLib.includes(snippet), `${parserLibPath} must keep parser facade snippet: ${snippet}`);
}

for (const forbidden of ["struct Parser<'text>", "fn parse_stylesheet_items", "fn parse_rule_list_items"]) {
  assert.ok(
    !parserLib.includes(forbidden),
    `${parserLibPath} must not contain parser engine implementation: ${forbidden}`,
  );
}
assert.ok(
  parserParse.includes("pub(crate) struct Parser<'text>"),
  `${parserParsePath} must own the parser engine`,
);

for (const forbidden of ["enum Command", "struct Cli", "fn build_file", "fn style_diagnostics"]) {
  assert.ok(!cliMain.includes(forbidden), `${cliMainPath} must not contain command implementation: ${forbidden}`);
}
for (const moduleName of [
  "build",
  "check",
  "commands",
  "diagnostics",
  "dispatch",
  "io",
  "lock",
  "output",
  "paths",
  "perceptual",
  "provenance",
  "query",
  "reports",
  "sif",
]) {
  assert.ok(cliMain.includes(`mod ${moduleName};`), `${cliMainPath} must declare CLI split module ${moduleName}`);
}

const queryModules = new Set(
  readdirSync(path.join(root, queryDiagnosticsDirPath))
    .filter((entry) => entry.endsWith(".rs"))
    .map((entry) => entry.replace(/\.rs$/u, "")),
);
for (const moduleName of [
  "cascade_runtime",
  "css_modules",
  "external_sif",
  "render",
  "sass",
  "sass_resolution",
  "single_file",
  "source_usage",
  "substrate",
  "types",
]) {
  assert.ok(
    queryModules.has(moduleName),
    `${queryDiagnosticsDirPath} must contain diagnostics family module ${moduleName}.rs`,
  );
}

const queryModuleLines = [...queryModules]
  .map((moduleName) => {
    const relativePath = `${queryDiagnosticsDirPath}/${moduleName}.rs`;
    return { moduleName, relativePath, lines: lineCount(read(relativePath)) };
  })
  .filter(({ moduleName }) => moduleName !== "mod");
const oversizedQueryModules = queryModuleLines.filter(({ lines }) => lines > 1_000);
assert.equal(
  oversizedQueryModules.length,
  0,
  `diagnostic family modules must stay below 1000 LOC:\n${oversizedQueryModules
    .map(({ relativePath, lines }) => `  ${relativePath}: ${lines}`)
    .join("\n")}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.core-layer-hygiene",
      parserLibLines: lineCount(parserLib),
      cliMainLines: lineCount(cliMain),
      queryDiagnosticsModules: queryModules.size,
      maxQueryDiagnosticFamilyLines: Math.max(...queryModuleLines.map(({ lines }) => lines)),
      violations: 0,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function lineCount(source: string): number {
  return source.length === 0 ? 0 : source.split("\n").length;
}

function assertLineBudget(relativePath: string, source: string, maxLines: number): void {
  const lines = lineCount(source);
  assert.ok(lines <= maxLines, `${relativePath} has ${lines} LOC; expected <= ${maxLines}`);
}
