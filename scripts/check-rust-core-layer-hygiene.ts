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
const rustCratesDirPath = "rust/crates";
const godFileCeilingsPath = "scripts/rust-godfile-ceilings.json";
const transformIrSpanShimBaselinePath = "scripts/transform-ir-span-shim-baseline.json";
const transformIrSpanShimSourceDirPath = "rust/crates/omena-transform-passes/src";
const retiringEngineStyleParserPath = "rust/crates/engine-style-parser/src/lib.rs";
const retiringEngineStyleParserManifestPath = "rust/crates/engine-style-parser/Cargo.toml";

const parserLib = read(parserLibPath);
const parserParse = read(parserParsePath);
const cliMain = read(cliMainPath);
const godFileCeilings = readGodFileCeilings(godFileCeilingsPath);
const transformIrSpanShimBaseline = readTransformIrSpanShimBaseline(
  transformIrSpanShimBaselinePath,
);

assertLineBudget(parserLibPath, parserLib, 220);
assertLineBudget(cliMainPath, cliMain, 120);
const godFileRatchetTargets = assertGodFileCeilings(godFileCeilings);
const transformIrSpanShim = assertTransformIrSpanShimBaseline(transformIrSpanShimBaseline);
assertRetiringEngineStyleParserExcluded(godFileCeilings);
const lintInheritance = assertWorkspaceLintInheritance();
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
  assert.ok(
    parserLib.includes(snippet),
    `${parserLibPath} must keep parser facade snippet: ${snippet}`,
  );
}

