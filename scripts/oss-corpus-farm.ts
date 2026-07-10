import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { runCheckerCli } from "../server/checker-cli/src";

type Dialect = "css" | "scss" | "less" | "sass";
type ExpectationKind =
  | "static-must-match"
  | "expected-sound-bail"
  | "parser-recovery"
  | "out-of-scope";
type Stage = "stage1-advisory" | "stage2-blocking";
type DiffKind = "pass" | "missing-baseline" | "pin-change" | "regression";

interface ExternalCorpusDifferentialManifestV1 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly selectionCriteria: OssCorpusFarmSelectionCriteriaV0;
  readonly fixtures: readonly ExternalCorpusEnvelopeV1[];
}

interface OssCorpusFarmSelectionCriteriaV0 {
  readonly minimumDialectCount: number;
  readonly maxSparsePathCountPerEntry: number;
  readonly maxChunkSourceBytes: number;
  readonly maxSelectedWorktreeFiles: number;
  readonly maxSelectedWorktreeBytes: number;
}

interface ExternalCorpusEnvelopeV1 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: Stage;
  readonly dialect?: Dialect;
  readonly expectationKind?: ExpectationKind;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
    readonly helperClasses: readonly string[];
    readonly layoutDependentHelpersExcluded: readonly string[];
  };
  readonly generation: {
    readonly tool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs?: readonly string[];
  };
  readonly provenance?: {
    readonly generationTool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs: readonly string[];
  };
  readonly chunks: readonly {
    readonly chunkId: string;
    readonly path: string;
    readonly sha256: string;
    readonly fixtureCount: number;
  }[];
}

interface BaselineLedgerV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly generatedBy: string;
  readonly baselines: readonly BaselineRecordV0[];
}

interface BaselineRecordV0 {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly factSetHash: string;
  readonly factCount: number;
}

interface CheckerReportV1 {
  readonly sourceFiles?: readonly string[];
  readonly styleFiles?: readonly string[];
  readonly summary?: {
    readonly warnings?: number;
    readonly hints?: number;
    readonly total?: number;
  };
  readonly findings?: readonly {
    readonly code?: string;
    readonly severity?: string;
    readonly message?: string;
    readonly filePath?: string;
    readonly range?: unknown;
  }[];
}

interface FactSetRecordV0 extends BaselineRecordV0 {
  readonly canonicalJson: string;
}

interface FarmReportV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly entryCount: number;
  readonly comparedCount: number;
  readonly passCount: number;
  readonly pinChangeCount: number;
  readonly regressionCount: number;
  readonly missingBaselineCount: number;
  readonly reports: readonly FarmEntryReportV0[];
}

interface FarmEntryReportV0 {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly factSetHash: string;
  readonly factCount: number;
  readonly diffKind: DiffKind;
}

interface RegressionManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly fixtures: readonly RegressionManifestFixtureV0[];
}

interface RegressionManifestFixtureV0 {
  readonly id: string;
  readonly path: string;
  readonly status: string;
  readonly issue?: {
    readonly repository: string;
    readonly number: number;
  };
  readonly sourceProvenance?: {
    readonly repository: string;
    readonly pin: string;
  };
  readonly minimization?: string;
}

const repoRoot = process.cwd();
const farmRoot = path.join(repoRoot, "rust/crates/omena-diff-test/oss-corpus-farm");
const manifestPath = path.join(farmRoot, "manifest.json");
const baselinePath = path.join(farmRoot, "baselines.json");
const reportPath = path.join(farmRoot, "report.json");
const regressionRoot = path.join(repoRoot, "rust/crates/omena-diff-test/regressions");
const regressionManifestPath = path.join(regressionRoot, "manifest.json");
const rawCaptureRoot = process.env.OMENA_OSS_CORPUS_CAPTURE_DIR
  ? path.resolve(repoRoot, process.env.OMENA_OSS_CORPUS_CAPTURE_DIR)
  : regressionRoot;

