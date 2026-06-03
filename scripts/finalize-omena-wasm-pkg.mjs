#!/usr/bin/env node
// Post-step for `build:omena-wasm`.
//
// `wasm-pack build rust/crates/omena-wasm --scope omena ...` derives the npm
// package name from the crate name, yielding `@omena/omena-wasm`. The decided
// published name is `@omena/wasm`, so rewrite `name` here. Also normalize the
// pre-rebrand `repository` URL (still `yongsk0066/css-module-explainer` in the
// committed pkg) to `omenien/omena-css` so it cannot be baked into the npm
// provenance / Sigstore subject at publish time (12 §8 / 14 §4 guard).
//
// Idempotent: safe to re-run; it only writes when a field actually changes.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const pkgJsonPath = path.join(repoRoot, "rust", "crates", "omena-wasm", "pkg", "package.json");

const TARGET_NAME = "@omena/wasm";
const REPOSITORY_URL = "https://github.com/omenien/omena-css";

if (!existsSync(pkgJsonPath)) {
  throw new Error(`Missing ${pkgJsonPath}; run \`pnpm build:omena-wasm\` (wasm-pack) first`);
}

const pkg = JSON.parse(readFileSync(pkgJsonPath, "utf8"));
const changes = [];

if (pkg.name !== TARGET_NAME) {
  changes.push(`name: ${pkg.name ?? "<unset>"} -> ${TARGET_NAME}`);
  pkg.name = TARGET_NAME;
}

const desiredRepository = { type: "git", url: REPOSITORY_URL };
const currentUrl = typeof pkg.repository === "string" ? pkg.repository : pkg.repository?.url;
if (currentUrl !== REPOSITORY_URL) {
  changes.push(`repository.url: ${currentUrl ?? "<unset>"} -> ${REPOSITORY_URL}`);
  pkg.repository = desiredRepository;
}

// Hard guard: never publish under the wrong name or a pre-rebrand URL.
if (pkg.name !== TARGET_NAME) {
  throw new Error(`Refusing to finalize: name is ${pkg.name}, expected ${TARGET_NAME}`);
}
const finalUrl = typeof pkg.repository === "string" ? pkg.repository : pkg.repository?.url;
if (finalUrl !== REPOSITORY_URL) {
  throw new Error(
    `Refusing to finalize: repository.url is ${finalUrl}, expected ${REPOSITORY_URL}`,
  );
}

if (changes.length === 0) {
  console.log(`omena-wasm pkg already finalized as ${TARGET_NAME} (no changes)`);
} else {
  writeFileSync(pkgJsonPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
  console.log(`Finalized ${path.relative(repoRoot, pkgJsonPath)}:\n  ${changes.join("\n  ")}`);
}
