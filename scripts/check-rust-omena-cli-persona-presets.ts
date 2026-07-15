import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type VerbStatus = "stub" | "reserved-alias" | "wired";

interface VerbManifest {
  readonly verbs: readonly {
    readonly verb: string;
    readonly status: VerbStatus;
  }[];
}

interface MissingCapability {
  readonly id: string;
  readonly owner: string;
  readonly availabilityCondition: string;
}

interface PersonaPreset {
  readonly id: string;
  readonly priority: number;
  readonly audience: string;
  readonly configPath: string;
  readonly verbs: readonly string[];
  readonly configGaps: readonly {
    readonly path: string;
    readonly owner: string;
    readonly availabilityCondition: string;
  }[];
  readonly missingCapabilities: readonly MissingCapability[];
}

interface PersonaManifest {
  readonly schemaVersion: string;
  readonly product: string;
  readonly consumption: {
    readonly kind: string;
    readonly syntax: string;
  };
  readonly presets: readonly PersonaPreset[];
}

const expectedPersonas = [
  ["workspace-maintenance", "largeFrontendMonorepo"],
  ["design-system-governance", "designSystemPlatform"],
  ["build-integration", "frameworkBundlerPluginAuthor"],
  ["migration-safety", "migrationToolAuthor"],
  ["assurance-gates", "highAssuranceCi"],
  ["semantic-research", "cssSassCompilerResearch"],
] as const;

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = readJson<PersonaManifest>("rust/crates/omena-cli/persona-presets.json");
const verbManifest = readJson<VerbManifest>("rust/crates/omena-cli/verb-census.json");
const personaSource = read("rust/crates/omena-cli/src/config/persona.rs");

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.persona-presets");
assert.deepEqual(manifest.consumption, {
  kind: "configExtends",
  syntax: 'extends = "omena:<preset-id>"',
});

const presets = manifest.presets.map((preset, index) =>
  process.env.OMENA_PERSONA_PRESET_TEST_INJECT_UNWIRED === "1" && index === 0
    ? { ...preset, verbs: [...preset.verbs, "check"] }
    : preset,
);
assert.deepEqual(
  presets.map(({ id, audience }) => [id, audience]),
  expectedPersonas,
  "persona roster and audience order are a committed product contract",
);
assert.deepEqual(
  presets.map(({ priority }) => priority),
  [1, 2, 3, 4, 5, 6],
  "persona priorities must be total and stable",
);

const sourceIds = extractPersonaIds(personaSource);
assert.deepEqual(
  presets.map(({ id }) => id),
  sourceIds,
  "the embedded resolver roster must match the persona manifest",
);
assert.equal(new Set(sourceIds).size, sourceIds.length, "persona ids must be unique");

const verbStatuses = new Map(verbManifest.verbs.map((row) => [row.verb, row.status]));
for (const preset of presets) {
  assert.ok(preset.verbs.length > 0, `${preset.id} must contain an executable verb bundle`);
  assert.equal(
    new Set(preset.verbs).size,
    preset.verbs.length,
    `${preset.id} must not repeat verbs`,
  );
  for (const verb of preset.verbs) {
    assert.equal(
      verbStatuses.get(verb),
      "wired",
      `${preset.id} may only include directly wired product verbs; ${verb} is not wired`,
    );
  }

  const configPath = path.join("rust/crates/omena-cli", preset.configPath);
  assert.ok(existsSync(path.join(repoRoot, configPath)), `${preset.id} config must exist`);
  assert.ok(read(configPath).trim().length > 0, `${preset.id} config must not be empty`);

  assert.equal(
    new Set(preset.configGaps.map(({ path: gapPath }) => gapPath)).size,
    preset.configGaps.length,
    `${preset.id} config gap paths must be unique`,
  );
  for (const gap of preset.configGaps) {
    assert.ok(gap.path.trim().length > 0, `${preset.id} config gap path must not be empty`);
    assert.ok(gap.owner.trim().length > 0, `${gap.path} must name its owner`);
    assert.ok(
      gap.availabilityCondition.trim().length > 0,
      `${gap.path} must state an availability condition`,
    );
  }

  assert.ok(
    preset.missingCapabilities.length > 0,
    `${preset.id} must state capabilities that the preset cannot supply yet`,
  );
  for (const capability of preset.missingCapabilities) {
    assert.ok(capability.id.trim().length > 0, `${preset.id} capability id must not be empty`);
    assert.ok(capability.owner.trim().length > 0, `${capability.id} must name its owner`);
    assert.ok(
      capability.availabilityCondition.trim().length > 0,
      `${capability.id} must state an availability condition`,
    );
  }
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-cli-persona-presets",
      presetCount: presets.length,
      audiences: presets.map(({ audience }) => audience),
      allVerbsDirectlyWired: true,
      missingCapabilityCount: presets.reduce(
        (count, preset) => count + preset.missingCapabilities.length,
        0,
      ),
      reportedConfigGapCount: presets.reduce(
        (count, preset) => count + preset.configGaps.length,
        0,
      ),
    },
    null,
    2,
  )}\n`,
);

function extractPersonaIds(source: string): readonly string[] {
  const declaration = "pub(super) const PERSONA_PRESET_IDS";
  const start = source.indexOf(declaration);
  assert.notEqual(start, -1, "persona resolver must expose its embedded roster");
  const end = source.indexOf("];", start);
  assert.ok(end > start, "persona resolver roster must be terminated");
  return [...source.slice(start, end).matchAll(/^\s+"([a-z0-9-]+)",$/gmu)].map((match) => match[1]);
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
