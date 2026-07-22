#!/usr/bin/env node
/* eslint-disable no-await-in-loop */
import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const DEFAULT_API_BASE = "https://crates.io/api/v1/crates";
const USER_AGENT = "omena-release-registry-state (https://github.com/omenien/omena-css)";

type FetchRegistry = (input: string | URL | Request, init?: RequestInit) => Promise<Response>;

export interface CrateRegistryState {
  readonly workspaceVersion: string;
  readonly publishable: readonly string[];
  readonly registered: readonly string[];
  readonly unregistered: readonly string[];
  readonly alreadyPublished: readonly string[];
  readonly remaining: readonly string[];
  readonly latestPublishedVersions: Readonly<Record<string, string>>;
}

export interface SemverBaselinePlan {
  readonly eligible: readonly string[];
  readonly noCheckableLibraryBaseline: readonly string[];
  readonly alreadyPublished: readonly string[];
}

export interface CratePublishMode {
  readonly effectiveMode: "trusted" | "bootstrap";
  readonly authenticationRequired: boolean;
}

export async function classifyCrateRegistryState({
  crateNames,
  workspaceVersion,
  fetchRegistry = fetch,
  apiBase = DEFAULT_API_BASE,
}: {
  readonly crateNames: readonly string[];
  readonly workspaceVersion: string;
  readonly fetchRegistry?: FetchRegistry;
  readonly apiBase?: string;
}): Promise<CrateRegistryState> {
  const registered: string[] = [];
  const unregistered: string[] = [];
  const alreadyPublished: string[] = [];
  const latestPublishedVersions: Record<string, string> = {};

  for (const name of crateNames) {
    const response = await fetchRegistry(`${apiBase}/${encodeURIComponent(name)}`, {
      headers: { "User-Agent": USER_AGENT },
    });
    if (response.status === 404) {
      unregistered.push(name);
      continue;
    }
    if (!response.ok) {
      throw new Error(`crates.io returned ${response.status} for ${name}`);
    }

    const payload = (await response.json()) as {
      readonly versions?: readonly { readonly num?: string; readonly yanked?: boolean }[];
    };
    if (!Array.isArray(payload.versions)) {
      throw new Error(`crates.io response for ${name} has no versions array`);
    }
    const latestVersion = selectLatestStableVersion(payload.versions);
    if (!latestVersion) {
      throw new Error(`crates.io response for ${name} has no stable published version`);
    }
    registered.push(name);
    latestPublishedVersions[name] = latestVersion;
    if (payload.versions.some((version) => version.num === workspaceVersion)) {
      alreadyPublished.push(name);
    }
  }

  const alreadySet = new Set(alreadyPublished);
  return {
    workspaceVersion,
    publishable: [...crateNames],
    registered,
    unregistered,
    alreadyPublished,
    remaining: crateNames.filter((name) => !alreadySet.has(name)),
    latestPublishedVersions,
  };
}

export async function deriveSemverBaselinePlan({
  registryState,
  baselineHasCheckableLibraryTarget,
}: {
  readonly registryState: CrateRegistryState;
  readonly baselineHasCheckableLibraryTarget: (name: string, version: string) => Promise<boolean>;
}): Promise<SemverBaselinePlan> {
  const alreadyPublished = new Set(registryState.alreadyPublished);
  const eligible: string[] = [];
  const noCheckableLibraryBaseline: string[] = [];

  for (const name of registryState.registered) {
    if (alreadyPublished.has(name)) continue;
    const baselineVersion = registryState.latestPublishedVersions[name];
    if (!baselineVersion) {
      throw new Error(`registered crate ${name} has no baseline version`);
    }
    if (await baselineHasCheckableLibraryTarget(name, baselineVersion)) {
      eligible.push(name);
    } else {
      noCheckableLibraryBaseline.push(name);
    }
  }

  return {
    eligible,
    noCheckableLibraryBaseline,
    alreadyPublished: [...registryState.alreadyPublished],
  };
}