const args = new Set(process.argv.slice(2));

void (async () => {
  if (args.has("--classifier-fixture")) {
    process.stdout.write(`${JSON.stringify(classifierFixtureReport())}\n`);
    return;
  }
  if (args.has("--path-policy-fixture")) {
    const candidates = ["src", "src/styles", ".", "", "../outside", "/absolute"];
    process.stdout.write(
      `${JSON.stringify(candidates.map((candidate) => [candidate, isBoundedPath(candidate)]))}\n`,
    );
    return;
  }

  if (args.has("--determinism-fixture")) {
    const fixturePath = valueAfter("--determinism-fixture");
    await checkDeterministicProjection(path.resolve(repoRoot, fixturePath));
    return;
  }

  const manifest = readManifest();
  assertManifest(manifest);

  if (args.has("--write-baseline")) {
    const fresh = await runFarm(manifest.fixtures);
    writeBaselines(fresh);
    writeReport(buildReport(fresh, fresh));
    return;
  }

  const baselines = readJson<BaselineLedgerV0>(baselinePath);
  assert.equal(baselines.schemaVersion, "0");
  assert.equal(baselines.product, "omena-diff-test.oss-corpus-farm.baselines");
  const fresh = await runFarm(manifest.fixtures);
  const report = buildReport(fresh, baselines.baselines);
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);

  const failures = report.reports.filter((entry) => entry.diffKind !== "pass");
  assert.deepEqual(
    failures,
    [],
    `oss corpus farm detected baseline differences:\n${JSON.stringify(failures, null, 2)}`,
  );
})();

async function checkDeterministicProjection(workspaceRoot: string): Promise<void> {
  assert.ok(existsSync(workspaceRoot), `determinism fixture must exist: ${workspaceRoot}`);
  const left = await projectWorkspaceFactSet({
    id: "local-determinism-fixture",
    repository: "local",
    pin: "local@0000000000000000000000000000000000000000",
    checkoutDir: workspaceRoot,
  });
  const right = await projectWorkspaceFactSet({
    id: "local-determinism-fixture",
    repository: "local",
    pin: "local@0000000000000000000000000000000000000000",
    checkoutDir: workspaceRoot,
  });
  assert.equal(left.factSetHash, right.factSetHash);
  assert.ok(left.factCount > 0, "deterministic fixture must produce at least one fact");
  process.stdout.write(
    `${JSON.stringify(
      {
        product: "omena-diff-test.oss-corpus-farm.determinism",
        factSetHash: left.factSetHash,
        factCount: left.factCount,
      },
      null,
      2,
    )}\n`,
  );
}

async function runFarm(entries: readonly ExternalCorpusEnvelopeV1[]): Promise<FactSetRecordV0[]> {
  const criteria = readManifest().selectionCriteria;
  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-oss-corpus-farm-"));
  try {
    const records: FactSetRecordV0[] = [];
    for (const entry of entries) {
      const id = entryId(entry);
      const checkoutDir = path.join(tempRoot, id);
      checkoutEntry(entry, checkoutDir);
      assertSelectedWorktreeWithinCeiling(entry, checkoutDir, criteria);
      records.push(
        await projectWorkspaceFactSet({
          id,
          repository: entry.source.repository,
          pin: entry.source.pin,
          checkoutDir,
        }),
      );
    }
    return records;
  } finally {
    rmSync(tempRoot, { force: true, recursive: true });
  }
}

function checkoutEntry(entry: ExternalCorpusEnvelopeV1, checkoutDir: string): void {
  const sha = sourceSha(entry.source.pin);
  run("git", ["init", "-q", checkoutDir]);
  run("git", ["-C", checkoutDir, "remote", "add", "origin", entry.source.repository]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "init", "--no-cone"]);
  run("git", ["-C", checkoutDir, "sparse-checkout", "set", ...entry.source.sparsePaths]);
  run("git", ["-C", checkoutDir, "fetch", "--depth", "1", "origin", sha]);
  run("git", ["-C", checkoutDir, "checkout", "-q", "--detach", "FETCH_HEAD"]);
  const actualSha = run("git", ["-C", checkoutDir, "rev-parse", "HEAD"]).stdout.trim();
  assert.equal(actualSha, sha, `${entryId(entry)} checkout did not resolve to the pinned sha`);
}

