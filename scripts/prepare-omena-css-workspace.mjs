#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const omenaCssCrates = [
  "omena-interner",
  "omena-syntax",
  "omena-parser",
  "omena-incremental",
  "omena-cascade",
  "omena-transform-cst",
  "omena-transform-passes",
  "omena-transform-bundle",
  "omena-transform-target",
  "omena-transform-print",
  "omena-transform-egg",
  "omena-cli",
  "omena-napi",
  "omena-wasm",
];
const omenaCssPublishOrder = [
  "omena-syntax",
  "omena-interner",
  "omena-parser",
  "omena-incremental",
  "omena-cascade",
  "omena-transform-cst",
  "omena-transform-passes",
  "omena-transform-bundle",
  "omena-transform-target",
  "omena-transform-print",
  "omena-transform-egg",
  "omena-cli",
  "omena-napi",
  "omena-wasm",
];
const externallyPublishedCrates = new Set(["omena-incremental"]);
const omenaCssDependencyVersion = "0.1";

const cliOptions = parseArgs(process.argv.slice(2));
const destination = cliOptions.temp
  ? mkTempWorkspace()
  : path.resolve(cliOptions.dest ?? path.join(repoRoot, "..", "omena-css"));

prepareWorkspace(destination, cliOptions);

function parseArgs(args) {
  const parsedOptions = {
    dest: undefined,
    force: false,
    initGit: false,
    preserveGit: false,
    publishDryRun: false,
    temp: false,
    verify: false,
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--dest") {
      const value = args[index + 1];
      if (value === undefined) {
        throw new Error("--dest requires a path");
      }
      parsedOptions.dest = value;
      index += 1;
      continue;
    }
    if (arg === "--force") {
      parsedOptions.force = true;
      continue;
    }
    if (arg === "--init-git") {
      parsedOptions.initGit = true;
      continue;
    }
    if (arg === "--preserve-git") {
      parsedOptions.preserveGit = true;
      continue;
    }
    if (arg === "--publish-dry-run") {
      parsedOptions.publishDryRun = true;
      continue;
    }
    if (arg === "--temp") {
      parsedOptions.temp = true;
      parsedOptions.force = true;
      continue;
    }
    if (arg === "--verify") {
      parsedOptions.verify = true;
      continue;
    }
    if (arg === "-h" || arg === "--help") {
      printHelp();
      process.exit(0);
    }
    throw new Error(`Unknown argument: ${arg}`);
  }

  return parsedOptions;
}

function printHelp() {
  process.stdout.write(`Usage:
  node scripts/prepare-omena-css-workspace.mjs [--dest <path>] [--force] [--verify] [--publish-dry-run] [--preserve-git] [--init-git]
  node scripts/prepare-omena-css-workspace.mjs --temp --verify --publish-dry-run

Creates a standalone omena-css workspace containing the publish-target crates.
Default destination: ../omena-css
`);
}

function mkTempWorkspace() {
  return path.join(tmpdir(), `omena-css-workspace-${process.pid}`);
}

