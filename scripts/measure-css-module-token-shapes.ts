import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import {
  resolveScanSurfaceForScanner,
  resolveUnmigratedScanRootForScanner,
} from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { formatGeneratedJson } from "./generated-json";

interface Definition {
  readonly modulePath: string;
  readonly rawName: string;
}

interface BundledModule {
  readonly modulePath: string;
  readonly css: string;
  readonly definitions: readonly {
    readonly rawName: string;
    readonly ordinal: number;
  }[];
}

interface InterfaceManifest {
  readonly moduleCount: number;
  readonly classExportCount: number;
  readonly modules: readonly {
    readonly stylePath: string;
    readonly classExports: readonly { readonly name: string }[];
  }[];
}

type ShapeName =
  | "moduleAndClassHash"
  | "modulePathHash"
  | "modulePathHashWithOrdinal"
  | "escapeFaithfulModuleHash";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outputPath = path.join(repoRoot, "rust/omena-css-module-token-shape-measurement.json");
const args = parseArgs(process.argv.slice(2));
const inRepoSurface = resolveScanSurfaceForScanner(import.meta.url, repoRoot);
const corpusSurface = resolveUnmigratedScanRootForScanner(
  import.meta.url,
  "external-checkout",
  repoRoot,
  args.corpusRoot,
);
const emitterSourceRevision = run("git", ["rev-parse", "HEAD"]).trim();
const emitterSourceStatus = run("git", [
  "status",
  "--porcelain",
  "--",
  "rust/Cargo.toml",
  "rust/Cargo.lock",
  "rust/crates",
]).trim();
assert.equal(
  emitterSourceStatus,
  "",
  `the measured Rust emitter must be clean: ${emitterSourceStatus}`,
);
const measurementCommand = [
  "node",
  "--import",
  "tsx",
  "scripts/measure-css-module-token-shapes.ts",
  ...process.argv.slice(2),
].join(" ");
const corpusPin = run("git", ["-C", args.corpusRoot, "rev-parse", "HEAD"]).trim();
const corpusRepository = run("git", ["-C", args.corpusRoot, "remote", "get-url", "origin"]).trim();
const identityCorpora = args.identityManifests.map(({ label, manifestPath }) =>
  measureManifestProvenance(label, manifestPath),
);
const corpusFiles = corpusSurface
  .gitOutput(["ls-files", "-z", "*.module.css"])
  .split("\0")
  .filter(Boolean)
  .toSorted();
assert.ok(corpusFiles.length > 0, "the size corpus has no tracked .module.css files");

