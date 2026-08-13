import { readFileSync } from "node:fs";
import path from "node:path";
import { CONTRACT_PARITY_GOLDEN_CORPUS_V2 } from "./contract-parity-golden-corpus-v2";
import {
  buildContractParitySnapshot,
  normalizeContractParitySnapshot,
  stableJsonStringify,
} from "./contract-parity-runtime";
import {
  assertContractParityV2FixtureSet,
  selectContractParityV2Entries,
} from "./contract-parity-v2-fixture-selection";

const fixturesRoot = path.join(process.cwd(), "test/_fixtures/contract-parity-v2");
const selectedEntries = selectContractParityV2Entries(
  CONTRACT_PARITY_GOLDEN_CORPUS_V2,
  process.argv.slice(2),
);

void (async () => {
  let exitCode = 0;

  assertContractParityV2FixtureSet(fixturesRoot, CONTRACT_PARITY_GOLDEN_CORPUS_V2);

  for (const entry of selectedEntries) {
    const fixturePath = path.join(fixturesRoot, `${entry.label}.json`);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const normalized = normalizeContractParitySnapshot(snapshot, entry.workspace.workspaceRoot);
    const actual = stableJsonStringify(normalized);
    const expected = stableJsonStringify(JSON.parse(readFileSync(fixturePath, "utf8")) as unknown);

    if (actual !== expected) {
      exitCode = 1;
      process.stderr.write(`mismatch ${entry.label}: ${fixturePath}\n`);
      continue;
    }

    if (entry.authoredMembership) {
      assertAuthoredMembership(entry.label, normalized, entry.authoredMembership);
    }

    process.stdout.write(`ok ${entry.label}\n`);
  }

  process.stdout.write(
    `checked contract-parity-v2 fixtures: selected=${selectedEntries.length} corpus=${CONTRACT_PARITY_GOLDEN_CORPUS_V2.length}\n`,
  );
  process.exitCode = exitCode;
})();

function assertAuthoredMembership(
  label: string,
  snapshot: Awaited<ReturnType<typeof buildContractParitySnapshot>>,
  expected: NonNullable<(typeof CONTRACT_PARITY_GOLDEN_CORPUS_V2)[number]["authoredMembership"]>,
): void {
  const membershipQueries = snapshot.output.queryResults.filter(
    (query) =>
      query.kind === "expression-semantics" || query.kind === "source-expression-resolution",
  );
  if (membershipQueries.length !== 2) {
    throw new Error(`${label}: authored membership expected two source query results`);
  }

  for (const query of membershipQueries) {
    const actual = {
      valueConstraintKind:
        query.payload.valueConstraintKind ?? query.payload.valueCertaintyConstraintKind,
      valuePrefix: query.payload.valuePrefix,
      valueSuffix: query.payload.valueSuffix,
      valueMinLen: query.payload.valueMinLen,
      selectorNames: query.payload.selectorNames,
    };
    if (stableJsonStringify(actual) !== stableJsonStringify(expected)) {
      throw new Error(
        `${label}:${query.kind}: authored membership mismatch\nexpected=${stableJsonStringify(expected)}actual=${stableJsonStringify(actual)}`,
      );
    }
  }
}
