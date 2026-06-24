import { execFileSync } from "node:child_process";
import { readdirSync } from "node:fs";
import path from "node:path";
import {
  isPackagedExtensionRuntime,
  resolveSelectedQueryBackendKind,
  shouldUseEngineShadowRunnerDaemon,
} from "../server/engine-host-node/src/selected-query-backend";
import {
  buildStyleDocumentWithOmenaParser,
  resolveRuntimeStyleDocumentBuilder,
} from "../server/engine-host-node/src/omena-parser-style-document-builder";
import {
  buildTsgoProbeInvocation,
  resolveTsgoBinaryPathForEnv,
} from "../server/engine-host-node/src/tsgo-probe-type-resolver";
import { buildTsgoTypeFactApiOptions } from "../server/engine-host-node/src/tsgo-type-fact-collector";
import {
  resolveLspServerRuntimeSelection,
  resolveOmenaLspServerPath,
} from "../client/src/lsp-server-runtime-config";

const repoRoot = process.cwd();
const vsixFiles = readdirSync(repoRoot).filter((file) => file.endsWith(".vsix"));
if (vsixFiles.length !== 1) {
  throw new Error(`Expected exactly one VSIX in ${repoRoot}, found ${vsixFiles.length}`);
}

const vsixFile = vsixFiles[0]!;
const vsixPath = path.join(repoRoot, vsixFile);
const entries = readVsixEntries(vsixPath);
const platformDir = `${process.platform}-${process.arch}`;
const binaryName =
  process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner";
const tsgoBinaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const omenaLspServerBinaryName =
  process.platform === "win32" ? "omena-lsp-server.exe" : "omena-lsp-server";
const minimumRunnerTargets = Number.parseInt(
  process.env.OMENA_PACKAGED_RUNNER_MIN_TARGETS ?? "1",
  10,
);
const minimumTsgoTargets = Number.parseInt(process.env.OMENA_PACKAGED_TSGO_MIN_TARGETS ?? "1", 10);
const minimumOmenaLspServerTargets = Number.parseInt(
  process.env.OMENA_PACKAGED_LSP_SERVER_MIN_TARGETS ?? "1",
  10,
);
const requiredRunnerPlatforms = (process.env.OMENA_PACKAGED_RUNNER_REQUIRED_PLATFORMS ?? "")
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);
const requiredTsgoPlatforms = (process.env.OMENA_PACKAGED_TSGO_REQUIRED_PLATFORMS ?? "")
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);
const requiredOmenaLspServerPlatforms = (
  process.env.OMENA_PACKAGED_LSP_SERVER_REQUIRED_PLATFORMS ?? ""
)
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);

if (!Number.isInteger(minimumRunnerTargets) || minimumRunnerTargets < 1) {
  throw new Error(
    `OMENA_PACKAGED_RUNNER_MIN_TARGETS must be a positive integer, got ${process.env.OMENA_PACKAGED_RUNNER_MIN_TARGETS}`,
  );
}

if (!Number.isInteger(minimumTsgoTargets) || minimumTsgoTargets < 1) {
  throw new Error(
    `OMENA_PACKAGED_TSGO_MIN_TARGETS must be a positive integer, got ${process.env.OMENA_PACKAGED_TSGO_MIN_TARGETS}`,
  );
}

if (!Number.isInteger(minimumOmenaLspServerTargets) || minimumOmenaLspServerTargets < 1) {
  throw new Error(
    `OMENA_PACKAGED_LSP_SERVER_MIN_TARGETS must be a positive integer, got ${process.env.OMENA_PACKAGED_LSP_SERVER_MIN_TARGETS}`,
  );
}

for (const entry of [
  "extension/package.json",
  "extension/dist/client/extension.js",
  `extension/dist/bin/${platformDir}/${binaryName}`,
  `extension/dist/bin/${platformDir}/${omenaLspServerBinaryName}`,
  `extension/dist/bin/${platformDir}/${tsgoBinaryName}`,
  `extension/dist/bin/${platformDir}/lib.d.ts`,
]) {
  assertEntry(entries, entry);
}

