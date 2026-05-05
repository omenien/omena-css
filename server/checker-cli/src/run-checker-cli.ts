import { readFileSync } from "node:fs";
import path from "node:path";
import type { CheckerReportV1 } from "../../engine-core-ts/src/contracts";
import type { ClassnameTransformMode } from "../../engine-core-ts/src/core/scss/classname-transform";
import { findLangForPath } from "../../engine-core-ts/src/core/scss/lang-registry";
import {
  runWorkspaceCheckCommand,
  type WorkspaceCheckOptions,
  type WorkspaceCheckResult,
  type WorkspaceCheckCommandCategory,
  type WorkspaceCheckCommandFilters,
  type WorkspaceCheckCommandPreset,
  type WorkspaceCheckCommandSeverity,
} from "../../engine-host-node/src/checker-host";
import {
  expandCheckerCodeBundles,
  isCheckerCodeBundle,
  listCheckerCodeBundles,
  type CheckerCodeBundle,
} from "../../engine-core-ts/src/core/checker/checker-code-bundles";
import { listCheckerRuleDescriptors } from "../../engine-core-ts/src/core/checker/checker-rule-registry";
import { buildCheckerJsonReport, type CheckerReportJsonV1 } from "./checker-report";
import {
  buildRustStyleRecoveryCanonicalProducer,
  type CheckerStyleRecoveryCanonicalProducerSignalV0,
} from "./rust-style-recovery-consumer";
import {
  buildRustSourceMissingCanonicalProducer,
  type CheckerSourceMissingCanonicalProducerSignalV0,
} from "./rust-source-missing-consumer";
import {
  buildRustStyleUnusedCanonicalProducer,
  type CheckerStyleUnusedCanonicalProducerSignalV0,
} from "./rust-style-unused-consumer";
import {
  buildRustFlowAnalysisConsumer,
  type RustFlowAnalysisConsumerV0,
} from "./rust-flow-analysis-consumer";
import type {
  RustSourceMissingConsistencyV0,
  RustStyleUnusedConsistencyV0,
  RustStyleRecoveryConsistencyV0,
} from "./checker-report";

export interface CheckerCliIO {
  readonly stdout: (message: string) => void;
  readonly stderr: (message: string) => void;
  readonly cwd: () => string;
  readonly stdin?: () => Promise<string>;
}

export type CheckerCliFailOn = "none" | "warning" | "hint";
export type CheckerCliFormat = "text" | "json";
export type CheckerCliCategory = WorkspaceCheckCommandCategory;
export type CheckerCliSeverity = WorkspaceCheckCommandSeverity;
export type CheckerCliSummaryMode = "full" | "summary" | "compact";
export type CheckerCliPreset = WorkspaceCheckCommandPreset;

export async function runCheckerCli(
  argv: readonly string[],
  io: CheckerCliIO = defaultCliIO(),
): Promise<number> {
  const parsed = await parseCliArgs(argv, io);
  if ("helpText" in parsed) {
    io.stdout(parsed.helpText);
    return 0;
  }
  if ("bundleText" in parsed) {
    io.stdout(parsed.bundleText);
    return 0;
  }
  if ("ruleText" in parsed) {
    io.stdout(parsed.ruleText);
    return 0;
  }
  if ("error" in parsed) {
    io.stderr(`${parsed.error}\n`);
    io.stderr(buildHelpText());
    return 2;
  }

  const command = await runWorkspaceCheckCommand({
    workspace: parsed.options,
    filters: parsed.filters,
  });
  const rustStyleRecoveryCanonicalProducer = parsed.rustStyleRecoveryConsumer
    ? await buildRustStyleRecoveryCanonicalProducer(parsed.options, command.checkerReport)
    : undefined;
  const rustSourceMissingCanonicalProducer = parsed.rustSourceMissingConsumer
    ? await buildRustSourceMissingCanonicalProducer(parsed.options, command.checkerReport)
    : undefined;
  const rustStyleUnusedCanonicalProducer = parsed.rustStyleUnusedConsumer
    ? await buildRustStyleUnusedCanonicalProducer(parsed.options, command.checkerReport)
    : undefined;
  const rustFlowAnalysisConsumer = parsed.rustFlowAnalysisConsumer
    ? await buildRustFlowAnalysisConsumer(parsed.options)
    : undefined;
  const jsonReport = buildCheckerJsonReport(
    command.workspaceCheck,
    command.checkerReport,
    parsed.options.workspaceRoot,
    parsed.filters,
    rustStyleRecoveryCanonicalProducer,
    rustSourceMissingCanonicalProducer,
    rustStyleUnusedCanonicalProducer,
    rustFlowAnalysisConsumer,
  );
  const rustStyleRecoveryConsistency = jsonReport.rustStyleRecoveryConsistency;
  const rustSourceMissingConsistency = jsonReport.rustSourceMissingConsistency;
  const rustStyleUnusedConsistency = jsonReport.rustStyleUnusedConsistency;
  writeResult(
    command.workspaceCheck,
    command.checkerReport,
    jsonReport,
    parsed,
    io,
    rustStyleRecoveryCanonicalProducer,
    rustStyleRecoveryConsistency,
    rustSourceMissingCanonicalProducer,
    rustSourceMissingConsistency,
    rustStyleUnusedCanonicalProducer,
    rustStyleUnusedConsistency,
    rustFlowAnalysisConsumer,
  );
  return shouldFail(command.checkerReport, parsed.failOn) ? 1 : 0;
}

