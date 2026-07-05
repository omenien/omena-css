import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

interface DischargeLedgerCheck {
  readonly product: string;
  readonly path: string;
  readonly entryCount: number;
  readonly coverage: readonly {
    readonly obligationFamily: string;
    readonly cellFamily: string;
    readonly cellCount: number;
    readonly exhaustive: boolean;
    readonly bound?: string;
  }[];
  readonly ledgerBackedFamilyCount: number;
  readonly proseOnlyFamilyCount: number;
  readonly pins: Record<string, string>;
  readonly defaultBuildSolverFree: boolean;
}

interface EvidenceAuthorityCheck {
  readonly product: string;
  readonly guaranteeFamilyCount: number;
  readonly productionStampSiteCount: number;
  readonly classifiedStampSiteCount: number;
  readonly ledgerFamilySiteCount: number;
  readonly ledgerStampCallerCount: number;
  readonly guaranteeFamilies: readonly string[];
  readonly violations: number;
}

interface ObligationFamilyClosureCheck {
  readonly product: string;
  readonly obligationFamilyCount: number;
  readonly guaranteeKindCount: number;
  readonly orphanFamilies: readonly string[];
  readonly extraCarrierFamilies: readonly string[];
  readonly hardClosure: boolean;
  readonly closurePassed: boolean;
}

interface SemanticPreservationModelCheck {
  readonly product: string;
  readonly artifactPath: string;
  readonly artifactSha256: string;
  readonly cascadeSeedCaseCount: number;
  readonly wptSeedCaseCount: number;
  readonly semanticObservationCaseCount: number;
  readonly rustGatePassed: boolean;
}

interface TranslationValidationKillRateCheck {
  readonly product: string;
  readonly stageReports: readonly {
    readonly stage: string;
    readonly corpusPath: string;
    readonly fixtureCount: number;
    readonly expectedRejectedCount: number;
    readonly supportedPassIds: readonly string[];
  }[];
  readonly fixtureCount: number;
  readonly expectedRejectedCount: number;
  readonly rustGatePassed: boolean;
}

interface FreshnessContract {
  readonly schemaVersion: string;
  readonly product: string;
  readonly gateId: string;
  readonly consumer: string;
  readonly requiredSignals: readonly string[];
  readonly failClosedMeaning: string;
}

const repoRoot = process.cwd();
const contractPath = path.join(
  repoRoot,
  "rust/crates/omena-cascade-proof/discharge-ledger/freshness-contract.v1.json",
);

const dischargeLedger = runJsonGate<DischargeLedgerCheck>("discharge ledger", [
  "node",
  "--import",
  "tsx",
  "./scripts/check-rust-discharge-ledger.ts",
  "--check",
]);
assert.equal(dischargeLedger.product, "omena-cascade-proof.discharge-ledger.check");
assert.ok(dischargeLedger.entryCount > 0, "discharge ledger must not be empty");
assert.equal(dischargeLedger.defaultBuildSolverFree, true);
assert.ok(dischargeLedger.ledgerBackedFamilyCount > 0);
assert.ok(dischargeLedger.proseOnlyFamilyCount > 0);

const requiredCoverageCells = new Set([
  "boxShorthandCombination",
  "longhandMerge",
  "scopeFlattenCandidate",
  "layerFlattenCandidate",
  "layerFlattenCascadeInversion",
  "staticSupportsCondition",
]);
for (const cellFamily of requiredCoverageCells) {
  const coverage = dischargeLedger.coverage.find((entry) => entry.cellFamily === cellFamily);
  assert.ok(coverage, `${cellFamily} coverage must be present`);
  assert.ok(coverage.cellCount > 0, `${cellFamily} coverage must not be empty`);
  assert.equal(coverage.exhaustive, true, `${cellFamily} coverage must be exhaustive`);
}
for (const [pinName, pinValue] of Object.entries(dischargeLedger.pins)) {
  assert.match(pinValue, /^[a-f0-9]{64}$|^z3-crate-\d+\.\d+\.\d+-gh-release$/u, pinName);
}

const evidenceAuthority = runJsonGate<EvidenceAuthorityCheck>("evidence authority", [
  "node",
  "--import",
  "tsx",
  "./scripts/check-rust-evidence-graph-single-authority.ts",
]);
assert.equal(evidenceAuthority.product, "rust.evidence-graph-single-authority");
assert.equal(evidenceAuthority.violations, 0);
assert.equal(
  evidenceAuthority.productionStampSiteCount,
  evidenceAuthority.classifiedStampSiteCount,
);
assert.equal(evidenceAuthority.ledgerFamilySiteCount, 1);
assert.equal(evidenceAuthority.ledgerStampCallerCount, 1);
assert.ok(
  evidenceAuthority.guaranteeFamilies.includes("LedgerBackedObligationDischarge"),
  "ledger-backed discharge family must stay registered",
);

const obligationClosure = runJsonGate<ObligationFamilyClosureCheck>("obligation closure", [
  "node",
  "--import",
  "tsx",
  "./scripts/check-rust-obligation-family-closure.ts",
]);
assert.equal(obligationClosure.product, "rust.rewrite-obligation-family-closure");
assert.equal(obligationClosure.guaranteeKindCount, 7);
assert.equal(obligationClosure.hardClosure, true);
assert.equal(obligationClosure.closurePassed, true);
assert.deepEqual(obligationClosure.orphanFamilies, []);
assert.deepEqual(obligationClosure.extraCarrierFamilies, []);

