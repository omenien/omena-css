import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface DischargeLedger {
  readonly schemaVersion: "1";
  readonly product: "omena-cascade-proof.discharge-ledger";
  readonly pins: {
    readonly theorySignatureHash: string;
    readonly specDigest: string;
    readonly encoderContentHash: string;
    readonly solverVersion: string;
  };
  readonly coverage: readonly DischargeCoverage[];
  readonly entries: readonly DischargeEntry[];
}

interface DischargeCoverage {
  readonly obligationFamily: string;
  readonly cellFamily: string;
  readonly cellCount: number;
  readonly exhaustive: boolean;
  readonly bound?: string;
}

interface DischargeEntry {
  readonly obligationFamily: string;
  readonly cellFamily: string;
  readonly obligationId: string;
  readonly l1Primitive: string;
  readonly cellKey: string;
  readonly canonicalTermCount: number;
  readonly canonicalTerms: readonly string[];
  readonly verdict: "accepted" | "rejected" | "unknown";
  readonly boundedness: { readonly kind: "exact" | "boundedK"; readonly k?: number };
  readonly referenceKind: string;
  readonly referenceSatResult: "sat" | "unsat" | "unknown";
  readonly solverKind: "z3";
  readonly solverSatResult: "sat" | "unsat" | "unknown";
  readonly referenceMatchesSolver: boolean;
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const ledgerPath = path.join(
  repoRoot,
  "rust/crates/omena-cascade-proof/discharge-ledger/ledger.v1.json",
);

const generatedSource = buildLedgerSource();
const generatedLedger = parseLedger(generatedSource);
assertLedgerShape(generatedLedger);

if (checkOnly) {
  assert.ok(
    existsSync(ledgerPath),
    "discharge ledger artifact is missing; run `pnpm update:rust-discharge-ledger`",
  );
  assert.equal(
    readFileSync(ledgerPath, "utf8"),
    generatedSource,
    "discharge ledger is stale; run `pnpm update:rust-discharge-ledger`",
  );
} else if (writeMode) {
  mkdirSync(path.dirname(ledgerPath), { recursive: true });
  writeFileSync(ledgerPath, generatedSource);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-cascade-proof.discharge-ledger.check",
      mode: checkOnly ? "check" : "write",
      path: path.relative(repoRoot, ledgerPath),
      entryCount: generatedLedger.entries.length,
      coverage: generatedLedger.coverage.map((entry) => ({
        cellFamily: entry.cellFamily,
        cellCount: entry.cellCount,
        exhaustive: entry.exhaustive,
        bound: entry.bound,
      })),
      pins: generatedLedger.pins,
    },
    null,
    2,
  )}\n`,
);

function buildLedgerSource(): string {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade-proof",
      "--features",
      "smt-z3",
      "--bin",
      "omena-cascade-discharge-ledger",
      "--quiet",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );
  assert.equal(
    result.status,
    0,
    `discharge ledger generator failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result.stdout;
}

function parseLedger(source: string): DischargeLedger {
  return JSON.parse(source) as DischargeLedger;
}

function assertLedgerShape(ledger: DischargeLedger): void {
  assert.equal(ledger.schemaVersion, "1");
  assert.equal(ledger.product, "omena-cascade-proof.discharge-ledger");
  assert.match(ledger.pins.theorySignatureHash, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.specDigest, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.encoderContentHash, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.solverVersion, /^z3-crate-\d+\.\d+\.\d+-gh-release$/u);
  assert.ok(ledger.entries.length > 0, "ledger must not be empty");

  const requiredCellFamilies = [
    "boxShorthandCombination",
    "longhandMerge",
    "scopeFlattenCandidate",
    "layerFlattenCandidate",
    "staticSupportsCondition",
    "layerFlattenCascadeInversion",
  ];
  assert.deepEqual(
    ledger.coverage.map((coverage) => coverage.cellFamily).toSorted(),
    requiredCellFamilies.toSorted(),
  );
  for (const family of requiredCellFamilies) {
    const coverage = ledger.coverage.find((entry) => entry.cellFamily === family);
    assert.ok(coverage, `${family} coverage must exist`);
    assert.ok(coverage.cellCount > 0, `${family} coverage must not be empty`);
    assert.equal(coverage.exhaustive, true, `${family} coverage must be marked exhaustive`);
  }
  assert.ok(
    ledger.coverage.find((entry) => entry.cellFamily === "layerFlattenCascadeInversion")?.bound,
    "bounded layer inversion coverage must state its bound",
  );

  const keys = new Set<string>();
  for (const entry of ledger.entries) {
    assert.match(entry.cellKey, /^[a-f0-9]{64}$/u);
    assert.ok(!keys.has(entry.cellKey), `${entry.cellKey} must be unique`);
    keys.add(entry.cellKey);
    assert.equal(entry.canonicalTermCount, entry.canonicalTerms.length);
    assert.equal(entry.solverKind, "z3");
    assert.equal(
      entry.referenceMatchesSolver,
      true,
      `${entry.cellFamily}/${entry.cellKey} reference and solver must agree`,
    );
    if (entry.cellFamily === "layerFlattenCascadeInversion") {
      assert.equal(entry.boundedness.kind, "boundedK");
      assert.equal(entry.boundedness.k, 3);
      assert.equal(entry.referenceKind, "boundedLayerOrderingPredicate");
    } else {
      assert.equal(entry.boundedness.kind, "exact");
      assert.equal(entry.referenceKind, "productStubBackend");
    }
  }
}