interface ParsedCliOptions {
  readonly options: WorkspaceCheckOptions;
  readonly filters: WorkspaceCheckCommandFilters;
  readonly format: CheckerCliFormat;
  readonly failOn: CheckerCliFailOn;
  readonly summaryMode: CheckerCliSummaryMode;
  readonly rustStyleRecoveryConsumer: boolean;
  readonly rustSourceMissingConsumer: boolean;
  readonly rustStyleUnusedConsumer: boolean;
  readonly rustFlowAnalysisConsumer: boolean;
}

async function parseCliArgs(
  argv: readonly string[],
  io: CheckerCliIO,
): Promise<
  | ParsedCliOptions
  | { readonly helpText: string }
  | { readonly bundleText: string }
  | { readonly ruleText: string }
  | { readonly error: string }
> {
  const cwd = io.cwd();
  const stdinFileList = argv.includes("--stdin-file-list") ? await readStdinFileList(io) : null;
  let workspaceRoot = cwd;
  let preset: CheckerCliPreset | null = null;
  let format: CheckerCliFormat = "text";
  let failOn: CheckerCliFailOn = "warning";
  let category: CheckerCliCategory = "all";
  let severity: CheckerCliSeverity = "all";
  let summaryMode: CheckerCliSummaryMode = "full";
  let rustStyleRecoveryConsumer = false;
  let rustSourceMissingConsumer = false;
  let rustStyleUnusedConsumer = false;
  let rustFlowAnalysisConsumer = false;
  let explicitFailOn = false;
  let explicitCategory = false;
  let explicitSeverity = false;
  let explicitSummaryMode = false;
  let explicitIncludeSelection = false;
  let classnameTransform: ClassnameTransformMode = "asIs";
  const pathAlias: Record<string, string> = {};
  const sourceFilePaths: string[] = [];
  const styleFilePaths: string[] = [];
  const includeCodes: string[] = [];
  const includeBundles: CheckerCodeBundle[] = [];
  const excludeCodes: string[] = [];
  let hasExplicitFileSelection = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!;

    if (arg === "--") continue;
    if (arg === "--help" || arg === "-h") return { helpText: buildHelpText() };
    if (arg === "--list-bundles") return { bundleText: buildBundleHelpText() };
    if (arg === "--list-rules") return { ruleText: buildRuleHelpText() };

    if (arg === "--root") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --root" };
      workspaceRoot = path.resolve(cwd, value);
      index += 1;
      continue;
    }

    if (arg === "--format") {
      const value = argv[index + 1];
      if (value !== "text" && value !== "json") return { error: "Expected --format text|json" };
      format = value;
      index += 1;
      continue;
    }

    if (arg === "--preset") {
      const value = argv[index + 1];
      if (value !== "ci" && value !== "changed-style" && value !== "changed-source") {
        return { error: "Expected --preset ci|changed-style|changed-source" };
      }
      preset = value;
      index += 1;
      continue;
    }

    if (arg === "--fail-on") {
      const value = argv[index + 1];
      if (value !== "none" && value !== "warning" && value !== "hint") {
        return { error: "Expected --fail-on none|warning|hint" };
      }
      failOn = value;
      explicitFailOn = true;
      index += 1;
      continue;
    }

    if (arg === "--category") {
      const value = argv[index + 1];
      if (value !== "all" && value !== "source" && value !== "style") {
        return { error: "Expected --category all|source|style" };
      }
      category = value;
      explicitCategory = true;
      index += 1;
      continue;
    }

    if (arg === "--severity") {
      const value = argv[index + 1];
      if (value !== "all" && value !== "warning" && value !== "hint") {
        return { error: "Expected --severity all|warning|hint" };
      }
      severity = value;
      explicitSeverity = true;
      index += 1;
      continue;
    }

    if (arg === "--summary") {
      summaryMode = "summary";
      explicitSummaryMode = true;
      continue;
    }

    if (arg === "--compact") {
      summaryMode = "compact";
      explicitSummaryMode = true;
      continue;
    }

    if (arg === "--rust-style-recovery-consumer") {
      rustStyleRecoveryConsumer = true;
      continue;
    }

    if (arg === "--rust-source-missing-consumer") {
      rustSourceMissingConsumer = true;
      continue;
    }

    if (arg === "--rust-style-unused-consumer") {
      rustStyleUnusedConsumer = true;
      continue;
    }

    if (arg === "--rust-flow-analysis-consumer") {
      rustFlowAnalysisConsumer = true;
      continue;
    }

    if (arg === "--classname-transform") {
      const value = argv[index + 1];
      if (
        value !== "asIs" &&
        value !== "camelCase" &&
        value !== "camelCaseOnly" &&
        value !== "dashes" &&
        value !== "dashesOnly"
      ) {
        return {
          error: "Expected --classname-transform asIs|camelCase|camelCaseOnly|dashes|dashesOnly",
        };
      }
      classnameTransform = value;
      index += 1;
      continue;
    }

    if (arg === "--path-alias") {
      const value = argv[index + 1];
      if (!value || !value.includes("=")) return { error: "Expected --path-alias prefix=target" };
      const eq = value.indexOf("=");
      const key = value.slice(0, eq);
      const target = value.slice(eq + 1);
      if (!key || !target) return { error: "Expected --path-alias prefix=target" };
      pathAlias[key] = target;
      index += 1;
      continue;
    }

    if (arg === "--include-code") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --include-code" };
      includeCodes.push(value);
      explicitIncludeSelection = true;
      index += 1;
      continue;
    }

    if (arg === "--include-bundle") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --include-bundle" };
      if (!isCheckerCodeBundle(value)) {
        return { error: `Expected --include-bundle ${buildBundleValueHelpText()}` };
      }
      includeBundles.push(value);
      explicitIncludeSelection = true;
      index += 1;
      continue;
    }

    if (arg === "--exclude-code") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --exclude-code" };
      excludeCodes.push(value);
      index += 1;
      continue;
    }

    if (arg === "--file-list") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --file-list" };
      hasExplicitFileSelection = true;
      addFileListEntries(readFileList(path.resolve(workspaceRoot, value)), workspaceRoot, {
        sourceFilePaths,
        styleFilePaths,
      });
      index += 1;
      continue;
    }

    if (arg === "--stdin-file-list") {
      hasExplicitFileSelection = true;
      addFileListEntries(stdinFileList ?? [], workspaceRoot, { sourceFilePaths, styleFilePaths });
      continue;
    }

    if (arg === "--source-file") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --source-file" };
      sourceFilePaths.push(path.resolve(workspaceRoot, value));
      hasExplicitFileSelection = true;
      index += 1;
      continue;
    }

    if (arg === "--style-file") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --style-file" };
      styleFilePaths.push(path.resolve(workspaceRoot, value));
      hasExplicitFileSelection = true;
      index += 1;
      continue;
    }

    if (arg === "--changed-file") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --changed-file" };
      const changedPath = path.resolve(workspaceRoot, value);
      hasExplicitFileSelection = true;
      if (isSourceFilePath(changedPath)) {
        sourceFilePaths.push(changedPath);
      } else if (findLangForPath(changedPath)) {
        styleFilePaths.push(changedPath);
      }
      index += 1;
      continue;
    }

    if (arg.startsWith("-")) return { error: `Unknown option: ${arg}` };
    workspaceRoot = path.resolve(cwd, arg);
  }

  if (preset) {
    const defaults = presetDefaults(preset);
    if (!explicitFailOn) failOn = defaults.failOn;
    if (!explicitCategory) category = defaults.category;
    if (!explicitSeverity) severity = defaults.severity;
    if (!explicitSummaryMode) summaryMode = defaults.summaryMode;
    if (!explicitIncludeSelection) {
      includeBundles.push(...defaults.includeBundles);
    }
  }

  const dedupedBundles = [...new Set(includeBundles)];
  return {
    options: {
      workspaceRoot,
      classnameTransform,
      pathAlias,
      ...(hasExplicitFileSelection
        ? {
            sourceFilePaths: [...new Set(sourceFilePaths)],
            styleFilePaths: [...new Set(styleFilePaths)],
          }
        : {}),
    },
    filters: {
      preset,
      category,
      severity,
      includeBundles: dedupedBundles,
      includeCodes: expandCheckerCodeBundles(dedupedBundles, [...new Set(includeCodes)]),
      excludeCodes: [...new Set(excludeCodes)],
    },
    format,
    failOn,
    summaryMode,
    rustStyleRecoveryConsumer,
    rustSourceMissingConsumer,
    rustStyleUnusedConsumer,
    rustFlowAnalysisConsumer,
  };
}

