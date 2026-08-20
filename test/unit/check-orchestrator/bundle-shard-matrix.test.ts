import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { findBundleShardMatrixDiagnostics } from "../../../packages/check-orchestrator/src/manifest/workflows";

// g131 stage-5 R5: the standing falsifiers for the bundle-shard consumption
// rule — three confirm rounds re-opened one species (a fromJSON reference
// satisfying the env branch from the wrong place); these arms pin every
// direction that was live-probed.

const repoRoot = path.resolve(__dirname, "../../..");

function rootWith(ci: string): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "omena-shard-matrix-"));
  mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
  writeFileSync(path.join(root, ".github/workflows/ci.yml"), ci);
  return root;
}

const GENERATED_OK = `name: CI
jobs:
  preflight:
    runs-on: ubuntu-latest
    outputs:
      closure-fast-shards: \${{ steps.closure-fast-shards.outputs.matrix }}
    steps:
      - id: closure-fast-shards
        run: pnpm omena-check shards rust/closure-fast --json
  closure-fast-shards:
    needs: preflight
    strategy:
      fail-fast: false
      matrix:
        shard: \${{ fromJSON(needs.preflight.outputs.closure-fast-shards) }}
    runs-on: ubuntu-latest
    steps:
      - env:
          CLOSURE_FAST_SHARD: \${{ matrix.shard }}
        run: pnpm omena-check run rust/closure-fast --summary --shard="$CLOSURE_FAST_SHARD"
  rust-contracts:
    strategy:
      fail-fast: false
      matrix:
        contracts-shard: [public-surface, rest]
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check bundle rust/contracts --summary --shard=\${{ matrix.contracts-shard }}
  rust-product-test-contracts:
    strategy:
      fail-fast: false
      matrix:
        contract-shard: [api-surface, structural-shadow, rest]
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check bundle rust/product-test-contracts --summary --shard=\${{ matrix.contract-shard }}
`;

function drifts(ci: string): string[] {
  return findBundleShardMatrixDiagnostics(rootWith(ci))
    .filter((diagnostic) => diagnostic.code === "bundle-shard-matrix-drift")
    .map((diagnostic) => diagnostic.message);
}

