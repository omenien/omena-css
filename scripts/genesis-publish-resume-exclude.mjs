#!/usr/bin/env node
// Genesis RESUME helper. `cargo publish --workspace` hard-errors ("crate X@V
// already exists on crates.io index") if ANY workspace member's version is
// already published — so it cannot resume a partial genesis (crates.io publish
// is non-atomic + the new-crate rate limit aborts mid-train). This derives a
// `--exclude <name>` list for every publishable crate already live at the
// current [workspace.package].version, so the publish targets ONLY the not-yet-
// published remainder, in cargo's own deps-first topo order. Idempotent: re-run
// after each partial wave and the exclude set grows until nothing remains.
//
// Emits (when $GITHUB_OUTPUT is set): exclude_args, remaining_count, remaining.
// Always prints a human summary to stderr. stdout = the exclude_args string.
//
// Queries crates.io sequentially on purpose — one polite request per crate beats
// 41 concurrent hits on the read API.
/* eslint-disable no-await-in-loop */
import { execSync } from "node:child_process";
import fs from "node:fs";

const UA = "omena-genesis-resume (https://github.com/omenien/omena-css)";
const MANIFEST = process.env.OMENA_MANIFEST ?? "rust/Cargo.toml";

const meta = JSON.parse(
  execSync(`cargo metadata --manifest-path ${MANIFEST} --no-deps --format-version 1`, {
    maxBuffer: 1 << 28,
  }),
);
// publishable = `publish` is absent/null (publish = false serializes to []).
const publishable = meta.packages
  .filter((p) => p.publish === null || p.publish === undefined)
  .map((p) => p.name)
  .toSorted();

const toml = fs.readFileSync(MANIFEST, "utf8");
const vm = toml.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/);
if (!vm) throw new Error(`no [workspace.package].version in ${MANIFEST}`);
const version = vm[1];

const already = [];
const remaining = [];
for (const name of publishable) {
  let live = false;
  try {
    const res = await fetch(`https://crates.io/api/v1/crates/${name}`, {
      headers: { "User-Agent": UA },
    });
    if (res.ok) {
      const data = await res.json();
      live = (data.versions ?? []).some((v) => v.num === version);
    } else if (res.status !== 404) {
      throw new Error(`crates.io ${res.status} for ${name}`);
    }
  } catch (err) {
    // Fail closed: if we cannot prove a crate is already published, do NOT
    // exclude it (re-publishing a live version is a harmless cargo error; the
    // dangerous direction would be skipping an unpublished crate).
    process.stderr.write(`  warn: ${name}: ${err.message} (treating as NOT-published)\n`);
  }
  (live ? already : remaining).push(name);
}

const excludeArgs = already.map((n) => `--exclude ${n}`).join(" ");
process.stderr.write(
  `genesis-resume: version=${version} publishable=${publishable.length} ` +
    `already=${already.length} remaining=${remaining.length}\n`,
);
process.stderr.write(
  `  remaining (${remaining.length}): ${remaining.join(", ") || "(none — train complete)"}\n`,
);

if (process.env.GITHUB_OUTPUT) {
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `exclude_args=${excludeArgs}\n`);
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `remaining_count=${remaining.length}\n`);
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `remaining=${remaining.join(",")}\n`);
}
process.stdout.write(excludeArgs);