export function semverCheckableLibraryPath(manifestSource: string): string | undefined {
  const libSection = readTomlTable(manifestSource, "lib");
  if (/^\s*proc-macro\s*=\s*true\s*(?:#.*)?$/mu.test(libSection)) {
    return undefined;
  }

  const crateTypeSource = libSection.match(/^\s*crate-type\s*=\s*\[([\s\S]*?)\]/mu)?.[1];
  if (crateTypeSource) {
    const crateTypes = [...crateTypeSource.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
    if (!crateTypes.some((crateType) => crateType === "lib" || crateType === "rlib")) {
      return undefined;
    }
  }

  return libSection.match(/^\s*path\s*=\s*"([^"]+)"/mu)?.[1] ?? "src/lib.rs";
}

export function resolveCratePublishMode({
  requestedMode,
  dryRun,
  unregisteredCount,
}: {
  readonly requestedMode: string;
  readonly dryRun: boolean;
  readonly unregisteredCount: number;
}): CratePublishMode {
  if (!new Set(["auto", "oidc", "bootstrap"]).has(requestedMode)) {
    throw new Error(`publish mode must be auto, oidc, or bootstrap (got ${requestedMode})`);
  }

  const effectiveMode =
    requestedMode === "auto"
      ? unregisteredCount > 0
        ? "bootstrap"
        : "trusted"
      : requestedMode === "bootstrap"
        ? "bootstrap"
        : "trusted";
  const authenticationRequired = !dryRun;

  if (authenticationRequired && unregisteredCount > 0 && effectiveMode !== "bootstrap") {
    throw new Error(
      `${unregisteredCount} never-published crate name(s) require bootstrap mode; OIDC Trusted Publishing cannot create a crate`,
    );
  }

  return { effectiveMode, authenticationRequired };
}

async function main(): Promise<void> {
  const manifestPath = readArg("--manifest-path") ?? "rust/Cargo.toml";
  const orderPath = readArg("--order-file");
  const outputPath = readArg("--output-file");
  const workspaceVersion = readWorkspaceVersion(manifestPath);
  const manifests = readPublishableCrates(manifestPath);
  const metadataNames = manifests.map((manifest) => manifest.name);
  const crateNames = orderPath ? readCanonicalOrder(orderPath, metadataNames) : metadataNames;
  const registryState = await classifyCrateRegistryState({ crateNames, workspaceVersion });
  const manifestByName = new Map(manifests.map((manifest) => [manifest.name, manifest]));
  const semverPlan = await deriveSemverBaselinePlan({
    registryState,
    baselineHasCheckableLibraryTarget: async (name, version) => {
      const manifest = manifestByName.get(name);
      if (!manifest) throw new Error(`missing workspace manifest for ${name}`);
      return releaseTagHasSemverCheckableLibraryTarget(version, manifest.manifestPath);
    },
  });
  const publishMode = resolveCratePublishMode({
    requestedMode: process.env.PUBLISH_MODE ?? "oidc",
    dryRun: process.env.DRY_RUN !== "false",
    unregisteredCount: registryState.unregistered.length,
  });
  const report = {
    ...registryState,
    semverEligible: semverPlan.eligible,
    semverNoCheckableLibraryBaseline: semverPlan.noCheckableLibraryBaseline,
    semverAlreadyPublished: semverPlan.alreadyPublished,
    publishMode,
  };
  const serialized = `${JSON.stringify(report, null, 2)}\n`;

  if (outputPath) {
    writeFileSync(outputPath, serialized);
  }
  if (process.env.GITHUB_OUTPUT) {
    const excludeArgs = registryState.alreadyPublished.map((name) => `--exclude ${name}`).join(" ");
    const outputs = [
      `registered_count=${registryState.registered.length}`,
      `unregistered_count=${registryState.unregistered.length}`,
      `unregistered=${registryState.unregistered.join(",")}`,
      `already_count=${registryState.alreadyPublished.length}`,
      `remaining_count=${registryState.remaining.length}`,
      `remaining=${registryState.remaining.join(",")}`,
      `semver_eligible_count=${semverPlan.eligible.length}`,
      `semver_no_checkable_library_baseline=${semverPlan.noCheckableLibraryBaseline.join(",")}`,
      `exclude_args=${excludeArgs}`,
      `effective_mode=${publishMode.effectiveMode}`,
      `authentication_required=${publishMode.authenticationRequired}`,
    ];
    appendFileSync(process.env.GITHUB_OUTPUT, `${outputs.join("\n")}\n`);
  }

  process.stdout.write(serialized);
}

function readWorkspaceVersion(manifestPath: string): string {
  const source = readFileSync(manifestPath, "utf8");
  const match = source.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/u);
  if (!match) {
    throw new Error(`no [workspace.package].version in ${manifestPath}`);
  }
  return match[1]!;
}

