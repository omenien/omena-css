import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { getAllStyleExtensions } from "../server/engine-core-ts/src/core/scss/lang-registry";
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  resolveSelectedQueryBackendKind,
  runRustSelectedQueryBackendJson,
} from "../server/engine-host-node/src/selected-query-backend";

interface ParsedArgs {
  readonly workspaceRoot: string;
  readonly selectorName: string;
  readonly newName: string;
  readonly targetStyleUri?: string;
  readonly dryRun: boolean;
  readonly json: boolean;
}

interface RenamePlanSummary {
  readonly schemaVersion: "0";
  readonly product: "omena-query.rename-plan";
  readonly selectorName: string;
  readonly newName: string;
  readonly targetStyleUri?: string;
  readonly editCount: number;
  readonly edits: readonly {
    readonly uri: string;
    readonly range: {
      readonly start: { readonly line: number; readonly character: number };
      readonly end: { readonly line: number; readonly character: number };
    };
    readonly newText: string;
  }[];
  readonly readySurfaces: readonly string[];
}

interface RenameDryRunOutput extends RenamePlanSummary {
  readonly consumer: "cme.rename.selector";
  readonly analysisSource: "omena-query";
  readonly dryRun: true;
}

const STYLE_EXTENSIONS = new Set(getAllStyleExtensions());
const SOURCE_EXTENSIONS = new Set([".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"]);
const SKIPPED_DIRS = new Set([
  ".git",
  ".next",
  ".turbo",
  "coverage",
  "dist",
  "node_modules",
  "out",
  "rust/target",
  "target",
]);

void main(process.argv.slice(2));

function main(argv: readonly string[]): void {
  const parsed = parseArgs(argv, process.cwd());
  if ("error" in parsed) {
    process.stderr.write(`${parsed.error}\n`);
    process.stderr.write(buildHelpText());
    process.exitCode = 2;
    return;
  }
  if ("helpText" in parsed) {
    process.stdout.write(parsed.helpText);
    return;
  }

  const backend = resolveSelectedQueryBackendKind(process.env);
  if (backend !== "rust-selected-query") {
    process.stderr.write(
      `cme rename selector requires OMENA_SELECTED_QUERY_BACKEND=rust-selected-query; resolved ${backend}.\n`,
    );
    process.exitCode = 1;
    return;
  }

  const workspace = collectWorkspaceSources(parsed.workspaceRoot);
  const summary = runRustSelectedQueryBackendJson<RenamePlanSummary>(
    SELECTED_QUERY_RUNNER_COMMANDS.renamePlan,
    {
      selectorName: parsed.selectorName,
      newName: parsed.newName,
      ...(parsed.targetStyleUri ? { targetStyleUri: parsed.targetStyleUri } : {}),
      styles: workspace.styles,
      sourceDocuments: workspace.sourceDocuments,
      packageManifests: workspace.packageManifests,
    },
  );
  if (summary.product !== "omena-query.rename-plan") {
    throw new Error(`Unexpected rename-plan product: ${summary.product}`);
  }

  const output: RenameDryRunOutput = {
    ...summary,
    consumer: "cme.rename.selector",
    analysisSource: "omena-query",
    dryRun: true,
  };

  if (parsed.json) {
    process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
    return;
  }
  process.stdout.write(formatRenameDryRun(output, parsed.workspaceRoot));
}

function parseArgs(
  argv: readonly string[],
  cwd: string,
): ParsedArgs | { readonly error: string } | { readonly helpText: string } {
  if (argv[0] === "--") return parseArgs(argv.slice(1), cwd);
  if (argv.length === 0 || argv[0] === "--help" || argv[0] === "-h") {
    return { helpText: buildHelpText() };
  }
  if (argv[0] !== "selector") {
    return { error: `Unknown rename command: ${argv[0]}` };
  }

  let workspaceRoot = cwd;
  let targetStylePath: string | undefined;
  let dryRun = false;
  let json = false;
  const positional: string[] = [];

  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index]!;
    if (arg === "--") continue;
    if (arg === "--help" || arg === "-h") return { helpText: buildHelpText() };
    if (arg === "--json") {
      json = true;
      continue;
    }
    if (arg === "--dry-run") {
      dryRun = true;
      continue;
    }
    if (arg === "--root") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --root" };
      workspaceRoot = path.resolve(cwd, value);
      index += 1;
      continue;
    }
    if (arg === "--target-style") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --target-style" };
      targetStylePath = value;
      index += 1;
      continue;
    }
    if (arg.startsWith("--")) return { error: `Unknown option: ${arg}` };
    positional.push(arg);
  }

  if (positional.length !== 2) {
    return { error: "Expected selector name and new name." };
  }
  if (!dryRun) {
    return { error: "Only --dry-run is supported for rename selector." };
  }

  const root = path.resolve(cwd, workspaceRoot);
  const targetStyleUri = targetStylePath
    ? normalizeTargetStylePath(targetStylePath, root, cwd)
    : undefined;

  return {
    workspaceRoot: root,
    selectorName: normalizeSelectorName(positional[0]!),
    newName: normalizeSelectorName(positional[1]!),
    ...(targetStyleUri ? { targetStyleUri } : {}),
    dryRun,
    json,
  };
}

