#!/usr/bin/env node
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const pkgDir = path.join(repoRoot, "rust", "crates", "omena-napi", "pkg");
const platformRoot = path.join(pkgDir, "npm");
const mainPackageJsonPath = path.join(pkgDir, "package.json");
const loaderPath = path.join(pkgDir, "index.js");

const PLATFORM_PACKAGES = [
  {
    name: "@omena/napi-linux-x64-gnu",
    dir: "napi-linux-x64-gnu",
    nodeFile: "index.linux-x64-gnu.node",
    os: ["linux"],
    cpu: ["x64"],
    libc: ["glibc"],
  },
  {
    name: "@omena/napi-linux-arm64-gnu",
    dir: "napi-linux-arm64-gnu",
    nodeFile: "index.linux-arm64-gnu.node",
    os: ["linux"],
    cpu: ["arm64"],
    libc: ["glibc"],
  },
  {
    name: "@omena/napi-darwin-x64",
    dir: "napi-darwin-x64",
    nodeFile: "index.darwin-x64.node",
    os: ["darwin"],
    cpu: ["x64"],
  },
  {
    name: "@omena/napi-darwin-arm64",
    dir: "napi-darwin-arm64",
    nodeFile: "index.darwin-arm64.node",
    os: ["darwin"],
    cpu: ["arm64"],
  },
  {
    name: "@omena/napi-win32-x64-msvc",
    dir: "napi-win32-x64-msvc",
    nodeFile: "index.win32-x64-msvc.node",
    os: ["win32"],
    cpu: ["x64"],
  },
];

const packageByName = new Map(
  PLATFORM_PACKAGES.map((platformPackage) => [platformPackage.name, platformPackage]),
);

if (!existsSync(mainPackageJsonPath)) {
  throw new Error(
    `Missing ${mainPackageJsonPath}; run pnpm omena-check run core/build/omena-napi first`,
  );
}
if (!existsSync(loaderPath)) {
  throw new Error(`Missing ${loaderPath}; run pnpm omena-check run core/build/omena-napi first`);
}

const workspaceVersion = readWorkspaceVersion();
const mainPackage = JSON.parse(readFileSync(mainPackageJsonPath, "utf8"));
assertMainPackage(workspaceVersion, mainPackage);
assertLoader();

const requiredPackageNames = parseRequiredPackageNames();
for (const packageName of requiredPackageNames) {
  const platformPackage = packageByName.get(packageName);
  if (!platformPackage) {
    throw new Error(`Unknown required @omena/napi native package: ${packageName}`);
  }
  assertPlatformPackage(workspaceVersion, platformPackage);
}

const currentPackage = currentPlatformPackage();
if (requiredPackageNames.includes(currentPackage.name)) {
  runPackedInstallSmoke(currentPackage);
}

