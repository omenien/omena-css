import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { compilerApi as ts } from "../../../../server/engine-core-ts/src/ts-facade";
import type tsTypes from "../../../../server/engine-core-ts/src/ts-facade";
import { loadCheckManifest } from "../manifest/index";
import {
  formatEvidenceJsonArtifact,
  loadEvidenceScanSurfaceManifest,
} from "./scan-surface-manifest";
import { defineScanSurface, resolveScanSurface } from "./scan-surface";
import {
  EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS,
  EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH,
  EVIDENCE_WRITER_COMMAND_DECLARATIONS,
  EVIDENCE_WRITER_NON_LITERAL_WRITE_REFUSALS,
  EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS,
  EVIDENCE_STATIC_WRITE_OUTPUT_AUTHORITY,
  resolveEvidenceWriterCommandOutputPaths,
  type EvidenceWriterCommandDeclaration,
} from "./writer-command-authority";

export const EVIDENCE_WRITER_REGISTRY_PATH = "rust/evidence-writer-registry.json";
export const EVIDENCE_WRITER_NON_LITERAL_WRITE_CENSUS_PATH =
  "rust/evidence-writer-nonliteral-write-census.json";
const WPT_TIER_ZERO_OUTPUT_PATHS = new Set([
  "rust/crates/omena-diff-test/wpt-corpus/extracted/tier-zero-coverage.json",
  "rust/crates/omena-diff-test/wpt-corpus/extracted/tier-zero-tuples.json",
]);

const ROOT_RUST_ARTIFACT_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/writer-registry.ts",
  mode: "index",
  pathspecs: ["rust"],
  includeUntracked: true,
  excludes: [],
});
const WRITER_SOURCE_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/writer-registry.ts",
  mode: "index",
  pathspecs: ["scripts/**", "packages/check-orchestrator/src/**"],
  includeUntracked: true,
  excludes: [],
});
const REPOSITORY_REFERENCE_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/writer-registry.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: true,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});

export const NOT_PREVIEWABLE_INPUT_KINDS = [
  "calendar-time",
  "environment",
  "git-history",
  "concurrent-worktree",
  "network-or-external-checkout",
  "built-binary",
  "toolchain-bytes",
] as const;

export type NotPreviewableInputKind = (typeof NOT_PREVIEWABLE_INPUT_KINDS)[number];
export type EvidenceArtifactClassification = "W1" | "W2" | "W3" | "W4";

/**
 * Every writable recipe is reproduced with its output removed unless the
 * writer must read that output as an input. Such exceptions are named here
 * with a reviewable reason and projected onto the generated registry row.
 */
const DOCUMENTATION_REFERENCE_SELF_INPUT_PATHS = [
  "docs/reference/README.md",
  "docs/reference/cli.md",
  "docs/reference/personas.md",
  "docs/reference/configuration.md",
  "docs/reference/editor-settings.md",
  "docs/reference/lsp-capabilities.md",
  "rust/crates/omena-cli/README.md",
  "docs/vscode-extension.md",
  "docs/sdk.md",
] as const;

export const EVIDENCE_FRESH_REPRODUCTION_EXEMPTIONS: readonly {
  readonly artifactPath: string;
  readonly kind: "reads-own-output";
  readonly reason: string;
}[] = [
  {
    artifactPath: "packages/check-orchestrator/ci-cost-ledger.json",
    kind: "reads-own-output",
    reason:
      "the reproducibility recipe replays job metadata from committed sourceRunIds and preserves gate samples whose GitHub summary artifacts may have expired",
  },
  {
    artifactPath: "packages/css-build-adapter/interface-member-snapshot.json",
    kind: "reads-own-output",
    reason:
      "the adapter reproduction mode validates current declarations against the committed approval baseline before canonically rewriting those approved bytes",
  },
  {
    artifactPath:
      "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-real-declarations.json",
    kind: "reads-own-output",
    reason:
      "the corpus writer preserves reviewer-authored adjudication, reason, owner, specification URL, and non-comparability fields from matching committed cases",
  },
  ...[
    "rust/crates/omena-abstract-value/data/closed-world-builtin-token-profiles.json",
    "rust/crates/omena-abstract-value/data/closed-world-keyword-closure-certificate.json",
  ].map((artifactPath) => ({
    artifactPath,
    kind: "reads-own-output" as const,
    reason:
      "the shared differential writer compiles omena-abstract-value, whose value-grammar module includes this generated JSON before the command recomputes and rewrites it",
  })),
  {
    artifactPath: "rust/evidence-writer-registry.json",
    kind: "reads-own-output",
    reason:
      "the registry builder validates every toolchain-byte owner against the repository path set, including the registry output itself",
  },
  {
    artifactPath: EVIDENCE_WRITER_NON_LITERAL_WRITE_CENSUS_PATH,
    kind: "reads-own-output",
    reason:
      "the official updater reads the committed census before writing so a newly introduced non-literal site cannot be adopted by regeneration",
  },
  ...DOCUMENTATION_REFERENCE_SELF_INPUT_PATHS.map(
    (
      artifactPath,
    ): {
      readonly artifactPath: string;
      readonly kind: "reads-own-output";
      readonly reason: string;
    } => ({
      artifactPath,
      kind: "reads-own-output",
      reason:
        "the documentation writer validates its committed public-document corpus before regenerating the command's full output set",
    }),
  ),
  {
    artifactPath: "rust/omena-identifier-authority-census.json",
    kind: "reads-own-output",
    reason:
      "the identifier census compares discovered sites with its committed shrink-only adoption policy before regeneration",
  },
];

export interface EvidenceWriterRegistryV0 {
  readonly schemaVersion: "0";
  readonly generatedBy: "pnpm omena-check evidence-writers --write";
  readonly commandAuthority: {
    readonly modulePath: typeof EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH;
    readonly sha256: string;
    readonly declaredCommandCount: number;
    readonly declaredOutputCount: number;
    readonly declaredStaticWriteOutputCount: number;
  };
  readonly artifacts: readonly EvidenceArtifactRowV0[];
  readonly notPreviewableInputs: readonly NotPreviewableInputV0[];
}

export interface EvidenceWriterNonLiteralWriteCensusV0 {
  readonly schemaVersion: "0";
  readonly generatedBy: "pnpm omena-check evidence-writers --write";
  readonly scope: {
    readonly pathspecs: readonly ["scripts/**", "packages/check-orchestrator/src/**"];
    readonly scannedScriptCount: number;
    readonly registryWriterScriptCount: number;
    readonly declaredWriterScriptCount: number;
    readonly broadWriterScriptCountWithNonLiteralSites: number;
    readonly broadNonLiteralWriteSiteCount: number;
    readonly unsweptRegistryWriterScriptCount: number;
    readonly unsweptRegistryWriterScriptsWithNonLiteralSites: number;
    readonly unsweptRegistryNonLiteralWriteSiteCount: number;
  };
  readonly reviewerBaseline: {
    readonly pin: "ba7308df84bf501b6cf3a13347dcf47571b0eb61";
    readonly broadWriterScriptCountWithNonLiteralSites: 122;
    readonly broadNonLiteralWriteSiteCount: 319;
    readonly unsweptRegistryWriterScriptsWithNonLiteralSites: 44;
  };
  readonly runtimeVariableOutputPaths: typeof EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS;
  readonly typedJustifications: typeof EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS;
  readonly sites: readonly EvidenceWriterNonLiteralWriteSiteV0[];
}

export interface EvidenceWriterNonLiteralWriteSiteV0 {
  readonly fingerprint: string;
  readonly writerScript: string;
  readonly line: number;
  readonly writeApi: string;
  readonly writeExpression: string;
  readonly disposition:
    | "declared-writer-sweep"
    | "unswept-registry-writer"
    | "outside-registry-writer";
}

export interface EvidenceArtifactRowV0 {
  readonly artifactPath: string;
  readonly classification: EvidenceArtifactClassification;
  readonly writerNodeKind: "normal" | "self-ratchet";
  readonly freshReproductionRequired?: true;
  readonly freshReproductionExemption?: {
    readonly kind: "reads-own-output";
    readonly reason: string;
  };
  readonly writerScripts: readonly string[];
  readonly writeCommand?: readonly string[];
  readonly alternateWriteCommands?: readonly (readonly string[])[];
  readonly requiredEnvironmentKeys?: readonly string[];
  readonly procedure?: readonly string[];
  readonly disposition?: "reviewer-retire-proposal";
  readonly inputPaths: readonly string[];
  readonly inputScannerPaths: readonly string[];
  readonly inputArtifactPaths: readonly string[];
  readonly writerGateIds: readonly string[];
  readonly consumerPaths: readonly string[];
  readonly consumerGateIds: readonly string[];
}

export interface NotPreviewableInputV0 {
  readonly ownerId: string;
  readonly kind: NotPreviewableInputKind;
  readonly detail: string;
  readonly gateIds: readonly string[];
}

export function evidenceWriterRegistryPath(repoRoot: string): string {
  return path.join(repoRoot, EVIDENCE_WRITER_REGISTRY_PATH);
}

export function loadEvidenceWriterRegistry(repoRoot: string): EvidenceWriterRegistryV0 | null {
  const registryPath = evidenceWriterRegistryPath(repoRoot);
  if (!existsSync(registryPath)) return null;
  const registry = JSON.parse(readFileSync(registryPath, "utf8")) as EvidenceWriterRegistryV0;
  validateEvidenceWriterRegistry(registry);
  return registry;
}

export function renderEvidenceWriterRegistry(registry: EvidenceWriterRegistryV0): string {
  validateEvidenceWriterRegistry(registry);
  return formatEvidenceJsonArtifact(registry, EVIDENCE_WRITER_REGISTRY_PATH);
}

export function evidenceWriterNonLiteralWriteCensusPath(repoRoot: string): string {
  return path.join(repoRoot, EVIDENCE_WRITER_NON_LITERAL_WRITE_CENSUS_PATH);
}

export function loadEvidenceWriterNonLiteralWriteCensus(
  repoRoot: string,
  requireCurrentAuthority = true,
): EvidenceWriterNonLiteralWriteCensusV0 | null {
  const censusPath = evidenceWriterNonLiteralWriteCensusPath(repoRoot);
  if (!existsSync(censusPath)) return null;
  const census = JSON.parse(
    readFileSync(censusPath, "utf8"),
  ) as EvidenceWriterNonLiteralWriteCensusV0;
  validateEvidenceWriterNonLiteralWriteCensus(census, requireCurrentAuthority);
  return census;
}

export function renderEvidenceWriterNonLiteralWriteCensus(
  census: EvidenceWriterNonLiteralWriteCensusV0,
): string {
  validateEvidenceWriterNonLiteralWriteCensus(census);
  return formatEvidenceJsonArtifact(census, EVIDENCE_WRITER_NON_LITERAL_WRITE_CENSUS_PATH);
}

const EVIDENCE_NON_LITERAL_WRITE_APIS = new Set([
  "appendFileSync",
  "copyFileSync",
  "cpSync",
  "createWriteStream",
  "linkSync",
  "outputFileSync",
  "renameSync",
  "symlinkSync",
  "writeFileSync",
  "writeSync",
]);

