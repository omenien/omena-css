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

export const EVIDENCE_WRITER_REGISTRY_PATH = "rust/evidence-writer-registry.json";

const ROOT_RUST_ARTIFACT_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/writer-registry.ts",
  mode: "index",
  pathspecs: ["rust"],
  includeUntracked: true,
  excludes: [],
});
const SCRIPT_SOURCE_SURFACE = defineScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/writer-registry.ts",
  mode: "index",
  pathspecs: ["scripts/**"],
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

export interface EvidenceWriterRegistryV0 {
  readonly schemaVersion: "0";
  readonly generatedBy: "pnpm omena-check evidence-writers --write";
  readonly artifacts: readonly EvidenceArtifactRowV0[];
  readonly notPreviewableInputs: readonly NotPreviewableInputV0[];
}

export interface EvidenceArtifactRowV0 {
  readonly artifactPath: string;
  readonly classification: EvidenceArtifactClassification;
  readonly writerNodeKind: "normal" | "self-ratchet";
  readonly writerScripts: readonly string[];
  readonly writeCommand?: readonly string[];
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

export function buildEvidenceWriterRegistry(repoRoot: string): EvidenceWriterRegistryV0 {
  const discovery = discoverEvidenceArtifacts(repoRoot);
  const { repositoryPaths, scriptPaths, artifactPaths, scriptAnalysisCache } = discovery;
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
  const writerScriptsByArtifact = new Map(
    artifactPaths.map((artifactPath) => [
      artifactPath,
      findArtifactWriterScripts(repoRoot, artifactPath, scriptPaths, scriptAnalysisCache),
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
    artifacts,
    notPreviewableInputs: declaredNotPreviewableInputs(),
  } satisfies EvidenceWriterRegistryV0;
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

interface WriterScriptAnalysis {
  readonly source: string;
  readonly sourceFile: tsTypes.SourceFile;
  readonly staticPathVariables: ReadonlyMap<string, string>;
}

interface EvidenceArtifactDiscovery {
  readonly repositoryPaths: readonly string[];
  readonly scriptPaths: readonly string[];
  readonly artifactPaths: readonly string[];
  readonly scriptAnalysisCache: Map<string, WriterScriptAnalysis>;
}

function discoverEvidenceArtifacts(repoRoot: string): EvidenceArtifactDiscovery {
  const repositoryPaths = resolveScanSurface(REPOSITORY_REFERENCE_SURFACE, { repoRoot }).paths;
  const scriptPaths = resolveScanSurface(SCRIPT_SOURCE_SURFACE, { repoRoot }).paths.filter(
    (candidate) => /\.(?:[cm]?js|ts)$/u.test(candidate),
  );
  const scriptAnalysisCache = new Map<string, WriterScriptAnalysis>();
  const rootArtifacts = [
    ...new Set([
      ...resolveScanSurface(ROOT_RUST_ARTIFACT_SURFACE, { repoRoot }).paths.filter(
        (candidate) => path.posix.dirname(candidate) === "rust" && candidate.endsWith(".json"),
      ),
      EVIDENCE_WRITER_REGISTRY_PATH,
    ]),
  ];
  const trackedWriterOutputs = repositoryPaths.filter(
    (candidate) =>
      candidate.endsWith(".json") &&
      findArtifactWriterScripts(repoRoot, candidate, scriptPaths, scriptAnalysisCache).length > 0,
  );
  return {
    repositoryPaths,
    scriptPaths,
    artifactPaths: [...new Set([...rootArtifacts, ...trackedWriterOutputs])].toSorted(),
    scriptAnalysisCache,
  };
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
  const specialCommand = specialWriterCommand(artifactPath);
  const manualProcedure = handAuthoredProcedure(artifactPath);
  const writerFlag =
    writerScripts.length === 1
      ? writerFlagForSource(readFileSync(path.join(input.repoRoot, writerScripts[0]!), "utf8"))
      : null;
  const classification: EvidenceArtifactClassification =
    specialCommand || updateCommands.length > 0
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
  const requiredEnvironmentKeys =
    artifactPath === "rust/omena-css-module-token-shape-measurement.json"
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
  const allInputs = [...new Set([...literalInputs, ...moduleInputs])].filter(
    (candidate) => !writerOutputPaths.has(candidate),
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
    ...new Set([
      ...writerScripts.flatMap((script) => scannerGateIds.get(script) ?? []),
      ...(artifactPath === "rust/evidence-scan-surfaces.json"
        ? ["tooling/evidence-scan-surfaces"]
        : []),
      ...(artifactPath === EVIDENCE_WRITER_REGISTRY_PATH ? ["tooling/evidence-affected-map"] : []),
    ]),
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
  return {
    artifactPath,
    classification,
    writerNodeKind:
      artifactPath === "rust/omena-identifier-authority-census.json" ? "self-ratchet" : "normal",
    writerScripts,
    ...(writeCommand ? { writeCommand } : {}),
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

function findArtifactWriterScripts(
  repoRoot: string,
  artifactPath: string,
  scriptPaths: readonly string[],
  analysisCache: Map<string, WriterScriptAnalysis>,
): readonly string[] {
  const basename = path.posix.basename(artifactPath);
  const requireExactPath = path.posix.dirname(artifactPath) !== "rust";
  const writers: string[] = [];
  for (const scriptPath of scriptPaths) {
    let analysis = analysisCache.get(scriptPath);
    if (!analysis) {
      const source = readFileSync(path.join(repoRoot, scriptPath), "utf8");
      const sourceFile = ts.createSourceFile(scriptPath, source, ts.ScriptTarget.Latest, true);
      analysis = {
        source,
        sourceFile,
        staticPathVariables: collectStaticPathVariables(repoRoot, sourceFile),
      };
      analysisCache.set(scriptPath, analysis);
    }
    const { source, sourceFile, staticPathVariables } = analysis;
    if (!source.includes(basename)) continue;
    if (requireExactPath) {
      let writesExactArtifact = false;
      const findExactWrite = (node: tsTypes.Node): void => {
        if (ts.isCallExpression(node) && callName(node.expression) === "writeFileSync") {
          const firstArgument = node.arguments[0];
          const resolved = firstArgument
            ? staticPathExpressionValue(firstArgument, sourceFile, staticPathVariables, repoRoot)
            : null;
          if (resolved && normalizeRepositoryPath(repoRoot, resolved) === artifactPath) {
            writesExactArtifact = true;
          }
        }
        if (!writesExactArtifact) ts.forEachChild(node, findExactWrite);
      };
      findExactWrite(sourceFile);
      if (writesExactArtifact) writers.push(scriptPath);
      continue;
    }
    const pathVariables = new Set<string>();
    let changed = true;
    while (changed) {
      changed = false;
      const collectPaths = (node: tsTypes.Node): void => {
        if (
          ts.isVariableDeclaration(node) &&
          ts.isIdentifier(node.name) &&
          node.initializer &&
          (node.initializer.getText(sourceFile).includes(basename) ||
            expressionUsesIdentifier(node.initializer, pathVariables)) &&
          !pathVariables.has(node.name.text)
        ) {
          pathVariables.add(node.name.text);
          changed = true;
        }
        ts.forEachChild(node, collectPaths);
      };
      collectPaths(sourceFile);
    }
    let writesArtifact = false;
    const findWrite = (node: tsTypes.Node): void => {
      if (!ts.isCallExpression(node)) {
        ts.forEachChild(node, findWrite);
        return;
      }
      const callee = node.expression;
      const calleeName = ts.isIdentifier(callee)
        ? callee.text
        : ts.isPropertyAccessExpression(callee)
          ? callee.name.text
          : "";
      const firstArgument = node.arguments[0];
      if (
        calleeName === "writeFileSync" &&
        firstArgument &&
        (firstArgument.getText(sourceFile).includes(basename) ||
          expressionUsesIdentifier(firstArgument, pathVariables))
      ) {
        writesArtifact = true;
      }
      ts.forEachChild(node, findWrite);
    };
    findWrite(sourceFile);
    if (writesArtifact) writers.push(scriptPath);
  }
  return writers.toSorted();
}

function collectStaticPathVariables(
  repoRoot: string,
  sourceFile: tsTypes.SourceFile,
): ReadonlyMap<string, string> {
  const values = new Map<string, string>();
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
  if (!ts.isCallExpression(expression)) return null;
  const calleeText = expression.expression.getText(sourceFile);
  if (calleeText === "process.cwd" && expression.arguments.length === 0) return repoRoot;
  const name = callName(expression.expression);
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

function expressionUsesIdentifier(
  expression: tsTypes.Expression,
  identifiers: ReadonlySet<string>,
): boolean {
  let found = false;
  const visit = (node: tsTypes.Node): void => {
    if (ts.isIdentifier(node) && identifiers.has(node.text)) found = true;
    if (!found) ts.forEachChild(node, visit);
  };
  visit(expression);
  return found;
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

function writerFlagForSource(source: string): string | null {
  for (const flag of ["--write", "--write-serde", "--update-census"] as const) {
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

function declaredNotPreviewableInputs(): readonly NotPreviewableInputV0[] {
  return [
    {
      ownerId: "scripts/check-rust-omena-diff-test-dialect-seed.ts",
      kind: "calendar-time",
      detail: "reviewAfter expiration is evaluated from the wall calendar",
    },
    {
      ownerId: "scripts/check-rust-omena-diff-test-external-corpus-differential.ts",
      kind: "calendar-time",
      detail: "reviewAfter expiration is evaluated from the wall calendar",
    },
    {
      ownerId: "scripts/check-rust-omena-diff-test-wpt-expectations.ts",
      kind: "calendar-time",
      detail: "reviewAfter expiration is evaluated from the wall calendar",
    },
    {
      ownerId: "scripts/check-rust-omena-diff-test-wpt-promotion.ts",
      kind: "calendar-time",
      detail: "reviewAfter expiration is evaluated from the wall calendar",
    },
    {
      ownerId: "scripts/check-rust-omena-diff-test-wpt-seed.ts",
      kind: "calendar-time",
      detail: "reviewAfter expiration is evaluated from the wall calendar",
    },
    {
      ownerId: "scripts/check-rust-omena-identifier-authority-census.selftest.mjs",
      kind: "environment",
      detail:
        "partition mutation job depends on OMENA_IDENTIFIER_AUTHORITY_MUTATION_PARTITION_COUNT and INDEX",
    },
    {
      ownerId: "scripts/check-rust-omena-identifier-authority-census.selftest.mjs",
      kind: "environment",
      detail: "checker-spawn falsifiers inject process environment keys",
    },
    {
      ownerId: "scripts/check-rust-published-crate-surface-register.ts",
      kind: "environment",
      detail: "initialization requires an operator-supplied measured registry state path",
    },
    {
      ownerId: "scripts/check-rust-omena-crate-boundary-reviews.ts",
      kind: "git-history",
      detail: "rev-list commit distance requires commit then measure then hand-edit then commit",
    },
    {
      ownerId: "packages/check-orchestrator/src/evidence/writer-runner.ts",
      kind: "concurrent-worktree",
      detail: "persistent mid-run input skew is rejected; ABA is outside the claim",
    },
    {
      ownerId: "scripts/oss-corpus-farm.ts",
      kind: "network-or-external-checkout",
      detail: "pinned external corpus fetch and checkout are not predicted from a local diff",
    },
    {
      ownerId: "scripts/measure-css-module-token-shapes.ts",
      kind: "network-or-external-checkout",
      detail:
        "the declared writer requires operator-supplied external corpus and identity-manifest paths",
    },
    {
      ownerId: "scripts/oss-corpus-farm.ts",
      kind: "built-binary",
      detail: "--omena-bin and OMENA_LINT_CENSUS_BINARY supply compiled bytes",
    },
    {
      ownerId: "packages/check-orchestrator/src/evidence/scan-surface-manifest.ts",
      kind: "toolchain-bytes",
      detail: "formatter output depends on the pnpm-lock-pinned oxfmt binary",
    },
  ];
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
  for (const ownerId of ownerIds) {
    if (!isRepositoryModulePath(ownerId)) {
      throw new Error(`not-previewable owner must be a repository module path: ${ownerId}`);
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
