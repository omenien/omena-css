import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const repositoryRoot = process.cwd();
const rustRoot = `${repositoryRoot}/rust`;
const binary = `${rustRoot}/target/debug/omena`;
const argumentsSet = new Set(process.argv.slice(2));

function cargoTest(testName: string): {
  readonly status: number | null;
  readonly transcript: string;
} {
  const run = spawnSync(
    "cargo",
    ["test", "-p", "omena-bundler", testName, "--", "--exact", "--nocapture"],
    {
      cwd: rustRoot,
      encoding: "utf8",
    },
  );
  return {
    status: run.status,
    transcript: `${run.stdout ?? ""}\n${run.stderr ?? ""}`,
  };
}

function queryTest(testName: string): {
  readonly status: number | null;
  readonly transcript: string;
} {
  const run = spawnSync(
    "cargo",
    ["test", "-p", "omena-query", testName, "--", "--exact", "--nocapture"],
    {
      cwd: rustRoot,
      encoding: "utf8",
    },
  );
  return {
    status: run.status,
    transcript: `${run.stdout ?? ""}\n${run.stderr ?? ""}`,
  };
}

function transformPassesTest(testName: string): {
  readonly command: string;
  readonly status: number | null;
  readonly transcript: string;
} {
  const args = ["test", "-p", "omena-transform-passes", testName, "--", "--exact", "--nocapture"];
  const run = spawnSync("cargo", args, {
    cwd: rustRoot,
    encoding: "utf8",
  });
  return {
    command: `cd ${rustRoot} && cargo ${args.join(" ")}`,
    status: run.status,
    transcript: `${run.stdout ?? ""}\n${run.stderr ?? ""}`,
  };
}

const absenceTestName = argumentsSet.has("--inject-recorded-module-reachability-absence-loss")
  ? "injected_missing_module_reachability_absence_test"
  : "tests::analyzed_empty_semantic_reachability_narrows_the_module_to_no_symbols";
const absenceRun = cargoTest(absenceTestName);
const missingRun = cargoTest(
  "tests::missing_semantic_reachability_preserves_symbols_with_typed_absence",
);
interface ReachabilityAnalysisCell {
  readonly state: "analyzed" | "unanalyzed";
  readonly cause: "inputNotProvided" | null;
  readonly analyzedEmptyCount: number;
  readonly unanalyzedCount: number;
  readonly projectedClassNameCount: number;
}
const reachabilityAnalysisCells = `${absenceRun.transcript}\n${missingRun.transcript}`
  .split("\n")
  .filter((line) => line.startsWith("REACHABILITY_ANALYSIS_CELL="))
  .map(
    (line) =>
      JSON.parse(line.slice("REACHABILITY_ANALYSIS_CELL=".length)) as ReachabilityAnalysisCell,
  );

// FALSIFIER: id=closed-world-admission-reachability-absence class=liveness via=--inject-recorded-module-reachability-absence-loss producer=can-fail owner=closed-world-admission-tier entry=silent-full-module-retention
assert.equal(
  absenceRun.status === 0 &&
    absenceRun.transcript.includes(
      "test tests::analyzed_empty_semantic_reachability_narrows_the_module_to_no_symbols ... ok",
    ) &&
    missingRun.status === 0 &&
    missingRun.transcript.includes(
      "test tests::missing_semantic_reachability_preserves_symbols_with_typed_absence ... ok",
    ) &&
    reachabilityAnalysisCells.some(
      (cell) =>
        cell.state === "analyzed" &&
        cell.cause === null &&
        cell.analyzedEmptyCount === 1 &&
        cell.unanalyzedCount === 0 &&
        cell.projectedClassNameCount === 0,
    ) &&
    reachabilityAnalysisCells.some(
      (cell) =>
        cell.state === "unanalyzed" &&
        cell.cause === "inputNotProvided" &&
        cell.analyzedEmptyCount === 0 &&
        cell.unanalyzedCount === 1 &&
        cell.projectedClassNameCount === 2,
    ),
  true,
  `an analyzed empty reachability set was collapsed into missing analysis:\n${absenceRun.transcript}`,
);