console.log(
  `@omena/napi package layout ok: required=${requiredPackageNames.join(", ")} version=${workspaceVersion}`,
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

function assertMainPackage(expectedVersion, pkg) {
  assertEqual(pkg.name, "@omena/napi", "main package name");
  assertEqual(pkg.version, expectedVersion, "main package version");
  assertEqual(pkg.main, "index.js", "main package entrypoint");
  assertEqual(pkg.types, "index.d.ts", "main package types");

  const optionalDependencies = pkg.optionalDependencies ?? {};
  for (const platformPackage of PLATFORM_PACKAGES) {
    assertEqual(
      optionalDependencies[platformPackage.name],
      expectedVersion,
      `optionalDependency ${platformPackage.name}`,
    );
  }

  const files = new Set(pkg.files ?? []);
  if (!files.has("index.js") || !files.has("index.d.ts")) {
    throw new Error(
      `@omena/napi files must include index.js and index.d.ts, got ${[...files].join(", ")}`,
    );
  }
  if (files.has("*.node")) {
    throw new Error(
      "@omena/napi main package must not bundle native .node files; use platform optional packages",
    );
  }
}

function assertLoader() {
  const loader = readFileSync(loaderPath, "utf8");
  if (loader.includes("css-module-explainer")) {
    throw new Error(`${path.relative(repoRoot, loaderPath)} still references css-module-explainer`);
  }
  for (const platformPackage of PLATFORM_PACKAGES) {
    if (!loader.includes(platformPackage.name)) {
      throw new Error(
        `${path.relative(repoRoot, loaderPath)} does not reference ${platformPackage.name}`,
      );
    }
  }
}

function assertPlatformPackage(expectedVersion, platformPackage) {
  const platformDir = path.join(platformRoot, platformPackage.dir);
  const packageJsonPath = path.join(platformDir, "package.json");
  const nodePath = path.join(platformDir, platformPackage.nodeFile);
  if (!existsSync(packageJsonPath)) {
    throw new Error(`Missing platform package manifest: ${packageJsonPath}`);
  }
  if (!existsSync(nodePath)) {
    throw new Error(`Missing platform package native binding: ${nodePath}`);
  }

  const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
  assertEqual(pkg.name, platformPackage.name, `${platformPackage.name} name`);
  assertEqual(pkg.version, expectedVersion, `${platformPackage.name} version`);
  assertEqual(pkg.main, platformPackage.nodeFile, `${platformPackage.name} main`);
  assertArrayEqual(pkg.os, platformPackage.os, `${platformPackage.name} os`);
  assertArrayEqual(pkg.cpu, platformPackage.cpu, `${platformPackage.name} cpu`);
  if (platformPackage.libc) {
    assertArrayEqual(pkg.libc, platformPackage.libc, `${platformPackage.name} libc`);
  }
}

function runPackedInstallSmoke(platformPackage) {
  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-napi-pack-"));
  const packDir = path.join(tempRoot, "pack");
  const consumerDir = path.join(tempRoot, "consumer");
  mkdirSync(packDir, { recursive: true });
  mkdirSync(consumerDir, { recursive: true });

  const mainTarball = packPackage(pkgDir, packDir);
  const platformTarball = packPackage(path.join(platformRoot, platformPackage.dir), packDir);

  writeFileSync(
    path.join(consumerDir, "package.json"),
    `${JSON.stringify({ private: true, type: "commonjs" }, null, 2)}\n`,
  );
  run(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--package-lock=false",
      "--omit=optional",
      mainTarball,
      platformTarball,
    ],
    { cwd: consumerDir },
  );

  const smoke = `
const binding = require('@omena/napi');
if (typeof binding.buildStyleSourcesWithContextJson !== 'function') {
  throw new Error('missing buildStyleSourcesWithContextJson');
}
const sources = JSON.stringify([
  { stylePath: 'Button.module.css', styleSource: '.button { color: red; }' }
]);
const json = binding.buildStyleSourcesWithContextJson(
  'Button.module.css',
  sources,
  [],
  '',
  ''
);
const result = JSON.parse(json);
if (
  result.product !== 'omena-query.consumer-build-style-source' ||
  typeof result.execution?.outputCss !== 'string' ||
  !result.execution.outputCss.includes('color:red')
) {
  throw new Error('buildStyleSourcesWithContextJson returned unexpected output');
}
`;
  run(process.execPath, ["-e", smoke], { cwd: consumerDir });
}

function packPackage(packageDir, packDir) {
  const child = run("npm", ["pack", "--json", "--pack-destination", packDir], {
    cwd: packageDir,
    encoding: "utf8",
  });
  const packed = JSON.parse(child.stdout);
  const fileName = packed[0]?.filename;
  if (!fileName) {
    throw new Error(`npm pack did not report a filename for ${packageDir}`);
  }
  const tarball = path.join(packDir, fileName);
  if (!existsSync(tarball)) {
    throw new Error(`npm pack reported missing tarball ${tarball}`);
  }
  return tarball;
}

function parseRequiredPackageNames() {
  const raw = process.env.OMENA_NAPI_REQUIRED_PACKAGES;
  if (raw) {
    return raw
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
  }
  return [currentPlatformPackage().name];
}

function currentPlatformPackage() {
  if (process.platform === "linux" && process.arch === "x64") {
    return packageByName.get("@omena/napi-linux-x64-gnu");
  }
  if (process.platform === "linux" && process.arch === "arm64") {
    return packageByName.get("@omena/napi-linux-arm64-gnu");
  }
  if (process.platform === "darwin" && process.arch === "x64") {
    return packageByName.get("@omena/napi-darwin-x64");
  }
  if (process.platform === "darwin" && process.arch === "arm64") {
    return packageByName.get("@omena/napi-darwin-arm64");
  }
  if (process.platform === "win32" && process.arch === "x64") {
    return packageByName.get("@omena/napi-win32-x64-msvc");
  }
  throw new Error(`No @omena/napi package mapping for ${process.platform}-${process.arch}`);
}

function run(command, args, options) {
  const child = spawnSync(command, args, {
    cwd: options?.cwd ?? repoRoot,
    encoding: options?.encoding ?? "utf8",
    stdio: options?.encoding ? "pipe" : "inherit",
  });
  if (child.status !== 0) {
    throw new Error(
      [
        `Command failed: ${command} ${args.join(" ")}`,
        `cwd=${options?.cwd ?? repoRoot}`,
        child.error ? `error=${child.error.message}` : null,
        child.stdout?.trim() ? `stdout=${child.stdout.trim()}` : null,
        child.stderr?.trim() ? `stderr=${child.stderr.trim()}` : null,
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return child;
}

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${expected}, got ${actual}`);
  }
}

function assertArrayEqual(actual, expected, label) {
  if (!Array.isArray(actual) || actual.length !== expected.length) {
    throw new Error(
      `${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
  for (const value of expected) {
    if (!actual.includes(value)) {
      throw new Error(
        `${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
      );
    }
  }
}