for (const forbidden of [
  "struct Parser<'text>",
  "fn parse_stylesheet_items",
  "fn parse_rule_list_items",
]) {
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
  assert.ok(
    !cliMain.includes(forbidden),
    `${cliMainPath} must not contain command implementation: ${forbidden}`,
  );
}
for (const moduleName of [
  "build",
  "commands",
  "config",
  "diagnostics",
  "dispatch",
  "facts",
  "io",
  "lock",
  "output",
  "paths",
  "perceptual",
  "product_verb",
  "provenance",
  "query",
  "reports",
  "sif",
]) {
  assert.ok(
    cliMain.includes(`mod ${moduleName};`),
    `${cliMainPath} must declare CLI split module ${moduleName}`,
  );
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
  "shared",
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

const queryDiagnosticFamilyModules = new Set([
  "cascade_runtime",
  "cross_file_scc",
  "css_modules",
  "replica_ensemble",
  "sass",
  "sass_resolution",
  "single_file",
  "source_usage",
]);
const queryDiagnosticInfrastructureModules = new Set(["render", "shared", "substrate", "types"]);
const queryDiagnosticSupportModules = new Set(["external_sif", "sass_builtins", "sass_symbols"]);
const queryDiagnosticLeafModules = [
  ...queryDiagnosticFamilyModules,
  ...queryDiagnosticSupportModules,
  "substrate",
];
const processGlobalTestAtomicCounters = assertNoProcessGlobalTestAtomicCounters();
const wildcardImportViolations: string[] = [];
const disallowedFamilyImportViolations: string[] = [];
for (const moduleName of queryDiagnosticLeafModules) {
  const relativePath = `${queryDiagnosticsDirPath}/${moduleName}.rs`;
  const source = read(relativePath);
  const lines = source.split("\n");
  lines.forEach((line, index) => {
    if (/^\s*use\s+super::\*\s*;/u.test(line)) {
      wildcardImportViolations.push(`${relativePath}:${index + 1}: ${line.trim()}`);
    }
    const match = /^\s*use\s+super::([a-z_][a-z0-9_]*)\b/u.exec(line);
    if (!match) {
      return;
    }
    const importedModule = match[1];
    if (!queryModules.has(importedModule)) {
      return;
    }
    const allowedImport =
      queryDiagnosticInfrastructureModules.has(importedModule) ||
      queryDiagnosticSupportModules.has(importedModule);
    if (!allowedImport && queryDiagnosticFamilyModules.has(importedModule)) {
      disallowedFamilyImportViolations.push(
        `${relativePath}:${index + 1}: ${moduleName} imports diagnostic family ${importedModule}`,
      );
    }
  });
}
assert.equal(
  wildcardImportViolations.length,
  0,
  `diagnostic family/support modules must not use wildcard parent imports:\n${wildcardImportViolations.join("\n")}`,
);
assert.equal(
  disallowedFamilyImportViolations.length,
  0,
  `diagnostic family modules must not import other diagnostic family modules directly:\n${disallowedFamilyImportViolations.join("\n")}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.core-layer-hygiene",
      parserLibLines: lineCount(parserLib),
      cliMainLines: lineCount(cliMain),
      godFileRatchetTargets,
      transformIrSpanShim,
      workspaceLintInheritance: lintInheritance,
      queryDiagnosticsModules: queryModules.size,
      maxQueryDiagnosticFamilyLines: Math.max(...queryModuleLines.map(({ lines }) => lines)),
      processGlobalTestAtomicCounters,
      queryDiagnosticsWildcardImports: wildcardImportViolations.length,
      queryDiagnosticsDisallowedFamilyImports: disallowedFamilyImportViolations.length,
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

interface GodFileCeilingManifest {
  readonly schemaVersion: string;
  readonly ceilings: Record<string, number>;
}

interface GodFileRatchetTarget {
  readonly path: string;
  readonly lines: number;
  readonly ceiling: number;
}

interface TransformIrSpanShimBaselineManifest {
  readonly schemaVersion: string;
  readonly product: string;
  readonly count: number;
  readonly expiry: {
    readonly kind: string;
    readonly mechanism: string;
    readonly notAfterUtcDate: string;
  };
}

interface TransformIrSpanShimSummary {
  readonly scannedFiles: number;
  readonly count: number;
  readonly baseline: number;
  readonly expiry: TransformIrSpanShimBaselineManifest["expiry"];
}

interface WorkspaceLintInheritanceSummary {
  readonly crates: number;
  readonly inherited: number;
  readonly missing: readonly string[];
}

interface ProcessGlobalTestAtomicCounterSummary {
  readonly scannedFiles: number;
  readonly processGlobalAtomicCounters: number;
}

function readGodFileCeilings(relativePath: string): ReadonlyMap<string, number> {
  const manifest = JSON.parse(read(relativePath)) as GodFileCeilingManifest;
  assert.equal(manifest.schemaVersion, "0", `${relativePath} schemaVersion must be 0`);
  assert.ok(
    manifest.ceilings && typeof manifest.ceilings === "object" && !Array.isArray(manifest.ceilings),
    `${relativePath} must contain a ceilings object`,
  );

  const entries = Object.entries(manifest.ceilings).toSorted(([left], [right]) =>
    left.localeCompare(right),
  );
  assert.ok(entries.length > 0, `${relativePath} must contain at least one ratchet target`);

  const ceilings = new Map<string, number>();
  for (const [targetPath, ceiling] of entries) {
    assert.ok(
      targetPath.startsWith("rust/crates/") && targetPath.endsWith(".rs"),
      `${relativePath} target must be a Rust crate source path: ${targetPath}`,
    );
    assert.ok(Number.isInteger(ceiling) && ceiling > 0, `${targetPath} ceiling must be positive`);
    assert.ok(existsSync(path.join(root, targetPath)), `${targetPath} ratchet target must exist`);
    ceilings.set(targetPath, ceiling);
  }
  return ceilings;
}

function assertGodFileCeilings(
  ceilings: ReadonlyMap<string, number>,
): readonly GodFileRatchetTarget[] {
  const targets: GodFileRatchetTarget[] = [];
  for (const [targetPath, ceiling] of ceilings) {
    const lines = lineCount(read(targetPath));
    assert.ok(
      lines <= ceiling,
      `${targetPath} has ${lines} LOC; expected <= ratchet ceiling ${ceiling}`,
    );
    targets.push({ path: targetPath, lines, ceiling });
  }
  return targets;
}

function readTransformIrSpanShimBaseline(
  relativePath: string,
): TransformIrSpanShimBaselineManifest {
  const manifest = JSON.parse(read(relativePath)) as TransformIrSpanShimBaselineManifest;
  assert.equal(manifest.schemaVersion, "0", `${relativePath} schemaVersion must be 0`);
  assert.equal(
    manifest.product,
    "rust.transform-ir-span-shim-baseline",
    `${relativePath} product must identify the transform IR span shim baseline`,
  );
  assert.ok(Number.isInteger(manifest.count), `${relativePath} count must be an integer`);
  assert.ok(manifest.count > 0, `${relativePath} count must be non-vacuous`);
  assert.ok(manifest.expiry && typeof manifest.expiry === "object", `${relativePath} needs expiry`);
  assert.equal(
    manifest.expiry.kind,
    "trackedRetirement",
    `${relativePath} expiry kind must be trackedRetirement`,
  );
  assert.equal(
    manifest.expiry.mechanism,
    "transform-ir-span-replacement-shim",
    `${relativePath} expiry mechanism must name the span replacement shim`,
  );
  assert.ok(
    /^\d{4}-\d{2}-\d{2}$/u.test(manifest.expiry.notAfterUtcDate),
    `${relativePath} expiry must carry an ISO UTC date`,
  );
  return manifest;
}

function assertTransformIrSpanShimBaseline(
  baseline: TransformIrSpanShimBaselineManifest,
): TransformIrSpanShimSummary {
  const scannedFiles = rustFilesUnder(transformIrSpanShimSourceDirPath);
  let count = 0;
  for (const relativePath of scannedFiles) {
    const lines = read(relativePath).split("\n");
    for (const line of lines) {
      if (
        /\bTransformIrSourceReplacementV0\s*\{/u.test(line) &&
        !/\bstruct\s+TransformIrSourceReplacementV0\b/u.test(line)
      ) {
        count += 1;
      }
    }
  }

  assert.equal(
    count,
    baseline.count,
    `TransformIrSourceReplacementV0 construction-site count ${count} must match ${transformIrSpanShimBaselinePath} baseline ${baseline.count}; lower both in the same change when retiring span-keyed sites`,
  );

  return {
    scannedFiles: scannedFiles.length,
    count,
    baseline: baseline.count,
    expiry: baseline.expiry,
  };
}

function assertRetiringEngineStyleParserExcluded(ceilings: ReadonlyMap<string, number>): void {
  assert.ok(
    !ceilings.has(retiringEngineStyleParserPath),
    `${retiringEngineStyleParserPath} is a retiring legacy oracle and must not be a god-file ratchet target`,
  );

  const manifest = read(retiringEngineStyleParserManifestPath);
  assert.ok(
    /\[package\.metadata\.omena\][\s\S]*?\brole\s*=\s*"I"/u.test(manifest),
    `${retiringEngineStyleParserManifestPath} must keep role = "I" for the ratchet exclusion`,
  );
  assert.ok(
    /^\s*publish\s*=\s*false\s*$/mu.test(manifest),
    `${retiringEngineStyleParserManifestPath} must keep publish = false for the ratchet exclusion`,
  );
}

function assertWorkspaceLintInheritance(): WorkspaceLintInheritanceSummary {
  const workspaceManifest = read("rust/Cargo.toml");
  assert.ok(
    workspaceManifest.includes("[workspace.lints.rust]"),
    "rust/Cargo.toml must declare [workspace.lints.rust]",
  );
  assert.ok(
    workspaceManifest.includes("[workspace.lints.clippy]"),
    "rust/Cargo.toml must declare [workspace.lints.clippy]",
  );

  const crateManifestPaths = readdirSync(path.join(root, rustCratesDirPath))
    .filter((entry) => statSync(path.join(root, rustCratesDirPath, entry)).isDirectory())
    .map((entry) => `${rustCratesDirPath}/${entry}/Cargo.toml`)
    .filter((relativePath) => existsSync(path.join(root, relativePath)))
    .toSorted();

  const missing: string[] = [];
  for (const manifestPath of crateManifestPaths) {
    const lintEntries = lintSectionEntries(read(manifestPath));
    if (lintEntries.length !== 1 || lintEntries[0] !== "workspace = true") {
      missing.push(manifestPath);
    }
  }

  assert.equal(
    missing.length,
    0,
    `workspace lint inheritance is incomplete; every crate must contain [lints] workspace = true:\n${missing.join("\n")}`,
  );

  return {
    crates: crateManifestPaths.length,
    inherited: crateManifestPaths.length - missing.length,
    missing,
  };
}

function assertNoProcessGlobalTestAtomicCounters(): ProcessGlobalTestAtomicCounterSummary {
  const scopedOutProductionEpochs = new Set([
    "rust/crates/omena-resolver/src/style_resolution.rs:static STYLE_IDENTITY_CACHE_VERSION",
    "rust/crates/omena-lsp-server/src/protocol.rs:static CANONICALIZE_PATH_CACHE_VERSION",
  ]);
  const counterCrates = [
    "rust/crates/omena-resolver/src",
    "rust/crates/omena-incremental/src",
    "rust/crates/omena-parser/src",
    "rust/crates/omena-lsp-server/src",
  ];
  const scannedFiles = counterCrates.flatMap((cratePath) => rustFilesUnder(cratePath));
  const violations: string[] = [];
  for (const relativePath of scannedFiles) {
    if (relativePath === "rust/crates/omena-testkit/src/instrumentation_session.rs") {
      continue;
    }
    const lines = read(relativePath).split("\n");
    lines.forEach((line, index) => {
      const match = /^(static\s+[A-Z0-9_]+):\s*Atomic(?:Usize|U64|U32|Bool)\b/u.exec(line);
      if (!match) {
        return;
      }
      const key = `${relativePath}:${match[1]}`;
      if (!scopedOutProductionEpochs.has(key)) {
        violations.push(`${relativePath}:${index + 1}: ${line.trim()}`);
      }
    });
  }
  assert.equal(
    violations.length,
    0,
    `test-support counters must use InstrumentationSessionV0 instead of process-global Atomics:\n${violations.join("\n")}`,
  );
  return {
    scannedFiles: scannedFiles.length,
    processGlobalAtomicCounters: violations.length,
  };
}

function rustFilesUnder(relativeDir: string): readonly string[] {
  const absoluteDir = path.join(root, relativeDir);
  const entries = readdirSync(absoluteDir).toSorted();
  const files: string[] = [];
  for (const entry of entries) {
    const absoluteEntry = path.join(absoluteDir, entry);
    const relativeEntry = `${relativeDir}/${entry}`;
    const stat = statSync(absoluteEntry);
    if (stat.isDirectory()) {
      files.push(...rustFilesUnder(relativeEntry));
    } else if (entry.endsWith(".rs")) {
      files.push(relativeEntry);
    }
  }
  return files;
}

function lintSectionEntries(source: string): string[] {
  const entries: string[] = [];
  let inLintSection = false;
  for (const line of source.split("\n")) {
    const trimmed = line.trim();
    if (/^\[.*\]$/u.test(trimmed)) {
      inLintSection = trimmed === "[lints]";
      continue;
    }
    if (!inLintSection || trimmed === "" || trimmed.startsWith("#")) {
      continue;
    }
    entries.push(trimmed.replace(/\s*#.*$/u, ""));
  }
  return entries;
}
