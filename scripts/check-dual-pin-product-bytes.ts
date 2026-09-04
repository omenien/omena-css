import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { resolveUnmigratedScanRootForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";

const REPO_ROOT = path.resolve(import.meta.dirname, "..");
const BASELINE_PIN = "cfaf03e5a09fd6e0a5f5293c30b44903411f1af4";
const PRECISION_MANIFEST_PREFIX = "OMENA_PRECISION_FLOOR_EXECUTABLE_MANIFEST=";
const CORPUS_ID = "dual-pin-product-byte-corpus-v0";
const SIF_CANONICAL_URL = `https://fixtures.omena.invalid/${CORPUS_ID}/entry.css`;
const MAX_BUFFER = 128 * 1024 * 1024;

const CORPUS_FILES = [
  {
    relativePath: "corpus/entry.css",
    content: '@import "./tokens.css";\n.app,.panel{color:red;}\n',
    sha256: "2166a8aaad6ce6dc37a33ec552939a8358592fcf86b4d8ac1a58dfd28b436d8a",
  },
  {
    relativePath: "corpus/tokens.css",
    content: ":root{--brand:rgb(12,34,56)}\n.token { color: var(--brand); }\n",
    sha256: "3954d27f116d8ebe86edecd062f7e4f9479a201c54d74b963bb558c78ed31060",
  },
] as const;

// Every product command runs in its own freshly materialized copy. Fixed paths
// are literal output files; the two build templates are resolved exclusively
// from the emitted split manifest and then checked against the complete file set.
const COMMAND_OUTPUT_PATHS = {
  format: {
    fixed: ["corpus/entry.css"],
    manifestDerived: [],
  },
  minify: {
    fixed: ["out/minified.css"],
    manifestDerived: [],
  },
  build: {
    fixed: ["out/build.css", "out/split/omena.bundle-split.manifest.json"],
    manifestDerived: [
      "out/split/<manifest.outputs[*].fileName>",
      "out/split/<manifest.outputs[*].sourceMapFile>",
    ],
  },
  sif: {
    fixed: ["out/interface.sif.json"],
    manifestDerived: [],
  },
  bundle: {
    fixed: ["out/bundle.css", "out/bundle-evidence.json"],
    manifestDerived: [],
  },
} as const;

type ProductCommandId = keyof typeof COMMAND_OUTPUT_PATHS;

interface CommandRunResult {
  readonly stdout: string;
  readonly stderr: string;
}

interface PrecisionManifest {
  readonly observations: readonly unknown[];
  readonly producerGateArms: readonly unknown[];
}

interface PrecisionSnapshot {
  readonly observations: Buffer;
  readonly producerGateArms: Buffer;
}

interface SplitManifestOutput {
  readonly sourcePath: string;
  readonly fileName: string;
  readonly sourceMapFile: string | null;
}

interface SplitManifest {
  readonly schemaVersion: number;
  readonly product: string;
  readonly outputCount: number;
  readonly outputs: readonly SplitManifestOutput[];
}

interface CapturedOutput {
  readonly relativePath: string;
  readonly bytes: Buffer;
}

type ProductSnapshot = Readonly<Record<ProductCommandId, readonly CapturedOutput[]>>;

interface OutputDriftMutationReceipt {
  readonly command: ProductCommandId;
  readonly relativePath: string;
  readonly expectedFailure: string;
}

const scratchParent = mkdtempSync(path.join(os.tmpdir(), "omena-dual-pin-product-bytes-"));

try {
  verifyDeclaredCorpusDigests();
  const headPin = runChecked("resolve HEAD", "git", ["rev-parse", "HEAD"], {
    cwd: REPO_ROOT,
  }).stdout.trim();
  assert.match(headPin, /^[0-9a-f]{40}$/u, `HEAD did not resolve to a full commit id: ${headPin}`);
  assert.notEqual(headPin, BASELINE_PIN, "HEAD must differ from the baseline pin");
  const cargoTargetDir = path.join(scratchParent, "cargo-target");

  const baselineTree = materializeGitTree(BASELINE_PIN, "baseline");
  const baselinePrecision = capturePrecisionSnapshot(baselineTree, cargoTargetDir, "baseline");
  const baselineProduct = captureProductSnapshot(baselineTree, cargoTargetDir, "baseline");
  rmSync(cargoTargetDir, { force: true, recursive: true });

  const headTree = materializeGitTree(headPin, "head");
  const headPrecision = capturePrecisionSnapshot(headTree, cargoTargetDir, "HEAD");
  const headProduct = captureProductSnapshot(headTree, cargoTargetDir, "HEAD");

  assertByteIdentity(
    "precision observations",
    baselinePrecision.observations,
    headPrecision.observations,
  );
  assertByteIdentity(
    "precision producer arms",
    baselinePrecision.producerGateArms,
    headPrecision.producerGateArms,
  );
  compareProductSnapshots(baselineProduct, headProduct);
  const outputDriftMutation = exerciseOutputDriftMutation(baselineProduct, headProduct);

  const productOutputs = productCommandIds().flatMap((commandId) =>
    baselineProduct[commandId].map((output) => ({
      command: commandId,
      path: output.relativePath,
      sha256: sha256(output.bytes),
    })),
  );
  writeFileSync(
    process.stdout.fd,
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "omena.dual-pin-product-bytes",
        baselinePin: BASELINE_PIN,
        headPin,
        corpus: {
          id: CORPUS_ID,
          files: CORPUS_FILES.map(({ relativePath, sha256: digest }) => ({
            path: relativePath,
            sha256: digest,
          })),
        },
        precisionFloor: {
          observationCount: 12,
          observationSha256: sha256(baselinePrecision.observations),
          producerArmCount: 6,
          producerArmSha256: sha256(baselinePrecision.producerGateArms),
        },
        productBytes: {
          commandCount: productCommandIds().length,
          outputCount: productOutputs.length,
          outputs: productOutputs,
          outputDriftMutation,
        },
        driftCount: 0,
      },
      null,
      2,
    )}\nDRIFT=0\n`,
  );
} finally {
  rmSync(scratchParent, { force: true, recursive: true });
}

function verifyDeclaredCorpusDigests(): void {
  assert.equal(CORPUS_FILES.length, 2, "the fixture corpus must contain exactly two files");
  for (const fixture of CORPUS_FILES) {
    assertSafeRelativePath(fixture.relativePath);
    assert.equal(
      sha256(Buffer.from(fixture.content)),
      fixture.sha256,
      `declared corpus digest drifted for ${fixture.relativePath}`,
    );
  }
}

function materializeGitTree(pin: string, label: string): string {
  const treeRoot = path.join(scratchParent, "tree");
  const archivePath = path.join(scratchParent, `${label}.tar`);
  rmSync(treeRoot, { force: true, recursive: true });
  mkdirSync(treeRoot, { recursive: true });
  runChecked(`archive ${label}`, "git", ["archive", "--format=tar", "--output", archivePath, pin], {
    cwd: REPO_ROOT,
  });
  runChecked(`extract ${label}`, "tar", ["-xf", archivePath, "-C", treeRoot]);
  rmSync(archivePath, { force: true });
  return treeRoot;
}

function capturePrecisionSnapshot(
  treeRoot: string,
  cargoTargetDir: string,
  label: string,
): PrecisionSnapshot {
  const result = runChecked(
    `${label} precision manifest`,
    "cargo",
    [
      "test",
      "--locked",
      "--manifest-path",
      path.join(treeRoot, "rust", "Cargo.toml"),
      "-p",
      "omena-query",
      "--all-features",
      "precision_floor_manifest",
      "--",
      "--nocapture",
    ],
    { cwd: treeRoot, env: cargoEnvironment(cargoTargetDir) },
  );
  const manifestLines = result.stdout
    .split(/\r?\n/u)
    .filter((line) => line.startsWith(PRECISION_MANIFEST_PREFIX));
  assert.equal(
    manifestLines.length,
    1,
    `${label} must print exactly one ${PRECISION_MANIFEST_PREFIX} line`,
  );
  const payload = manifestLines[0]?.slice(PRECISION_MANIFEST_PREFIX.length);
  assert.ok(payload, `${label} precision manifest payload is empty`);
  const manifest = JSON.parse(payload) as PrecisionManifest;
  assert.ok(Array.isArray(manifest.observations), `${label} observations must be an array`);
  assert.ok(Array.isArray(manifest.producerGateArms), `${label} producerGateArms must be an array`);
  assert.equal(manifest.observations.length, 12, `${label} precision observation count drifted`);
  assert.equal(manifest.producerGateArms.length, 6, `${label} producer arm count drifted`);
  return {
    observations: extractJsonArrayBytes(payload, "observations"),
    producerGateArms: extractJsonArrayBytes(payload, "producerGateArms"),
  };
}

function extractJsonArrayBytes(payload: string, property: string): Buffer {
  const propertyToken = `"${property}":`;
  const propertyIndex = payload.indexOf(propertyToken);
  assert.notEqual(propertyIndex, -1, `precision manifest omits ${property}`);
  const arrayStart = payload.indexOf("[", propertyIndex + propertyToken.length);
  assert.notEqual(arrayStart, -1, `precision manifest ${property} is not an array`);
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = arrayStart; index < payload.length; index += 1) {
    const character = payload[index];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === '"') {
        inString = false;
      }
      continue;
    }
    if (character === '"') {
      inString = true;
      continue;
    }
    if (character === "[") depth += 1;
    if (character !== "]") continue;
    depth -= 1;
    if (depth === 0) return Buffer.from(payload.slice(arrayStart, index + 1));
  }
  assert.fail(`precision manifest ${property} array is unterminated`);
}

function captureProductSnapshot(
  treeRoot: string,
  cargoTargetDir: string,
  label: string,
): ProductSnapshot {
  runChecked(
    `${label} omena build`,
    "cargo",
    [
      "build",
      "--locked",
      "--manifest-path",
      path.join(treeRoot, "rust", "Cargo.toml"),
      "-p",
      "omena-cli",
      "--bin",
      "omena",
      "--quiet",
    ],
    { cwd: treeRoot, env: cargoEnvironment(cargoTargetDir) },
  );
  const binaryPath = path.join(cargoTargetDir, "debug", "omena");
  assert.ok(existsSync(binaryPath), `${label} omena binary was not built`);

  return {
    format: runProductCommand(binaryPath, "format"),
    minify: runProductCommand(binaryPath, "minify"),
    build: runProductCommand(binaryPath, "build"),
    sif: runProductCommand(binaryPath, "sif"),
    bundle: runProductCommand(binaryPath, "bundle"),
  };
}

function runProductCommand(binaryPath: string, commandId: ProductCommandId): CapturedOutput[] {
  const commandRoot = path.join(scratchParent, "product-workspaces", commandId);
  rmSync(commandRoot, { force: true, recursive: true });
  mkdirSync(commandRoot, { recursive: true });
  materializeCorpus(commandRoot);

  const entryPath = resolveRelative(commandRoot, "corpus/entry.css");
  const tokensPath = resolveRelative(commandRoot, "corpus/tokens.css");
  const outRoot = resolveRelative(commandRoot, "out");
  mkdirSync(outRoot, { recursive: true });

  const args = productCommandArgs(commandId, { entryPath, tokensPath, outRoot });
  runChecked(`omena ${commandId}`, binaryPath, args, {
    cwd: commandRoot,
    env: deterministicEnvironment(),
  });

  const mutableInputs = commandId === "format" ? new Set(["corpus/entry.css"]) : new Set<string>();
  assertUnchangedCorpusFiles(commandRoot, mutableInputs);
  if (commandId === "format") {
    assert.notEqual(
      sha256(readFileSync(entryPath)),
      CORPUS_FILES[0].sha256,
      "omena fmt did not exercise a source-byte mutation",
    );
  }

  const outputPaths = resolveCommandOutputPaths(commandId, commandRoot);
  const allowedWorkspacePaths = new Set([
    ...CORPUS_FILES.map((fixture) => fixture.relativePath),
    ...outputPaths,
  ]);
  assert.deepEqual(
    listRelativeFiles(
      resolveUnmigratedScanRootForScanner(
        import.meta.url,
        "non-repo-temp-tree",
        REPO_ROOT,
        commandRoot,
      ),
      commandRoot,
    ),
    [...allowedWorkspacePaths].toSorted(),
    `omena ${commandId} emitted an unexpected or incomplete file set`,
  );
  return outputPaths.toSorted().map((relativePath) => ({
    relativePath,
    bytes: readFileSync(resolveRelative(commandRoot, relativePath)),
  }));
}

function productCommandArgs(
  commandId: ProductCommandId,
  paths: { readonly entryPath: string; readonly tokensPath: string; readonly outRoot: string },
): string[] {
  const { entryPath, tokensPath, outRoot } = paths;
  switch (commandId) {
    case "format":
      return ["fmt", entryPath, "--mode", "pretty"];
    case "minify":
      return [
        "minify",
        entryPath,
        "--profile",
        "safe",
        "--backend",
        "omena",
        "--output",
        path.join(outRoot, "minified.css"),
      ];
    case "build":
      return [
        "build",
        entryPath,
        "--bundle",
        "--source",
        tokensPath,
        "--source-map",
        "--json",
        "--output",
        path.join(outRoot, "build.css"),
        "--split-out-dir",
        path.join(outRoot, "split"),
      ];
    case "sif":
      return [
        "sif",
        "generate",
        entryPath,
        "--canonical-url",
        SIF_CANONICAL_URL,
        "--output",
        path.join(outRoot, "interface.sif.json"),
      ];
    case "bundle":
      return [
        "bundle",
        entryPath,
        "--source",
        tokensPath,
        "--css-out",
        path.join(outRoot, "bundle.css"),
        "--evidence",
        path.join(outRoot, "bundle-evidence.json"),
      ];
  }
}

function resolveCommandOutputPaths(commandId: ProductCommandId, commandRoot: string): string[] {
  const fixed = [...COMMAND_OUTPUT_PATHS[commandId].fixed];
  if (commandId !== "build") return fixed;

  const manifestRelativePath = "out/split/omena.bundle-split.manifest.json";
  const manifest = JSON.parse(
    readFileSync(resolveRelative(commandRoot, manifestRelativePath), "utf8"),
  ) as SplitManifest;
  assert.equal(manifest.schemaVersion, 0, "split manifest schema version drifted");
  assert.equal(
    manifest.product,
    "omena-cli.bundle-code-split-manifest",
    "split manifest product drifted",
  );
  assert.equal(manifest.outputCount, 2, "split manifest output count drifted");
  assert.equal(manifest.outputs.length, 2, "split manifest outputs length drifted");

  const expectedSourcePaths = CORPUS_FILES.map((fixture) =>
    resolveRelative(commandRoot, fixture.relativePath),
  ).toSorted();
  assert.deepEqual(
    manifest.outputs.map((output) => output.sourcePath).toSorted(),
    expectedSourcePaths,
    "split manifest source-path set drifted",
  );

  const dynamicPaths: string[] = [];
  for (const output of manifest.outputs) {
    assert.equal(path.basename(output.fileName), output.fileName, "unsafe split output file name");
    assert.match(output.fileName, /^[A-Za-z0-9_.-]+\.css$/u, "invalid split CSS file name");
    assert.equal(
      output.sourceMapFile,
      `${output.fileName}.map`,
      `split output ${output.fileName} must carry its source-map sidecar`,
    );
    dynamicPaths.push(`out/split/${output.fileName}`);
    dynamicPaths.push(`out/split/${output.sourceMapFile}`);
  }
  assert.equal(new Set(dynamicPaths).size, 4, "split output paths must be unique");
  return [...fixed, ...dynamicPaths];
}

function materializeCorpus(commandRoot: string): void {
  for (const fixture of CORPUS_FILES) {
    const fixturePath = resolveRelative(commandRoot, fixture.relativePath);
    mkdirSync(path.dirname(fixturePath), { recursive: true });
    writeFileSync(fixturePath, fixture.content);
    assert.equal(
      sha256(readFileSync(fixturePath)),
      fixture.sha256,
      `materialized corpus digest drifted for ${fixture.relativePath}`,
    );
  }
}

function assertUnchangedCorpusFiles(commandRoot: string, mutableInputs: ReadonlySet<string>): void {
  for (const fixture of CORPUS_FILES) {
    if (mutableInputs.has(fixture.relativePath)) continue;
    assert.equal(
      sha256(readFileSync(resolveRelative(commandRoot, fixture.relativePath))),
      fixture.sha256,
      `omena command mutated undeclared input ${fixture.relativePath}`,
    );
  }
}

function compareProductSnapshots(baseline: ProductSnapshot, head: ProductSnapshot): void {
  for (const commandId of productCommandIds()) {
    const baselineOutputs = baseline[commandId];
    const headOutputs = head[commandId];
    assert.deepEqual(
      baselineOutputs.map((output) => output.relativePath),
      headOutputs.map((output) => output.relativePath),
      `${commandId} output path set drifted`,
    );
    for (let index = 0; index < baselineOutputs.length; index += 1) {
      const baselineOutput = baselineOutputs[index];
      const headOutput = headOutputs[index];
      assert.ok(baselineOutput && headOutput, `${commandId} output pairing is incomplete`);
      assertByteIdentity(
        `${commandId}:${baselineOutput.relativePath}`,
        baselineOutput.bytes,
        headOutput.bytes,
      );
    }
  }
}

function exerciseOutputDriftMutation(
  baseline: ProductSnapshot,
  head: ProductSnapshot,
): OutputDriftMutationReceipt {
  const command: ProductCommandId = "bundle";
  const relativePath = "out/bundle.css";
  const mutatedOutputs = head[command].map((output) => {
    if (output.relativePath !== relativePath) return output;
    const bytes = Buffer.from(output.bytes);
    assert.ok(bytes.length > 0, `${command}:${relativePath} must be non-empty`);
    bytes[0] = (bytes[0] ?? 0) ^ 1;
    return { ...output, bytes };
  });
  assert.ok(
    mutatedOutputs.some((output) => output.relativePath === relativePath),
    `output-drift mutation subject is absent: ${command}:${relativePath}`,
  );
  const mutatedSnapshot: ProductSnapshot = { ...head, [command]: mutatedOutputs };
  let expectedFailure = "";
  try {
    compareProductSnapshots(baseline, mutatedSnapshot);
  } catch (error) {
    expectedFailure = error instanceof Error ? error.message : String(error);
  }
  assert.match(
    expectedFailure,
    /^bundle:out\/bundle\.css byte drift: baseline=[0-9a-f]{64} HEAD=[0-9a-f]{64}$/u,
    "output-drift mutation did not fire the byte comparator",
  );
  return { command, relativePath, expectedFailure };
}

function assertByteIdentity(label: string, baseline: Buffer, head: Buffer): void {
  assert.ok(
    baseline.equals(head),
    `${label} byte drift: baseline=${sha256(baseline)} HEAD=${sha256(head)}`,
  );
}

function productCommandIds(): ProductCommandId[] {
  return ["format", "minify", "build", "sif", "bundle"];
}

function listRelativeFiles(
  surface: ReturnType<typeof resolveUnmigratedScanRootForScanner>,
  root: string,
  relativeDirectory = "",
): string[] {
  const directory = relativeDirectory ? resolveRelative(root, relativeDirectory) : root;
  const files: string[] = [];
  for (const entry of surface.readdirSync(directory, { withFileTypes: true })) {
    const relativePath = relativeDirectory ? `${relativeDirectory}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      files.push(...listRelativeFiles(surface, root, relativePath));
    } else {
      assert.ok(entry.isFile(), `unexpected non-file output ${relativePath}`);
      files.push(relativePath);
    }
  }
  return files.toSorted();
}

