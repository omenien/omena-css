import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import {
  buildCheckPlan,
  buildCheckSurfaceReport,
  loadCheckManifest,
  renderCheckInventory,
  renderCheckPlan,
  renderCheckSurfaceReport,
  resolveGateTarget,
  runDoctor,
} from "../manifest/index";
import type { CheckGate } from "../manifest/index";
import { pnpmRunCommand } from "./commands";

interface ParsedArgs {
  readonly command: string;
  readonly target: string | null;
  readonly dryRun: boolean;
  readonly json: boolean;
  readonly check: boolean;
  readonly write: boolean;
  readonly extraArgs: readonly string[];
}

const parsedArgs = parseArgs(process.argv.slice(2));
const manifest = loadCheckManifest();

switch (parsedArgs.command) {
  case "list":
    printList(parsedArgs.json);
    break;
  case "run":
    runTarget(parsedArgs, false);
    break;
  case "bundle":
    runTarget(parsedArgs, true);
    break;
  case "plan":
    printPlan(parsedArgs);
    break;
  case "doctor":
    runDoctorCommand(parsedArgs.json);
    break;
  case "surface":
    printSurface(parsedArgs.json);
    break;
  case "inventory":
    runInventoryCommand(parsedArgs);
    break;
  case "help":
  case "--help":
  case "-h":
    printHelp();
    break;
  default:
    fail(`Unknown command "${parsedArgs.command}". Run "pnpm omena-check help".`);
}

function parseArgs(argv: readonly string[]): ParsedArgs {
  const separatorIndex = argv.indexOf("--");
  const visibleArgs = separatorIndex === -1 ? argv : argv.slice(0, separatorIndex);
  const extraArgs = separatorIndex === -1 ? [] : argv.slice(separatorIndex + 1);
  const flags = new Set(visibleArgs.filter((arg) => arg.startsWith("-")));
  const positionals = visibleArgs.filter((arg) => !arg.startsWith("-"));

  return {
    command: positionals[0] ?? "help",
    target: positionals[1] ?? null,
    dryRun: flags.has("--dry") || flags.has("--dry-run"),
    json: flags.has("--json"),
    check: flags.has("--check"),
    write: flags.has("--write"),
    extraArgs,
  };
}

function printList(json: boolean): void {
  if (json) {
    console.log(
      JSON.stringify(
        manifest.gates.map(
          ({
            id,
            scriptName,
            scope,
            kind,
            origin,
            referencedScripts,
            referencedTargets,
            ciTier,
            ciGroup,
            deprecatedAliases,
            deprecatedBy,
          }) => ({
            id,
            scriptName,
            scope,
            kind,
            origin,
            referencedScripts,
            referencedTargets,
            ciTier,
            ciGroup,
            deprecatedAliases,
            deprecatedBy,
          }),
        ),
        null,
        2,
      ),
    );
    return;
  }

  const rows = manifest.gates.map((gate) => [
    gate.id.padEnd(48),
    gate.kind.padEnd(7),
    gate.origin.padEnd(8),
    gate.scope.padEnd(9),
    gate.scriptName,
  ]);
  console.log("id".padEnd(48), "kind".padEnd(7), "origin".padEnd(8), "scope".padEnd(9), "script");
  console.log("-".repeat(102));
  for (const row of rows) {
    console.log(row.join("  "));
  }
}

function runTarget(parsed: ParsedArgs, bundleOnly: boolean): void {
  if (!parsed.target) {
    fail(`Missing target. Run "pnpm omena-check ${parsed.command} <id-or-script>".`);
  }

  const gate = resolveTarget(parsed.target);
  if (bundleOnly && gate.kind !== "bundle" && gate.kind !== "alias") {
    fail(`Target "${parsed.target}" is not a bundle. Use "pnpm omena-check run ${gate.id}".`);
  }

  if (parsed.dryRun) {
    console.log(renderGateCommands(gate, parsed.extraArgs).map(formatCommandDisplay).join("\n"));
    return;
  }

  process.exit(executeGate(gate, parsed.extraArgs, new Set<string>()));
}

function printPlan(parsed: ParsedArgs): void {
  if (!parsed.target) {
    fail('Missing target. Run "pnpm omena-check plan <id-or-script>".');
  }

  const plan = buildCheckPlan(manifest, resolveTarget(parsed.target));
  if (parsed.json) {
    console.log(JSON.stringify(plan, null, 2));
    return;
  }

  console.log(renderCheckPlan(plan));
}

function resolveTarget(target: string) {
  const gate = resolveGateTarget(manifest, target);
  if (!gate) {
    fail(`Unknown target "${target}". Run "pnpm omena-check list".`);
  }
  return gate;
}

interface RunnableCommand {
  readonly executable: string;
  readonly args: readonly string[];
  readonly display: readonly string[];
}

