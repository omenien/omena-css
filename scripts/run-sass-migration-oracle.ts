import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { assertPinnedDartSassVersion, runPinnedDartSass } from "./lib/dart-sass-cli";

interface OracleRequest {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.sass-migration-oracle-request";
  readonly workspaceRoot: string;
  readonly files: readonly OracleFile[];
  readonly edits: readonly OracleEdit[];
}

interface OracleFile {
  readonly path: string;
  readonly source: string;
}

interface OracleEdit {
  readonly uri: string;
  readonly start: number;
  readonly end: number;
  readonly replacementText: string;
}

interface CompileResult {
  readonly status: number | null;
  readonly css?: string;
  readonly stderr: string;
}

const repoRoot = process.cwd();
const request = JSON.parse(readFileSync(0, "utf8")) as OracleRequest;
assert.equal(request.schemaVersion, "0");
assert.equal(request.product, "omena-cli.sass-migration-oracle-request");
assert.ok(request.files.length > 0, "oracle request must contain Sass workspace files");
assert.ok(request.edits.length > 0, "oracle request must contain migration edits");
const compilerVersion = assertPinnedDartSassVersion(repoRoot);
const beforeRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-migration-before-"));
const afterRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-migration-after-"));

try {
  const byPath = new Map(request.files.map((file) => [path.resolve(file.path), file.source]));
  const editsByPath = new Map<string, OracleEdit[]>();
  for (const edit of request.edits) {
    const editPath = path.resolve(fileUriToPath(edit.uri));
    const edits = editsByPath.get(editPath) ?? [];
    edits.push(edit);
    editsByPath.set(editPath, edits);
  }

  for (const [filePath, source] of byPath) {
    const relative = relativeWorkspacePath(request.workspaceRoot, filePath);
    writeWorkspaceFile(beforeRoot, relative, source);
    writeWorkspaceFile(afterRoot, relative, applyEdits(source, editsByPath.get(filePath) ?? []));
  }

  const workspaceEntries = [...byPath.keys()]
    .filter(isSassEntry)
    .toSorted((left, right) => left.localeCompare(right));
  const compileEntries =
    workspaceEntries.length > 0
      ? workspaceEntries
      : [...editsByPath.keys()].toSorted((left, right) => left.localeCompare(right));
  const relativeEntries = compileEntries.map((filePath) =>
    relativeWorkspacePath(request.workspaceRoot, filePath),
  );
  const before = compileSassWorkspace(beforeRoot, relativeEntries);
  const after = compileSassWorkspace(afterRoot, relativeEntries);
  const matched =
    before.status === 0 &&
    after.status === 0 &&
    normalizeCss(before.css ?? "") === normalizeCss(after.css ?? "");
  const results = [...editsByPath.keys()]
    .toSorted((left, right) => left.localeCompare(right))
    .map((filePath) => {
      const output: {
        uri: string;
        matched: boolean;
        beforeStatus: number | null;
        afterStatus: number | null;
        beforeCssSha256?: string;
        afterCssSha256?: string;
        beforeStderr: string;
        afterStderr: string;
      } = {
        uri: filePath,
        matched,
        beforeStatus: before.status,
        afterStatus: after.status,
        beforeStderr: normalizeStderr(before.stderr, beforeRoot),
        afterStderr: normalizeStderr(after.stderr, afterRoot),
      };
      if (before.css !== undefined) output.beforeCssSha256 = sha256(before.css);
      if (after.css !== undefined) output.afterCssSha256 = sha256(after.css);
      return output;
    });

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "omena-cli.sass-migration-oracle-result",
        compiler: { name: "dart-sass", package: "sass", version: compilerVersion.split(" ")[0] },
        allMatched: results.every((result) => result.matched),
        results,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(beforeRoot, { force: true, recursive: true });
  rmSync(afterRoot, { force: true, recursive: true });
}

function relativeWorkspacePath(workspaceRoot: string, filePath: string): string {
  const relative = path.relative(path.resolve(workspaceRoot), path.resolve(filePath));
  assert.ok(relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative));
  return relative;
}

function writeWorkspaceFile(root: string, relative: string, source: string): void {
  const outputPath = path.join(root, relative);
  mkdirSync(path.dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, source);
}

function applyEdits(source: string, edits: readonly OracleEdit[]): string {
  const output = edits
    .toSorted((left, right) => right.start - left.start)
    .reduce(
      (bytes, edit) => {
        assert.ok(edit.start >= 0 && edit.start <= edit.end && edit.end <= bytes.length);
        return Buffer.concat([
          bytes.subarray(0, edit.start),
          Buffer.from(edit.replacementText, "utf8"),
          bytes.subarray(edit.end),
        ]);
      },
      Buffer.from(source, "utf8"),
    );
  return output.toString("utf8");
}

function compileSass(root: string, relative: string): CompileResult {
  const inputPath = path.join(root, relative);
  const outputPath = path.join(root, `.oracle-output-${sha256(relative)}.css`);
  const result = runPinnedDartSass(
    ["--no-source-map", "--style", "expanded", inputPath, outputPath],
    repoRoot,
  );
  return {
    status: result.status,
    ...(result.status === 0 ? { css: readFileSync(outputPath, "utf8") } : {}),
    stderr: result.stderr,
  };
}

function compileSassWorkspace(root: string, relatives: readonly string[]): CompileResult {
  const results = relatives.map((relative) => ({ relative, result: compileSass(root, relative) }));
  const failed = results.find(({ result }) => result.status !== 0);
  return {
    status: failed?.result.status ?? 0,
    ...(failed
      ? {}
      : {
          css: results
            .map(({ relative, result }) => `${relative}\0${normalizeCss(result.css ?? "")}`)
            .join("\0"),
        }),
    stderr: results
      .filter(({ result }) => result.stderr.length > 0)
      .map(({ relative, result }) => `${relative}: ${result.stderr}`)
      .join("\n"),
  };
}

function isSassEntry(filePath: string): boolean {
  return /\.(?:sass|scss)$/u.test(filePath) && !path.basename(filePath).startsWith("_");
}

function fileUriToPath(uri: string): string {
  return uri.startsWith("file://") ? uri.slice("file://".length) : uri;
}

function normalizeCss(css: string): string {
  return `${css.trimEnd()}\n`;
}

function normalizeStderr(stderr: string, root: string): string {
  return stderr.replaceAll(root, "<oracle-workdir>");
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}