function presetDefaults(preset: CheckerCliPreset): {
  readonly failOn: CheckerCliFailOn;
  readonly category: CheckerCliCategory;
  readonly severity: CheckerCliSeverity;
  readonly summaryMode: CheckerCliSummaryMode;
  readonly includeBundles: readonly CheckerCodeBundle[];
} {
  switch (preset) {
    case "ci":
      return {
        failOn: "warning",
        category: "all",
        severity: "warning",
        summaryMode: "summary",
        includeBundles: ["ci-default"],
      };
    case "changed-style":
      return {
        failOn: "warning",
        category: "style",
        severity: "all",
        summaryMode: "compact",
        includeBundles: ["style-recovery", "style-unused"],
      };
    case "changed-source":
      return {
        failOn: "warning",
        category: "source",
        severity: "all",
        summaryMode: "compact",
        includeBundles: ["source-missing"],
      };
    default:
      preset satisfies never;
      return {
        failOn: "warning",
        category: "all",
        severity: "all",
        summaryMode: "full",
        includeBundles: [],
      };
  }
}

function writeResult(
  result: WorkspaceCheckResult,
  report: CheckerReportV1,
  jsonReport: CheckerReportJsonV1,
  parsed: Pick<ParsedCliOptions, "format" | "summaryMode">,
  io: CheckerCliIO,
  rustStyleRecoveryCanonicalProducer?: CheckerStyleRecoveryCanonicalProducerSignalV0,
  rustStyleRecoveryConsistency?: RustStyleRecoveryConsistencyV0,
  rustSourceMissingCanonicalProducer?: CheckerSourceMissingCanonicalProducerSignalV0,
  rustSourceMissingConsistency?: RustSourceMissingConsistencyV0,
  rustStyleUnusedCanonicalProducer?: CheckerStyleUnusedCanonicalProducerSignalV0,
  rustStyleUnusedConsistency?: RustStyleUnusedConsistencyV0,
  rustFlowAnalysisConsumer?: RustFlowAnalysisConsumerV0,
): void {
  if (parsed.format === "json") {
    io.stdout(`${JSON.stringify(jsonReport, null, 2)}\n`);
    return;
  }

  if (parsed.summaryMode === "full") {
    for (const finding of report.findings) {
      io.stdout(
        `${finding.filePath}:${finding.range.start.line + 1}:${finding.range.start.character + 1} [${
          finding.severity
        }] ${finding.code} ${finding.message}\n`,
      );
      for (const evidenceLine of formatFindingEvidenceLines(finding, "  ")) {
        io.stdout(evidenceLine);
      }
    }
  }

  if (parsed.summaryMode === "compact") {
    for (const [filePath, findings] of groupFindingsByFile(report)) {
      const relativePath = path.relative(process.cwd(), filePath) || filePath;
      io.stdout(`${relativePath} (${findings.length})\n`);
      for (const finding of findings) {
        io.stdout(
          `  ${finding.severity} ${finding.code} ${finding.range.start.line + 1}:${
            finding.range.start.character + 1
          } ${finding.message}\n`,
        );
        for (const evidenceLine of formatFindingEvidenceLines(finding, "    ")) {
          io.stdout(evidenceLine);
        }
      }
    }
  }

  io.stdout(
    `Checked ${result.sourceFiles.length} source files and ${result.styleFiles.length} style modules. ` +
      `${result.summary.total} findings (${result.summary.warnings} warnings, ${result.summary.hints} hints).\n`,
  );

  if (rustStyleRecoveryCanonicalProducer) {
    io.stdout(
      `Rust style-recovery consumer: findings=${rustStyleRecoveryCanonicalProducer.canonicalCandidate.summary.total} ` +
        `consistent=${rustStyleRecoveryConsistency?.findingsMatch === true} ` +
        `releaseGate=${rustStyleRecoveryCanonicalProducer.boundedCheckerGate.includedInRustReleaseBundle}\n`,
    );
  }

  if (rustSourceMissingCanonicalProducer) {
    io.stdout(
      `Rust source-missing consumer: findings=${rustSourceMissingCanonicalProducer.canonicalCandidate.summary.total} ` +
        `consistent=${rustSourceMissingConsistency?.findingsMatch === true} ` +
        `releaseGate=${rustSourceMissingCanonicalProducer.boundedCheckerGate.includedInRustReleaseBundle} ` +
        `flowGraphs=${rustSourceMissingCanonicalProducer.flowEvidence.graphCount}\n`,
    );
  }

  if (rustStyleUnusedCanonicalProducer) {
    io.stdout(
      `Rust style-unused consumer: findings=${rustStyleUnusedCanonicalProducer.canonicalCandidate.summary.total} ` +
        `consistent=${rustStyleUnusedConsistency?.findingsMatch === true} ` +
        `releaseGate=${rustStyleUnusedCanonicalProducer.boundedCheckerGate.includedInRustReleaseBundle}\n`,
    );
  }

  if (rustFlowAnalysisConsumer) {
    io.stdout(
      `Rust flow-analysis consumer: graphs=${rustFlowAnalysisConsumer.graphCount} ` +
        `nodes=${rustFlowAnalysisConsumer.nodeCount} ` +
        `converged=${rustFlowAnalysisConsumer.convergedGraphCount}/${rustFlowAnalysisConsumer.graphCount}\n`,
    );
  }
}