function executeGate(gate: CheckGate, extraArgs: readonly string[], stack: Set<string>): number {
  if (stack.has(gate.id)) {
    fail(`Declared gate dependency cycle reached "${gate.id}".`);
  }

  const commands = gate.commandParts
    ? [directCommand(gate.commandParts, extraArgs)]
    : gate.origin === "declared"
      ? []
      : [pnpmRunCommand(gate.scriptName, extraArgs)];

  if (commands.length > 0) {
    const command = commands[0];
    if (!command) {
      fail(`Gate "${gate.id}" produced no runnable command.`);
    }
    const result = spawnSync(command.executable, command.args, {
      cwd: manifest.rootDir,
      stdio: "inherit",
      shell: false,
    });
    if (result.error) {
      console.error(`Failed to start "${command.display[0]}": ${result.error.message}`);
    }
    return result.status ?? 1;
  }

  if (!gate.referencedTargets || gate.referencedTargets.length === 0) {
    fail(`Declared gate "${gate.id}" has no command or deps to execute.`);
  }

  if (gate.kind !== "alias" && extraArgs.length > 0) {
    fail(
      `Extra args can only be forwarded through declared commands or aliases, not "${gate.id}".`,
    );
  }

  stack.add(gate.id);
  for (const target of gate.referencedTargets) {
    const status = executeGate(
      resolveTarget(target),
      gate.kind === "alias" ? extraArgs : [],
      stack,
    );
    if (status !== 0) {
      stack.delete(gate.id);
      return status;
    }
  }
  stack.delete(gate.id);
  return 0;
}

function renderGateCommands(
  gate: CheckGate,
  extraArgs: readonly string[],
): readonly RunnableCommand[] {
  if (gate.commandParts) {
    return [directCommand(gate.commandParts, extraArgs)];
  }

  if (gate.origin !== "declared") {
    return [pnpmRunCommand(gate.scriptName, extraArgs)];
  }

  if (gate.kind !== "alias" && extraArgs.length > 0) {
    fail(
      `Extra args can only be forwarded through declared commands or aliases, not "${gate.id}".`,
    );
  }

  return (gate.referencedTargets ?? []).flatMap((target) =>
    renderGateCommands(resolveTarget(target), gate.kind === "alias" ? extraArgs : []),
  );
}

function directCommand(
  commandParts: readonly string[],
  extraArgs: readonly string[],
): RunnableCommand {
  const [executable, ...args] = commandParts;
  if (!executable) {
    fail("Declared command has no executable.");
  }
  return {
    executable,
    args: [...args, ...extraArgs],
    display: [executable, ...args, ...extraArgs],
  };
}

function formatCommandDisplay(command: RunnableCommand): string {
  return command.display.join(" ");
}

function runDoctorCommand(json: boolean): void {
  const diagnostics = runDoctor(manifest);
  const errorCount = diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
  const warningCount = diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length;

  if (json) {
    console.log(JSON.stringify({ errorCount, warningCount, diagnostics }, null, 2));
  } else if (diagnostics.length === 0) {
    console.log(`check-orchestrator doctor: ok (${manifest.gates.length} scripts mirrored)`);
  } else {
    for (const diagnostic of diagnostics) {
      console.log(`${diagnostic.severity}: ${diagnostic.code}: ${diagnostic.message}`);
    }
  }

  process.exit(errorCount === 0 ? 0 : 1);
}

function printSurface(json: boolean): void {
  const report = buildCheckSurfaceReport(manifest);
  if (json) {
    console.log(JSON.stringify(report, null, 2));
    return;
  }

  console.log(renderCheckSurfaceReport(report));
}

function runInventoryCommand(parsed: ParsedArgs): void {
  if (parsed.check && parsed.write) {
    fail("Use either --check or --write, not both.");
  }

  const inventory = renderCheckInventory(manifest);
  const inventoryPath = path.join(manifest.rootDir, "packages/check-orchestrator/CHECKS.md");

  if (parsed.write) {
    writeFileSync(inventoryPath, `${inventory}\n`);
    return;
  }

  if (parsed.check) {
    const current = existsSync(inventoryPath) ? readFileSync(inventoryPath, "utf8") : "";
    if (current !== `${inventory}\n`) {
      fail("Check inventory is out of date. Run `pnpm omena-check inventory --write`.");
    }
    console.log("check-orchestrator inventory: ok");
    return;
  }

  console.log(inventory);
}

function printHelp(): void {
  console.log(`Usage:
  pnpm omena-check list [--json]
  pnpm omena-check run <id-or-script> [--dry] [-- extra args]
  pnpm omena-check bundle <id-or-script> [--dry] [-- extra args]
  pnpm omena-check plan <id-or-script> [--json]
  pnpm omena-check doctor [--json]
  pnpm omena-check surface [--json]
  pnpm omena-check inventory [--check|--write]
`);
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}
