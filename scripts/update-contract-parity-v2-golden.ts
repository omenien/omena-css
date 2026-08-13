import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
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
  mkdirSync(fixturesRoot, { recursive: true });
  const writtenFixturePaths: string[] = [];

  for (const entry of selectedEntries) {
    // oxlint-disable-next-line eslint/no-await-in-loop
    const snapshot = await buildContractParitySnapshot(entry);
    const normalized = normalizeContractParitySnapshot(snapshot, entry.workspace.workspaceRoot);
    const fixturePath = path.join(fixturesRoot, `${entry.label}.json`);
    writeFileSync(fixturePath, stableJsonStringify(normalized), "utf8");
    writtenFixturePaths.push(fixturePath);
    process.stdout.write(`updated ${entry.label}\n`);
  }

  execFileSync("pnpm", ["exec", "oxfmt", ...writtenFixturePaths], {
    stdio: "inherit",
  });
  assertContractParityV2FixtureSet(fixturesRoot, CONTRACT_PARITY_GOLDEN_CORPUS_V2);
  process.stdout.write(
    `updated contract-parity-v2 fixtures: selected=${selectedEntries.length} corpus=${CONTRACT_PARITY_GOLDEN_CORPUS_V2.length}\n`,
  );
})();
