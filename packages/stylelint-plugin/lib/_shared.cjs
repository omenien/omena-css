const path = require("node:path");
const fs = require("node:fs");
const { execFileSync } = require("node:child_process");

const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../../../");
const DIRECT_STYLE_DIAGNOSTICS_CACHE = new Map();
const DIRECT_STYLE_DIAGNOSTIC_CODES = new Set([
  "missing-custom-property",
  "missing-keyframes",
  "missing-sass-symbol",
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "unused-selector",
]);
const OMENA_QUERY_STYLE_DIAGNOSTIC_CODE_MAP = new Map([
  ["missingCustomProperty", "missing-custom-property"],
  ["missingKeyframes", "missing-keyframes"],
  ["missingSassSymbol", "missing-sass-symbol"],
  ["missingComposedModule", "missing-composed-module"],
  ["missingComposedSelector", "missing-composed-selector"],
  ["missingValueModule", "missing-value-module"],
  ["missingImportedValue", "missing-imported-value"],
  ["unusedSelector", "unused-selector"],
]);

module.exports = {
  STYLE_MODULE_FILE_PATTERN,
  createFindingRule,
  getRuleOptions,
  offsetForRangePosition,
  runStyleChecks,
};

function createFindingRule({ stylelint, ruleName, code, possible = [true] }) {
  const ruleFunction = (primary, secondaryOptions = {}) => {
    return (root, result) => {
      const valid = stylelint.utils.validateOptions(result, ruleName, {
        actual: primary,
        possible,
      });
      if (!valid) return;

      const filePath = root.source?.input?.file;
      if (!filePath || !STYLE_MODULE_FILE_PATTERN.test(filePath)) return;
      const sourceText = root.source?.input?.css ?? root.toString();

      const ruleOptions = getRuleOptions(filePath, secondaryOptions);
      const findings = runStyleChecks(filePath, ruleOptions, [code], sourceText);

      for (const finding of findings) {
        stylelint.utils.report({
          result,
          ruleName,
          message: finding.message,
          node: root,
          index: offsetForRangePosition(sourceText, finding.range.start),
          endIndex: offsetForRangePosition(sourceText, finding.range.end),
        });
      }
    };
  };

  return stylelint.createPlugin(ruleName, ruleFunction);
}

function getRuleOptions(filePath, secondaryOptions = {}) {
  return {
    workspaceRoot: resolveWorkspaceRoot(filePath, secondaryOptions.workspaceRoot),
    classnameTransform: secondaryOptions.classnameTransform ?? "asIs",
    pathAlias: secondaryOptions.pathAlias ?? {},
  };
}

function runStyleChecks(
  filePath,
  ruleOptions,
  includeCodes = ["unused-selector"],
  sourceText = "",
) {
  return readDirectStyleDiagnostics(filePath, ruleOptions, includeCodes, sourceText);
}

function readDirectStyleDiagnostics(filePath, ruleOptions, includeCodes, sourceText) {
  if (!canUseDirectStyleDiagnostics(includeCodes)) return [];
  const workspaceStylePaths = resolveWorkspaceStyleModulePaths(ruleOptions.workspaceRoot, filePath);
  const workspaceSourceDocumentPaths = resolveWorkspaceSourceDocumentPaths(
    ruleOptions.workspaceRoot,
  );

  const cacheKey = JSON.stringify({
    filePath,
    sourceText,
    includeCodes: [...includeCodes].toSorted(),
    workspaceStylePaths: workspaceStylePathSignature(workspaceStylePaths),
    workspaceSourceDocumentPaths: workspaceStylePathSignature(workspaceSourceDocumentPaths),
    cli: process.env.OMENA_CLI_BIN ?? null,
  });
  const cached = DIRECT_STYLE_DIAGNOSTICS_CACHE.get(cacheKey);
  if (cached) return cached;

  const report = readOmenaCliStyleDiagnostics(
    filePath,
    ruleOptions,
    workspaceStylePaths,
    workspaceSourceDocumentPaths,
  );

  const includeCodeSet = new Set(includeCodes);
  const findings = (report.diagnostics ?? [])
    .map((diagnostic) => {
      const code = OMENA_QUERY_STYLE_DIAGNOSTIC_CODE_MAP.get(diagnostic.code);
      if (!code) return null;
      return {
        filePath,
        code,
        category: "style",
        severity: "warning",
        range: diagnostic.range,
        message: diagnostic.message,
      };
    })
    .filter((finding) => finding && includeCodeSet.has(finding.code));

  DIRECT_STYLE_DIAGNOSTICS_CACHE.set(cacheKey, findings);
  return findings;
}

