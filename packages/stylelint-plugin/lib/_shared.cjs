const path = require("node:path");
const { execFileSync } = require("node:child_process");

const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../../../");
const STYLE_CHECK_REPORT_CACHE = new Map();

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
      const findings = runStyleChecks(filePath, ruleOptions, [code]);

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

function runStyleChecks(filePath, ruleOptions, includeCodes = ["unused-selector"]) {
  const report = readStyleCheckReport(ruleOptions);
  const includeCodeSet = new Set(includeCodes);
  return (report.findings ?? []).filter(
    (finding) => finding.filePath === filePath && includeCodeSet.has(finding.code),
  );
}

function readStyleCheckReport(ruleOptions) {
  const cacheKey = JSON.stringify({
    workspaceRoot: ruleOptions.workspaceRoot,
    classnameTransform: ruleOptions.classnameTransform,
    pathAlias: Object.entries(ruleOptions.pathAlias).toSorted(([a], [b]) => a.localeCompare(b)),
  });
  const cached = STYLE_CHECK_REPORT_CACHE.get(cacheKey);
  if (cached) return cached;

  const args = [
    "--silent",
    "check:workspace",
    "--",
    ruleOptions.workspaceRoot,
    "--category",
    "style",
    "--severity",
    "all",
    "--format",
    "json",
    "--fail-on",
    "none",
    "--classname-transform",
    ruleOptions.classnameTransform,
  ];

  for (const [key, value] of Object.entries(ruleOptions.pathAlias)) {
    args.push("--path-alias", `${key}=${value}`);
  }

  const stdout = execFileSync("pnpm", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  const report = JSON.parse(stdout);
  STYLE_CHECK_REPORT_CACHE.set(cacheKey, report);
  return report;
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
