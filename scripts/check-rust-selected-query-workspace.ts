import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import type { WorkspaceCheckerFinding } from "../server/engine-core-ts/src/core/checker";
import { findLangForPath } from "../server/engine-core-ts/src/core/scss/lang-registry";
import { runWorkspaceCheckCommand } from "../server/engine-host-node/src/checker-host";

const boundedWorkspaceRoot = mkdtempSync(
  path.join(os.tmpdir(), "cme-rust-selected-query-workspace-"),
);

const DEFAULT_FILTERS = {
  preset: null,
  category: "all",
  severity: "all",
  includeBundles: [],
  includeCodes: [],
  excludeCodes: [],
} as const;

const SOURCE_EXTENSIONS = new Set([".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs"]);

void (async () => {
  try {
    await checkBoundedWorkspace();
  } finally {
    rmSync(boundedWorkspaceRoot, { recursive: true, force: true });
  }

  await checkCurrentWorkspaceNoOverreport();
})();

async function checkBoundedWorkspace(): Promise<void> {
  writeFixture(
    "src/App.tsx",
    [
      "import classNames from 'classnames/bind';",
      "import styles from './Button.module.scss';",
      "const cx = classNames.bind(styles);",
      "const size: 'small' | 'large' = Math.random() > 0.5 ? 'small' : 'large';",
      "const value = cx(size);",
      "",
    ].join("\n"),
  );
  writeFixture("src/Button.module.scss", [".small {}", ".unused {}"].join("\n"));

  const result = await runWorkspaceCheckCommand({
    workspace: {
      workspaceRoot: boundedWorkspaceRoot,
      env: {
        ...process.env,
        OMENA_ENGINE_SHADOW_RUNNER: "prebuilt",
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      },
    },
    filters: DEFAULT_FILTERS,
  });

  const codes = result.checkerReport.findings.map((finding) => finding.code).toSorted();
  assert(codes.includes("missing-resolved-class-values"), `missing source finding: ${codes}`);
  assert(codes.includes("unused-selector"), `missing style finding: ${codes}`);

  process.stdout.write(
    `rust-selected-query bounded workspace ok: findings=${result.checkerReport.summary.total} codes=${codes.join(",")}\n`,
  );
}

async function checkCurrentWorkspaceNoOverreport(): Promise<void> {
  const repoWorkspaceRoot = process.cwd();
  const trackedFiles = resolveTrackedWorkspaceCheckFiles(repoWorkspaceRoot);
  const current = await runWorkspaceCheckCommand({
    workspace: {
      workspaceRoot: repoWorkspaceRoot,
      ...trackedFiles,
      env: {
        ...process.env,
        OMENA_SELECTED_QUERY_BACKEND: "typescript-current",
      },
    },
    filters: DEFAULT_FILTERS,
  });
  const rust = await runWorkspaceCheckCommand({
    workspace: {
      workspaceRoot: repoWorkspaceRoot,
      ...trackedFiles,
      env: {
        ...process.env,
        OMENA_ENGINE_SHADOW_RUNNER: "prebuilt",
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      },
    },
    filters: DEFAULT_FILTERS,
  });

  const currentKeys = new Set(
    current.workspaceCheck.findings.map((entry) => findingKey(repoWorkspaceRoot, entry)),
  );
  const rustKeys = new Set(
    rust.workspaceCheck.findings.map((entry) => findingKey(repoWorkspaceRoot, entry)),
  );
  const extraRustFindings = rust.workspaceCheck.findings.filter(
    (entry) => !currentKeys.has(findingKey(repoWorkspaceRoot, entry)),
  );
  const missingCurrentWarnings = current.workspaceCheck.findings.filter(
    (entry) =>
      entry.finding.severity === "warning" && !rustKeys.has(findingKey(repoWorkspaceRoot, entry)),
  );

  assert.equal(
    extraRustFindings.length,
    0,
    `rust-selected-query produced extra workspace findings:\n${formatFindings(
      repoWorkspaceRoot,
      extraRustFindings,
    )}`,
  );
  assert.equal(
    missingCurrentWarnings.length,
    0,
    `rust-selected-query missed current workspace warnings:\n${formatFindings(
      repoWorkspaceRoot,
      missingCurrentWarnings,
    )}`,
  );

  const missingCurrentHints = current.workspaceCheck.findings.filter(
    (entry) =>
      entry.finding.severity === "hint" && !rustKeys.has(findingKey(repoWorkspaceRoot, entry)),
  );

  process.stdout.write(
    [
      `rust-selected-query full workspace no-overreport ok: current=${summaryLabel(
        current.checkerReport.summary,
      )}`,
      `rust=${summaryLabel(rust.checkerReport.summary)}`,
      `trackedSources=${trackedFiles.sourceFilePaths.length}`,
      `trackedStyles=${trackedFiles.styleFilePaths.length}`,
      `allowedMissingHints=${missingCurrentHints.length}`,
      "",
    ].join(" "),
  );
}

function resolveTrackedWorkspaceCheckFiles(root: string): {
  readonly sourceFilePaths: readonly string[];
  readonly styleFilePaths: readonly string[];
} {
  const output = execFileSync("git", ["ls-files", "-z"], {
    cwd: root,
    encoding: "utf8",
  });
  const filePaths = output
    .split("\0")
    .filter(Boolean)
    .map((relativePath) => path.join(root, relativePath));
  return {
    sourceFilePaths: filePaths.filter(isSourceFile).toSorted(),
    styleFilePaths: filePaths.filter((filePath) => findLangForPath(filePath) !== null).toSorted(),
  };
}

function isSourceFile(filePath: string): boolean {
  return SOURCE_EXTENSIONS.has(path.extname(filePath));
}

function writeFixture(relativePath: string, content: string): void {
  const filePath = path.join(boundedWorkspaceRoot, relativePath);
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, content, "utf8");
}

function findingKey(root: string, entry: WorkspaceCheckerFinding): string {
  const range = entry.finding.range;
  return [
    path.relative(root, entry.filePath),
    entry.finding.category,
    entry.finding.severity,
    entry.finding.code,
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  ].join("\0");
}

function formatFindings(root: string, findings: readonly WorkspaceCheckerFinding[]): string {
  return findings.map((entry) => formatFinding(root, entry)).join("\n");
}

function formatFinding(root: string, entry: WorkspaceCheckerFinding): string {
  const range = entry.finding.range;
  return `${path.relative(root, entry.filePath)}:${range.start.line + 1}:${
    range.start.character + 1
  } [${entry.finding.severity}] ${entry.finding.code}`;
}

function summaryLabel(summary: {
  readonly warnings: number;
  readonly hints: number;
  readonly total: number;
}): string {
  return `${summary.total} findings (${summary.warnings} warnings, ${summary.hints} hints)`;
}