function canUseDirectStyleDiagnostics(includeCodes) {
  if (!includeCodes.every((code) => DIRECT_STYLE_DIAGNOSTIC_CODES.has(code))) return false;
  return true;
}

function readOmenaCliStyleDiagnostics(
  filePath,
  ruleOptions,
  workspaceStylePaths,
  workspaceSourceDocumentPaths,
) {
  const invocation = resolveOmenaCliInvocation();
  if (!invocation) {
    throw new Error("Unable to find omena CLI. Set OMENA_CLI_BIN to a built omena-cli binary.");
  }

  const args = [...invocation.args, "style-diagnostics", filePath, "--json"];
  for (const sourcePath of workspaceStylePaths) {
    args.push("--source", sourcePath);
  }
  for (const sourceDocumentPath of workspaceSourceDocumentPaths) {
    args.push("--source-document", sourceDocumentPath);
  }
  const stdout = execFileSync(invocation.command, args, {
    cwd: ruleOptions.workspaceRoot,
    encoding: "utf8",
    env: process.env,
  });
  return unwrapOmenaCliResponse(JSON.parse(stdout), "omena-cli.style-diagnostics");
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

function resolveWorkspaceStyleModulePaths(workspaceRoot, targetFilePath) {
  const root = path.resolve(workspaceRoot);
  const target = path.resolve(targetFilePath);
  const paths = [];
  collectWorkspaceStyleModulePaths(root, paths);
  return paths.filter((candidate) => candidate !== target).toSorted();
}

function resolveWorkspaceSourceDocumentPaths(workspaceRoot) {
  const root = path.resolve(workspaceRoot);
  const paths = [];
  collectWorkspaceSourceDocumentPaths(root, paths);
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

function collectWorkspaceSourceDocumentPaths(dir, paths) {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (shouldSkipWorkspaceDir(entry.name)) continue;
      collectWorkspaceSourceDocumentPaths(entryPath, paths);
      continue;
    }
    if (entry.isFile() && /\.(c|m)?(jsx?|tsx?)$/.test(entryPath)) {
      paths.push(entryPath);
    }
  }
}

function shouldSkipWorkspaceDir(name) {
  return new Set([".git", "node_modules", "dist", "build", "coverage", ".next", "target"]).has(
    name,
  );
}

function workspaceStylePathSignature(paths) {
  return paths.map((stylePath) => {
    try {
      const stat = fs.statSync(stylePath);
      return `${stylePath}:${stat.size}:${stat.mtimeMs}`;
    } catch {
      return `${stylePath}:missing`;
    }
  });
}

function resolveWorkspaceRoot(filePath, configuredRoot) {
  if (configuredRoot) return path.resolve(configuredRoot);
  return path.dirname(filePath);
}

function offsetForRangePosition(sourceText, position) {
  let line = 0;
  let offset = 0;

  while (line < position.line && offset < sourceText.length) {
    const nextNewline = sourceText.indexOf("\n", offset);
    if (nextNewline === -1) return sourceText.length;
    offset = nextNewline + 1;
    line += 1;
  }

  return Math.min(offset + position.character, sourceText.length);
}
