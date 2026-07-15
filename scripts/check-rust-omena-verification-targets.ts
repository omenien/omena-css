import { strict as assert } from "node:assert";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Availability = "available" | "not-yet" | "skipped";
type Scope = "userWorkspace" | "engineSelf";

interface EvidenceReference {
  readonly path: string;
  readonly symbol: string;
}

interface VerificationTarget {
  readonly id: string;
  readonly scope: Scope;
  readonly availability: Availability;
  readonly executor: string | null;
  readonly description: string;
  readonly evidence: readonly EvidenceReference[];
  readonly limitation: string;
}

interface VerificationManifest {
  readonly schemaVersion: string;
  readonly product: string;
  readonly targets: readonly VerificationTarget[];
  readonly ciAdapters: readonly { readonly verb: string; readonly executor: string }[];
}

interface VerbManifest {
  readonly verbs: readonly { readonly verb: string; readonly status: string }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const engineScopeMarker = ["omena-verification", "scope: engine-self"].join("-");
const manifest = readJson<VerificationManifest>("rust/crates/omena-cli/verification-targets.json");
const verbManifest = readJson<VerbManifest>("rust/crates/omena-cli/verb-census.json");
const verificationSource = read("rust/crates/omena-cli/src/verification/workspace.rs");
const productionVerificationSource = verificationSource.split("#[cfg(test)]", 1)[0] ?? "";
const ciSource = read("rust/crates/omena-cli/src/ci.rs");
const productionCiSource = ciSource.split("#[cfg(test)]", 1)[0] ?? "";

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.verification-targets");
assert.ok(manifest.targets.length > 0, "verification target manifest must not be empty");

const ids = manifest.targets.map((target) => target.id);
assert.equal(new Set(ids).size, ids.length, "verification target ids must be unique");
assert.ok(
  manifest.targets.some(
    (target) => target.scope === "userWorkspace" && target.availability === "available",
  ),
  "at least one user-workspace verification target must be available",
);

for (const target of manifest.targets) {
  assert.ok(target.description.trim().length > 0, `${target.id} must describe user meaning`);
  assert.ok(target.limitation.trim().length > 0, `${target.id} must state an honest limitation`);
  assert.ok(target.evidence.length > 0, `${target.id} must cite an existing mechanism`);
  assert.equal(
    target.availability === "available",
    target.executor !== null,
    `${target.id} availability must match executable wiring`,
  );
  for (const evidence of target.evidence) {
    const source = read(evidence.path);
    assert.ok(
      source.includes(evidence.symbol),
      `${target.id} evidence symbol ${evidence.symbol} is missing from ${evidence.path}`,
    );
  }
}

const scriptsDirectory = path.join(repoRoot, "scripts");
const scannedEngineScripts = readdirSync(scriptsDirectory)
  .filter((fileName) => fileName.startsWith("check-rust-") && fileName.endsWith(".ts"))
  .map((fileName) => `scripts/${fileName}`)
  .filter((relativePath) => read(relativePath).includes(engineScopeMarker))
  .sort();
const manifestedEngineScripts = manifest.targets
  .filter((target) => target.scope === "engineSelf")
  .flatMap((target) => target.evidence.map((evidence) => evidence.path))
  .sort();
assert.deepEqual(
  manifestedEngineScripts,
  scannedEngineScripts,
  "engine-self roster must be derived from scope-marked developer gates",
);

const engineScriptSet = new Set(scannedEngineScripts);
const misclassifiedUserTargets = manifest.targets
  .filter((target) => target.scope === "userWorkspace")
  .filter((target) => target.evidence.some((evidence) => engineScriptSet.has(evidence.path)))
  .map((target) => target.id);
assert.deepEqual(
  misclassifiedUserTargets,
  [],
  "developer-harness gates must never be classified as user-workspace verification",
);

const verbStatuses = new Map(verbManifest.verbs.map((row) => [row.verb, row.status]));
for (const adapter of manifest.ciAdapters) {
  assert.ok(verbStatuses.has(adapter.verb), `CI adapter ${adapter.verb} must name a product verb`);
  assert.equal(
    verbStatuses.get(adapter.verb),
    "wired",
    `CI adapter ${adapter.verb} must only activate after direct product wiring`,
  );
  assert.ok(adapter.executor.length > 0, `CI adapter ${adapter.verb} must name an executor`);
}
assert.equal(
  new Set(manifest.ciAdapters.map((adapter) => adapter.verb)).size,
  manifest.ciAdapters.length,
  "CI adapter verbs must be unique",
);
for (const forbidden of [
  "omena_parser::",
  "Regex::new",
  "Command::new",
  "fs::write",
  "write_artifact",
  "print_json",
]) {
  assert.ok(
    !productionVerificationSource.includes(forbidden),
    `verification orchestration must consume existing mechanisms instead of introducing ${forbidden}`,
  );
}
assert.ok(
  productionCiSource.includes("compose_components(root.as_path(), manifest.verbs, &adapters)"),
  "CI command must pass the committed product verb census to the component composer",
);
assert.ok(
  productionCiSource.includes("for row in rows"),
  "CI component composition must iterate every supplied product verb row",
);
assert.ok(
  productionCiSource.includes("adapters.get(row.verb.as_str())"),
  "CI composition must resolve check adapters from the verification manifest",
);
assert.ok(
  productionCiSource.includes("the wired product verb has no read-only CI check contract"),
  "wired verbs without a check contract must remain visible as skipped",
);
assert.ok(
  productionVerificationSource.includes("report.external_tool_evidence"),
  "external comparison must consume the witness produced by the shared Sass bridge",
);
assert.ok(
  productionVerificationSource.includes("translation_validation_binding(value)"),
  "translation-validation wording must consume the config vocabulary authority",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-verification-targets",
      targetCount: manifest.targets.length,
      availableUserTargetCount: manifest.targets.filter(
        (target) => target.scope === "userWorkspace" && target.availability === "available",
      ).length,
      engineSelfTargetCount: scannedEngineScripts.length,
      misclassifiedUserTargetCount: misclassifiedUserTargets.length,
      ciAdapterCount: manifest.ciAdapters.length,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
