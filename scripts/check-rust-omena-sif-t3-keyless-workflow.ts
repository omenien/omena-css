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
for (const input of ["ref", "source_path", "canonical_url", "output_name"]) {
  assert.match(workflow, new RegExp(`^\\s{6}${input}:\\s*$`, "m"));
}

assert.match(workflow, /^permissions:\s*$/m);
assert.match(workflow, /^\s{2}contents:\s*read\s*$/m);
assert.match(workflow, /^\s{2}id-token:\s*write\s*$/m);
assert.match(workflow, /^\s{2}attestations:\s*write\s*$/m);

assert.ok(
  workflow.includes("actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"),
  "SIF attestation workflow must use the pinned checkout action",
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
  workflow.includes("./rust/target/release/omena-cli sif generate"),
  "SIF attestation workflow must generate the SIF through the shipped CLI surface",
);
assert.ok(
  workflow.includes("actions/attest-build-provenance@e8998f949152b193b063cb0ec769d69d929409be"),
  "SIF attestation workflow must use the pinned keyless provenance action",
);
assert.match(workflow, /subject-path:\s*dist\/sif\/\*\.sif\.json/);
assert.ok(
  workflow.includes("actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"),
  "SIF attestation workflow must publish the generated SIF artifact for review",
);
assert.match(workflow, /if-no-files-found:\s*error/);

assert.ok(
  !workflow.includes("secrets."),
  "SIF keyless attestation must not depend on long-lived repository secrets",
);
assert.ok(
  workflow.includes("source_path must be a repository-relative path without"),
  "SIF attestation workflow must reject path traversal and absolute source paths",
);
assert.ok(
  workflow.includes("source_path must resolve inside the checked-out repository"),
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
      generationSurface: "omena-cli sif generate",
      attestationSubject: "dist/sif/*.sif.json",
    },
    null,
    2,
  )}\n`,
);
