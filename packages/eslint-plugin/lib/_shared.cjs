const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { execFileSync } = require("node:child_process");
const fastGlob = require("fast-glob");
const {
  buildStyleFileWatcherGlob,
  findLangForPath,
} = require("../../../server/engine-core-ts/dist/core/scss/lang-registry.js");
const {
  checkSourceDocument,
} = require("../../../server/engine-core-ts/dist/core/checker/check-source-document.js");
const {
  formatCheckerFinding: formatLegacyCheckerFinding,
} = require("../../../server/engine-core-ts/dist/checker-surface/format-checker-finding.js");
const {
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
} = require("../../../server/engine-host-node/dist/checker-host/workspace-check-support.js");

const DEFAULT_IGNORES = ["**/node_modules/**", "**/dist/**", "**/.git/**"];
const SOURCE_FILE_PATTERN = /\.[cm]?[jt]sx?$/;
const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|sass|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../../../");
const HOST_CACHE = new Map();
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
  formatCheckerFinding,
  getRuleOptions,
  getWorkspaceHost,
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
  const directFindings = readDirectSourceDiagnostics(context, ruleOptions);
  if (directFindings) return directFindings;

  const host = getWorkspaceHost(ruleOptions);
  return checkSourceDocument(
    {
      documentUri: pathToFileURL(context.filename).href,
      content: context.sourceCode.text,
      filePath: context.filename,
      version: 1,
    },
    {
      analysisCache: host.analysisHost.analysisCache,
      styleDocumentForPath: host.styleHost.styleDocumentForPath,
      typeResolver: host.analysisHost.typeResolver,
      workspaceRoot: ruleOptions.workspaceRoot,
    },
    {
      includeMissingModule: ruleOptions.includeMissingModule,
    },
  );
}

function formatCheckerFinding(finding, workspaceRoot) {
  if (typeof finding.message === "string" && finding.message.length > 0) {
    return finding.message;
  }
  return formatLegacyCheckerFinding(finding, workspaceRoot);
}

function readDirectSourceDiagnostics(context, ruleOptions) {
  const includeCodes = ruleOptions.includeCodes;
  if (!canUseDirectSourceDiagnostics(includeCodes)) return null;

  const workspaceStylePaths = resolveWorkspaceStyleModulePaths(ruleOptions.workspaceRoot);
  const cacheKey = JSON.stringify({
    filePath: context.filename,
    sourceText: context.sourceCode.text,
    includeCodes: [...includeCodes].toSorted(),
    workspaceStylePaths: workspacePathSignature(workspaceStylePaths),
    backend: process.env.OMENA_ESLINT_QUERY_BACKEND ?? null,
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

function canUseDirectSourceDiagnostics(includeCodes) {
  if (!Array.isArray(includeCodes) || includeCodes.length === 0) return false;
  if (!includeCodes.every((code) => DIRECT_SOURCE_DIAGNOSTIC_CODES.has(code))) return false;
  if (process.env.OMENA_ESLINT_QUERY_BACKEND === "legacy") return false;
  if (process.env.OMENA_ESLINT_QUERY_BACKEND === "omena-cli") return true;
  return Boolean(process.env.OMENA_CLI_BIN);
}

function readOmenaCliSourceDiagnostics(filePath, ruleOptions, workspaceStylePaths) {
  const invocation = resolveOmenaCliInvocation();
  if (!invocation) return null;

  try {
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
    return JSON.parse(stdout);
  } catch (error) {
    if (process.env.OMENA_ESLINT_QUERY_BACKEND === "omena-cli") {
      throw error;
    }
    return null;
  }
}

function resolveOmenaCliInvocation() {
  if (process.env.OMENA_CLI_BIN) {
    return { command: process.env.OMENA_CLI_BIN, args: [] };
  }

  const manifestPath = path.join(REPO_ROOT, "rust/Cargo.toml");
  if (process.env.OMENA_ESLINT_QUERY_BACKEND === "omena-cli" && fs.existsSync(manifestPath)) {
    return {
      command: "cargo",
      args: ["run", "--manifest-path", manifestPath, "-p", "omena-cli", "--quiet", "--"],
    };
  }

  return null;
}

function getWorkspaceHost({ workspaceRoot, classnameTransform, pathAlias }) {
  const cacheKey = JSON.stringify({
    workspaceRoot,
    classnameTransform,
    pathAlias: Object.entries(pathAlias).toSorted(([a], [b]) => a.localeCompare(b)),
  });
  const cached = HOST_CACHE.get(cacheKey);
  if (cached) return cached;

  const styleFiles = fastGlob
    .sync(buildStyleFileWatcherGlob(), {
      cwd: workspaceRoot,
      absolute: true,
      onlyFiles: true,
      followSymbolicLinks: false,
      ignore: DEFAULT_IGNORES,
    })
    .filter((filePath) => findLangForPath(filePath) !== null)
    .toSorted();

  const styleHost = createWorkspaceStyleHost({
    styleFiles,
    classnameTransform,
  });
  styleHost.preloadStyleDocuments();
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot,
    classnameTransform,
    pathAlias,
    styleDocumentForPath: styleHost.styleDocumentForPath,
  });

  const host = { styleHost, analysisHost };
  HOST_CACHE.set(cacheKey, host);
  return host;
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
