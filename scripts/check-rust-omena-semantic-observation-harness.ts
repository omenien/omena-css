import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import type { EngineInputV2Json } from "../server/engine-core-ts/src/contracts/engine-v2-input-idl.generated";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import { OMENA_SEMANTIC_OBSERVATION_CORPUS } from "./omena-semantic-observation-corpus";
import {
  runObservation,
  runObservationContract,
  type TheoryObservationHarnessSummaryV0,
} from "./omena-semantic-observation-runtime";

const STYLE_PATH = "/tmp/Component.module.scss";
const SOURCE_PATH = "/tmp/Component.tsx";
const WORKSPACE_ROOT = "/tmp";

function range(startLine: number, startCharacter: number, endLine: number, endCharacter: number) {
  return {
    start: {
      line: startLine,
      character: startCharacter,
    },
    end: {
      line: endLine,
      character: endCharacter,
    },
  };
}

function sampleEngineInput(): EngineInputV2Json {
  return {
    version: "2",
    workspace: {
      root: WORKSPACE_ROOT,
      classnameTransform: "asIs",
      settingsKey: "transform:asIs;alias:",
    },
    sources: [
      {
        filePath: SOURCE_PATH,
        document: {
          classExpressions: [
            {
              id: "expr-button",
              kind: "literal",
              scssModulePath: STYLE_PATH,
              range: range(4, 12, 4, 18),
              className: "button",
            },
            {
              id: "expr-primary",
              kind: "symbolRef",
              scssModulePath: STYLE_PATH,
              range: range(5, 12, 5, 24),
              rootBindingDeclId: "decl-primary",
            },
          ],
        },
      },
    ],
    styles: [
      {
        filePath: STYLE_PATH,
        document: {
          selectors: [
            {
              name: "button",
              viewKind: "canonical",
              canonicalName: "button",
              range: range(0, 1, 0, 7),
              nestedSafety: "flat",
            },
            {
              name: "button--primary",
              viewKind: "canonical",
              canonicalName: "button--primary",
              range: range(1, 1, 1, 17),
              nestedSafety: "flat",
            },
          ],
        },
      },
    ],
    typeFacts: [
      {
        filePath: SOURCE_PATH,
        expressionId: "expr-button",
        facts: {
          kind: "exact",
          values: ["button"],
        },
      },
      {
        filePath: SOURCE_PATH,
        expressionId: "expr-primary",
        facts: {
          kind: "constrained",
          constraintKind: "prefix",
          prefix: "button--",
        },
      },
    ],
  };
}

async function runSyntheticObservation(
  styleSource: string,
): Promise<TheoryObservationHarnessSummaryV0> {
  return runObservation({
    stylePath: STYLE_PATH,
    styleSource,
    engineInput: sampleEngineInput(),
  });
}

void (async () => {
  execFileSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-semantic",
      "theory_observation_harness",
    ],
    {
      cwd: process.cwd(),
      stdio: "inherit",
    },
  );

  const ready = await runSyntheticObservation(".button { &__icon { color: red; } }");
  assert.equal(ready.product, "omena-semantic.theory-observation-harness");
  assert.equal(ready.selectorIdentity.status, "ready");
  assert.equal(ready.selectorIdentity.renameSafe, true);
  assert.equal(ready.sourceEvidence.status, "ready");
  assert.equal(ready.downstreamReadiness.status, "ready");
  assert.deepEqual(ready.blockingGaps, []);
  assert.deepEqual(ready.nextPriorities, ["externalCorpus", "traitDogfooding"]);
  process.stdout.write(
    `validated omena-semantic observation harness: ready selectors=${ready.selectorIdentity.observedSelectorCount} sourceExpressions=${ready.sourceEvidence.expressionCount}\n`,
  );

  const blocked = await runSyntheticObservation(".button { &.active { color: red; } }");
  assert.equal(blocked.selectorIdentity.status, "partial");
  assert.equal(blocked.selectorIdentity.rewriteBlockedSelectorCount, 1);
  assert.equal(blocked.downstreamReadiness.status, "partial");
  assert.deepEqual(blocked.blockingGaps, ["selectorRewriteSafety", "downstreamReadiness"]);
  process.stdout.write(
    `validated omena-semantic observation harness: blocked selectors=${blocked.selectorIdentity.rewriteBlockedSelectorCount} gaps=${blocked.blockingGaps.join(",")}\n`,
  );

  for (const entry of OMENA_SEMANTIC_OBSERVATION_CORPUS) {
    process.stdout.write(`== omena-semantic-observation-corpus:${entry.label} ==\n`);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry.contract);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runObservation({
      stylePath: entry.styleFilePath,
      styleSource: readFileSync(entry.styleFilePath, "utf8"),
      engineInput: snapshot.input,
    });
    // oxlint-disable-next-line eslint/no-await-in-loop
    const contract = await runObservationContract({
      stylePath: entry.styleFilePath,
      styleSource: readFileSync(entry.styleFilePath, "utf8"),
      engineInput: snapshot.input,
    });

    assert.equal(actual.product, "omena-semantic.theory-observation-harness");
    assert.equal(actual.graphProduct, "omena-semantic.style-semantic-graph");
    assert.equal(actual.selectorIdentity.status, entry.expected.selectorIdentityStatus);
    assert.equal(actual.sourceEvidence.status, entry.expected.sourceEvidenceStatus);
    assert.equal(actual.downstreamReadiness.status, entry.expected.downstreamReadinessStatus);
    assert.ok(
      actual.selectorIdentity.observedSelectorCount >= entry.expected.minSelectorCount,
      `${entry.label}: expected at least ${entry.expected.minSelectorCount} selectors, got ${actual.selectorIdentity.observedSelectorCount}`,
    );
    assert.ok(
      actual.sourceEvidence.expressionCount >= entry.expected.minExpressionCount,
      `${entry.label}: expected at least ${entry.expected.minExpressionCount} expressions, got ${actual.sourceEvidence.expressionCount}`,
    );
    assert.equal(actual.sourceEvidence.cmeCoupled, true);
    assert.equal(actual.couplingBoundary.cmeCoupledObservationCount, 2);
    assert.equal(actual.couplingBoundary.splitRecommendation, "keep-integrated-observe-boundary");
    assert.equal(contract.product, "omena-semantic.theory-observation-contract");
    assert.equal(contract.observationProduct, actual.product);
    assert.equal(contract.ready, entry.expected.ready);
    assert.equal(contract.publishReady, entry.expected.publishReady);
    assert.deepEqual(contract.publishBlockingGaps, entry.expected.publishBlockingGaps);
    assert.deepEqual(contract.observationGaps, entry.expected.observationGaps);
    assert.equal(contract.selectorIdentityStatus, actual.selectorIdentity.status);
    assert.equal(contract.sourceEvidenceStatus, actual.sourceEvidence.status);
    assert.equal(contract.downstreamReadinessStatus, actual.downstreamReadiness.status);
    process.stdout.write(
      [
        "validated external observation corpus:",
        `selectors=${actual.selectorIdentity.observedSelectorCount}`,
        `expressions=${actual.sourceEvidence.expressionCount}`,
        `status=${actual.downstreamReadiness.status}`,
        `publishReady=${contract.publishReady}`,
      ].join(" "),
    );
    process.stdout.write("\n\n");
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
