import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/product-path-matrix
 *
 * Validates the crate reachability matrix that separates structural Cargo roles
 * from actual product/evidence surfaces.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const matrixPath = path.join(repoRoot, "rust/omena-product-path-matrix.json");

const VALID_SURFACES = new Set([
  "product-runtime",
  "cli-runtime",
  "published-binding",
  "check-evidence",
  "legacy-oracle",
  "research-fixture",
  "support",
  "umbrella",
]);

const PRODUCT_ROOTS = ["omena-lsp-server", "omena-cli", "omena-napi", "omena-wasm"] as const;
const CLI_ROOTS = ["omena-cli"] as const;

interface ProductPathMatrix {
  readonly schemaVersion: string;
  readonly product: string;
  readonly entries: readonly ProductPathMatrixEntry[];
}

interface ProductPathMatrixEntry {
  readonly crate: string;
  readonly role: string;
  readonly surface: string;
  readonly entrypoints: readonly string[];
  readonly evidence: readonly string[];
  readonly gates?: readonly string[];
}

interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
  readonly optional?: boolean;
}

interface CargoPackage {
  readonly name: string;
  readonly dependencies: readonly CargoDependency[];
  readonly metadata?: {
    readonly omena?: {
      readonly role?: string;
    };
  };
}

const matrix = JSON.parse(readFileSync(matrixPath, "utf8")) as ProductPathMatrix;
assert.equal(matrix.schemaVersion, "0", "product-path matrix schemaVersion must be 0");
assert.equal(
  matrix.product,
  "omena-css.product-path-matrix",
  "unexpected product-path matrix product marker",
);

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const packagesByName = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));
const memberNames = new Set(packagesByName.keys());
const entriesByCrate = new Map<string, ProductPathMatrixEntry>();

for (const entry of matrix.entries) {
  assert.ok(
    !entriesByCrate.has(entry.crate),
    `duplicate product-path matrix entry: ${entry.crate}`,
  );
  entriesByCrate.set(entry.crate, entry);
  assert.ok(memberNames.has(entry.crate), `matrix entry names non-workspace crate: ${entry.crate}`);
  assert.ok(
    VALID_SURFACES.has(entry.surface),
    `${entry.crate} has invalid surface ${entry.surface}`,
  );
  assert.ok(entry.entrypoints.length > 0, `${entry.crate} must list at least one entrypoint`);
  assert.ok(entry.evidence.length > 0, `${entry.crate} must list at least one evidence path`);

  const role = packagesByName.get(entry.crate)?.metadata?.omena?.role;
  assert.equal(entry.role, role, `${entry.crate} matrix role must match Cargo metadata`);

  for (const evidencePath of entry.evidence) {
    assert.ok(
      existsSync(path.join(repoRoot, evidencePath)),
      `${entry.crate} evidence path does not exist: ${evidencePath}`,
    );
  }

  if (entry.surface === "check-evidence") {
    assert.ok(
      (entry.gates ?? []).length > 0,
      `${entry.crate} check-evidence entry must name the gate/test that consumes it`,
    );
  }
}

const missing = [...memberNames].filter((crate) => !entriesByCrate.has(crate)).sort();
assert.equal(
  missing.length,
  0,
  `product-path matrix is missing workspace crates:\n  ${missing.join("\n  ")}`,
);
const extra = [...entriesByCrate.keys()].filter((crate) => !memberNames.has(crate)).sort();
assert.equal(
  extra.length,
  0,
  `product-path matrix contains non-workspace crates:\n  ${extra.join("\n  ")}`,
);

function dependencyClosure(roots: readonly string[], includeOptional: boolean): Set<string> {
  const seen = new Set<string>();
  const stack = [...roots];
  while (stack.length > 0) {
    const current = stack.pop()!;
    if (seen.has(current)) {
      continue;
    }
    seen.add(current);
    const pkg = packagesByName.get(current);
    if (!pkg) {
      continue;
    }
    for (const dep of pkg.dependencies) {
      if (dep.kind === "dev") {
        continue;
      }
      if (!includeOptional && dep.optional) {
        continue;
      }
      if (memberNames.has(dep.name)) {
        stack.push(dep.name);
      }
    }
  }
  return seen;
}

const defaultProductReachable = dependencyClosure(PRODUCT_ROOTS, false);
const optionalProductReachable = dependencyClosure(PRODUCT_ROOTS, true);
const optionalCliReachable = dependencyClosure(CLI_ROOTS, true);

const productSurfaceNotReachable: string[] = [];
const cliSurfaceNotReachable: string[] = [];
const bindingSurfaceNotReachable: string[] = [];
const legacyReachable: string[] = [];
const researchProductWired: string[] = [];

for (const entry of matrix.entries) {
  if (entry.surface === "product-runtime" && !defaultProductReachable.has(entry.crate)) {
    productSurfaceNotReachable.push(entry.crate);
  }
  if (entry.surface === "cli-runtime" && !optionalCliReachable.has(entry.crate)) {
    cliSurfaceNotReachable.push(entry.crate);
  }
  if (entry.surface === "published-binding" && !optionalProductReachable.has(entry.crate)) {
    bindingSurfaceNotReachable.push(entry.crate);
  }
  if (entry.surface === "legacy-oracle" && defaultProductReachable.has(entry.crate)) {
    legacyReachable.push(entry.crate);
  }
  if (entry.surface === "research-fixture") {
    const libPath = path.join(repoRoot, "rust/crates", entry.crate, "src/lib.rs");
    if (existsSync(libPath)) {
      const source = readFileSync(libPath, "utf8");
      const moduleDocs = source
        .split("\n")
        .filter((line) => line.startsWith("//!"))
        .join("\n");
      if (moduleDocs.includes("product-wired")) {
        researchProductWired.push(entry.crate);
      }
    }
  }
}

assert.equal(
  productSurfaceNotReachable.length,
  0,
  `product-runtime matrix entries must be default-reachable from product roots:\n  ${productSurfaceNotReachable.join("\n  ")}`,
);
assert.equal(
  cliSurfaceNotReachable.length,
  0,
  `cli-runtime matrix entries must be reachable from omena-cli, including opt-in CLI features:\n  ${cliSurfaceNotReachable.join("\n  ")}`,
);
assert.equal(
  bindingSurfaceNotReachable.length,
  0,
  `published-binding matrix entries must be reachable from product roots:\n  ${bindingSurfaceNotReachable.join("\n  ")}`,
);
assert.equal(
  legacyReachable.length,
  0,
  `legacy-oracle entries must not be default-reachable from product roots:\n  ${legacyReachable.join("\n  ")}`,
);
assert.equal(
  researchProductWired.length,
  0,
  `research-fixture entries must not claim product-wired in module rustdoc:\n  ${researchProductWired.join("\n  ")}`,
);

const surfaceCounts = matrix.entries.reduce<Record<string, number>>((acc, entry) => {
  acc[entry.surface] = (acc[entry.surface] ?? 0) + 1;
  return acc;
}, {});

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.product-path-matrix",
      members: metadata.packages.length,
      entries: matrix.entries.length,
      surfaceCounts,
      productRoots: PRODUCT_ROOTS,
      defaultProductReachableCount: defaultProductReachable.size,
      optionalProductReachableCount: optionalProductReachable.size,
      violations: 0,
    },
    null,
    2,
  )}\n`,
);