const reachabilityCorpusRun = queryTest(
  "tests::consumer_reachability::attributed_empty_projection_removes_unreachable_parse_derived_names",
);
interface ReachabilityCorpusCell {
  readonly fixtureId: string;
  readonly state: "analyzed" | "unanalyzed";
  readonly cause: "inputNotProvided" | null;
  readonly closedWorldOutcome: "closed" | "open";
  readonly semanticRemovalCount: number;
  readonly outputCss: string;
  readonly productBytesEqualToConservative: boolean;
}
const reachabilityCorpusCells = reachabilityCorpusRun.transcript
  .split("\n")
  .filter((line) => line.startsWith("REACHABILITY_CORPUS_CELL="))
  .map(
    (line) => JSON.parse(line.slice("REACHABILITY_CORPUS_CELL=".length)) as ReachabilityCorpusCell,
  );
const reachabilityCorpusFixtureIds = [
  ...new Set(reachabilityCorpusCells.map((cell) => cell.fixtureId)),
].toSorted();
const analyzedProductByteDeltaInputIds = reachabilityCorpusCells
  .filter(
    (cell) =>
      cell.state === "analyzed" &&
      cell.closedWorldOutcome === "closed" &&
      cell.semanticRemovalCount > 0 &&
      !cell.productBytesEqualToConservative,
  )
  .map((cell) => cell.fixtureId)
  .toSorted();
const conservativeByteIdenticalInputIds = reachabilityCorpusCells
  .filter(
    (cell) =>
      cell.state === "unanalyzed" &&
      cell.cause === "inputNotProvided" &&
      cell.closedWorldOutcome === "closed" &&
      cell.semanticRemovalCount === 0 &&
      cell.productBytesEqualToConservative,
  )
  .map((cell) => cell.fixtureId)
  .toSorted();
const conservativeUnanalyzedCount = reachabilityAnalysisCells
  .filter((cell) => cell.state === "unanalyzed")
  .reduce((count, cell) => count + cell.unanalyzedCount, 0);

// FALSIFIER: id=closed-world-admission-reachability-corpus class=liveness via=restore-analyzed-empty-filter producer=can-fail owner=closed-world-admission-tier entry=product-byte-diff-by-analysis-state
assert.equal(
  reachabilityCorpusRun.status === 0 &&
    reachabilityCorpusFixtureIds.length >= 2 &&
    reachabilityCorpusCells.length === reachabilityCorpusFixtureIds.length * 2 &&
    analyzedProductByteDeltaInputIds.length === reachabilityCorpusFixtureIds.length &&
    conservativeByteIdenticalInputIds.length === reachabilityCorpusFixtureIds.length &&
    conservativeUnanalyzedCount > 0,
  true,
  `the product corpus did not distinguish analyzed-empty from conservative reachability:\n${reachabilityCorpusRun.transcript}`,
);

const matrixTestName = argumentsSet.has("--inject-composes-admission-matrix-loss")
  ? "tests::runtime_boundary::injected_missing_composes_admission_matrix"
  : "tests::runtime_boundary::closed_world_composes_admission_covers_tri_state_and_independent_producer_matrix";
const matrixRun = transformPassesTest(matrixTestName);

interface MatrixCell {
  readonly state: string;
  readonly fixture: string;
  readonly refusedCount: number;
  readonly reasonKind: string | null;
  readonly evidenceScope: string | null;
  readonly outputCss: string;
  readonly observedBundleEdgeCount: number;
}

const matrixCells = matrixRun.transcript
  .split("\n")
  .filter((line) => line.startsWith("CLOSED_WORLD_COMPOSES_ADMISSION_CELL="))
  .map(
    (line) => JSON.parse(line.slice("CLOSED_WORLD_COMPOSES_ADMISSION_CELL=".length)) as MatrixCell,
  );
const matrixCell = (fixture: string): MatrixCell | undefined =>
  matrixCells.find((cell) => cell.fixture === fixture);
