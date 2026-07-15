import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface CargoPackage {
  readonly id: string;
  readonly name: string;
  readonly manifest_path: string;
}

interface CargoNode {
  readonly id: string;
  readonly dependencies: readonly string[];
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
  readonly resolve: { readonly nodes: readonly CargoNode[] };
}

interface InternalTypeRow {
  readonly crate: string;
  readonly typeName: string;
  readonly sourcePath: string;
  readonly line: number;
}

interface ResponseSurfaceCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-sdk.response-surface-split-census";
  readonly publicResponseTypes: readonly string[];
  readonly debugReportTypes: readonly string[];
  readonly publicReachableTypes: readonly string[];
  readonly debugReachableTypes: readonly string[];
  readonly internalTypeCountByCrate: Readonly<Record<string, number>>;
  readonly internalTypes: readonly InternalTypeRow[];
  readonly publicInternalLeaks: readonly string[];
}

const repoRoot = process.cwd();
const generatedPath = "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs";
const contractPath = "contracts/engine-sdk-workflow/main.tsp";
const censusPath = path.join(repoRoot, "rust/omena-response-surface-split-census.json");
const writeMode = process.argv.includes("--write");
const generated = fs.readFileSync(path.join(repoRoot, generatedPath), "utf8");
const contract = fs.readFileSync(path.join(repoRoot, contractPath), "utf8");
const graph = parseGeneratedTypeGraph(generated);
const publicResponseTypes = [...graph.keys()]
  .filter((name) => name.endsWith("ResponseV0") && !name.includes("Debug"))
  .toSorted();
const debugReportTypes = [...graph.keys()]
  .filter((name) => name.endsWith("DebugReportV0"))
  .toSorted();
assert.equal(publicResponseTypes.length, 5, "workflow contract must expose five public responses");
assert.ok(debugReportTypes.length > 0, "workflow contract must expose an opt-in debug report");

const publicReachable = reachableTypes(graph, publicResponseTypes);
const debugReachable = reachableTypes(graph, debugReportTypes);
if (process.env.OMENA_RESPONSE_SPLIT_TEST_INJECT_INTERNAL === "1") {
  publicReachable.add("IrNodeIdV0");
}

const internalTypes = scanReachableWorkspaceTypes();
const internalTypeNames = new Set(internalTypes.map((row) => row.typeName));
assert.ok(
  internalTypeNames.has("IrNodeIdV0"),
  "internal type census must include transform IR ids",
);
const publicInternalLeaks = [...publicReachable]
  .filter((name) => internalTypeNames.has(name))
  .toSorted();
assert.deepEqual(publicInternalLeaks, [], "public SDK responses embed internal-only Rust types");

const internalTypeCountByCrate: Record<string, number> = {};
for (const row of internalTypes) {
  internalTypeCountByCrate[row.crate] = (internalTypeCountByCrate[row.crate] ?? 0) + 1;
}
const census: ResponseSurfaceCensus = {
  schemaVersion: "0",
  product: "omena-sdk.response-surface-split-census",
  publicResponseTypes,
  debugReportTypes,
  publicReachableTypes: [...publicReachable].toSorted(),
  debugReachableTypes: [...debugReachable].toSorted(),
  internalTypeCountByCrate: Object.fromEntries(Object.entries(internalTypeCountByCrate).toSorted()),
  internalTypes,
  publicInternalLeaks,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
  execFileSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    stdio: "inherit",
  });
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "response surface split census is stale",
  );
}

process.stdout.write(
  `Omena response surface split OK: public=${publicResponseTypes.length} debug=${debugReportTypes.length} internal=${internalTypes.length}\n`,
);

function parseGeneratedTypeGraph(source: string): Map<string, Set<string>> {
  const graph = new Map<string, Set<string>>();
  const declarationPattern = /pub (?:struct|enum) ([A-Z][A-Za-z0-9_]*)[^{]*\{([\s\S]*?)\n\}/gu;
  for (const match of source.matchAll(declarationPattern)) {
    const [, typeName, body] = match;
    const dependencies = new Set<string>();
    for (const token of body.match(/\b[A-Z][A-Za-z0-9_]*\b/gu) ?? []) {
      if (!["Vec", "Option", "String", "Value", typeName].includes(token)) dependencies.add(token);
    }
    graph.set(typeName, dependencies);
  }
  return graph;
}

function reachableTypes(
  graph: ReadonlyMap<string, ReadonlySet<string>>,
  roots: readonly string[],
): Set<string> {
  const reached = new Set<string>();
  const queue = [...roots];
  while (queue.length > 0) {
    const current = queue.pop();
    if (!current || reached.has(current)) continue;
    reached.add(current);
    for (const dependency of graph.get(current) ?? []) {
      if (!reached.has(dependency)) queue.push(dependency);
    }
  }
  return reached;
}

function scanReachableWorkspaceTypes(): InternalTypeRow[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", "rust/Cargo.toml", "--format-version", "1"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as CargoMetadata;
  const packageById = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
  const queryPackage = metadata.packages.find((pkg) => pkg.name === "omena-query");
  assert.ok(queryPackage, "omena-query package missing from cargo metadata");
  const nodeById = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const reachableIds = new Set<string>();
  const queue = [queryPackage.id];
  while (queue.length > 0) {
    const id = queue.pop();
    if (!id || reachableIds.has(id)) continue;
    reachableIds.add(id);
    for (const dependency of nodeById.get(id)?.dependencies ?? []) queue.push(dependency);
  }

  const workspaceRoot = path.join(repoRoot, "rust/crates");
  const wireTypes = new Set(graph.keys());
  for (const match of contract.matchAll(/\b(?:model|enum|alias)\s+([A-Z][A-Za-z0-9_]*)\b/gu)) {
    wireTypes.add(match[1]);
  }
  const rows: InternalTypeRow[] = [];
  for (const id of [...reachableIds].toSorted()) {
    const pkg = packageById.get(id);
    if (!pkg || !path.resolve(pkg.manifest_path).startsWith(workspaceRoot)) continue;
    const sourceRoot = path.join(path.dirname(pkg.manifest_path), "src");
    for (const sourcePath of listRustSources(sourceRoot)) {
      const lines = fs.readFileSync(sourcePath, "utf8").split("\n");
      for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index].replace(/\/\/.*$/u, "");
        const match = /\b(?:struct|enum|type)\s+([A-Z][A-Za-z0-9_]*)\b/u.exec(line);
        if (!match || wireTypes.has(match[1])) continue;
        rows.push({
          crate: pkg.name,
          typeName: match[1],
          sourcePath: path.relative(repoRoot, sourcePath),
          line: index + 1,
        });
      }
    }
  }
  return rows.toSorted((left, right) =>
    `${left.crate}:${left.typeName}:${left.sourcePath}:${left.line}`.localeCompare(
      `${right.crate}:${right.typeName}:${right.sourcePath}:${right.line}`,
    ),
  );
}

function listRustSources(directory: string): string[] {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) return listRustSources(absolute);
    return entry.isFile() && entry.name.endsWith(".rs") ? [absolute] : [];
  });
}
