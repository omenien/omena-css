import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workflowPath = path.join(repoRoot, ".github/workflows/sif-keyless-attestation.yml");
const args = new Set(process.argv.slice(2).filter((arg) => arg !== "--"));
const injectUnconsumedShardBatch = args.has("--inject-unconsumed-shard-batch");
const injectUnboundedPublicName = args.has("--inject-unbounded-public-name");
assert.deepEqual(
  [...args].filter(
    (arg) =>
      arg.startsWith("--") &&
      arg !== "--inject-unconsumed-shard-batch" &&
      arg !== "--inject-unbounded-public-name",
  ),
  [],
  "unknown T3 keyless workflow gate option",
);
let workflow = readFileSync(workflowPath, "utf8");
if (injectUnconsumedShardBatch) {
  workflow += '\n# dist/sif/omena-sif-shard-batch.json requestedTrustTier:"t3"\n';
}
if (injectUnboundedPublicName) {
  workflow = workflow.replace(
    'if [[ "${normalized_name}" =~ g[^[:alnum:]]*[0-9]+|goal[^[:alnum:]]*[0-9]+|stage[^[:alnum:]]*[0-9]+|redproof|pw[^[:alnum:]]*[0-9]+|cd[^[:alnum:]]*[0-9]+|wave[^[:alnum:]]*[0-9]+ ]]; then',
    "if false; then",
  );
}
const packageJson = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
  readonly scripts: Record<string, string>;
};

assert.match(workflow, /^name:\s*SIF Keyless Attestation$/m);
assert.match(workflow, /^\s*workflow_dispatch:\s*$/m);
for (const input of ["ref", "manifest_path", "source_path", "canonical_url", "output_name"]) {
  assert.match(workflow, new RegExp(`^\\s{6}${input}:\\s*$`, "m"));
}

assert.match(workflow, /^permissions:\s*$/m);
assert.match(workflow, /^\s{2}contents:\s*read\s*$/m);
assert.match(workflow, /^\s{2}id-token:\s*write\s*$/m);
assert.match(workflow, /^\s{2}attestations:\s*write\s*$/m);

