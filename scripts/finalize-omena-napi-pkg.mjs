#!/usr/bin/env node
// Post-step for `build:omena-napi`.
//
// `napi build ... -o rust/crates/omena-napi/pkg` emits the loader (index.js),
// types (index.d.ts), and the current platform binary into pkg/. This script
// stamps the publish manifests from the crate-train version and prepares the
// standard optional native-package layout:
//
//   @omena/napi                  -> JS loader + types, optionalDependencies
//   @omena/napi-linux-x64-gnu    -> index.linux-x64-gnu.node
//   @omena/napi-linux-arm64-gnu  -> index.linux-arm64-gnu.node
//   @omena/napi-darwin-x64       -> index.darwin-x64.node
//   @omena/napi-darwin-arm64     -> index.darwin-arm64.node
//   @omena/napi-win32-x64-msvc   -> index.win32-x64-msvc.node
//
// Idempotent: safe to re-run after each matrix artifact download.
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const pkgDir = path.join(repoRoot, "rust", "crates", "omena-napi", "pkg");
const pkgJsonPath = path.join(pkgDir, "package.json");
const loaderPath = path.join(pkgDir, "index.js");
const platformRoot = path.join(pkgDir, "npm");

const MAIN_PACKAGE_NAME = "@omena/napi";
const REPOSITORY_URL = "https://github.com/omenien/omena-css";
const DESCRIPTION = "Node native bindings for the Omena CSS parser and transform checks";
const NODE_ENGINE = ">=22";

const PLATFORM_PACKAGES = [
  {
    name: "@omena/napi-linux-x64-gnu",
    dir: "napi-linux-x64-gnu",
    nodeFile: "index.linux-x64-gnu.node",
    target: "x86_64-unknown-linux-gnu",
    os: ["linux"],
    cpu: ["x64"],
    libc: ["glibc"],
  },
  {
    name: "@omena/napi-linux-arm64-gnu",
    dir: "napi-linux-arm64-gnu",
    nodeFile: "index.linux-arm64-gnu.node",
    target: "aarch64-unknown-linux-gnu",
    os: ["linux"],
    cpu: ["arm64"],
    libc: ["glibc"],
  },
  {
    name: "@omena/napi-darwin-x64",
    dir: "napi-darwin-x64",
    nodeFile: "index.darwin-x64.node",
    target: "x86_64-apple-darwin",
    os: ["darwin"],
    cpu: ["x64"],
  },
  {
    name: "@omena/napi-darwin-arm64",
    dir: "napi-darwin-arm64",
    nodeFile: "index.darwin-arm64.node",
    target: "aarch64-apple-darwin",
    os: ["darwin"],
    cpu: ["arm64"],
  },
  {
    name: "@omena/napi-win32-x64-msvc",
    dir: "napi-win32-x64-msvc",
    nodeFile: "index.win32-x64-msvc.node",
    target: "x86_64-pc-windows-msvc",
    os: ["win32"],
    cpu: ["x64"],
  },
];

if (!existsSync(pkgDir)) {
  throw new Error(`Missing ${pkgDir}; run \`napi build ... -o pkg\` (build:omena-napi) first`);
}

if (!existsSync(loaderPath)) {
  throw new Error(`Missing ${loaderPath}; run \`napi build --platform ...\` first`);
}

const workspaceVersion = readWorkspaceVersion();
rewriteLoader(workspaceVersion);
writeMainPackageJson(workspaceVersion);
const preparedPackages = writePlatformPackages(workspaceVersion);

console.log(
  [
    `Finalized ${path.relative(repoRoot, pkgJsonPath)}: ${MAIN_PACKAGE_NAME}@${workspaceVersion}`,
    `Prepared native packages: ${preparedPackages.length ? preparedPackages.join(", ") : "(none)"}`,
  ].join("\n"),
);

function readWorkspaceVersion() {
  const workspaceCargoToml = readFileSync(path.join(repoRoot, "rust", "Cargo.toml"), "utf8");
  const versionMatch = workspaceCargoToml.match(
    /\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/,
  );
  if (!versionMatch) {
    throw new Error("Could not read [workspace.package].version from rust/Cargo.toml");
  }
  return versionMatch[1];
}

function rewriteLoader(expectedVersion) {
  const original = readFileSync(loaderPath, "utf8");
  const next = original
    .replaceAll("css-module-explainer-", "@omena/napi-")
    .replaceAll(
      /bindingPackageVersion !== '[^']+'/g,
      `bindingPackageVersion !== '${expectedVersion}'`,
    )
    .replaceAll(/expected [^ ]+ but got/g, `expected ${expectedVersion} but got`);

  if (next.includes("css-module-explainer")) {
    throw new Error(
      `Refusing to finalize ${path.relative(repoRoot, loaderPath)}: stale css-module-explainer package name remains`,
    );
  }

  if (!next.includes("@omena/napi-linux-x64-gnu")) {
    throw new Error(
      `Refusing to finalize ${path.relative(repoRoot, loaderPath)}: @omena/napi fallback names were not generated`,
    );
  }

  if (next !== original) {
    writeFileSync(loaderPath, next, "utf8");
  }
}

function writeMainPackageJson(expectedVersion) {
  const pkg = {
    name: MAIN_PACKAGE_NAME,
    version: expectedVersion,
    description: DESCRIPTION,
    license: "MIT",
    repository: { type: "git", url: REPOSITORY_URL },
    main: "index.js",
    types: "index.d.ts",
    engines: { node: NODE_ENGINE },
    files: ["index.js", "index.d.ts"],
    optionalDependencies: Object.fromEntries(
      PLATFORM_PACKAGES.map((platformPackage) => [platformPackage.name, expectedVersion]),
    ),
    napi: {
      binaryName: "index",
      targets: PLATFORM_PACKAGES.map((platformPackage) => platformPackage.target),
    },
    publishConfig: { access: "public" },
  };

  writeFileSync(pkgJsonPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
}

function writePlatformPackages(expectedVersion) {
  rmSync(platformRoot, { recursive: true, force: true });
  mkdirSync(platformRoot, { recursive: true });

  const prepared = [];
  for (const platformPackage of PLATFORM_PACKAGES) {
    const sourceBinary = path.join(pkgDir, platformPackage.nodeFile);
    if (!existsSync(sourceBinary)) {
      continue;
    }

    const platformDir = path.join(platformRoot, platformPackage.dir);
    mkdirSync(platformDir, { recursive: true });
    copyFileSync(sourceBinary, path.join(platformDir, platformPackage.nodeFile));

    const pkg = {
      name: platformPackage.name,
      version: expectedVersion,
      description: `${DESCRIPTION} (${platformPackage.dir})`,
      license: "MIT",
      repository: { type: "git", url: REPOSITORY_URL },
      main: platformPackage.nodeFile,
      engines: { node: NODE_ENGINE },
      os: platformPackage.os,
      cpu: platformPackage.cpu,
      ...(platformPackage.libc ? { libc: platformPackage.libc } : {}),
      files: [platformPackage.nodeFile],
      publishConfig: { access: "public" },
    };

    writeFileSync(
      path.join(platformDir, "package.json"),
      `${JSON.stringify(pkg, null, 2)}\n`,
      "utf8",
    );
    prepared.push(platformPackage.name);
  }
  return prepared;
}
