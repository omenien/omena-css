import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));

void main(process.argv.slice(2));

function main(argv: readonly string[]): void {
  if (argv[0] === "--") {
    main(argv.slice(1));
    return;
  }

  const [command, maybeSubcommand, ...rest] = argv;

  if (!command || command === "--help" || command === "-h") {
    process.stdout.write(buildHelpText());
    return;
  }

  if (command === "explain") {
    const explainArgs = maybeSubcommand === "expression" ? rest : argv.slice(1);
    runScript("explain-expression.ts", explainArgs);
    return;
  }

  process.stderr.write(`Unknown cme command: ${command}\n`);
  process.stderr.write(buildHelpText());
  process.exitCode = 2;
}

function runScript(scriptName: string, argv: readonly string[]): void {
  const result = spawnSync(
    process.execPath,
    ["--import", "tsx", path.join(scriptDir, scriptName), ...argv],
    {
      cwd: process.cwd(),
      env: process.env,
      stdio: "inherit",
    },
  );

  if (result.error) {
    process.stderr.write(`${result.error.message}\n`);
    process.exitCode = 1;
    return;
  }

  process.exitCode = result.status ?? 1;
}

function buildHelpText(): string {
  return [
    "Usage:",
    "  pnpm cme explain <file>:<line>:<column> [options]",
    "  pnpm cme explain expression <file>:<line>:<column> [options]",
    "",
    "Commands:",
    "  explain             Explain a source class expression and its value provenance",
    "  explain expression  Alias for explain",
    "",
    "Options for explain:",
    "  --root <path>       Workspace root (defaults to cwd)",
    "  --json              Emit JSON instead of text",
    "  --help, -h          Show command help",
    "",
  ].join("\n");
}