const intactShape = matrixCell("shape");
const intactClean = matrixCell("clean");
const sourceOpenShape = matrixCell("source-open-shape");
const scannedEmpty = matrixCells.find((cell) => cell.state === "scannedEmpty");
const sourceSetOpen = matrixCells.find((cell) => cell.state === "sourceSetOpen");
const injectionRun = [...argumentsSet].some((argument) => argument.startsWith("--inject-"));
const carrierHopProbe = injectionRun ? null : runCarrierHopProbe();
const matrixEvidenceGreen =
  matrixCells.length === 5 &&
  matrixCells.every((cell) => cell.outputCss.trim().length > 0) &&
  semanticMatrixEvidenceGreen(matrixCells) &&
  intactShape?.reasonKind === "livenessNotClosed" &&
  intactShape.refusedCount === 1 &&
  intactClean?.reasonKind === null &&
  intactClean.refusedCount === 0 &&
  sourceOpenShape?.reasonKind === "livenessNotClosed" &&
  sourceOpenShape.refusedCount === 1 &&
  sourceOpenShape.evidenceScope === "sourceSetOpen" &&
  scannedEmpty?.reasonKind === null &&
  scannedEmpty.evidenceScope === null &&
  sourceSetOpen?.reasonKind === null &&
  sourceSetOpen.evidenceScope === "sourceSetOpen";

// These cells describe measured executor values only.
// The subprocess owns the measured command and return code.
// Each cell owns only values observed from its bundle and execution summary.
// Carrier-origin independence is measured by the detached-worktree probe below.
// The matrix must not manufacture a second producer by mutating a sealed fixture.
// FALSIFIER: id=closed-world-admission-composes-matrix class=liveness via=--inject-composes-admission-matrix-loss producer=can-fail owner=closed-world-admission-tier entry=tri-state-and-independent-producer-matrix
assert.equal(
  matrixRun.status === 0 &&
    matrixEvidenceGreen &&
    (injectionRun || carrierHopProbe?.detected === true) &&
    matrixRun.transcript.includes(
      "test tests::runtime_boundary::closed_world_composes_admission_covers_tri_state_and_independent_producer_matrix ... ok",
    ),
  true,
  `closed-world composes admission matrix did not execute:\n${matrixRun.transcript}`,
);
const depthTestName = argumentsSet.has("--inject-composes-depth-two-loss")
  ? "tests::runtime_boundary::injected_missing_composes_depth_two"
  : "tests::runtime_boundary::closed_world_composes_admission_checks_depth_two_target";
const depthRun = transformPassesTest(depthTestName);