async function projectWorkspaceFactSet(input: {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
  readonly checkoutDir: string;
}): Promise<FactSetRecordV0> {
  let stdout = "";
  let stderr = "";
  let exitCode: number;
  try {
    exitCode = await runCheckerCli(
      [input.checkoutDir, "--preset", "ci", "--fail-on", "none", "--format", "json"],
      {
        stdout: (message) => {
          stdout += message;
        },
        stderr: (message) => {
          stderr += message;
        },
        cwd: () => repoRoot,
      },
    );
  } catch (error) {
    maybeWriteRawReproducer(input, {
      reason: `checker threw: ${(error as Error).message}`,
      exitCode: 1,
      stdoutJson: "not-checked",
    });
    throw error;
  }
  if (exitCode !== 0) {
    maybeWriteRawReproducer(input, {
      reason: `checker exited ${exitCode}\n${stderr}`,
      exitCode,
      stdoutJson: "not-checked",
    });
  }
  assert.equal(exitCode, 0, `${input.id} checker exited ${exitCode}\n${stderr}`);
  let report: CheckerReportV1;
  try {
    report = JSON.parse(stdout) as CheckerReportV1;
  } catch (error) {
    maybeWriteRawReproducer(input, {
      reason: `checker json parse failed: ${(error as Error).message}`,
      exitCode: 0,
      stdoutJson: "unparseable",
    });
    throw error;
  }
  const { facts, contentFactCount } = projectReportFacts(input.checkoutDir, report);
  assert.ok(contentFactCount > 0, `${input.id} produced an empty content-derived fact set`);
  const canonicalJson = stableStringify({
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.fact-set",
    id: input.id,
    repository: input.repository,
    pin: input.pin,
    facts,
  });
  return {
    id: input.id,
    repository: input.repository,
    pin: input.pin,
    factSetHash: sha256(canonicalJson),
    factCount: facts.length,
    canonicalJson,
  };
}

function projectReportFacts(
  workspaceRoot: string,
  report: CheckerReportV1,
): { readonly facts: readonly unknown[]; readonly contentFactCount: number } {
  const contentFacts: unknown[] = [];
  for (const filePath of report.sourceFiles ?? []) {
    contentFacts.push({
      kind: "source-file",
      path: relativeWorkspacePath(workspaceRoot, filePath),
    });
  }
  for (const filePath of report.styleFiles ?? []) {
    contentFacts.push({ kind: "style-file", path: relativeWorkspacePath(workspaceRoot, filePath) });
  }
  const facts: unknown[] = [...contentFacts];
  facts.push({
    kind: "summary",
    warnings: report.summary?.warnings ?? 0,
    hints: report.summary?.hints ?? 0,
    total: report.summary?.total ?? 0,
  });
  for (const finding of report.findings ?? []) {
    const findingFact = {
      kind: "finding",
      code: finding.code ?? "",
      severity: finding.severity ?? "",
      message: finding.message ?? "",
      filePath: finding.filePath ? relativeWorkspacePath(workspaceRoot, finding.filePath) : "",
      range: finding.range ?? null,
    };
    contentFacts.push(findingFact);
    facts.push(findingFact);
  }
  return {
    facts: facts.sort((left, right) =>
      stableStringify(left).localeCompare(stableStringify(right), "en"),
    ),
    contentFactCount: contentFacts.length,
  };
}

