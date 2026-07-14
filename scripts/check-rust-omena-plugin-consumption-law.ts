import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface PluginSourceRow {
  readonly sourcePath: string;
  readonly digest: string;
  readonly rootedReferences: readonly string[];
}

interface PluginConsumptionLawCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.plugin-consumption-law-census";
  readonly allowlistedReferencePrefixes: readonly string[];
  readonly pluginSources: readonly PluginSourceRow[];
  readonly snapshotMethods: readonly string[];
  readonly transformIrMethods: readonly string[];
  readonly violationCount: number;
}

const repoRoot = process.cwd();
const runnerRoot = path.join(repoRoot, "rust/crates/omena-query-transform-runner");
const pluginRoot = path.join(runnerRoot, "src/plugins");
const apiPath = path.join(runnerRoot, "src/plugin_api.rs");
const censusPath = path.join(runnerRoot, "plugin-consumption-law-census.json");
const writeMode = process.argv.includes("--write");
const injectForbiddenSymbol = process.argv.includes("--inject-forbidden-symbol");
const allowlistedReferencePrefixes = ["crate::plugin_api"] as const;
const forbiddenReachThroughTypes = [
  "SourceSyntaxIndexV0",
  "StyleSemanticGraphSummaryV0",
  "TransformIrV0",
] as const;

const pluginSourcePaths = listRustSources(pluginRoot).map((absolutePath) =>
  path.relative(repoRoot, absolutePath),
);
assert.ok(pluginSourcePaths.length > 0, "plugin implementation census must be non-vacuous");

const violations: string[] = [];
const pluginSources = pluginSourcePaths.map((sourcePath) => {
  const source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
  const rootedReferences = collectRootedReferences(source);
  for (const reference of rootedReferences) {
    if (!allowlistedReferencePrefixes.some((prefix) => isWithinPrefix(reference, prefix))) {
      violations.push(`${sourcePath}: rooted reference ${reference} is outside the plugin API`);
    }
  }
  return {
    sourcePath,
    digest: createHash("sha256").update(source).digest("hex"),
    rootedReferences,
  } satisfies PluginSourceRow;
});

if (injectForbiddenSymbol) {
  violations.push(
    "rust/crates/omena-query-transform-runner/src/plugins/injected.rs: rooted reference crate::materialize_transform_module_evaluation_native_edits is outside the plugin API",
  );
}

const apiSource = fs.readFileSync(apiPath, "utf8");
const snapshotStruct = extractNamedBlock(apiSource, "pub struct PluginWorkspaceSnapshotV0");
const transformIrStruct = extractNamedBlock(apiSource, "pub struct PluginTransformIrV0");
assert.equal(
  collectPublicFields(snapshotStruct).length,
  0,
  "plugin workspace snapshot fields must remain private",
);
assert.equal(
  collectPublicFields(transformIrStruct).length,
  0,
  "plugin transform IR fields must remain private",
);

const snapshotImpl = extractNamedBlock(apiSource, "impl<'snapshot> PluginWorkspaceSnapshotV0");
const transformIrImpl = extractNamedBlock(apiSource, "impl<'ir> PluginTransformIrV0");
const snapshotMethods = collectPublicMethods(snapshotImpl);
const transformIrMethods = collectPublicMethods(transformIrImpl);
assert.deepEqual(
  snapshotMethods,
  ["class_universe", "completions_at_offset", "hover_at_offset", "new", "snapshot_id"],
  "plugin snapshot public surface changed; review the read-only consumption law",
);
assert.deepEqual(
  transformIrMethods,
  ["mutation_count", "new", "nodes", "printed_css", "replace_node", "source_byte_len", "source_id"],
  "plugin transform IR public surface changed; review the constrained mutation law",
);
assert.ok(
  !/pub\s+(?:const\s+)?fn\s+\w+[^{;]*&mut\s+self/u.test(snapshotImpl),
  "plugin snapshot must not expose mutable methods",
);
for (const forbiddenType of forbiddenReachThroughTypes) {
  assert.ok(
    !new RegExp(`(?:->|pub\\s+(?:const\\s+)?fn)[^\\n{;]*\\b${forbiddenType}\\b`, "u").test(
      snapshotImpl,
    ),
    `plugin snapshot must not expose ${forbiddenType}`,
  );
}
assert.ok(
  transformIrImpl.includes("IrTransactionV0::new(self.ir, self.plugin_id, region)"),
  "plugin IR mutations must pass through the transaction boundary",
);
assert.ok(
  !/pub\s+(?:const\s+)?fn\s+\w+[^{;]*->\s*&\s*mut\s+TransformIrV0/u.test(transformIrImpl),
  "plugin transform IR must not expose the raw mutable IR",
);

assert.deepEqual(violations, [], "plugin implementation crossed the allowlisted consumption law");

const census: PluginConsumptionLawCensus = {
  schemaVersion: "0",
  product: "omena.plugin-consumption-law-census",
  allowlistedReferencePrefixes,
  pluginSources,
  snapshotMethods,
  transformIrMethods,
  violationCount: violations.length,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
  formatJsonFile(censusPath);
} else {
  assert.ok(fs.existsSync(censusPath), "missing plugin consumption-law census; run with --write");
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "plugin consumption-law census is stale; regenerate and review every implementation reference",
  );
}

process.stdout.write(
  `Omena plugin consumption law OK: sources=${pluginSources.length} references=${pluginSources.reduce((count, row) => count + row.rootedReferences.length, 0)} violations=0\n`,
);

function listRustSources(directory: string): string[] {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) return listRustSources(absolutePath);
      return entry.isFile() && entry.name.endsWith(".rs") ? [absolutePath] : [];
    })
    .toSorted();
}

function collectRootedReferences(source: string): string[] {
  const references = [
    ...source.matchAll(
      /\b(?:crate|super|self|std|core|alloc|omena_[a-z0-9_]+)(?:::[A-Za-z_][A-Za-z0-9_]*)+/gu,
    ),
  ].map((match) => match[0]);
  return [...new Set(references)].toSorted();
}

function isWithinPrefix(reference: string, prefix: string): boolean {
  return reference === prefix || reference.startsWith(`${prefix}::`);
}

function extractNamedBlock(source: string, marker: string): string {
  const markerIndex = source.indexOf(marker);
  assert.ok(markerIndex >= 0, `missing Rust surface marker: ${marker}`);
  const openIndex = source.indexOf("{", markerIndex);
  assert.ok(openIndex >= 0, `missing Rust block for marker: ${marker}`);
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(markerIndex, index + 1);
  }
  throw new Error(`unterminated Rust block for marker: ${marker}`);
}

function collectPublicFields(structBlock: string): string[] {
  return [...structBlock.matchAll(/^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/gmu)].map(
    (match) => match[1],
  );
}

function collectPublicMethods(implBlock: string): string[] {
  return [...implBlock.matchAll(/\bpub\s+(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)]
    .map((match) => match[1])
    .toSorted();
}

function formatJsonFile(filePath: string): void {
  execFileSync(process.execPath, [path.join(repoRoot, "node_modules/oxfmt/bin/oxfmt"), filePath], {
    cwd: repoRoot,
    stdio: "inherit",
  });
}