export function buildEvidenceWriterNonLiteralWriteCensus(
  repoRoot: string,
  registry: EvidenceWriterRegistryV0,
): EvidenceWriterNonLiteralWriteCensusV0 {
  const scriptPaths = resolveScanSurface(WRITER_SOURCE_SURFACE, { repoRoot }).paths.filter(
    (candidate) => /\.(?:[cm]?js|ts)$/u.test(candidate),
  );
  const registryWriterScripts = new Set<string>(
    registry.artifacts.flatMap((row) => row.writerScripts),
  );
  const declaredWriterScripts = new Set<string>(
    EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((declaration) => declaration.writerScripts),
  );
  const unsweptRegistryWriterScripts = new Set<string>(
    [...registryWriterScripts].filter((script) => !declaredWriterScripts.has(script)),
  );
  const sites: EvidenceWriterNonLiteralWriteSiteV0[] = [];
  const occurrenceBySiteShape = new Map<string, number>();
  for (const scriptPath of scriptPaths) {
    const source = readFileSync(path.join(repoRoot, scriptPath), "utf8");
    const sourceFile = ts.createSourceFile(scriptPath, source, ts.ScriptTarget.Latest, true);
    const visit = (node: tsTypes.Node): void => {
      if (ts.isCallExpression(node)) {
        const writeApi = callName(node.expression);
        const firstArgument = node.arguments[0];
        if (
          writeApi &&
          EVIDENCE_NON_LITERAL_WRITE_APIS.has(writeApi) &&
          firstArgument &&
          !ts.isStringLiteral(firstArgument) &&
          !ts.isNoSubstitutionTemplateLiteral(firstArgument)
        ) {
          const writeExpression = normalizeWriteExpression(firstArgument.getText(sourceFile));
          const siteShape = `${scriptPath}\0${writeApi}\0${writeExpression}`;
          const occurrence = (occurrenceBySiteShape.get(siteShape) ?? 0) + 1;
          occurrenceBySiteShape.set(siteShape, occurrence);
          sites.push({
            fingerprint: evidenceNonLiteralWriteSiteFingerprint(siteShape, occurrence),
            writerScript: scriptPath,
            line: sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
            writeApi,
            writeExpression,
            disposition: declaredWriterScripts.has(scriptPath)
              ? "declared-writer-sweep"
              : unsweptRegistryWriterScripts.has(scriptPath)
                ? "unswept-registry-writer"
                : "outside-registry-writer",
          });
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
  }
  sites.sort((left, right) =>
    compareText(
      `${left.writerScript}:${String(left.line).padStart(8, "0")}:${left.writeApi}:${left.writeExpression}`,
      `${right.writerScript}:${String(right.line).padStart(8, "0")}:${right.writeApi}:${right.writeExpression}`,
    ),
  );
  const unsweptSites = sites.filter((site) => site.disposition === "unswept-registry-writer");
  const census = {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-writers --write",
    scope: {
      pathspecs: ["scripts/**", "packages/check-orchestrator/src/**"],
      scannedScriptCount: scriptPaths.length,
      registryWriterScriptCount: registryWriterScripts.size,
      declaredWriterScriptCount: declaredWriterScripts.size,
      broadWriterScriptCountWithNonLiteralSites: new Set(sites.map((site) => site.writerScript))
        .size,
      broadNonLiteralWriteSiteCount: sites.length,
      unsweptRegistryWriterScriptCount: unsweptRegistryWriterScripts.size,
      unsweptRegistryWriterScriptsWithNonLiteralSites: new Set(
        unsweptSites.map((site) => site.writerScript),
      ).size,
      unsweptRegistryNonLiteralWriteSiteCount: unsweptSites.length,
    },
    reviewerBaseline: {
      pin: "ba7308df84bf501b6cf3a13347dcf47571b0eb61",
      broadWriterScriptCountWithNonLiteralSites: 122,
      broadNonLiteralWriteSiteCount: 319,
      unsweptRegistryWriterScriptsWithNonLiteralSites: 44,
    },
    runtimeVariableOutputPaths: EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS,
    typedJustifications: EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS,
    sites,
  } as const satisfies EvidenceWriterNonLiteralWriteCensusV0;
  validateEvidenceWriterNonLiteralWriteCensus(census);
  const runtimeJustifications = EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS.filter(
    (entry) => entry.kind === "runtime-variable-output",
  );
  for (const justification of runtimeJustifications) {
    const row = registry.artifacts.find(
      (artifact) => artifact.artifactPath === justification.outputPath,
    );
    if (
      row?.classification !== "W2" ||
      !row.writerScripts.includes(justification.writerScript) ||
      !sites.some(
        (site) =>
          site.writerScript === justification.writerScript &&
          site.writeExpression === justification.writeExpression,
      )
    ) {
      throw new Error(
        `runtime-variable output justification is not bound to its W2 write site: ${justification.outputPath}`,
      );
    }
  }
  return census;
}

export function assertEvidenceWriterNonLiteralWriteCensusDecreaseOnly(
  previous: EvidenceWriterNonLiteralWriteCensusV0 | null,
  next: EvidenceWriterNonLiteralWriteCensusV0,
): void {
  validateEvidenceWriterNonLiteralWriteCensus(next);
  if (!previous) {
    throw new Error("committed evidence writer non-literal census is required before --write");
  }
  validateEvidenceWriterNonLiteralWriteCensus(previous, false);
  const previousFingerprints = new Set(previous.sites.map((site) => site.fingerprint));
  const introduced = next.sites.find((site) => !previousFingerprints.has(site.fingerprint));
  if (introduced) {
    throw new Error(
      `decrease-only evidence writer census refuses a new non-literal write site: ${introduced.writerScript}:${introduced.line}:${introduced.writeExpression}`,
    );
  }
  if (
    next.scope.broadNonLiteralWriteSiteCount > previous.scope.broadNonLiteralWriteSiteCount ||
    next.scope.broadWriterScriptCountWithNonLiteralSites >
      previous.scope.broadWriterScriptCountWithNonLiteralSites ||
    next.scope.unsweptRegistryWriterScriptsWithNonLiteralSites >
      previous.scope.unsweptRegistryWriterScriptsWithNonLiteralSites ||
    next.scope.unsweptRegistryNonLiteralWriteSiteCount >
      previous.scope.unsweptRegistryNonLiteralWriteSiteCount
  ) {
    throw new Error("evidence writer non-literal census is not decrease-only");
  }
}

function validateEvidenceWriterNonLiteralWriteCensus(
  census: EvidenceWriterNonLiteralWriteCensusV0,
  requireCurrentAuthority = true,
): void {
  if (
    census.schemaVersion !== "0" ||
    census.generatedBy !== "pnpm omena-check evidence-writers --write"
  ) {
    throw new Error("invalid evidence writer non-literal census header");
  }
  if (census.scope.broadNonLiteralWriteSiteCount !== census.sites.length) {
    throw new Error("evidence writer non-literal census site count drifted");
  }
  if (
    requireCurrentAuthority &&
    JSON.stringify(census.runtimeVariableOutputPaths) !==
      JSON.stringify(EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS)
  ) {
    throw new Error("evidence writer non-literal census runtime-variable outputs drifted");
  }
  if (
    requireCurrentAuthority &&
    JSON.stringify(census.typedJustifications) !==
      JSON.stringify(EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS)
  ) {
    throw new Error("evidence writer non-literal census typed justifications drifted");
  }
  const fingerprints = new Set<string>();
  const occurrenceBySiteShape = new Map<string, number>();
  for (const site of census.sites) {
    const siteShape = `${site.writerScript}\0${site.writeApi}\0${site.writeExpression}`;
    const occurrence = (occurrenceBySiteShape.get(siteShape) ?? 0) + 1;
    occurrenceBySiteShape.set(siteShape, occurrence);
    const expectedFingerprint = evidenceNonLiteralWriteSiteFingerprint(siteShape, occurrence);
    if (site.fingerprint !== expectedFingerprint || fingerprints.has(site.fingerprint)) {
      throw new Error(
        `evidence writer non-literal census fingerprint drifted: ${site.writerScript}:${site.line}`,
      );
    }
    fingerprints.add(site.fingerprint);
  }
}

function evidenceNonLiteralWriteSiteFingerprint(siteShape: string, occurrence: number): string {
  return createHash("sha256").update(`${siteShape}\0${occurrence}`).digest("hex");
}

export function buildEvidenceWriterRegistry(repoRoot: string): EvidenceWriterRegistryV0 {
  assertGovernedNonLiteralWriteCoverage(repoRoot);
  const discovery = discoverEvidenceArtifacts(repoRoot);
  const { repositoryPaths, scriptPaths, artifactPaths, staticWriterScriptsByOutput } = discovery;
  assertEvidenceWriterDeclarationWriteWitnessCoverage(repoRoot, discovery);
  const referencePathsByArtifact = repositoryReferencePathsByArtifact(
    repoRoot,
    artifactPaths,
    repositoryPaths,
  );
  const packageScripts = readPackageScripts(repoRoot);
  const checkManifest = loadCheckManifest(repoRoot);
  const scanManifest = loadEvidenceScanSurfaceManifest(repoRoot);
  if (!scanManifest)
    throw new Error("evidence scan surface manifest is required before writer census");
  const scannerGateIds = new Map(
    scanManifest.scanners.flatMap((row) =>
      row.disposition === "RETIRED"
        ? []
        : [[row.scannerPath, "gateIds" in row ? row.gateIds : []] as const],
    ),
  );
  const directWriterScriptsByArtifact = new Map(
    artifactPaths.map((artifactPath) => [
      artifactPath,
      [
        ...new Set([
          ...(staticWriterScriptsByOutput.get(artifactPath) ?? []),
          ...declaredCommandsForOutput(repoRoot, artifactPath).flatMap((row) => row.writerScripts),
        ]),
      ].toSorted(),
    ]),
  );
  const updateCommandValues = Object.entries(packageScripts)
    .filter(([scriptName]) => scriptName.startsWith("update:"))
    .map(([, command]) => command);
  const scriptImportersByModule = buildScriptImporters(repoRoot, scriptPaths);
  const writerScriptsByArtifact = new Map(
    [...directWriterScriptsByArtifact].map(([artifactPath, directWriterScripts]) => [
      artifactPath,
      expandWriterScriptsWithEntrypoints(
        directWriterScripts,
        scriptImportersByModule,
        updateCommandValues,
      ),
    ]),
  );
  const writerOutputPathsByScript = new Map<string, string[]>();
  for (const [artifactPath, writerScripts] of writerScriptsByArtifact) {
    for (const writerScript of writerScripts) {
      writerOutputPathsByScript.set(writerScript, [
        ...(writerOutputPathsByScript.get(writerScript) ?? []),
        artifactPath,
      ]);
    }
  }
  const repositoryPathSet = new Set(repositoryPaths);
  const artifactPathSet = new Set(artifactPaths);

  const artifacts = artifactPaths.map((artifactPath) => {
    const writerScripts = writerScriptsByArtifact.get(artifactPath) ?? [];
    const updateCommands = Object.entries(packageScripts)
      .flatMap(([scriptName, command]) => {
        if (!scriptName.startsWith("update:")) return [];
        const writerScript = writerScripts.find((candidate) => command.includes(candidate));
        return writerScript ? [{ scriptName, command, writerScript }] : [];
      })
      .toSorted((left, right) => compareText(left.scriptName, right.scriptName));
    const references = referencePathsByArtifact.get(artifactPath) ?? [];
    return classifyArtifact({
      repoRoot,
      artifactPath,
      writerScripts,
      updateCommands,
      references,
      scannerGateIds,
      checkManifest,
      repositoryPathSet,
      artifactPathSet,
      writerOutputPathsByScript,
    });
  });
  const registry = {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-writers --write",
    commandAuthority: {
      modulePath: EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH,
      sha256: createHash("sha256")
        .update(readFileSync(path.join(repoRoot, EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH)))
        .digest("hex"),
      declaredCommandCount: EVIDENCE_WRITER_COMMAND_DECLARATIONS.length,
      declaredOutputCount: new Set(
        EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((row) =>
          resolveEvidenceWriterCommandOutputPaths(repoRoot, row),
        ),
      ).size,
      declaredStaticWriteOutputCount: EVIDENCE_STATIC_WRITE_OUTPUT_AUTHORITY.length,
    },
    artifacts,
    notPreviewableInputs: deriveNotPreviewableInputs({
      repoRoot,
      artifacts,
      scanManifest,
      checkManifest,
      repositoryPathSet,
    }),
  } satisfies EvidenceWriterRegistryV0;
  assertUpdateCommandWriterCoverage(packageScripts, registry.artifacts);
  assertEvidenceWriterAuthorityCoverage(registry, EVIDENCE_WRITER_COMMAND_DECLARATIONS, repoRoot);
  assertEvidenceWriterOutputAuthorityCoverage(registry, {
    rootArtifactPaths: discovery.rootArtifactPaths,
    writeCallOutputPaths: discovery.writeCallOutputPaths,
    declaredCommandOutputPaths: discovery.declaredCommandOutputPaths,
    staticWriteOutputPaths: EVIDENCE_STATIC_WRITE_OUTPUT_AUTHORITY,
  });
  for (const input of registry.notPreviewableInputs) {
    if (!existsSync(path.join(repoRoot, input.ownerId))) {
      throw new Error(`not-previewable owner module is missing: ${input.ownerId}`);
    }
  }
  return registry;
}

export function discoverEvidenceArtifactPaths(repoRoot: string): readonly string[] {
  return discoverEvidenceArtifacts(repoRoot).artifactPaths;
}

export interface EvidenceWriterOutputAuthority {
  readonly rootArtifactPaths: readonly string[];
  readonly writeCallOutputPaths: readonly string[];
  readonly declaredCommandOutputPaths: readonly string[];
  readonly staticWriteOutputPaths: readonly string[];
}

export function discoverEvidenceWriterOutputAuthority(
  repoRoot: string,
): EvidenceWriterOutputAuthority {
  const discovery = discoverEvidenceArtifacts(repoRoot);
  return {
    rootArtifactPaths: discovery.rootArtifactPaths,
    writeCallOutputPaths: discovery.writeCallOutputPaths,
    declaredCommandOutputPaths: discovery.declaredCommandOutputPaths,
    staticWriteOutputPaths: EVIDENCE_STATIC_WRITE_OUTPUT_AUTHORITY,
  };
}

interface WriterScriptAnalysis {
  readonly source: string;
  readonly sourceFile: tsTypes.SourceFile;
  readonly staticPathVariables: ReadonlyMap<string, string>;
  readonly lexical: StaticPathLexicalIndex;
  readonly lexicalStaticPathValues: ReadonlyMap<tsTypes.Identifier, string>;
}

interface StaticPathLexicalIndex {
  readonly declarationFor: (identifier: tsTypes.Identifier) => tsTypes.Identifier | null;
}

interface EvidenceArtifactDiscovery {
  readonly repositoryPaths: readonly string[];
  readonly scriptPaths: readonly string[];
  readonly artifactPaths: readonly string[];
  readonly rootArtifactPaths: readonly string[];
  readonly writeCallOutputPaths: readonly string[];
  readonly declaredCommandOutputPaths: readonly string[];
  readonly staticWriterScriptsByOutput: ReadonlyMap<string, readonly string[]>;
  readonly scriptAnalysisCache: Map<string, WriterScriptAnalysis>;
}

export interface UnclassifiedGovernedNonLiteralWriteSite {
  readonly writerScript: string;
  readonly line: number;
  readonly writeExpression: string;
}

export function findUnclassifiedGovernedNonLiteralWriteSites(
  repoRoot: string,
): readonly UnclassifiedGovernedNonLiteralWriteSite[] {
  const repositoryPaths = resolveScanSurface(REPOSITORY_REFERENCE_SURFACE, { repoRoot }).paths;
  const repositoryPathSet = new Set(repositoryPaths);
  const scriptPaths = resolveScanSurface(WRITER_SOURCE_SURFACE, { repoRoot }).paths.filter(
    (candidate) => /\.(?:[cm]?js|ts)$/u.test(candidate),
  );
  const analysisCache = buildWriterScriptAnalysisCache(repoRoot, scriptPaths);
  const governedScripts = new Set<string>(
    EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((declaration) => declaration.writerScripts),
  );
  const dataExpressionsByScript = new Map<string, Set<string>>();
  for (const declaration of EVIDENCE_WRITER_COMMAND_DECLARATIONS as readonly EvidenceWriterCommandDeclaration[]) {
    for (const writerScript of declaration.writerScripts) {
      const expressions = dataExpressionsByScript.get(writerScript) ?? new Set<string>();
      for (const output of declaration.manifestOutputPaths ?? []) {
        expressions.add(output.writeExpression);
      }
      for (const writeExpression of declaration.writeExpressions ?? []) {
        expressions.add(writeExpression);
      }
      for (const witness of declaration.outputWriteWitnesses ?? []) {
        if (witness.writerScript === writerScript) expressions.add(witness.writeExpression);
      }
      dataExpressionsByScript.set(writerScript, expressions);
    }
  }
  const refusalKeys = new Set(
    EVIDENCE_WRITER_NON_LITERAL_WRITE_REFUSALS.map(
      (entry) => `${entry.writerScript}\0${entry.writeExpression}`,
    ),
  );
  const unclassified: UnclassifiedGovernedNonLiteralWriteSite[] = [];
  for (const [scriptPath, analysis] of analysisCache) {
    if (!governedScripts.has(scriptPath)) continue;
    const visit = (
      node: tsTypes.Node,
      loopValues: ReadonlyMap<tsTypes.Identifier, readonly StaticDataValue[]> = new Map(),
    ): void => {
      if (ts.isCallExpression(node) && isFileWriteCall(node.expression)) {
        const firstArgument = node.arguments[0];
        if (
          firstArgument &&
          !ts.isStringLiteral(firstArgument) &&
          !ts.isNoSubstitutionTemplateLiteral(firstArgument)
        ) {
          const writeExpression = firstArgument.getText(analysis.sourceFile).replace(/\s+/gu, " ");
          const resolvedRepositoryOutput = lexicalStaticPathExpressionValues(firstArgument, {
            repoRoot,
            scriptPath,
            analysis,
            analysisCache,
            loopValues,
          }).some((resolved) => repositoryPathSet.has(normalizeRepositoryPath(repoRoot, resolved)));
          const dataDeclared =
            dataExpressionsByScript.get(scriptPath)?.has(writeExpression) ?? false;
          const refused = refusalKeys.has(`${scriptPath}\0${writeExpression}`);
          if (!resolvedRepositoryOutput && !dataDeclared && !refused) {
            unclassified.push({
              writerScript: scriptPath,
              line: analysis.sourceFile.getLineAndCharacterOfPosition(node.getStart()).line + 1,
              writeExpression,
            });
          }
        }
      }
      if (ts.isForOfStatement(node) && ts.isVariableDeclarationList(node.initializer)) {
        const iterationValues = staticIterationValues(
          evaluateStaticDataExpression(node.expression, {
            repoRoot,
            scriptPath,
            analysis,
            analysisCache,
            loopValues,
            resolving: new Set(),
          }),
        );
        const nestedLoopValues = new Map(loopValues);
        for (const declaration of node.initializer.declarations) {
          bindStaticIterationValues(declaration.name, iterationValues, nestedLoopValues);
        }
        visit(node.statement, nestedLoopValues);
        return;
      }
      ts.forEachChild(node, (child) => visit(child, loopValues));
    };
    visit(analysis.sourceFile);
  }
  return unclassified.toSorted((left, right) =>
    compareText(
      `${left.writerScript}:${String(left.line).padStart(8, "0")}`,
      `${right.writerScript}:${String(right.line).padStart(8, "0")}`,
    ),
  );
}

export function assertGovernedNonLiteralWriteCoverage(repoRoot: string): void {
  const unclassified = findUnclassifiedGovernedNonLiteralWriteSites(repoRoot);
  if (unclassified.length > 0) {
    const first = unclassified[0]!;
    throw new Error(
      `governed non-literal write site has no output declaration or typed refusal: ${first.writerScript}:${first.line}:${first.writeExpression}`,
    );
  }
  const refusalKeys = EVIDENCE_WRITER_NON_LITERAL_WRITE_REFUSALS.map(
    (entry) => `${entry.writerScript}\0${entry.writeExpression}`,
  );
  if (new Set(refusalKeys).size !== refusalKeys.length) {
    throw new Error("governed non-literal write refusals must be unique");
  }
  for (const refusal of EVIDENCE_WRITER_NON_LITERAL_WRITE_REFUSALS) {
    if (refusal.reason.trim().length === 0) {
      throw new Error(
        `governed non-literal write refusal has no reason: ${refusal.writerScript}:${refusal.writeExpression}`,
      );
    }
    const sourcePath = path.join(repoRoot, refusal.writerScript);
    if (
      !existsSync(sourcePath) ||
      !readFileSync(sourcePath, "utf8").includes(refusal.writeExpression)
    ) {
      throw new Error(
        `governed non-literal write refusal is stale: ${refusal.writerScript}:${refusal.writeExpression}`,
      );
    }
  }
}

export interface EvidenceWriterDeclarationWriteWitnessCoverageSummary {
  readonly witnessedOutputs: readonly string[];
  readonly staticallyBoundWitnessOutputs: readonly string[];
  readonly unresolvedWitnessOutputs: readonly string[];
}

export function assertEvidenceWriterDeclarationWriteWitnessCoverage(
  repoRoot: string,
  discovery: EvidenceArtifactDiscovery = discoverEvidenceArtifacts(repoRoot),
): EvidenceWriterDeclarationWriteWitnessCoverageSummary {
  const declarations: readonly EvidenceWriterCommandDeclaration[] =
    EVIDENCE_WRITER_COMMAND_DECLARATIONS;
  const declarationCountsByScript = new Map<string, number>();
  for (const declaration of declarations) {
    for (const writerScript of declaration.writerScripts) {
      declarationCountsByScript.set(
        writerScript,
        (declarationCountsByScript.get(writerScript) ?? 0) + 1,
      );
    }
  }
  const writeExpressionsByScript = new Map<string, ReadonlySet<string>>();
  const writeOutputPathsByScriptAndExpression = new Map<
    string,
    ReadonlyMap<string, ReadonlySet<string>>
  >();
  for (const [scriptPath, analysis] of discovery.scriptAnalysisCache) {
    const expressions = new Set<string>();
    const outputPathsByExpression = new Map<string, Set<string>>();
    const visit = (
      node: tsTypes.Node,
      loopValues: ReadonlyMap<tsTypes.Identifier, readonly StaticDataValue[]> = new Map(),
    ): void => {
      if (ts.isCallExpression(node) && isFileWriteCall(node.expression)) {
        const firstArgument = node.arguments[0];
        if (firstArgument) {
          const expression = normalizeWriteExpression(firstArgument.getText(analysis.sourceFile));
          expressions.add(expression);
          const resolvedOutputPaths = outputPathsByExpression.get(expression) ?? new Set<string>();
          for (const resolved of lexicalStaticPathExpressionValues(firstArgument, {
            repoRoot,
            scriptPath,
            analysis,
            analysisCache: discovery.scriptAnalysisCache,
            loopValues,
            includeConditionalBranches: true,
          })) {
            resolvedOutputPaths.add(normalizeRepositoryPath(repoRoot, resolved));
          }
          outputPathsByExpression.set(expression, resolvedOutputPaths);
        }
      }
      if (ts.isForOfStatement(node) && ts.isVariableDeclarationList(node.initializer)) {
        const iterationValues = staticIterationValues(
          evaluateStaticDataExpression(node.expression, {
            repoRoot,
            scriptPath,
            analysis,
            analysisCache: discovery.scriptAnalysisCache,
            loopValues,
            resolving: new Set(),
            includeConditionalBranches: true,
          }),
        );
        const nestedLoopValues = new Map(loopValues);
        for (const declaration of node.initializer.declarations) {
          bindStaticIterationValues(declaration.name, iterationValues, nestedLoopValues);
        }
        visit(node.statement, nestedLoopValues);
        return;
      }
      ts.forEachChild(node, (child) => visit(child, loopValues));
    };
    visit(analysis.sourceFile);
    writeExpressionsByScript.set(scriptPath, expressions);
    writeOutputPathsByScriptAndExpression.set(scriptPath, outputPathsByExpression);
  }

  const justificationKeys = new Set<string>();
  const consumedJustificationKeys = new Set<string>();
  for (const justification of EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS) {
    const key = `${justification.commandId ?? ""}\0${justification.outputPath}`;
    if (justificationKeys.has(key)) {
      throw new Error(`evidence writer output justification is duplicated: ${key}`);
    }
    justificationKeys.add(key);
    if (justification.reason.trim().length === 0) {
      throw new Error(`evidence writer output justification has no reason: ${key}`);
    }
    const sourcePath = path.join(repoRoot, justification.writerScript);
    if (
      !existsSync(sourcePath) ||
      !readFileSync(sourcePath, "utf8").includes(justification.writeExpression)
    ) {
      throw new Error(
        `evidence writer output justification is stale: ${justification.writerScript}:${justification.writeExpression}`,
      );
    }
    if (justification.kind === "runtime-variable-output") {
      const sources = new Set(justification.variableFields?.map((field) => field.source) ?? []);
      for (const source of ["timestamp", "wallTime", "sha", "worktreeClean", "host"] as const) {
        if (!sources.has(source)) {
          throw new Error(
            `runtime-variable output justification omits ${source}: ${justification.outputPath}`,
          );
        }
      }
      const artifact = JSON.parse(
        readFileSync(path.join(repoRoot, justification.outputPath), "utf8"),
      ) as unknown;
      const fieldPaths = new Set<string>();
      for (const field of justification.variableFields ?? []) {
        if (fieldPaths.has(field.fieldPath)) {
          throw new Error(
            `runtime-variable output justification duplicates field path ${field.fieldPath}: ${justification.outputPath}`,
          );
        }
        fieldPaths.add(field.fieldPath);
        if (!jsonValueContainsFieldPath(artifact, field.fieldPath)) {
          throw new Error(
            `runtime-variable output justification field path is absent: ${justification.outputPath}:${field.fieldPath}`,
          );
        }
      }
    } else if (justification.variableFields !== undefined) {
      throw new Error(
        `non-runtime output justification cannot declare variable fields: ${justification.outputPath}`,
      );
    }
  }

  for (const outputPath of EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS) {
    const justification = EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS.find(
      (entry) => entry.outputPath === outputPath && entry.kind === "runtime-variable-output",
    );
    if (!justification) {
      throw new Error(
        `runtime-variable evidence output requires a typed justification: ${outputPath}`,
      );
    }
  }
  for (const justification of EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS) {
    if (
      justification.kind === "runtime-variable-output" &&
      !EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS.includes(
        justification.outputPath as (typeof EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS)[number],
      )
    ) {
      throw new Error(
        `runtime-variable output justification lacks an independent classification: ${justification.outputPath}`,
      );
    }
  }

  const witnessedOutputs = new Set<string>();
  const staticallyBoundWitnessOutputs = new Set<string>();
  for (const declaration of declarations) {
    const outputPaths = resolveEvidenceWriterCommandOutputPaths(repoRoot, declaration);
    const outputPathSet = new Set(outputPaths);
    const witnessedOutputPaths = new Set<string>();
    for (const witness of declaration.outputWriteWitnesses ?? []) {
      if (!declaration.writerScripts.includes(witness.writerScript)) {
        throw new Error(
          `evidence writer output witness names an undeclared script: ${declaration.commandId}:${witness.writerScript}`,
        );
      }
      const normalizedExpression = normalizeWriteExpression(witness.writeExpression);
      if (!writeExpressionsByScript.get(witness.writerScript)?.has(normalizedExpression)) {
        throw new Error(
          `evidence writer output witness is not an AST-visible write: ${declaration.commandId}:${witness.writerScript}:${witness.writeExpression}`,
        );
      }
      const resolvedOutputPaths =
        writeOutputPathsByScriptAndExpression
          .get(witness.writerScript)
          ?.get(normalizedExpression) ?? new Set<string>();
      for (const outputPath of witness.outputPaths) {
        if (!outputPathSet.has(outputPath)) {
          throw new Error(
            `evidence writer output witness names an undeclared output: ${declaration.commandId}:${outputPath}`,
          );
        }
        if (witnessedOutputPaths.has(outputPath)) {
          throw new Error(
            `evidence writer output has duplicate write witnesses: ${declaration.commandId}:${outputPath}`,
          );
        }
        witnessedOutputPaths.add(outputPath);
        const witnessKey = `${declaration.commandId}:${outputPath}`;
        witnessedOutputs.add(witnessKey);
        if (resolvedOutputPaths.size > 0) {
          if (!resolvedOutputPaths.has(outputPath)) {
            throw new Error(
              `evidence writer output witness does not resolve to its declared output: ${declaration.commandId}:${outputPath}:${witness.writeExpression}`,
            );
          }
          staticallyBoundWitnessOutputs.add(witnessKey);
        }
      }
    }
    for (const manifestOutput of declaration.manifestOutputPaths ?? []) {
      const manifestPaths = resolveEvidenceWriterCommandOutputPaths(repoRoot, {
        ...declaration,
        outputPaths: [],
        manifestOutputPaths: [manifestOutput],
      });
      const expression = normalizeWriteExpression(manifestOutput.writeExpression);
      const hasAstWitness = declaration.writerScripts.some((writerScript) =>
        writeExpressionsByScript.get(writerScript)?.has(expression),
      );
      if (!hasAstWitness) {
        throw new Error(
          `evidence writer manifest output has no AST-visible write: ${declaration.commandId}:${manifestOutput.writeExpression}`,
        );
      }
      for (const outputPath of manifestPaths) witnessedOutputPaths.add(outputPath);
    }

    const hasSharedWriter = declaration.writerScripts.some(
      (writerScript) => (declarationCountsByScript.get(writerScript) ?? 0) > 1,
    );
    for (const outputPath of outputPaths) {
      const justification = EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS.find(
        (entry) => entry.commandId === declaration.commandId && entry.outputPath === outputPath,
      );
      if (justification) {
        if (!declaration.writerScripts.includes(justification.writerScript)) {
          throw new Error(
            `evidence writer output justification names an undeclared script: ${declaration.commandId}:${justification.writerScript}`,
          );
        }
        consumedJustificationKeys.add(`${declaration.commandId}\0${outputPath}`);
        continue;
      }
      if (witnessedOutputPaths.has(outputPath)) continue;
      const exactWriters = discovery.staticWriterScriptsByOutput.get(outputPath) ?? [];
      if (
        !hasSharedWriter &&
        exactWriters.some((script) => declaration.writerScripts.includes(script))
      ) {
        continue;
      }
      throw new Error(
        `evidence writer declared output has no command-scoped AST write witness or typed justification: ${declaration.commandId}:${outputPath}`,
      );
    }
  }
  for (const justification of EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS) {
    if (justification.commandId === undefined) continue;
    const key = `${justification.commandId}\0${justification.outputPath}`;
    if (!consumedJustificationKeys.has(key)) {
      throw new Error(`evidence writer output justification is unbound: ${key}`);
    }
  }
  return {
    witnessedOutputs: [...witnessedOutputs].toSorted(),
    staticallyBoundWitnessOutputs: [...staticallyBoundWitnessOutputs].toSorted(),
    unresolvedWitnessOutputs: [...witnessedOutputs]
      .filter((output) => !staticallyBoundWitnessOutputs.has(output))
      .toSorted(),
  };
}

function jsonValueContainsFieldPath(value: unknown, fieldPath: string): boolean {
  const segments = fieldPath.split(".");
  if (segments.length === 0 || segments.some((segment) => segment.length === 0)) return false;
  let current: readonly unknown[] = [value];
  for (const segment of segments) {
    const arraySegment = segment.endsWith("[]");
    const property = arraySegment ? segment.slice(0, -2) : segment;
    if (property.length === 0) return false;
    const next: unknown[] = [];
    for (const candidate of current) {
      if (typeof candidate !== "object" || candidate === null || !(property in candidate)) {
        return false;
      }
      const propertyValue = (candidate as Record<string, unknown>)[property];
      if (arraySegment) {
        if (!Array.isArray(propertyValue) || propertyValue.length === 0) return false;
        next.push(...propertyValue);
      } else {
        next.push(propertyValue);
      }
    }
    current = next;
  }
  return current.length > 0;
}

function normalizeWriteExpression(expression: string): string {
  return expression.replace(/\s+/gu, " ").trim();
}

function discoverEvidenceArtifacts(repoRoot: string): EvidenceArtifactDiscovery {
  const repositoryPaths = resolveScanSurface(REPOSITORY_REFERENCE_SURFACE, { repoRoot }).paths;
  const scriptPaths = resolveScanSurface(WRITER_SOURCE_SURFACE, { repoRoot }).paths.filter(
    (candidate) => /\.(?:[cm]?js|ts)$/u.test(candidate),
  );
  const scriptAnalysisCache = buildWriterScriptAnalysisCache(repoRoot, scriptPaths);
  const rootArtifacts = [
    ...new Set([
      ...resolveScanSurface(ROOT_RUST_ARTIFACT_SURFACE, { repoRoot }).paths.filter(
        (candidate) => path.posix.dirname(candidate) === "rust" && candidate.endsWith(".json"),
      ),
      EVIDENCE_WRITER_REGISTRY_PATH,
    ]),
  ];
  const staticWriterScriptsByOutput = discoverTrackedStaticWriteOutputs(
    repoRoot,
    repositoryPaths,
    scriptAnalysisCache,
  );
  const trackedWriterOutputs = [...staticWriterScriptsByOutput.keys()].toSorted();
  const declaredCommandOutputPaths = [
    ...new Set(
      EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((row) =>
        resolveEvidenceWriterCommandOutputPaths(repoRoot, row),
      ),
    ),
  ].toSorted();
  return {
    repositoryPaths,
    scriptPaths,
    artifactPaths: [
      ...new Set([...rootArtifacts, ...trackedWriterOutputs, ...declaredCommandOutputPaths]),
    ].toSorted(),
    rootArtifactPaths: rootArtifacts.toSorted(),
    writeCallOutputPaths: trackedWriterOutputs.toSorted(),
    declaredCommandOutputPaths,
    staticWriterScriptsByOutput,
    scriptAnalysisCache,
  };
}

function buildWriterScriptAnalysisCache(
  repoRoot: string,
  scriptPaths: readonly string[],
): Map<string, WriterScriptAnalysis> {
  const cache = new Map<string, WriterScriptAnalysis>();
  for (const scriptPath of scriptPaths) {
    const source = readFileSync(path.join(repoRoot, scriptPath), "utf8");
    const sourceFile = ts.createSourceFile(scriptPath, source, ts.ScriptTarget.Latest, true);
    const lexical = buildStaticPathLexicalIndex(sourceFile);
    cache.set(scriptPath, {
      source,
      sourceFile,
      staticPathVariables: collectStaticPathVariables(repoRoot, sourceFile),
      lexical,
      lexicalStaticPathValues: collectLexicalStaticPathValues(repoRoot, sourceFile, lexical),
    });
  }
  const scriptPathSet = new Set(scriptPaths);
  let changed = true;
  while (changed) {
    changed = false;
    for (const [scriptPath, analysis] of cache) {
      const importedValues = new Map<string, string>();
      for (const statement of analysis.sourceFile.statements) {
        if (
          !ts.isImportDeclaration(statement) ||
          !ts.isStringLiteral(statement.moduleSpecifier) ||
          !statement.importClause?.namedBindings ||
          !ts.isNamedImports(statement.importClause.namedBindings)
        ) {
          continue;
        }
        const importedModule = resolveLocalImportedModulePath(
          scriptPath,
          statement.moduleSpecifier.text,
          scriptPathSet,
        );
        if (!importedModule) continue;
        const importedAnalysis = cache.get(importedModule);
        if (!importedAnalysis) continue;
        for (const element of statement.importClause.namedBindings.elements) {
          const importedName = element.propertyName?.text ?? element.name.text;
          const value = importedAnalysis.staticPathVariables.get(importedName);
          if (value !== undefined) importedValues.set(element.name.text, value);
        }
      }
      const nextValues = collectStaticPathVariables(repoRoot, analysis.sourceFile, importedValues);
      const nextLexicalValues = collectLexicalStaticPathValues(
        repoRoot,
        analysis.sourceFile,
        analysis.lexical,
        importedValues,
      );
      if (
        nextValues.size !== analysis.staticPathVariables.size ||
        [...nextValues].some(([name, value]) => analysis.staticPathVariables.get(name) !== value) ||
        nextLexicalValues.size !== analysis.lexicalStaticPathValues.size ||
        [...nextLexicalValues].some(
          ([identifier, value]) => analysis.lexicalStaticPathValues.get(identifier) !== value,
        )
      ) {
        cache.set(scriptPath, {
          ...analysis,
          staticPathVariables: nextValues,
          lexicalStaticPathValues: nextLexicalValues,
        });
        changed = true;
      }
    }
  }
  return cache;
}

function discoverTrackedStaticWriteOutputs(
  repoRoot: string,
  repositoryPaths: readonly string[],
  analysisCache: ReadonlyMap<string, WriterScriptAnalysis>,
): ReadonlyMap<string, readonly string[]> {
  const repositoryPathSet = new Set(repositoryPaths);
  const writersByOutput = new Map<string, string[]>();
  for (const [scriptPath, analysis] of analysisCache) {
    const visit = (
      node: tsTypes.Node,
      loopValues: ReadonlyMap<tsTypes.Identifier, readonly StaticDataValue[]> = new Map(),
    ): void => {
      if (ts.isCallExpression(node) && isFileWriteCall(node.expression)) {
        const firstArgument = node.arguments[0];
        const resolvedValues = firstArgument
          ? lexicalStaticPathExpressionValues(firstArgument, {
              repoRoot,
              scriptPath,
              analysis,
              analysisCache,
              loopValues,
            })
          : [];
        for (const resolved of resolvedValues) {
          const candidate = normalizeRepositoryPath(repoRoot, resolved);
          if (repositoryPathSet.has(candidate)) {
            writersByOutput.set(candidate, [...(writersByOutput.get(candidate) ?? []), scriptPath]);
          }
        }
      }
      if (ts.isForOfStatement(node) && ts.isVariableDeclarationList(node.initializer)) {
        const iterationValues = staticIterationValues(
          evaluateStaticDataExpression(node.expression, {
            repoRoot,
            scriptPath,
            analysis,
            analysisCache,
            loopValues,
            resolving: new Set(),
          }),
        );
        const nestedLoopValues = new Map(loopValues);
        for (const declaration of node.initializer.declarations) {
          bindStaticIterationValues(declaration.name, iterationValues, nestedLoopValues);
        }
        visit(node.statement, nestedLoopValues);
        return;
      }
      ts.forEachChild(node, (child) => visit(child, loopValues));
    };
    visit(analysis.sourceFile);
  }
  return new Map(
    [...writersByOutput]
      .toSorted(([left], [right]) => compareText(left, right))
      .map(([outputPath, writerScripts]) => [outputPath, [...new Set(writerScripts)].toSorted()]),
  );
}

function classifyArtifact(input: {
  readonly repoRoot: string;
  readonly artifactPath: string;
  readonly writerScripts: readonly string[];
  readonly updateCommands: readonly {
    readonly scriptName: string;
    readonly command: string;
    readonly writerScript: string;
  }[];
  readonly references: readonly string[];
  readonly scannerGateIds: ReadonlyMap<string, readonly string[]>;
  readonly checkManifest: ReturnType<typeof loadCheckManifest>;
  readonly repositoryPathSet: ReadonlySet<string>;
  readonly artifactPathSet: ReadonlySet<string>;
  readonly writerOutputPathsByScript: ReadonlyMap<string, readonly string[]>;
}): EvidenceArtifactRowV0 {
  const { artifactPath, writerScripts, updateCommands, references, scannerGateIds, checkManifest } =
    input;
  const declaredCommands = declaredCommandsForOutput(input.repoRoot, artifactPath);
  const declaredCommand = declaredCommands[0];
  const specialCommand = declaredCommand?.writeCommand ?? specialWriterCommand(artifactPath);
  const manualProcedure = handAuthoredProcedure(artifactPath);
  const writerFlag =
    writerScripts.length === 1
      ? writerFlagForSource(
          readFileSync(path.join(input.repoRoot, writerScripts[0]!), "utf8"),
          artifactPath,
        )
      : null;
  const classification: EvidenceArtifactClassification =
    declaredCommand || specialCommand || updateCommands.length > 0
      ? "W1"
      : writerScripts.length > 0
        ? "W2"
        : references.length === 0
          ? "W4"
          : "W3";
  const updateCommand = updateCommands[0];
  const writeCommand = specialCommand
    ? specialCommand
    : classification === "W1"
      ? commandSegmentForWriter(updateCommand?.command ?? "", updateCommand?.writerScript ?? "")
      : classification === "W2"
        ? selfWriterCommand(artifactPath, writerScripts[0]!, writerFlag)
        : undefined;
  const requiredEnvironmentKeys = WPT_TIER_ZERO_OUTPUT_PATHS.has(artifactPath)
    ? ["OMENA_WPT_ROOT"]
    : artifactPath === "rust/omena-css-module-token-shape-measurement.json"
      ? [
          "OMENA_TOKEN_CORPUS_ROOT",
          "OMENA_TOKEN_IDENTITY_REACT_TS_CSS",
          "OMENA_TOKEN_IDENTITY_MKN",
          "OMENA_TOKEN_IDENTITY_DOCUSAURUS",
        ]
      : artifactPath === "rust/omena-published-crate-surface-register.json"
        ? ["OMENA_PUBLISHED_CRATE_REGISTRY_STATE"]
        : undefined;
  const writerOutputPaths = new Set(
    writerScripts.flatMap(
      (writerScript) => input.writerOutputPathsByScript.get(writerScript) ?? [],
    ),
  );
  const literalInputs = literalRepositoryInputs(
    input.repoRoot,
    writerScripts,
    input.repositoryPathSet,
  );
  const moduleInputs = localWriterModuleInputs(
    input.repoRoot,
    writerScripts,
    input.repositoryPathSet,
  );
  const declaredInputs = declaredCommands.flatMap((row) => row.inputPaths ?? []);
  const inferredInputs = [...literalInputs, ...moduleInputs].filter(
    (candidate) => !writerOutputPaths.has(candidate),
  );
  const allInputs = [...new Set([...inferredInputs, ...declaredInputs])].filter(
    (candidate) => !writerScripts.includes(candidate),
  );
  const inputArtifactPaths = [
    ...new Set([
      ...artifactDependencies(artifactPath),
      ...allInputs.filter((candidate) => input.artifactPathSet.has(candidate)),
    ]),
  ].toSorted();
  const inputPaths = allInputs
    .filter((candidate) => !input.artifactPathSet.has(candidate))
    .toSorted();
  const inputScannerPaths =
    artifactPath === EVIDENCE_WRITER_REGISTRY_PATH
      ? [...scannerGateIds.keys()].toSorted()
      : artifactPath === "rust/evidence-scan-surfaces.json"
        ? ["packages/check-orchestrator/src/evidence/scanner-analysis.ts"]
        : writerScripts.filter((script) => scannerGateIds.has(script));
  const writerGateIds = [
    ...new Set(
      checkManifest.gates
        .filter((gate) =>
          [writeCommand, ...declaredCommands.slice(1).map((row) => row.writeCommand)]
            .filter((command): command is readonly string[] => command !== undefined)
            .some((command) => gateRunsWriterCommand(gate.command, command)),
        )
        .map((gate) => gate.id),
    ),
  ].toSorted();
  const consumerPaths = references
    .filter(
      (candidate) => candidate !== "packages/check-orchestrator/src/evidence/writer-registry.ts",
    )
    .toSorted();
  const consumerGateIds = [
    ...new Set(
      checkManifest.gates
        .filter((gate) => consumerPaths.some((consumerPath) => gate.command.includes(consumerPath)))
        .map((gate) => gate.id),
    ),
  ].toSorted();
  const freshReproductionExemption = EVIDENCE_FRESH_REPRODUCTION_EXEMPTIONS.find(
    (entry) => entry.artifactPath === artifactPath,
  );
  return {
    artifactPath,
    classification,
    writerNodeKind:
      artifactPath === "rust/omena-identifier-authority-census.json" ? "self-ratchet" : "normal",
    ...(freshReproductionExemption
      ? {
          freshReproductionExemption: {
            kind: freshReproductionExemption.kind,
            reason: freshReproductionExemption.reason,
          },
        }
      : classification === "W1" ||
          artifactPath === "rust/omena-published-crate-surface-register.json"
        ? { freshReproductionRequired: true as const }
        : {}),
    writerScripts,
    ...(writeCommand ? { writeCommand } : {}),
    ...(declaredCommands.length > 1
      ? { alternateWriteCommands: declaredCommands.slice(1).map((row) => row.writeCommand) }
      : {}),
    ...(requiredEnvironmentKeys ? { requiredEnvironmentKeys } : {}),
    ...(classification === "W3"
      ? { procedure: manualProcedure ?? genericHandAuthoredProcedure(artifactPath) }
      : {}),
    ...(classification === "W4" ? { disposition: "reviewer-retire-proposal" as const } : {}),
    inputPaths,
    inputScannerPaths,
    inputArtifactPaths,
    writerGateIds,
    consumerPaths,
    consumerGateIds,
  };
}

function declaredCommandsForOutput(
  repoRoot: string,
  artifactPath: string,
): readonly EvidenceWriterCommandDeclaration[] {
  return EVIDENCE_WRITER_COMMAND_DECLARATIONS.filter((row) =>
    resolveEvidenceWriterCommandOutputPaths(repoRoot, row).some(
      (outputPath) => outputPath === artifactPath,
    ),
  );
}

function gateRunsWriterCommand(gateCommand: string, writerCommand: readonly string[]): boolean {
  const normalizedGate = gateCommand.replaceAll("./", "").replace(/\s+/gu, " ").trim();
  const normalizedWriter = writerCommand
    .join(" ")
    .replaceAll("./", "")
    .replace(/\s+/gu, " ")
    .trim();
  return normalizedGate.split(/\s*&&\s*/u).some((segment) => segment === normalizedWriter);
}

export function assertEvidenceWriterAuthorityCoverage(
  registry: EvidenceWriterRegistryV0,
  declarations: readonly EvidenceWriterCommandDeclaration[] = EVIDENCE_WRITER_COMMAND_DECLARATIONS,
  repoRoot = process.cwd(),
): void {
  const rowsByPath = new Map(registry.artifacts.map((row) => [row.artifactPath, row]));
  for (const declaration of declarations) {
    const declarationOutputPaths = resolveEvidenceWriterCommandOutputPaths(repoRoot, declaration);
    if (declarationOutputPaths.length === 0) {
      throw new Error(`evidence writer command declares no outputs: ${declaration.commandId}`);
    }
    for (const outputPath of declarationOutputPaths) {
      const row = rowsByPath.get(outputPath);
      if (!row) {
        throw new Error(
          `evidence writer authority output is absent from registry: ${declaration.commandId}:${outputPath}`,
        );
      }
      const declaredRecipeIsRecorded = [
        row.writeCommand,
        ...(row.alternateWriteCommands ?? []),
      ].some((command) => JSON.stringify(command) === JSON.stringify(declaration.writeCommand));
      if (!declaredRecipeIsRecorded) {
        throw new Error(
          `evidence writer authority command drifted: ${declaration.commandId}:${outputPath}`,
        );
      }
      for (const writerScript of declaration.writerScripts) {
        if (!row.writerScripts.includes(writerScript)) {
          throw new Error(
            `evidence writer authority source is absent: ${declaration.commandId}:${writerScript}`,
          );
        }
      }
      for (const inputPath of declaration.inputPaths ?? []) {
        if (![...row.inputPaths, ...row.inputArtifactPaths].includes(inputPath)) {
          throw new Error(
            `evidence writer authority input is absent: ${declaration.commandId}:${inputPath}`,
          );
        }
      }
    }
  }
}

export function assertEvidenceWriterOutputAuthorityCoverage(
  registry: EvidenceWriterRegistryV0,
  authority: EvidenceWriterOutputAuthority,
): void {
  const registryPaths = new Set(registry.artifacts.map((row) => row.artifactPath));
  for (const [authorityKind, outputPaths] of Object.entries(authority)) {
    for (const outputPath of outputPaths) {
      if (!registryPaths.has(outputPath)) {
        throw new Error(`evidence ${authorityKind} output is absent from registry: ${outputPath}`);
      }
    }
  }
  const rootPaths = new Set(authority.rootArtifactPaths);
  const discoveredNonRootWrites = authority.writeCallOutputPaths
    .filter((outputPath) => !rootPaths.has(outputPath))
    .toSorted();
  const independentStaticWrites = [...new Set(authority.staticWriteOutputPaths)].toSorted();
  for (const outputPath of discoveredNonRootWrites) {
    if (!independentStaticWrites.includes(outputPath)) {
      throw new Error(`static write output has no independent authority: ${outputPath}`);
    }
  }
  for (const outputPath of independentStaticWrites) {
    if (!discoveredNonRootWrites.includes(outputPath)) {
      throw new Error(`independent static write output was not discovered: ${outputPath}`);
    }
  }
}

export function assertUpdateCommandWriterCoverage(
  packageScripts: Readonly<Record<string, string>>,
  artifacts: readonly EvidenceArtifactRowV0[],
): void {
  const recordedCommands = new Set(
    artifacts
      .flatMap((row) => [row.writeCommand, ...(row.alternateWriteCommands ?? [])])
      .filter((command): command is readonly string[] => command !== undefined)
      .map((command) => JSON.stringify(command)),
  );
  for (const [scriptName, command] of Object.entries(packageScripts)) {
    if (!scriptName.startsWith("update:")) continue;
    for (const segment of command.split(/\s*&&\s*/u)) {
      const words = splitCommandWords(segment);
      const normalized = words[0]?.includes("=") ? ["env", ...words] : words;
      if (!recordedCommands.has(JSON.stringify(normalized))) {
        throw new Error(`update command has no total writer-output authority: ${scriptName}`);
      }
    }
  }
}

function localWriterModuleInputs(
  repoRoot: string,
  writerScripts: readonly string[],
  repositoryPathSet: ReadonlySet<string>,
): readonly string[] {
  const inputs = new Set<string>();
  for (const writerScript of writerScripts) {
    const source = readFileSync(path.join(repoRoot, writerScript), "utf8");
    const sourceFile = ts.createSourceFile(writerScript, source, ts.ScriptTarget.Latest, true);
    for (const statement of sourceFile.statements) {
      if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
        continue;
      }
      const specifier = statement.moduleSpecifier.text;
      if (!specifier.startsWith(".")) continue;
      const base = path.posix.normalize(
        path.posix.join(path.posix.dirname(writerScript), specifier),
      );
      for (const candidate of [
        base,
        `${base}.ts`,
        `${base}.tsx`,
        `${base}.mjs`,
        `${base}.js`,
        `${base}/index.ts`,
      ]) {
        if (repositoryPathSet.has(candidate)) {
          inputs.add(candidate);
          break;
        }
      }
    }
  }
  return [...inputs].toSorted();
}

function expandWriterScriptsWithEntrypoints(
  directWriterScripts: readonly string[],
  importersByModule: ReadonlyMap<string, readonly string[]>,
  updateCommands: readonly string[],
): readonly string[] {
  const reachable = new Set(directWriterScripts);
  const queue = [...directWriterScripts];
  while (queue.length > 0) {
    const writerModule = queue.shift()!;
    for (const importer of importersByModule.get(writerModule) ?? []) {
      if (reachable.has(importer)) continue;
      reachable.add(importer);
      queue.push(importer);
    }
  }
  const commandEntrypoints = [...reachable].filter((candidate) =>
    updateCommands.some((command) => command.includes(candidate)),
  );
  if (commandEntrypoints.length === 0) return directWriterScripts;
  return [...reachable].toSorted();
}

function buildScriptImporters(
  repoRoot: string,
  scriptPaths: readonly string[],
): ReadonlyMap<string, readonly string[]> {
  const scriptPathSet = new Set(scriptPaths);
  const importersByModule = new Map<string, string[]>();
  for (const scriptPath of scriptPaths) {
    for (const importedModule of localImportedModulePaths(repoRoot, scriptPath, scriptPathSet)) {
      importersByModule.set(importedModule, [
        ...(importersByModule.get(importedModule) ?? []),
        scriptPath,
      ]);
    }
  }
  return importersByModule;
}

function localImportedModulePaths(
  repoRoot: string,
  modulePath: string,
  repositoryPathSet: ReadonlySet<string>,
): readonly string[] {
  const source = readFileSync(path.join(repoRoot, modulePath), "utf8");
  const sourceFile = ts.createSourceFile(modulePath, source, ts.ScriptTarget.Latest, true);
  const imports = new Set<string>();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) {
      continue;
    }
    const specifier = statement.moduleSpecifier.text;
    const importedModule = resolveLocalImportedModulePath(modulePath, specifier, repositoryPathSet);
    if (importedModule) imports.add(importedModule);
  }
  return [...imports].toSorted();
}