function shouldFail(report: CheckerReportV1, failOn: CheckerCliFailOn): boolean {
  if (failOn === "none") return false;
  if (failOn === "hint") return report.summary.total > 0;
  return report.summary.warnings > 0;
}

function isSourceFilePath(filePath: string): boolean {
  return /\.(?:[cm]?[jt]sx?)$/u.test(filePath) || filePath.endsWith(".d.ts");
}

function addFileListEntries(
  entries: readonly string[],
  workspaceRoot: string,
  acc: {
    readonly sourceFilePaths: string[];
    readonly styleFilePaths: string[];
  },
): void {
  for (const entry of entries) {
    const resolved = path.resolve(workspaceRoot, entry);
    if (isSourceFilePath(resolved)) {
      acc.sourceFilePaths.push(resolved);
      continue;
    }
    if (findLangForPath(resolved)) {
      acc.styleFilePaths.push(resolved);
    }
  }
}

function readFileList(filePath: string): readonly string[] {
  return parseFileList(readFileSync(filePath, "utf8"));
}

async function readStdinFileList(io: CheckerCliIO): Promise<readonly string[]> {
  const raw = io.stdin ? await io.stdin() : await readProcessStdin();
  return parseFileList(raw);
}

function parseFileList(raw: string): readonly string[] {
  return raw
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.startsWith("#"));
}