function prepareWorkspace(destinationPath, workspaceOptions) {
  if (existsSync(destinationPath)) {
    const entries = readdirSync(destinationPath);
    if (!workspaceOptions.force && entries.length > 0) {
      throw new Error(`Destination is not empty: ${destinationPath}. Pass --force to replace it.`);
    }
    if (workspaceOptions.force) {
      clearDestination(destinationPath, workspaceOptions);
    }
  }

  mkdirSync(path.join(destinationPath, "crates"), { recursive: true });
  copyRootFiles(destinationPath);
  writeRootCargoToml(destinationPath);
  writeRootDocs(destinationPath);
  writeCiWorkflow(destinationPath);
  writePublishWorkflow(destinationPath);

  for (const crateName of omenaCssCrates) {
    const source = path.join(repoRoot, "rust", "crates", crateName);
    const target = path.join(destinationPath, "crates", crateName);
    cpSync(source, target, {
      filter: (entryPath) => !entryPath.split(path.sep).includes("target"),
      recursive: true,
    });
    rewriteCrateManifest(path.join(target, "Cargo.toml"));
  }

  if (workspaceOptions.initGit) {
    initGitRepository(destinationPath);
  }

  if (workspaceOptions.verify) {
    verifyWorkspace(destinationPath);
  }
  if (workspaceOptions.publishDryRun) {
    verifyPublishDryRun(destinationPath);
  }

  process.stdout.write(
    JSON.stringify(
      {
        destination: destinationPath,
        crateCount: omenaCssCrates.length,
        crates: omenaCssCrates,
        initializedGit: workspaceOptions.initGit,
        preservedGit: workspaceOptions.preserveGit,
        publishDryRun: workspaceOptions.publishDryRun,
        verified: workspaceOptions.verify,
      },
      null,
      2,
    ),
  );
  process.stdout.write("\n");
}

function clearDestination(destinationPath, workspaceOptions) {
  if (!workspaceOptions.preserveGit || !existsSync(path.join(destinationPath, ".git"))) {
    rmSync(destinationPath, { force: true, recursive: true });
    return;
  }

  for (const entry of readdirSync(destinationPath)) {
    if (entry === ".git") {
      continue;
    }
    rmSync(path.join(destinationPath, entry), { force: true, recursive: true });
  }
}

function copyRootFiles(destinationPath) {
  cpSync(path.join(repoRoot, "LICENSE"), path.join(destinationPath, "LICENSE"));
  writeFileSync(
    path.join(destinationPath, "rust-toolchain.toml"),
    '[toolchain]\nchannel = "stable"\n',
  );
  cpSync(path.join(repoRoot, "rust", "rustfmt.toml"), path.join(destinationPath, "rustfmt.toml"));
  writeFileSync(
    path.join(destinationPath, ".gitignore"),
    ["/target", "Cargo.lock", ".DS_Store", ""].join("\n"),
  );
}

function writeRootCargoToml(destinationPath) {
  const members = omenaCssCrates.map((crateName) => `  "crates/${crateName}",`).join("\n");
  writeFileSync(
    path.join(destinationPath, "Cargo.toml"),
    `[workspace]
members = [
${members}
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = true

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
salsa = { version = "0.26.1", default-features = false, features = ["macros", "inventory"] }
criterion = "0.8.2"
cstree = "0.14.0"
rustc-hash = "2.1.2"
smol_str = "0.3.6"
clap = { version = "4.6.1", features = ["derive"] }
napi = "3.8.6"
napi-derive = "3.5.5"
serde-wasm-bindgen = "0.6.5"
wasm-bindgen = "0.2.121"

[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
dbg_macro = "warn"
todo = "warn"
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
`,
  );
}

function writeRootDocs(destinationPath) {
  writeFileSync(
    path.join(destinationPath, "README.md"),
    `# omena-css

Standalone Rust workspace for the Omena CSS parser, semantic substrates, cascade
model, and transform-planning crates.

This repository is staged from the CSS Module Explainer monorepo. The workspace
keeps the publish-target crates together so parser, incremental, cascade, and
transform boundaries can be verified as one product surface.

## Crates

${omenaCssCrates.map((crateName) => `- \`${crateName}\``).join("\n")}

## Verification