function resolveLocalImportedModulePath(
  modulePath: string,
  specifier: string,
  repositoryPathSet: ReadonlySet<string>,
): string | null {
  if (!specifier.startsWith(".")) return null;
  const base = path.posix.normalize(path.posix.join(path.posix.dirname(modulePath), specifier));
  for (const candidate of [
    base,
    `${base}.ts`,
    `${base}.tsx`,
    `${base}.mjs`,
    `${base}.js`,
    `${base}/index.ts`,
  ]) {
    if (repositoryPathSet.has(candidate)) return candidate;
  }
  return null;
}

function literalRepositoryInputs(
  repoRoot: string,
  writerScripts: readonly string[],
  repositoryPathSet: ReadonlySet<string>,
): readonly string[] {
  const inputs = new Set<string>();
  for (const writerScript of writerScripts) {
    const source = readFileSync(path.join(repoRoot, writerScript), "utf8");
    const sourceFile = ts.createSourceFile(writerScript, source, ts.ScriptTarget.Latest, true);
    const staticPaths = collectStaticPathVariables(repoRoot, sourceFile);
    for (const value of staticPaths.values()) {
      const candidate = normalizeRepositoryPath(repoRoot, value);
      if (repositoryPathSet.has(candidate)) inputs.add(candidate);
    }
    const visit = (node: tsTypes.Node): void => {
      if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
        const candidate = normalizeRepositoryPath(repoRoot, node.text);
        if (repositoryPathSet.has(candidate)) inputs.add(candidate);
      }
      if (ts.isCallExpression(node)) {
        const value = staticPathExpressionValue(node, sourceFile, staticPaths, repoRoot);
        if (value !== null) {
          const candidate = normalizeRepositoryPath(repoRoot, value);
          if (repositoryPathSet.has(candidate)) inputs.add(candidate);
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
  }
  return [...inputs].toSorted();
}

interface StaticPathLexicalScope {
  readonly parent: StaticPathLexicalScope | null;
  readonly bindings: ReadonlyMap<string, tsTypes.Identifier>;
}

function buildStaticPathLexicalIndex(sourceFile: tsTypes.SourceFile): StaticPathLexicalIndex {
  const resolved = new Map<tsTypes.Identifier, tsTypes.Identifier | null>();
  const declarations = new Set<tsTypes.Identifier>();
  const createScope = (
    node: tsTypes.Node,
    parent: StaticPathLexicalScope | null,
  ): StaticPathLexicalScope => {
    const bindings = collectStaticPathScopeBindings(node);
    for (const declaration of bindings.values()) declarations.add(declaration);
    return { parent, bindings };
  };
  const rootScope = createScope(sourceFile, null);
  const visit = (node: tsTypes.Node, inheritedScope: StaticPathLexicalScope): void => {
    const scope =
      node === sourceFile
        ? rootScope
        : isStaticPathScopeNode(node)
          ? createScope(node, inheritedScope)
          : inheritedScope;
    if (ts.isIdentifier(node)) {
      if (declarations.has(node)) {
        resolved.set(node, node);
      } else {
        let current: StaticPathLexicalScope | null = scope;
        let declaration: tsTypes.Identifier | null = null;
        while (current && !declaration) {
          declaration = current.bindings.get(node.text) ?? null;
          current = current.parent;
        }
        resolved.set(node, declaration);
      }
    }
    ts.forEachChild(node, (child) => visit(child, scope));
  };
  visit(sourceFile, rootScope);
  return { declarationFor: (identifier) => resolved.get(identifier) ?? null };
}

function isStaticPathScopeNode(node: tsTypes.Node): boolean {
  return (
    ts.isBlock(node) ||
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isArrowFunction(node) ||
    ts.isMethodDeclaration(node) ||
    ts.isCatchClause(node) ||
    ts.isForStatement(node) ||
    ts.isForInStatement(node) ||
    ts.isForOfStatement(node)
  );
}

function collectStaticPathScopeBindings(
  node: tsTypes.Node,
): ReadonlyMap<string, tsTypes.Identifier> {
  const bindings = new Map<string, tsTypes.Identifier>();
  const add = (identifier: tsTypes.Identifier | undefined): void => {
    if (identifier) bindings.set(identifier.text, identifier);
  };
  const addBindingName = (name: tsTypes.BindingName): void => {
    if (ts.isIdentifier(name)) {
      add(name);
      return;
    }
    for (const element of name.elements) {
      if (!ts.isOmittedExpression(element)) addBindingName(element.name);
    }
  };
  if (
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isArrowFunction(node) ||
    ts.isMethodDeclaration(node)
  ) {
    if ("name" in node && node.name && ts.isIdentifier(node.name)) add(node.name);
    for (const parameter of node.parameters) addBindingName(parameter.name);
    return bindings;
  }
  if (ts.isCatchClause(node)) {
    if (node.variableDeclaration) addBindingName(node.variableDeclaration.name);
    return bindings;
  }
  if (ts.isForStatement(node) || ts.isForInStatement(node) || ts.isForOfStatement(node)) {
    if (node.initializer && ts.isVariableDeclarationList(node.initializer)) {
      for (const declaration of node.initializer.declarations) addBindingName(declaration.name);
    }
    return bindings;
  }
  if (!ts.isSourceFile(node) && !ts.isBlock(node)) return bindings;
  for (const statement of node.statements) {
    if (ts.isImportDeclaration(statement)) {
      add(statement.importClause?.name);
      const namedBindings = statement.importClause?.namedBindings;
      if (namedBindings && ts.isNamespaceImport(namedBindings)) add(namedBindings.name);
      if (namedBindings && ts.isNamedImports(namedBindings)) {
        for (const element of namedBindings.elements) add(element.name);
      }
      continue;
    }
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        addBindingName(declaration.name);
      }
      continue;
    }
    if (ts.isFunctionDeclaration(statement) && statement.name) {
      add(statement.name);
      continue;
    }
    if (ts.isClassDeclaration(statement) && statement.name) add(statement.name);
  }
  return bindings;
}

