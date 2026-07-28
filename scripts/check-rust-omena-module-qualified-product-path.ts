import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface SymbolPopulation {
  readonly totalOccurrences: number;
  readonly definitionOccurrences: number;
  readonly reexportOccurrences: number;
  readonly testOccurrences: number;
  readonly otherProductionOccurrences: number;
  readonly productOccurrencesByCrate: Readonly<Record<string, number>>;
}

interface QualifiedExecutorPopulation extends SymbolPopulation {
  readonly symbols: readonly string[];
  readonly productCallers: readonly {
    readonly crateName: string;
    readonly sourcePath: string;
    readonly symbol: string;
  }[];
}

interface ProductPathPopulation {
  readonly qualifiedExecutor: QualifiedExecutorPopulation;
  readonly moduleQualifiedSymbols: SymbolPopulation;
  readonly moduleQualifiedShake: SymbolPopulation;
}

interface ModuleQualifiedProductPathCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.module-qualified-product-path-census";
  readonly policyDigest: string;
  readonly productCrates: readonly string[];
  readonly entryPin: {
    readonly testedCommit: string;
    readonly population: ProductPathPopulation;
  };
  readonly current: ProductPathPopulation;
}

interface Occurrence {
  readonly sourcePath: string;
  readonly line: number;
  readonly text: string;
  readonly testOnly: boolean;
}

interface RustSourceFile {
  readonly sourcePath: string;
  readonly source: string;
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-module-qualified-product-path-census.json");
const writeMode = process.argv.includes("--write");
const injectMissingExecutor = process.argv.includes("--inject-missing-qualified-executor");
const productCrates = [
  "omena-query",
  "omena-cli",
  "omena-napi",
  "omena-wasm",
  "omena-lsp-server",
] as const;
const qualifiedExecutorSymbols = [
  "execute_transform_passes_on_module_with_dialect_context_and_closed_world_bundle",
  "execute_transform_passes_on_module_with_dialect_context_policy_and_closed_world_bundle",
  "execute_transform_passes_on_module_with_dialect_context_policy_and_closed_world_bundle_and_retained_class_names",
] as const;
const classificationPolicy = {
  version: 1,
  rustRoot: "rust",
  productCrates,
  qualifiedExecutorSymbols,
  testClassification: ["tests-directory", "cfg-test-module"],
  definitionClassification: "pub-fn-declaration",
  reexportClassification: "non-call-symbol-in-rust-lib",
};
const policyDigest = createHash("sha256")
  .update(JSON.stringify(classificationPolicy))
  .digest("hex");
const sourceFiles = rustSourceFiles(path.join(repoRoot, "rust"));

const qualifiedOccurrences = qualifiedExecutorSymbols.flatMap((symbol) =>
  collectOccurrences(sourceFiles, symbol),
);
if (injectMissingExecutor) {
  const definitionIndex = qualifiedOccurrences.findIndex((occurrence) =>
    occurrence.text.includes("pub fn execute_transform_passes_on_module_"),
  );
  assert.notEqual(definitionIndex, -1, "qualified executor definition injection target is absent");
  qualifiedOccurrences.splice(definitionIndex, 1);
}

const current: ProductPathPopulation = {
  qualifiedExecutor: classifyQualifiedExecutors(qualifiedOccurrences, qualifiedExecutorSymbols),
  moduleQualifiedSymbols: classifySymbol(
    collectOccurrences(sourceFiles, "module_qualified_symbols"),
    "module_qualified_symbols",
  ),
  moduleQualifiedShake: classifySymbol(
    collectOccurrences(sourceFiles, "module_qualified_shake"),
    "module_qualified_shake",
  ),
};

// FALSIFIER: id=module-qualified-gate-001 class=liveness via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(
  current.qualifiedExecutor.definitionOccurrences >= 1,
  "the module-qualified executor family must retain a production definition",
);
assert.equal(
  Object.keys(current.qualifiedExecutor.productOccurrencesByCrate).length,
  productCrates.length,
  "every product crate must remain represented in the caller census",
);

const existing = fs.existsSync(censusPath)
  ? (JSON.parse(fs.readFileSync(censusPath, "utf8")) as ModuleQualifiedProductPathCensus)
  : undefined;
