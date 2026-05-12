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
];

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
  node scripts/prepare-omena-css-workspace.mjs [--dest <path>] [--force] [--verify] [--init-git]
  node scripts/prepare-omena-css-workspace.mjs --temp --verify

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
      rmSync(destinationPath, { force: true, recursive: true });
    }
  }

  mkdirSync(path.join(destinationPath, "crates"), { recursive: true });
  copyRootFiles(destinationPath);
  writeRootCargoToml(destinationPath);
  writeRootDocs(destinationPath);
  writeCiWorkflow(destinationPath);

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

  process.stdout.write(
    JSON.stringify(
      {
        destination: destinationPath,
        crateCount: omenaCssCrates.length,
        crates: omenaCssCrates,
        initializedGit: workspaceOptions.initGit,
        verified: workspaceOptions.verify,
      },
      null,
      2,
    ),
  );
  process.stdout.write("\n");
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
\`\`\`
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