function readPublishableCrates(
  manifestPath: string,
): { readonly name: string; readonly manifestPath: string }[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", manifestPath, "--no-deps", "--format-version", "1"],
      { encoding: "utf8", maxBuffer: 1 << 28 },
    ),
  ) as {
    readonly packages: readonly {
      readonly name: string;
      readonly manifest_path: string;
      readonly publish?: readonly string[] | null;
    }[];
  };
  return metadata.packages
    .filter((manifest) => !Array.isArray(manifest.publish) || manifest.publish.length > 0)
    .map((manifest) => ({ name: manifest.name, manifestPath: manifest.manifest_path }))
    .toSorted((left, right) => left.name.localeCompare(right.name));
}

function releaseTagHasSemverCheckableLibraryTarget(version: string, manifestPath: string): boolean {
  const tag = `release-v${version}`;
  const relativeManifest = path.relative(process.cwd(), manifestPath).split(path.sep).join("/");
  const manifestSource = execFileSync("git", ["show", `${tag}:${relativeManifest}`], {
    encoding: "utf8",
  });
  const declaredPath = semverCheckableLibraryPath(manifestSource);
  if (!declaredPath) return false;
  const libraryPath = path.posix.join(path.posix.dirname(relativeManifest), declaredPath);
  return (
    spawnSync("git", ["cat-file", "-e", `${tag}:${libraryPath}`], {
      stdio: "ignore",
    }).status === 0
  );
}

function readTomlTable(source: string, tableName: string): string {
  const lines = source.split(/\r?\n/u);
  const tableStart = lines.findIndex((line) => line.trim() === `[${tableName}]`);
  const tableLines = tableStart === -1 ? [] : lines.slice(tableStart + 1);
  const nextTableOffset = tableLines.findIndex((line) => /^\s*\[/u.test(line));
  return tableLines.slice(0, nextTableOffset === -1 ? undefined : nextTableOffset).join("\n");
}

function selectLatestStableVersion(
  versions: readonly { readonly num?: string; readonly yanked?: boolean }[],
): string | undefined {
  return versions
    .filter(
      (version): version is { readonly num: string; readonly yanked?: boolean } =>
        version.yanked !== true &&
        typeof version.num === "string" &&
        /^\d+\.\d+\.\d+$/u.test(version.num),
    )
    .map((version) => version.num)
    .toSorted(compareStableVersionsDescending)[0];
}

function compareStableVersionsDescending(left: string, right: string): number {
  const leftParts = left.split(".").map(Number);
  const rightParts = right.split(".").map(Number);
  for (let index = 0; index < 3; index += 1) {
    const difference = rightParts[index]! - leftParts[index]!;
    if (difference !== 0) return difference;
  }
  return 0;
}

function readCanonicalOrder(orderPath: string, metadataNames: readonly string[]): string[] {
  const orderedNames = readFileSync(orderPath, "utf8")
    .split(/\r?\n/u)
    .map((name) => name.trim())
    .filter(Boolean);
  const expected = [...metadataNames].toSorted();
  const actual = [...orderedNames].toSorted();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${orderPath} does not contain the complete publishable workspace crate set`);
  }
  return orderedNames;
}

function readArg(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${name} requires a value`);
  }
  return value;
}

const entrypoint = process.argv[1] ? pathToFileURL(path.resolve(process.argv[1])).href : "";
if (entrypoint === import.meta.url) {
  void main().catch((error: unknown) => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