// FALSIFIER: id=closed-world-admission-composes-depth-two class=liveness via=--inject-composes-depth-two-loss producer=can-fail owner=closed-world-admission-tier entry=depth-two-target-retention
assert.equal(
  depthRun.status === 0 &&
    depthRun.transcript.includes(
      "test tests::runtime_boundary::closed_world_composes_admission_checks_depth_two_target ... ok",
    ),
  true,
  `closed-world composes depth-two arm did not execute:\n${depthRun.transcript}`,
);
// Product-path evidence is intentionally separate from the executor-plane cells.
// The bundled shape arm exercises composed-selector closure through the CLI.
// The bundled clean arm establishes that complete liveness evidence still admits.
// The plain arm has no transitive source-set closure declaration.
// A single-module plain bundle therefore cannot observe an inbound composed edge.
// Its source-set-open disclosure does not prevent high-certainty selector removal.
// That removal is recorded below instead of being presented as a closed empty scan.
// The no-engine-input arm is the refusal-shape product fixture.
// It measures the process return code and stdout instead of supplying literals.
// Its four pass refusals preserve every source rule and perform no semantic removal.
// The product-path injection changes the actual two-module source fixture.
// It removes the composes declaration and must lose the retained base selector.
// No injected branch substitutes a constant assertion result.
// Carrier-hop loss remains owned by the detached-worktree probe.
function sourceReference(
  fixtureRoot: string,
  id: string,
  stylePath: string,
  className: string,
): Record<string, unknown> {
  const range = {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 1 },
  };
  return {
    filePath: join(fixtureRoot, `${id}.tsx`),
    document: {
      classExpressions: [
        {
          id,
          kind: "styleAccess",
          scssModulePath: stylePath,
          range,
          className,
          rootBindingDeclId: null,
          accessPath: [className],
        },
      ],
    },
    bindingGraph: null,
  };
}
function selector(className: string): Record<string, unknown> {
  return {
    name: className,
    viewKind: "canonical",
    canonicalName: className,
    range: {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 1 },
    },
    nestedSafety: "safe",
    composes: null,
    bemSuffix: null,
  };
}
function writeEngineInput(
  fixtureRoot: string,
  fileName: string,
  entryPath: string,
  basePath: string,
  entrySource: string,
  baseSource: string,
  baseReachable: readonly string[],
): string {
  const enginePath = join(fixtureRoot, fileName);
  const sourceRows = [
    sourceReference(fixtureRoot, "entry-card", entryPath, "card"),
    ...baseReachable.map((name) => sourceReference(fixtureRoot, `base-${name}`, basePath, name)),
  ];
  const typeFacts = [
    {
      filePath: join(fixtureRoot, "entry-card.tsx"),
      expressionId: "entry-card",
      facts: { kind: "exact", values: ["card"] },
      controlFlowGraph: null,
    },
    ...baseReachable.map((name) => ({
      filePath: join(fixtureRoot, `base-${name}.tsx`),
      expressionId: `base-${name}`,
      facts: { kind: "exact", values: [name] },
      controlFlowGraph: null,
    })),
  ];
  writeFileSync(
    enginePath,
    JSON.stringify(
      {
        version: "2",
        workspace: {
          root: fixtureRoot,
          classnameTransform: "asIs",
          settingsKey: `closed-world-admission-${fileName}`,
        },
        sources: sourceRows,
        styles: [
          {
            filePath: entryPath,
            source: entrySource,
            document: { selectors: [selector("card")] },
          },
          {
            filePath: basePath,
            source: baseSource,
            document: {
              selectors: [selector("base"), selector("other"), selector("dead")],
            },
          },
        ],
        typeFacts,
      },
      null,
      2,
    ),
  );
  return enginePath;
}
function runProductArm(
  fixtureRoot: string,
  label: string,
  targetPath: string,
  sourcePath: string,
  enginePath: string | null,
  bundle: boolean,
) {
  const outputPath = join(fixtureRoot, `${label}.css`);
  const args = ["build", targetPath];
  if (bundle) args.push("--bundle", "--linked-emission");
  args.push("--tree-shake", "--strict-verification");
  if (enginePath) args.push("--engine-input-json", enginePath);
  args.push("--source", sourcePath, "--output", outputPath, "--json");
  const run = spawnSync(binary, args, { cwd: fixtureRoot, encoding: "utf8" });
  const response =
    run.status === 0 && run.stdout ? (JSON.parse(run.stdout) as Record<string, any>) : null;
  const admission = response?.payload?.execution?.closedWorldAdmission ?? null;
  const firstReason = admission?.refusalReasons?.[0]?.reasons?.[0] ?? null;
  const outputCss = run.status === 0 ? readFileSync(outputPath, "utf8") : "";
  return {
    label,
    command: [binary, ...args].join(" "),
    rc: run.status,
    stdoutLength: Buffer.byteLength(run.stdout ?? "", "utf8"),
    refusedCount: admission?.refusedCount ?? null,
    reasonKind: firstReason?.kind ?? null,
    refusalReasons: admission?.refusalReasons ?? [],
    evidenceScope: admission?.evidenceScope ?? null,
    outputCssLength: outputCss.length,
    outputCss,
    bundleEdges: response?.payload?.bundle?.bundleEdges ?? [],
    semanticRemovals: response?.payload?.execution?.semanticRemovals ?? [],
  };
}
const build = spawnSync("cargo", ["build", "-p", "omena-cli"], { cwd: rustRoot, encoding: "utf8" });

