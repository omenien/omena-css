import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

type SassDialectV0 = "scss" | "sass";

interface ExternalCorpusEnvelopeV1 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: "stage1-advisory" | "stage2-blocking";
  readonly dialect?: SassDialectV0;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
    readonly helperClasses: readonly string[];
    readonly layoutDependentHelpersExcluded: readonly string[];
  };
  readonly knownFailurePolicy: {
    readonly path: string;
    readonly schemaVersion: string;
    readonly stage2Blocking: boolean;
  };
  readonly generation: {
    readonly tool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs?: readonly string[];
  };
  readonly provenance: {
    readonly generationTool: string;
    readonly selectionPath: string;
    readonly oraclePinRefs: readonly string[];
  };
  readonly sparsePathFixtureCounts: readonly SparsePathFixtureCountV0[];
  readonly chunks: readonly ExternalCorpusChunkV1[];
}

interface ExistingSassSpecManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: "stage1-advisory" | "stage2-blocking";
  readonly source: ExternalCorpusEnvelopeV1["source"];
  readonly knownFailurePolicy: ExternalCorpusEnvelopeV1["knownFailurePolicy"];
}

interface ExternalCorpusChunkV1 {
  readonly chunkId: string;
  readonly path: string;
  readonly stage: "stage1-advisory" | "stage2-blocking";
  readonly sha256: string;
  readonly fixtureCount: number;
  readonly sparsePathFixtureCounts: readonly SparsePathFixtureCountV0[];
}

interface SparsePathFixtureCountV0 {
  readonly sparsePath: string;
  readonly fixtureCount: number;
}

interface ImportedSassSpecChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunkId: string;
  readonly sourcePin: string;
  readonly fixtures: readonly ImportedSassSpecFixtureV0[];
}

interface ImportedSassSpecFixtureV0 {
  readonly id: string;
  readonly upstreamPath: string;
  readonly dialect: SassDialectV0;
  readonly inputPath: string;
  readonly source: string;
  readonly memberPaths: readonly string[];
  readonly optionsYaml?: string;
  readonly expectedCss?: string;
  readonly expectedError?: string;
  readonly expectedWarning?: string;
}