function collectWorkspaceSources(workspaceRoot: string): {
  readonly styles: readonly { readonly stylePath: string; readonly styleSource: string }[];
  readonly sourceDocuments: readonly {
    readonly sourcePath: string;
    readonly sourceSource: string;
  }[];
  readonly packageManifests: readonly {
    readonly packageJsonPath: string;
    readonly packageJsonSource: string;
  }[];
} {
  if (!existsSync(workspaceRoot)) {
    throw new Error(`Workspace root does not exist: ${workspaceRoot}`);
  }

  const files = listWorkspaceFiles(workspaceRoot);
  return {
    styles: files
      .filter((filePath) => isStyleModulePath(filePath))
      .map((stylePath) => ({
        stylePath,
        styleSource: readFileSync(stylePath, "utf8"),
      })),
    sourceDocuments: files
      .filter((filePath) => SOURCE_EXTENSIONS.has(path.extname(filePath)))
      .map((sourcePath) => ({
        sourcePath,
        sourceSource: readFileSync(sourcePath, "utf8"),
      })),
    packageManifests: files
      .filter((filePath) => path.basename(filePath) === "package.json")
      .map((packageJsonPath) => ({
        packageJsonPath,
        packageJsonSource: readFileSync(packageJsonPath, "utf8"),
      })),
  };
}

function listWorkspaceFiles(root: string): readonly string[] {
  const files: string[] = [];
  const walk = (dirPath: string): void => {
    for (const entry of readdirSync(dirPath)) {
      const entryPath = path.join(dirPath, entry);
      const relativePath = path.relative(root, entryPath);
      const stat = statSync(entryPath);
      if (stat.isDirectory()) {
        if (SKIPPED_DIRS.has(entry) || SKIPPED_DIRS.has(relativePath)) continue;
        walk(entryPath);
        continue;
      }
      if (stat.isFile()) files.push(entryPath);
    }
  };
  walk(root);
  files.sort();
  return files;
}

function isStyleModulePath(filePath: string): boolean {
  for (const extension of STYLE_EXTENSIONS) {
    if (filePath.endsWith(extension)) return true;
  }
  return false;
}

function normalizeSelectorName(selectorName: string): string {
  return selectorName.trim().replace(/^\./u, "");
}

function normalizeTargetStylePath(value: string, workspaceRoot: string, cwd: string): string {
  if (value.startsWith("file://")) return fileURLToPath(value);
  if (path.isAbsolute(value)) return path.normalize(value);
  const cwdResolved = path.resolve(cwd, value);
  if (cwdResolved.startsWith(`${workspaceRoot}${path.sep}`) || cwdResolved === workspaceRoot) {
    return cwdResolved;
  }
  return path.resolve(workspaceRoot, value);
}

function formatRenameDryRun(output: RenameDryRunOutput, workspaceRoot: string): string {
  const lines = [
    `Consumer: ${output.consumer}`,
    `Analysis source: ${output.analysisSource}`,
    `Product: ${output.product}`,
    `Selector: ${output.selectorName} -> ${output.newName}`,
    `Dry run: ${String(output.dryRun)}`,
    `Edits: ${output.editCount}`,
  ];
  for (const edit of output.edits) {
    lines.push(
      `- ${relativeOrAbsolute(edit.uri, workspaceRoot)}:${edit.range.start.line + 1}:${
        edit.range.start.character + 1
      } -> ${JSON.stringify(edit.newText)}`,
    );
  }
  return `${lines.join("\n")}\n`;
}

function relativeOrAbsolute(filePath: string, workspaceRoot: string): string {
  const relativePath = path.relative(workspaceRoot, filePath);
  if (!relativePath || relativePath.startsWith("..")) return filePath;
  return relativePath;
}

function buildHelpText(): string {
  return [
    "Usage: pnpm cme rename selector <selector> <new-name> --dry-run [options]",
    "",
    "Options:",
    "  --root <path>          Workspace root (defaults to cwd)",
    "  --target-style <path>  Restrict edits to one CSS Module target",
    "  --dry-run             Print the planned workspace edits without writing files",
    "  --json                Emit JSON instead of text",
    "  --help, -h            Show this help text",
    "",
  ].join("\n");
}