function resolveRelative(root: string, relativePath: string): string {
  assertSafeRelativePath(relativePath);
  return path.join(root, ...relativePath.split("/"));
}

function assertSafeRelativePath(relativePath: string): void {
  assert.ok(relativePath.length > 0, "relative path must not be empty");
  assert.equal(
    path.posix.normalize(relativePath),
    relativePath,
    `path is not normalized: ${relativePath}`,
  );
  assert.ok(!path.posix.isAbsolute(relativePath), `path must be relative: ${relativePath}`);
  assert.ok(!relativePath.split("/").includes(".."), `path escapes its root: ${relativePath}`);
}

function cargoEnvironment(cargoTargetDir: string): NodeJS.ProcessEnv {
  return deterministicEnvironment({
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: cargoTargetDir,
    CARGO_TERM_COLOR: "never",
  });
}

function deterministicEnvironment(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  return {
    ...process.env,
    CI: "1",
    LANG: "C",
    LC_ALL: "C",
    NO_COLOR: "1",
    SOURCE_DATE_EPOCH: "0",
    TERM: "dumb",
    TZ: "UTC",
    ...extra,
  };
}

function runChecked(
  label: string,
  command: string,
  args: readonly string[],
  options: { readonly cwd?: string; readonly env?: NodeJS.ProcessEnv } = {},
): CommandRunResult {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd ?? REPO_ROOT,
    env: options.env ?? deterministicEnvironment(),
    encoding: "utf8",
    maxBuffer: MAX_BUFFER,
  });
  if (result.error) throw result.error;
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  assert.equal(
    result.status,
    0,
    `${label} failed (status=${result.status})\nstdout=${stdout}\nstderr=${stderr}`,
  );
  return { stdout, stderr };
}

function sha256(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}