const runnerTargets = readPackagedRunnerTargets(entries);
const tsgoTargets = readPackagedTsgoTargets(entries);
const omenaLspServerTargets = readPackagedOmenaLspServerTargets(entries);
if (runnerTargets.length < minimumRunnerTargets) {
  throw new Error(
    `Expected at least ${minimumRunnerTargets} packaged runner target(s), found ${runnerTargets.length}: ${runnerTargets.join(", ")}`,
  );
}
if (tsgoTargets.length < minimumTsgoTargets) {
  throw new Error(
    `Expected at least ${minimumTsgoTargets} packaged tsgo target(s), found ${tsgoTargets.length}: ${tsgoTargets.join(", ")}`,
  );
}
if (omenaLspServerTargets.length < minimumOmenaLspServerTargets) {
  throw new Error(
    `Expected at least ${minimumOmenaLspServerTargets} packaged omena-lsp-server target(s), found ${omenaLspServerTargets.length}: ${omenaLspServerTargets.join(", ")}`,
  );
}

for (const platform of requiredRunnerPlatforms) {
  if (!runnerTargets.some((target) => target.startsWith(`${platform}-`))) {
    throw new Error(
      `VSIX is missing packaged runner for required platform ${platform}; found ${runnerTargets.join(", ")}`,
    );
  }
}
for (const platform of requiredTsgoPlatforms) {
  if (!tsgoTargets.some((target) => target.startsWith(`${platform}-`))) {
    throw new Error(
      `VSIX is missing packaged tsgo for required platform ${platform}; found ${tsgoTargets.join(", ")}`,
    );
  }
}
for (const platform of requiredOmenaLspServerPlatforms) {
  if (!omenaLspServerTargets.some((target) => target.startsWith(`${platform}-`))) {
    throw new Error(
      `VSIX is missing packaged omena-lsp-server for required platform ${platform}; found ${omenaLspServerTargets.join(", ")}`,
    );
  }
}

for (const prefix of [
  "extension/rust/",
  "extension/client/",
  "extension/dist/server/",
  "extension/test/",
  "extension/scripts/",
  "extension/server/engine-host-node/",
  "extension/server/lsp-server/",
  "extension/.runner-artifacts/",
]) {
  assertNoPrefix(entries, prefix);
}

const packagedRoot = path.join(path.parse(repoRoot).root, "extension");
const fileExists = (filePath: string): boolean => {
  const relative = path.relative(packagedRoot, filePath);
  if (relative.startsWith("..") || path.isAbsolute(relative)) return false;
  return entries.has(`extension/${toPosixPath(relative)}`);
};
const packagedEnv = { OMENA_PROJECT_ROOT: packagedRoot } as NodeJS.ProcessEnv;

if (!isPackagedExtensionRuntime(packagedEnv, fileExists)) {
  throw new Error("VSIX file set did not satisfy packaged extension runtime detection");
}

const packagedTsgoPath = resolveTsgoBinaryPathForEnv(packagedEnv, fileExists);
if (!fileExists(packagedTsgoPath)) {
  throw new Error(`VSIX file set did not satisfy packaged tsgo detection: ${packagedTsgoPath}`);
}

const packagedOmenaLspServerPath = resolveOmenaLspServerPath(packagedRoot, packagedEnv, fileExists);
if (!packagedOmenaLspServerPath || !fileExists(packagedOmenaLspServerPath)) {
  throw new Error(
    `VSIX file set did not satisfy packaged omena-lsp-server detection: ${
      packagedOmenaLspServerPath ?? "null"
    }`,
  );
}
const packagedLspRuntime = resolveLspServerRuntimeSelection(
  "auto",
  packagedRoot,
  packagedEnv,
  fileExists,
);
if (
  packagedLspRuntime.runtime !== "omena-lsp-server" ||
  packagedLspRuntime.command !== packagedOmenaLspServerPath
) {
  throw new Error(
    `Expected packaged auto LSP runtime to use ${packagedOmenaLspServerPath}, got ${JSON.stringify(
      packagedLspRuntime,
    )}`,
  );
}

const tsgoInvocation = buildTsgoProbeInvocation(
  packagedRoot,
  path.join(packagedRoot, "tsconfig.json"),
  packagedEnv,
  fileExists,
);
if (!tsgoInvocation || tsgoInvocation.command !== packagedTsgoPath) {
  throw new Error(
    `Expected packaged tsgo probe invocation to use ${packagedTsgoPath}, got ${
      tsgoInvocation?.command ?? "null"
    }`,
  );
}

const tsgoTypeFactApiOptions = buildTsgoTypeFactApiOptions(packagedRoot, packagedEnv, fileExists);
if (tsgoTypeFactApiOptions.tsserverPath !== packagedTsgoPath) {
  throw new Error(
    `Expected packaged tsgo type-fact API to use ${packagedTsgoPath}, got ${
      tsgoTypeFactApiOptions.tsserverPath ?? "unset"
    }`,
  );
}

