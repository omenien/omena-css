import { resolveScanSurfaceForScanner } from "../evidence/scan-surface-manifest";
import {
  buildEvidenceScanSurfaceManifest,
  evidenceScanSurfaceFreshnessDiagnostics,
} from "../evidence/scan-surface-registry";
import {
  evidenceScanSurfaceManifestPath,
  renderEvidenceScanSurfaceManifest,
} from "../evidence/scan-surface-manifest";
import {
  assertEvidenceWriterNonLiteralWriteCensusDecreaseOnly,
  buildEvidenceWriterNonLiteralWriteCensus,
  buildEvidenceWriterRegistry,
  evidenceWriterNonLiteralWriteCensusPath,
  evidenceWriterRegistryPath,
  loadEvidenceWriterNonLiteralWriteCensus,
  loadEvidenceWriterRegistry,
  renderEvidenceWriterNonLiteralWriteCensus,
  renderEvidenceWriterRegistry,
} from "../evidence/writer-registry";
import { buildEvidenceAffectedPlan } from "../evidence/affected-evidence";
import { resolveEvidenceWriterCommand, runDigestPinnedWriter } from "../evidence/writer-runner";
import { loadEvidenceScanSurfaceManifest } from "../evidence/scan-surface-manifest";
import { resolveScanSurface } from "../evidence/scan-surface";
import { detectScannerFiles } from "../evidence/scanner-analysis";
import {
  EVIDENCE_PREVIEW_EMPTY_BUDGET_MS,
  runEvidenceAffectedPreview,
} from "../evidence/preview-budget";
import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { buildAffectedCheckPlan } from "../affected";
import {
  adoptAndWriteRegistry,
  checkCiWorkflow,
  resolveCiWorkflowVerdict,
  writeCiWorkflow,
} from "../manifest/ci-workflow";
import {
  computeCostLedgerDigest,
  costLedgerPath,
  costLedgerToday,
  findCostLedgerDiagnostics,
  loadCostLedger,
  packMeasuredPartition,
  percentile,
} from "../manifest/cost-ledger";
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
import { bundleShardNames, resolveShardMembers } from "../manifest/shards";
import { CI_PROBE_PROFILES, resolveCiProbeProfile } from "../probes";
import { pnpmRunCommand } from "./commands";
import { runCapturedCommand, runLiveCommand, type CapturedCommandResult } from "./execution";
import { resolveSummaryMemberArgs } from "./summary-args";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

interface ParsedArgs {
  readonly command: string;
  readonly target: string | null;
  readonly dryRun: boolean;
  readonly json: boolean;
  readonly check: boolean;
  readonly write: boolean;
  readonly summary: boolean;
  readonly evidence: boolean;
  readonly preview: boolean;
  readonly shard: string | null;
  readonly base: string;
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
  const shardFlag = visibleArgs.find((arg) => arg.startsWith("--shard="));
  const baseFlag = visibleArgs.find((arg) => arg.startsWith("--base="));

  return {
    command: positionals[0] ?? "help",
    target: positionals[1] ?? null,
    dryRun: flags.has("--dry") || flags.has("--dry-run") || forwardedDryRun,
    json: flags.has("--json"),
    check: flags.has("--check"),
    write: flags.has("--write"),
    summary: flags.has("--summary"),
    evidence: flags.has("--evidence"),
    preview: flags.has("--preview"),
    shard: shardFlag ? shardFlag.slice("--shard=".length) : null,
    base: baseFlag ? baseFlag.slice("--base=".length) : "origin/master",
    extraArgs,
  };
}

async function runProbeCommand(parsed: ParsedArgs): Promise<void> {
  if (!parsed.target) {
    const profiles = CI_PROBE_PROFILES.map(({ id, target, description, platforms }) => ({
      id,
      target,
      description,
      platforms,
    }));
    if (parsed.json) {
      console.log(JSON.stringify(profiles, null, 2));
      return;
    }
    for (const profile of profiles) {
      console.log(
        `${profile.id.padEnd(20)} ${profile.description} [${profile.platforms.join(", ")}]`,
      );
    }
    return;
  }

  const profile = resolveCiProbeProfile(parsed.target);
  if (!profile) {
    fail(`Unknown CI probe profile "${parsed.target}". Run "pnpm omena-check probe".`);
  }
  if (!profile.platforms.includes(process.platform)) {
    fail(
      `CI probe profile "${profile.id}" requires ${profile.platforms.join(", ")}; current platform is ${process.platform}.`,
    );
  }

  const gate = resolveTarget(profile.target);
  if (parsed.dryRun) {
    console.log(renderGateCommands(gate, []).map(formatCommandDisplay).join("\n"));
    return;
  }
  process.exit(await executeGate(gate, [], new Set<string>(), new Set<string>()));
}

async function printAffectedChecks(parsed: ParsedArgs): Promise<void> {
  const changedPaths = collectChangedPaths(parsed.base);
  if (parsed.evidence) {
    await printAffectedEvidence(parsed, changedPaths);
    return;
  }
  const plan = buildAffectedCheckPlan(changedPaths);
  if (parsed.json) {
    console.log(JSON.stringify({ base: parsed.base, ...plan }, null, 2));
    return;
  }

  console.log(`Affected checks against ${parsed.base}:`);
  if (plan.changedPaths.length === 0) {
    console.log("  no changed paths");
    return;
  }
  for (const profile of plan.profiles) {
    console.log(`  probe: ${profile}`);
  }
  console.log(
    `  final full CI: ${plan.requiresFullCi ? "required" : "still required at merge boundary"}`,
  );
  for (const entry of plan.reasons) {
    const fallback = entry.requiresFullCi ? " [full-ci]" : "";
    console.log(`  ${entry.path}: ${entry.reason}${fallback}`);
  }
}

