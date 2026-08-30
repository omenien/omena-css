import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES,
  RUST_PRODUCT_TEST_SHARDS,
  rustProductTestCargoInvocations,
} from "./lib/rust-product-test-plan";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const injectDefaultOn = process.argv.includes("--inject-default-on");
const injectSilentSkip = process.argv.includes("--inject-silent-skip");
const injectSecondOwner = process.argv.includes("--inject-second-owner");
const injectUndeclaredRefusal = process.argv.includes("--inject-undeclared-refusal");

const cargoManifestPaths = execFileSync("git", ["ls-files", "rust/**/Cargo.toml"], {
  cwd: repoRoot,
  encoding: "utf8",
})
  .trim()
  .split("\n")
  .filter(Boolean);
const manifests = new Map(
  cargoManifestPaths.map((relativePath) => [
    relativePath,
    readFileSync(path.join(repoRoot, relativePath), "utf8"),
  ]),
);

const bridgeManifestPath = "rust/crates/omena-bridge/Cargo.toml";
const queryManifestPath = "rust/crates/omena-query/Cargo.toml";
const cliManifestPath = "rust/crates/omena-cli/Cargo.toml";
const lspManifestPath = "rust/crates/omena-lsp-server/Cargo.toml";
const napiManifestPath = "rust/crates/omena-napi/Cargo.toml";
if (injectDefaultOn) {
  manifests.set(
    bridgeManifestPath,
    requiredManifest(bridgeManifestPath).replace("default = []", 'default = ["sif-attestation"]'),
  );
}
if (injectSecondOwner) {
  manifests.set(
    napiManifestPath,
    `${requiredManifest(napiManifestPath)}\n# injected owner\nsif-attestation = []\n`,
  );
}

const bridgeManifest = requiredManifest(bridgeManifestPath);
const queryManifest = requiredManifest(queryManifestPath);
const cliManifest = requiredManifest(cliManifestPath);
const lspManifest = requiredManifest(lspManifestPath);
const napiManifest = requiredManifest(napiManifestPath);

assert.match(bridgeManifest, /^default = \[\]$/m, "SIF attestation must remain off by default");
assert.match(
  bridgeManifest,
  /^sif-attestation = \["dep:sha2", "dep:sigstore-verify"\]$/m,
  "the bridge feature must own both host-only cryptographic dependencies",
);
assert.match(bridgeManifest, /^sha2 = \{ workspace = true, optional = true \}$/m);
assert.match(
  bridgeManifest,
  /^sigstore-verify = \{ version = "0\.11\.0", default-features = false, optional = true \}$/m,
);
assert.match(
  queryManifest,
  /^sif-attestation = \["omena-bridge\/sif-attestation"\]$/m,
  "omena-query must forward the bridge feature without enabling it by default",
);
assert.match(
  cliManifest,
  /omena-query = \{[\s\S]*?features = \[[\s\S]*?"sif-attestation",[\s\S]*?\]\s*\}/m,
  "the CLI dependency must be the single product owner that enables attestation",
);
assert.doesNotMatch(lspManifest, /sif-attestation/);
assert.doesNotMatch(napiManifest, /sif-attestation/);

const workspaceProductTestShard = RUST_PRODUCT_TEST_SHARDS.find(
  (shard) => shard.workspaceRemainder,
);
assert.ok(workspaceProductTestShard, "the product-test plan must own the workspace remainder");
assert.deepEqual(RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES, ["omena-lsp-server"]);
const workspaceProductTestInvocations = rustProductTestCargoInvocations(workspaceProductTestShard);
assert.ok(
  workspaceProductTestInvocations.some(
    (invocation) =>
      invocation.args.includes("--workspace") &&
      invocation.args.some(
        (value, index) =>
          value === "--exclude" && invocation.args[index + 1] === "omena-lsp-server",
      ),
  ),
  "workspace-wide all-features tests must not unify the CLI-owned attestation feature into LSP",
);
assert.ok(
  workspaceProductTestInvocations.some((invocation) =>
    invocation.args.some(
      (value, index) => value === "-p" && invocation.args[index + 1] === "omena-lsp-server",
    ),
  ),
  "LSP must retain a standalone all-features product-test invocation",
);