// Supply-chain intent: the action must be pinned to a full commit SHA. Dependabot
// legitimately rotates the SHA (grouped, cooled-down), so asserting one frozen
// value just rots in the release tier — assert the pin FORM instead.
assert.match(
  workflow,
  /actions\/checkout@[0-9a-f]{40}\b/,
  "SIF attestation workflow must use a SHA-pinned checkout action",
);
const checkoutIndex = workflow.indexOf("- uses: actions/checkout@");
const provenanceGuardIndex = workflow.indexOf(
  "- name: Verify attestation checkout matches provenance source",
);
const rustSetupIndex = workflow.indexOf("- uses: ./.github/actions/setup-rust-pinned");
const attestationIndex = workflow.indexOf("- name: Attest SIF build provenance");
assert.ok(
  checkoutIndex >= 0 &&
    checkoutIndex < provenanceGuardIndex &&
    provenanceGuardIndex < rustSetupIndex &&
    rustSetupIndex < attestationIndex,
  "SIF provenance guard must run immediately after checkout and before setup, generation, or attestation",
);
for (const guardLine of [
  "OMENA_PROVENANCE_SOURCE_SHA: ${{ github.sha }}",
  'if [[ ! "${OMENA_PROVENANCE_SOURCE_SHA}" =~ ^[0-9a-f]{40}$ ]]; then',
  'checked_out_sha="$(git rev-parse HEAD)"',
  'if [[ "${checked_out_sha}" != "${OMENA_PROVENANCE_SOURCE_SHA}" ]]; then',
  'echo "publish provenance source mismatch" >&2',
  'echo "publish provenance source verified: ${checked_out_sha}"',
]) {
  assert.ok(workflow.includes(guardLine), `SIF provenance guard is missing ${guardLine}`);
}
assert.equal(
  workflow.includes("exit 0"),
  false,
  "SIF provenance guard must not turn a mismatch refusal into success",
);
assert.ok(
  workflow.includes("uses: ./.github/actions/setup-rust-pinned"),
  "SIF attestation workflow must use the pinned Rust toolchain setup",
);
assert.ok(
  workflow.includes("cargo build --manifest-path rust/Cargo.toml -p omena-cli --release --locked"),
  "SIF attestation workflow must build the release CLI from the checked-out ref",
);
assert.ok(
  workflow.includes("./rust/target/release/omena sif generate"),
  "SIF attestation workflow must generate the SIF through the shipped CLI surface",
);
assert.ok(
  workflow.includes("./rust/target/release/omena sif generate-attestation-subject"),
  "SIF attestation workflow must generate the signed subject through the shipped CLI surface",
);
assert.ok(
  workflow.includes("--trust-tier t3"),
  "SIF attestation workflow must bind the published advisory tier into the signed subject",
);
assert.ok(
  workflow.includes("manifest_path is mutually exclusive with source_path and canonical_url"),
  "SIF attestation workflow must keep manifest and single-source modes mutually exclusive",
);
assert.ok(
  workflow.includes('jq -e \'type == "array" and length > 0'),
  "SIF attestation workflow must validate the manifest shape before generating artifacts",
);
assert.ok(
  workflow.includes("done < <(jq -c '.[]' \"${MANIFEST_PATH}\")"),
  "SIF attestation workflow must iterate every manifest entry",
);
assert.ok(
  workflow.includes('sif_args+=(--sif "${output_path}")'),
  "SIF attestation workflow must pass every generated SIF into lock packaging",
);
assert.ok(
  workflow.includes("./rust/target/release/omena lock update"),
  "SIF attestation workflow must package a lock entry beside the generated SIF",
);
assert.ok(
  workflow.includes("--lockfile dist/sif/omena.lock"),
  "SIF attestation workflow must write a distributable omena.lock beside the SIF",
);
assert.ok(
  workflow.includes('.entries | length >= 1 and all(.[]; .trustTier == "t1"'),
  "SIF attestation workflow must validate every generated lock entry trust tier",
);
assert.ok(
  !workflow.includes("omena-sif-shard-batch"),
  "SIF attestation workflow must not publish the unconsumed shard batch",
);
assert.ok(
  !workflow.includes('requestedTrustTier:"t3"'),
  "SIF attestation workflow must not label T1 lock entries as requested T3 shards",
);
assert.match(
  workflow,
  /actions\/attest-build-provenance@[0-9a-f]{40}\b/,
  "SIF attestation workflow must use the pinned keyless provenance action",
);
assert.match(workflow, /subject-path:\s*\|\s*\n\s+dist\/sif\/\*\.attestation-subject\.json/);
assert.ok(
  !workflow.includes("subject-path: |\n            dist/sif/*.sif.json"),
  "raw SIF bytes must not be the elevated attestation subject",
);
assert.match(
  workflow,
  /actions\/upload-artifact@[0-9a-f]{40}\b/,
  "SIF attestation workflow must publish the generated SIF artifact for review",
);
assert.match(
  workflow,
  /path:\s*\|\s*\n\s+dist\/sif\/\*\.sif\.json\s*\n\s+dist\/sif\/\*\.attestation-subject\.json\s*\n\s+dist\/sif\/omena\.lock\s*\n\s+dist\/sif\/omena\.lock\.report\.json/,
);
assert.match(workflow, /if-no-files-found:\s*error/);

