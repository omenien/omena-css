import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

interface TideAuthorityContract {
  readonly schemaVersion: string;
  readonly testCount: number;
  readonly files: readonly {
    readonly path: string;
    readonly testCount: number;
    readonly sha256: string;
  }[];
  readonly futureHandoffRequirements: readonly string[];
}

const root = process.cwd();
const contractPath = "rust/crates/omena-lsp-server/tests/tide-sole-authority-contract.json";
const contract = JSON.parse(
  readFileSync(path.join(root, contractPath), "utf8"),
) as TideAuthorityContract;
const testPattern = /^\s*#\[test\]\s*$/gm;

assert.equal(contract.schemaVersion, "omena.tide-sole-authority.v0");
assert.deepEqual(
  contract.futureHandoffRequirements,
  ["unchangedTideTestCorpus", "flushTimeConeClosure", "zeroInterfaceOverheadAccepted"],
  "future authority handoff must remain separately gated",
);

let measuredTestCount = 0;
for (const file of contract.files) {
  const source = readFileSync(path.join(root, file.path), "utf8");
  const testCount = [...source.matchAll(testPattern)].length;
  const sha256 = createHash("sha256").update(source).digest("hex");
  assert.equal(testCount, file.testCount, `${file.path} Tide test count changed`);
  assert.equal(sha256, file.sha256, `${file.path} Tide test bytes changed`);
  measuredTestCount += testCount;
}
assert.equal(measuredTestCount, contract.testCount);
assert.equal(contract.testCount, 21);

const tideRoot = path.join(root, "rust/crates/omena-lsp-server/src/tide");
const tideSources = collectRustSources(tideRoot)
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");
assert.ok(
  !tideSources.includes("omena_reactive"),
  "Tide must not read from or delegate scheduling to the reactive observer",
);

console.log(
  JSON.stringify(
    {
      schemaVersion: contract.schemaVersion,
      product: "rust.omena-lsp-server.tide-sole-authority",
      files: contract.files.length,
      tests: measuredTestCount,
      reactiveReferencesFromTide: 0,
      futureHandoffRequirements: contract.futureHandoffRequirements,
    },
    null,
    2,
  ),
);

function collectRustSources(directory: string): string[] {
  return evidenceScanSurface
    .readdirSync(directory)
    .flatMap((entry) => {
      const candidate = path.join(directory, entry);
      return statSync(candidate).isDirectory() ? collectRustSources(candidate) : [candidate];
    })
    .filter((candidate) => candidate.endsWith(".rs"))
    .toSorted();
}
