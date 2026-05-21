import { assertCheckerCanonicalCandidateEqual } from "../packages/cme-checker/src";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  deriveTsCheckerStyleUnusedCanonicalCandidate,
  STYLE_UNUSED_ENTRY,
} from "./rust-checker-style-unused-shared";
import { runShadowCheckerStyleUnusedCanonicalCandidate } from "./rust-shadow-shared";

void (async () => {
  process.stdout.write(`== rust-checker-style-unused:${STYLE_UNUSED_ENTRY.label} ==\n`);
  const snapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);
  const expected = deriveTsCheckerStyleUnusedCanonicalCandidate(snapshot);
  const actual = await runShadowCheckerStyleUnusedCanonicalCandidate(snapshot);
  assertCheckerCanonicalCandidateEqual(
    actual,
    expected,
    `${STYLE_UNUSED_ENTRY.label}: checker style-unused canonical candidate mismatch`,
  );
  process.stdout.write(
    `findings=${actual.summary.total} files=${actual.distinctFileCount} codes=${JSON.stringify(actual.codeCounts)}\n\n`,
  );
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