const fixtureRoot = mkdtempSync(join(tmpdir(), "omena-closed-world-admission-"));
let productArms: readonly ReturnType<typeof runProductArm>[] = [];
try {
  const entryPath = join(fixtureRoot, "entry.module.css");
  const basePath = join(fixtureRoot, "base.module.css");
  const entrySource = argumentsSet.has("--inject-composes-product-path-loss")
    ? ".card { color: red; }"
    : '.card { composes: base from "./base.module.css"; color: red; }';
  const baseSource = ".base { padding: 8px; } .other { color: green; } .dead { color: gray; }";
  writeFileSync(entryPath, entrySource);
  writeFileSync(basePath, baseSource);
  const shapeEngine = writeEngineInput(
    fixtureRoot,
    "shape-engine.json",
    entryPath,
    basePath,
    entrySource,
    baseSource,
    ["other"],
  );
  const cleanEngine = writeEngineInput(
    fixtureRoot,
    "clean-engine.json",
    entryPath,
    basePath,
    entrySource,
    baseSource,
    ["base", "other"],
  );
  productArms = [
    runProductArm(fixtureRoot, "bundle-shape", basePath, entryPath, shapeEngine, true),
    runProductArm(fixtureRoot, "bundle-clean", basePath, entryPath, cleanEngine, true),
    runProductArm(fixtureRoot, "plain-source-open", basePath, entryPath, shapeEngine, false),
    runProductArm(fixtureRoot, "bundle-no-engine-input", basePath, entryPath, null, true),
  ];
} finally {
  rmSync(fixtureRoot, { recursive: true, force: true });
}
const [bundleShape, bundleClean, plainSourceOpen, bundleNoEngineInput] = productArms;
const plainBaseRemoval = plainSourceOpen?.semanticRemovals.find(
  (removal: any) => removal?.name === "base",
) as Record<string, unknown> | undefined;
const productPathGreen =
  build.status === 0 &&
  productArms.length === 4 &&
  productArms.every((arm) => arm.rc === 0 && arm.stdoutLength > 0 && arm.outputCssLength > 0) &&
  bundleShape?.bundleEdges.length === 1 &&
  bundleShape?.refusedCount === 0 &&
  bundleShape?.outputCss.includes("padding: 8px") === true &&
  bundleShape?.outputCss.includes("color: green") === true &&
  bundleShape?.outputCss.includes("color: gray") === false &&
  bundleClean?.refusedCount === 0 &&
  plainSourceOpen?.refusedCount === 0 &&
  plainSourceOpen?.evidenceScope === "sourceSetOpen" &&
  plainSourceOpen.outputCss.includes("padding: 8px") === false &&
  plainBaseRemoval?.certainty === "high" &&
  bundleNoEngineInput?.refusedCount === 4 &&
  bundleNoEngineInput.refusalReasons.length === 4 &&
  bundleNoEngineInput.semanticRemovals.length === 0 &&
  bundleNoEngineInput.outputCss.includes("padding: 8px") &&
  bundleNoEngineInput.outputCss.includes("color: green") &&
  bundleNoEngineInput.outputCss.includes("color: gray");
// FALSIFIER: id=closed-world-admission-composes-product-path class=liveness via=--inject-composes-product-path-loss producer=can-fail owner=closed-world-admission-tier entry=bundle-and-plain-composes-closure
assert.equal(
  productPathGreen,
  true,
  `closed-world composes product path did not retain live CSS while removing dead CSS:\n${JSON.stringify(
    { buildStatus: build.status, buildStderr: build.stderr, productArms },
    null,
    2,
  )}`,
);

interface CallGraphFunction {
  readonly key: string;
  readonly name: string;
  readonly body: string;
}

function extractFunctions(file: string): readonly CallGraphFunction[] {
  const source = readFileSync(join(repositoryRoot, file), "utf8");
  const functions: CallGraphFunction[] = [];
  for (const match of source.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/gu)) {
    const name = match[1];
    const signatureStart = match.index ?? 0;
    const bodyStart = source.indexOf("{", signatureStart);
    if (!name || bodyStart < 0) continue;
    let depth = 0;
    let bodyEnd = bodyStart;
    for (; bodyEnd < source.length; bodyEnd += 1) {
      if (source[bodyEnd] === "{") depth += 1;
      if (source[bodyEnd] === "}") depth -= 1;
      if (depth === 0) {
        bodyEnd += 1;
        break;
      }
    }
    functions.push({
      key: `${file}::${name}`,
      name,
      body: source.slice(bodyStart, bodyEnd),
    });
  }
  return functions;
}