function buildReport(
  fresh: readonly BaselineRecordV0[],
  baselines: readonly BaselineRecordV0[],
): FarmReportV0 {
  const baselineById = new Map(baselines.map((baseline) => [baseline.id, baseline]));
  const reports = fresh.map((record): FarmEntryReportV0 => {
    const baseline = baselineById.get(record.id);
    const diffKind: DiffKind = !baseline
      ? "missing-baseline"
      : baseline.pin !== record.pin
        ? "pin-change"
        : baseline.factSetHash === record.factSetHash
          ? "pass"
          : "regression";
    return {
      id: record.id,
      repository: record.repository,
      pin: record.pin,
      factSetHash: record.factSetHash,
      factCount: record.factCount,
      diffKind,
    };
  });
  return {
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.report",
    entryCount: reports.length,
    comparedCount: reports.filter((entry) => entry.diffKind !== "missing-baseline").length,
    passCount: reports.filter((entry) => entry.diffKind === "pass").length,
    pinChangeCount: reports.filter((entry) => entry.diffKind === "pin-change").length,
    regressionCount: reports.filter((entry) => entry.diffKind === "regression").length,
    missingBaselineCount: reports.filter((entry) => entry.diffKind === "missing-baseline").length,
    reports,
  };
}

function classifierFixtureReport(): FarmReportV0 {
  const record = (id: string, pin: string, factSetHash: string): FactSetRecordV0 => ({
    id,
    repository: "https://github.com/example/project",
    pin,
    factSetHash,
    factCount: 1,
    canonicalJson: "[]",
  });
  const basePin = "example/project@0000000000000000000000000000000000000001";
  const nextPin = "example/project@0000000000000000000000000000000000000002";
  const baselines = [
    record("pass", basePin, "same"),
    record("pin-change", basePin, "same"),
    record("regression", basePin, "before"),
  ];
  const fresh = [
    record("pass", basePin, "same"),
    record("pin-change", nextPin, "same"),
    record("regression", basePin, "after"),
    record("missing-baseline", basePin, "new"),
  ];
  return buildReport(fresh, baselines);
}

function writeBaselines(records: readonly BaselineRecordV0[]): void {
  const ledger: BaselineLedgerV0 = {
    schemaVersion: "0",
    product: "omena-diff-test.oss-corpus-farm.baselines",
    generatedBy: "scripts/oss-corpus-farm.ts",
    baselines: records.map(({ id, repository, pin, factSetHash, factCount }) => ({
      id,
      repository,
      pin,
      factSetHash,
      factCount,
    })),
  };
  writeFileSync(baselinePath, `${JSON.stringify(ledger, null, 2)}\n`);
}

function writeReport(report: FarmReportV0): void {
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
}

function readManifest(): ExternalCorpusDifferentialManifestV1 {
  return readJson<ExternalCorpusDifferentialManifestV1>(manifestPath);
}

