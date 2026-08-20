// g131-S6: contract-parity smoke drivers (v1+v2 rows in one driver).
/* oxlint-disable no-await-in-loop */

import { CONTRACT_PARITY_CORPUS } from "../contract-parity-corpus-v1";
import { CONTRACT_PARITY_CORPUS_V2 } from "../contract-parity-corpus-v2";
import {
  buildContractParitySnapshot,
  buildContractParitySnapshotV1,
} from "../contract-parity-runtime";

async function run_contract_parity_v1_smoke(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS) {
    process.stdout.write(`== ${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshotV1(entry);

    process.stdout.write(
      `input: ${snapshot.input.sources.length} sources, ${snapshot.input.styles.length} styles, ${snapshot.input.typeFacts.length} type facts\n`,
    );
    process.stdout.write(
      `output: ${snapshot.output.queryResults.length} query results, ${snapshot.output.checkerReport.summary.total} findings\n\n`,
    );
  }
}

async function run_contract_parity_v2_smoke(): Promise<void> {
  for (const entry of CONTRACT_PARITY_CORPUS_V2) {
    process.stdout.write(`== ${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);

    process.stdout.write(
      `input: ${snapshot.input.sources.length} sources, ${snapshot.input.styles.length} styles, ${snapshot.input.typeFacts.length} type facts\n`,
    );
    process.stdout.write(
      `output: ${snapshot.output.queryResults.length} query results, ${snapshot.output.checkerReport.summary.total} findings\n\n`,
    );
  }
}

export const CONTRACT_PARITY_SMOKE_FAMILY: {
  readonly [slug: string]: () => Promise<void>;
} = {
  "contract-parity-v1-smoke": run_contract_parity_v1_smoke,
  "contract-parity-v2-smoke": run_contract_parity_v2_smoke,
};
