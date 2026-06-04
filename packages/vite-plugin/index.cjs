const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const DEFAULT_INCLUDE = /\.module\.css$/;
const REPO_ROOT = path.resolve(__dirname, "../..");

function omenaCss(options = {}) {
  const include = options.include ?? DEFAULT_INCLUDE;
  const pluginName = options.name ?? "omena-css";

  return {
    name: pluginName,
    enforce: options.enforce ?? "pre",
    transform(code, id) {
      if (!matchesInclude(id, include)) return null;
      if (!fs.existsSync(id)) return null;
      if (options.requireDiskSource !== false) {
        const diskSource = fs.readFileSync(id, "utf8");
        if (diskSource !== code) {
          this.warn?.(
            `[${pluginName}] skipped ${id}: transform input differs from disk source; set requireDiskSource=false to allow disk-backed transforms.`,
          );
          return null;
        }
      }

      const output = runOmenaBuild(id, options);
      if (output.code === code) return null;
      return {
        code: output.code,
        map: output.map,
      };
    },
  };
}

function runOmenaBuild(filePath, options) {
  const invocation = resolveOmenaCliInvocation(options);
  const args = [...invocation.args, "build", filePath];
  const includeSourceMap = options.sourceMap !== false;
  for (const passId of options.passes ?? []) {
    args.push("--pass", passId);
  }
  if (options.targetQuery) {
    args.push("--target-query", options.targetQuery);
  }
  if (options.closedStyleWorld) {
    args.push("--closed-style-world");
  }
  for (const sourcePath of options.sources ?? []) {
    args.push("--source", sourcePath);
  }
  for (const manifestPath of options.packageManifests ?? []) {
    args.push("--package-manifest", manifestPath);
  }
  if (includeSourceMap) {
    args.push("--json", "--source-map");
  }
  const stdout = execFileSync(invocation.command, args, {
    cwd: options.cwd ?? process.cwd(),
    encoding: "utf8",
    env: process.env,
  });
  if (!includeSourceMap) {
    return { code: stdout, map: null };
  }
  const summary = JSON.parse(stdout);
  if (typeof summary.execution?.outputCss !== "string") {
    throw new Error(`[omena-css] invalid omena build JSON: missing execution.outputCss`);
  }
  return {
    code: summary.execution.outputCss,
    map: summary.sourceMapV3 ?? null,
  };
}

function resolveOmenaCliInvocation(options) {
  if (options.omenaBin) {
    return { command: options.omenaBin, args: [] };
  }
  if (process.env.OMENA_CLI_BIN) {
    return { command: process.env.OMENA_CLI_BIN, args: [] };
  }
  const manifestPath = options.cargoManifestPath ?? path.join(REPO_ROOT, "rust/Cargo.toml");
  if (fs.existsSync(manifestPath)) {
    return {
      command: "cargo",
      args: ["run", "--manifest-path", manifestPath, "-p", "omena-cli", "--quiet", "--"],
    };
  }
  throw new Error("Unable to find omena CLI. Set omenaBin, OMENA_CLI_BIN, or cargoManifestPath.");
}

function matchesInclude(id, include) {
  if (include instanceof RegExp) return include.test(id);
  if (typeof include === "function") return Boolean(include(id));
  if (Array.isArray(include)) return include.some((entry) => matchesInclude(id, entry));
  if (typeof include === "string") return id.endsWith(include);
  return false;
}

module.exports = {
  omenaCss,
  default: omenaCss,
};
