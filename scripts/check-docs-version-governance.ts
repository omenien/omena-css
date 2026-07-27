import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface ChangesetConfig {
  readonly ignore: readonly string[];
}

interface LinkedEmissionContract {
  readonly conditions: {
    readonly majorVersionBoundary: {
      readonly minimumMajorVersion: number;
    };
  };
}

interface PackageManifest {
  readonly name?: string;
  readonly private?: boolean;
  readonly version: string;
}

interface TagGrammarReport {
  readonly crateTrainTagPrefix: string;
  readonly vsixTagPrefix: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const governancePath = "docs/governance/version-governance.md";
const governance = read(governancePath);
const rows = parseDerivedRows(governance);
const rootPackage = readJson<PackageManifest>("package.json");
const workspaceVersion = readWorkspaceVersion();
const changesetConfig = readJson<ChangesetConfig>(".changeset/config.json");
const linkedEmission = readJson<LinkedEmissionContract>(
  "rust/omena-linked-emission-default-precondition.json",
);
const tagGrammar = readTagGrammarReport();
const ignoredPackages = [...changesetConfig.ignore];
const separateFirstPublishPackages = readPrivatePackageNames()
  .filter((name) => !ignoredPackages.includes(name))
  .toSorted();
const npmPublishWorkflow = read(".github/workflows/_publish-npm.yml");
const releaseManagedNpmBindings = deriveNpmBindingFamilies(npmPublishWorkflow);

const expectedRows = new Map<string, string>([
  ["extensionVersion", rootPackage.version],
  ["extensionVersionLine", `${semverMajor(rootPackage.version)}.x`],
  ["crateTrainVersion", workspaceVersion],
  ["crateTrainVersionLine", `${semverMajor(workspaceVersion)}.x`],
  ["crateTrainTagPrefix", tagGrammar.crateTrainTagPrefix],
  ["extensionTagPrefix", tagGrammar.vsixTagPrefix],
  [
    "linkedEmissionReservedMajor",
    String(linkedEmission.conditions.majorVersionBoundary.minimumMajorVersion),
  ],
  ["changesetIgnoredPackages", ignoredPackages.join(", ")],
  ["separateFirstPublishPackages", separateFirstPublishPackages.join(", ")],
  ["releaseManagedNpmBindings", releaseManagedNpmBindings.join(", ")],
]);

assert.deepEqual(
  [...rows.entries()],
  [...expectedRows.entries()],
  `${governancePath} derived rows must match their authoritative sources`,
);

for (const heading of [
  "## Independent axes",
  "## Reserved majors",
  "## Pre-1.0 breaking changes",
  "## Publish status",
]) {
  assert.ok(governance.includes(heading), `${governancePath} must include ${heading}`);
}
for (const apiSnapshot of [
  "rust/crates/omena-query/tests/snapshots/public-api.txt",
  "rust/crates/omena-bundler/tests/snapshots/public-api.txt",
]) {
  assert.ok(existsSync(path.join(repoRoot, apiSnapshot)), `missing API snapshot ${apiSnapshot}`);
  assert.ok(governance.includes(apiSnapshot), `${governancePath} must cite ${apiSnapshot}`);
}
assert.ok(
  governance.includes("full-corpus differential coverage") &&
    governance.includes("zero unexpected-\ndivergence census"),
  `${governancePath} must retain every linked-emission admission condition`,
);
assert.ok(
  governance.includes("first public publish requires a separate decision"),
  `${governancePath} must keep first-publish decisions explicit`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.version-governance",
      extensionVersion: rootPackage.version,
      crateTrainVersion: workspaceVersion,
      linkedEmissionReservedMajor:
        linkedEmission.conditions.majorVersionBoundary.minimumMajorVersion,
      crateTrainTagPrefix: tagGrammar.crateTrainTagPrefix,
      extensionTagPrefix: tagGrammar.vsixTagPrefix,
      changesetIgnoredPackageCount: ignoredPackages.length,
      separateFirstPublishPackageCount: separateFirstPublishPackages.length,
      releaseManagedNpmBindingFamilyCount: releaseManagedNpmBindings.length,
      currentClaimsMatch: true,
    },
    null,
    2,
  )}\n`,
);

function parseDerivedRows(source: string): Map<string, string> {
  const derivedRows = new Map<string, string>();
  for (const match of source.matchAll(/^\|\s*`([^`]+)`\s*\|\s*`([^`]*)`\s*\|/gmu)) {
    assert.ok(!derivedRows.has(match[1]!), `duplicate governance row ${match[1]}`);
    derivedRows.set(match[1]!, match[2]!);
  }
  return derivedRows;
}

function readWorkspaceVersion(): string {
  const cargo = read("rust/Cargo.toml");
  const workspacePackage = cargo.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/u);
  assert.ok(workspacePackage, "rust/Cargo.toml must define [workspace.package].version");
  return workspacePackage[1]!;
}

function readPrivatePackageNames(): string[] {
  const packagesRoot = path.join(repoRoot, "packages");
  return readdirSync(packagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join("packages", entry.name, "package.json"))
    .filter((manifestPath) => existsSync(path.join(repoRoot, manifestPath)))
    .map((manifestPath) => readJson<PackageManifest>(manifestPath))
    .filter(
      (manifest): manifest is PackageManifest & { readonly name: string } =>
        manifest.private === true && typeof manifest.name === "string",
    )
    .map((manifest) => manifest.name);
}

function deriveNpmBindingFamilies(workflow: string): string[] {
  const families = ["@omena/napi", "@omena/napi-*", "@omena/wasm"];
  for (const family of families) {
    assert.ok(workflow.includes(family), `_publish-npm.yml must name ${family}`);
  }
  assert.match(workflow, /publish_wasm:[\s\S]*?default: true/u);
  assert.match(workflow, /publish_napi:[\s\S]*?default: true/u);
  return families;
}

function readTagGrammarReport(): TagGrammarReport {
  const result = spawnSync(
    process.execPath,
    ["--import", "tsx", "./scripts/check-release-tag-grammar.ts"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as TagGrammarReport;
}

function semverMajor(version: string): number {
  assert.match(version, /^\d+\.\d+\.\d+$/u, `expected stable semver, got ${version}`);
  return Number.parseInt(version.split(".")[0]!, 10);
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