if (existing) {
  const entrySymbols = existing.entryPin.population.qualifiedExecutor.symbols;
  const entrySourceFiles = gitRustSourceFiles(existing.entryPin.testedCommit, [
    ...entrySymbols,
    "module_qualified_symbols",
    "module_qualified_shake",
  ]);
  const independentlyDerivedEntryPopulation: ProductPathPopulation = {
    qualifiedExecutor: classifyQualifiedExecutors(
      entrySymbols.flatMap((symbol) => collectOccurrences(entrySourceFiles, symbol)),
      entrySymbols,
    ),
    moduleQualifiedSymbols: classifySymbol(
      collectOccurrences(entrySourceFiles, "module_qualified_symbols"),
      "module_qualified_symbols",
    ),
    moduleQualifiedShake: classifySymbol(
      collectOccurrences(entrySourceFiles, "module_qualified_shake"),
      "module_qualified_shake",
    ),
  };
  // FALSIFIER: id=module-qualified-gate-002 class=liveness via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.deepEqual(
    existing.entryPin.population,
    independentlyDerivedEntryPopulation,
    `entry population does not match immutable Git sources at ${existing.entryPin.testedCommit}`,
  );
}
const entryPin = existing?.entryPin ?? {
  testedCommit: gitHead(),
  population: current,
};
const census: ModuleQualifiedProductPathCensus = {
  schemaVersion: "0",
  product: "omena.module-qualified-product-path-census",
  policyDigest,
  productCrates,
  entryPin,
  current,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
  execFileSync(path.join(repoRoot, "node_modules/.bin/oxfmt"), ["--write", censusPath], {
    cwd: repoRoot,
    stdio: "inherit",
  });
} else {
  // FALSIFIER: id=module-qualified-gate-003 class=liveness via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(existing, `module-qualified product-path census is missing: ${censusPath}`);
  // FALSIFIER: id=module-qualified-gate-004 class=liveness via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.deepEqual(existing, census, "module-qualified product-path census is stale");
}

process.stdout.write(
  `Omena module-qualified product path census OK: ` +
    `qualified=${current.qualifiedExecutor.totalOccurrences} ` +
    `productCallers=${current.qualifiedExecutor.productCallers.length} ` +
    `symbols=${current.moduleQualifiedSymbols.totalOccurrences} ` +
    `shake=${current.moduleQualifiedShake.totalOccurrences}\n`,
);

function classifyQualifiedExecutors(
  occurrences: readonly Occurrence[],
  symbols: readonly string[],
): QualifiedExecutorPopulation {
  const base = classifySymbol(occurrences, "execute_transform_passes_on_module_");
  const productCallers = occurrences
    .filter(
      (occurrence) =>
        !occurrence.testOnly &&
        !isDefinition(occurrence) &&
        !isReexport(occurrence) &&
        occurrence.text.includes("("),
    )
    .flatMap((occurrence) => {
      const crateName = crateNameForPath(occurrence.sourcePath);
      const symbol = symbols
        .filter((candidate) => occurrence.text.includes(candidate))
        .toSorted((left, right) => right.length - left.length)[0];
      if (
        !crateName ||
        !productCrates.includes(crateName as (typeof productCrates)[number]) ||
        !symbol
      ) {
        return [];
      }
      return [{ crateName, sourcePath: occurrence.sourcePath, symbol }];
    })
    .sort(
      (left, right) =>
        left.crateName.localeCompare(right.crateName) ||
        left.sourcePath.localeCompare(right.sourcePath) ||
        left.symbol.localeCompare(right.symbol),
    );
  return {
    ...base,
    symbols,
    productCallers,
  };
}

function classifySymbol(
  occurrences: readonly Occurrence[],
  definitionNeedle: string,
): SymbolPopulation {
  const productOccurrencesByCrate = Object.fromEntries(
    productCrates.map((crateName) => [crateName, 0]),
  ) as Record<string, number>;
  let definitionOccurrences = 0;
  let reexportOccurrences = 0;
  let testOccurrences = 0;
  let otherProductionOccurrences = 0;

  for (const occurrence of occurrences) {
    if (occurrence.testOnly) {
      testOccurrences += 1;
      continue;
    }
    if (isDefinition(occurrence, definitionNeedle)) {
      definitionOccurrences += 1;
      continue;
    }
    if (isReexport(occurrence)) {
      reexportOccurrences += 1;
      continue;
    }
    const crateName = crateNameForPath(occurrence.sourcePath);
    if (crateName && productCrates.includes(crateName as (typeof productCrates)[number])) {
      productOccurrencesByCrate[crateName] += 1;
    } else {
      otherProductionOccurrences += 1;
    }
  }

  return {
    totalOccurrences: occurrences.length,
    definitionOccurrences,
    reexportOccurrences,
    testOccurrences,
    otherProductionOccurrences,
    productOccurrencesByCrate,
  };
}

