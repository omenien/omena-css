import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

type SassDialectV0 = "scss" | "sass";

interface ExistingSassSpecManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly source: {
    readonly kind: "pinned-repository";
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
    readonly helperClasses: readonly string[];
    readonly layoutDependentHelpersExcluded: readonly string[];
  };
}

interface ConformanceSassSpecChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunkId: string;
  readonly sourcePin: string;
  readonly fixtures: readonly ConformanceSassSpecFixtureV0[];
}

interface ConformanceSassSpecFixtureV0 {
  readonly id: string;
  readonly upstreamPath: string;
  readonly dialect: SassDialectV0;
  readonly inputPath: string;
  readonly source: string;
  readonly memberPaths: readonly string[];
  readonly expectedCss: string;
}

interface HrxMemberV0 {
  readonly path: string;
  readonly bytes: Buffer;
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const toolPath = "scripts/generate-sass-spec-conformance-corpus.ts";
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus");
const sourceRoot = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/fixtures/sass-spec-conformance",
);
const sourceRootRelative = "rust/crates/omena-diff-test/fixtures/sass-spec-conformance";
const baseManifestPath = path.join(corpusRoot, "manifest.json");
const manifestPath = path.join(corpusRoot, "conformance-smoke-manifest.json");
const chunkPath = "conformance-smoke.json";
const chunkId = "sass-spec-conformance-smoke";

const baseManifest = readJson<ExistingSassSpecManifestV0>(baseManifestPath);
assert.equal(baseManifest.schemaVersion, "0");
assert.equal(baseManifest.product, "omena-diff-test.sass-spec-seed-corpus.manifest");
assert.equal(baseManifest.source.kind, "pinned-repository");
assert.match(baseManifest.source.pin, /^sass\/sass-spec@[0-9a-f]{40}$/u);

const archivePaths = findFiles(sourceRoot, ".hrx");
assert.ok(archivePaths.length > 0, "sass-spec conformance fixture root must contain HRX archives");

const fixtures = archivePaths
  .map((archivePath) => importArchive(archivePath))
  .sort((left, right) => left.id.localeCompare(right.id));
assert.equal(
  new Set(fixtures.map((fixture) => fixture.id)).size,
  fixtures.length,
  "conformance fixture ids must be unique",
);

const chunk: ConformanceSassSpecChunkV0 = {
  schemaVersion: "0",
  product: "omena-diff-test.sass-spec-conformance-corpus.chunk",
  chunkId,
  sourcePin: baseManifest.source.pin,
  fixtures,
};
const chunkSource = stableJson(chunk);
const manifest = {
  schemaVersion: "0",
  product: "omena-diff-test.sass-spec-conformance-corpus.manifest",
  source: {
    repository: baseManifest.source.repository,
    pin: baseManifest.source.pin,
    sourceRoot: sourceRootRelative,
  },
  generation: {
    tool: toolPath,
    purpose:
      "transform-pass cascade-conformance oracle corpus; fixtures are dart-sass-oracle-backed and are NOT part of the static-evaluator expectation buckets",
  },
  chunk: {
    chunkId,
    path: chunkPath,
    sha256: sha256(chunkSource),
    fixtureCount: fixtures.length,
  },
};
const manifestSource = stableJson(manifest);

if (checkOnly) {
  assert.equal(readFileSync(path.join(corpusRoot, chunkPath), "utf8"), chunkSource);
  assert.equal(readFileSync(manifestPath, "utf8"), manifestSource);
} else {
  writeFileSync(path.join(corpusRoot, chunkPath), chunkSource);
  writeFileSync(manifestPath, manifestSource);
}

process.stdout.write(
  stableJson({
    product: "omena-diff-test.sass-spec-conformance-corpus",
    mode: checkOnly ? "check" : "write",
    sourcePin: baseManifest.source.pin,
    fixtureCount: fixtures.length,
    chunkSha256: sha256(chunkSource),
    generatedFiles: [chunkPath, path.basename(manifestPath)],
  }),
);

function importArchive(archivePath: string): ConformanceSassSpecFixtureV0 {
  const upstreamPath = path.posix.join(
    "spec",
    path.relative(path.join(sourceRoot, "spec"), archivePath).split(path.sep).join("/"),
  );
  const members = parseHrxArchive(readFileSync(archivePath));
  const memberByPath = new Map(members.map((member) => [member.path, member]));
  const inputMembers = members.filter(
    (member) => member.path.endsWith("input.scss") || member.path.endsWith("input.sass"),
  );
  assert.equal(
    inputMembers.length,
    1,
    `${upstreamPath} must contain exactly one input member for conformance import`,
  );
  const inputMember = inputMembers[0];
  const dialect: SassDialectV0 = inputMember.path.endsWith(".sass") ? "sass" : "scss";
  const outputMember = memberByPath.get(siblingMemberPath(inputMember.path, "output.css"));
  assert.ok(outputMember, `${upstreamPath} must contain output.css`);
  assert.ok(
    !memberByPath.has(siblingMemberPath(inputMember.path, "error")) &&
      !memberByPath.has(siblingMemberPath(inputMember.path, "warning")),
    `${upstreamPath} must not expect an error or warning`,
  );
  return {
    id: fixtureIdFromUpstreamPath(upstreamPath, inputMember.path),
    upstreamPath,
    dialect,
    inputPath: inputMember.path,
    source: inputMember.bytes.toString("utf8"),
    memberPaths: members.map((member) => member.path),
    expectedCss: outputMember.bytes.toString("utf8"),
  };
}

function fixtureIdFromUpstreamPath(upstreamPath: string, inputPath: string): string {
  const basePath =
    inputPath === "input.scss" || inputPath === "input.sass"
      ? upstreamPath
      : `${upstreamPath.replace(/\.hrx$/u, "")}/${inputPath.replace(/\/input\.(s[ac]ss)$/u, "")}`;
  return basePath
    .replace(/\.hrx$/u, "")
    .replaceAll("/", ".")
    .replaceAll("_", "-")
    .replaceAll(/[^a-zA-Z0-9.-]/gu, "-");
}

function siblingMemberPath(inputPath: string, siblingName: string): string {
  const directory = path.posix.dirname(inputPath);
  return directory === "." ? siblingName : `${directory}/${siblingName}`;
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
      } else if (lineStart === 0) {
        assert.equal(lineStart, 0, "HRX archives must begin with a member delimiter");
      }
      const nextPath = body.subarray(5).toString("utf8").trim();
      if (nextPath.length === 0) {
        currentPath = undefined;
        contentStart = lineEnd;
        cursor = lineEnd;
        continue;
      }
      currentPath = nextPath;
      assert.ok(!seenPaths.has(currentPath), `duplicate HRX member path: ${currentPath}`);
      seenPaths.add(currentPath);
      contentStart = lineEnd;
    } else {
      assert.ok(
        currentPath !== undefined || body.every((byte) => byte === 0x3d),
        "HRX archives must begin with a member delimiter",
      );
    }

    cursor = lineEnd;
  }

  if (currentPath !== undefined) {
    members.push({
      path: currentPath,
      bytes: source.subarray(contentStart),
    });
  }
  assert.ok(members.length > 0, "HRX archive must contain at least one member");
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

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}