\`\`\`sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo publish --dry-run --manifest-path crates/omena-syntax/Cargo.toml
\`\`\`

## Publishing

Publishing is manual through the \`Publish Crates\` GitHub Actions workflow.
Run the workflow in \`dry-run\` mode first, then run \`publish\` only after CI is
green and the crates.io order has been checked. The workflow intentionally skips
\`omena-incremental\` because it publishes from its own Omena repository.

## Documentation

- [Overview](docs/overview.md)
- [Quickstart](docs/quickstart.md)
- [API reference](docs/api-reference.md)
- [Benchmarks](docs/benchmarks.md)
- [Release process](docs/release.md)
- [Paper draft outline](docs/paper-draft.md)
`,
  );
  writeFileSync(
    path.join(destinationPath, "CONTRIBUTING.md"),
    `# Contributing

Run formatting, tests, and clippy before opening a pull request:

\`\`\`sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
\`\`\`

## Commit Messages

Use plain imperative commit subjects:

\`\`\`text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
\`\`\`

Do not use internal planning labels, phase names, or issue-triage shorthand in
commit messages. Public history should describe the product change directly.
`,
  );
  writeFileSync(
    path.join(destinationPath, "CODE_OF_CONDUCT.md"),
    `# Code of Conduct

Be respectful, precise, and constructive. Keep discussion focused on the code,
the evidence, and the product goals of the omena-css workspace.
`,
  );
  writePublicDocs(destinationPath);
  writeGithubTemplates(destinationPath);
}