assert.ok(
  !workflow.includes("secrets."),
  "SIF keyless attestation must not depend on long-lived repository secrets",
);
assert.ok(
  workflow.includes("must be a repository-relative path without '..'"),
  "SIF attestation workflow must reject path traversal and absolute source paths",
);
assert.ok(
  workflow.includes("must resolve inside the checked-out repository"),
  "SIF attestation workflow must reject symlink escapes outside the repository",
);
assert.ok(
  workflow.includes("output_name must contain only letters"),
  "SIF attestation workflow must constrain artifact output names",
);
assert.ok(
  workflow.includes(
    'if [[ "${normalized_name}" =~ g[^[:alnum:]]*[0-9]+|goal[^[:alnum:]]*[0-9]+|stage[^[:alnum:]]*[0-9]+|redproof|pw[^[:alnum:]]*[0-9]+|cd[^[:alnum:]]*[0-9]+|wave[^[:alnum:]]*[0-9]+ ]]; then',
  ),
  "SIF attestation workflow must reject internal program identifiers anywhere in public names",
);
assert.ok(
  workflow.includes('validate_public_name output_name "${output_name}"'),
  "SIF attestation workflow must validate the public artifact name",
);
assert.ok(
  workflow.includes('validate_public_name canonical_url "${canonical_url}"'),
  "SIF attestation workflow must validate the signed canonical URL",
);
const staticPublicNames = [...workflow.matchAll(/^\s*(?:name|default):\s*(.+)$/gmu)]
  .map((match) => match[1]?.trim() ?? "")
  .join("\n");
assert.doesNotMatch(
  staticPublicNames,
  /g[^a-z0-9]*[0-9]+|goal[^a-z0-9]*[0-9]+|stage[^a-z0-9]*[0-9]+|redproof|pw[^a-z0-9]*[0-9]+|cd[^a-z0-9]*[0-9]+|wave[^a-z0-9]*[0-9]+/imu,
  "workflow and attestation subjects must not publish internal identifier-shaped names",
);
const forbiddenPublicName =
  /g[^a-z0-9]*[0-9]+|goal[^a-z0-9]*[0-9]+|stage[^a-z0-9]*[0-9]+|redproof|pw[^a-z0-9]*[0-9]+|cd[^a-z0-9]*[0-9]+|wave[^a-z0-9]*[0-9]+/iu;
for (const leakedName of [
  "g124shardverifier",
  "stage5-shard",
  "redproof-shard",
  "goal124-shard",
  "pw13",
  "cd124",
  "wave12",
  "g-124-shard",
  "g_124",
  "g.124-shard",
  "g--124-shard",
]) {
  assert.match(
    leakedName,
    forbiddenPublicName,
    `workflow-specific public-name policy must reject ${leakedName}`,
  );
}
for (const semanticName of ["sif-shard-verifier", "published-style-interface", "release-shard"]) {
  assert.doesNotMatch(
    semanticName,
    forbiddenPublicName,
    `workflow-specific public-name policy must permit ${semanticName}`,
  );
}

const boundary = packageJson.scripts["check:rust-omena-sif-boundary"];
assert.ok(boundary, "package.json must define check:rust-omena-sif-boundary");
assert.ok(
  boundary.includes("check:rust-omena-sif-t3-keyless-workflow"),
  "rust/omena-sif/boundary must include the T3 keyless workflow gate",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-sif.t3-keyless-workflow",
      workflow: ".github/workflows/sif-keyless-attestation.yml",
      keyless: true,
      longLivedSecrets: false,
      generationSurface: "omena sif generate",
      lockSurface: "omena lock update",
      attestationSubject: "dist/sif/*.attestation-subject.json",
      unconsumedShardBatch: "not-published",
      provenanceSourceGuard: {
        sourceSha: "github.sha",
        sourceShaValidation: "full-lowercase-hex-40",
        mismatchExit: 1,
        beforeSetupAndAttestation: true,
      },
    },
    null,
    2,
  )}\n`,
);

// FALSIFIER: id=sif-t3-keyless-unconsumed-batch-reintroduction class=workflowMutation via=--inject-unconsumed-shard-batch expected=RED owner=omena-sif-keyless-workflow
// FALSIFIER: id=sif-keyless-public-name-policy class=workflowMutation via=--inject-unbounded-public-name expected=RED owner=omena-sif-keyless-workflow
