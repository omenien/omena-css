import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
  const run = spawnSync("cargo", ["test", "-p", "omena-bundler", testName, "--", "--exact"], {
    cwd: rustRoot,
    encoding: "utf8",
  });
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
  : "tests::empty_semantic_reachability_records_module_input_absence_without_narrowing";
const absenceRun = cargoTest(absenceTestName);

// FALSIFIER: id=closed-world-admission-reachability-absence class=liveness via=--inject-recorded-module-reachability-absence-loss producer=can-fail owner=closed-world-admission-tier entry=silent-full-module-retention
assert.equal(
  absenceRun.status === 0 &&
    absenceRun.transcript.includes(
      "test tests::empty_semantic_reachability_records_module_input_absence_without_narrowing ... ok",
    ),
  true,
  `module reachability absence was not recorded by the closed-world bundle:\n${absenceRun.transcript}`,
);

const matrixTestName = argumentsSet.has("--inject-composes-admission-matrix-loss")
  ? "tests::runtime_boundary::injected_missing_composes_admission_matrix"
  : "tests::runtime_boundary::closed_world_composes_admission_covers_tri_state_and_independent_producer_matrix";
const matrixRun = transformPassesTest(matrixTestName);

interface MatrixCell {
  readonly state: string;
  readonly producer: string;
  readonly fixture: string;
  readonly rc: number;
  readonly refusedCount: number;
  readonly reasonKind: string | null;
  readonly evidenceScope: string | null;
  readonly outputCss: string;
  readonly observedBundleEdgeCount: number;
}

const matrixCells = matrixRun.transcript
  .split("\n")
  .filter((line) => line.startsWith("G111_S4_CELL="))
  .map((line) => JSON.parse(line.slice("G111_S4_CELL=".length)) as MatrixCell);
const matrixCell = (producer: string, fixture: string): MatrixCell | undefined =>
  matrixCells.find((cell) => cell.producer === producer && cell.fixture === fixture);
const intactShape = matrixCell("intact", "shape");
const intactClean = matrixCell("intact", "clean");
const neuteredShape = matrixCell("neutered", "shape");
const neuteredClean = matrixCell("neutered", "clean");
const sourceOpenShape = matrixCell("intact", "source-open-shape");
const scannedEmpty = matrixCells.find((cell) => cell.state === "scannedEmpty");
const sourceSetOpen = matrixCells.find((cell) => cell.state === "sourceSetOpen");
const matrixEvidenceGreen =
  matrixCells.length === 7 &&
  matrixCells.every((cell) => cell.rc === 0 && cell.outputCss.trim().length > 0) &&
  intactShape?.reasonKind === "livenessNotClosed" &&
  intactShape.refusedCount === 1 &&
  intactClean?.reasonKind === null &&
  intactClean.refusedCount === 0 &&
  neuteredShape?.reasonKind === "evidenceUnavailable" &&
  neuteredShape.observedBundleEdgeCount === 1 &&
  neuteredClean?.reasonKind === "evidenceUnavailable" &&
  neuteredClean.observedBundleEdgeCount === 1 &&
  sourceOpenShape?.reasonKind === "livenessNotClosed" &&
  sourceOpenShape.refusedCount === 1 &&
  sourceOpenShape.evidenceScope === "sourceSetOpen" &&
  scannedEmpty?.reasonKind === null &&
  scannedEmpty.evidenceScope === null &&
  sourceSetOpen?.reasonKind === null &&
  sourceSetOpen.evidenceScope === "sourceSetOpen";

// FALSIFIER: id=closed-world-admission-composes-matrix class=liveness via=--inject-composes-admission-matrix-loss producer=can-fail owner=closed-world-admission-tier entry=tri-state-and-independent-producer-matrix
assert.equal(
  matrixRun.status === 0 &&
    matrixEvidenceGreen &&
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

interface ProductArm {
  readonly label: string;
  readonly command: string;
  readonly rc: number | null;
  readonly refusedCount: number | null;
  readonly reasonKind: string | null;
  readonly evidenceScope: string | null;
  readonly outputCssLength: number;
  readonly outputCss: string;
  readonly bundleEdges: readonly unknown[];
  readonly semanticRemovals: readonly unknown[];
}

function runProductArm(
  fixtureRoot: string,
  label: string,
  targetPath: string,
  sourcePath: string,
  enginePath: string,
  bundle: boolean,
): ProductArm {
  const outputPath = join(fixtureRoot, `${label}.css`);
  const args = ["build", targetPath];
  if (bundle) args.push("--bundle", "--linked-emission");
  args.push(
    "--tree-shake",
    "--strict-verification",
    "--engine-input-json",
    enginePath,
    "--source",
    sourcePath,
    "--output",
    outputPath,
    "--json",
  );
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
    refusedCount: admission?.refusedCount ?? null,
    reasonKind: firstReason?.kind ?? null,
    evidenceScope: admission?.evidenceScope ?? null,
    outputCssLength: outputCss.length,
    outputCss,
    bundleEdges: response?.payload?.bundle?.bundleEdges ?? [],
    semanticRemovals: response?.payload?.execution?.semanticRemovals ?? [],
  };
}

const build = spawnSync("cargo", ["build", "-p", "omena-cli"], {
  cwd: rustRoot,
  encoding: "utf8",
});
const fixtureRoot = mkdtempSync(join(tmpdir(), "omena-closed-world-admission-"));
let productArms: readonly ProductArm[] = [];
try {
  const entryPath = join(fixtureRoot, "entry.module.css");
  const basePath = join(fixtureRoot, "base.module.css");
  const entrySource = '.card { composes: base from "./base.module.css"; color: red; }';
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
  ];
} finally {
  rmSync(fixtureRoot, { recursive: true, force: true });
}

const [bundleShape, bundleClean, plainSourceOpen] = productArms;
const productPathGreen =
  build.status === 0 &&
  productArms.length === 3 &&
  productArms.every((arm) => arm.rc === 0 && arm.outputCssLength > 0) &&
  bundleShape?.bundleEdges.length === 1 &&
  bundleShape?.refusedCount === 0 &&
  bundleShape?.outputCss.includes("padding: 8px") === true &&
  bundleShape?.outputCss.includes("color: green") === true &&
  bundleShape?.outputCss.includes("color: gray") === false &&
  bundleClean?.refusedCount === 0 &&
  plainSourceOpen?.refusedCount === 0 &&
  plainSourceOpen?.evidenceScope === "sourceSetOpen";

// FALSIFIER: id=closed-world-admission-composes-product-path class=liveness via=--inject-composes-product-path-loss producer=can-fail owner=closed-world-admission-tier entry=bundle-and-plain-composes-closure
assert.equal(
  argumentsSet.has("--inject-composes-product-path-loss") ? false : productPathGreen,
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

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena.closed-world-admission-tier",
      absenceTest: "1 passed",
      composesMatrixTest: "1 passed",
      composesMatrixCommand: matrixRun.command,
      composesMatrixCells: matrixCells,
      composesDepthTwoTest: "1 passed",
      productArms,
      antiTautology: {
        equivalentDirectGrepPasses,
        callGraphPathExists: reaches(referenceRoot, semanticProducer),
      },
    },
    null,
    2,
  )}\n`,
);
