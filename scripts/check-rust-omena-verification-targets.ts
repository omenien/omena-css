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
  assert.ok(adapter.executor.length > 0, `CI adapter ${adapter.verb} must name an executor`);
}
assert.equal(
  new Set(manifest.ciAdapters.map((adapter) => adapter.verb)).size,
  manifest.ciAdapters.length,
  "CI adapter verbs must be unique",
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