function assertManifest(manifest: ExternalCorpusDifferentialManifestV1): void {
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.oss-corpus-farm.manifest");
  assert.equal(manifest.mode, "pinned-repo-fact-set");
  assert.ok(manifest.fixtures.length > 0, "oss corpus farm manifest must not be empty");
  assertSelectionCriteria(manifest.selectionCriteria);
  const dialects = new Set(manifest.fixtures.map((entry) => entry.dialect));
  assert.ok(
    dialects.size >= manifest.selectionCriteria.minimumDialectCount,
    "oss corpus farm manifest must meet its dialect floor",
  );
  assert.ok(dialects.has("css"), "oss corpus farm manifest must include css");
  assert.ok(dialects.has("scss"), "oss corpus farm manifest must include scss");
  assert.ok(dialects.has("less"), "oss corpus farm manifest must include less");
  for (const entry of manifest.fixtures) {
    assert.equal(entry.stage, "stage1-advisory");
    assert.equal(entry.expectationKind, "out-of-scope");
    assert.ok(entry.source.repository.startsWith("https://github.com/"));
    assert.ok(isSha(sourceSha(entry.source.pin)), `${entryId(entry)} must pin a 40-character sha`);
    assert.ok(entry.source.sparsePaths.length > 0, `${entryId(entry)} must declare sparse paths`);
    assert.ok(
      entry.source.sparsePaths.length <= manifest.selectionCriteria.maxSparsePathCountPerEntry,
      `${entryId(entry)} sparse path count must stay within the manifest ceiling`,
    );
    assert.ok(
      entry.source.sparsePaths.every(isBoundedPath),
      `${entryId(entry)} sparse paths must stay bounded`,
    );
    const refs = [
      ...(entry.generation.oraclePinRefs ?? []),
      ...(entry.provenance?.oraclePinRefs ?? []),
    ];
    assert.ok(refs.includes("spdx:MIT"), `${entryId(entry)} must record a permissive SPDX id`);
    assert.ok(
      refs.includes(`repo-sha:${sourceSha(entry.source.pin)}`),
      `${entryId(entry)} provenance sha must match source pin`,
    );
    assert.ok(entry.chunks.length > 0, `${entryId(entry)} must declare at least one chunk`);
    for (const chunk of entry.chunks) {
      assert.ok(isBoundedPath(chunk.path), `${chunk.chunkId} chunk path must stay bounded`);
      const chunkPath = path.join(farmRoot, chunk.path);
      assert.ok(existsSync(chunkPath), `${chunk.chunkId} chunk source must exist`);
      assert.ok(
        statSync(chunkPath).size <= manifest.selectionCriteria.maxChunkSourceBytes,
        `${chunk.chunkId} chunk source must stay within the manifest byte ceiling`,
      );
      assert.equal(sha256(readFileSync(chunkPath)), chunk.sha256);
      assert.ok(chunk.fixtureCount > 0, `${chunk.chunkId} fixture count must be non-zero`);
    }
  }
}

function assertSelectionCriteria(criteria: OssCorpusFarmSelectionCriteriaV0): void {
  assert.ok(criteria.minimumDialectCount >= 3, "selection criteria must require all seed dialects");
  assert.ok(
    criteria.maxSparsePathCountPerEntry > 0,
    "selection criteria must bound sparse path count",
  );
  assert.ok(criteria.maxChunkSourceBytes > 0, "selection criteria must bound chunk bytes");
  assert.ok(
    criteria.maxSelectedWorktreeFiles > 0,
    "selection criteria must bound selected worktree files",
  );
  assert.ok(
    criteria.maxSelectedWorktreeBytes > 0,
    "selection criteria must bound selected worktree bytes",
  );
}

function assertSelectedWorktreeWithinCeiling(
  entry: ExternalCorpusEnvelopeV1,
  checkoutDir: string,
  criteria: OssCorpusFarmSelectionCriteriaV0,
): void {
  const files = listTrackedFiles(checkoutDir);
  const byteCount = files.reduce((sum, filePath) => sum + statSync(filePath).size, 0);
  assert.ok(
    files.length <= criteria.maxSelectedWorktreeFiles,
    `${entryId(entry)} selected worktree file count ${files.length} exceeds ${criteria.maxSelectedWorktreeFiles}`,
  );
  assert.ok(
    byteCount <= criteria.maxSelectedWorktreeBytes,
    `${entryId(entry)} selected worktree bytes ${byteCount} exceeds ${criteria.maxSelectedWorktreeBytes}`,
  );
}