const semanticModel = runJsonGate<SemanticPreservationModelCheck>("semantic preservation model", [
  "node",
  "--import",
  "tsx",
  "./scripts/check-rust-semantic-preservation-model-conformance.ts",
]);
assert.equal(
  semanticModel.product,
  "omena-transform-passes.semantic-preservation-model-conformance.check",
);
assert.equal(semanticModel.rustGatePassed, true);
assert.ok(semanticModel.cascadeSeedCaseCount > 0);
assert.ok(semanticModel.wptSeedCaseCount > 0);
assert.ok(semanticModel.semanticObservationCaseCount > 0);
assert.match(semanticModel.artifactSha256, /^[a-f0-9]{64}$/u);

const translationKillRate = runJsonGate<TranslationValidationKillRateCheck>(
  "translation preservation kill rate",
  ["node", "--import", "tsx", "./scripts/check-rust-translation-validation-kill-rate.ts"],
);
assert.equal(
  translationKillRate.product,
  "omena-transform-passes.translation-validation-kill-rate.check",
);
assert.equal(translationKillRate.rustGatePassed, true);
assert.ok(translationKillRate.fixtureCount > 0);
assert.equal(translationKillRate.fixtureCount, translationKillRate.expectedRejectedCount);
assert.deepEqual(
  translationKillRate.stageReports.map((entry) => entry.stage),
  ["simple-structural", "merge-structural", "shake-structural", "flatten-structural"],
);
for (const report of translationKillRate.stageReports) {
  assert.ok(report.fixtureCount > 0, `${report.stage} corpus must not be empty`);
  assert.ok(report.expectedRejectedCount > 0, `${report.stage} corpus must contain rejected cases`);
}
const flattenStage = translationKillRate.stageReports.at(-1);
assert.ok(flattenStage, "flatten stage report must exist");
assert.deepEqual([...flattenStage.supportedPassIds].toSorted(), [
  "layer-flatten",
  "nesting-unwrap",
  "scope-flatten",
]);

const runtimeLookupRuns = 3;
for (let index = 0; index < runtimeLookupRuns; index += 1) {
  runCommand("runtime lookup probe", [
    "cargo",
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade-proof",
    "ledger_lookup_",
  ]);
}

const contract = readContract();
assert.equal(contract.schemaVersion, "1");
assert.equal(contract.product, "omena-cascade-proof.discharge-ledger.freshness-contract");
assert.equal(contract.gateId, "DISCHARGE-LEDGER-FRESH");
for (const signal of [
  "discharge-ledger-drift",
  "default-build-solver-free",
  "single-evidence-authority",
  "obligation-family-closure",
  "semantic-preservation-model",
  "translation-preservation-kill-rate",
]) {
  assert.ok(contract.requiredSignals.includes(signal), `${signal} must be in the contract`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.verification-plane-closure",
      closurePassed: true,
      stateOfVerification: {
        dischargeLedger: {
          entryCount: dischargeLedger.entryCount,
          ledgerBackedFamilyCount: dischargeLedger.ledgerBackedFamilyCount,
          proseOnlyFamilyCount: dischargeLedger.proseOnlyFamilyCount,
          coverageCellCount: dischargeLedger.coverage.length,
          pins: dischargeLedger.pins,
          defaultBuildSolverFree: dischargeLedger.defaultBuildSolverFree,
        },
        runtimeLookup: {
          repeatedRunCount: runtimeLookupRuns,
          probe: "omena-cascade-proof ledger lookup tests",
        },
        evidenceAuthority: {
          guaranteeFamilyCount: evidenceAuthority.guaranteeFamilyCount,
          productionStampSiteCount: evidenceAuthority.productionStampSiteCount,
          ledgerFamilySiteCount: evidenceAuthority.ledgerFamilySiteCount,
          ledgerStampCallerCount: evidenceAuthority.ledgerStampCallerCount,
        },
        obligationFamilies: {
          obligationFamilyCount: obligationClosure.obligationFamilyCount,
          guaranteeKindCount: obligationClosure.guaranteeKindCount,
          hardClosure: obligationClosure.hardClosure,
        },
        semanticPreservation: {
          artifactPath: semanticModel.artifactPath,
          artifactSha256: semanticModel.artifactSha256,
          cascadeSeedCaseCount: semanticModel.cascadeSeedCaseCount,
          wptSeedCaseCount: semanticModel.wptSeedCaseCount,
          semanticObservationCaseCount: semanticModel.semanticObservationCaseCount,
        },
        translationPreservation: {
          stageCount: translationKillRate.stageReports.length,
          fixtureCount: translationKillRate.fixtureCount,
          expectedRejectedCount: translationKillRate.expectedRejectedCount,
          stages: translationKillRate.stageReports.map((entry) => ({
            stage: entry.stage,
            fixtureCount: entry.fixtureCount,
            supportedPassIds: entry.supportedPassIds,
          })),
        },
        freshnessContract: {
          path: path.relative(repoRoot, contractPath),
          gateId: contract.gateId,
          requiredSignalCount: contract.requiredSignals.length,
          contractSha256: createHash("sha256")
            .update(readFileSync(contractPath, "utf8"))
            .digest("hex"),
        },
      },
    },
    null,
    2,
  )}\n`,
);

function runJsonGate<T>(label: string, command: readonly string[]): T {
  const result = runCommand(label, command);
  return JSON.parse(result.stdout) as T;
}

function runCommand(
  label: string,
  [command, ...args]: readonly string[],
): { readonly stdout: string; readonly stderr: string } {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 64,
  });
  assert.equal(
    result.status,
    0,
    `${label} failed\ncommand=${[command, ...args].join(" ")}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout, stderr: result.stderr };
}

function readContract(): FreshnessContract {
  return JSON.parse(readFileSync(contractPath, "utf8")) as FreshnessContract;
}
