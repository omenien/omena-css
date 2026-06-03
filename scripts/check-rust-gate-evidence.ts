import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";
import {
  RUST_GATE_EVIDENCE_CORPUS,
  RUST_GATE_EVIDENCE_VARIANTS,
  type RustGateEvidenceVariant,
} from "./rust-gate-evidence-corpus";
import {
  parseRustGateRepeatArg,
  summarizeRustGateRuns,
  type RustGateEvidenceSummary,
} from "./rust-gate-evidence-utils";

interface RustGateEvidenceResult {
  readonly variant: string;
  readonly label: string;
  readonly durationsMs: readonly number[];
  readonly exitCode: number;
  readonly summary: RustGateEvidenceSummary;
}

const argv = process.argv.slice(2);
const jsonMode = argv.includes("--json");
const repeatCount = parseRustGateRepeatArg(argv);
const selectedVariantLabels = parseVariantArgs(argv);
const selectedVariants = resolveVariants(selectedVariantLabels);
const results: RustGateEvidenceResult[] = [];

for (const variant of selectedVariants) {
  for (const entry of RUST_GATE_EVIDENCE_CORPUS.filter((candidate) =>
    candidate.variants ? candidate.variants.includes(variant.label) : true,
  )) {
    const durationsMs: number[] = [];
    let exitCode = 0;

    for (let runIndex = 0; runIndex < repeatCount; runIndex += 1) {
      const start = performance.now();
      const child = spawnSync("pnpm", entry.argv, {
        cwd: process.cwd(),
        encoding: "utf8",
        stdio: jsonMode ? "pipe" : "inherit",
        env: {
          ...process.env,
          ...variant.env,
        },
      });
      durationsMs.push(performance.now() - start);
      if ((child.status ?? 1) !== 0) {
        exitCode = child.status ?? 1;
        if (jsonMode) {
          const command = ["pnpm", ...entry.argv].join(" ");
          process.stderr.write(
            [
              `== rust-gate-evidence failure: ${variant.label}:${entry.label} ==`,
              `command: ${command}`,
              `exitCode: ${exitCode}`,
              child.stdout ? `stdout:\n${child.stdout.trimEnd()}` : "",
              child.stderr ? `stderr:\n${child.stderr.trimEnd()}` : "",
            ]
              .filter(Boolean)
              .join("\n")
              .concat("\n"),
          );
        }
      }
    }

    results.push({
      variant: variant.label,
      label: entry.label,
      durationsMs,
      exitCode,
      summary: summarizeRustGateRuns(durationsMs),
    });
  }
}

if (jsonMode) {
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "3",
        tool: "omena/rust-gate-evidence",
        repeatCount,
        variants: selectedVariants.map((variant) => ({
          label: variant.label,
        })),
        results: results.map((result) => ({
          variant: result.variant,
          label: result.label,
          durationsMs: result.durationsMs.map((durationMs) => Number(durationMs.toFixed(2))),
          exitCode: result.exitCode,
          summary: roundSummary(result.summary),
        })),
      },
      null,
      2,
    )}\n`,
  );
} else {
  process.stdout.write("== Rust gate evidence ==\n");
  for (const variant of selectedVariants) {
    process.stdout.write(`-- ${variant.label} --\n`);
    for (const result of results.filter((candidate) => candidate.variant === variant.label)) {
      process.stdout.write(
        `${result.label}: ${result.exitCode === 0 ? "ok" : "failed"} (${formatSummary(result.summary)})\n`,
      );
    }
  }
}

process.exitCode = results.some((result) => result.exitCode !== 0) ? 1 : 0;

function parseVariantArgs(cliArgs: readonly string[]): readonly string[] {
  const labels: string[] = [];

  for (let index = 0; index < cliArgs.length; index += 1) {
    const arg = cliArgs[index];
    if (arg !== "--variant") continue;
    const value = cliArgs[index + 1];
    if (value) {
      labels.push(value);
      index += 1;
    }
  }

  return labels;
}

function resolveVariants(labels: readonly string[]): readonly RustGateEvidenceVariant[] {
  if (labels.length === 0) return RUST_GATE_EVIDENCE_VARIANTS;

  const selected = labels.map((label) =>
    RUST_GATE_EVIDENCE_VARIANTS.find((variant) => variant.label === label),
  );
  const missing = labels.filter((_, index) => !selected[index]);
  if (missing.length > 0) {
    throw new Error(`Unknown rust gate evidence variant: ${missing.join(", ")}`);
  }

  return selected;
}

function roundSummary(summary: RustGateEvidenceSummary) {
  return {
    runCount: summary.runCount,
    minMs: Number(summary.minMs.toFixed(2)),
    averageMs: Number(summary.averageMs.toFixed(2)),
    p50Ms: Number(summary.p50Ms.toFixed(2)),
    p95Ms: Number(summary.p95Ms.toFixed(2)),
    maxMs: Number(summary.maxMs.toFixed(2)),
  };
}

function formatSummary(summary: RustGateEvidenceSummary): string {
  if (summary.runCount === 1) {
    return `${summary.averageMs.toFixed(2)} ms`;
  }

  return [
    `${summary.runCount} runs`,
    `min ${summary.minMs.toFixed(2)} ms`,
    `avg ${summary.averageMs.toFixed(2)} ms`,
    `p50 ${summary.p50Ms.toFixed(2)} ms`,
    `p95 ${summary.p95Ms.toFixed(2)} ms`,
    `max ${summary.maxMs.toFixed(2)} ms`,
  ].join(", ");
}