async function readProcessStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  return new Promise((resolve, reject) => {
    process.stdin.on("data", (chunk) => {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    });
    process.stdin.on("end", () => {
      resolve(Buffer.concat(chunks).toString("utf8"));
    });
    process.stdin.on("error", reject);
  });
}

function buildHelpText(): string {
  return [
    "Usage: pnpm check:workspace -- [root] [options]",
    "",
    "Options:",
    "  --root <path>                 Workspace root (defaults to cwd)",
    "  --format <text|json>         Output format (default: text)",
    "  --preset <ci|changed-style|changed-source> Apply a preset bundle",
    "  --fail-on <none|warning|hint> Exit threshold (default: warning)",
    "  --category <all|source|style> Filter findings by category",
    "  --severity <all|warning|hint> Filter findings by severity",
    "  --summary                    Print summary only for text output",
    "  --compact                    Group text output by file for changed-file workflows",
    "  --rust-style-recovery-consumer Consume the bounded Rust style-recovery producer alongside the TS checker result",
    "  --rust-source-missing-consumer Consume the bounded Rust source-missing producer alongside the TS checker result",
    "  --rust-style-unused-consumer Consume the bounded Rust style-unused producer alongside the TS checker result",
    "  --rust-flow-analysis-consumer Include Rust expression-domain flow analysis summary in checker output",
    "  --classname-transform <mode> Style transform mode",
    "  --path-alias <prefix=target> Repeatable native path-alias override",
    "  --include-code <code>        Restrict findings to one rule code (repeatable)",
    "  --include-bundle <bundle>    Restrict findings to one named code bundle",
    `                               Available: ${buildBundleValueHelpText()}`,
    "  --list-bundles               Print named code bundles and exit",
    "  --list-rules                 Print checker rule metadata and exit",
    "  --exclude-code <code>        Remove one rule code from output (repeatable)",
    "  --source-file <path>         Restrict source checking to one file (repeatable)",
    "  --style-file <path>          Restrict style checking to one file (repeatable)",
    "  --changed-file <path>        Auto-route changed source/style file (repeatable)",
    "  --file-list <path>           Read changed file paths from a newline-delimited file",
    "  --stdin-file-list            Read changed file paths from stdin",
    "  --help                       Show this help",
    "",
  ].join("\n");
}