function collectLexicalStaticPathValues(
  repoRoot: string,
  sourceFile: tsTypes.SourceFile,
  lexical: StaticPathLexicalIndex,
  importedValues: ReadonlyMap<string, string> = new Map(),
): ReadonlyMap<tsTypes.Identifier, string> {
  const values = new Map<tsTypes.Identifier, string>();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !statement.importClause?.namedBindings) continue;
    const namedBindings = statement.importClause.namedBindings;
    if (!ts.isNamedImports(namedBindings)) continue;
    for (const element of namedBindings.elements) {
      const value = importedValues.get(element.name.text);
      if (value !== undefined) values.set(element.name, value);
    }
  }
  let changed = true;
  while (changed) {
    changed = false;
    const visit = (node: tsTypes.Node): void => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        !values.has(node.name)
      ) {
        const value = lexicalStaticPathExpressionValue(
          node.initializer,
          { sourceFile, lexical, lexicalStaticPathValues: values },
          repoRoot,
        );
        if (value !== null) {
          values.set(node.name, value);
          changed = true;
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
  }
  return values;
}

function lexicalStaticPathExpressionValue(
  expression: tsTypes.Expression,
  analysis: Pick<WriterScriptAnalysis, "sourceFile" | "lexical" | "lexicalStaticPathValues">,
  repoRoot: string,
): string | null {
  const { sourceFile, lexical, lexicalStaticPathValues } = analysis;
  if (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) {
    return expression.text;
  }
  if (ts.isIdentifier(expression)) {
    const declaration = lexical.declarationFor(expression);
    return declaration ? (lexicalStaticPathValues.get(declaration) ?? null) : null;
  }
  if (ts.isPropertyAccessExpression(expression)) {
    if (
      expression.expression.getText(sourceFile) === "import.meta" &&
      expression.name.text === "dirname"
    ) {
      return path.dirname(path.join(repoRoot, sourceFile.fileName));
    }
  }
  if (!ts.isCallExpression(expression)) return null;
  const calleeText = expression.expression.getText(sourceFile);
  if (calleeText === "process.cwd" && expression.arguments.length === 0) return repoRoot;
  const name = callName(expression.expression);
  if (
    name === "fileURLToPath" &&
    expression.arguments[0]?.getText(sourceFile) === "import.meta.url"
  ) {
    return path.join(repoRoot, sourceFile.fileName);
  }
  if (name === "dirname" && expression.arguments.length === 1) {
    const value = lexicalStaticPathExpressionValue(expression.arguments[0]!, analysis, repoRoot);
    return value === null ? null : path.dirname(value);
  }
  if (name !== "join" && name !== "resolve") return null;
  const evaluatedParts = expression.arguments.map((argument) =>
    lexicalStaticPathExpressionValue(argument, analysis, repoRoot),
  );
  if (evaluatedParts.some((value) => value === null)) return null;
  const parts = evaluatedParts as string[];
  return name === "join" ? path.join(...parts) : path.resolve(repoRoot, ...parts);
}

