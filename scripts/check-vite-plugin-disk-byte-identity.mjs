import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const styleModulePattern = /\.module\.(?:css|less|scss)$/u;
const baselineArgument = readArgument("--baseline");
const currentArgument = readArgument("--current") ?? "HEAD";

if (!baselineArgument) {
  throw new Error(
    "Usage: node scripts/check-vite-plugin-disk-byte-identity.mjs --baseline <git-ref> [--current <git-ref>]",
  );
}

const baseline = resolveCommit(baselineArgument);
const current = resolveCommit(currentArgument);
const baselineFiles = listStyleModules(baseline);
const currentFiles = listStyleModules(current);
const baselineSet = new Set(baselineFiles);
const currentSet = new Set(currentFiles);
const removed = baselineFiles.filter((file) => !currentSet.has(file));
const added = currentFiles.filter((file) => !baselineSet.has(file));

if (removed.length > 0) {
  throw new Error(
    `Vite disk-backed byte identity cannot drop baseline inputs: ${removed.join(", ")}`,
  );
}

const baselinePlugin = loadPluginAt(baseline);
const currentPlugin = loadPluginAt(current);
const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-disk-byte-"));

try {
  for (const [index, relativePath] of baselineFiles.entries()) {
    const extension = path.extname(relativePath);
    const probePath = path.join(tempRoot, `input-${index}.module${extension}`);
    const source = readFileAt(baseline, relativePath);
    fs.writeFileSync(probePath, source, "utf8");
    const baselineBytes = await transformBytes(baselinePlugin, probePath, source);
    const currentBytes = await transformBytes(currentPlugin, probePath, source);
    if (!baselineBytes.equals(currentBytes)) {
      throw new Error(`Vite disk-backed output bytes changed for ${relativePath}`);
    }
  }
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}

console.log(
  `Vite disk-backed byte identity: compared=${baselineFiles.length} identical=${baselineFiles.length} added=${added.length} removed=${removed.length} baseline=${baseline} current=${current}`,
);

function readArgument(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return null;
  const value = process.argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${name} requires a git ref.`);
  }
  return value;
}

function git(args, options = {}) {
  return execFileSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
}

function resolveCommit(ref) {
  return git(["rev-parse", "--verify", `${ref}^{commit}`]).trim();
}

function listStyleModules(ref) {
  return git([
    "ls-tree",
    "-r",
    "--name-only",
    ref,
    "--",
    "examples",
    "test/_fixtures/real-project-corpus",
  ])
    .split("\n")
    .filter((file) => styleModulePattern.test(file))
    .sort();
}

function readFileAt(ref, relativePath) {
  return git(["show", `${ref}:${relativePath}`]);
}

function loadPluginAt(ref) {
  const adapterFilename = path.join(repoRoot, "packages/css-build-adapter/index.cjs");
  const pluginFilename = path.join(repoRoot, "packages/vite-plugin/index.cjs");
  const semanticPassIds = JSON.parse(
    readFileAt(ref, "packages/css-build-adapter/semantic-minify-pass-ids.json"),
  );
  const adapter = evaluateCommonJs(
    readFileAt(ref, "packages/css-build-adapter/index.cjs"),
    adapterFilename,
    (specifier) => {
      if (specifier === "./semantic-minify-pass-ids.json") return semanticPassIds;
      return require(specifier);
    },
  );
  return evaluateCommonJs(
    readFileAt(ref, "packages/vite-plugin/index.cjs"),
    pluginFilename,
    (specifier) => {
      if (specifier === "@omena/css-build-adapter") return adapter;
      return require(specifier);
    },
  ).omenaCss;
}

function evaluateCommonJs(source, filename, localRequire) {
  const module = { exports: {} };
  const evaluate = new Function("exports", "require", "module", "__filename", "__dirname", source);
  evaluate(module.exports, localRequire, module, filename, path.dirname(filename));
  return module.exports;
}

async function transformBytes(createPlugin, filePath, source) {
  const warnings = [];
  const plugin = createPlugin({
    configFile: false,
    cwd: repoRoot,
    engine: createProbeEngine(),
    moduleInterface: false,
    passes: [],
  });
  const result = await plugin.transform.call(
    { warn: (message) => warnings.push(String(message)) },
    source,
    filePath,
  );
  if (!result || typeof result.code !== "string") {
    throw new Error(`Expected transformed disk-backed output for ${filePath}`);
  }
  if (warnings.length > 0) {
    throw new Error(`Unexpected disk-backed warnings: ${warnings.join(" | ")}`);
  }
  return Buffer.from(`${JSON.stringify({ code: result.code, map: result.map })}\n`, "utf8");
}

function createProbeEngine() {
  return {
    buildSnapshotIdentity() {
      return {
        schemaVersion: "0",
        product: "omena-query.build-snapshot-digest",
        contentHashAlgorithm: "blake3",
        digest: "blake3:test-probe-snapshot",
        targetSourceDigest: "blake3:test-probe-target-source",
      };
    },
    buildStyleSourcesWithContextJson(targetPath, sourcesJson) {
      const [source] = JSON.parse(sourcesJson);
      return JSON.stringify({
        execution: {
          outputCss: `${source.styleSource}\n/* deterministic disk-backed probe */\n`,
          executedPassIds: [],
        },
        sourceMapV3: {
          version: 3,
          sources: [targetPath],
          names: [],
          mappings: "AAAA",
        },
      });
    },
  };
}