const callGraphFiles = [
  "rust/crates/omena-transform-passes/src/runtime/executor.rs",
  "rust/crates/omena-parser/src/closed_world/contract.rs",
  "rust/crates/omena-parser/src/closed_world/authority.rs",
  "rust/crates/omena-bundler/src/lib.rs",
  "rust/crates/omena-query/src/style/transform.rs",
];
const functions = callGraphFiles.flatMap(extractFunctions);
const keysByName = new Map<string, string[]>();
for (const fn of functions) {
  keysByName.set(fn.name, [...(keysByName.get(fn.name) ?? []), fn.key]);
}
const graph = new Map<string, Set<string>>();
for (const fn of functions) {
  const outgoing = new Set<string>();
  for (const call of fn.body.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)) {
    for (const target of keysByName.get(call[1] ?? "") ?? []) outgoing.add(target);
  }
  graph.set(fn.key, outgoing);
}
const referenceRoot =
  "rust/crates/omena-transform-passes/src/runtime/executor.rs::closed_world_admission_o2_reasons";
const semanticProducer =
  "rust/crates/omena-query/src/style/transform.rs::transform_bundle_semantic_reachability_input_from_context_and_attribution";
if (argumentsSet.has("--inject-admission-reference-callgraph-indirection")) {
  const indirection = "injected::one_level_indirection";
  graph.get(referenceRoot)?.add(indirection);
  graph.set(indirection, new Set([semanticProducer]));
}
function reaches(start: string, target: string): boolean {
  const pending = [start];
  const seen = new Set<string>();
  while (pending.length > 0) {
    const current = pending.pop();
    if (!current || seen.has(current)) continue;
    if (current === target) return true;
    seen.add(current);
    pending.push(...(graph.get(current) ?? []));
  }
  return false;
}
const referenceBody = functions.find((fn) => fn.key === referenceRoot)?.body ?? "";
const equivalentDirectGrepPasses = !referenceBody.includes(
  "transform_bundle_semantic_reachability_input_from_context_and_attribution",
);

// FALSIFIER: id=closed-world-admission-reference-callgraph class=liveness via=--inject-admission-reference-callgraph-indirection producer=can-fail owner=closed-world-admission-tier entry=disjoint-reference-producer
assert.equal(
  equivalentDirectGrepPasses && !reaches(referenceRoot, semanticProducer),
  true,
  "closed-world admission reference side transitively reaches the semantic reachability producer",
);

interface CarrierHopProbe {
  readonly baselineCommand: string;
  readonly baselineRc: number | null;
  readonly perturbedCommand: string;
  readonly perturbedRc: number | null;
  readonly detected: boolean;
  readonly scope: string;
  readonly operationalCost: string;
  readonly sigkillLeakRisk: string;
}

