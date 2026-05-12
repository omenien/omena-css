import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const packageJson = read("package.json");

assert.ok(
  packageJson.includes('"check:rust-omena-parser-boundary"') &&
    packageJson.includes("rust/omena-parser/forward-canary"),
  "rust/omena-parser/boundary must include the forward-only parser canary gate",
);
assert.ok(
  packageJson.includes("rust/omena-parser/style-facts-parity") &&
    packageJson.includes("rust/omena-parser/differential-corpus") &&
    packageJson.includes("rust/omena-parser/cutover-readiness"),
  "parser boundary must keep parity, differential, and cutover readiness gates together",
);
assert.ok(
  packageJson.includes('"check:rust-omena-lsp-server-lane"') &&
    packageJson.includes("rust/omena-lsp-server/boundary") &&
    packageJson.includes("rust/omena-lsp-server/provider-parity") &&
    packageJson.includes("rust/omena-lsp-server/runtime-loop"),
  "forward canary must keep the Rust LSP lane wired through boundary, parity, and runtime-loop checks",
);
assert.ok(
  packageJson.includes('"check:rust-phase-2-swap-readiness"') &&
    packageJson.includes("rust/omena-lsp-server/lane") &&
    packageJson.includes("rust/lsp-runtime-loop"),
  "phase-2 swap readiness must exercise the Rust LSP lane and runtime loop",
);
assert.ok(
  packageJson.includes('"check:rust-release-bundle"') &&
    packageJson.includes("rust/omena-parser/boundary") &&
    packageJson.includes("rust/parser/public-product") &&
    packageJson.includes("rust/omena-bridge/boundary") &&
    packageJson.includes("rust/omena-cascade/boundary") &&
    packageJson.includes("rust/gate/evidence -- --variant tsgo"),
  "release bundle must keep parser, public product, bridge, cascade, and evidence gates together",
);
assert.ok(
  !packageJson.includes("RUST_PARSER=engine-style-parser") &&
    !packageJson.includes("RUST_PARSER=omena-parser"),
  "parser canary must be forward-only and not rely on parser-selection env toggles",
);

const lspBoundary = read("rust/crates/omena-lsp-server/src/boundary.rs");
assert.ok(
  lspBoundary.includes('migration_status: "rustStable"'),
  "Rust LSP boundary must advertise rustStable migration status for the forward canary",
);
assert.ok(
  lspBoundary.includes("node_fallback_allowed: false"),
  "Rust LSP thin-client endpoint must not allow Node fallback in the forward canary",
);
assert.ok(
  lspBoundary.includes("ownProviderExecution") &&
    lspBoundary.includes("ownWorkspaceState") &&
    lspBoundary.includes("ownTsgoClientLifecycle"),
  "Rust LSP endpoint must own provider execution, workspace state, and tsgo lifecycle",
);

const thinClientGate = read("scripts/check-rust-omena-lsp-server-thin-client-boundary.ts");
assert.ok(
  thinClientGate.includes("assert.equal(rustEndpoint.nodeFallbackAllowed, false)") &&
    thinClientGate.includes("assert.equal(clientEndpoint.nodeFallbackAllowed, false)"),
  "thin-client gate must assert fallback=false on both Rust and client endpoint contracts",
);

const cutoverGate = read("scripts/check-rust-omena-parser-cutover-readiness.ts");
assert.ok(
  cutoverGate.includes("must not depend on engine-style-parser") &&
    cutoverGate.includes("must not invoke engine-style-parser in the product parser lane"),
  "cutover readiness must block legacy parser dependencies and invocations in product lanes",
);

process.stdout.write(
  "validated omena-parser forward canary: rustStable=true fallback=false lspLane=true releaseEvidence=true\n",
);

function read(filePath: string): string {
  return readFileSync(filePath, "utf8");
}
