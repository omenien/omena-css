import { spawnSync, execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/product-lab-closure
 *
 * Reports whether research/lab crates are present in product normal dependency
 * closures. This is intentionally separate from feature reachability: a crate
 * can be linked into a product even when its opt-in analysis feature is dormant.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rustManifest = path.join(repoRoot, "rust/Cargo.toml");
const rustDir = path.join(repoRoot, "rust");

const PRODUCT_ROOTS = ["omena-lsp-server", "omena-cli"] as const;
const LAB_CRATES = [
  "omena-smt",
  "omena-categorical",
  "omena-variational",
  "omena-ensemble",
  "omena-rg-flow",
  "omena-reachability-datalog-lab",
] as const;

interface CargoDepKind {
  readonly kind: string | null;
  readonly target: string | null;
}

interface CargoNodeDep {
  readonly name: string;
  readonly pkg: string;
  readonly dep_kinds: readonly CargoDepKind[];
}

interface CargoResolveNode {
  readonly id: string;
  readonly deps: readonly CargoNodeDep[];
}

interface CargoPackage {
  readonly id: string;
  readonly name: string;
  readonly metadata?: {
    readonly omena?: {
      readonly role?: string;
    };
  };
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
  readonly resolve: {
    readonly nodes: readonly CargoResolveNode[];
  };
}

interface CargoTreeCorroboration {
  readonly command: string;
  readonly status: number | null;
  readonly present: boolean;
  readonly absenceMatched: boolean;
  readonly excerpt: string;
}

interface RootLabClosureResult {
  readonly root: string;
  readonly presentLabCrates: readonly string[];
  readonly absentLabCrates: readonly string[];
  readonly corroboration: Readonly<Record<string, CargoTreeCorroboration>>;
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-default-features", "--format-version", "1", "--manifest-path", rustManifest],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 128 * 1024 * 1024 },
  ),
) as CargoMetadata;

const smtStubBackendPath = path.join(repoRoot, "rust/crates/omena-smt/src/backend/stub.rs");
const smtBackendMod = readFileSync(
  path.join(repoRoot, "rust/crates/omena-smt/src/backend/mod.rs"),
  "utf8",
);
const smtLib = readFileSync(path.join(repoRoot, "rust/crates/omena-smt/src/lib.rs"), "utf8");
const smtManifest = readFileSync(path.join(repoRoot, "rust/crates/omena-smt/Cargo.toml"), "utf8");
const cascadeProofLib = readFileSync(
  path.join(repoRoot, "rust/crates/omena-cascade-proof/src/lib.rs"),
  "utf8",
);
const datalogLabLibPath = path.join(
  repoRoot,
  "rust/crates/omena-reachability-datalog-lab/src/lib.rs",
);
const datalogLabLib = readFileSync(datalogLabLibPath, "utf8");
const streamingIfdsLib = readFileSync(
  path.join(repoRoot, "rust/crates/omena-streaming-ifds/src/lib.rs"),
  "utf8",
);

assert.equal(existsSync(smtStubBackendPath), false, "omena-smt must not own a stub backend file");
assert.equal(
  /\bStubSmtBackendV0\b/.test(smtBackendMod) || /\bStubSmtBackendV0\b/.test(smtLib),
  false,
  "omena-smt must not export the product-owned stub backend",
);
assert.equal(/\bsmt-stub\b/.test(smtManifest), false, "omena-smt must not expose smt-stub");
assert.ok(
  /fn check_canonical_input_v0\(&self, input: &CanonicalSmtInputV0\) -> SmtBackendCheckV0;/.test(
    smtBackendMod,
  ),
  "omena-smt backend trait must require concrete solver implementations",
);
assert.ok(
  /\bStubSmtBackendV0\b/.test(cascadeProofLib),
  "omena-cascade-proof must own the solver-free product backend",
);

const packagesByName = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));
const packagesById = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
const nodesById = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));

for (const root of PRODUCT_ROOTS) {
  assert.ok(packagesByName.has(root), `product root is missing from cargo metadata: ${root}`);
}
for (const crate of LAB_CRATES) {
  const pkg = packagesByName.get(crate);
  assert.ok(pkg, `lab crate is missing from cargo metadata: ${crate}`);
  assert.equal(pkg.metadata?.omena?.role, "R1", `${crate} must remain tagged role=R1`);
}

function directDependencyNames(crate: string): string[] {
  const pkg = packagesByName.get(crate);
  assert.ok(pkg, `crate is missing from cargo metadata: ${crate}`);
  const node = nodesById.get(pkg.id);
  assert.ok(node, `crate is missing from cargo resolve graph: ${crate}`);
  return node.deps.map((dep) => packagesById.get(dep.pkg)?.name ?? dep.name).sort();
}

function functionSpan(source: string, functionName: string): string {
  const start = source.indexOf(`pub fn ${functionName}`);
  assert.notEqual(start, -1, `${functionName} must exist`);
  const next = source.indexOf("\npub fn ", start + 1);
  return source.slice(start, next === -1 ? source.length : next);
}

function implSpan(source: string, implName: string): string {
  const start = source.indexOf(`impl ${implName}`);
  assert.notEqual(start, -1, `${implName} impl must exist`);
  const next = source.indexOf("\nimpl ", start + 1);
  return source.slice(start, next === -1 ? source.length : next);
}

