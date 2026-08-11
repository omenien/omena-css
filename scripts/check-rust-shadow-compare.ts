import { strict as assert } from "node:assert";
import type { EngineInputV2 } from "../server/engine-core-ts/src/contracts";
import {
  CONTRACT_PARITY_CORPUS_V2,
  type ContractParityAuthoredMembershipV2,
} from "./contract-parity-corpus-v2";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  assertShadowSummaryMatch,
  deriveTsShadowSummary,
  runShadow,
  runShadowExpressionDomainEvaluatorCandidatesInput,
  runShadowSourceResolutionCandidatesInput,
} from "./rust-shadow-shared";

const OVERLAP_PARITY_LABEL = "source-prefix-suffix-overlap-parity-v2";

void (async () => {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== rust-shadow-compare:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveTsShadowSummary(snapshot);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runShadow(snapshot);

    assertShadowSummaryMatch(entry.label, actual, expected);

    if (entry.authoredMembership) {
      if (entry.label === OVERLAP_PARITY_LABEL) {
        // oxlint-disable-next-line eslint/no-await-in-loop
        await assertRustOverlapDefaults(snapshot.input, entry.authoredMembership);
      }
      // oxlint-disable-next-line eslint/no-await-in-loop
      const membership = await runShadowSourceResolutionCandidatesInput(
        authoredMembershipInput(snapshot.input, entry.authoredMembership),
      );
      assert.equal(
        membership.candidates.length,
        1,
        `${entry.label}: authored membership must reach exactly one Rust source candidate`,
      );
      assert.deepEqual(
        membership.candidates[0]?.selectorNames,
        entry.authoredMembership.selectorNames,
        `${entry.label}: Rust source-resolution membership must match the authored selector set`,
      );
    }

    process.stdout.write(
      `matched summary fields: sources=${actual.sourceCount} styles=${actual.styleCount} typeFacts=${actual.typeFactCount} queries=${actual.queryResultCount} findings=${actual.checkerTotalFindings}\n\n`,
    );
  }
})();

function authoredMembershipInput(
  input: EngineInputV2,
  expected: ContractParityAuthoredMembershipV2,
): EngineInputV2 {
  assert.equal(input.typeFacts.length, 1, "authored membership requires one type-fact entry");
  return {
    ...input,
    typeFacts: input.typeFacts.map(({ controlFlowGraph: _controlFlowGraph, ...entry }) => ({
      ...entry,
      facts: {
        kind: "constrained",
        constraintKind: "prefixSuffix",
        prefix: expected.valuePrefix,
        suffix: expected.valueSuffix,
        minLen: expected.valueMinLen,
      },
    })),
  };
}

async function assertRustOverlapDefaults(
  input: EngineInputV2,
  expected: ContractParityAuthoredMembershipV2,
): Promise<void> {
  const prefixSuffix = await runShadowExpressionDomainEvaluatorCandidatesInput(
    authoredOverlapDefaultInput(input, expected, "prefixSuffix"),
  );
  assertRustOverlapDefaultValue(prefixSuffix, "prefixSuffix", expected);

  const composite = await runShadowExpressionDomainEvaluatorCandidatesInput(
    authoredOverlapDefaultInput(input, expected, "composite"),
  );
  assertRustOverlapDefaultValue(composite, "composite", expected);
}

function authoredOverlapDefaultInput(
  input: EngineInputV2,
  expected: ContractParityAuthoredMembershipV2,
  constraintKind: "prefixSuffix" | "composite",
): EngineInputV2 {
  const edgeChars = Array.from(new Set(Array.from(expected.valuePrefix + expected.valueSuffix)))
    .toSorted()
    .join("");
  return {
    ...input,
    typeFacts: input.typeFacts.map(({ controlFlowGraph: _controlFlowGraph, ...entry }) => ({
      ...entry,
      facts: {
        kind: "constrained",
        constraintKind,
        prefix: expected.valuePrefix,
        suffix: expected.valueSuffix,
        ...(constraintKind === "composite" ? { charMust: edgeChars, charMay: edgeChars } : {}),
      },
    })),
  };
}

function assertRustOverlapDefaultValue(
  candidates: Awaited<ReturnType<typeof runShadowExpressionDomainEvaluatorCandidatesInput>>,
  expectedKind: "prefixSuffix" | "composite",
  expected: ContractParityAuthoredMembershipV2,
): void {
  assert.equal(candidates.results.length, 1, `${expectedKind}: Rust default probe candidate count`);
  const value = candidates.results[0]?.payload.valueDomainProvenanceTree.value as
    | Readonly<Record<string, unknown>>
    | undefined;
  assert.deepEqual(
    {
      kind: value?.kind,
      prefix: value?.prefix,
      suffix: value?.suffix,
      minLength: value?.minLength,
    },
    {
      kind: expectedKind,
      prefix: expected.valuePrefix,
      suffix: expected.valueSuffix,
      minLength: expected.valueMinLen,
    },
    `${expectedKind}: Rust byte-domain default overlap must match the authored ASCII minimum`,
  );
}