interface StaticDataArray {
  readonly kind: "array";
  readonly items: readonly StaticDataValue[];
}

interface StaticDataObject {
  readonly kind: "object";
  readonly properties: ReadonlyMap<string, readonly StaticDataValue[]>;
}

interface StaticDataMap {
  readonly kind: "map";
  readonly entries: readonly {
    readonly key: StaticDataValue;
    readonly value: StaticDataValue;
  }[];
}

interface StaticDataUnknown {
  readonly kind: "unknown";
}

type StaticDataValue =
  | string
  | StaticDataArray
  | StaticDataObject
  | StaticDataMap
  | StaticDataUnknown;

const STATIC_DATA_UNKNOWN: StaticDataUnknown = { kind: "unknown" };

interface StaticDataEvaluationContext {
  readonly repoRoot: string;
  readonly scriptPath: string;
  readonly analysis: WriterScriptAnalysis;
  readonly analysisCache: ReadonlyMap<string, WriterScriptAnalysis>;
  readonly loopValues: ReadonlyMap<tsTypes.Identifier, readonly StaticDataValue[]>;
  readonly resolving?: Set<string>;
  readonly includeConditionalBranches?: boolean;
}

function lexicalStaticPathExpressionValues(
  expression: tsTypes.Expression,
  context: StaticDataEvaluationContext,
): readonly string[] {
  const values = evaluateStaticDataExpression(expression, {
    ...context,
    resolving: context.resolving ?? new Set(),
  }).filter((value): value is string => typeof value === "string");
  if (values.length > 0) return [...new Set(values)].toSorted();
  const scalar = lexicalStaticPathExpressionValue(expression, context.analysis, context.repoRoot);
  return scalar === null ? [] : [scalar];
}