function writePublicDocs(destinationPath) {
  const docsDirectory = path.join(destinationPath, "docs");
  mkdirSync(docsDirectory, { recursive: true });
  writeFileSync(
    path.join(docsDirectory, "overview.md"),
    `# Overview

omena-css is a Rust workspace for CSS-family parsing, semantic substrates,
cascade modeling, incremental recomputation, and conservative CSS transforms.

The workspace is split into small crates so parser, cascade, incremental, and
transform responsibilities can be tested and published independently while still
sharing one release train.

## Crate Layers

- Syntax and interning: \`omena-syntax\`, \`omena-interner\`
- Parser surface: \`omena-parser\`
- Incremental substrate: \`omena-incremental\`
- Cascade substrate: \`omena-cascade\`
- Transform substrate: \`omena-transform-cst\`, \`omena-transform-passes\`,
  \`omena-transform-bundle\`, \`omena-transform-target\`,
  \`omena-transform-print\`, \`omena-transform-egg\`
- Consumer surfaces: \`omena-cli\`, \`omena-napi\`, \`omena-wasm\`

## Current Product Surface

The first public surface focuses on parser and transform foundations:

- CSS, SCSS, Sass, and Less dialect classification.
- Recovery-aware parser summaries for CSS Modules and style facts.
- Cascade ordering, specificity, custom-property substitution, and transform
  proof helpers.
- Conservative transform planning and execution surfaces with explicit
  provenance.
- Node native JSON binding substrate through \`omena-napi\`.
- Browser-side in-memory parser and transform bindings through \`omena-wasm\`.

## Design Rules

- Keep parser facts canonical at the parser boundary.
- Keep cascade-sensitive rewrites behind proof helpers.
- Keep source-map provenance attached to every emitted transform result.
- Prefer public crate names and product terms over private planning labels.
`,
  );
  writeFileSync(
    path.join(docsDirectory, "quickstart.md"),
    `# Quickstart

## Verify the Workspace

\`\`\`sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
\`\`\`

## Use a Crate

Add the crate that matches the layer you need:

\`\`\`sh
cargo add omena-parser
cargo add omena-cascade
cargo add omena-transform-passes
\`\`\`

Most consumers should start with \`omena-parser\` for source facts or
\`omena-transform-passes\` for transform planning. Lower-level crates remain
public so integrations can opt into smaller boundaries when needed.

## Install the CLI

\`\`\`sh
cargo install omena-cli
omena check path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena passes
\`\`\`

Use the checkout form when developing the workspace locally:

\`\`\`sh
cargo run -p omena-cli -- check path/to/file.module.scss
cargo run -p omena-cli -- build path/to/file.css --pass whitespace-strip
cargo run -p omena-cli -- passes
\`\`\`

## Use the Browser Binding

\`omena-wasm\` is an in-memory binding for browser and playground consumers.
It does not read from the filesystem; pass source text and a path-like label so
the dialect can be inferred. Generate a web package with \`wasm-pack build
crates/omena-wasm --target web\`, then import the generated module:

\`\`\`js
import init, { checkStyleSource, buildStyleSource } from "./pkg/omena_wasm.js";

await init();
const facts = checkStyleSource(".card { color: red; }", "demo.module.css");
const built = buildStyleSource(".card { color: #ffffff; }", "demo.css", [
  "color-compression",
]);
\`\`\`

## Use the Node Native Binding Substrate

\`omena-napi\` is the Rust N-API substrate for future npm packaging. It exposes
JSON-string APIs so Node clients can consume the same parser and transform
contracts without depending on unstable Rust structs. A future npm wrapper can
export this shape:

\`\`\`js
import {
  checkStyleSourceJson,
  buildStyleSourceJson,
} from "omena-napi";

const facts = JSON.parse(
  checkStyleSourceJson(".card { color: red; }", "demo.module.css"),
);
const built = JSON.parse(
  buildStyleSourceJson(".card { color: #ffffff; }", "demo.css", [
    "color-compression",
  ]),
);
\`\`\`

## Publish Readiness

Run the manual GitHub Actions publish workflow in \`dry-run\` mode first. For a
local check, package the crate you changed:

\`\`\`sh
cargo package --list --manifest-path crates/omena-parser/Cargo.toml
cargo publish --dry-run --manifest-path crates/omena-parser/Cargo.toml
\`\`\`
`,
  );
  writeFileSync(
    path.join(docsDirectory, "api-reference.md"),
    `# API Reference

This page summarizes the stable public boundaries exposed by the initial
workspace. Use crate rustdoc for full type-level documentation.

## Parser

\`omena-parser\` exposes parse and lex results, dialect classification, parser
summaries, CSS Modules intermediate summaries, and canonical producer signals.

Primary consumers:

- Editors and language servers that need style facts.
- Transform engines that need parser-owned source summaries.
- Differential tests that compare token and CST behavior.

## Cascade

\`omena-cascade\` exposes cascade keys, specificity, declaration winners,
selector-context witnesses, custom-property substitution, and proof helpers for
scope, layer, supports, and box-shorthand rewrites.

Primary consumers:

- Semantic analyzers that need cascade-aware ranking.
- Transform passes that need proof-carrying safety checks.
- Test harnesses that need deterministic cascade witnesses.

## Transform

\`omena-transform-cst\` defines transform contracts and DAG metadata.
\`omena-transform-passes\` registers and plans safe mutations.
\`omena-transform-bundle\`, \`omena-transform-target\`,
\`omena-transform-print\`, and \`omena-transform-egg\` split bundle planning,
target lowering, emission, and equality-saturation concerns.

Primary consumers:

- CSS build tools.
- Editor quick-fix pipelines.
- Benchmark and conformance runners.

## CLI

\`omena-cli\` exposes the first command-line consumer surface:

- \`omena check <file>\` reports parser-owned facts and parse-error counts.
- \`omena build <file>\` runs the conservative transform pipeline.
- \`omena passes\` lists accepted transform pass ids.

## Wasm

\`omena-wasm\` exposes the first browser-side in-memory consumer surface:

- \`checkStyleSource(source, path)\` reports parser-owned facts.
- \`buildStyleSource(source, path, passIds)\` runs conservative transform passes.
- \`listTransformPasses()\` lists accepted transform pass ids.

## Node Native Binding

\`omena-napi\` exposes the first Node native binding substrate:

- \`checkStyleSourceJson(source, path)\` reports parser-owned facts as JSON.
- \`buildStyleSourceJson(source, path, passIds)\` runs conservative transform
  passes and returns JSON.
- \`listTransformPassesJson()\` lists accepted transform pass ids as JSON.
`,
  );
  writeFileSync(
    path.join(docsDirectory, "benchmarks.md"),
    `# Benchmarks

The public benchmark story is intentionally evidence-based. Benchmark changes
must report the command, input set, machine class, and comparison baseline.

## Current Baseline Checks

- Parser product-cutover checks compare parser output against the current
  product lane.
- Runtime loop checks track request-path latency for hover, definition,
  references, and completion.
- Fuzz checks cover parser, cascade, incremental, and transform safety targets.

## Reporting Template

\`\`\`text
Command:
Inputs:
Machine:
Baseline:
Result:
Regression threshold:
Notes:
\`\`\`

Do not treat a single synthetic benchmark as product readiness. Parser,
cascade, transform, editor, and packaging paths each need their own evidence.
`,
  );
  writeFileSync(
    path.join(docsDirectory, "release.md"),
    `# Release Process

omena-css uses one workspace release train for the public crates in this repo.
Patch releases may be crate-specific when only one crate needs a compatibility
or packaging fix.

## Required Checks

\`\`\`sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
\`\`\`

Run the \`Publish Crates\` GitHub Actions workflow in \`dry-run\` mode before
publishing. Publish only after CI is green and dependency order has been
checked.

## Commit Messages

Use plain imperative commit subjects:

\`\`\`text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
\`\`\`

Avoid private planning labels in public history, docs, rustdoc, and release
notes.
`,
  );
  writeFileSync(
    path.join(docsDirectory, "paper-draft.md"),
    `# Paper Draft Outline

This is the initial public outline for the research track behind omena-css.
It is not a submitted paper; it records the external-facing argument and the
evidence that must exist before publication.

## Candidate 1: Cascade-Proven CSS Transforms

- Problem: many CSS transforms are syntactically simple but semantically unsafe
  without cascade, layer, scope, or selector evidence.
- Contribution: proof-carrying transform helpers that reject unsafe rewrites
  unless the caller provides closed-world evidence.
- Evaluation: compare accepted and rejected transform candidates across real
  CSS Modules, SCSS, and Less projects.

## Candidate 2: Incremental CSS-Family Analysis

- Problem: editor latency depends on reusing parser, cascade, and transform
  facts across small edits.
- Contribution: incremental fact boundaries for style analysis and conservative
  transform planning.
- Evaluation: measure cold and warm editor request latency across project-size
  buckets.

## Candidate 3: Parser-Owned Style Facts

- Problem: editor integrations often duplicate style parsing in ad hoc request
  handlers.
- Contribution: parser-owned canonical fact production for CSS Modules and
  CSS-family dialects.
- Evaluation: compare diagnostics, hover, definition, references, and transform
  results before and after request handlers consume parser-owned facts.
`,
  );
}