async function printAffectedEvidence(
  parsed: ParsedArgs,
  changedPaths: readonly string[],
): Promise<void> {
  if (parsed.check && parsed.write) fail("Use either --check or --write, not both.");
  if (parsed.write && parsed.preview) {
    fail("Evidence --write and --preview are separate modes; run them as separate commands.");
  }
  const scanManifest = loadEvidenceScanSurfaceManifest(manifest.rootDir);
  const writerRegistry = loadEvidenceWriterRegistry(manifest.rootDir);
  if (!scanManifest) {
    fail(
      "Evidence scan surface manifest is absent; run `pnpm omena-check evidence-surfaces --write`.",
    );
  }
  if (!writerRegistry) {
    fail("Evidence writer registry is absent; run `pnpm omena-check evidence-writers --write`.");
  }
  const freshnessDiagnostics = evidenceScanSurfaceFreshnessDiagnostics(
    manifest.rootDir,
    scanManifest,
  );
  if (freshnessDiagnostics.length > 0) {
    fail(
      `Evidence scan surface manifest is stale: ${freshnessDiagnostics[0]}. Run ` +
        "`pnpm omena-check evidence-surfaces --write`.",
    );
  }
  const detectorRow = scanManifest.scanners.find(
    (row) => row.scannerPath === "packages/check-orchestrator/src/evidence/scanner-analysis.ts",
  );
  if (!detectorRow || detectorRow.disposition !== "MIGRATED") {
    fail("Evidence scanner universe has no migrated detector surface.");
  }
  const expectedScannerPaths = detectScannerFiles(manifest.rootDir, detectorRow.spec).map(
    (entry) => entry.scannerPath,
  );
  const plan = buildEvidenceAffectedPlan({
    changedPaths,
    scanManifest,
    writerRegistry,
    expectedScannerPaths,
    knownGateIds: manifest.gates.map((gate) => gate.id),
  });
  if (parsed.write)
    await runAffectedEvidenceWriters(plan.writerOrder, scanManifest, writerRegistry);
  let preview: Awaited<ReturnType<typeof runEvidenceAffectedPreview>> | undefined;
  if (parsed.preview) {
    if (process.env.OMENA_EVIDENCE_SWEEP === "0") {
      console.log(
        "evidence affected preview: skipped by OMENA_EVIDENCE_SWEEP=0 (LEFTHOOK=0 remains the whole-hook override)",
      );
      return;
    }
    const ledger = loadCostLedger(manifest.rootDir);
    if (!ledger) fail("ci-cost-ledger.json is absent; evidence preview cannot price gates.");
    preview = await runEvidenceAffectedPreview({
      plan,
      manifest,
      ledger,
      executeGate: async (gateId) =>
        executeGate(
          resolveTarget(gateId),
          [],
          new Set<string>(),
          new Set<string>(),
          null,
          parsed.json,
        ),
    });
    if (changedPaths.length === 0 && performance.now() > EVIDENCE_PREVIEW_EMPTY_BUDGET_MS) {
      fail(
        `empty evidence preview exceeded ${EVIDENCE_PREVIEW_EMPTY_BUDGET_MS}ms: ${Math.round(performance.now())}ms`,
      );
    }
  }
  if (parsed.json) {
    console.log(
      JSON.stringify(
        {
          base: parsed.base,
          mode: parsed.write ? "write" : "check",
          ...plan,
          ...(preview ? { preview } : {}),
        },
        null,
        2,
      ),
    );
    return;
  }
  console.log(`Affected evidence against ${parsed.base} (${parsed.write ? "write" : "check"}):`);
  console.log(
    `  refresh sufficiency: ${plan.evidenceRefreshSufficientAlone ? "narrow evidence refresh" : "INSUFFICIENT-ALONE"}`,
  );
  if (plan.fallbackToFullEvidence) {
    for (const reason of plan.fallbackReasons) console.log(`  FULL evidence fallback: ${reason}`);
  }
  for (const artifactPath of plan.writerOrder) console.log(`  writer-order: ${artifactPath}`);
  for (const row of plan.notRefreshed) {
    const instruction = Array.isArray(row.instruction)
      ? row.instruction.join(" -> ")
      : row.instruction;
    console.log(`  NOT-REFRESHED ${row.classification}: ${row.artifactPath}: ${instruction}`);
  }
  for (const entry of plan.notPreviewableInputs) {
    console.log(
      `  NOT-PREVIEWABLE ${entry.kind}: ${entry.ownerId}: gates=${entry.gateIds.join(",")}: ${entry.detail}`,
    );
  }
  for (const entry of plan.commitThenRefresh) {
    console.log(`  COMMIT-THEN-REFRESH ${entry.artifactPath}: ${entry.steps.join(" -> ")}`);
  }
  if (preview) {
    console.log(
      `  PREVIEW ran estimate=${preview.budget.estimatedRunMs}ms measured=${preview.receipts.reduce((sum, receipt) => sum + receipt.elapsedMs, 0)}ms`,
    );
    console.log(
      `  PREVIEW skipped priced=${preview.budget.pricedSkippedMs}ms unbounded=${preview.budget.unboundedSkippedCount}`,
    );
    for (const entry of preview.budget.skipped) {
      console.log(
        `  PREVIEW-SKIPPED ${entry.gateId} p95=${entry.p95Ms === null ? "unbounded" : `${entry.p95Ms}ms`}`,
      );
    }
    for (const gateId of preview.budget.notPreviewableSkippedGateIds) {
      console.log(`  NOT-PREVIEWABLE-SKIPPED ${gateId}`);
    }
    for (const gateId of preview.budget.omittedWriteModeGateIds) {
      console.log(`  PREVIEW-OMITTED-WRITE-MODE ${gateId}`);
    }
  }
  console.log(
    `  scanners affected=${plan.affectedScannerPaths.length} excluded=${plan.excludedScannerPaths.length} gates=${plan.gateIds.length}`,
  );
}