function isDefinition(
  occurrence: Occurrence,
  needle = "execute_transform_passes_on_module_",
): boolean {
  return occurrence.text.includes("pub fn ") && occurrence.text.includes(needle);
}

function isReexport(occurrence: Occurrence): boolean {
  return (
    occurrence.sourcePath.endsWith("/src/lib.rs") &&
    !occurrence.text.includes("(") &&
    !occurrence.text.includes(":")
  );
}

function crateNameForPath(sourcePath: string): string | undefined {
  const match = /^rust\/crates\/([^/]+)\//u.exec(sourcePath);
  return match?.[1];
}

function collectOccurrences(sourceFiles: readonly RustSourceFile[], symbol: string): Occurrence[] {
  const occurrences: Occurrence[] = [];
  const symbolPattern = new RegExp(`\\b${escapeRegExp(symbol)}\\b`, "u");
  for (const { sourcePath, source } of sourceFiles) {
    const testLines = cfgTestLineNumbers(sourcePath, source);
    for (const [index, text] of source.split("\n").entries()) {
      if (!symbolPattern.test(text)) {
        continue;
      }
      occurrences.push({
        sourcePath,
        line: index + 1,
        text: text.trim(),
        testOnly: testLines.has(index + 1),
      });
    }
  }
  return occurrences;
}

function cfgTestLineNumbers(sourcePath: string, source: string): Set<number> {
  const lines = source.split("\n");
  if (sourcePath.includes("/tests/")) {
    return new Set(lines.map((_, index) => index + 1));
  }

  const testLines = new Set<number>();
  let braceDepth = 0;
  let pendingCfgTest = false;
  let testModuleDepth: number | undefined;
  for (const [index, line] of lines.entries()) {
    const lineNumber = index + 1;
    if (line.includes("#[cfg(test)]")) {
      pendingCfgTest = true;
    }
    const opens = countCharacter(line, "{");
    const closes = countCharacter(line, "}");
    if (pendingCfgTest && /\bmod\s+[A-Za-z0-9_]+\s*\{/u.test(line)) {
      testModuleDepth = braceDepth + opens - closes;
      pendingCfgTest = false;
    }
    if (testModuleDepth !== undefined) {
      testLines.add(lineNumber);
    }
    braceDepth += opens - closes;
    if (testModuleDepth !== undefined && braceDepth < testModuleDepth) {
      testModuleDepth = undefined;
    }
  }
  return testLines;
}

function countCharacter(value: string, character: string): number {
  return [...value].filter((item) => item === character).length;
}

function rustSourceFiles(root: string): RustSourceFile[] {
  const files: RustSourceFile[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (entry.name === "target") {
      continue;
    }
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...rustSourceFiles(entryPath));
      continue;
    }
    if (entry.name.endsWith(".rs")) {
      files.push({
        sourcePath: path.relative(repoRoot, entryPath),
        source: fs.readFileSync(entryPath, "utf8"),
      });
    }
  }
  return files.sort((left, right) => left.sourcePath.localeCompare(right.sourcePath));
}

function gitRustSourceFiles(commit: string, symbols: readonly string[]): RustSourceFile[] {
  const grepArgs = ["grep", "-l"];
  for (const symbol of symbols) {
    grepArgs.push("-e", symbol);
  }
  grepArgs.push(commit, "--", "rust/**/*.rs");
  const grep = spawnSync("git", grepArgs, {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(
    grep.status,
    0,
    `cannot derive entry population from ${commit}: ${grep.stderr.trim()}`,
  );
  return grep.stdout
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => (line.startsWith(`${commit}:`) ? line.slice(commit.length + 1) : line))
    .map((sourcePath) => ({
      sourcePath,
      source: gitOutput(["show", `${commit}:${sourcePath}`]),
    }))
    .sort((left, right) => left.sourcePath.localeCompare(right.sourcePath));
}

function gitHead(): string {
  return gitOutput(["rev-parse", "HEAD"]).trim();
}

function gitOutput(args: readonly string[]): string {
  const result = spawnSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ${args.join(" ")} failed: ${result.stderr.trim()}`);
  return result.stdout;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