function writeGithubTemplates(destinationPath) {
  const githubDirectory = path.join(destinationPath, ".github");
  const issueTemplateDirectory = path.join(githubDirectory, "ISSUE_TEMPLATE");
  mkdirSync(issueTemplateDirectory, { recursive: true });
  writeFileSync(
    path.join(githubDirectory, "PULL_REQUEST_TEMPLATE.md"),
    `## Summary

## Verification

- [ ] \`cargo fmt --all --check\`
- [ ] \`cargo test --workspace\`
- [ ] \`cargo clippy --workspace --all-targets --all-features -- -D warnings\`

## Notes

Use a plain imperative commit subject and avoid private planning labels in
public docs, rustdoc, release notes, and commit history.
`,
  );
  writeFileSync(
    path.join(issueTemplateDirectory, "bug_report.yml"),
    `name: Bug report
description: Report incorrect parser, cascade, transform, or packaging behavior.
title: "Describe the failing behavior"
labels: ["bug"]
body:
  - type: textarea
    id: observed
    attributes:
      label: Observed behavior
      description: What happened?
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected behavior
      description: What should have happened?
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: Reproduction
      description: Include CSS, SCSS, Less, Rust code, or commands needed to reproduce.
    validations:
      required: true
  - type: textarea
    id: verification
    attributes:
      label: Verification
      description: Commands or checks you ran.
`,
  );
  writeFileSync(
    path.join(issueTemplateDirectory, "feature_request.yml"),
    `name: Feature request
description: Propose a parser, cascade, transform, tooling, or documentation addition.
title: "Describe the product capability"
labels: ["enhancement"]
body:
  - type: textarea
    id: problem
    attributes:
      label: Problem
      description: What user-facing or integration problem does this solve?
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposal
      description: Describe the API, behavior, or documentation change.
    validations:
      required: true
  - type: textarea
    id: evidence
    attributes:
      label: Evidence
      description: Link examples, specs, benchmarks, or downstream consumers.
`,
  );
}