const featureMentioningManifests = [...manifests]
  .filter(([, source]) => source.includes("sif-attestation"))
  .map(([relativePath]) => relativePath)
  .sort();
assert.deepEqual(featureMentioningManifests, [
  bridgeManifestPath,
  cliManifestPath,
  queryManifestPath,
]);

const bridgeLib = readFileSync(path.join(repoRoot, "rust/crates/omena-bridge/src/lib.rs"), "utf8");
const signatureSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-bridge/src/external_sif_signature.rs"),
  "utf8",
);
let resolutionSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-bridge/src/style_resolution.rs"),
  "utf8",
);
if (injectSilentSkip) {
  resolutionSource = resolutionSource.replace(
    "Err(OmenaBridgeExternalSifShardRefusalV1::AttestationVerificationUnavailable)",
    "Ok(())",
  );
}
assert.match(
  bridgeLib,
  /#\[cfg\(any\(feature = "sif-attestation", test\)\)\]\s*mod external_sif_signature;/m,
);
assert.equal(countLiteral(signatureSource, "sigstore_verify"), 5);
assert.match(
  resolutionSource,
  /#\[cfg\(not\(any\(feature = "sif-attestation", test\)\)\)\]\s*fn verify_recorded_shard_verdict\([\s\S]*?Err\(OmenaBridgeExternalSifShardRefusalV1::AttestationVerificationUnavailable\)\s*\}/m,
  "feature-off verification must return the typed refusal",
);
assert.match(
  resolutionSource,
  /Err\(OmenaBridgeExternalSifShardRefusalV1::AttestationVerificationUnavailable\) => \{[\s\S]*?return Err\(format!\([\s\S]*?OmenaBridgeExternalSifShardRefusalV1::AttestationVerificationUnavailable\.code\(\)/m,
  "fresh generation must propagate the unavailable capability instead of silently downgrading",
);
assert.match(
  resolutionSource,
  /Self::AttestationVerificationUnavailable => "attestationVerificationUnavailable"/,
);

let semverIntentSource = readFileSync(
  path.join(repoRoot, "rust/omena-rust-semver-intent.json"),
  "utf8",
);
if (injectUndeclaredRefusal) {
  semverIntentSource = semverIntentSource.replace(
    "variant OmenaBridgeExternalSifShardRefusalV1:AttestationVerificationUnavailable",
    "variant OmenaBridgeExternalSifShardRefusalV1:InjectedMissingDeclaration",
  );
}
const semverIntent = JSON.parse(semverIntentSource) as {
  readonly intents: readonly {
    readonly crate: string;
    readonly expectedFailures: readonly {
      readonly lint: string;
      readonly evidenceNeedles: readonly string[];
    }[];
  }[];
};
const bridgeIntent = semverIntent.intents.find((intent) => intent.crate === "omena-bridge");
assert.ok(bridgeIntent, "omena-bridge requires an explicit pre-1.0 semver intent");
assert.ok(
  bridgeIntent.expectedFailures.some(
    (failure) =>
      failure.lint === "enum_variant_added" &&
      failure.evidenceNeedles.includes(
        "variant OmenaBridgeExternalSifShardRefusalV1:AttestationVerificationUnavailable",
      ),
  ),
  "the typed refusal variant must be bound to its cargo-semver-checks evidence",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-bridge.sif-attestation-feature-check",
      defaultEnabled: false,
      owner: "omena-cli",
      forwardingCrate: "omena-query",
      featureMentioningManifests,
      featureOwnedDependencies: ["sha2", "sigstore-verify"],
      featureOffRefusal: "attestationVerificationUnavailable",
      lspFeatureEnabled: false,
      napiFeatureEnabled: false,
      lspProductTestFeatureIsolation: true,
    },
    null,
    2,
  )}\n`,
);

function requiredManifest(relativePath: string): string {
  const source = manifests.get(relativePath);
  assert.ok(source, `missing tracked manifest ${relativePath}`);
  return source;
}

function countLiteral(source: string, value: string): number {
  return source.split(value).length - 1;
}