async function runAffectedEvidenceWriters(
  writerOrder: readonly string[],
  scanManifest: NonNullable<ReturnType<typeof loadEvidenceScanSurfaceManifest>>,
  writerRegistry: NonNullable<ReturnType<typeof loadEvidenceWriterRegistry>>,
): Promise<void> {
  const artifactByPath = new Map(writerRegistry.artifacts.map((row) => [row.artifactPath, row]));
  const scannerByPath = new Map(scanManifest.scanners.map((row) => [row.scannerPath, row]));
  const completedCommands = new Set<string>();
  for (const artifactPath of writerOrder) {
    const row = artifactByPath.get(artifactPath);
    if (!row || row.classification === "W3" || row.classification === "W4") continue;
    if (!row.writeCommand) throw new Error(`writeable artifact lacks a command: ${artifactPath}`);
    const rowCommandKeys = [row.writeCommand, ...(row.alternateWriteCommands ?? [])].map(
      (command) => JSON.stringify(command),
    );
    if (rowCommandKeys.some((candidate) => completedCommands.has(candidate))) continue;
    const commandKey = JSON.stringify(row.writeCommand);
    const commandRows = writerRegistry.artifacts.filter((candidate) =>
      [candidate.writeCommand, ...(candidate.alternateWriteCommands ?? [])]
        .filter((command): command is readonly string[] => command !== undefined)
        .some((command) => JSON.stringify(command) === commandKey),
    );
    const requiredEnvironmentKeys = [
      ...new Set(commandRows.flatMap((candidate) => candidate.requiredEnvironmentKeys ?? [])),
    ].toSorted();
    const resolvedWriteCommand = resolveEvidenceWriterCommand(
      row.writeCommand,
      requiredEnvironmentKeys,
    );
    const inputPaths = new Set<string>(["package.json", "pnpm-lock.yaml"]);
    for (const commandRow of commandRows) {
      for (const writerScript of commandRow.writerScripts) inputPaths.add(writerScript);
      for (const inputPath of commandRow.inputPaths) inputPaths.add(inputPath);
      for (const inputArtifactPath of commandRow.inputArtifactPaths) {
        if (inputArtifactPath !== commandRow.artifactPath) inputPaths.add(inputArtifactPath);
      }
      for (const scannerPath of commandRow.inputScannerPaths) {
        inputPaths.add(scannerPath);
        const scanner = scannerByPath.get(scannerPath);
        if (scanner?.disposition === "MIGRATED") {
          for (const inputPath of resolveScanSurface(scanner.spec, { repoRoot: manifest.rootDir })
            .paths) {
            if (!commandRows.some((candidate) => candidate.artifactPath === inputPath)) {
              inputPaths.add(inputPath);
            }
          }
        }
      }
    }
    await runDigestPinnedWriter({
      repoRoot: manifest.rootDir,
      command: resolvedWriteCommand,
      inputPaths: [...inputPaths],
      outputPaths: commandRows.map((candidate) => candidate.artifactPath),
      requireFreshReproduction: commandRows.some(
        (candidate) =>
          candidate.freshReproductionRequired === true || candidate.classification === "W2",
      ),
      freshReproductionExemptOutputPaths: commandRows
        .filter((candidate) => candidate.freshReproductionExemption?.kind === "reads-own-output")
        .map((candidate) => candidate.artifactPath),
    });
    completedCommands.add(commandKey);
  }
}

function collectChangedPaths(base: string): readonly string[] {
  const paths = new Set<string>();
  for (const args of [
    ["diff", "--name-only", "--diff-filter=ACDMRT", `${base}...HEAD`],
    ["diff", "--name-only", "--diff-filter=ACDMRT"],
    ["diff", "--cached", "--name-only", "--diff-filter=ACDMRT"],
  ]) {
    for (const changedPath of runGitLines(args)) {
      paths.add(changedPath);
    }
  }
  for (const changedPath of evidenceScanSurface
    .gitOutput(["ls-files", "--others", "--exclude-standard"])
    .split(/\r?\n/u)
    .filter(Boolean)) {
    paths.add(changedPath);
  }
  return [...paths].toSorted();
}

