const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const SOURCE_FILE_PATTERN = /\.[cm]?[jt]sx?$/;
const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|sass|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../../../");
const DIRECT_SOURCE_DIAGNOSTICS_CACHE = new Map();
const DIRECT_SOURCE_DIAGNOSTIC_CODES = new Set([
  "missing-module",
  "missing-static-class",
  "missing-template-prefix",
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
]);
const OMENA_QUERY_SOURCE_DIAGNOSTIC_CODE_MAP = new Map([
  ["missingModule", "missing-module"],
  ["missingStaticClass", "missing-static-class"],
  ["missingTemplatePrefix", "missing-template-prefix"],
  ["missingResolvedClassValues", "missing-resolved-class-values"],
  ["missingResolvedClassDomain", "missing-resolved-class-domain"],
]);

module.exports = {
  SOURCE_FILE_PATTERN,
  formatQueryDiagnostic,
  getRuleOptions,
  resolveWorkspaceRoot,
  runSourceChecks,
  toEslintLoc,
};

function getRuleOptions(context) {
  const options = context.options[0] ?? {};
  const workspaceRoot = resolveWorkspaceRoot(context.filename, options.workspaceRoot);
  return {
    workspaceRoot,
    classnameTransform: options.classnameTransform ?? "asIs",
    includeMissingModule: options.includeMissingModule ?? true,
    pathAlias: options.pathAlias ?? {},
  };
}

function runSourceChecks(context, ruleOptions) {
  return readDirectSourceDiagnostics(context, ruleOptions);
}

function formatQueryDiagnostic(finding) {
  if (typeof finding.message === "string" && finding.message.length > 0) {
    return finding.message;
  }
  return `Omena source diagnostic '${finding.code}'.`;
}

function readDirectSourceDiagnostics(context, ruleOptions) {
  const includeCodes = resolveIncludeCodes(ruleOptions);

  const workspaceStylePaths = resolveWorkspaceStyleModulePaths(ruleOptions.workspaceRoot);
  const cacheKey = JSON.stringify({
    filePath: context.filename,
    sourceText: context.sourceCode.text,
    includeCodes: [...includeCodes].toSorted(),
    workspaceStylePaths: workspacePathSignature(workspaceStylePaths),
    cli: process.env.OMENA_CLI_BIN ?? null,
  });
  const cached = DIRECT_SOURCE_DIAGNOSTICS_CACHE.get(cacheKey);
  if (cached) return cached;

  const report = readOmenaCliSourceDiagnostics(context.filename, ruleOptions, workspaceStylePaths);
  if (!report) return null;

  const includeCodeSet = new Set(includeCodes);
  const findings = (report.diagnostics ?? [])
    .map((diagnostic) => {
      const code = OMENA_QUERY_SOURCE_DIAGNOSTIC_CODE_MAP.get(diagnostic.code);
      if (!code) return null;
      return {
        filePath: context.filename,
        code,
        category: "source",
        severity: "warning",
        range: diagnostic.range,
        message: diagnostic.message,
      };
    })
    .filter((finding) => finding && includeCodeSet.has(finding.code));

  DIRECT_SOURCE_DIAGNOSTICS_CACHE.set(cacheKey, findings);
  return findings;
}

function resolveIncludeCodes(ruleOptions) {
  const includeCodes =
    Array.isArray(ruleOptions.includeCodes) && ruleOptions.includeCodes.length > 0
      ? ruleOptions.includeCodes
      : [...DIRECT_SOURCE_DIAGNOSTIC_CODES];
  if (!includeCodes.every((code) => DIRECT_SOURCE_DIAGNOSTIC_CODES.has(code))) {
    return [];
  }
  if (ruleOptions.includeMissingModule === false) {
    return includeCodes.filter((code) => code !== "missing-module");
  }
  return includeCodes;
}

function readOmenaCliSourceDiagnostics(filePath, ruleOptions, workspaceStylePaths) {
  const invocation = resolveOmenaCliInvocation();
  if (!invocation) {
    throw new Error("Unable to find omena CLI. Set OMENA_CLI_BIN to a built omena-cli binary.");
  }

  const args = [
    ...invocation.args,
    "source-diagnostics",
    filePath,
    "--source-path",
    filePath,
    "--json",
  ];
  for (const sourcePath of workspaceStylePaths) {
    args.push("--source", sourcePath);
  }
  const stdout = execFileSync(invocation.command, args, {
    cwd: ruleOptions.workspaceRoot,
    encoding: "utf8",
    env: process.env,
  });
  return unwrapOmenaCliResponse(JSON.parse(stdout), "omena-cli.source-diagnostics");
}

function unwrapOmenaCliResponse(value, expectedProduct) {
  if (
    !value ||
    typeof value !== "object" ||
    value.schemaVersion !== "0" ||
    value.product !== expectedProduct ||
    !("payload" in value)
  ) {
    throw new Error(`Unexpected omena CLI response envelope for ${expectedProduct}.`);
  }
  return value.payload;
}

function resolveOmenaCliInvocation() {
  if (process.env.OMENA_CLI_BIN) {
    return { command: process.env.OMENA_CLI_BIN, args: [] };
  }

  const manifestPath = path.join(REPO_ROOT, "rust/Cargo.toml");
  if (fs.existsSync(manifestPath)) {
    return {
      command: "cargo",
      args: ["run", "--manifest-path", manifestPath, "-p", "omena-cli", "--quiet", "--"],
    };
  }

  return null;
}

function resolveWorkspaceStyleModulePaths(workspaceRoot) {
  const root = path.resolve(workspaceRoot);
  const paths = [];
  collectWorkspaceStyleModulePaths(root, paths);
  return paths.toSorted();
}

function collectWorkspaceStyleModulePaths(dir, paths) {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (shouldSkipWorkspaceDir(entry.name)) continue;
      collectWorkspaceStyleModulePaths(entryPath, paths);
      continue;
    }
    if (entry.isFile() && STYLE_MODULE_FILE_PATTERN.test(entryPath)) {
      paths.push(entryPath);
    }
  }
}

function shouldSkipWorkspaceDir(name) {
  return new Set([".git", "node_modules", "dist", "build", "coverage", ".next", "target"]).has(
    name,
  );
}

function workspacePathSignature(paths) {
  return paths.map((workspacePath) => {
    try {
      const stat = fs.statSync(workspacePath);
      return `${workspacePath}:${stat.size}:${stat.mtimeMs}`;
    } catch {
      return `${workspacePath}:missing`;
    }
  });
}

function resolveWorkspaceRoot(filePath, configuredRoot) {
  if (configuredRoot) return path.resolve(configuredRoot);
  let current = path.dirname(filePath);
  while (true) {
    if (
      fs.existsSync(path.join(current, "tsconfig.json")) ||
      fs.existsSync(path.join(current, "package.json"))
    ) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) return path.dirname(filePath);
    current = parent;
  }
}

function toEslintLoc(range) {
  return {
    start: {
      line: range.start.line + 1,
      column: range.start.character,
    },
    end: {
      line: range.end.line + 1,
      column: range.end.character,
    },
  };
}
