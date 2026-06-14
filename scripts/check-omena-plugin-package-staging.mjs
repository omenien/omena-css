#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

const repoRoot = process.cwd();
const tempRoot = mkdtempSync(path.join(os.tmpdir(), "omena-plugin-package-staging-"));
const stageRoot = path.join(tempRoot, "stage");
const packRoot = path.join(tempRoot, "packs");
const consumerRoot = path.join(tempRoot, "consumer");
const consumerRequire = createRequire(path.join(consumerRoot, "index.cjs"));

const PACKAGES = [
  {
    name: "@omena/css-build-adapter",
    dir: "css-build-adapter",
    exports: ["createOmenaBuildState", "rebuildAndCache", "runOmenaBuild"],
  },
  {
    name: "@omena/vite-plugin",
    dir: "vite-plugin",
    exports: ["omenaCss"],
  },
  {
    name: "@omena/postcss-plugin",
    dir: "postcss-plugin",
    exports: ["omenaPostcss"],
  },
];

mkdirSync(packRoot, { recursive: true });
mkdirSync(consumerRoot, { recursive: true });

const workspaceVersion = readWorkspaceVersion();
run(process.execPath, ["./scripts/finalize-omena-plugin-pkgs.mjs", "--out", stageRoot], {
  cwd: repoRoot,
});

const tarballs = [];
for (const packageSpec of PACKAGES) {
  const packageRoot = path.join(stageRoot, packageSpec.dir);
  const manifestPath = path.join(packageRoot, "package.json");
  assert.ok(existsSync(manifestPath), `staged package must include ${manifestPath}`);

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  assert.equal(manifest.name, packageSpec.name);
  assert.equal(manifest.version, workspaceVersion);
  assert.equal(manifest.private, undefined, `${packageSpec.name} must not be private`);
  assert.equal(manifest.publishConfig?.access, "public");
  assertNoWorkspaceProtocol(packageSpec.name, manifest.dependencies);
  assertNoWorkspaceProtocol(packageSpec.name, manifest.peerDependencies);
  assertNoWorkspaceProtocol(packageSpec.name, manifest.optionalDependencies);

  const packed = run("npm", ["pack", "--json", "--pack-destination", packRoot], {
    cwd: packageRoot,
  });
  const packReport = JSON.parse(packed.stdout);
  const fileName = packReport[0]?.filename;
  assert.ok(fileName, `npm pack must report a tarball for ${packageSpec.name}`);
  const tarball = path.join(packRoot, fileName);
  assert.ok(existsSync(tarball), `npm pack must write ${tarball}`);
  tarballs.push(tarball);
}

writeFileSync(
  path.join(consumerRoot, "package.json"),
  `${JSON.stringify({ private: true, type: "commonjs" }, null, 2)}\n`,
);
run(
  "npm",
  [
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--package-lock=false",
    ...tarballs,
  ],
  { cwd: consumerRoot },
);

for (const packageSpec of PACKAGES) {
  const imported = requireFromConsumer(packageSpec.name);
  for (const exportName of packageSpec.exports) {
    assert.equal(
      typeof imported[exportName],
      "function",
      `${packageSpec.name} must expose function export ${exportName}`,
    );
  }
}

console.log(
  `Omena plugin package staging ok: ${PACKAGES.map(({ name }) => `${name}@${workspaceVersion}`).join(", ")}`,
);

function readWorkspaceVersion() {
  const workspaceCargoToml = readFileSync(path.join(repoRoot, "rust", "Cargo.toml"), "utf8");
  const versionMatch = workspaceCargoToml.match(
    /\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/,
  );
  assert.ok(versionMatch, "workspace package version must be present in rust/Cargo.toml");
  return versionMatch[1];
}

function assertNoWorkspaceProtocol(packageName, dependencies) {
  if (!dependencies) return;
  for (const [dependencyName, range] of Object.entries(dependencies)) {
    assert.equal(
      typeof range === "string" && range.startsWith("workspace:"),
      false,
      `${packageName} dependency ${dependencyName} must not use ${range}`,
    );
  }
}

function requireFromConsumer(packageName) {
  const requireScript = [
    `const imported = require(${JSON.stringify(packageName)});`,
    `console.log(JSON.stringify(Object.keys(imported).sort()));`,
  ].join("\n");
  run(process.execPath, ["-e", requireScript], { cwd: consumerRoot });
  return consumerRequire(packageName);
}

function run(command, args, options = {}) {
  const child = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    encoding: "utf8",
    env: { ...process.env, ...(options.env ?? {}) },
  });
  if (child.status !== 0) {
    throw new Error(
      [
        `Command failed: ${command} ${args.join(" ")}`,
        `cwd: ${options.cwd ?? repoRoot}`,
        `status: ${child.status}`,
        `stdout:\n${child.stdout}`,
        `stderr:\n${child.stderr}`,
      ].join("\n"),
    );
  }
  return child;
}