const datalogLabDeps = directDependencyNames("omena-reachability-datalog-lab");
const datalogFactKeySpan = functionSpan(datalogLabLib, "datalog_fact_keys_v0");
const datalogFactKeyForbiddenRefs = [
  "BatchHypergraphConnectivityOracle",
  "collect_reachable_node_ids",
].filter((needle) => datalogFactKeySpan.includes(needle));
const demandFactKeySpan = [
  functionSpan(streamingIfdsLib, "run_streaming_ifds_demand_v0"),
  functionSpan(streamingIfdsLib, "run_streaming_ifds_demand_with_index_v0"),
  implSpan(streamingIfdsLib, "StreamingIFDSDemandIndexV0"),
  implSpan(streamingIfdsLib, "StreamingIFDSDemandSliceV0"),
].join("\n");
const demandFactKeyForbiddenRefs = [
  "propagate_ifds_facts_with_table",
  "run_streaming_ifds_exact_v0",
  "omena_streaming_ifds_batch_fact_keys_v0",
].filter((needle) => demandFactKeySpan.includes(needle));

assert.equal(
  datalogLabDeps.includes("omena-streaming-ifds"),
  false,
  "omena-reachability-datalog-lab must not depend on omena-streaming-ifds",
);
assert.deepEqual(
  datalogFactKeyForbiddenRefs,
  [],
  "datalog_fact_keys_v0 must not call the batch reachability oracle",
);
assert.deepEqual(
  demandFactKeyForbiddenRefs,
  [],
  "run_streaming_ifds_demand_v0 must not call the batch fact-key paths",
);

function hasNormalDep(dep: CargoNodeDep): boolean {
  return dep.dep_kinds.some((kind) => kind.kind === null);
}

function normalDependencyClosure(root: string): Set<string> {
  const rootPackage = packagesByName.get(root);
  assert.ok(rootPackage, `root package not found: ${root}`);

  const seenIds = new Set<string>();
  const stack = [rootPackage.id];
  while (stack.length > 0) {
    const current = stack.pop()!;
    if (seenIds.has(current)) {
      continue;
    }
    seenIds.add(current);
    const node = nodesById.get(current);
    if (!node) {
      continue;
    }
    for (const dep of node.deps) {
      if (hasNormalDep(dep)) {
        stack.push(dep.pkg);
      }
    }
  }
  return new Set(
    [...seenIds]
      .map((id) => packagesById.get(id)?.name)
      .filter((name): name is string => name !== undefined),
  );
}

function runCargoTree(root: string, crate: string): CargoTreeCorroboration {
  const args = ["tree", "-e", "normal", "--no-default-features", "-p", root, "-i", crate];
  const result = spawnSync("cargo", args, {
    cwd: rustDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const text = `${result.stdout ?? ""}${result.stderr ?? ""}`.trim();
  const absenceMatched = /did not match any packages/.test(text);
  return {
    command: `cargo ${args.join(" ")}`,
    status: result.status,
    present: result.status === 0,
    absenceMatched,
    excerpt: text.split("\n").slice(0, 8).join("\n"),
  };
}

const results: RootLabClosureResult[] = PRODUCT_ROOTS.map((root) => {
  const closure = normalDependencyClosure(root);
  const presentLabCrates = LAB_CRATES.filter((crate) => closure.has(crate));
  const absentLabCrates = LAB_CRATES.filter((crate) => !closure.has(crate));
  const corroboration = Object.fromEntries(
    LAB_CRATES.map((crate) => [crate, runCargoTree(root, crate)]),
  ) as Record<string, CargoTreeCorroboration>;

  for (const crate of LAB_CRATES) {
    const cargoTree = corroboration[crate];
    const metadataPresent = closure.has(crate);
    assert.equal(
      cargoTree.present,
      metadataPresent,
      `${root}/${crate} cargo metadata closure and cargo tree corroboration disagree`,
    );
    if (!metadataPresent) {
      assert.ok(
        cargoTree.absenceMatched,
        `${root}/${crate} absence should be corroborated by cargo tree's did-not-match message`,
      );
    }
  }

  return {
    root,
    presentLabCrates,
    absentLabCrates,
    corroboration,
  };
});

const violatingRoots = results.filter((result) => result.presentLabCrates.length > 0);
const hardFail = process.env.OMENA_FEATURE_REACHABILITY_HARDFAIL === "1";

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.product-lab-closure",
      mode: hardFail ? "hard-fail" : "report-only",
      resolution: "cargo metadata --no-default-features normal dependency closure",
      corroboration: "cargo tree -e normal --no-default-features -p <root> -i <labcrate>",
      smtStubEvaluatorOwner: "omena-cascade-proof",
      reachabilityDatalogFactKeyIndependence: {
        crate: "omena-reachability-datalog-lab",
        absentDependency: "omena-streaming-ifds",
        forbiddenReferences: ["BatchHypergraphConnectivityOracle", "collect_reachable_node_ids"],
        forbiddenReferenceCount: datalogFactKeyForbiddenRefs.length,
      },
      streamingIfdsDemandFactKeyIndependence: {
        crate: "omena-streaming-ifds",
        function: "run_streaming_ifds_demand_v0",
        forbiddenReferences: [
          "propagate_ifds_facts_with_table",
          "run_streaming_ifds_exact_v0",
          "omena_streaming_ifds_batch_fact_keys_v0",
        ],
        forbiddenReferenceCount: demandFactKeyForbiddenRefs.length,
      },
      productRoots: PRODUCT_ROOTS,
      labCrates: LAB_CRATES,
      roots: results,
      violationRootCount: violatingRoots.length,
    },
    null,
    2,
  )}\n`,
);

if (violatingRoots.length > 0) {
  const summary = violatingRoots
    .map((result) => `  ${result.root}: present=[${result.presentLabCrates.join(", ")}]`)
    .join("\n");
  if (hardFail) {
    assert.fail(`product/lab closure violation:\n${summary}`);
  }
  process.stderr.write(`warning: product/lab closure violation (report-only):\n${summary}\n`);
}
