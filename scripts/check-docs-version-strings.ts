import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { readRustPackageMetadata } from "./rust-package-metadata";

interface LspBoundarySummary {
  readonly thinClientEndpoint: {
    readonly splitRepository: string;
    readonly cargoInstallCommand: string;
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const extensionVersion = readJson<{ readonly version: string }>("package.json").version;
const lspPackageMetadata = readRustPackageMetadata("omena-lsp-server", repoRoot);
const workspaceVersion = lspPackageMetadata.version;
const lspRepository = lspPackageMetadata.repository;

const formerExtensionVersions = new Set(
  [...read("CHANGELOG.md").matchAll(/^## \[([0-9]+\.[0-9]+\.[0-9]+)\]/gmu)]
    .map((match) => match[1]!)
    .filter((version) => version !== extensionVersion),
);
const formerWorkspaceVersions = new Set(
  readReleaseTags().filter((version) => version !== workspaceVersion),
);

const targetMarkdownFiles = [
  "README.md",
  ...walkFiles(path.join(repoRoot, "docs"), (file) => file.endsWith(".md")).map((file) =>
    path.relative(repoRoot, file),
  ),
];
const diagnostics: string[] = [];

for (const relativePath of targetMarkdownFiles) {
  lintCurrentStateProse(relativePath, read(relativePath));
}

const versionClaimFiles = [
  "rust/crates/omena-lsp-server/src/boundary.rs",
  "scripts/check-rust-omena-lsp-server-standalone-distribution.ts",
  "scripts/check-rust-omena-lsp-server-boundary.ts",
  "scripts/check-rust-omena-lsp-server-thin-client-boundary.ts",
  "docs/clients/neovim.md",
  "docs/clients/zed.md",
];
for (const relativePath of versionClaimFiles) {
  lintLspVersionClaims(relativePath, read(relativePath));
}

const boundary = readLspBoundary();
const expectedInstallCommand = `cargo install omena-lsp-server --version ${workspaceVersion}`;
if (boundary.thinClientEndpoint.cargoInstallCommand !== expectedInstallCommand) {
  diagnostics.push(
    `Rust LSP boundary emits "${boundary.thinClientEndpoint.cargoInstallCommand}"; expected "${expectedInstallCommand}" from the workspace version`,
  );
}
if (boundary.thinClientEndpoint.splitRepository !== lspRepository) {
  diagnostics.push(
    `Rust LSP boundary repository is ${boundary.thinClientEndpoint.splitRepository}; expected ${lspRepository} from the crate manifest`,
  );
}
for (const clientDoc of ["docs/clients/neovim.md", "docs/clients/zed.md"]) {
  const source = read(clientDoc);
  if (!source.includes(lspRepository)) {
    diagnostics.push(`${clientDoc} must link to the crate repository ${lspRepository}`);
  }
}

assert.deepEqual(diagnostics, [], diagnostics.join("\n"));

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.version-strings",
      extensionVersion,
      workspaceVersion,
      formerExtensionVersions: formerExtensionVersions.size,
      formerWorkspaceVersions: formerWorkspaceVersions.size,
      markdownFiles: targetMarkdownFiles.length,
      lspVersionClaimFiles: versionClaimFiles.length,
    },
    null,
    2,
  )}\n`,
);

function lintCurrentStateProse(relativePath: string, source: string): void {
  let inFence = false;
  for (const [index, line] of source.split("\n").entries()) {
    if (line.trimStart().startsWith("```")) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    for (const match of line.matchAll(/\b([0-9]+\.[0-9]+\.[0-9]+)\b/gu)) {
      const version = match[1]!;
      const extensionClaim =
        relativePath === "README.md" ||
        relativePath === "docs/vscode-extension.md" ||
        /\b(?:extension|VS Code|release|current|stable core|version)\b/iu.test(line);
      const workspaceClaim =
        /\b(?:crate|Cargo|crates\.io|omena-lsp-server|workspace version)\b/iu.test(line);
      if (extensionClaim && formerExtensionVersions.has(version)) {
        diagnostics.push(
          `${relativePath}:${index + 1} uses former extension version ${version} in a current-state claim`,
        );
      }
      if (workspaceClaim && formerWorkspaceVersions.has(version)) {
        diagnostics.push(
          `${relativePath}:${index + 1} uses former Rust workspace version ${version} in a current-state claim`,
        );
      }
    }

    if (/current\s+`[0-9]+\.[0-9]+`.*(?:lock point|milestone|phase)/iu.test(line)) {
      diagnostics.push(
        `${relativePath}:${index + 1} exposes an internal numeric milestone as a current product label`,
      );
    }
  }
}

function lintLspVersionClaims(relativePath: string, source: string): void {
  for (const [index, line] of source.split("\n").entries()) {
    const claim =
      line.match(/cargo install omena-lsp-server --version ([0-9]+\.[0-9]+\.[0-9]+)/u) ??
      line.match(/expectedVersion\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"/u);
    if (claim && claim[1] !== workspaceVersion) {
      diagnostics.push(
        `${relativePath}:${index + 1} claims omena-lsp-server ${claim[1]}; expected ${workspaceVersion}`,
      );
    }
  }
}

function readReleaseTags(): readonly string[] {
  const result = spawnSync("git", ["tag", "--list", "release-v*"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout
    .split("\n")
    .map((tag) => tag.trim().match(/^release-v([0-9]+\.[0-9]+\.[0-9]+)$/u)?.[1])
    .filter((version): version is string => Boolean(version));
}

function readLspBoundary(): LspBoundarySummary {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-lsp-server",
      "--bin",
      "omena-lsp-server-boundary",
      "--quiet",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as LspBoundarySummary;
}

function walkFiles(directory: string, predicate: (file: string) => boolean): string[] {
  if (!existsSync(directory)) return [];
  const files: string[] = [];
  for (const entry of readdirSync(directory)) {
    const absolutePath = path.join(directory, entry);
    if (statSync(absolutePath).isDirectory()) files.push(...walkFiles(absolutePath, predicate));
    else if (predicate(absolutePath)) files.push(absolutePath);
  }
  return files.toSorted();
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
