#!/usr/bin/env node
// Stages the build-tool integration packages for npm publication.
//
// Source package manifests intentionally remain private workspace packages while the
// bundler is still an omena-css mode. This script creates publish-only manifests
// with the crate-train version, public metadata, and concrete internal dependency
// versions so npm never sees `workspace:*`.
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const repoRoot = path.resolve(import.meta.dirname, "..");
const REPOSITORY_URL = "https://github.com/omenien/omena-css";
const NODE_ENGINE = ">=22";

const PACKAGE_SPECS = [
  {
    name: "@omena/css-build-adapter",
    sourceDir: "packages/css-build-adapter",
    outputDir: "css-build-adapter",
    description: "Shared build adapter for Omena CSS Vite and PostCSS integrations",
    additionalFiles: ["semantic-minify-pass-ids.json"],
  },
  {
    name: "@omena/vite-plugin",
    sourceDir: "packages/vite-plugin",
    outputDir: "vite-plugin",
    description: "Vite integration for the Omena CSS in-process build pipeline",
  },
  {
    name: "@omena/postcss-plugin",
    sourceDir: "packages/postcss-plugin",
    outputDir: "postcss-plugin",
    description: "PostCSS integration for the Omena CSS in-process build pipeline",
  },
];

const cliArgs = process.argv.slice(2).filter((argument) => argument !== "--");
const { values } = parseArgs({
  args: cliArgs,
  options: {
    out: { type: "string", short: "o" },
  },
});

const outputRoot = values.out
  ? path.resolve(values.out)
  : path.join(repoRoot, "dist", "npm-plugin-packages");
const workspaceVersion = readWorkspaceVersion();

rmSync(outputRoot, { recursive: true, force: true });
mkdirSync(outputRoot, { recursive: true });

const stagedPackages = [];
for (const spec of PACKAGE_SPECS) {
  stagedPackages.push(stagePackage(spec, workspaceVersion));
}

