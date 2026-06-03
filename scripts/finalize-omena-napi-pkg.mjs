#!/usr/bin/env node
// Post-step for `build:omena-napi`.
//
// `napi build ... -o rust/crates/omena-napi/pkg` emits the loader (index.js),
// types (index.d.ts), and the platform binary (index.<triple>.node) into pkg/,
// but does NOT write an npm package.json. pkg/ is a gitignored build dir, so the
// manifest cannot be committed there either. This script WRITES pkg/package.json
// (the publish manifest for @omena/napi), stamping the version from
// [workspace.package].version (axis A, single source of truth) so it can never
// drift from the crate-train version.
//
// Single-package-bundles-the-.node form: every platform's index.<triple>.node is
// collected into the one @omena/napi package (files: ["*.node", ...]); the
// napi-rs loader in index.js selects the right binary at require() time.
//
// Idempotent: safe to re-run.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const pkgDir = path.join(repoRoot, "rust", "crates", "omena-napi", "pkg");
const pkgJsonPath = path.join(pkgDir, "package.json");

if (!existsSync(pkgDir)) {
  throw new Error(`Missing ${pkgDir}; run \`napi build ... -o pkg\` (build:omena-napi) first`);
}

// Axis-A version: the single [workspace.package].version (NOT changesets).
const workspaceCargoToml = readFileSync(path.join(repoRoot, "rust", "Cargo.toml"), "utf8");
const versionMatch = workspaceCargoToml.match(
  /\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/,
);
if (!versionMatch) {
  throw new Error("Could not read [workspace.package].version from rust/Cargo.toml");
}
const version = versionMatch[1];

const REPOSITORY_URL = "https://github.com/omenien/omena-css";
const pkg = {
  name: "@omena/napi",
  version,
  description: "Node native bindings for the Omena CSS parser and transform checks",
  license: "MIT",
  repository: { type: "git", url: REPOSITORY_URL },
  main: "index.js",
  types: "index.d.ts",
  engines: { node: ">=22" },
  files: ["index.js", "index.d.ts", "*.node"],
  napi: {
    binaryName: "index",
    targets: [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc",
    ],
  },
};

writeFileSync(pkgJsonPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
console.log(`Finalized ${path.relative(repoRoot, pkgJsonPath)}: @omena/napi@${version}`);
