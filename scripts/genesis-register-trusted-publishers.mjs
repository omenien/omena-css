#!/usr/bin/env node
// One-off: register the GitHub Trusted Publisher on crates.io for EVERY publishable
// crate in the train, so future releases (0.3.0+) publish via OIDC and the stored
// bootstrap CRATES_IO_TOKEN can be deleted. crates.io configures Trusted Publishing
// PER-CRATE — 41 manual web-form entries — but
// `POST /api/v1/trusted_publishing/github_configs` accepts an API token, so this
// scripts it. Idempotent: skips crates already configured for this repo+workflow.
//
// Usage:
//   CRATES_IO_API_TOKEN=<token> node scripts/genesis-register-trusted-publishers.mjs
// Get the token at crates.io -> Account Settings -> API Tokens -> New Token (it needs
// account access to manage trusted publishing; an unscoped token is simplest). The
// token is read from the env only — never pass it on the command line or commit it.
/* eslint-disable no-await-in-loop */
import { execSync } from "node:child_process";

const TOKEN = process.env.CRATES_IO_API_TOKEN;
if (!TOKEN) {
  console.error(
    "error: set CRATES_IO_API_TOKEN (crates.io -> Account Settings -> API Tokens -> New Token)",
  );
  process.exit(1);
}

const REPO_OWNER = "omenien";
const REPO_NAME = "omena-css";
const WORKFLOW = "_publish-crate-train.yml";
const ENVIRONMENT = "release";
const UA = "omena-tp-register (https://github.com/omenien/omena-css)";
const BASE = "https://crates.io/api/v1/trusted_publishing/github_configs";
const headers = { "User-Agent": UA, Authorization: TOKEN, "Content-Type": "application/json" };

const meta = JSON.parse(
  execSync("cargo metadata --manifest-path rust/Cargo.toml --no-deps --format-version 1", {
    maxBuffer: 1 << 28,
  }),
);
const crates = meta.packages
  .filter((p) => p.publish === null || p.publish === undefined)
  .map((p) => p.name)
  .toSorted();

// Pre-list existing configs for idempotency.
const existing = new Set();
const listRes = await fetch(BASE, { headers });
if (listRes.ok) {
  const data = await listRes.json();
  for (const c of data.github_configs ?? []) {
    if (
      c.repository_owner === REPO_OWNER &&
      c.repository_name === REPO_NAME &&
      c.workflow_filename === WORKFLOW
    ) {
      existing.add(c.crate);
    }
  }
} else {
  console.error(
    `warn: could not list existing configs (HTTP ${listRes.status}); POST will still report duplicates`,
  );
}

let created = 0;
let skipped = 0;
let failed = 0;
for (const crate of crates) {
  if (existing.has(crate)) {
    console.log(`skip   ${crate} (already configured)`);
    skipped++;
    continue;
  }
  const body = JSON.stringify({
    github_config: {
      crate,
      repository_owner: REPO_OWNER,
      repository_name: REPO_NAME,
      workflow_filename: WORKFLOW,
      environment: ENVIRONMENT,
    },
  });
  const res = await fetch(BASE, { method: "POST", headers, body });
  if (res.ok) {
    console.log(`create ${crate}`);
    created++;
  } else {
    const text = await res.text();
    if (res.status === 409 || /already/i.test(text)) {
      console.log(`skip   ${crate} (already configured)`);
      skipped++;
    } else {
      console.error(`FAIL   ${crate}: HTTP ${res.status} ${text.slice(0, 200)}`);
      failed++;
    }
  }
}

console.log(`\ndone: created=${created} skipped=${skipped} failed=${failed} (of ${crates.length})`);
if (failed > 0) process.exit(1);
