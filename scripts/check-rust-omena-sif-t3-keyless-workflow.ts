import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workflowPath = path.join(repoRoot, ".github/workflows/sif-keyless-attestation.yml");
const workflow = readFileSync(workflowPath, "utf8");
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
assert.match(
  workflow,
  /actions\/attest-build-provenance@[0-9a-f]{40}\b/,
  "SIF attestation workflow must use the pinned keyless provenance action",
);
assert.match(
  workflow,
  /subject-path:\s*\|\s*\n\s+dist\/sif\/\*\.sif\.json\s*\n\s+dist\/sif\/omena\.lock/,
);
assert.match(
  workflow,
  /actions\/upload-artifact@[0-9a-f]{40}\b/,
  "SIF attestation workflow must publish the generated SIF artifact for review",
);
assert.match(
  workflow,
  /path:\s*\|\s*\n\s+dist\/sif\/\*\.sif\.json\s*\n\s+dist\/sif\/omena\.lock\s*\n\s+dist\/sif\/omena\.lock\.report\.json/,
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
      attestationSubject: "dist/sif/*.sif.json + dist/sif/omena.lock",
    },
    null,
    2,
  )}\n`,
);
