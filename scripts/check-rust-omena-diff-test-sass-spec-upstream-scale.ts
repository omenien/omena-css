import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

interface ExternalCorpusEnvelopeV1 {
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
  };
}

interface SassSpecUpstreamScaleArtifactV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
  };
  readonly archiveExtension: ".hrx";
  readonly archiveCount: number;
  readonly sparsePathArchiveCounts: readonly SparsePathArchiveCountV0[];
}

interface SparsePathArchiveCountV0 {
  readonly sparsePath: string;
  readonly archiveCount: number;
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check") || !process.argv.includes("--write");
const writeMode = process.argv.includes("--write");
const fetchMode = process.argv.includes("--fetch") || writeMode;
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus");
const manifestPath = path.join(corpusRoot, "imported-smoke-manifest.json");
const artifactPath = path.join(corpusRoot, "upstream-scale.json");

const manifest = readJson<ExternalCorpusEnvelopeV1>(manifestPath);
assert.match(manifest.source.pin, /^sass\/sass-spec@[0-9a-f]{40}$/u);
assert.ok(manifest.source.sparsePaths.length > 0, "sass-spec sparse path list must not be empty");

const currentArtifact = existsSync(artifactPath)
  ? readJson<SassSpecUpstreamScaleArtifactV0>(artifactPath)
  : undefined;

if (!fetchMode) {
  assert.ok(currentArtifact, "upstream scale artifact is missing");
  assertArtifactMatchesManifest(currentArtifact, manifest);
  assert.ok(currentArtifact.archiveCount > 0, "upstream archive count must be non-empty");
  assert.equal(
    currentArtifact.archiveCount,
    sumSparsePathArchiveCounts(currentArtifact.sparsePathArchiveCounts),
    "upstream archive count must equal sparse-path archive counts",
  );
  printSummary(currentArtifact, "check");
  process.exit(0);
}

const checkoutRoot = checkoutSassSpec(manifest);
try {
  const sparsePathArchiveCounts = manifest.source.sparsePaths.map((sparsePath) => ({
    sparsePath,
    archiveCount: countHrxArchives(path.join(checkoutRoot, sparsePath)),
  }));
  const artifact: SassSpecUpstreamScaleArtifactV0 = {
    schemaVersion: "0",
    product: "omena-diff-test.sass-spec-upstream-scale",
    source: manifest.source,
    archiveExtension: ".hrx",
    archiveCount: sumSparsePathArchiveCounts(sparsePathArchiveCounts),
    sparsePathArchiveCounts,
  };
  const artifactSource = stableJson(artifact);

  if (writeMode) {
    writeFileSync(artifactPath, artifactSource);
  }
  if (checkOnly && currentArtifact !== undefined) {
    assert.equal(readFileSync(artifactPath, "utf8"), artifactSource);
  }
  assertArtifactMatchesManifest(artifact, manifest);
  assert.ok(artifact.archiveCount > 0, "upstream archive count must be non-empty");
  printSummary(artifact, writeMode ? "write" : "fetch-check");
} finally {
  if (process.env.OMENA_SASS_SPEC_UPSTREAM_ROOT === undefined) {
    rmSync(path.dirname(checkoutRoot), { recursive: true, force: true });
  }
}

function checkoutSassSpec(manifest: ExternalCorpusEnvelopeV1): string {
  const configuredRoot = process.env.OMENA_SASS_SPEC_UPSTREAM_ROOT;
  if (configuredRoot !== undefined && configuredRoot.length > 0) {
    assert.ok(existsSync(configuredRoot), "OMENA_SASS_SPEC_UPSTREAM_ROOT must exist");
    const head = runGit(["rev-parse", "HEAD"], configuredRoot).trim();
    assert.equal(
      head,
      sourceSha(manifest.source.pin),
      "configured sass-spec checkout pin mismatch",
    );
    return configuredRoot;
  }

  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-spec-upstream-"));
  const checkoutRoot = path.join(tempRoot, "sass-spec");
  runGit(
    ["clone", "--filter=blob:none", "--sparse", manifest.source.repository, checkoutRoot],
    repoRoot,
  );
  runGit(["sparse-checkout", "set", ...manifest.source.sparsePaths], checkoutRoot);
  runGit(["checkout", sourceSha(manifest.source.pin)], checkoutRoot);
  return checkoutRoot;
}

function runGit(args: readonly string[], cwd: string): string {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  });
  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `git ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result.stdout;
}

function sourceSha(pin: string): string {
  const match = /^sass\/sass-spec@([0-9a-f]{40})$/u.exec(pin);
  assert.ok(match, `unexpected sass-spec source pin: ${pin}`);
  return match[1];
}

function countHrxArchives(root: string): number {
  assert.ok(existsSync(root), `sparse path root must exist: ${root}`);
  let count = 0;
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      count += countHrxArchives(entryPath);
    } else if (entry.isFile() && entry.name.endsWith(".hrx")) {
      count += 1;
    }
  }
  return count;
}

function assertArtifactMatchesManifest(
  artifact: SassSpecUpstreamScaleArtifactV0,
  manifest: ExternalCorpusEnvelopeV1,
): void {
  assert.equal(artifact.schemaVersion, "0");
  assert.equal(artifact.product, "omena-diff-test.sass-spec-upstream-scale");
  assert.equal(artifact.archiveExtension, ".hrx");
  assert.deepEqual(artifact.source, manifest.source);
  assert.deepEqual(
    artifact.sparsePathArchiveCounts.map((entry) => entry.sparsePath),
    manifest.source.sparsePaths,
  );
}

function sumSparsePathArchiveCounts(counts: readonly SparsePathArchiveCountV0[]): number {
  return counts.reduce((total, entry) => total + entry.archiveCount, 0);
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function stableJson(value: unknown): string {
  return `${inlineStringArrays(JSON.stringify(value, null, 2))}\n`;
}

function inlineStringArrays(source: string): string {
  return source.replace(/\[\n((?:\s+"(?:[^"\\]|\\.)*"(?:,\n)?)+)\s+\]/g, (_match, body) => {
    const values = String(body)
      .trim()
      .split(/\n/)
      .map((line) => line.trim().replace(/,$/, ""));
    const inlineArray = `[${values.join(", ")}]`;
    return inlineArray.length <= 100 ? inlineArray : String(_match);
  });
}

function printSummary(artifact: SassSpecUpstreamScaleArtifactV0, mode: string): void {
  process.stdout.write(
    stableJson({
      product: "omena-diff-test.sass-spec-upstream-scale.check",
      mode,
      sourcePin: artifact.source.pin,
      archiveCount: artifact.archiveCount,
      sparsePathArchiveCounts: artifact.sparsePathArchiveCounts,
    }),
  );
}
