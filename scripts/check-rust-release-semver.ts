#!/usr/bin/env node
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

import { runDeclaredRustSemverCheck } from "./lib/rust-semver-intent.ts";

const repoRoot = process.cwd();
const crate = requiredArg("--crate");
const baselineVersion = requiredArg("--baseline-version");
const workspaceVersion = readWorkspaceVersion();
const result = runDeclaredRustSemverCheck({
  repoRoot,
  crate,
  workspaceVersion,
  baselineArgs: ["--baseline-version", baselineVersion],
  allFeatures: true,
});

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.release-semver",
      crate,
      baselineVersion,
      workspaceVersion,
      ...result,
    },
    null,
    2,
  )}\n`,
);

function readWorkspaceVersion(): string {
  const source = readFileSync("rust/Cargo.toml", "utf8");
  const match = source.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/u);
  assert(match, "rust/Cargo.toml must define workspace.package.version");
  return match[1]!;
}

function requiredArg(name: string): string {
  const index = process.argv.indexOf(name);
  const value = index >= 0 ? process.argv[index + 1] : undefined;
  assert(value, `${name} is required`);
  return value;
}