function maybeWriteRawReproducer(
  input: {
    readonly id: string;
    readonly repository: string;
    readonly pin: string;
    readonly checkoutDir: string;
  },
  event: {
    readonly reason: string;
    readonly exitCode: number;
    readonly stdoutJson: "not-checked" | "unparseable";
  },
): void {
  if (process.env.OMENA_OSS_CORPUS_CAPTURE_RAW !== "1") return;
  const fixtureDir = path.join(rawCaptureRoot, input.id);
  mkdirSync(fixtureDir, { recursive: true });
  const files = listLoadedFiles(input.checkoutDir).slice(0, 64);
  const fixture = [
    `--- expect: raw-reproducer`,
    `repository: ${input.repository}`,
    `pin: ${input.pin}`,
    `minimization: raw`,
    `captureBacklog: PARKED-HRX-DDMIN`,
    `exitCode: ${event.exitCode}`,
    `stdoutJson: ${event.stdoutJson}`,
    `reason: ${event.reason.replace(/\r?\n/gu, " | ")}`,
    ...files.flatMap((filePath) => [
      `--- file: ${relativeWorkspacePath(input.checkoutDir, filePath)} encoding:hex`,
      readFileSync(filePath).toString("hex"),
    ]),
  ].join("\n");
  writeFileSync(path.join(fixtureDir, "fixture.omena"), `${fixture}\n`);
  updateRawCaptureManifest(input);
}

function updateRawCaptureManifest(input: {
  readonly id: string;
  readonly repository: string;
  readonly pin: string;
}): void {
  const manifestPathForCapture =
    rawCaptureRoot === regressionRoot
      ? regressionManifestPath
      : path.join(rawCaptureRoot, "manifest.json");
  const manifest: RegressionManifestV0 = existsSync(manifestPathForCapture)
    ? readJson<RegressionManifestV0>(manifestPathForCapture)
    : {
        schemaVersion: "0",
        product: "omena-diff-test.regression-corpus",
        fixtures: [],
      };
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.regression-corpus");
  const fixture: RegressionManifestFixtureV0 = {
    id: input.id,
    path: `${input.id}/fixture.omena`,
    status: "raw",
    sourceProvenance: {
      repository: input.repository,
      pin: input.pin,
    },
    minimization: "raw",
  };
  const fixtures = manifest.fixtures.filter((entry) => entry.id !== input.id);
  fixtures.push(fixture);
  mkdirSync(path.dirname(manifestPathForCapture), { recursive: true });
  writeFileSync(manifestPathForCapture, `${JSON.stringify({ ...manifest, fixtures }, null, 2)}\n`);
}

function listLoadedFiles(root: string): string[] {
  return listTrackedFiles(root).filter((filePath) =>
    /\.(?:css|scss|sass|less|jsx?|tsx?|json)$/u.test(filePath),
  );
}

function listTrackedFiles(root: string): string[] {
  const result = run("git", ["-C", root, "ls-files"]);
  return result.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((filePath) => path.join(root, filePath))
    .filter((filePath) => existsSync(filePath));
}

function entryId(entry: ExternalCorpusEnvelopeV1): string {
  const chunk = entry.chunks[0];
  assert.ok(chunk, "oss corpus farm entry must include a chunk id");
  return chunk.chunkId;
}

function sourceSha(pin: string): string {
  const sha = pin.split("@").at(-1) ?? "";
  assert.ok(isSha(sha), `invalid source pin sha: ${pin}`);
  return sha;
}

function isSha(value: string): boolean {
  return /^[0-9a-f]{40}$/u.test(value);
}

function isBoundedPath(value: string): boolean {
  return (
    value.length > 0 &&
    value !== "." &&
    !path.isAbsolute(value) &&
    !value.split(/[\\/]/u).includes("..")
  );
}

function relativeWorkspacePath(workspaceRoot: string, filePath: string): string {
  const relativePath = path.relative(workspaceRoot, filePath);
  return relativePath || ".";
}

function stableStringify(value: unknown): string {
  return JSON.stringify(sortForJson(value));
}

function sortForJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortForJson);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .sort(([left], [right]) => left.localeCompare(right, "en"))
      .map(([key, child]) => [key, sortForJson(child)]),
  );
}

function sha256(input: string | Buffer): string {
  return createHash("sha256").update(input).digest("hex");
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function valueAfter(flag: string): string {
  const index = process.argv.indexOf(flag);
  const value = process.argv[index + 1];
  assert.ok(value, `missing value for ${flag}`);
  return value;
}

function run(command: string, args: readonly string[]): { readonly stdout: string } {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 64,
  });
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