function evaluateStaticDataExpression(
  expression: tsTypes.Expression,
  context: StaticDataEvaluationContext & { readonly resolving: Set<string> },
): readonly StaticDataValue[] {
  const unwrapped = unwrapStaticDataExpression(expression);
  if (ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)) {
    return [unwrapped.text];
  }
  if (ts.isTemplateExpression(unwrapped)) {
    let prefixes = [unwrapped.head.text];
    for (const span of unwrapped.templateSpans) {
      const replacements = evaluateStaticDataExpression(span.expression, context).filter(
        (value): value is string => typeof value === "string",
      );
      if (replacements.length === 0) return [];
      prefixes = prefixes.flatMap((prefix) =>
        replacements.map((replacement) => `${prefix}${replacement}${span.literal.text}`),
      );
    }
    return prefixes;
  }
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken
  ) {
    return [
      ...evaluateStaticDataExpression(unwrapped.left, context),
      ...evaluateStaticDataExpression(unwrapped.right, context),
    ];
  }
  if (ts.isConditionalExpression(unwrapped) && context.includeConditionalBranches === true) {
    return [
      ...evaluateStaticDataExpression(unwrapped.whenTrue, context),
      ...evaluateStaticDataExpression(unwrapped.whenFalse, context),
    ];
  }
  if (ts.isIdentifier(unwrapped)) return evaluateStaticIdentifier(unwrapped, context);
  if (ts.isArrayLiteralExpression(unwrapped)) {
    return [
      {
        kind: "array",
        items: unwrapped.elements.map(
          (element) => evaluateStaticDataExpression(element, context)[0] ?? STATIC_DATA_UNKNOWN,
        ),
      },
    ];
  }
  if (ts.isObjectLiteralExpression(unwrapped)) {
    const properties = new Map<string, readonly StaticDataValue[]>();
    for (const property of unwrapped.properties) {
      if (!ts.isPropertyAssignment(property)) continue;
      const name = staticPropertyName(property.name);
      if (name !== null)
        properties.set(name, evaluateStaticDataExpression(property.initializer, context));
    }
    return [{ kind: "object", properties }];
  }
  if (ts.isNewExpression(unwrapped) && callName(unwrapped.expression) === "Map") {
    const iterable = unwrapped.arguments?.[0];
    if (!iterable) return [];
    const arrays = evaluateStaticDataExpression(iterable, context).filter(
      (value): value is StaticDataArray => typeof value !== "string" && value.kind === "array",
    );
    return arrays.map((array) => ({
      kind: "map" as const,
      entries: array.items.flatMap((item) =>
        typeof item !== "string" && item.kind === "array" && item.items.length >= 2
          ? [{ key: item.items[0]!, value: item.items[1]! }]
          : [],
      ),
    }));
  }
  if (ts.isPropertyAccessExpression(unwrapped)) {
    if (
      unwrapped.expression.getText(context.analysis.sourceFile) === "import.meta" &&
      unwrapped.name.text === "dirname"
    ) {
      return [path.dirname(path.join(context.repoRoot, context.scriptPath))];
    }
    return evaluateStaticDataExpression(unwrapped.expression, context).flatMap((value) =>
      typeof value !== "string" && value.kind === "object"
        ? (value.properties.get(unwrapped.name.text) ?? [])
        : [],
    );
  }
  if (!ts.isCallExpression(unwrapped)) return [];
  const calleeText = unwrapped.expression.getText(context.analysis.sourceFile);
  if (calleeText === "process.cwd" && unwrapped.arguments.length === 0) return [context.repoRoot];
  const name = callName(unwrapped.expression);
  if (
    name === "fileURLToPath" &&
    unwrapped.arguments[0]?.getText(context.analysis.sourceFile) === "import.meta.url"
  ) {
    return [path.join(context.repoRoot, context.scriptPath)];
  }
  if (name === "dirname" && unwrapped.arguments.length === 1) {
    return evaluateStaticDataExpression(unwrapped.arguments[0]!, context)
      .filter((value): value is string => typeof value === "string")
      .map((value) => path.dirname(value));
  }
  if (name?.startsWith("selectContractParity") && unwrapped.arguments[0]) {
    return evaluateStaticDataExpression(unwrapped.arguments[0], context);
  }
  if (name !== "join" && name !== "resolve") return [];
  let combinations: string[][] = [[]];
  for (const argument of unwrapped.arguments) {
    const parts = evaluateStaticDataExpression(argument, context).filter(
      (value): value is string => typeof value === "string",
    );
    if (parts.length === 0) return [];
    combinations = combinations.flatMap((combination) =>
      parts.map((part) => [...combination, part]),
    );
  }
  return combinations.map((parts) =>
    name === "join" ? path.join(...parts) : path.resolve(context.repoRoot, ...parts),
  );
}

function evaluateStaticIdentifier(
  identifier: tsTypes.Identifier,
  context: StaticDataEvaluationContext & { readonly resolving: Set<string> },
): readonly StaticDataValue[] {
  const declaration = context.analysis.lexical.declarationFor(identifier);
  if (!declaration) return [];
  const loopValue = context.loopValues.get(declaration);
  if (loopValue) return loopValue;
  const scalar = context.analysis.lexicalStaticPathValues.get(declaration);
  if (scalar !== undefined) return [scalar];
  const key = `${context.scriptPath}:${declaration.pos}:${declaration.end}`;
  if (context.resolving.has(key)) return [];
  context.resolving.add(key);
  try {
    const parent = declaration.parent;
    if (ts.isVariableDeclaration(parent) && parent.initializer) {
      return evaluateStaticDataExpression(parent.initializer, context);
    }
    if (parent.kind === ts.SyntaxKind.ImportSpecifier) {
      const importSpecifier = parent as typeof parent & {
        readonly propertyName?: { readonly text: string };
        readonly name: { readonly text: string };
      };
      const importDeclaration = enclosingImportDeclaration(parent);
      if (!importDeclaration || !ts.isStringLiteral(importDeclaration.moduleSpecifier)) return [];
      const importedModule = resolveLocalImportedModulePath(
        context.scriptPath,
        importDeclaration.moduleSpecifier.text,
        new Set(context.analysisCache.keys()),
      );
      if (!importedModule) return [];
      const importedAnalysis = context.analysisCache.get(importedModule);
      if (!importedAnalysis) return [];
      const importedName = importSpecifier.propertyName?.text ?? importSpecifier.name.text;
      const exportedDeclaration = findTopLevelVariableDeclaration(
        importedAnalysis.sourceFile,
        importedName,
      );
      if (!exportedDeclaration?.initializer) return [];
      return evaluateStaticDataExpression(exportedDeclaration.initializer, {
        ...context,
        scriptPath: importedModule,
        analysis: importedAnalysis,
        loopValues: new Map(),
      });
    }
    return [];
  } finally {
    context.resolving.delete(key);
  }
}

function unwrapStaticDataExpression(expression: tsTypes.Expression): tsTypes.Expression {
  let current = expression;
  while (
    ts.isParenthesizedExpression(current) ||
    ts.isAsExpression(current) ||
    ts.isSatisfiesExpression(current)
  ) {
    current = current.expression;
  }
  return current;
}

function staticPropertyName(name: tsTypes.PropertyName): string | null {
  return ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)
    ? name.text
    : null;
}

function enclosingImportDeclaration(node: tsTypes.Node): tsTypes.ImportDeclaration | null {
  let current: tsTypes.Node | undefined = node;
  while (current && !ts.isSourceFile(current)) {
    if (ts.isImportDeclaration(current)) return current;
    current = current.parent;
  }
  return null;
}

function findTopLevelVariableDeclaration(
  sourceFile: tsTypes.SourceFile,
  name: string,
): tsTypes.VariableDeclaration | null {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name) return declaration;
    }
  }
  return null;
}

function staticIterationValues(values: readonly StaticDataValue[]): readonly StaticDataValue[] {
  return values.flatMap((value) => {
    if (typeof value === "string" || value.kind === "unknown" || value.kind === "object") return [];
    if (value.kind === "array") return value.items;
    return value.entries.map((entry) => ({
      kind: "array" as const,
      items: [entry.key, entry.value],
    }));
  });
}

function bindStaticIterationValues(
  bindingName: tsTypes.BindingName,
  iterationValues: readonly StaticDataValue[],
  loopValues: Map<tsTypes.Identifier, readonly StaticDataValue[]>,
): void {
  if (ts.isIdentifier(bindingName)) {
    loopValues.set(bindingName, iterationValues);
    return;
  }
  if (bindingName.kind === ts.SyntaxKind.ArrayBindingPattern) {
    for (let index = 0; index < bindingName.elements.length; index += 1) {
      const element = bindingName.elements[index];
      if (!element || ts.isOmittedExpression(element)) continue;
      const nestedValues = iterationValues.flatMap((value) =>
        typeof value !== "string" && value.kind === "array" && value.items[index]
          ? [value.items[index]!]
          : [],
      );
      bindStaticIterationValues(element.name, nestedValues, loopValues);
    }
    return;
  }
  for (const element of bindingName.elements) {
    const propertyName = element.propertyName ? staticPropertyName(element.propertyName) : null;
    const key = propertyName ?? (ts.isIdentifier(element.name) ? element.name.text : null);
    const nestedValues =
      key === null
        ? []
        : iterationValues.flatMap((value) =>
            typeof value !== "string" && value.kind === "object"
              ? (value.properties.get(key) ?? [])
              : [],
          );
    bindStaticIterationValues(element.name, nestedValues, loopValues);
  }
}

