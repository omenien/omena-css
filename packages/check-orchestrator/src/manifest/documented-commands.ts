import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import type { CheckDiagnostic, CheckGate } from "./types";

const PNPM_COMMAND_REF = /\bpnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)/g;
const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;

const DOCUMENTED_COMMAND_ROOTS = [
  "README.md",
  "CONTRIBUTING.md",
  "docs",
  "packages",
  "examples",
] as const;

const IGNORED_DIRECTORIES = new Set([
  ".git",
  ".next",
  ".turbo",
  "coverage",
  "dist",
  "node_modules",
  "out",
  "target",
]);

const IGNORED_MARKDOWN_FILES = new Set(["packages/check-orchestrator/CHECKS.md"]);

const PNPM_BUILTINS = new Set([
  "add",
  "audit",
  "changeset",
  "create",
  "dlx",
  "exec",
  "i",
  "install",
  "link",
  "publish",
  "remove",
  "run",
  "update",
  "why",
]);

const PROTECTED_PUBLIC_SCRIPTS = new Set(["build", "check", "package", "test"]);

const KNOWN_RETIRED_DOCUMENTED_COMMANDS = new Set([
  "check:rust-",
  "check:rust-input-producers-git-consumer",
  "check:rust-parser-git-consumer",
  "check:rust-parser-index-producer",
  "check:rust-split-consumer-pins",
  "check:rust-split-publish-readiness",
]);

export function findDocumentedPublicScriptDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const documentedCommands = collectDocumentedReferences(rootDir);

  for (const command of documentedCommands.scriptRefs) {
    if (!shouldProtectCommand(command.command, gates)) continue;
    if (findGateByPublicTarget(gates, command.command)) continue;

    diagnostics.push({
      severity: "error",
      code: "documented-public-script-missing",
      message: `${command.relativePath}:${command.line} documents "pnpm ${command.command}", but no package script or declared compatibility alias exposes it.`,
    });
  }

  for (const ref of documentedCommands.targetRefs) {
    const gate = findGateByTarget(gates, ref.target);
    if (!gate) {
      diagnostics.push({
        severity: "error",
        code: "documented-omena-check-target-missing",
        message: `${ref.relativePath}:${ref.line} documents "pnpm omena-check ${ref.command} ${ref.target}", but no manifest gate exposes that target.`,
      });
      continue;
    }
    if (ref.command === "bundle" && gate.kind !== "bundle" && gate.kind !== "alias") {
      diagnostics.push({
        severity: "error",
        code: "documented-omena-check-target-not-bundle",
        message: `${ref.relativePath}:${ref.line} documents "pnpm omena-check bundle ${ref.target}", but target "${gate.id}" is a ${gate.kind}.`,
      });
    }
  }

  return diagnostics;
}

interface DocumentedReferences {
  readonly scriptRefs: readonly DocumentedCommandRef[];
  readonly targetRefs: readonly DocumentedTargetRef[];
}

interface DocumentedCommandRef {
  readonly command: string;
  readonly relativePath: string;
  readonly line: number;
}

interface DocumentedTargetRef {
  readonly command: "run" | "bundle";
  readonly target: string;
  readonly relativePath: string;
  readonly line: number;
}

function collectDocumentedReferences(rootDir: string): DocumentedReferences {
  const refs: MutableDocumentedReferences = {
    scriptRefs: [],
    targetRefs: [],
  };

  for (const root of DOCUMENTED_COMMAND_ROOTS) {
    const absolutePath = path.join(rootDir, root);
    if (!existsSync(absolutePath)) continue;
    if (statSync(absolutePath).isDirectory()) {
      collectMarkdownCommands(rootDir, absolutePath, refs);
      continue;
    }
    collectFileCommands(rootDir, absolutePath, refs);
  }

  return refs;
}

interface MutableDocumentedReferences {
  readonly scriptRefs: DocumentedCommandRef[];
  readonly targetRefs: DocumentedTargetRef[];
}

function collectMarkdownCommands(
  rootDir: string,
  directory: string,
  refs: MutableDocumentedReferences,
): void {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (IGNORED_DIRECTORIES.has(entry.name)) continue;
      collectMarkdownCommands(rootDir, path.join(directory, entry.name), refs);
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".md")) {
      collectFileCommands(rootDir, path.join(directory, entry.name), refs);
    }
  }
}

function collectFileCommands(
  rootDir: string,
  absolutePath: string,
  refs: MutableDocumentedReferences,
): void {
  const relativePath = path.relative(rootDir, absolutePath);
  if (IGNORED_MARKDOWN_FILES.has(relativePath)) return;

  const content = readFileSync(absolutePath, "utf8");
  const lineStarts = buildLineStarts(content);
  for (const match of content.matchAll(PNPM_COMMAND_REF)) {
    const command = match[1];
    if (!command) continue;
    if (PNPM_BUILTINS.has(command) || command === "omena-check") continue;
    refs.scriptRefs.push({
      command,
      relativePath,
      line: lineForIndex(lineStarts, match.index ?? 0),
    });
  }

  for (const match of content.matchAll(OMENA_CHECK_TARGET_REF)) {
    const command = match[1];
    const target = match[2];
    if ((command !== "run" && command !== "bundle") || !target) continue;
    refs.targetRefs.push({
      command,
      target,
      relativePath,
      line: lineForIndex(lineStarts, match.index ?? 0),
    });
  }
}

function shouldProtectCommand(command: string, gates: readonly CheckGate[]): boolean {
  if (KNOWN_RETIRED_DOCUMENTED_COMMANDS.has(command)) return false;
  return (
    PROTECTED_PUBLIC_SCRIPTS.has(command) ||
    command.startsWith("check:") ||
    command.startsWith("release:") ||
    command.startsWith("test:") ||
    Boolean(findGateByPublicTarget(gates, command))
  );
}

function findGateByPublicTarget(gates: readonly CheckGate[], command: string): CheckGate | null {
  return (
    gates.find((gate) => gate.scriptName === command) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(command)) ??
    null
  );
}

function findGateByTarget(gates: readonly CheckGate[], target: string): CheckGate | null {
  return (
    gates.find((gate) => gate.id === target) ??
    gates.find((gate) => gate.scriptName === target && !gate.deprecatedBy) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.scriptName === target) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`)) ??
    null
  );
}

function buildLineStarts(content: string): readonly number[] {
  const starts = [0];
  for (let index = 0; index < content.length; index += 1) {
    if (content[index] === "\n") starts.push(index + 1);
  }
  return starts;
}

function lineForIndex(lineStarts: readonly number[], index: number): number {
  let line = 1;
  for (const start of lineStarts.slice(1)) {
    if (start > index) {
      return line;
    }
    line += 1;
  }
  return lineStarts.length;
}