function runGitLines(args: readonly string[]): readonly string[] {
  const result = spawnSync("git", args, {
    cwd: manifest.rootDir,
    encoding: "utf8",
    shell: false,
  });
  if (result.error) {
    fail(`Failed to start git: ${result.error.message}`);
  }
  if ((result.status ?? 1) !== 0) {
    fail(String(result.stderr).trim() || `git ${args.join(" ")} failed`);
  }
  return String(result.stdout)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
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
            executor,
            timeoutMinutes,
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
            ...manifest.lifecycleByGateId.get(id),
            scriptName,
            scope,
            kind,
            origin,
            executor,
            timeoutMinutes,
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

async function runTarget(parsed: ParsedArgs, bundleOnly: boolean): Promise<void> {
  if (!parsed.target) {
    fail(`Missing target. Run "pnpm omena-check ${parsed.command} <id-or-script>".`);
  }

  const gate = resolveTarget(parsed.target);
  if (bundleOnly && gate.kind !== "bundle" && gate.kind !== "alias") {
    fail(`Target "${parsed.target}" is not a bundle. Use "pnpm omena-check run ${gate.id}".`);
  }

  if (parsed.shard !== null && !parsed.summary) {
    fail(`--shard=${parsed.shard} requires --summary (shards are a bundle summary-run mode).`);
  }
  if (parsed.shard !== null && gate.kind !== "bundle") {
    fail(`--shard=${parsed.shard} requires a bundle target, got "${gate.id}" (${gate.kind}).`);
  }

  if (parsed.dryRun) {
    console.log(renderGateCommands(gate, parsed.extraArgs).map(formatCommandDisplay).join("\n"));
    return;
  }

  if (parsed.summary) {
    await runWithSummary(gate, parsed.extraArgs, parsed.shard);
    return;
  }

  process.exit(await executeGate(gate, parsed.extraArgs, new Set<string>(), new Set<string>()));
}

interface CheckResult {
  readonly status: "pass" | "fail";
  readonly title: string;
  readonly durationMs: number;
  readonly output: string;
  readonly outputBytes: number;
  readonly outputTruncated: boolean;
  readonly outputPaths: readonly string[];
  readonly timedOut: boolean;
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
async function runWithSummary(
  gate: CheckGate,
  extraArgs: readonly string[],
  shard: string | null = null,
): Promise<never> {
  const summaryStartedAt = performance.now();
  const isCI = Boolean(process.env.GITHUB_ACTIONS || process.env.CI);
  const isGitHub = Boolean(process.env.GITHUB_ACTIONS);
  const memberSpecs = getReferencedTargetSpecs(gate);
  let members =
    (gate.kind === "bundle" || gate.kind === "alias") && memberSpecs.length > 0
      ? memberSpecs
      : [{ target: gate.id } as CheckTargetRef];
  if (shard !== null) {
    const shardMembers = resolveShardMembers(
      gate.id,
      shard,
      memberSpecs.map((spec) => spec.target),
    );
    members = members.filter((spec) => shardMembers.has(spec.target));
    if (members.length !== shardMembers.size) {
      fail(
        `Shard "${shard}" of "${gate.id}" resolved ${shardMembers.size} member(s) but only ${members.length} matched bundle specs.`,
      );
    }
    if (members.length === 0) {
      fail(`Shard "${shard}" of "${gate.id}" resolved zero members.`);
    }
    console.log(
      `Running shard "${shard}" of ${gate.id}: ${members.length}/${memberSpecs.length} gate(s).`,
    );
  }

  const childEnv: NodeJS.ProcessEnv = { ...process.env };
  if ((process.stdout.isTTY || isCI) && !childEnv.FORCE_COLOR) {
    childEnv.FORCE_COLOR = "1";
    childEnv.CARGO_TERM_COLOR ??= "always";
  }

  const results: CheckResult[] = [];
  const maxGateOutputBytes = resolvePositiveIntegerEnv(
    "OMENA_CHECK_SUMMARY_MAX_OUTPUT_BYTES",
    1_048_576,
  );
  // Reuse only dependencies from fully successful members so a failed gate is never
  // hidden by the summary runner's cross-member deduplication.
  const completed = new Set<string>();
  for (const targetSpec of members) {
    const member = resolveTarget(targetSpec.target);
    const memberArgs = resolveSummaryMemberArgs(gate, targetSpec, extraArgs);
    const memberCompleted = new Set(completed);
    const commands = renderGateCommands(member, memberArgs, memberCompleted);
    const start = performance.now();
    let status: "pass" | "fail" = "pass";
    let output = "";
    let outputBytes = 0;
    let outputTruncated = false;
    let timedOut = false;
    const outputPaths: string[] = [];
    const memberDeadlineMs = gateDeadlineMs(null, member.timeoutMinutes);
    const perCommandOutputBytes = Math.max(
      1,
      Math.floor(maxGateOutputBytes / Math.max(1, commands.length)),
    );
    for (const [commandIndex, command] of commands.entries()) {
      const runnableCommand = commandWithDeadline(command, memberDeadlineMs);
      if (!runnableCommand) {
        output += `\nGate timeout expired before command ${commandIndex + 1} started.\n`;
        status = "fail";
        timedOut = true;
        break;
      }
      // Summary mode preserves gate order and per-gate command short-circuiting.
      // oxlint-disable-next-line eslint/no-await-in-loop
      const run = await runCommandWithCapturedOutput(
        runnableCommand,
        childEnv,
        member.id,
        isCI,
        perCommandOutputBytes,
        summaryOutputPath(gate.id, shard, member.id, commandIndex),
      );
      output += run.output;
      outputBytes += run.outputBytes;
      outputTruncated ||= run.outputTruncated;
      timedOut ||= run.timedOut;
      if (run.outputPath) {
        outputPaths.push(path.relative(manifest.rootDir, run.outputPath));
      }
      if (run.error) {
        output += `\nFailed to start "${runnableCommand.display[0]}": ${run.error.message}\n`;
        status = "fail";
        break;
      }
      if (run.timedOut) {
        output += `\nTimed out after ${formatDuration(runnableCommand.timeoutMs ?? 0)}.\n`;
        status = "fail";
        break;
      }
      if ((run.code ?? 1) !== 0) {
        status = "fail";
        break;
      }
    }
    if (status === "pass") {
      for (const executionKey of memberCompleted) {
        completed.add(executionKey);
      }
    }
    const result: CheckResult = {
      status,
      title: member.id,
      durationMs: Math.round(performance.now() - start),
      output,
      outputBytes,
      outputTruncated,
      outputPaths,
      timedOut,
    };
    results.push(result);
    emitGateOutput(result, isGitHub);
  }

  renderSummaryTable(gate.id, results);
  writeSummaryArtifact(gate.id, shard, results, Math.round(performance.now() - summaryStartedAt));
  appendGitHubStepSummary(gate.id, shard, results);
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

type RenderedCommand = ReturnType<typeof renderGateCommands>[number];

function runCommandWithCapturedOutput(
  command: RenderedCommand,
  env: NodeJS.ProcessEnv,
  targetId: string,
  emitHeartbeat: boolean,
  maxOutputBytes: number,
  spillPath: string,
): Promise<CapturedCommandResult> {
  const startedAt = performance.now();
  const heartbeatMs = resolvePositiveIntegerEnv("OMENA_CHECK_SUMMARY_HEARTBEAT_MS", 120_000, true);
  const heartbeat =
    emitHeartbeat && heartbeatMs > 0
      ? setInterval(() => {
          console.log(
            dim(
              `summary gate still running: ${targetId} ${formatDuration(
                performance.now() - startedAt,
              )}`,
            ),
          );
        }, heartbeatMs)
      : null;
  heartbeat?.unref();

  return runCapturedCommand(command, {
    cwd: manifest.rootDir,
    env: { ...env, ...command.envOverrides },
    ...(command.timeoutMs !== undefined ? { timeoutMs: command.timeoutMs } : {}),
    maxOutputBytes,
    spillPath,
  }).finally(() => {
    if (heartbeat) clearInterval(heartbeat);
  });
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
    if (result.outputTruncated) {
      console.log(
        `Output retained as a bounded tail (${result.outputBytes} bytes total); full log: ${result.outputPaths.join(", ") || "not persisted"}`,
      );
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
  if (result.outputTruncated) {
    console.log(
      dim(
        `  output truncated (${result.outputBytes} bytes); ${result.outputPaths.join(", ") || "full log not persisted"}`,
      ),
    );
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

function writeSummaryArtifact(
  bundleId: string,
  shard: string | null,
  results: readonly CheckResult[],
  wallDurationMs: number,
): void {
  const artifactPath =
    process.env.OMENA_CHECK_SUMMARY_JSON ??
    (process.env.GITHUB_ACTIONS
      ? path.join(
          manifest.rootDir,
          ".omena-ci",
          `check-summary-${sanitizeArtifactSegment(bundleId)}${shard ? `-${sanitizeArtifactSegment(shard)}` : ""}.json`,
        )
      : null);
  if (!artifactPath) {
    return;
  }
  mkdirSync(path.dirname(artifactPath), { recursive: true });
  const payload = {
    schemaVersion: "2",
    bundle: bundleId,
    shard,
    runnerPartition: summaryRunnerPartition(),
    generatedAt: new Date().toISOString(),
    gitSha: process.env.GITHUB_SHA ?? null,
    runId: process.env.GITHUB_RUN_ID ?? null,
    wallDurationMs,
    cumulativeGateDurationMs: results.reduce((sum, result) => sum + result.durationMs, 0),
    passedCount: results.filter((result) => result.status === "pass").length,
    failedCount: results.filter((result) => result.status === "fail").length,
    results: results.map(
      ({ status, title, durationMs, outputBytes, outputTruncated, outputPaths, timedOut }) => ({
        status,
        title,
        durationMs,
        outputBytes,
        outputTruncated,
        outputPaths,
        timedOut,
      }),
    ),
  };
  writeFileSync(artifactPath, `${JSON.stringify(payload, null, 2)}\n`);
}

function summaryRunnerPartition(): {
  readonly kind: "identifier-authority-mutation";
  readonly index: number;
  readonly count: number;
} | null {
  const countVariable = "OMENA_IDENTIFIER_AUTHORITY_MUTATION_PARTITION_COUNT";
  const indexVariable = "OMENA_IDENTIFIER_AUTHORITY_MUTATION_PARTITION_INDEX";
  const countText = process.env[countVariable];
  const indexText = process.env[indexVariable];
  if (countText === undefined && indexText === undefined) return null;
  if (countText === undefined || indexText === undefined) {
    throw new Error(`${countVariable} and ${indexVariable} must be paired`);
  }
  const count = Number.parseInt(countText, 10);
  const index = Number.parseInt(indexText, 10);
  if (
    !Number.isSafeInteger(count) ||
    count < 1 ||
    !Number.isSafeInteger(index) ||
    index < 0 ||
    index >= count
  ) {
    throw new Error(
      `invalid identifier-authority mutation runner partition ${indexText}/${countText}`,
    );
  }
  return { kind: "identifier-authority-mutation", index, count };
}

function appendGitHubStepSummary(
  bundleId: string,
  shard: string | null,
  results: readonly CheckResult[],
): void {
  const stepSummaryPath = process.env.GITHUB_STEP_SUMMARY;
  if (!stepSummaryPath) return;

  const passed = results.filter((result) => result.status === "pass").length;
  const rows = results
    .map(
      (result) =>
        `| ${result.status === "pass" ? "PASS" : "FAIL"} | \`${result.title}\` | ${formatDuration(result.durationMs)} |`,
    )
    .join("\n");
  appendFileSync(
    stepSummaryPath,
    [
      `## ${bundleId}${shard ? ` / ${shard}` : ""}`,
      "",
      `Passed ${passed}/${results.length} gates.`,
      "",
      "| Result | Gate | Duration |",
      "| --- | --- | ---: |",
      rows,
      "",
    ].join("\n"),
  );
}

function sanitizeArtifactSegment(value: string): string {
  return value.replaceAll(/[^A-Za-z0-9._-]+/g, "-").replaceAll(/^-+|-+$/g, "");
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

function printShards(parsed: ParsedArgs): void {
  if (!parsed.target) {
    fail('Missing bundle target. Run "pnpm omena-check shards <bundle-id>".');
  }
  const gate = resolveTarget(parsed.target);
  const shardNames = bundleShardNames(gate.id);
  if (shardNames.length === 0) {
    fail(`Bundle "${gate.id}" has no shard table.`);
  }
  if (parsed.json) {
    console.log(JSON.stringify(shardNames));
    return;
  }
  for (const shardName of shardNames) {
    console.log(shardName);
  }
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
  readonly timeoutMs?: number;
  readonly envOverrides?: NodeJS.ProcessEnv;
}

async function executeGate(
  gate: CheckGate,
  extraArgs: readonly string[],
  stack: Set<string>,
  completed: Set<string>,
  parentDeadlineMs: number | null = null,
  captureOutput = false,
): Promise<number> {
  const deadlineMs = gateDeadlineMs(parentDeadlineMs, gate.timeoutMinutes);
  const executionKey = gateExecutionKey(gate, extraArgs);
  if (completed.has(executionKey)) {
    return 0;
  }
  if (stack.has(gate.id)) {
    fail(`Declared gate dependency cycle reached "${gate.id}".`);
  }

  const commands =
    gate.executor === "direct"
      ? directCommandsForGate(gate, extraArgs)
      : gate.executor === "package-script"
        ? [withGateTimeout(pnpmRunCommand(gate.scriptName, extraArgs), gate.timeoutMinutes)]
        : [];

  if (commands.length > 0) {
    for (const baseCommand of commands) {
      const command = commandWithDeadline(baseCommand, deadlineMs);
      if (!command) {
        console.error(`Gate "${gate.id}" exceeded its timeout before the command started.`);
        return 124;
      }
      // Direct sequences preserve shell `&&` semantics without spawning a shell.
      // oxlint-disable-next-line eslint/no-await-in-loop
      const commandOptions = {
        cwd: manifest.rootDir,
        env: { ...process.env, ...command.envOverrides },
        ...(command.timeoutMs !== undefined ? { timeoutMs: command.timeoutMs } : {}),
      };
      const result = captureOutput
        ? await runCapturedCommand(command, { ...commandOptions, maxOutputBytes: 1024 * 1024 })
        : await runLiveCommand(command, commandOptions);
      if (result.error) {
        console.error(`Failed to start "${command.display[0]}": ${result.error.message}`);
      }
      if (result.timedOut) {
        console.error(
          `Gate "${gate.id}" timed out after ${formatDuration(command.timeoutMs ?? 0)}.`,
        );
      }
      const status = result.code ?? 1;
      if (status !== 0) {
        const capturedOutput =
          captureOutput && "output" in result && typeof result.output === "string"
            ? result.output
            : "";
        if (capturedOutput.trim().length > 0) {
          process.stderr.write(
            capturedOutput.endsWith("\n") ? capturedOutput : `${capturedOutput}\n`,
          );
        }
        return status;
      }
    }
    completed.add(executionKey);
    return 0;
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
    // Dependency order is part of the gate contract and failures short-circuit the remaining deps.
    // oxlint-disable-next-line eslint/no-await-in-loop
    const status = await executeGate(
      resolveTarget(targetSpec.target),
      getDepExtraArgs(gate, targetSpec.args ?? [], extraArgs),
      stack,
      completed,
      deadlineMs,
      captureOutput,
    );
    if (status !== 0) {
      stack.delete(gate.id);
      return status;
    }
  }
  stack.delete(gate.id);
  completed.add(executionKey);
  return 0;
}

function renderGateCommands(
  gate: CheckGate,
  extraArgs: readonly string[],
  completed: Set<string> = new Set<string>(),
  stack: Set<string> = new Set<string>(),
): readonly RunnableCommand[] {
  const executionKey = gateExecutionKey(gate, extraArgs);
  if (completed.has(executionKey)) {
    return [];
  }
  if (stack.has(gate.id)) {
    fail(`Declared gate dependency cycle reached "${gate.id}".`);
  }

  if (gate.executor === "direct") {
    completed.add(executionKey);
    return directCommandsForGate(gate, extraArgs);
  }

  if (gate.executor === "package-script") {
    completed.add(executionKey);
    return [withGateTimeout(pnpmRunCommand(gate.scriptName, extraArgs), gate.timeoutMinutes)];
  }

  if (gate.kind !== "alias" && extraArgs.length > 0) {
    fail(
      `Extra args can only be forwarded through declared commands or aliases, not "${gate.id}".`,
    );
  }

  stack.add(gate.id);
  const commands = getReferencedTargetSpecs(gate).flatMap((targetSpec) =>
    renderGateCommands(
      resolveTarget(targetSpec.target),
      getDepExtraArgs(gate, targetSpec.args ?? [], extraArgs),
      completed,
      stack,
    ),
  );
  stack.delete(gate.id);
  completed.add(executionKey);
  return commands;
}

function gateExecutionKey(gate: CheckGate, extraArgs: readonly string[]): string {
  return `${gate.id}\0${JSON.stringify(extraArgs)}`;
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
  timeoutMinutes?: number,
  envOverrides?: NodeJS.ProcessEnv,
): RunnableCommand {
  const [executable, ...args] = commandParts;
  if (!executable) {
    fail("Declared command has no executable.");
  }
  return {
    executable,
    args: [...args, ...extraArgs],
    display: [executable, ...args, ...extraArgs],
    ...timeoutFields(timeoutMinutes),
    ...(envOverrides ? { envOverrides } : {}),
  };
}

function directCommandsForGate(
  gate: CheckGate,
  extraArgs: readonly string[],
): readonly RunnableCommand[] {
  const commandSequence = gate.commandSequence ?? (gate.commandParts ? [gate.commandParts] : []);
  const envOverrides =
    gate.origin === "package" || gate.origin === "package+declared"
      ? {
          npm_lifecycle_event: gate.scriptName,
          npm_lifecycle_script: gate.command,
        }
      : undefined;
  return commandSequence.map((commandParts, index) =>
    directCommand(
      commandParts,
      index === commandSequence.length - 1 ? extraArgs : [],
      gate.timeoutMinutes,
      envOverrides,
    ),
  );
}

function withGateTimeout(command: RunnableCommand, timeoutMinutes?: number): RunnableCommand {
  return {
    ...command,
    ...timeoutFields(timeoutMinutes),
  };
}

function timeoutFields(timeoutMinutes?: number): Pick<RunnableCommand, "timeoutMs"> {
  return timeoutMinutes === undefined
    ? {}
    : { timeoutMs: Math.max(1, Math.round(timeoutMinutes * 60_000)) };
}

function gateDeadlineMs(parentDeadlineMs: number | null, timeoutMinutes?: number): number | null {
  const ownDeadlineMs =
    timeoutMinutes === undefined ? null : Date.now() + Math.max(1, timeoutMinutes * 60_000);
  if (parentDeadlineMs === null) return ownDeadlineMs;
  if (ownDeadlineMs === null) return parentDeadlineMs;
  return Math.min(parentDeadlineMs, ownDeadlineMs);
}

function commandWithDeadline(
  command: RunnableCommand,
  deadlineMs: number | null,
): RunnableCommand | null {
  if (deadlineMs === null) return command;
  const remainingMs = Math.floor(deadlineMs - Date.now());
  if (remainingMs <= 0) return null;
  return {
    ...command,
    timeoutMs:
      command.timeoutMs === undefined ? remainingMs : Math.min(command.timeoutMs, remainingMs),
  };
}

function summaryOutputPath(
  bundleId: string,
  shard: string | null,
  memberId: string,
  commandIndex: number,
): string {
  return path.join(
    manifest.rootDir,
    ".omena-ci",
    "check-output",
    `${sanitizeArtifactSegment(bundleId)}${shard ? `-${sanitizeArtifactSegment(shard)}` : ""}`,
    `${sanitizeArtifactSegment(memberId)}-${commandIndex + 1}.log`,
  );
}

function resolvePositiveIntegerEnv(name: string, fallback: number, allowZero = false): number {
  const raw = process.env[name];
  if (raw === undefined) return fallback;
  const value = Number(raw);
  const minimum = allowZero ? 0 : 1;
  if (!Number.isSafeInteger(value) || value < minimum) {
    fail(`${name} must be an integer greater than or equal to ${minimum}, got "${raw}".`);
  }
  return value;
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

function runCiWorkflowCommand(parsed: ParsedArgs): void {
  const rootDir = manifest.rootDir;
  if (parsed.extraArgs.includes("--adopt") || process.argv.includes("--adopt")) {
    const registry = adoptAndWriteRegistry(rootDir);
    process.stdout.write(`ci-workflow registry adopted: ${registry.jobs.length} jobs\n`);
    return;
  }
  if (parsed.write) {
    writeCiWorkflow(rootDir);
    process.stdout.write("ci-workflow: .github/workflows/ci.yml regenerated from the registry\n");
    return;
  }
  const outcome = checkCiWorkflow(rootDir);
  const verdict = resolveCiWorkflowVerdict(outcome, process.env.OMENA_CI_GENERATED_OVERRIDE);
  if (verdict === "ok") {
    process.stdout.write(
      `${JSON.stringify({ product: "omena.check-orchestrator.ci-workflow", drift: "none" })}\n`,
    );
    return;
  }
  if (verdict === "override-warning") {
    process.stdout.write(
      `::warning::ci-workflow drift OVERRIDDEN (break-glass, disclose as an incident): ` +
        `${process.env.OMENA_CI_GENERATED_OVERRIDE} — ${outcome.reason}\n`,
    );
    return;
  }
  process.stderr.write(`ci-workflow drift: ${outcome.reason}\n`);
  process.exitCode = 1;
}

// g131-S1: build the committed cost ledger from real green-run receipts.
// Runs LOCALLY with gh credentials (no CI commit path, no `actions:` grant);
// CI only ever runs the --check side.
function runCostLedgerCommand(parsed: ParsedArgs): void {
  const rootDir = manifest.rootDir;
  if (parsed.write) {
    const reuseSourceRuns = parsed.extraArgs.includes("--reuse-source-runs");
    const reusedLedger = reuseSourceRuns ? loadCostLedger(rootDir) : null;
    if (reuseSourceRuns && (!reusedLedger || reusedLedger.sourceRunIds.length === 0)) {
      fail("cost-ledger --reuse-source-runs requires a committed ledger with sourceRunIds");
    }
    const runLimitArg = parsed.extraArgs.find((arg) => arg.startsWith("--runs="));
    const runLimit = runLimitArg ? Number(runLimitArg.slice("--runs=".length)) : 10;
    const gh = (args: readonly string[]): string => {
      const result = spawnSync("gh", [...args], { cwd: rootDir, encoding: "utf8" });
      if (result.status !== 0) {
        fail(`gh ${args[0]} failed: ${result.stderr?.trim() || result.status}`);
      }
      return result.stdout;
    };
    const sourceRunIds = reusedLedger
      ? [...reusedLedger.sourceRunIds]
      : (
          JSON.parse(
            gh([
              "run",
              "list",
              "--workflow",
              "CI",
              "--branch",
              "master",
              "--status",
              "success",
              "--limit",
              String(runLimit),
              "--json",
              "databaseId",
            ]),
          ) as readonly { databaseId: number }[]
        ).map((run) => String(run.databaseId));
    if (sourceRunIds.length === 0) fail("no successful CI runs found to build the ledger from");

    const jobWall = new Map<string, number[]>();
    const jobQueue = new Map<string, number[]>();
    for (const runId of sourceRunIds) {
      const payload = JSON.parse(
        gh(["api", `repos/{owner}/{repo}/actions/runs/${runId}/jobs?per_page=100`]),
      ) as {
        readonly jobs: readonly {
          readonly name: string;
          readonly conclusion: string | null;
          readonly created_at: string;
          readonly started_at: string;
          readonly completed_at: string | null;
        }[];
      };
      for (const job of payload.jobs) {
        if (job.conclusion !== "success" || !job.completed_at) continue;
        const created = Date.parse(job.created_at);
        const started = Date.parse(job.started_at);
        const completed = Date.parse(job.completed_at);
        // Matrix legs report as "name (leg, ...)" — fold them onto the job.
        const jobName = job.name.replace(/\s*\(.*\)$/u, "");
        if (completed > started) {
          (jobWall.get(jobName) ?? jobWall.set(jobName, []).get(jobName)!).push(
            completed - started,
          );
        }
        if (started > created) {
          (jobQueue.get(jobName) ?? jobQueue.set(jobName, []).get(jobName)!).push(
            started - created,
          );
        }
      }
    }

    const gateSamples = new Map<string, number[]>();
    const downloadRoot = path.join(rootDir, ".omena-ci", "cost-ledger-artifacts");
    if (!reusedLedger) {
      for (const runId of sourceRunIds) {
        const runDir = path.join(downloadRoot, runId);
        mkdirSync(runDir, { recursive: true });
        const download = spawnSync(
          "gh",
          ["run", "download", runId, "--pattern", "*summary*", "--dir", runDir],
          { cwd: rootDir, encoding: "utf8" },
        );
        if (download.status !== 0) continue; // runs predating S0 carry no summary artifacts
        const stack = [runDir];
        while (stack.length > 0) {
          const dir = stack.pop()!;
          for (const entry of evidenceScanSurface.readdirSync(dir, { withFileTypes: true })) {
            const full = path.join(dir, entry.name);
            if (entry.isDirectory()) stack.push(full);
            else if (/^check-summary-.*\.json$/.test(entry.name)) {
              const summary = JSON.parse(readFileSync(full, "utf8")) as {
                readonly results?: readonly {
                  readonly title: string;
                  readonly durationMs: number;
                  readonly status?: string;
                  readonly timedOut?: boolean;
                }[];
              };
              // Only PASSING, non-timed-out rows may feed the p50/p95 samples —
              // job rows are already success-filtered at two levels; gate rows
              // must match (stage-5 lens: a failed-member artifact inside a
              // green-classified run would silently bias the partition input).
              for (const row of (summary.results ?? []).filter(
                (candidate) => candidate.status === "pass" && !candidate.timedOut,
              )) {
                (gateSamples.get(row.title) ?? gateSamples.set(row.title, []).get(row.title)!).push(
                  row.durationMs,
                );
              }
            }
          }
        }
      }
    }

    // Completed-run job metadata remains queryable, but summary artifacts can expire.
    // Reproduction therefore re-derives jobs while retaining the committed, digest-sealed gate samples.
    const ledgerBody = {
      generatedAt: reusedLedger?.generatedAt ?? costLedgerToday(),
      sourceRunIds,
      gates: reusedLedger
        ? [...reusedLedger.gates]
        : [...gateSamples.entries()]
            .toSorted(([left], [right]) => left.localeCompare(right))
            .map(([gateId, samples]) => ({
              gateId,
              p50Ms: percentile(samples, 0.5),
              p95Ms: percentile(samples, 0.95),
              sampleCount: samples.length,
            })),
      jobs: [...jobWall.entries()]
        .toSorted(([left], [right]) => left.localeCompare(right))
        .map(([jobName, wall]) => ({
          jobName,
          wallP50Ms: percentile(wall, 0.5),
          wallP95Ms: percentile(wall, 0.95),
          queueP50Ms: percentile(jobQueue.get(jobName) ?? [], 0.5),
          queueP95Ms: percentile(jobQueue.get(jobName) ?? [], 0.95),
          sampleCount: wall.length,
        })),
    };
    const ledger = {
      schemaVersion: "1",
      product: "omena.check-orchestrator.ci-cost-ledger",
      ...ledgerBody,
      recordsDigest: computeCostLedgerDigest(ledgerBody),
    };
    writeFileSync(costLedgerPath(rootDir), `${JSON.stringify(ledger, null, 2)}\n`);
    process.stdout.write(
      `cost-ledger: ${ledger.gates.length} gate row(s), ${ledger.jobs.length} job row(s) from ${sourceRunIds.length} run(s)\n`,
    );
    return;
  }
  const diagnostics = findCostLedgerDiagnostics(rootDir);
  for (const diagnostic of diagnostics) {
    process.stdout.write(`${diagnostic.severity}: ${diagnostic.code}: ${diagnostic.message}\n`);
  }
  if (diagnostics.some((diagnostic) => diagnostic.severity === "error")) {
    process.exitCode = 1;
    return;
  }
  const ledger = loadCostLedger(rootDir);
  process.stdout.write(
    `${JSON.stringify({
      product: "omena.check-orchestrator.ci-cost-ledger",
      gates: ledger?.gates.length ?? 0,
      jobs: ledger?.jobs.length ?? 0,
      generatedAt: ledger?.generatedAt ?? null,
    })}\n`,
  );
}

// g131-S2: propose a measured partition for a bundle from ledger p95 — the
// committed table in shards.ts stays the authority (small reviewable diff);
// this command computes the pack and REFUSES to propose one that empties the
// rest shard (the hard-fail at shard resolution would otherwise fire in CI).
function runPartitionCommand(parsed: ParsedArgs): void {
  const target = parsed.target;
  if (!target) fail("Usage: pnpm omena-check partition <bundle-id> [--target-minutes=8]");
  const gate = resolveTarget(target);
  if (gate.kind !== "bundle" && gate.kind !== "alias") {
    fail(`Target "${target}" is not a bundle.`);
  }
  const ledger = loadCostLedger(manifest.rootDir);
  if (!ledger) fail("ci-cost-ledger.json is absent; run `pnpm omena-check cost-ledger --write`.");
  const targetArg = parsed.extraArgs.find((arg) => arg.startsWith("--target-minutes="));
  const targetMs = (targetArg ? Number(targetArg.slice("--target-minutes=".length)) : 8) * 60_000;
  const p95ByGate = new Map(ledger.gates.map((row) => [row.gateId, row.p95Ms]));
  const members = (gate.referencedTargets ?? []).map((memberTarget) => {
    const member = resolveTarget(memberTarget);
    return { id: member.id, p95Ms: p95ByGate.get(member.id) ?? null };
  });
  const unmeasured = members.filter((member) => member.p95Ms === null);
  if (unmeasured.length > 0) {
    fail(
      `bundle "${gate.id}" has ${unmeasured.length} member(s) with no ledger p95 ` +
        `(${unmeasured.map((member) => member.id).join(", ")}); a measured partition needs ` +
        "summary receipts for every member — accumulate runs and re-run `cost-ledger --write`.",
    );
  }
  let partition;
  try {
    partition = packMeasuredPartition(
      members.map((member) => ({ id: member.id, p95Ms: member.p95Ms ?? 0 })),
      targetMs,
    );
  } catch (error) {
    fail(`bundle "${gate.id}": ${error instanceof Error ? error.message : String(error)}`);
  }
  const proposal = {
    bundle: gate.id,
    targetMinutes: targetMs / 60_000,
    sourceLedgerGeneratedAt: ledger.generatedAt,
    named: partition.named.map((bin, index) => ({ shard: `pack-${index + 1}`, ...bin })),
    rest: partition.rest,
  };
  process.stdout.write(`${JSON.stringify(proposal, null, 2)}\n`);
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

async function runEvidenceSurfacesCommand(parsed: ParsedArgs): Promise<void> {
  if (parsed.check && parsed.write) {
    fail("Use either --check or --write, not both.");
  }
  const expected = renderEvidenceScanSurfaceManifest(
    await buildEvidenceScanSurfaceManifest(manifest.rootDir),
  );
  const manifestPath = evidenceScanSurfaceManifestPath(manifest.rootDir);
  if (parsed.write) {
    writeFileSync(manifestPath, expected);
    console.log("evidence scan surface manifest: written");
    return;
  }
  if (parsed.check) {
    const current = existsSync(manifestPath) ? readFileSync(manifestPath, "utf8") : "";
    if (current !== expected) {
      fail(
        "Evidence scan surface manifest is out of date. Run `pnpm omena-check evidence-surfaces --write`.",
      );
    }
    console.log("evidence scan surface manifest: ok");
    return;
  }
  const parsedManifest = JSON.parse(expected) as {
    readonly scanners: readonly { readonly disposition: string }[];
  };
  console.log(
    `evidence scan surfaces: ${parsedManifest.scanners.length} scanner row(s), ` +
      `${parsedManifest.scanners.filter((row) => row.disposition === "MIGRATED").length} migrated`,
  );
}

function runEvidenceWritersCommand(parsed: ParsedArgs): void {
  if (parsed.check && parsed.write) {
    fail("Use either --check or --write, not both.");
  }
  const registry = buildEvidenceWriterRegistry(manifest.rootDir);
  const census = buildEvidenceWriterNonLiteralWriteCensus(manifest.rootDir, registry);
  const outputs = [
    {
      label: "evidence writer registry",
      path: evidenceWriterRegistryPath(manifest.rootDir),
      source: renderEvidenceWriterRegistry(registry),
    },
    {
      label: "evidence writer non-literal census",
      path: evidenceWriterNonLiteralWriteCensusPath(manifest.rootDir),
      source: renderEvidenceWriterNonLiteralWriteCensus(census),
    },
  ];
  if (parsed.write) {
    assertEvidenceWriterNonLiteralWriteCensusDecreaseOnly(
      loadEvidenceWriterNonLiteralWriteCensus(manifest.rootDir, false),
      census,
    );
    for (const output of outputs) writeFileSync(output.path, output.source);
    for (const output of outputs) console.log(`${output.label}: written`);
    return;
  }
  if (parsed.check) {
    for (const output of outputs) {
      const current = existsSync(output.path) ? readFileSync(output.path, "utf8") : "";
      if (current !== output.source) {
        fail(`${output.label} is out of date. Run \`pnpm omena-check evidence-writers --write\`.`);
      }
      console.log(`${output.label}: ok`);
    }
    return;
  }
  const counts = Object.groupBy(registry.artifacts, (row) => row.classification);
  console.log(
    `evidence writers: ${registry.artifacts.length} artifact row(s), ` +
      `W1=${counts.W1?.length ?? 0} W2=${counts.W2?.length ?? 0} ` +
      `W3=${counts.W3?.length ?? 0} W4=${counts.W4?.length ?? 0}`,
  );
}

function printHelp(): void {
  console.log(`Usage:
  pnpm omena-check list [--json]
  pnpm omena-check run <id-or-script> [--dry] [-- extra args]
  pnpm omena-check bundle <id-or-script> [--dry] [-- extra args]
  pnpm omena-check plan <id-or-script> [--json]
  pnpm omena-check shards <bundle-id> [--json]
  pnpm omena-check doctor [--json]
  pnpm omena-check surface [--json]
  pnpm omena-check inventory [--check|--write]
  pnpm omena-check evidence-surfaces [--check|--write]
  pnpm omena-check evidence-writers [--check|--write]
  pnpm omena-check probe [profile] [--dry|--json]
  pnpm omena-check affected [--base=<git-ref>] [--json]
  pnpm omena-check affected --evidence [--base=<git-ref>] [--check|--write] [--preview] [--json]
    --write and --preview are separate modes; --preview --json captures child gate output
`);
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}

async function dispatch(): Promise<void> {
  switch (parsedArgs.command) {
    case "list":
      printList(parsedArgs.json);
      break;
    case "run":
      await runTarget(parsedArgs, false);
      break;
    case "bundle":
      await runTarget(parsedArgs, true);
      break;
    case "plan":
      printPlan(parsedArgs);
      break;
    case "shards":
      printShards(parsedArgs);
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
    case "evidence-surfaces":
      await runEvidenceSurfacesCommand(parsedArgs);
      break;
    case "evidence-writers":
      runEvidenceWritersCommand(parsedArgs);
      break;
    case "ci-workflow":
      runCiWorkflowCommand(parsedArgs);
      break;
    case "cost-ledger":
      runCostLedgerCommand(parsedArgs);
      break;
    case "partition":
      runPartitionCommand(parsedArgs);
      break;
    case "probe":
      await runProbeCommand(parsedArgs);
      break;
    case "affected":
      await printAffectedChecks(parsedArgs);
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

dispatch().catch((error: unknown) => {
  fail(error instanceof Error ? error.message : String(error));
});