function collectStaticPathVariables(
  repoRoot: string,
  sourceFile: tsTypes.SourceFile,
  seedValues: ReadonlyMap<string, string> = new Map(),
): ReadonlyMap<string, string> {
  const values = new Map(seedValues);
  let changed = true;
  while (changed) {
    changed = false;
    const visit = (node: tsTypes.Node): void => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.initializer &&
        !values.has(node.name.text)
      ) {
        const value = staticPathExpressionValue(node.initializer, sourceFile, values, repoRoot);
        if (value !== null) {
          values.set(node.name.text, value);
          changed = true;
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
  }
  return values;
}

function staticPathExpressionValue(
  expression: tsTypes.Expression,
  sourceFile: tsTypes.SourceFile,
  values: ReadonlyMap<string, string>,
  repoRoot: string,
): string | null {
  if (ts.isStringLiteral(expression) || ts.isNoSubstitutionTemplateLiteral(expression)) {
    return expression.text;
  }
  if (ts.isIdentifier(expression)) return values.get(expression.text) ?? null;
  if (ts.isPropertyAccessExpression(expression)) {
    const expressionText = expression.expression.getText(sourceFile);
    if (expressionText === "import.meta" && expression.name.text === "dirname") {
      return path.dirname(path.join(repoRoot, sourceFile.fileName));
    }
  }
  if (!ts.isCallExpression(expression)) return null;
  const calleeText = expression.expression.getText(sourceFile);
  if (calleeText === "process.cwd" && expression.arguments.length === 0) return repoRoot;
  const name = callName(expression.expression);
  if (
    name === "fileURLToPath" &&
    expression.arguments[0]?.getText(sourceFile) === "import.meta.url"
  ) {
    return path.join(repoRoot, sourceFile.fileName);
  }
  if (name === "dirname" && expression.arguments.length === 1) {
    const value = staticPathExpressionValue(expression.arguments[0]!, sourceFile, values, repoRoot);
    return value === null ? null : path.dirname(value);
  }
  if (name !== "join" && name !== "resolve") return null;
  const evaluatedParts = expression.arguments.map((argument) =>
    staticPathExpressionValue(argument, sourceFile, values, repoRoot),
  );
  if (evaluatedParts.some((value) => value === null)) return null;
  const parts = evaluatedParts as string[];
  return name === "join" ? path.join(...parts) : path.resolve(repoRoot, ...parts);
}

function normalizeRepositoryPath(repoRoot: string, value: string): string {
  const relative = path.isAbsolute(value) ? path.relative(repoRoot, value) : value;
  return relative.replaceAll("\\", "/").replace(/^\.\//u, "");
}

function callName(expression: tsTypes.LeftHandSideExpression): string | null {
  if (ts.isIdentifier(expression)) return expression.text;
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  return null;
}

function isFileWriteCall(expression: tsTypes.LeftHandSideExpression): boolean {
  const name = callName(expression);
  return name === "writeFileSync" || name === "writeFile";
}

function commandSegmentForWriter(command: string, writerScript: string): readonly string[] {
  const segment = command.split(/\s*&&\s*/u).find((candidate) => candidate.includes(writerScript));
  if (!segment) {
    throw new Error(`update command does not expose a segment for writer ${writerScript}`);
  }
  return splitCommandWords(segment);
}

function selfWriterCommand(
  artifactPath: string,
  writerScript: string,
  writerFlag: string | null,
): readonly string[] {
  if (artifactPath === "rust/omena-css-module-token-shape-measurement.json") {
    return [
      "node",
      "--import",
      "tsx",
      writerScript,
      "--corpus-root",
      "${OMENA_TOKEN_CORPUS_ROOT}",
      "--identity-manifest",
      "react-ts-css=${OMENA_TOKEN_IDENTITY_REACT_TS_CSS}",
      "--identity-manifest",
      "mkn=${OMENA_TOKEN_IDENTITY_MKN}",
      "--identity-manifest",
      "docusaurus=${OMENA_TOKEN_IDENTITY_DOCUSAURUS}",
      "--write",
    ];
  }
  if (artifactPath === "rust/omena-published-crate-surface-register.json") {
    return [
      "node",
      "--import",
      "tsx",
      `./${writerScript}`,
      "--initialize-from",
      "${OMENA_PUBLISHED_CRATE_REGISTRY_STATE}",
    ];
  }
  return ["node", "--import", "tsx", `./${writerScript}`, ...(writerFlag ? [writerFlag] : [])];
}

function splitCommandWords(command: string): readonly string[] {
  const words = command.match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/gu) ?? [];
  return words.map((word) =>
    (word.startsWith('"') && word.endsWith('"')) || (word.startsWith("'") && word.endsWith("'"))
      ? word.slice(1, -1)
      : word,
  );
}

export function gateCommandNamesModuleAsEntry(command: string, modulePath: string): boolean {
  return command.split(/\s*&&\s*/u).some((segment) => {
    const words = [...splitCommandWords(segment)];
    while (words[0]?.includes("=") && !words[0]?.startsWith("--")) words.shift();
    if (words[0] === "env") {
      words.shift();
      while (words[0]?.includes("=") && !words[0]?.startsWith("--")) words.shift();
    }
    if (words[0] === "pnpm") {
      words.shift();
      if ((words[0] as string | undefined) === "exec") words.shift();
    }
    const executable = path.posix.basename(words.shift() ?? "");
    if (executable === "node") {
      while (words.length > 0) {
        const word = words.shift()!;
        if (["--import", "--loader", "--require", "-r"].includes(word)) {
          words.shift();
          continue;
        }
        if (word.startsWith("--import=") || word.startsWith("--loader=")) continue;
        if (word.startsWith("-")) continue;
        return normalizeCommandModulePath(word) === modulePath;
      }
      return false;
    }
    if (executable === "tsx") {
      const entry = words.find((word) => !word.startsWith("-"));
      return entry !== undefined && normalizeCommandModulePath(entry) === modulePath;
    }
    return false;
  });
}

function normalizeCommandModulePath(value: string): string {
  return value.replace(/^\.\//u, "").replaceAll("\\", "/");
}

function repositoryReferencePathsByArtifact(
  repoRoot: string,
  artifactPaths: readonly string[],
  repositoryPaths: readonly string[],
): ReadonlyMap<string, readonly string[]> {
  const artifactPathsByBasename = new Map<string, string[]>();
  const referencesByArtifact = new Map<string, string[]>();
  for (const artifactPath of artifactPaths) {
    const basename = path.posix.basename(artifactPath);
    artifactPathsByBasename.set(basename, [
      ...(artifactPathsByBasename.get(basename) ?? []),
      artifactPath,
    ]);
    referencesByArtifact.set(artifactPath, []);
  }
  for (const candidate of repositoryPaths) {
    if (candidate === EVIDENCE_WRITER_REGISTRY_PATH || candidate.startsWith(".personal_docs/")) {
      continue;
    }
    const source = readFileSync(path.join(repoRoot, candidate), "utf8");
    for (const [basename, matchingArtifacts] of artifactPathsByBasename) {
      if (!source.includes(basename)) continue;
      for (const artifactPath of matchingArtifacts) {
        if (candidate !== artifactPath) referencesByArtifact.get(artifactPath)?.push(candidate);
      }
    }
  }
  return new Map(
    [...referencesByArtifact].map(([artifactPath, references]) => [
      artifactPath,
      references.toSorted(),
    ]),
  );
}

function writerFlagForSource(source: string, artifactPath: string): string | null {
  const preferredFlag =
    artifactPath === "packages/css-build-adapter/interface-member-snapshot.json"
      ? "--reproduce-adapter"
      : null;
  if (
    preferredFlag &&
    (source.includes(`"${preferredFlag}"`) || source.includes(`'${preferredFlag}'`))
  ) {
    return preferredFlag;
  }
  for (const flag of [
    "--write",
    "--write-serde",
    "--reproduce-adapter",
    "--write-adapter",
    "--update-census",
  ] as const) {
    if (source.includes(`"${flag}"`) || source.includes(`'${flag}'`)) return flag;
  }
  return null;
}

function specialWriterCommand(artifactPath: string): readonly string[] | null {
  switch (artifactPath) {
    case "rust/evidence-scan-surfaces.json":
      return ["pnpm", "omena-check", "evidence-surfaces", "--write"];
    case EVIDENCE_WRITER_REGISTRY_PATH:
      return ["pnpm", "omena-check", "evidence-writers", "--write"];
    default:
      return null;
  }
}

function handAuthoredProcedure(artifactPath: string): readonly string[] | null {
  switch (artifactPath) {
    case "rust/product-surface-boundary-reviews.json":
      return [
        "commit the source change",
        "run --measure and hand-edit the measured review row",
        "commit the refreshed evidence",
      ];
    case "rust/omena-domain-claim-rename-map.json":
      return ["hand-edit rename map", "run domain claim census", "review diff", "commit evidence"];
    case "rust/omena-product-path-matrix.json":
      return [
        "hand-edit path matrix",
        "run product path matrix gate",
        "review diff",
        "commit evidence",
      ];
    default:
      return null;
  }
}

function genericHandAuthoredProcedure(artifactPath: string): readonly string[] {
  return [
    `hand-edit ${artifactPath}`,
    "run every consuming check listed by the orchestrator",
    "review semantic diff",
    "commit evidence",
  ];
}

function artifactDependencies(artifactPath: string): readonly string[] {
  switch (artifactPath) {
    case "rust/omena-domain-claim-census.json":
      return ["rust/omena-response-surface-split-census.json"];
    case "rust/omena-identifier-authority-census.json":
      return [
        "rust/omena-identifier-authority-census.json",
        "rust/omena-syntax-authority-raw-scan-census.json",
      ];
    case EVIDENCE_WRITER_REGISTRY_PATH:
      return ["rust/evidence-scan-surfaces.json"];
    default:
      return [];
  }
}

export type NotPreviewableInputSeed = Omit<NotPreviewableInputV0, "gateIds">;

function deriveNotPreviewableInputs(input: {
  readonly repoRoot: string;
  readonly artifacts: readonly EvidenceArtifactRowV0[];
  readonly scanManifest: NonNullable<ReturnType<typeof loadEvidenceScanSurfaceManifest>>;
  readonly checkManifest: ReturnType<typeof loadCheckManifest>;
  readonly repositoryPathSet: ReadonlySet<string>;
}): readonly NotPreviewableInputV0[] {
  const seeds: NotPreviewableInputSeed[] = [];
  const candidateSourcePaths = new Set([
    ...input.artifacts.flatMap((artifact) => artifact.writerScripts),
    ...input.scanManifest.scanners.flatMap((row) =>
      row.disposition === "RETIRED" ? [] : [row.scannerPath],
    ),
    "packages/check-orchestrator/src/evidence/writer-runner.ts",
  ]);
  for (const sourcePath of input.repositoryPathSet) {
    if (
      (sourcePath.startsWith("scripts/") ||
        sourcePath.startsWith("packages/check-orchestrator/src/")) &&
      /\.(?:[cm]?js|ts)$/u.test(sourcePath) &&
      input.checkManifest.gates.some((gate) => gate.command.includes(sourcePath))
    ) {
      candidateSourcePaths.add(sourcePath);
    }
  }
  for (const sourcePath of candidateSourcePaths) {
    if (!/\.(?:[cm]?js|ts)$/u.test(sourcePath) || !input.repositoryPathSet.has(sourcePath)) {
      continue;
    }
    seeds.push(
      ...detectNotPreviewableInputSeedsForSource(
        sourcePath,
        readFileSync(path.join(input.repoRoot, sourcePath), "utf8"),
      ),
    );
  }
  for (const artifact of input.artifacts) {
    if (artifact.requiredEnvironmentKeys?.length) {
      seeds.push({
        ownerId: artifact.artifactPath,
        kind: "environment",
        detail: `writer command requires ${artifact.requiredEnvironmentKeys.join(", ")}`,
      });
    }
    const formatterModules = formatterInvokingModulesForArtifact(
      input.repoRoot,
      artifact,
      input.repositoryPathSet,
    );
    if (formatterModules.length === 0) continue;
    seeds.push({
      ownerId: artifact.artifactPath,
      kind: "toolchain-bytes",
      detail: `formatter bytes from ${formatterModules.join(", ")}`,
    });
  }
  const uniqueSeeds = new Map<string, NotPreviewableInputSeed>();
  for (const seed of seeds) uniqueSeeds.set(`${seed.kind}\0${seed.ownerId}`, seed);
  return [...uniqueSeeds.values()]
    .map((seed) => {
      const gateIds = gateIdsForNotPreviewableOwner(
        seed.ownerId,
        seed.kind,
        input.artifacts,
        input.checkManifest,
      );
      if (gateIds.length === 0) {
        throw new Error(`not-previewable owner has no derived gate ids: ${seed.ownerId}`);
      }
      return { ...seed, gateIds };
    })
    .toSorted(
      (left, right) =>
        compareText(left.kind, right.kind) ||
        compareText(left.ownerId, right.ownerId) ||
        compareText(left.detail, right.detail),
    );
}

export function detectNotPreviewableInputSeedsForSource(
  sourcePath: string,
  source: string,
): readonly NotPreviewableInputSeed[] {
  if (sourcePath === "packages/check-orchestrator/src/evidence/writer-registry.ts") return [];
  const sourceFile = ts.createSourceFile(sourcePath, source, ts.ScriptTarget.Latest, true);
  const seeds: NotPreviewableInputSeed[] = [];
  if (
    sourcePath.startsWith("scripts/check-rust-omena-diff-test-") &&
    sourceContainsIdentifier(sourceFile, new Set(["reviewAfter", "review_after"]))
  ) {
    seeds.push({
      ownerId: sourcePath,
      kind: "calendar-time",
      detail: "reviewAfter policy is evaluated from calendar state",
    });
  }
  if (sourceReadsDynamicProcessEnvironment(sourceFile)) {
    seeds.push({
      ownerId: sourcePath,
      kind: "environment",
      detail: "process environment is read by the governed module",
    });
  }
  if (sourceContainsCalledStringLiteral(sourceFile, new Set(["rev-list"]))) {
    seeds.push({
      ownerId: sourcePath,
      kind: "git-history",
      detail: "git rev-list history is read by the governed module",
    });
  }
  if (
    (sourceContainsIdentifier(sourceFile, new Set(["beforeInputDigests"])) &&
      sourceContainsIdentifier(sourceFile, new Set(["afterInputDigests"]))) ||
    (sourceContainsIdentifier(sourceFile, new Set(["inputsBefore"])) &&
      sourceContainsIdentifier(sourceFile, new Set(["inputsAfter"])))
  ) {
    seeds.push({
      ownerId: sourcePath,
      kind: "concurrent-worktree",
      detail: "writer acceptance depends on persistent mid-run worktree state",
    });
  }
  if (
    sourceContainsCalledStringLiteral(sourceFile, new Set(["clone", "fetch"])) ||
    sourceContainsArgumentFlagUse(
      sourceFile,
      new Set(["--corpus-root", "--identity-manifest", "--wpt-root"]),
    )
  ) {
    seeds.push({
      ownerId: sourcePath,
      kind: "network-or-external-checkout",
      detail: "network checkout or operator-supplied external corpus bytes are read",
    });
  }
  if (
    sourceContainsArgumentFlagUse(sourceFile, new Set(["--omena-bin"])) ||
    sourceContainsIdentifier(sourceFile, new Set(["OMENA_LINT_CENSUS_BINARY"]))
  ) {
    seeds.push({
      ownerId: sourcePath,
      kind: "built-binary",
      detail: "an operator-supplied compiled binary is read",
    });
  }
  return seeds;
}

function sourceContainsIdentifier(
  sourceFile: tsTypes.SourceFile,
  names: ReadonlySet<string>,
): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (ts.isIdentifier(node) && names.has(node.text)) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function sourceReadsDynamicProcessEnvironment(sourceFile: tsTypes.SourceFile): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (
      ts.isElementAccessExpression(node) &&
      ts.isPropertyAccessExpression(node.expression) &&
      ts.isIdentifier(node.expression.expression) &&
      node.expression.expression.text === "process" &&
      node.expression.name.text === "env" &&
      node.argumentExpression !== undefined &&
      !ts.isStringLiteral(node.argumentExpression) &&
      !ts.isNoSubstitutionTemplateLiteral(node.argumentExpression)
    ) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function sourceContainsArgumentFlagUse(
  sourceFile: tsTypes.SourceFile,
  flags: ReadonlySet<string>,
): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (
      (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) &&
      flags.has(node.text) &&
      (ts.isCallExpression(node.parent) ||
        ts.isBinaryExpression(node.parent) ||
        hasCallExpressionAncestor(node))
    ) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function sourceContainsCalledStringLiteral(
  sourceFile: tsTypes.SourceFile,
  values: ReadonlySet<string>,
): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (
      (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) &&
      values.has(node.text) &&
      hasCallExpressionAncestor(node)
    ) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function formatterInvokingModulesForArtifact(
  repoRoot: string,
  artifact: EvidenceArtifactRowV0,
  repositoryPathSet: ReadonlySet<string>,
): readonly string[] {
  return detectFormatterInvokingModules(repoRoot, artifact.writerScripts, repositoryPathSet);
}

export function detectFormatterInvokingModules(
  repoRoot: string,
  writerScripts: readonly string[],
  repositoryPathSet: ReadonlySet<string>,
): readonly string[] {
  const formatterModules = new Set<string>();
  for (const writerScript of writerScripts) {
    if (!/\.(?:[cm]?js|ts)$/u.test(writerScript) || !repositoryPathSet.has(writerScript)) continue;
    const source = readFileSync(path.join(repoRoot, writerScript), "utf8");
    if (sourceInvokesOxfmt(writerScript, source)) {
      formatterModules.add(writerScript);
      continue;
    }
    const sourceFile = ts.createSourceFile(writerScript, source, ts.ScriptTarget.Latest, true);
    const calledIdentifiers = new Set<string>();
    const visit = (node: tsTypes.Node): void => {
      if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
        calledIdentifiers.add(node.expression.text);
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
    for (const statement of sourceFile.statements) {
      if (
        !ts.isImportDeclaration(statement) ||
        !ts.isStringLiteral(statement.moduleSpecifier) ||
        !statement.importClause?.namedBindings ||
        !ts.isNamedImports(statement.importClause.namedBindings)
      ) {
        continue;
      }
      const importedModule = resolveLocalImportedModulePath(
        writerScript,
        statement.moduleSpecifier.text,
        repositoryPathSet,
      );
      if (!importedModule) continue;
      const formatterImportIsCalled = statement.importClause.namedBindings.elements.some(
        (element) => calledIdentifiers.has(element.name.text),
      );
      if (formatterImportIsCalled) {
        const importedSource = readFileSync(path.join(repoRoot, importedModule), "utf8");
        const formatterExportIsCalled = statement.importClause.namedBindings.elements.some(
          (element) =>
            calledIdentifiers.has(element.name.text) &&
            exportedFunctionInvokesOxfmt(
              importedModule,
              importedSource,
              element.propertyName?.text ?? element.name.text,
            ),
        );
        if (formatterExportIsCalled) formatterModules.add(importedModule);
      }
    }
  }
  return [...formatterModules].toSorted();
}

function sourceInvokesOxfmt(scriptPath: string, source: string): boolean {
  const sourceFile = ts.createSourceFile(scriptPath, source, ts.ScriptTarget.Latest, true);
  const formatterImports = importedOxfmtBindings(sourceFile);
  let invokesFormatter = false;
  const visit = (node: tsTypes.Node): void => {
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      formatterImports.has(node.expression.text)
    ) {
      invokesFormatter = true;
      return;
    }
    if (
      (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) &&
      /(?:^|[/\\])oxfmt(?:$|[/\\])|^oxfmt$/u.test(node.text) &&
      hasCallExpressionAncestor(node)
    ) {
      invokesFormatter = true;
      return;
    }
    if (!invokesFormatter) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return invokesFormatter;
}

function exportedFunctionInvokesOxfmt(
  scriptPath: string,
  source: string,
  exportedName: string,
): boolean {
  const sourceFile = ts.createSourceFile(scriptPath, source, ts.ScriptTarget.Latest, true);
  const formatterImports = importedOxfmtBindings(sourceFile);
  const callableBodies = new Map<string, tsTypes.Node>();
  const exportedBindings = new Map<string, string>();
  for (const statement of sourceFile.statements) {
    if (ts.isFunctionDeclaration(statement) && statement.name && statement.body) {
      callableBodies.set(statement.name.text, statement.body);
      if (hasExportModifier(statement))
        exportedBindings.set(statement.name.text, statement.name.text);
      continue;
    }
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        if (
          ts.isIdentifier(declaration.name) &&
          declaration.initializer &&
          (ts.isArrowFunction(declaration.initializer) ||
            ts.isFunctionExpression(declaration.initializer))
        ) {
          callableBodies.set(declaration.name.text, declaration.initializer.body);
          if (hasExportModifier(statement)) {
            exportedBindings.set(declaration.name.text, declaration.name.text);
          }
        }
      }
      continue;
    }
    if (
      ts.isExportDeclaration(statement) &&
      statement.exportClause &&
      statement.exportClause.kind === ts.SyntaxKind.NamedExports
    ) {
      for (const element of statement.exportClause.elements) {
        exportedBindings.set(element.name.text, element.propertyName?.text ?? element.name.text);
      }
    }
  }

  const entry = exportedBindings.get(exportedName);
  if (!entry) return false;
  const visited = new Set<string>();
  const invokesFrom = (functionName: string): boolean => {
    if (visited.has(functionName)) return false;
    visited.add(functionName);
    const body = callableBodies.get(functionName);
    if (!body) return false;
    let invokesFormatter = false;
    const visit = (node: tsTypes.Node): void => {
      if (invokesFormatter) return;
      if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
        if (formatterImports.has(node.expression.text) || invokesFrom(node.expression.text)) {
          invokesFormatter = true;
          return;
        }
      }
      if (
        (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) &&
        /(?:^|[/\\])oxfmt(?:$|[/\\])|^oxfmt$/u.test(node.text) &&
        hasCallExpressionAncestor(node)
      ) {
        invokesFormatter = true;
        return;
      }
      ts.forEachChild(node, visit);
    };
    visit(body);
    return invokesFormatter;
  };
  return invokesFrom(entry);
}