describe("bundle shard-table/matrix consumption (g131 R2-R5)", () => {
  it("is quiet on the real repository", () => {
    expect(
      findBundleShardMatrixDiagnostics(repoRoot).filter(
        (diagnostic) => diagnostic.code === "bundle-shard-matrix-drift",
      ),
    ).toEqual([]);
  });

  it("is quiet on the sanctioned shapes (inline equality + producer-bound generated matrix)", () => {
    expect(drifts(GENERATED_OK)).toEqual([]);
  });

  it("RED: an inline matrix missing a table shard (the original 9/13 silent drop)", () => {
    const mutated = GENERATED_OK.replace(
      "contracts-shard: [public-surface, rest]",
      "contracts-shard: [rest]",
    );
    expect(drifts(mutated).join(";")).toContain('job "rust-contracts"');
  });

  it("RED: env-form without any generated matrix (the R2-confirm evasion)", () => {
    const mutated = GENERATED_OK.replace(
      "      - run: pnpm omena-check bundle rust/contracts --summary --shard=${{ matrix.contracts-shard }}",
      '      - env:\n          S: ${{ matrix.contracts-shard }}\n        run: pnpm omena-check bundle rust/contracts --summary --shard="$S"',
    ).replace("contracts-shard: [public-surface, rest]", "contracts-shard: [rest]");
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: a fromJSON reference in a COMMENT does not sanction the env form (R3-confirm evasion)", () => {
    const mutated = GENERATED_OK.replace(
      "      - run: pnpm omena-check bundle rust/contracts --summary --shard=${{ matrix.contracts-shard }}",
      '      # shard: ${{ fromJSON(needs.preflight.outputs.contracts-shards) }}\n      - env:\n          S: ${{ matrix.contracts-shard }}\n        run: pnpm omena-check bundle rust/contracts --summary --shard="$S"',
    ).replace("contracts-shard: [public-surface, rest]", "contracts-shard: [rest]");
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: a fromJSON DECOY outside strategy.matrix does not sanction the env form (R4-confirm evasion)", () => {
    const mutated = GENERATED_OK.replace(
      "      - run: pnpm omena-check bundle rust/contracts --summary --shard=${{ matrix.contracts-shard }}",
      '      - env:\n          S: ${{ matrix.contracts-shard }}\n          DECOY: ${{ fromJSON(needs.preflight.outputs.not-a-real-output) }}\n        run: pnpm omena-check bundle rust/contracts --summary --shard="$S"',
    ).replace("contracts-shard: [public-surface, rest]", "contracts-shard: [rest]");
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: a generated matrix from the WRONG bundle's table does not sanction (R4-confirm D2)", () => {
    const mutated = GENERATED_OK.replace(
      "        contracts-shard: [public-surface, rest]",
      "        contracts-shard: ${{ fromJSON(needs.preflight.outputs.closure-fast-shards) }}",
    ).replace(
      "      - run: pnpm omena-check bundle rust/contracts --summary --shard=${{ matrix.contracts-shard }}",
      '      - env:\n          S: ${{ matrix.contracts-shard }}\n        run: pnpm omena-check bundle rust/contracts --summary --shard="$S"',
    );
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: a producer whose shards run is only MENTIONED (comment/name), not executed (R5-confirm rider)", () => {
    const mutated = GENERATED_OK.replace(
      "      - id: closure-fast-shards\n        run: pnpm omena-check shards rust/closure-fast --json",
      "      - id: closure-fast-shards\n        run: echo hand-written # omena-check shards rust/closure-fast --json",
    );
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: the output mapped from a step UNRELATED to the shards run (R5-confirm rider)", () => {
    const mutated = GENERATED_OK.replace(
      "      closure-fast-shards: \${{ steps.closure-fast-shards.outputs.matrix }}",
      "      closure-fast-shards: \${{ steps.unrelated.outputs.matrix }}",
    ).replace(
      "      - id: closure-fast-shards\n        run: pnpm omena-check shards rust/closure-fast --json",
      "      - id: unrelated\n        run: echo literal-matrix\n      - id: closure-fast-shards\n        run: pnpm omena-check shards rust/closure-fast --json",
    );
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: a two-bundle producer cannot sanction the WRONG bundle's consumer (R5-confirm rider)", () => {
    // preflight also derives rust/contracts, but the consumed OUTPUT is still
    // mapped from the closure-fast step — a contracts consumer binding the
    // closure-fast output must RED.
    const mutated = GENERATED_OK.replace(
      "        contracts-shard: [public-surface, rest]",
      "        contracts-shard: \${{ fromJSON(needs.preflight.outputs.closure-fast-shards) }}",
    )
      .replace(
        "      - run: pnpm omena-check bundle rust/contracts --summary --shard=\${{ matrix.contracts-shard }}",
        '      - env:\n          S: \${{ matrix.contracts-shard }}\n        run: pnpm omena-check bundle rust/contracts --summary --shard="$S"',
      )
      .replace(
        "      - id: closure-fast-shards\n        run: pnpm omena-check shards rust/closure-fast --json",
        "      - id: closure-fast-shards\n        run: pnpm omena-check shards rust/closure-fast --json\n      - id: contracts-shards\n        run: pnpm omena-check shards rust/contracts --json",
      );
    expect(drifts(mutated).join(";")).toContain("env-bound --shard");
  });

  it("RED: an unconsumed table (no --shard invocation at all)", () => {
    const mutated = GENERATED_OK.replace(
      "      - run: pnpm omena-check bundle rust/contracts --summary --shard=${{ matrix.contracts-shard }}",
      "      - run: echo not-consuming",
    );
    expect(drifts(mutated).join(";")).toContain("no ci.yml job consumes it");
  });
});