interface HrxMemberV0 {
  readonly path: string;
  readonly bytes: Buffer;
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const toolPath = "scripts/generate-sass-spec-corpus.ts";
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus");
const sourceRoot = path.join(repoRoot, "rust/crates/omena-diff-test/fixtures/sass-spec-import");
const sourceRootRelative = "rust/crates/omena-diff-test/fixtures/sass-spec-import";
const baseManifestPath = path.join(corpusRoot, "manifest.json");
const manifestPath = path.join(corpusRoot, "imported-smoke-manifest.json");
const chunkPath = "imported-smoke.json";
const chunkId = "sass-spec-import-smoke";
const oraclePinRefs = ["dart-sass"] as const;

const baseManifest = readJson<ExistingSassSpecManifestV0>(baseManifestPath);
assert.equal(baseManifest.schemaVersion, "0");
assert.equal(baseManifest.product, "omena-diff-test.sass-spec-seed-corpus.manifest");
assert.match(baseManifest.source.pin, /^sass\/sass-spec@[0-9a-f]{40}$/u);

const archivePaths = findFiles(sourceRoot, ".hrx");
assert.ok(archivePaths.length > 0, "sass-spec import fixture root must contain HRX archives");

const fixtures = archivePaths.map((archivePath) => importArchive(archivePath, baseManifest.source));
assert.ok(fixtures.length > 0, "sass-spec import must emit at least one fixture");

const chunk: ImportedSassSpecChunkV0 = {
  schemaVersion: "0",
  product: "omena-diff-test.sass-spec-imported-corpus.chunk",
  chunkId,
  sourcePin: baseManifest.source.pin,
  fixtures,
};
const chunkSource = stableJson(chunk);
const chunkSha256 = createHash("sha256").update(chunkSource).digest("hex");
const sparsePathFixtureCounts = countSparsePathFixtures(baseManifest.source.sparsePaths, fixtures);
const manifest: ExternalCorpusEnvelopeV1 = {
  schemaVersion: "0",
  product: "omena-diff-test.sass-spec-imported-corpus.manifest",
  stage: baseManifest.stage,
  source: baseManifest.source,
  knownFailurePolicy: baseManifest.knownFailurePolicy,
  generation: {
    tool: toolPath,
    selectionPath: sourceRootRelative,
    oraclePinRefs,
  },
  provenance: {
    generationTool: toolPath,
    selectionPath: sourceRootRelative,
    oraclePinRefs,
  },
  sparsePathFixtureCounts,
  chunks: [
    {
      chunkId,
      path: chunkPath,
      stage: baseManifest.stage,
      sha256: chunkSha256,
      fixtureCount: fixtures.length,
      sparsePathFixtureCounts,
    },
  ],
};
const manifestSource = stableJson(manifest);

if (checkOnly) {
  assert.equal(readFileSync(path.join(corpusRoot, chunkPath), "utf8"), chunkSource);
  assert.equal(readFileSync(manifestPath, "utf8"), manifestSource);
} else {
  mkdirSync(corpusRoot, { recursive: true });
  writeFileSync(path.join(corpusRoot, chunkPath), chunkSource);
  writeFileSync(manifestPath, manifestSource);
}

process.stdout.write(
  stableJson({
    product: "omena-diff-test.sass-spec-corpus-generator",
    mode: checkOnly ? "check" : "write",
    sourcePin: baseManifest.source.pin,
    fixtureCount: fixtures.length,
    chunkCount: 1,
    chunks: [
      {
        chunkId,
        fixtureCount: fixtures.length,
        sha256: chunkSha256,
      },
    ],
    generatedFiles: [chunkPath, path.basename(manifestPath)],
  }),
);

function importArchive(
  archivePath: string,
  sourcePolicy: ExistingSassSpecManifestV0["source"],
): ImportedSassSpecFixtureV0 {
  const upstreamPath = path.relative(sourceRoot, archivePath).split(path.sep).join("/");
  assert.ok(
    sourcePolicy.sparsePaths.some(
      (sparsePath) => upstreamPath === sparsePath || upstreamPath.startsWith(`${sparsePath}/`),
    ),
    `${upstreamPath} must be covered by the sass-spec sparse path policy`,
  );
  const members = parseHrxArchive(readFileSync(archivePath));
  const memberByPath = new Map(members.map((member) => [member.path, member]));
  const inputMember = members.find(
    (member) => member.path.endsWith(".scss") || member.path.endsWith(".sass"),
  );
  assert.ok(inputMember, `${upstreamPath} must contain input.scss or input.sass`);
  const dialect = inputMember.path.endsWith(".sass") ? "sass" : "scss";
  const fixture = {
    id: fixtureIdFromUpstreamPath(upstreamPath),
    upstreamPath,
    dialect,
    inputPath: inputMember.path,
    source: inputMember.bytes.toString("utf8"),
    memberPaths: members.map((member) => member.path),
    optionsYaml: memberByPath.get("options.yml")?.bytes.toString("utf8"),
    expectedCss: memberByPath.get("output.css")?.bytes.toString("utf8"),
    expectedError: memberByPath.get("error")?.bytes.toString("utf8"),
    expectedWarning: memberByPath.get("warning")?.bytes.toString("utf8"),
  } satisfies ImportedSassSpecFixtureV0;
  assert.ok(
    fixture.expectedCss !== undefined ||
      fixture.expectedError !== undefined ||
      fixture.expectedWarning !== undefined,
    `${upstreamPath} must contain output.css, error, or warning`,
  );
  return fixture;
}

function parseHrxArchive(source: Buffer): readonly HrxMemberV0[] {
  const members: HrxMemberV0[] = [];
  let cursor = 0;
  let currentPath: string | undefined;
  let contentStart = 0;
  const seenPaths = new Set<string>();

  while (cursor < source.length) {
    const lineStart = cursor;
    const newlineOffset = source.subarray(lineStart).indexOf(0x0a);
    const lineEnd = newlineOffset === -1 ? source.length : lineStart + newlineOffset + 1;
    const line = source.subarray(lineStart, lineEnd);
    const body = trimLineEnding(line);

    if (body.subarray(0, 5).equals(Buffer.from("<===>"))) {
      if (currentPath !== undefined) {
        members.push({
          path: currentPath,
          bytes: source.subarray(contentStart, lineStart),
        });
      } else {
        assert.equal(lineStart, 0, "HRX archives must begin with a member delimiter");
      }
      currentPath = body.subarray(5).toString("utf8").trim();
      assert.ok(currentPath.length > 0, "HRX member delimiter must name a path");
      assert.ok(!seenPaths.has(currentPath), `duplicate HRX member path: ${currentPath}`);
      seenPaths.add(currentPath);
      contentStart = lineEnd;
    } else {
      assert.ok(currentPath !== undefined, "HRX archives must begin with a member delimiter");
    }

    cursor = lineEnd;
  }

  assert.ok(currentPath !== undefined, "HRX archive must contain at least one member");
  members.push({
    path: currentPath,
    bytes: source.subarray(contentStart),
  });
  return members;
}

function trimLineEnding(line: Buffer): Buffer {
  let end = line.length;
  if (end > 0 && line[end - 1] === 0x0a) {
    end -= 1;
  }
  if (end > 0 && line[end - 1] === 0x0d) {
    end -= 1;
  }
  return line.subarray(0, end);
}

function fixtureIdFromUpstreamPath(upstreamPath: string): string {
  return upstreamPath
    .replace(/\.hrx$/u, "")
    .replaceAll("/", ".")
    .replaceAll("_", "-")
    .replaceAll(/[^a-zA-Z0-9.-]/gu, "-");
}

function findFiles(root: string, extension: string): readonly string[] {
  if (!existsSync(root)) {
    return [];
  }
  const entries = readdirSync(root)
    .map((entry) => path.join(root, entry))
    .toSorted();
  return entries.flatMap((entry) => {
    if (statSync(entry).isDirectory()) {
      return findFiles(entry, extension);
    }
    return entry.endsWith(extension) ? [entry] : [];
  });
}

function countSparsePathFixtures(
  sparsePaths: readonly string[],
  fixtureSet: readonly ImportedSassSpecFixtureV0[],
): readonly SparsePathFixtureCountV0[] {
  return sparsePaths.map((sparsePath) => ({
    sparsePath,
    fixtureCount: fixtureSet.filter(
      (fixture) =>
        fixture.upstreamPath === sparsePath || fixture.upstreamPath.startsWith(`${sparsePath}/`),
    ).length,
  }));
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
    return inlineArray.length <= 80 ? inlineArray : String(_match);
  });
}