function buildBundleValueHelpText(): string {
  return listCheckerCodeBundles()
    .map(({ bundle }) => bundle)
    .join("|");
}

function buildBundleHelpText(): string {
  return [
    "Named checker code bundles:",
    ...listCheckerCodeBundles().map(({ bundle, codes }) => `  ${bundle}: ${codes.join(", ")}`),
    "",
  ].join("\n");
}

function buildRuleHelpText(): string {
  return [
    "Checker rules:",
    ...listCheckerRuleDescriptors().map(
      (rule) =>
        `  ${rule.code}: category=${rule.category}, tier=${rule.tier}, severity=${rule.defaultSeverity}, fixability=${rule.fixability}, presets=${rule.presets.join("|") || "none"}`,
    ),
    "",
  ].join("\n");
}

function defaultCliIO(): CheckerCliIO {
  return {
    stdout: (message) => process.stdout.write(message),
    stderr: (message) => process.stderr.write(message),
    cwd: () => process.cwd(),
  };
}

function groupFindingsByFile(
  report: CheckerReportV1,
): ReadonlyMap<string, readonly CheckerReportV1["findings"][number][]> {
  const grouped = new Map<string, CheckerReportV1["findings"][number][]>();
  for (const finding of report.findings) {
    const filePath = finding.filePath;
    const existing = grouped.get(filePath);
    if (existing) {
      existing.push(finding);
      continue;
    }
    grouped.set(filePath, [finding]);
  }
  return grouped;
}

function formatFindingEvidenceLines(
  finding: CheckerReportV1["findings"][number],
  indent: string,
): readonly string[] {
  const lines: string[] = [];
  if (finding.analysisReason) {
    lines.push(`${indent}analysis: ${finding.analysisReason}\n`);
  }
  if (finding.valueCertaintyShapeLabel) {
    lines.push(`${indent}value shape: ${finding.valueCertaintyShapeLabel}\n`);
  }
  if (finding.valueDomainDerivationLabel) {
    lines.push(`${indent}value derivation: ${finding.valueDomainDerivationLabel}\n`);
  }
  return lines;
}