function writeCiWorkflow(destinationPath) {
  const workflowDirectory = path.join(destinationPath, ".github", "workflows");
  mkdirSync(workflowDirectory, { recursive: true });
  writeFileSync(
    path.join(workflowDirectory, "ci.yml"),
    `name: CI

on:
  push:
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo test --workspace
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings
`,
  );
}

function writePublishWorkflow(destinationPath) {
  const workflowDirectory = path.join(destinationPath, ".github", "workflows");
  mkdirSync(workflowDirectory, { recursive: true });
  const publishCrateRows = omenaCssPublishOrder
    .map((crateName) => `            ${crateName}`)
    .join("\n");
  writeFileSync(
    path.join(workflowDirectory, "publish.yml"),
    `name: Publish Crates

on:
  workflow_dispatch:
    inputs:
      mode:
        description: "Run publish readiness checks or publish crates"
        required: true
        default: "dry-run"
        type: choice
        options:
          - dry-run
          - publish

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    env:
      CARGO_REGISTRY_TOKEN: \${{ secrets.CRATES_IO_TOKEN }}
      PUBLISH_MODE: \${{ inputs.mode }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo test --workspace
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings
      - name: Publish crates
        shell: bash
        run: |
          set -euo pipefail

          crates=(
${publishCrateRows}
          )

          has_local_omena_dependencies() {
            grep -Eq '^omena-[a-z0-9-]+ = \\{ path = "\\.\\./omena-[a-z0-9-]+".* \\}$' "$1"
          }

          crate_exists() {
            cargo info "$1" --registry crates-io >/dev/null 2>&1
          }

          publish_with_retry() {
            local crate="$1"
            local manifest="crates/$crate/Cargo.toml"
            local publish_log

            if crate_exists "$crate"; then
              echo "$crate already exists on crates.io; skipping"
              return 2
            fi

            for attempt in 1 2 3 4 5 6; do
              publish_log="$(mktemp)"
              if cargo publish --manifest-path "$manifest" 2>&1 | tee "$publish_log"; then
                rm -f "$publish_log"
                return
              fi
              if crate_exists "$crate"; then
                echo "$crate became available after a publish retry; continuing"
                rm -f "$publish_log"
                return
              fi
              if grep -q "Too Many Requests" "$publish_log"; then
                echo "publish rate-limited for $crate on attempt $attempt; waiting for crates.io new-crate window"
                rm -f "$publish_log"
                sleep 630
                continue
              fi
              rm -f "$publish_log"
              echo "publish failed for $crate on attempt $attempt; waiting for registry propagation"
              sleep $((attempt * 30))
            done

            cargo publish --manifest-path "$manifest"
          }

          for crate in "\${crates[@]}"; do
            manifest="crates/$crate/Cargo.toml"

            if [[ "$crate" == "omena-incremental" ]]; then
              echo "$crate publishes from its own Omena repository; skipping"
              continue
            fi

            if [[ "$PUBLISH_MODE" == "dry-run" ]]; then
              cargo package --list --manifest-path "$manifest" >/dev/null
              if has_local_omena_dependencies "$manifest"; then
                echo "$crate package surface checked; full dry-run waits for upstream Omena crates on crates.io"
              else
                cargo publish --dry-run --manifest-path "$manifest"
              fi
              continue
            fi

            if [[ "$PUBLISH_MODE" == "publish" ]]; then
              if publish_with_retry "$crate"; then
                sleep 30
              else
                publish_status="$?"
                if [[ "$publish_status" == "2" ]]; then
                  continue
                fi
                exit "$publish_status"
              fi
              continue
            fi

            echo "unknown publish mode: $PUBLISH_MODE" >&2
            exit 1
          done
`,
  );
}

