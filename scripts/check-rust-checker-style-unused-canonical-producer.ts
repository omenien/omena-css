import { deepStrictEqual } from "node:assert";
import { buildCheckerBoundedGate } from "../packages/cme-checker/src";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  deriveTsCheckerStyleUnusedCanonicalCandidate,
  STYLE_UNUSED_ENTRY,
} from "./rust-checker-style-unused-shared";
import {
  runShadowCheckerStyleUnusedCanonicalProducer,
  type CheckerStyleUnusedCanonicalProducerSignalV0,
} from "./rust-shadow-shared";

void (async () => {
  process.stdout.write(`== rust-checker-style-unused-producer:${STYLE_UNUSED_ENTRY.label} ==\n`);
  const snapshot = await buildContractParitySnapshot(STYLE_UNUSED_ENTRY);
  const canonicalCandidate = deriveTsCheckerStyleUnusedCanonicalCandidate(snapshot);
  const actual = await runShadowCheckerStyleUnusedCanonicalProducer(snapshot);

  const expected: CheckerStyleUnusedCanonicalProducerSignalV0 = {
    schemaVersion: "0",
    inputVersion: canonicalCandidate.inputVersion,
    canonicalCandidate,
    boundedCheckerGate: buildCheckerBoundedGate("style-unused"),
  };

  deepStrictEqual(actual, expected, "checker style-unused canonical producer mismatch");
  process.stdout.write(
    `findings=${actual.canonicalCandidate.summary.total} releaseGate=${actual.boundedCheckerGate.includedInRustReleaseBundle}\n\n`,
  );
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
