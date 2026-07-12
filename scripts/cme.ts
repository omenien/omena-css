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
    runRustCli(argv);
    return;
  }

  if (command === "explain" && maybeSubcommand === "expression") {
    runScript("explain-expression.ts", rest);
    return;
  }

  if (command === "rename") {
    runScript("rename-selector.ts", argv.slice(1));
    return;
  }

  runRustCli(argv);
}

function runRustCli(argv: readonly string[]): void {
  const manifestPath = path.join(scriptDir, "..", "rust", "Cargo.toml");
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      manifestPath,
      "-p",
      "omena-cli",
      "--bin",
      "omena",
      "--",
      ...argv,
    ],
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