run("cargo", ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli", "--bin", "omena"]);
run("cargo", [
  "build",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-parser",
  "--bin",
  "omena-parser-style-facts",
]);
const omenaBinary = path.join(repoRoot, "rust/target/debug/omena");
const factsBinary = path.join(repoRoot, "rust/target/debug/omena-parser-style-facts");
const bundledModules = corpusFiles.map(bundleModule);
const baseline = bundledModules.map(({ css }) => css).join("");
const baselineBytes = byteMetrics(baseline);

const hashWidth = 6;
const shapeExpectation =
  "Per-definition hashes are expected to cost more gzip bytes than per-module hashes and both measured reference shapes because they remove within-module prefix sharing.";
const shaped = Object.fromEntries(
  (
    [
      "moduleAndClassHash",
      "modulePathHash",
      "modulePathHashWithOrdinal",
      "escapeFaithfulModuleHash",
    ] as const
  ).map((shape) => [shape, measureShape(shape, hashWidth)]),
) as Record<ShapeName, ReturnType<typeof measureShape>>;
assert.ok(
  shaped.moduleAndClassHash.gzipBytes > shaped.modulePathHash.gzipBytes,
  "the pre-stated per-definition hash size expectation did not hold",
);

const hashLengthCurve = [4, 6, 8].map((width) => ({
  width,
  moduleAndClassHash: measureShape("moduleAndClassHash", width),
  modulePathHash: measureShape("modulePathHash", width),
}));
const trackedWorkspaceDefinitions = collectTrackedWorkspaceDefinitions();
const externalDefinitions = args.identityManifests.flatMap(({ label, manifestPath }) =>
  collectManifestDefinitions(label, manifestPath),
);
const identityPopulation = uniqueDefinitions([
  ...trackedWorkspaceDefinitions,
  ...externalDefinitions,
]);
const identityModules = [
  ...new Set([
    ...collectTrackedWorkspaceModulePaths(),
    ...args.identityManifests.flatMap(({ label, manifestPath }) =>
      collectManifestModulePaths(label, manifestPath),
    ),
  ]),
].toSorted();

const collisionWidth = 2;
const engineeredModulePairs = {
  moduleAndClassHash: findPrefixSharingModules(collisionWidth, true),
  modulePathHash: findPrefixSharingModules(collisionWidth, false),
};
const engineeredFailures = {
  moduleAndClassHash: captureCollision(
    "moduleAndClassHash",
    engineeredModulePairs.moduleAndClassHash.map((modulePath) => ({
      modulePath,
      rawName: "shared",
    })),
    collisionWidth,
  ),
  modulePathHash: captureCollision(
    "modulePathHash",
    engineeredModulePairs.modulePathHash.map((modulePath) => ({
      modulePath,
      rawName: "shared",
    })),
    collisionWidth,
  ),
};
assertInjective("moduleAndClassHash", identityPopulation, hashWidth);
assertInjective("modulePathHash", identityPopulation, hashWidth);
assertModuleHashPrefixInjective(identityModules, hashWidth);

const stabilityExpectation =
  "Both ordinal-free shapes are expected to preserve existing tokens after front insertion and sibling-module addition, while file rename and a changed workspace-root boundary rotate path-derived identities.";
const stability = {
  moduleAndClassHash: measureStability("moduleAndClassHash", hashWidth),
  modulePathHash: measureStability("modulePathHash", hashWidth),
};

const rootInputs = measureRootInputs();
const normalizer = measureNormalizer();
const result = {
  schemaVersion: "0",
  product: "omena-css-module-token-shape-measurement",
  method: {
    corpus: "facebook/docusaurus",
    corpusRepository,
    corpusPin,
    moduleFileCount: corpusFiles.length,
    emitter: "rust/target/debug/omena bundle, one tracked module per invocation",
    emitterSourceRevision,
    emitterSourceDirtyPathCount: 0,
    gzipLevel: 9,
    hash: "sha256-base64url-prefix",
    hashWidth,
    identityCorpora,
    reproduction: {
      cwd: "repository-root",
      command: measurementCommand,
    },
  },
  baseline: baselineBytes,
  size: {
    preStatedExpectation: shapeExpectation,
    expectationHeld: true,
    shapes: shaped,
    hashLengthCurve,
  },
  stability: {
    preStatedExpectation: stabilityExpectation,
    shapes: stability,
  },
  identityInputs: {
    rootInputs,
    normalizer,
    configurationHashExcluded: true,
    filesystemDiscoveryAllowed: false,
    outsideRootDisposition: "error",
  },
  collisionDetection: {
    collisionWidth,
    engineeredPairs: engineeredModulePairs,
    engineeredFailures,
    selectedWidth: hashWidth,
    selectedWidthPopulation: {
      moduleCount: identityModules.length,
      definitionCount: identityPopulation.length,
      moduleAndClassHashCollisionCount: 0,
      modulePathHashCollisionCount: 0,
      modulePathHashPrefixCollisionCount: 0,
    },
  },
  prerequisites: {
    trailingHexEscape:
      "Moot for both measured shapes because their sanitized local segment cannot contain a backslash; it reactivates for an escape-faithful local segment.",
    encoderDecoder: "No encoder or decoder plane is introduced by either measured shape.",
    fourSymbolGeneralization:
      "The selected identity must not structurally preclude later keyframe, value, or custom-property use; none is implemented by this measurement.",
  },
};

void emitResult();

async function emitResult(): Promise<void> {
  const serializedResult = await formatGeneratedJson(outputPath, result);
  if (args.write) writeFileSync(outputPath, serializedResult);
  process.stdout.write(serializedResult);
}

function bundleModule(relativePath: string): BundledModule {
  const css = run(omenaBinary, ["bundle", relativePath], args.corpusRoot);
  const definitions = [...css.matchAll(/\b_([A-Za-z][A-Za-z0-9-]*)_([0-9]+)\b/gu)].map((match) => ({
    rawName: match[1],
    ordinal: Number(match[2]),
  }));
  return {
    modulePath: relativePath.replaceAll("\\", "/"),
    css,
    definitions,
  };
}

function measureShape(
  shape: ShapeName,
  width: number,
): {
  readonly rawBytes: number;
  readonly gzipBytes: number;
  readonly rawPercent: number;
  readonly gzipPercent: number;
} {
  const css = bundledModules
    .map((module) =>
      module.css.replace(
        /\b_([A-Za-z][A-Za-z0-9-]*)_([0-9]+)\b/gu,
        (_token, rawName: string, ordinal: string) =>
          tokenFor(shape, module.modulePath, rawName, Number(ordinal), width),
      ),
    )
    .join("");
  const metrics = byteMetrics(css);
  return {
    ...metrics,
    rawPercent: percent(metrics.rawBytes, baselineBytes.rawBytes),
    gzipPercent: percent(metrics.gzipBytes, baselineBytes.gzipBytes),
  };
}

function tokenFor(
  shape: ShapeName,
  modulePath: string,
  rawName: string,
  ordinal: number,
  width: number,
): string {
  const sanitized = sanitize(rawName);
  switch (shape) {
    case "moduleAndClassHash":
      return `_${hashPrefix(`${modulePath}\0${rawName}`, width)}_${sanitized}`;
    case "modulePathHash":
      return `_${hashPrefix(modulePath, width)}_${sanitized}`;
    case "modulePathHashWithOrdinal":
      return `_${hashPrefix(modulePath, width)}_${sanitized}_${ordinal}`;
    case "escapeFaithfulModuleHash":
      return `_${hashPrefix(modulePath, width)}_${rawName}`;
  }
}

function byteMetrics(css: string): { readonly rawBytes: number; readonly gzipBytes: number } {
  return {
    rawBytes: Buffer.byteLength(css),
    gzipBytes: gzipSync(css, { level: 9 }).byteLength,
  };
}

function percent(value: number, baselineValue: number): number {
  return Number((((value - baselineValue) / baselineValue) * 100).toFixed(2));
}

function hashPrefix(input: string, width: number): string {
  return createHash("sha256").update(input).digest("base64url").slice(0, width);
}

function sanitize(rawName: string): string {
  const sanitized = [...rawName]
    .map((character) => (/[A-Za-z0-9_-]/u.test(character) ? character : "_"))
    .join("");
  return sanitized || "class";
}

function measureStability(
  shape: Extract<ShapeName, "moduleAndClassHash" | "modulePathHash">,
  width: number,
) {
  const sample = bundledModules.find((module) => module.definitions.length > 1);
  assert.ok(sample, "stability corpus needs a module with at least two definitions");
  const baselineTokens = sample.definitions.map(({ rawName, ordinal }) =>
    tokenFor(shape, sample.modulePath, rawName, ordinal, width),
  );
  const frontInsertionTokens = sample.definitions.map(({ rawName, ordinal }) =>
    tokenFor(shape, sample.modulePath, rawName, ordinal + 1, width),
  );
  const renamedPath = `${sample.modulePath}.renamed`;
  const renamedTokens = sample.definitions.map(({ rawName, ordinal }) =>
    tokenFor(shape, renamedPath, rawName, ordinal, width),
  );
  const changedRootBoundaryPath = `workspace/${sample.modulePath}`;
  const rootBoundaryTokens = sample.definitions.map(({ rawName, ordinal }) =>
    tokenFor(shape, changedRootBoundaryPath, rawName, ordinal, width),
  );
  const repeatedTokens = sample.definitions.map(({ rawName, ordinal }) =>
    tokenFor(shape, sample.modulePath, rawName, ordinal, width),
  );
  return {
    measuredModule: sample.modulePath,
    measuredDefinitionCount: sample.definitions.length,
    inFileFrontInsertionChangedExistingTokenCount: differenceCount(
      baselineTokens,
      frontInsertionTokens,
    ),
    siblingModuleAdditionChangedExistingTokenCount: 0,
    fileMoveRenameChangedTokenCount: differenceCount(baselineTokens, renamedTokens),
    workspaceRootBoundaryChangeChangedTokenCount: differenceCount(
      baselineTokens,
      rootBoundaryTokens,
    ),
    repeatedRunChangedTokenCount: differenceCount(baselineTokens, repeatedTokens),
  };
}

function differenceCount(left: readonly string[], right: readonly string[]): number {
  assert.equal(left.length, right.length);
  return left.filter((value, index) => value !== right[index]).length;
}

function collectTrackedWorkspaceDefinitions(): Definition[] {
  const files = inRepoSurface
    .gitOutput([
      "ls-files",
      "-z",
      "*.module.css",
      "*.module.scss",
      "*.module.sass",
      "*.module.less",
    ])
    .split("\0")
    .filter(Boolean);
  return files.flatMap((relativePath) => {
    const dialect = path.extname(relativePath).slice(1);
    const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    const parsed = JSON.parse(
      run(factsBinary, [], repoRoot, JSON.stringify({ styleSource: source, dialect })),
    ) as { readonly classSelectorNames: readonly string[] };
    return [...new Set(parsed.classSelectorNames)].map((rawName) => ({
      modulePath: `in-repo/${relativePath}`,
      rawName,
    }));
  });
}

function collectTrackedWorkspaceModulePaths(): string[] {
  return inRepoSurface
    .gitOutput([
      "ls-files",
      "-z",
      "*.module.css",
      "*.module.scss",
      "*.module.sass",
      "*.module.less",
    ])
    .split("\0")
    .filter(Boolean)
    .map((relativePath) => `in-repo/${relativePath}`);
}

function collectManifestDefinitions(label: string, manifestPath: string): Definition[] {
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as InterfaceManifest;
  assert.equal(manifest.modules.length, manifest.moduleCount);
  const definitions = manifest.modules.flatMap((module) => {
    return module.classExports.map(({ name }) => ({
      modulePath: normalizeManifestModulePath(label, module.stylePath),
      rawName: name,
    }));
  });
  assert.equal(definitions.length, manifest.classExportCount);
  return definitions;
}

function measureManifestProvenance(label: string, manifestPath: string) {
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as InterfaceManifest;
  assert.equal(manifest.modules.length, manifest.moduleCount);
  const repositoryRoot = manifestRepositoryRoot(label, manifest);
  return {
    label,
    repository: run("git", ["-C", repositoryRoot, "remote", "get-url", "origin"]).trim(),
    pin: run("git", ["-C", repositoryRoot, "rev-parse", "HEAD"]).trim(),
    moduleCount: manifest.moduleCount,
    definitionCount: manifest.classExportCount,
    manifestGeneration: {
      cwd: "repository-root",
      command: `rust/target/debug/omena modules emit --interface-file ${manifestPath} ${repositoryRoot}`,
    },
  };
}

function manifestRepositoryRoot(label: string, manifest: InterfaceManifest): string {
  const firstModule = manifest.modules[0];
  assert.ok(firstModule, `${label} manifest is empty`);
  const normalizedPath = firstModule.stylePath.replaceAll("\\", "/");
  const marker = normalizedPath.lastIndexOf(`/${label}/`);
  assert.ok(marker >= 0, `${label} is absent from manifest module paths`);
  const repositoryRoot = normalizedPath.slice(0, marker + label.length + 1);
  assert.ok(
    manifest.modules.every((module) =>
      module.stylePath.replaceAll("\\", "/").startsWith(`${repositoryRoot}/`),
    ),
    `${label} manifest spans more than one repository root`,
  );
  return repositoryRoot;
}

function collectManifestModulePaths(label: string, manifestPath: string): string[] {
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as InterfaceManifest;
  assert.equal(manifest.modules.length, manifest.moduleCount);
  return manifest.modules.map((module) => normalizeManifestModulePath(label, module.stylePath));
}

function normalizeManifestModulePath(label: string, stylePath: string): string {
  const normalizedPath = stylePath.replaceAll("\\", "/");
  const marker = normalizedPath.lastIndexOf(`/${label}/`);
  const relativePath =
    marker >= 0 ? normalizedPath.slice(marker + label.length + 2) : path.basename(normalizedPath);
  return `${label}/${relativePath}`;
}

function uniqueDefinitions(definitions: readonly Definition[]): Definition[] {
  const byKey = new Map<string, Definition>();
  for (const definition of definitions) {
    byKey.set(`${definition.modulePath}\0${definition.rawName}`, definition);
  }
  return [...byKey.values()].toSorted((left, right) =>
    `${left.modulePath}\0${left.rawName}`.localeCompare(`${right.modulePath}\0${right.rawName}`),
  );
}

function assertInjective(
  shape: Extract<ShapeName, "moduleAndClassHash" | "modulePathHash">,
  definitions: readonly Definition[],
  width: number,
): void {
  const ownerByToken = new Map<string, Definition>();
  for (const definition of definitions) {
    const token = tokenFor(shape, definition.modulePath, definition.rawName, 0, width);
    const owner = ownerByToken.get(token);
    if (
      owner &&
      (owner.modulePath !== definition.modulePath || owner.rawName !== definition.rawName)
    ) {
      throw new Error(
        `${shape} collision at width ${width}: ${JSON.stringify(owner)} and ${JSON.stringify(
          definition,
        )} map to ${JSON.stringify(token)}`,
      );
    }
    ownerByToken.set(token, definition);
  }
}

function assertModuleHashPrefixInjective(modulePaths: readonly string[], width: number): void {
  const ownerByPrefix = new Map<string, string>();
  for (const modulePath of modulePaths) {
    const prefix = hashPrefix(modulePath, width);
    const owner = ownerByPrefix.get(prefix);
    if (owner && owner !== modulePath) {
      throw new Error(
        `modulePathHash prefix collision at width ${width}: ${JSON.stringify(
          owner,
        )} and ${JSON.stringify(modulePath)} map to ${JSON.stringify(prefix)}`,
      );
    }
    ownerByPrefix.set(prefix, modulePath);
  }
}

function captureCollision(
  shape: Extract<ShapeName, "moduleAndClassHash" | "modulePathHash">,
  definitions: readonly Definition[],
  width: number,
): string {
  try {
    assertInjective(shape, definitions, width);
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
  throw new Error(`${shape} engineered ${width}-character collision was not detected`);
}

function findPrefixSharingModules(width: number, includeRawName: boolean): [string, string] {
  const ownerByPrefix = new Map<string, string>();
  for (let index = 0; index < 100_000; index += 1) {
    const modulePath = `engineered/module-${index}.module.css`;
    const prefix = hashPrefix(includeRawName ? `${modulePath}\0shared` : modulePath, width);
    const owner = ownerByPrefix.get(prefix);
    if (owner) return [owner, modulePath];
    ownerByPrefix.set(prefix, modulePath);
  }
  throw new Error(`could not engineer a ${width}-character hash-prefix collision`);
}

function measureRootInputs() {
  const surfaces = [
    {
      surface: "cliModules",
      source: "rust/crates/omena-cli/src/modules.rs",
      requiredNeedle: "workspace_root",
      status: "available",
    },
    {
      surface: "lspWorkspaceWorkflow",
      source: "rust/crates/omena-lsp-server/src",
      requiredNeedle: "workspace_root",
      status: "available",
    },
    {
      surface: "napiWorkspaceWorkflow",
      source: "rust/crates/omena-napi/src/sdk_workspace.rs",
      requiredNeedle: "pub fn new(workspace_root: String",
      status: "available",
    },
    {
      surface: "wasmWorkspaceWorkflow",
      source: "rust/crates/omena-wasm/src/sdk_workspace.rs",
      requiredNeedle: "pub fn new(workspace_root: String",
      status: "available",
    },
    {
      surface: "napiDirectBundleApi",
      source: "rust/crates/omena-napi/src/lib.rs",
      requiredNeedle: "pub fn bundle_style_sources_with_context_json(",
      status: "missing-required-root",
      absenceScope: "signature",
    },
    {
      surface: "wasmDirectBundleApi",
      source: "rust/crates/omena-wasm/src/lib.rs",
      requiredNeedle: "pub fn bundle_style_sources_with_context(",
      status: "missing-required-root",
      absenceScope: "signature",
    },
    {
      surface: "rawBundleApi",
      source: "rust/crates/omena-query/src/style/transform.rs",
      requiredNeedle: "pub struct OmenaQueryBundlePlanInputV0",
      status: "missing-required-root",
      absenceScope: "brace-body",
    },
    {
      surface: "syntheticHarness",
      source: "rust/crates/omena-query/src/tests/transform_facade.rs",
      requiredNeedle: "OmenaQueryBundlePlanInputV0 {",
      status: "missing-required-root",
      absenceScope: "brace-body",
    },
  ] as const;
  for (const surface of surfaces) {
    const sourcePath = path.join(repoRoot, surface.source);
    const source = readPathOrDirectory(sourcePath);
    assert.ok(
      source.includes(surface.requiredNeedle),
      `${surface.surface} evidence needle ${surface.requiredNeedle} is missing`,
    );
    if (surface.status === "missing-required-root") {
      assert.ok("absenceScope" in surface);
      const scope = rustRootInputScope(source, surface.requiredNeedle, surface.absenceScope);
      assert.doesNotMatch(
        scope,
        /\bworkspace_?root\b|\bworkspaceRoot\b/u,
        `${surface.surface} unexpectedly gained a workspace-root input`,
      );
    }
  }
  return surfaces.map(({ surface, source, requiredNeedle, status }) => ({
    surface,
    status,
    evidence: {
      source,
      needle: requiredNeedle,
    },
  }));
}

function rustRootInputScope(
  source: string,
  anchor: string,
  scopeKind: "signature" | "brace-body",
): string {
  const start = source.indexOf(anchor);
  assert.ok(start >= 0, `missing Rust root-input scope anchor ${anchor}`);
  const openingBrace = source.indexOf("{", start);
  assert.ok(openingBrace >= 0, `missing Rust scope body after ${anchor}`);
  if (scopeKind === "signature") return source.slice(start, openingBrace);
  let depth = 0;
  for (let offset = openingBrace; offset < source.length; offset += 1) {
    if (source[offset] === "{") depth += 1;
    if (source[offset] === "}") depth -= 1;
    if (depth === 0) return source.slice(start, offset + 1);
  }
  throw new Error(`unterminated Rust scope after ${anchor}`);
}

function measureNormalizer() {
  const source = readFileSync(path.join(repoRoot, "rust/crates/omena-query/src/types.rs"), "utf8");
  assert.ok(source.includes("fn normalize_omena_query_style_path"));
  assert.ok(source.includes("normalize_omena_transform_bundle_path"));
  return {
    source: "normalize_omena_query_style_path",
    visibility: "pub(crate)",
    separatorAndComponentNormalization: true,
    callerRootRelativization: false,
  };
}

function readPathOrDirectory(target: string): string {
  const listed = spawnSync("find", [target, "-type", "f", "-name", "*.rs", "-print0"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (listed.status === 0 && listed.stdout) {
    return listed.stdout
      .split("\0")
      .filter(Boolean)
      .map((file) => readFileSync(file, "utf8"))
      .join("\n");
  }
  return readFileSync(target, "utf8");
}

function run(
  command: string,
  commandArgs: readonly string[],
  cwd = repoRoot,
  stdin?: string,
): string {
  const result = spawnSync(command, commandArgs, {
    cwd,
    input: stdin,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  assert.equal(
    result.status,
    0,
    [command, ...commandArgs, result.stdout, result.stderr].filter(Boolean).join("\n"),
  );
  return result.stdout;
}

function parseArgs(rawArgs: readonly string[]): {
  readonly corpusRoot: string;
  readonly identityManifests: readonly {
    readonly label: string;
    readonly manifestPath: string;
  }[];
  readonly write: boolean;
} {
  let corpusRoot = "";
  const identityManifests: { label: string; manifestPath: string }[] = [];
  let write = false;
  for (let index = 0; index < rawArgs.length; index += 1) {
    const argument = rawArgs[index];
    if (argument === "--corpus-root") {
      corpusRoot = path.resolve(rawArgs[++index] ?? "");
    } else if (argument === "--identity-manifest") {
      const value = rawArgs[++index] ?? "";
      const separator = value.indexOf("=");
      assert.ok(separator > 0, "--identity-manifest expects label=/absolute/path.json");
      identityManifests.push({
        label: value.slice(0, separator),
        manifestPath: path.resolve(value.slice(separator + 1)),
      });
    } else if (argument === "--write") {
      write = true;
    } else {
      throw new Error(`unknown argument ${argument}`);
    }
  }
  assert.ok(corpusRoot, "--corpus-root is required");
  assert.ok(identityManifests.length >= 3, "three external identity manifests are required");
  return { corpusRoot, identityManifests, write };
}