const defaultBackend = resolveSelectedQueryBackendKind(packagedEnv, fileExists);
if (defaultBackend !== "rust-selected-query") {
  throw new Error(`Expected packaged default backend rust-selected-query, got ${defaultBackend}`);
}

const defaultStyleBuilder = resolveRuntimeStyleDocumentBuilder(packagedEnv, fileExists);
if (defaultStyleBuilder !== buildStyleDocumentWithOmenaParser) {
  throw new Error("Expected packaged default style-document builder to use omena-parser");
}

const autoBackend = resolveSelectedQueryBackendKind(
  { ...packagedEnv, OMENA_SELECTED_QUERY_BACKEND: "auto" },
  fileExists,
);
if (autoBackend !== "rust-selected-query") {
  throw new Error(`Expected packaged auto backend rust-selected-query, got ${autoBackend}`);
}

if (!shouldUseEngineShadowRunnerDaemon(packagedEnv, fileExists)) {
  throw new Error("Expected packaged Rust selected-query runtime to use daemon by default");
}

if (
  shouldUseEngineShadowRunnerDaemon(
    { ...packagedEnv, OMENA_ENGINE_SHADOW_RUNNER_DAEMON: "0" },
    fileExists,
  )
) {
  throw new Error("Expected explicit OMENA_ENGINE_SHADOW_RUNNER_DAEMON=0 to disable daemon usage");
}

const explicitTypescriptBackend = resolveSelectedQueryBackendKind(
  { ...packagedEnv, OMENA_SELECTED_QUERY_BACKEND: "typescript-current" },
  fileExists,
);
if (explicitTypescriptBackend !== "typescript-current") {
  throw new Error(
    `Expected explicit typescript-current override to win, got ${explicitTypescriptBackend}`,
  );
}

const explicitTypescriptStyleBuilder = resolveRuntimeStyleDocumentBuilder(
  { ...packagedEnv, OMENA_STYLE_DOCUMENT_BUILDER: "typescript-current" },
  fileExists,
);
if (explicitTypescriptStyleBuilder !== undefined) {
  throw new Error("Expected explicit typescript-current style builder override to win");
}

console.log(
  `Packaged selected-query default ok: ${vsixFile} -> ${defaultBackend} styleBuilder=omena-parser daemon=on lsp=${packagedLspRuntime.runtime} lspTargets=${omenaLspServerTargets.join(",")}`,
);

function readVsixEntries(filePath: string): ReadonlySet<string> {
  const output = execFileSync("unzip", ["-Z1", filePath], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  return new Set(output.split(/\r?\n/u).filter(Boolean));
}

function readPackagedRunnerTargets(vsixEntries: ReadonlySet<string>): readonly string[] {
  const targetDirs = new Set<string>();
  for (const entry of vsixEntries) {
    const match = /^extension\/dist\/bin\/([^/]+)\/engine-shadow-runner(?:\.exe)?$/u.exec(entry);
    if (match) targetDirs.add(match[1]!);
  }
  return [...targetDirs].toSorted();
}

function readPackagedTsgoTargets(vsixEntries: ReadonlySet<string>): readonly string[] {
  const targetDirs = new Set<string>();
  for (const entry of vsixEntries) {
    const match = /^extension\/dist\/bin\/([^/]+)\/tsgo(?:\.exe)?$/u.exec(entry);
    if (match) targetDirs.add(match[1]!);
  }
  return [...targetDirs].toSorted();
}

function readPackagedOmenaLspServerTargets(vsixEntries: ReadonlySet<string>): readonly string[] {
  const targetDirs = new Set<string>();
  for (const entry of vsixEntries) {
    const match = /^extension\/dist\/bin\/([^/]+)\/omena-lsp-server(?:\.exe)?$/u.exec(entry);
    if (match) targetDirs.add(match[1]!);
  }
  return [...targetDirs].toSorted();
}

function assertEntry(vsixEntries: ReadonlySet<string>, entry: string): void {
  if (!vsixEntries.has(entry)) {
    throw new Error(`VSIX is missing required entry: ${entry}`);
  }
}

function assertNoPrefix(vsixEntries: ReadonlySet<string>, prefix: string): void {
  const match = [...vsixEntries].find((entry) => entry.startsWith(prefix));
  if (match) {
    throw new Error(`VSIX unexpectedly includes checkout-only entry: ${match}`);
  }
}

function toPosixPath(filePath: string): string {
  return filePath.split(path.sep).join("/");
}