console.log(
  [
    `Staged Omena build-tool packages at ${path.relative(repoRoot, outputRoot)}`,
    ...stagedPackages.map(
      ({ name, version, dir }) => `  ${name}@${version} -> ${path.relative(repoRoot, dir)}`,
    ),
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

function stagePackage(spec, version) {
  const sourceDir = path.join(repoRoot, spec.sourceDir);
  const outputDir = path.join(outputRoot, spec.outputDir);
  const sourceManifestPath = path.join(sourceDir, "package.json");
  if (!existsSync(sourceManifestPath)) {
    throw new Error(`Missing package manifest for ${spec.name}: ${sourceManifestPath}`);
  }

  mkdirSync(outputDir, { recursive: true });
  copyRequiredFile(sourceDir, outputDir, "index.cjs");
  copyRequiredFile(sourceDir, outputDir, "index.d.ts");
  for (const fileName of spec.additionalFiles ?? []) {
    copyRequiredFile(sourceDir, outputDir, fileName);
  }
  copyOptionalFile(sourceDir, outputDir, "README.md");
  copyFileSync(path.join(repoRoot, "LICENSE"), path.join(outputDir, "LICENSE"));

  const sourceManifest = JSON.parse(readFileSync(sourceManifestPath, "utf8"));
  const publishManifest = normalizePublishManifest(spec, sourceManifest, version);
  assertPublishManifest(spec, publishManifest, version);
  writeFileSync(
    path.join(outputDir, "package.json"),
    `${JSON.stringify(publishManifest, null, 2)}\n`,
    "utf8",
  );

  return { name: publishManifest.name, version: publishManifest.version, dir: outputDir };
}

function copyRequiredFile(sourceDir, outputDir, fileName) {
  const sourcePath = path.join(sourceDir, fileName);
  if (!existsSync(sourcePath)) {
    throw new Error(`Missing required publish file: ${sourcePath}`);
  }
  copyFileSync(sourcePath, path.join(outputDir, fileName));
}

function copyOptionalFile(sourceDir, outputDir, fileName) {
  const sourcePath = path.join(sourceDir, fileName);
  if (existsSync(sourcePath)) {
    copyFileSync(sourcePath, path.join(outputDir, fileName));
  }
}

function normalizePublishManifest(spec, sourceManifest, version) {
  const manifest = {
    name: spec.name,
    version,
    description: spec.description,
    license: "MIT",
    repository: { type: "git", url: REPOSITORY_URL },
    homepage: "https://github.com/omenien/omena-css#readme",
    bugs: { url: "https://github.com/omenien/omena-css/issues" },
    main: sourceManifest.main ?? "./index.cjs",
    types: sourceManifest.types ?? "./index.d.ts",
    exports: sourceManifest.exports,
    files: existingPublishFiles(path.join(repoRoot, spec.sourceDir), spec.additionalFiles),
    engines: { node: NODE_ENGINE },
    ...copyObjectField(sourceManifest, "dependencies", version),
    ...copyObjectField(sourceManifest, "peerDependencies", version),
    ...copyObjectField(sourceManifest, "optionalDependencies", version),
    publishConfig: { access: "public" },
  };

  return removeUndefinedFields(manifest);
}

function copyObjectField(sourceManifest, fieldName, version) {
  const value = sourceManifest[fieldName];
  if (!value) return {};
  return { [fieldName]: normalizeDependencies(value, version, fieldName) };
}

function normalizeDependencies(dependencies, version, fieldName) {
  return Object.fromEntries(
    Object.entries(dependencies).map(([name, range]) => {
      if (typeof range === "string" && range.startsWith("workspace:")) {
        if (!name.startsWith("@omena/")) {
          throw new Error(`Refusing to rewrite non-Omena workspace dependency ${name}`);
        }
        return [name, version];
      }
      if (
        fieldName === "optionalDependencies" &&
        (name === "@omena/napi" || name === "@omena/wasm")
      ) {
        return [name, `^${version}`];
      }
      return [name, range];
    }),
  );
}

function existingPublishFiles(sourceDir, additionalFiles = []) {
  return ["index.cjs", "index.d.ts", "README.md", "LICENSE", ...additionalFiles].filter(
    (fileName) => {
      if (fileName === "LICENSE") return true;
      return existsSync(path.join(sourceDir, fileName));
    },
  );
}

function removeUndefinedFields(object) {
  return Object.fromEntries(Object.entries(object).filter(([, value]) => value !== undefined));
}

function assertPublishManifest(spec, manifest, version) {
  if (manifest.name !== spec.name) {
    throw new Error(`${spec.name}: staged name drifted to ${manifest.name}`);
  }
  if (manifest.version !== version || manifest.version === "0.0.0") {
    throw new Error(`${spec.name}: staged version is ${manifest.version}, expected ${version}`);
  }
  if (manifest.private === true) {
    throw new Error(`${spec.name}: staged manifest must not be private`);
  }
  const repositoryUrl =
    typeof manifest.repository === "string" ? manifest.repository : manifest.repository?.url;
  if (repositoryUrl !== REPOSITORY_URL) {
    throw new Error(`${spec.name}: repository.url is ${repositoryUrl}, expected ${REPOSITORY_URL}`);
  }
  assertNoWorkspaceProtocol(spec.name, manifest.dependencies);
  assertNoWorkspaceProtocol(spec.name, manifest.peerDependencies);
  assertNoWorkspaceProtocol(spec.name, manifest.optionalDependencies);
}

function assertNoWorkspaceProtocol(packageName, dependencies) {
  if (!dependencies) return;
  for (const [dependencyName, range] of Object.entries(dependencies)) {
    if (typeof range === "string" && range.startsWith("workspace:")) {
      throw new Error(`${packageName}: ${dependencyName} still uses ${range}`);
    }
  }
}
