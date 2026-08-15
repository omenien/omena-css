import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type FixtureCase = {
  readonly id: string;
  readonly packageSelector: string;
  readonly source: string;
  readonly expectedDisposition?: "present" | "absent";
};

type AcquisitionReceipt = {
  readonly schemaVersion: string;
  readonly product: string;
  readonly acquisitionOwner: string;
  readonly networkAccess: string;
  readonly packageSpec: string;
  readonly npmMetadataPath: string;
  readonly provenanceDisposition: "present" | "absent";
  readonly provenanceCandidateCount: number;
  readonly command: readonly string[] | null;
};

const repoRoot = process.cwd();
const args = new Set(process.argv.slice(2).filter((argument) => argument !== "--"));
const injectDropAbsenceCase = args.has("--inject-drop-absence-case");
assert.deepEqual(
  [...args].filter(
    (argument) => argument.startsWith("--") && argument !== "--inject-drop-absence-case",
  ),
  [],
  "unknown npm provenance acquisition gate option",
);

const fixtureRegistry = JSON.parse(
  readFileSync(
    path.join(
      repoRoot,
      "rust/crates/omena-sif/tests/fixtures/npm-provenance-metadata-registry-v1.json",
    ),
    "utf8",
  ),
) as { readonly cases: readonly FixtureCase[] };
const cases = fixtureRegistry.cases.filter(
  (fixture) =>
    fixture.expectedDisposition !== undefined &&
    !(injectDropAbsenceCase && fixture.expectedDisposition === "absent"),
);
const workspace = mkdtempSync(path.join(tmpdir(), "omena-sif-npm-provenance-"));

try {
  const receipts: AcquisitionReceipt[] = [];
  for (const fixture of cases) {
    const inputPath = path.join(workspace, `${fixture.id}.input.json`);
    const outputPath = path.join(workspace, `${fixture.id}.metadata.json`);
    const receiptPath = path.join(workspace, `${fixture.id}.receipt.json`);
    writeFileSync(inputPath, fixture.source, "utf8");
    const result = spawnSync(
      process.execPath,
      [
        "--import",
        "tsx",
        "scripts/acquire-omena-sif-npm-provenance.ts",
        fixture.packageSelector,
        "--metadata-file",
        inputPath,
        "--output",
        outputPath,
        "--receipt",
        receiptPath,
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(
      result.status,
      0,
      `${fixture.id} acquisition failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
    );
    const receipt = JSON.parse(readFileSync(receiptPath, "utf8")) as AcquisitionReceipt;
    assert.equal(receipt.schemaVersion, "1", fixture.id);
    assert.equal(receipt.product, "omena-sif.npm-provenance-acquisition", fixture.id);
    assert.equal(receipt.acquisitionOwner, "operator-supplied-metadata", fixture.id);
    assert.equal(receipt.networkAccess, "none", fixture.id);
    assert.equal(receipt.packageSpec, fixture.packageSelector, fixture.id);
    assert.equal(receipt.npmMetadataPath, outputPath, fixture.id);
    assert.equal(receipt.provenanceDisposition, fixture.expectedDisposition, fixture.id);
    assert.equal(receipt.command, null, fixture.id);
    assert.deepEqual(
      JSON.parse(readFileSync(outputPath, "utf8")),
      JSON.parse(fixture.source),
      `${fixture.id} metadata must feed the disk-file CLI path without semantic rewriting`,
    );
    receipts.push(receipt);
  }
  assert.ok(
    receipts.some((receipt) => receipt.provenanceDisposition === "present") &&
      receipts.some((receipt) => receipt.provenanceDisposition === "absent"),
    "the fixture registry must exercise present and absent acquisition dispositions",
  );

  const acquisitionSource = readFileSync(
    path.join(repoRoot, "scripts/acquire-omena-sif-npm-provenance.ts"),
    "utf8",
  );
  assert.ok(
    acquisitionSource.includes('spawnSync("npm", ["view", options.packageSpec, "--json"]'),
    "online acquisition must delegate network ownership to the platform npm CLI",
  );
  for (const forbiddenClient of ["fetch(", "axios", "undici", "reqwest", "hyper"]) {
    assert.ok(
      !acquisitionSource.includes(forbiddenClient),
      `acquisition script must not introduce a direct network client: ${forbiddenClient}`,
    );
  }

  const packageJson = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
    readonly scripts: Readonly<Record<string, string>>;
  };
  assert.equal(
    packageJson.scripts["acquire:sif-npm-provenance"],
    "node --import tsx ./scripts/acquire-omena-sif-npm-provenance.ts",
  );
  assert.ok(
    packageJson.scripts["check:rust-omena-sif-boundary"]?.includes(
      "check:rust-omena-sif-npm-provenance-acquisition",
    ),
    "the SIF boundary must run the offline npm provenance acquisition gate",
  );
  const sassCompatibility = readFileSync(path.join(repoRoot, "docs/sass-compat.md"), "utf8");
  assert.ok(
    sassCompatibility.includes(
      "The acquisition receipt names `platform-npm-cli` as the network owner.",
    ),
    "the SIF user contract must name the external npm network owner",
  );
  assert.ok(
    sassCompatibility.includes("Native registry fetching\ninside Omena remains deferred."),
    "the SIF user contract must preserve the native-fetch deferral",
  );
  assert.ok(
    sassCompatibility.includes('provenanceDisposition: "absent"'),
    "the SIF user contract must explain the recorded absence disposition",
  );

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "1",
      product: "omena-sif.npm-provenance-acquisition-gate",
      fixtureCaseCount: receipts.length,
      presentCount: receipts.filter((receipt) => receipt.provenanceDisposition === "present")
        .length,
      absentCount: receipts.filter((receipt) => receipt.provenanceDisposition === "absent").length,
      liveNetworkRequests: 0,
      acquisitionOwner: "platform-npm-cli",
      nativeFetch: "deferred",
    })}\n`,
  );
} finally {
  rmSync(workspace, { recursive: true, force: true });
}

// FALSIFIER: id=sif-npm-provenance-acquisition-absence class=fixtureRegistryMutation via=--inject-drop-absence-case expected=RED owner=omena-sif