function importedOxfmtBindings(sourceFile: tsTypes.SourceFile): ReadonlySet<string> {
  const bindings = new Set<string>();
  for (const statement of sourceFile.statements) {
    if (
      !ts.isImportDeclaration(statement) ||
      !ts.isStringLiteral(statement.moduleSpecifier) ||
      statement.moduleSpecifier.text !== "oxfmt" ||
      !statement.importClause
    ) {
      continue;
    }
    if (statement.importClause.name) bindings.add(statement.importClause.name.text);
    const namedBindings = statement.importClause.namedBindings;
    if (namedBindings && ts.isNamespaceImport(namedBindings)) {
      bindings.add(namedBindings.name.text);
    } else if (namedBindings && ts.isNamedImports(namedBindings)) {
      for (const element of namedBindings.elements) bindings.add(element.name.text);
    }
  }
  return bindings;
}

function hasExportModifier(node: tsTypes.Node): boolean {
  let exported = false;
  ts.forEachChild(node, (child) => {
    if (child.kind === ts.SyntaxKind.ExportKeyword) exported = true;
  });
  return exported;
}

function hasCallExpressionAncestor(node: tsTypes.Node): boolean {
  let current: tsTypes.Node | undefined = node.parent;
  while (current && !ts.isSourceFile(current)) {
    if (ts.isCallExpression(current) || ts.isNewExpression(current)) return true;
    if (
      ts.isFunctionDeclaration(current) ||
      ts.isFunctionExpression(current) ||
      ts.isArrowFunction(current)
    ) {
      return false;
    }
    current = current.parent;
  }
  return false;
}

function gateIdsForNotPreviewableOwner(
  ownerId: string,
  kind: NotPreviewableInputKind,
  artifacts: readonly EvidenceArtifactRowV0[],
  checkManifest: ReturnType<typeof loadCheckManifest>,
): readonly string[] {
  const directEntryGateIds = new Set<string>();
  for (const gate of checkManifest.gates) {
    if (gateCommandNamesModuleAsEntry(gate.command, ownerId)) {
      directEntryGateIds.add(gate.id);
    }
  }

  const artifactOwner = artifacts.find((row) => row.artifactPath === ownerId);
  if (artifactOwner && artifactOwner.writerGateIds.length > 0) {
    return [...artifactOwner.writerGateIds].toSorted();
  }

  const writerGateIds = new Set(
    artifacts
      .filter((row) => row.writerScripts.includes(ownerId))
      .flatMap((row) => row.writerGateIds),
  );
  if (kind === "toolchain-bytes" || kind === "concurrent-worktree") {
    if (writerGateIds.size > 0) return [...writerGateIds].toSorted();
    if (directEntryGateIds.size > 0) return [...directEntryGateIds].toSorted();
  } else if (
    directEntryGateIds.size === 1 ||
    (ownerId.startsWith("scripts/") && directEntryGateIds.size > 0)
  ) {
    // A marker in a shared CLI module blinds only commands that name that
    // module as their unambiguous executable entrypoint. A multiplexed module
    // cannot attribute one function-local marker to every sibling command;
    // a dedicated script, by contrast, executes its module body for every
    // command that names it.
    return [...directEntryGateIds].toSorted();
  } else if (writerGateIds.size > 0) {
    return [...writerGateIds].toSorted();
  }

  const gateIds = new Set<string>();
  if (
    gateIds.size === 0 &&
    checkManifest.gates.some((gate) => gate.id === "tooling/evidence-affected-map")
  ) {
    // Standalone W2/tool owners have no product gate. Their only executable
    // governance boundary is the registry/preview meta-gate itself.
    gateIds.add("tooling/evidence-affected-map");
  }
  return [...gateIds].toSorted();
}

function readPackageScripts(repoRoot: string): Readonly<Record<string, string>> {
  const parsed = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
    readonly scripts?: Readonly<Record<string, string>>;
  };
  return parsed.scripts ?? {};
}

function validateEvidenceWriterRegistry(registry: EvidenceWriterRegistryV0): void {
  if (registry.schemaVersion !== "0") {
    throw new Error(
      `unsupported evidence writer registry schema ${String(registry.schemaVersion)}`,
    );
  }
  if (
    registry.commandAuthority?.modulePath !== EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH ||
    !/^[0-9a-f]{64}$/u.test(registry.commandAuthority.sha256) ||
    registry.commandAuthority.declaredCommandCount < 1 ||
    registry.commandAuthority.declaredOutputCount < 1 ||
    registry.commandAuthority.declaredStaticWriteOutputCount < 1
  ) {
    throw new Error("evidence writer command authority is missing or invalid");
  }
  const paths = registry.artifacts.map((row) => row.artifactPath);
  if (
    new Set(paths).size !== paths.length ||
    paths.some((entry, index) => entry !== [...paths].toSorted()[index])
  ) {
    throw new Error("evidence writer artifacts must be unique and sorted");
  }
  if (registry.artifacts.some((row) => !["W1", "W2", "W3", "W4"].includes(row.classification))) {
    throw new Error("evidence writer registry contains an unclassified artifact");
  }
  for (const row of registry.artifacts) {
    if ((row.classification === "W1" || row.classification === "W2") && !row.writeCommand) {
      throw new Error(`writeable evidence artifact lacks a declared command: ${row.artifactPath}`);
    }
    if (
      row.classification === "W1" &&
      (row.freshReproductionRequired === true) === Boolean(row.freshReproductionExemption)
    ) {
      throw new Error(
        `W1 evidence artifact must require fresh reproduction or carry one exemption: ${row.artifactPath}`,
      );
    }
    if (
      row.freshReproductionExemption &&
      (row.freshReproductionExemption.kind !== "reads-own-output" ||
        row.freshReproductionExemption.reason.trim().length === 0)
    ) {
      throw new Error(`evidence fresh-reproduction exemption is invalid: ${row.artifactPath}`);
    }
    if (
      (row.classification === "W1" || row.classification === "W2") &&
      row.inputPaths.length === 0 &&
      row.inputScannerPaths.length === 0 &&
      row.inputArtifactPaths.length === 0 &&
      (row.requiredEnvironmentKeys?.length ?? 0) === 0
    ) {
      throw new Error(
        `writeable evidence artifact has an unexplained empty input set: ${row.artifactPath}`,
      );
    }
    if (
      row.artifactPath === "rust/omena-published-crate-surface-register.json" &&
      (!row.writeCommand?.includes("--initialize-from") ||
        !row.writeCommand.includes("${OMENA_PUBLISHED_CRATE_REGISTRY_STATE}") ||
        !row.requiredEnvironmentKeys?.includes("OMENA_PUBLISHED_CRATE_REGISTRY_STATE"))
    ) {
      throw new Error(
        "published-crate surface register writer must bind a measured registry-state input",
      );
    }
  }
  const artifactPathSet = new Set(paths);
  for (const row of registry.artifacts) {
    for (const dependency of row.inputArtifactPaths) {
      if (!artifactPathSet.has(dependency)) {
        throw new Error(
          `evidence artifact ${row.artifactPath} references unknown input artifact ${dependency}`,
        );
      }
    }
  }
  const ownerIds = registry.notPreviewableInputs.map((entry) => entry.ownerId);
  for (const [index, ownerId] of ownerIds.entries()) {
    if (!isRepositoryModulePath(ownerId)) {
      throw new Error(`not-previewable owner must be a repository module path: ${ownerId}`);
    }
    const gateIds = registry.notPreviewableInputs[index]!.gateIds;
    if (
      gateIds.length === 0 ||
      new Set(gateIds).size !== gateIds.length ||
      gateIds.some((gateId, gateIndex) => !gateId || gateId !== [...gateIds].toSorted()[gateIndex])
    ) {
      throw new Error(
        `not-previewable owner gate ids must be non-empty, unique, and sorted: ${ownerId}`,
      );
    }
  }
  const kinds = new Set(registry.notPreviewableInputs.map((entry) => entry.kind));
  for (const kind of NOT_PREVIEWABLE_INPUT_KINDS) {
    if (!kinds.has(kind)) throw new Error(`not-previewable input kind is missing: ${kind}`);
  }
}

function isRepositoryModulePath(value: string): boolean {
  return (
    value.length > 0 &&
    !path.posix.isAbsolute(value) &&
    !value.startsWith("../") &&
    value.includes("/") &&
    path.posix.extname(value).length > 0
  );
}

export function writerRegistryDigest(registry: EvidenceWriterRegistryV0): string {
  return createHash("sha256").update(renderEvidenceWriterRegistry(registry)).digest("hex");
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
