import { readdirSync } from "node:fs";
import path from "node:path";
import type { ContractParityEntryV2 } from "./contract-parity-corpus-v2";

export function selectContractParityV2Entries(
  corpus: readonly ContractParityEntryV2[],
  argv: readonly string[],
): readonly ContractParityEntryV2[] {
  const corpusByLabel = new Map<string, ContractParityEntryV2>();
  for (const entry of corpus) {
    if (corpusByLabel.has(entry.label)) {
      throw new Error(`duplicate contract-parity-v2 corpus label: ${entry.label}`);
    }
    corpusByLabel.set(entry.label, entry);
  }

  const requestedLabels: string[] = [];
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg !== "--label") throw new Error(`unknown argument: ${arg}`);
    const label = argv[index + 1];
    if (!label || label.startsWith("--")) throw new Error("--label requires a corpus label");
    requestedLabels.push(label);
    index += 1;
  }

  if (requestedLabels.length === 0) return corpus;
  if (new Set(requestedLabels).size !== requestedLabels.length) {
    throw new Error(`duplicate --label selection: ${requestedLabels.join(", ")}`);
  }

  return requestedLabels.map((label) => {
    const entry = corpusByLabel.get(label);
    if (!entry) throw new Error(`unknown contract-parity-v2 corpus label: ${label}`);
    return entry;
  });
}

export function assertContractParityV2FixtureSet(
  fixturesRoot: string,
  corpus: readonly ContractParityEntryV2[],
): void {
  const expected = corpus.map((entry) => entry.label).toSorted();
  const actual = readdirSync(fixturesRoot)
    .filter((fileName) => path.extname(fileName) === ".json")
    .map((fileName) => path.basename(fileName, ".json"))
    .toSorted();

  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `contract-parity-v2 fixture set mismatch\nexpected=${JSON.stringify(expected)}\nactual=${JSON.stringify(actual)}`,
    );
  }
}