function runCarrierHopProbe(): CarrierHopProbe {
  const testName = "tests::external_composes_names_reach_the_sealed_closed_world_bundle";
  const probeRoot = mkdtempSync(join(tmpdir(), "omena-composes-carrier-hop-"));
  const worktreeRoot = join(probeRoot, "worktree");
  const add = spawnSync("git", ["worktree", "add", "--detach", "--quiet", worktreeRoot, "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
  if (add.status !== 0) {
    rmSync(probeRoot, { recursive: true, force: true });
    return {
      baselineCommand: `temporary detached worktree: cargo test -p omena-bundler ${testName} -- --exact`,
      baselineRc: null,
      perturbedCommand: `git worktree add --detach ${worktreeRoot} HEAD`,
      perturbedRc: add.status,
      detected: false,
      scope: "current HEAD in one detached worktree",
      operationalCost: "creates a detached worktree and compiles the focused bundler test twice",
      sigkillLeakRisk:
        "the finally cleanup covers ordinary exit but SIGKILL can leave the worktree registered",
    };
  }

  let baselineStatus: number | null = null;
  let perturbedStatus: number | null = null;
  let baselineTranscript = "";
  let perturbedTranscript = "";
  let cleanupStatus: number | null = null;
  try {
    const sourcePath = join(worktreeRoot, "rust/crates/omena-bundler/src/lib.rs");
    const source = readFileSync(sourcePath, "utf8");
    const cargoOptions = {
      cwd: join(worktreeRoot, "rust"),
      encoding: "utf8" as const,
      env: {
        ...process.env,
        CARGO_TARGET_DIR: join(rustRoot, "target", "carrier-hop-probe"),
      },
    };
    writeFileSync(sourcePath, source);
    const baseline = spawnSync(
      "cargo",
      ["test", "-p", "omena-bundler", testName, "--", "--exact"],
      cargoOptions,
    );
    baselineStatus = baseline.status;
    baselineTranscript = `${baseline.stdout ?? ""}\n${baseline.stderr ?? ""}`;
    const carrierCopy =
      "                        local_names: edge.local_names.clone(),\n" +
      "                        remote_names: edge.remote_names.clone(),";
    const carrierLoss =
      "                        local_names: Vec::new(),\n" +
      "                        remote_names: Vec::new(),";
    if (!source.includes(carrierCopy)) {
      throw new Error("carrier-hop probe could not locate the linker-input name copy");
    }
    writeFileSync(sourcePath, source.replace(carrierCopy, carrierLoss));
    const perturbed = spawnSync(
      "cargo",
      ["test", "-p", "omena-bundler", testName, "--", "--exact"],
      cargoOptions,
    );
    perturbedStatus = perturbed.status;
    perturbedTranscript = `${perturbed.stdout ?? ""}\n${perturbed.stderr ?? ""}`;
  } finally {
    const cleanup = spawnSync("git", ["worktree", "remove", "--force", worktreeRoot], {
      cwd: repositoryRoot,
      encoding: "utf8",
    });
    cleanupStatus = cleanup.status;
    rmSync(probeRoot, { recursive: true, force: true });
  }

  return {
    baselineCommand: `temporary detached worktree: cargo test -p omena-bundler ${testName} -- --exact`,
    baselineRc: baselineStatus,
    perturbedCommand:
      "temporary detached worktree: drop local_names/remote_names in linker_input_from_module_facts, then rerun the same test",
    perturbedRc: perturbedStatus,
    detected:
      baselineStatus === 0 &&
      baselineTranscript.includes(
        "test tests::external_composes_names_reach_the_sealed_closed_world_bundle ... ok",
      ) &&
      perturbedStatus !== 0 &&
      perturbedTranscript.includes(
        "test tests::external_composes_names_reach_the_sealed_closed_world_bundle ... FAILED",
      ) &&
      cleanupStatus === 0,
    scope: "current HEAD in one detached worktree",
    operationalCost: "creates a detached worktree and compiles the focused bundler test twice",
    sigkillLeakRisk:
      "the finally cleanup covers ordinary exit but SIGKILL can leave the worktree registered",
  };
}

function semanticMatrixEvidenceGreen(cells: readonly MatrixCell[]): boolean {
  type SemanticCell = MatrixCell & {
    readonly semanticObservedPassCount: number;
    readonly semanticPreservedPassCount: number;
    readonly semanticBlockedPassCount: number;
  };
  const semanticCells = cells as readonly SemanticCell[];
  const expected = new Map([
    ["inboundNonempty:shape", [1, 0, 1, 1]],
    ["inboundNonempty:clean", [1, 1, 0, 1]],
    ["scannedEmpty:empty", [1, 1, 0, 0]],
    ["sourceSetOpen:shape", [1, 1, 0, 0]],
    ["inboundNonempty:source-open-shape", [1, 0, 1, 1]],
  ]);
  return semanticCells.every((cell) => {
    const values = expected.get(`${cell.state}:${cell.fixture}`);
    return (
      values?.[0] === cell.semanticObservedPassCount &&
      values[1] === cell.semanticPreservedPassCount &&
      values[2] === cell.semanticBlockedPassCount &&
      values[3] === cell.observedBundleEdgeCount
    );
  });
}

const semanticPreservationSource = readFileSync(
  join(repositoryRoot, "rust/crates/omena-transform-passes/src/runtime/semantic_preservation.rs"),
  "utf8",
);
const executorSource = readFileSync(
  join(repositoryRoot, "rust/crates/omena-transform-passes/src/runtime/executor.rs"),
  "utf8",
);
const mechanismACollectorNeedles = [
  "collect_tree_shake_css_keyframe_removals_from_ir(",
  "collect_tree_shake_css_modules_value_removals_from_ir(",
  "collect_tree_shake_css_custom_property_removals_from_ir(",
];
for (const needle of mechanismACollectorNeedles) {
  if (!semanticPreservationSource.includes(needle)) {
    throw new Error(`semantic-preservation reading evidence lost collector ${needle}`);
  }
}
const forbiddenArmClaim = ["mechanismAArm", "Covered\\s*[:=]\\s*true"].join("");
const armClaimMatches = scanArmClaims(forbiddenArmClaim);
if (argumentsSet.has("--inject-mechanism-a-arm-claim")) {
  armClaimMatches.push(`injected:1:${["mechanismAArm", "Covered: true"].join("")}`);
}
if (armClaimMatches.length !== 0) {
  throw new Error(
    `mechanism (a) is reading-only but an arm-covered claim exists:\n${armClaimMatches.join("\n")}`,
  );
}
const capturedDigestNeedle = ["captured_module_qualified_", "ownership_digest"].join("");
const digestComparisonNeedle = `${capturedDigestNeedle} != bundle.module_qualified_ownership_digest()`;
const digestAccessorNeedle = "bundle.module_qualified_ownership_digest()";
const digestAccessorReadCount = executorSource.split(digestAccessorNeedle).length - 1;
if (
  digestAccessorReadCount !== 0 &&
  (!executorSource.includes(capturedDigestNeedle) ||
    !executorSource.includes(digestComparisonNeedle))
) {
  throw new Error(
    "an ownership-digest read re-entered without a captured plan-time carrier comparison",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena.closed-world-admission-tier",
      absenceTest: "1 passed",
      missingAnalysisTest: "1 passed",
      reachabilityAnalysisCells,
      reachabilityCorpusDiff: {
        fixtureCount: reachabilityCorpusFixtureIds.length,
        cells: reachabilityCorpusCells,
        analyzedProductByteDeltaInputIds,
        conservativeByteIdenticalInputIds,
        conservativeUnanalyzedCount,
      },
      composesMatrixTest: "1 passed",
      composesMatrixCommand: matrixRun.command,
      composesMatrixSubprocessRc: matrixRun.status,
      composesMatrixCells: matrixCells,
      carrierHopProbe,
      composesDepthTwoTest: "1 passed",
      productArms,
      o2Oracle: {
        classification: "degraded-shared-carrier",
        limitation:
          "admission and closure consume the same composed-edge carrier, so carrier-origin loss is delegated to the separate linker-hop probe",
        residualDiscrimination:
          "the executor plane still distinguishes closed shape gaps, clean closure, scanned-empty evidence, and source-set-open disclosure",
        observationCount:
          "the executor matrix emits and asserts the observed count; production admission no longer carries the unreachable greater-than-deduplicated-length branch",
      },
      mechanismA: {
        mechanismAArmCovered: false,
        evidence: "reading-only shared-collector inspection",
        collectorCount: mechanismACollectorNeedles.length,
        closingMeasurement:
          "a product fixture where a keyframe, CSS Modules value, or custom property is live only through a cross-module edge and the ignored-source-range comparator is observed",
        overclaimScanRc: armClaimMatches.length === 0 ? 1 : 0,
      },
      digestReentryTripwire: {
        accessorReadCount: digestAccessorReadCount,
        capturedDigestCarrierPresent: executorSource.includes(capturedDigestNeedle),
        executeComparisonPresent: executorSource.includes(digestComparisonNeedle),
      },
      antiTautology: {
        classification: "disclosure-only-current-crate-dag",
        equivalentDirectGrepPasses,
        callGraphPathExists: reaches(referenceRoot, semanticProducer),
        limitation:
          "the current dependency direction prevents this reference-to-producer path; the five-file extractor and synthetic indirection check do not establish a production-reachable failure",
      },
    },
    null,
    2,
  )}\n`,
);

function scanArmClaims(pattern: string): string[] {
  const matches: string[] = [];
  const forbiddenPattern = new RegExp(pattern, "u");
  const scanDirectory = (directory: string): void => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (entry.name !== "target") scanDirectory(join(directory, entry.name));
      } else if (entry.isFile()) {
        const path = join(directory, entry.name);
        readFileSync(path, "utf8")
          .split("\n")
          .forEach((line, index) => {
            if (forbiddenPattern.test(line)) {
              matches.push(`${path.slice(repositoryRoot.length + 1)}:${index + 1}:${line}`);
            }
          });
      }
    }
  };
  scanDirectory(join(repositoryRoot, "rust"));
  scanDirectory(join(repositoryRoot, "scripts"));
  return matches;
}