function rewriteCrateManifest(manifestPath) {
  let manifest = readFileSync(manifestPath, "utf8");
  manifest = manifest.replaceAll(
    'repository = "https://github.com/yongsk0066/css-module-explainer"',
    'repository = "https://github.com/omenien/omena-css"',
  );
  if (!/^keywords = \[/m.test(manifest)) {
    manifest = manifest.replace(
      /^readme = "README\.md"$/m,
      'readme = "README.md"\nkeywords = ["omena", "css", "parser", "analysis"]\ncategories = ["development-tools", "parser-implementations"]',
    );
  }
  manifest = manifest.replace(
    /^(omena-[a-z0-9-]+ = \{ path = "\.\.\/omena-[a-z0-9-]+") \}$/gm,
    `$1, version = "${omenaCssDependencyVersion}" }`,
  );
  writeFileSync(manifestPath, manifest);
}

function initGitRepository(destinationPath) {
  execFileSync("git", ["init"], { cwd: destinationPath, stdio: "inherit" });
  execFileSync("git", ["add", "."], { cwd: destinationPath, stdio: "inherit" });
  execFileSync("git", ["commit", "-m", "Initial omena-css workspace"], {
    cwd: destinationPath,
    stdio: "inherit",
  });
}

function verifyWorkspace(destinationPath) {
  execFileSync("cargo", ["fmt", "--all", "--check"], {
    cwd: destinationPath,
    env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" },
    stdio: "inherit",
  });
  execFileSync("cargo", ["test", "--workspace"], {
    cwd: destinationPath,
    env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" },
    stdio: "inherit",
  });
  execFileSync(
    "cargo",
    ["clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
    {
      cwd: destinationPath,
      env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" },
      stdio: "inherit",
    },
  );
}

function verifyPublishDryRun(destinationPath) {
  for (const crateName of omenaCssPublishOrder) {
    const manifestPath = path.join(destinationPath, "crates", crateName, "Cargo.toml");
    assertVersionedLocalDependencies(manifestPath);
    execFileSync("cargo", ["package", "--list", "--manifest-path", manifestPath], {
      cwd: destinationPath,
      env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" },
      stdio: "ignore",
    });

    if (externallyPublishedCrates.has(crateName)) {
      process.stderr.write(
        `validated package surface for ${crateName}; crate already publishes from its own omena repository\n`,
      );
      continue;
    }

    if (hasLocalOmenaDependencies(manifestPath)) {
      process.stderr.write(
        `validated package surface for ${crateName}; full publish dry-run waits for upstream omena crates on crates.io\n`,
      );
      continue;
    }

    execFileSync("cargo", ["publish", "--dry-run", "--manifest-path", manifestPath], {
      cwd: destinationPath,
      env: { ...process.env, RUSTUP_TOOLCHAIN: "stable" },
      stdio: "inherit",
    });
  }
}

function hasLocalOmenaDependencies(manifestPath) {
  return /^omena-[a-z0-9-]+ = \{ path = "\.\.\/omena-[a-z0-9-]+"/m.test(
    readFileSync(manifestPath, "utf8"),
  );
}

function assertVersionedLocalDependencies(manifestPath) {
  const manifest = readFileSync(manifestPath, "utf8");
  const localDependencies =
    manifest.match(/^omena-[a-z0-9-]+ = \{ path = "\.\.\/omena-[a-z0-9-]+".* \}$/gm) ?? [];

  for (const dependency of localDependencies) {
    if (!/, version = "[^"]+"/.test(dependency)) {
      throw new Error(
        `Local omena dependency must include a publish version in ${manifestPath}: ${dependency}`,
      );
    }
  }
}
