#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

interface CargoMetadataPackage {
  readonly id: string;
  readonly name: string;
  readonly manifest_path: string;
  readonly publish?: readonly string[] | null;
}

interface CargoMetadata {
  readonly packages: readonly CargoMetadataPackage[];
  readonly workspace_members: readonly string[];
}

export interface PublishableWorkspaceCrate {
  readonly name: string;
  readonly crateRoot: string;
}

export function selectPublishableWorkspaceCrates(
  metadata: CargoMetadata,
): readonly PublishableWorkspaceCrate[] {
  const workspaceMembers = new Set(metadata.workspace_members);
  const crates = metadata.packages
    .filter((pkg) => workspaceMembers.has(pkg.id))
    .filter((pkg) => !Array.isArray(pkg.publish) || pkg.publish.length > 0)
    .map((pkg) => ({ name: pkg.name, crateRoot: path.dirname(pkg.manifest_path) }))
    .toSorted((left, right) => left.name.localeCompare(right.name));

  if (crates.length === 0) {
    throw new Error("cargo metadata contains no publishable workspace crates");
  }
  if (new Set(crates.map((crate) => crate.name)).size !== crates.length) {
    throw new Error("publishable workspace crate names must be unique");
  }
  return crates;
}

export function renderCargoPublishWorkspaceConfig(
  crates: readonly PublishableWorkspaceCrate[],
): string {
  const entries = crates.map(
    (crate) => `${JSON.stringify(crate.name)} = { path = ${JSON.stringify(crate.crateRoot)} }`,
  );
  return [
    "# Generated from cargo metadata for local resolution during workspace publish verification.",
    "[patch.crates-io]",
    ...entries,
    "",
  ].join("\n");
}

function main(): void {
  const manifestPath = readArg("--manifest-path") ?? "rust/Cargo.toml";
  const outputPath = readArg("--output-file");
  if (!outputPath) {
    throw new Error("--output-file is required");
  }

  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", manifestPath, "--no-deps", "--format-version", "1"],
      { encoding: "utf8", maxBuffer: 1 << 28 },
    ),
  ) as CargoMetadata;
  const crates = selectPublishableWorkspaceCrates(metadata);
  writeFileSync(outputPath, renderCargoPublishWorkspaceConfig(crates));
  process.stdout.write(
    `${JSON.stringify({
      product: "cargo.publish-workspace-config",
      publishableCrateCount: crates.length,
      outputPath,
    })}\n`,
  );
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
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
