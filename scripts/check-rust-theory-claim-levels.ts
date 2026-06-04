import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

// Crates that anchor the product call-graph. A crate that declares
// `claim_level: ... product-wired ...` must be reachable from one of these
// through the Cargo dependency DAG — `omena-napi`/`omena-wasm` → `omena-query`
// → cascade checker → `omena-query-checker-orchestrator` → `omena-checker` →
// theory crates. Reachability here means the dependency closure (call-graph),
// not mere token presence, so a future crate that paints itself `product-wired`
// without being wired into the product chain fails this gate.
const PRODUCT_ROOTS = ["omena-query", "omena-query-checker-orchestrator"] as const;
const PRODUCT_WIRED_MARKER = "product-wired";

const TARGETS = [
  {
    path: "rust/crates/omena-abstract-value/src/lib.rs",
    required: [
      "claim_level:",
      "product-wired class-value, selector-projection, provenance, and reduced-product substrate",
      "cascade-family remains research-staged",
      "not a completed abstract-interpretation theorem",
    ],
  },
  {
    path: "rust/crates/omena-categorical/src/lib.rs",
    required: ["claim_level:", "product-wired additive evidence", "not a completed categorical"],
  },
  {
    path: "rust/crates/omena-smt/src/lib.rs",
    required: ["claim_level:", "opt-in solver-backed checking", "not default build SMT"],
  },
  {
    path: "rust/crates/omena-variational/src/lib.rs",
    required: ["claim_level:", "product-wired posterior inference", "not a corpus-calibrated"],
  },
  {
    path: "rust/crates/omena-zk-audit/src/lib.rs",
    required: ["claim_level:", "opt-in arkworks proof round-trip", "default build stays"],
  },
  {
    path: "rust/crates/omena-zk-circuit/src/lib.rs",
    required: ["claim_level:", "constraint-generation substrate", "not a standalone proving"],
  },
  {
    path: "rust/crates/omena-rg-flow/src/lib.rs",
    required: [
      "claim_level:",
      "opt-in deep-analysis Jacobian-spectrum approximation",
      "not a default product decision mechanism",
      "not a full",
    ],
  },
  {
    path: "rust/crates/omena-lawvere/src/lib.rs",
    required: ["claim_level:", "differential commutativity witness", "not a global"],
  },
  {
    path: "rust/crates/omena-streaming-ifds/src/lib.rs",
    required: ["claim_level:", "exact default live-analysis mechanism", "not an asymptotic"],
  },
  {
    path: "rust/crates/omena-ensemble/src/lib.rs",
    required: [
      "claim_level:",
      "product-wired cross-file consistency hint substrate",
      "not a default product",
    ],
  },
  {
    path: "rust/crates/omena-refinement/src/lib.rs",
    required: [
      "claim_level:",
      "product-wired cascade refinement bridge substrate",
      "not Liquid-Haskell",
    ],
  },
] as const;

function crateNameFromTargetPath(targetPath: string): string {
  const match = /rust\/crates\/([^/]+)\/src\/lib\.rs$/u.exec(targetPath);
  assert.ok(match, `unexpected claim-level target path layout: ${targetPath}`);
  return match[1];
}

// Transitive workspace-internal dependency closure of the product roots. Optional
// (feature-gated) edges are excluded: a crate reachable only behind an off-by-
// default feature is not part of the default product call-graph and must not be
// allowed to claim `product-wired`.
function productReachableCrates(): Set<string> {
  const result = spawnSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--format-version", "1", "--no-deps"],
    { cwd: process.cwd(), encoding: "utf8", maxBuffer: 1024 * 1024 * 64 },
  );
  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `cargo metadata failed (status=${result.status})\nstderr=${result.stderr}`,
  );
  const metadata = JSON.parse(result.stdout) as {
    packages: ReadonlyArray<{
      name: string;
      dependencies: ReadonlyArray<{ name: string; optional?: boolean }>;
    }>;
  };
  const workspaceByName = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));
  const reachable = new Set<string>();
  const stack: string[] = [...PRODUCT_ROOTS];
  while (stack.length > 0) {
    const name = stack.pop() as string;
    if (reachable.has(name)) {
      continue;
    }
    reachable.add(name);
    const pkg = workspaceByName.get(name);
    if (!pkg) {
      continue;
    }
    for (const dependency of pkg.dependencies) {
      if (dependency.optional) {
        continue;
      }
      if (workspaceByName.has(dependency.name)) {
        stack.push(dependency.name);
      }
    }
  }
  return reachable;
}

let reachableCache: Set<string> | undefined;
let productWiredCount = 0;

for (const target of TARGETS) {
  const source = readFileSync(target.path, "utf8");
  const moduleDocs = source
    .split("\n")
    .filter((line) => line.startsWith("//!"))
    .map((line) => line.replace(/^\/\/!\s?/u, "").trim())
    .join(" ");
  for (const required of target.required) {
    assert.ok(
      moduleDocs.includes(required),
      `${target.path} must include claim-level rustdoc token: ${required}`,
    );
  }

  if (moduleDocs.includes(PRODUCT_WIRED_MARKER)) {
    productWiredCount += 1;
    reachableCache ??= productReachableCrates();
    const crateName = crateNameFromTargetPath(target.path);
    assert.ok(
      reachableCache.has(crateName),
      `${crateName} declares claim_level '${PRODUCT_WIRED_MARKER}' but is not reachable from the product call-graph (${PRODUCT_ROOTS.join(
        " / ",
      )} dependency closure); downgrade the claim or wire the crate into the product chain`,
    );
  }
}

process.stdout.write(
  `validated theory claim_level rustdoc: crateCount=${TARGETS.length} marker=claim_level productWired=${productWiredCount} reachabilityRoots=${PRODUCT_ROOTS.join(
    "+",
  )}\n`,
);
