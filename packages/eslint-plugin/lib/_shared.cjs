const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { execFileSync } = require("node:child_process");

const SOURCE_FILE_PATTERN = /\.[cm]?[jt]sx?$/;
const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|sass|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../../../");
const MAX_CACHED_WORKSPACE_SESSIONS = 8;
const DIRECT_SOURCE_DIAGNOSTICS_CACHE = new Map();
const WORKSPACE_SESSION_CACHE = new Map();
let resolvedNapiBinding;
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
  workspaceSessionBackendReport,
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
  const styleSnapshot = readWorkspaceStyleSnapshot(workspaceStylePaths);
  const configContentDigest = resolveConfigContentDigest(ruleOptions.workspaceRoot);
  const napiBinding = resolveOmenaNapiBinding();
  const cacheKey = JSON.stringify({
    filePath: context.filename,
    sourceText: context.sourceCode.text,
    includeCodes: [...includeCodes].toSorted(),
    workspaceStyleSnapshot: styleSnapshot.digest,
    configContentDigest,
    backend: napiBinding ? "napi-session" : (process.env.OMENA_CLI_BIN ?? "cargo-cli"),
  });
  const cached = DIRECT_SOURCE_DIAGNOSTICS_CACHE.get(cacheKey);
  if (cached) return cached;

  const report = napiBinding
    ? readOmenaNapiSourceDiagnostics(
        napiBinding,
        context.filename,
        context.sourceCode.text,
        ruleOptions,
        styleSnapshot,
        configContentDigest,
      )
    : readOmenaCliSourceDiagnostics(context.filename, ruleOptions, workspaceStylePaths);
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

function readOmenaNapiSourceDiagnostics(
  binding,
  filePath,
  source,
  ruleOptions,
  styleSnapshot,
  configContentDigest,
) {
  const cacheKey = `${path.resolve(ruleOptions.workspaceRoot)}\0${configContentDigest}`;
  let cached = WORKSPACE_SESSION_CACHE.get(cacheKey);
  if (!cached) {
    cached = {
      session: new binding.CachedWorkspace(
        ruleOptions.workspaceRoot,
        configContentDigest,
        JSON.stringify(styleSnapshot.sources),
      ),
      styleDigest: styleSnapshot.digest,
    };
    WORKSPACE_SESSION_CACHE.set(cacheKey, cached);
    evictOldWorkspaceSessions();
  } else if (cached.styleDigest !== styleSnapshot.digest) {
    cached.session.replaceStyleSourcesJson(JSON.stringify(styleSnapshot.sources));
    cached.styleDigest = styleSnapshot.digest;
  }
  return JSON.parse(cached.session.sourceDiagnosticsJson(filePath, source, "[]"));
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

function resolveOmenaNapiBinding() {
  if (resolvedNapiBinding !== undefined) return resolvedNapiBinding;
  if (process.env.OMENA_DISABLE_NAPI_SESSION === "1") {
    resolvedNapiBinding = null;
    return null;
  }
  const candidates = [
    process.env.OMENA_NAPI_BINDING,
    path.join(REPO_ROOT, "rust/crates/omena-napi/pkg"),
    "@omena/napi",
  ].filter(Boolean);
  for (const candidate of candidates) {
    try {
      const binding = require(candidate);
      if (
        typeof binding.CachedWorkspace === "function" &&
        typeof binding.workspaceSessionCacheReportJson === "function"
      ) {
        resolvedNapiBinding = binding;
        return binding;
      }
    } catch {
      // Native residency is optional; the direct CLI path remains authoritative.
    }
  }
  resolvedNapiBinding = null;
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

function readWorkspaceStyleSnapshot(paths) {
  const digest = crypto.createHash("sha256");
  const sources = paths.map((stylePath) => {
    const styleSource = fs.readFileSync(stylePath, "utf8");
    digest.update(stylePath);
    digest.update("\0");
    digest.update(styleSource);
    digest.update("\0");
    return { stylePath, styleSource };
  });
  return { sources, digest: digest.digest("hex") };
}

function evictOldWorkspaceSessions() {
  while (WORKSPACE_SESSION_CACHE.size > MAX_CACHED_WORKSPACE_SESSIONS) {
    const oldestKey = WORKSPACE_SESSION_CACHE.keys().next().value;
    if (oldestKey === undefined) return;
    WORKSPACE_SESSION_CACHE.delete(oldestKey);
  }
}

function resolveConfigContentDigest(workspaceRoot) {
  if (process.env.OMENA_CONFIG_CONTENT_DIGEST) {
    return process.env.OMENA_CONFIG_CONTENT_DIGEST;
  }
  const inputs = collectConfigSnapshotInputs(workspaceRoot);
  const digest = crypto.createHash("sha256");
  digest.update(path.resolve(workspaceRoot));
  for (const inputPath of inputs) {
    digest.update("\0");
    digest.update(inputPath);
    digest.update("\0");
    digest.update(fs.readFileSync(inputPath));
  }
  return digest.digest("hex");
}

function collectConfigSnapshotInputs(workspaceRoot) {
  const inputs = [];
  let current = path.resolve(workspaceRoot);
  while (true) {
    for (const fileName of [
      "omena.toml",
      "omena.config.toml",
      "omena.config.json",
      ".editorconfig",
    ]) {
      const candidate = path.join(current, fileName);
      if (fs.existsSync(candidate)) inputs.push(candidate);
    }
    if (
      fs.existsSync(path.join(current, ".git")) ||
      fs.existsSync(path.join(current, "pnpm-workspace.yaml"))
    ) {
      break;
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return inputs.toSorted();
}

function workspaceSessionBackendReport(workspaceRoot) {
  const binding = resolveOmenaNapiBinding();
  return {
    route: binding ? "napiSession" : "directCli",
    configContentDigest: resolveConfigContentDigest(workspaceRoot),
    javascriptSessionCount: WORKSPACE_SESSION_CACHE.size,
    nativeCache: binding ? JSON.parse(binding.workspaceSessionCacheReportJson()) : null,
  };
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
