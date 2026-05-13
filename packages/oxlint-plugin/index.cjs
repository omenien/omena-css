const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const REPO_ROOT = path.resolve(__dirname, "../..");
const SOURCE_FILE_PATTERN = /\.[cm]?[jt]sx?$/;
const STYLE_MODULE_FILE_PATTERN = /\.module\.(css|scss|sass|less)$/;
const DIAGNOSTIC_RULES = new Map([
  [
    "missing-module",
    {
      queryCode: "missingModule",
      description: "Report missing CSS Module imports.",
    },
  ],
  [
    "missing-static-class",
    {
      queryCode: "missingStaticClass",
      description: "Report missing static CSS Module class references.",
    },
  ],
  [
    "missing-template-prefix",
    {
      queryCode: "missingTemplatePrefix",
      description: "Report template prefixes that do not match any CSS Module selector.",
    },
  ],
  [
    "missing-resolved-class-values",
    {
      queryCode: "missingResolvedClassValues",
      description: "Report dynamic class values that resolve outside the CSS Module selector set.",
    },
  ],
  [
    "missing-resolved-class-domain",
    {
      queryCode: "missingResolvedClassDomain",
      description: "Report dynamic class domains that cannot be proven against the selector set.",
    },
  ],
]);

const plugin = {
  meta: {
    name: "omena-oxlint-plugin",
    version: "0.0.0",
  },
  rules: Object.fromEntries(
    [...DIAGNOSTIC_RULES.entries()].map(([ruleName, rule]) => [
      ruleName,
      createDiagnosticRule(ruleName, rule),
    ]),
  ),
};

module.exports = plugin;

function createDiagnosticRule(ruleName, rule) {
  return {
    meta: {
      type: "problem",
      docs: {
        description: rule.description,
      },
      schema: [
        {
          type: "object",
          additionalProperties: false,
          properties: {
            workspaceRoot: { type: "string" },
            omenaBin: { type: "string" },
            cargoManifestPath: { type: "string" },
          },
        },
      ],
    },
    create(context) {
      const filename = getContextFilename(context);
      if (!filename || filename === "<input>" || !SOURCE_FILE_PATTERN.test(filename)) return {};

      return {
        "Program:exit"() {
          const options = context.options?.[0] ?? {};
          const workspaceRoot = resolveWorkspaceRoot(filename, options.workspaceRoot);
          const report = readOmenaSourceDiagnostics(filename, workspaceRoot, options);
          for (const diagnostic of report.diagnostics ?? []) {
            if (diagnostic.code !== rule.queryCode) continue;
            context.report({
              loc: toLoc(diagnostic.range),
              message: diagnostic.message ?? `${ruleName} reported by omena source diagnostics.`,
            });
          }
        },
      };
    },
  };
}

function readOmenaSourceDiagnostics(filePath, workspaceRoot, options) {
  const invocation = resolveOmenaCliInvocation(options);
  const args = [
    ...invocation.args,
    "source-diagnostics",
    filePath,
    "--source-path",
    filePath,
    "--json",
  ];
  for (const sourcePath of resolveWorkspaceStyleModulePaths(workspaceRoot)) {
    args.push("--source", sourcePath);
  }
  const stdout = execFileSync(invocation.command, args, {
    cwd: workspaceRoot,
    encoding: "utf8",
    env: process.env,
  });
  return JSON.parse(stdout);
}

function resolveOmenaCliInvocation(options) {
  if (options.omenaBin) {
    return { command: options.omenaBin, args: [] };
  }
  if (process.env.CME_OMENA_CLI_BIN) {
    return { command: process.env.CME_OMENA_CLI_BIN, args: [] };
  }
  const manifestPath = options.cargoManifestPath ?? path.join(REPO_ROOT, "rust/Cargo.toml");
  if (fs.existsSync(manifestPath)) {
    return {
      command: "cargo",
      args: ["run", "--manifest-path", manifestPath, "-p", "omena-cli", "--quiet", "--"],
    };
  }
  throw new Error(
    "Unable to find omena CLI. Set omenaBin, CME_OMENA_CLI_BIN, or cargoManifestPath.",
  );
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

function resolveWorkspaceStyleModulePaths(workspaceRoot) {
  const paths = [];
  collectWorkspaceStyleModulePaths(workspaceRoot, paths);
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

function getContextFilename(context) {
  if (typeof context.filename === "string") return context.filename;
  if (typeof context.getFilename === "function") return context.getFilename();
  return null;
}

function toLoc(range) {
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
