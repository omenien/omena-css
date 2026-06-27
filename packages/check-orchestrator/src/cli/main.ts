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
import type { CheckGate, CheckTargetRef } from "../manifest/index";
import { pnpmRunCommand } from "./commands";

interface ParsedArgs {
  readonly command: string;
  readonly target: string | null;
  readonly dryRun: boolean;
  readonly json: boolean;
  readonly check: boolean;
  readonly write: boolean;
  readonly summary: boolean;
  readonly extraArgs: readonly string[];
}

const parsedArgs = parseArgs(process.argv.slice(2));
const manifest = loadCheckManifest();

// Dispatch is invoked at the END of this module (see bottom), after every const
// declaration is initialized — the summary renderers reference module-scope consts.

function parseArgs(argv: readonly string[]): ParsedArgs {
  const separatorIndex = argv.indexOf("--");
  const visibleArgs = separatorIndex === -1 ? argv : argv.slice(0, separatorIndex);
  const rawExtraArgs = separatorIndex === -1 ? [] : argv.slice(separatorIndex + 1);
  const forwardedDryRun = rawExtraArgs.some((arg) => arg === "--dry" || arg === "--dry-run");
  const extraArgs = rawExtraArgs.filter((arg) => arg !== "--dry" && arg !== "--dry-run");
  const flags = new Set(visibleArgs.filter((arg) => arg.startsWith("-")));
  const positionals = visibleArgs.filter((arg) => !arg.startsWith("-"));

  return {
    command: positionals[0] ?? "help",
    target: positionals[1] ?? null,
    dryRun: flags.has("--dry") || flags.has("--dry-run") || forwardedDryRun,
    json: flags.has("--json"),
    check: flags.has("--check"),
    write: flags.has("--write"),
    summary: flags.has("--summary"),
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
            referencedTargetSpecs,
            ciTier,
            ciGroup,
            ciReason,
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
            referencedTargetSpecs,
            ciTier,
            ciGroup,
            ciReason,
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

  if (parsed.summary) {
    runWithSummary(gate, parsed.extraArgs);
  }

  process.exit(executeGate(gate, parsed.extraArgs, new Set<string>()));
}

interface CheckResult {
  readonly status: "pass" | "fail";
  readonly title: string;
  readonly durationMs: number;
  readonly output: string;
}

const useColor = Boolean(process.stdout.isTTY || process.env.FORCE_COLOR) && !process.env.NO_COLOR;
const paint = (code: number, text: string): string => (useColor ? `[${code}m${text}[0m` : text);
const green = (text: string): string => paint(32, text);
const red = (text: string): string => paint(31, text);
const dim = (text: string): string => paint(2, text);
const bold = (text: string): string => paint(1, text);

function formatDuration(ms: number): string {
  return ms >= 1000 ? `${(ms / 1000).toFixed(1)}s` : `${ms}ms`;
}

// `run --summary`: run EVERY member of a bundle (never early-return), capturing each
// member's outcome, then render an aggregated table + (in CI) GitHub annotations, and
// exit non-zero iff any member failed. The default `run` path is untouched (live stdio).
function runWithSummary(gate: CheckGate, extraArgs: readonly string[]): never {
  const isCI = Boolean(process.env.GITHUB_ACTIONS || process.env.CI);
  const isGitHub = Boolean(process.env.GITHUB_ACTIONS);
  const memberSpecs = getReferencedTargetSpecs(gate);
  const members =
    (gate.kind === "bundle" || gate.kind === "alias") && memberSpecs.length > 0
      ? memberSpecs
      : [{ target: gate.id } as CheckTargetRef];

  const childEnv: NodeJS.ProcessEnv = { ...process.env };
  if ((process.stdout.isTTY || isCI) && !childEnv.FORCE_COLOR) {
    childEnv.FORCE_COLOR = "1";
    childEnv.CARGO_TERM_COLOR ??= "always";
  }

  const results: CheckResult[] = [];
  for (const targetSpec of members) {
    const member = resolveTarget(targetSpec.target);
    const memberArgs = getDepExtraArgs(gate, targetSpec.args ?? [], extraArgs);
    const commands = renderGateCommands(member, memberArgs);
    const start = performance.now();
    let status: "pass" | "fail" = "pass";
    let output = "";
    for (const command of commands) {
      const run = spawnSync(command.executable, command.args, {
        cwd: manifest.rootDir,
        shell: false,
        encoding: "utf8",
        env: childEnv,
        maxBuffer: 256 * 1024 * 1024,
      });
      output += (run.stdout ?? "") + (run.stderr ?? "");
      if (run.error) {
        output += `\nFailed to start "${command.display[0]}": ${run.error.message}\n`;
        status = "fail";
        break;
      }
      if ((run.status ?? 1) !== 0) {
        status = "fail";
        break;
      }
    }
    const result: CheckResult = {
      status,
      title: member.id,
      durationMs: Math.round(performance.now() - start),
      output,
    };
    results.push(result);
    emitGateOutput(result, isGitHub);
  }

  renderSummaryTable(gate.id, results);
  writeSummaryArtifact(gate.id, results);
  const failed = results.filter((result) => result.status === "fail");
  if (isGitHub) {
    let annotated = 0;
    for (const result of failed) {
      if (annotated >= 10) {
        console.log(
          `::warning::${failed.length - annotated} more gate failure(s) omitted from annotations (GitHub caps 10/step); see the summary table + artifact.`,
        );
        break;
      }
      console.log(`::error title=${result.title}::Gate "${result.title}" failed.`);
      annotated += 1;
    }
  }
  process.exit(failed.length > 0 ? 1 : 0);
}

function emitGateOutput(result: CheckResult, isGitHub: boolean): void {
  const icon = result.status === "pass" ? green("✔") : red("✖");
  const line = `${icon} ${result.title} ${dim(formatDuration(result.durationMs))}`;
  if (isGitHub) {
    const open = result.status === "pass" ? "::group::" : "::group::✖ ";
    console.log(`${open}${result.title} (${formatDuration(result.durationMs)})`);
    if (result.output.trim().length > 0) {
      process.stdout.write(result.output.endsWith("\n") ? result.output : `${result.output}\n`);
    }
    console.log("::endgroup::");
    if (result.status === "fail") {
      console.log(red(`✖ ${result.title} failed`));
    }
    return;
  }
  console.log(line);
  if (result.status === "fail" && result.output.trim().length > 0) {
    const tail = result.output.trimEnd().split("\n").slice(-40).join("\n");
    process.stdout.write(`${tail}\n`);
  }
}

function renderSummaryTable(bundleId: string, results: readonly CheckResult[]): void {
  const passed = results.filter((result) => result.status === "pass").length;
  const failed = results.length - passed;
  const total = results.reduce((sum, result) => sum + result.durationMs, 0);
  const width = Math.max(4, ...results.map((result) => result.title.length));
  console.log("");
  console.log(bold(`Summary: ${bundleId}`));
  console.log(dim("-".repeat(width + 18)));
  for (const result of results) {
    const icon = result.status === "pass" ? green("PASS") : red("FAIL");
    console.log(
      `${icon}  ${result.title.padEnd(width)}  ${dim(formatDuration(result.durationMs))}`,
    );
  }
  console.log(dim("-".repeat(width + 18)));
  const tally = `${green(`${passed} passed`)}${failed > 0 ? `, ${red(`${failed} failed`)}` : ""}`;
  console.log(`${tally}  ${dim(`(${results.length} gates, ${formatDuration(total)})`)}`);
}

function writeSummaryArtifact(bundleId: string, results: readonly CheckResult[]): void {
  const artifactPath = process.env.OMENA_CHECK_SUMMARY_JSON;
  if (!artifactPath) {
    return;
  }
  const payload = {
    bundle: bundleId,
    results: results.map(({ status, title, durationMs }) => ({ status, title, durationMs })),
  };
  writeFileSync(artifactPath, `${JSON.stringify(payload, null, 2)}\n`);
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
  for (const targetSpec of getReferencedTargetSpecs(gate)) {
    const status = executeGate(
      resolveTarget(targetSpec.target),
      getDepExtraArgs(gate, targetSpec.args ?? [], extraArgs),
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

  return getReferencedTargetSpecs(gate).flatMap((targetSpec) =>
    renderGateCommands(
      resolveTarget(targetSpec.target),
      getDepExtraArgs(gate, targetSpec.args ?? [], extraArgs),
    ),
  );
}

function getReferencedTargetSpecs(gate: CheckGate): readonly CheckTargetRef[] {
  return (
    gate.referencedTargetSpecs ??
    gate.referencedTargets?.map((target) => ({
      target,
    })) ??
    []
  );
}

function getDepExtraArgs(
  gate: CheckGate,
  targetArgs: readonly string[],
  extraArgs: readonly string[],
): readonly string[] {
  return gate.kind === "alias" ? [...targetArgs, ...extraArgs] : targetArgs;
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

function dispatch(): void {
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
}

dispatch();
