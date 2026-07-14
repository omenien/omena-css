// omena-verification-scope: engine-self
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface DischargeLedger {
  readonly schemaVersion: "1";
  readonly product: "omena-cascade-proof.discharge-ledger";
  readonly pins: {
    readonly theorySignatureHash: string;
    readonly specDigest: string;
    readonly encoderContentHash: string;
    readonly solverVersion: string;
  };
  readonly coverage: readonly DischargeCoverage[];
  readonly entries: readonly DischargeEntry[];
}

interface DischargeCoverage {
  readonly obligationFamily: string;
  readonly cellFamily: string;
  readonly cellCount: number;
  readonly exhaustive: boolean;
  readonly bound?: string;
}

interface DischargeEntry {
  readonly obligationFamily: string;
  readonly cellFamily: string;
  readonly obligationId: string;
  readonly l1Primitive: string;
  readonly cellKey: string;
  readonly canonicalTermCount: number;
  readonly canonicalTerms: readonly string[];
  readonly verdict: "accepted" | "rejected" | "unknown";
  readonly boundedness: { readonly kind: "exact" | "boundedK"; readonly k?: number };
  readonly referenceKind: string;
  readonly referenceSatResult: "sat" | "unsat" | "unknown";
  readonly solverKind: "z3";
  readonly solverSatResult: "sat" | "unsat" | "unknown";
  readonly referenceMatchesSolver: boolean;
}

type DischargeFamilyCoverageMode = "ledgerBacked" | "proseOnly";

interface DischargeFamilyCoveragePolicy {
  readonly family: string;
  readonly mode: DischargeFamilyCoverageMode;
  readonly cellFamilies?: readonly string[];
}

const dischargeFamilyCoveragePolicy: readonly DischargeFamilyCoveragePolicy[] = [
  { family: "cascadeSafetyFloor", mode: "proseOnly" },
  { family: "cascadeObligationDeclaration", mode: "proseOnly" },
  { family: "computedValuePreservation", mode: "proseOnly" },
  { family: "whitespaceBoundary", mode: "proseOnly" },
  { family: "commentSourceMapProvenance", mode: "proseOnly" },
  { family: "numericLiteralEquivalence", mode: "proseOnly" },
  { family: "dimensionComputedValue", mode: "proseOnly" },
  { family: "colorLiteralEquivalence", mode: "proseOnly" },
  { family: "urlTokenGrammar", mode: "proseOnly" },
  { family: "stringTextAndFontValue", mode: "proseOnly" },
  { family: "selectorSpecificityAndCascade", mode: "proseOnly" },
  {
    family: "longhandShorthandCascadeOutcome",
    mode: "ledgerBacked",
    cellFamilies: ["boxShorthandCombination", "longhandMerge"],
  },
  { family: "declarationCascadeOrder", mode: "proseOnly" },
  { family: "ruleMergeWinnerOrder", mode: "proseOnly" },
  { family: "selectorIdentityAndModuleSemantics", mode: "proseOnly" },
  { family: "semanticMarkerRetention", mode: "proseOnly" },
  { family: "targetPrefixAddition", mode: "proseOnly" },
  { family: "stalePrefixRemovalMapping", mode: "proseOnly" },
  { family: "targetFallbackBranch", mode: "proseOnly" },
  { family: "colorSpaceTargetEquivalence", mode: "proseOnly" },
  { family: "targetColorPrecision", mode: "proseOnly" },
  { family: "directionalityOption", mode: "proseOnly" },
  { family: "nestedSelectorSpecificity", mode: "proseOnly" },
  { family: "scopedMatching", mode: "ledgerBacked", cellFamilies: ["scopeFlattenCandidate"] },
  {
    family: "layerOrderComparison",
    mode: "ledgerBacked",
    cellFamilies: ["layerFlattenCandidate", "layerFlattenCascadeInversion"],
  },
  {
    family: "targetFeaturePredicate",
    mode: "ledgerBacked",
    cellFamilies: ["staticSupportsCondition"],
  },
  { family: "mediaPredicate", mode: "proseOnly" },
  { family: "containerPredicate", mode: "proseOnly" },
  { family: "nativeCssStaticValue", mode: "proseOnly" },
  { family: "calcExpressionEquivalence", mode: "proseOnly" },
  { family: "importWrapperProvenance", mode: "proseOnly" },
  { family: "scssNamespaceProvenance", mode: "proseOnly" },
  { family: "lessNamespaceProvenance", mode: "proseOnly" },
  { family: "selectorIdentityMap", mode: "proseOnly" },
  { family: "composedClassProvenance", mode: "proseOnly" },
  { family: "valueGraphResolution", mode: "proseOnly" },
  { family: "customPropertyFixedPoint", mode: "proseOnly" },
  { family: "sourceClassReachability", mode: "proseOnly" },
  { family: "animationNameReachability", mode: "proseOnly" },
  { family: "valueGraphReachability", mode: "proseOnly" },
  { family: "varReachability", mode: "proseOnly" },
  { family: "deadMediaWitness", mode: "proseOnly" },
  { family: "deadSupportsWitness", mode: "proseOnly" },
  { family: "designTokenPackageProvenance", mode: "proseOnly" },
  { family: "sourceMapTransformTrace", mode: "proseOnly" },
];

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const ledgerPath = path.join(
  repoRoot,
  "rust/crates/omena-cascade-proof/discharge-ledger/ledger.v1.json",
);

const generatedSource = buildLedgerSource();
const generatedLedger = parseLedger(generatedSource);
assertLedgerShape(generatedLedger);
assertLayerInversionPolarity(generatedLedger);
assertFamilyCoverageClosure(generatedLedger);
assertLonghandMergeCoverageAuthority();
assertRuntimePinConstants(generatedLedger);
assertDefaultBuildSolverPurity();

if (checkOnly) {
  assert.ok(
    existsSync(ledgerPath),
    "discharge ledger artifact is missing; run `pnpm update:rust-discharge-ledger`",
  );
  assert.equal(
    readFileSync(ledgerPath, "utf8"),
    generatedSource,
    "discharge ledger is stale; run `pnpm update:rust-discharge-ledger`",
  );
} else if (writeMode) {
  mkdirSync(path.dirname(ledgerPath), { recursive: true });
  writeFileSync(ledgerPath, generatedSource);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-cascade-proof.discharge-ledger.check",
      mode: checkOnly ? "check" : "write",
      path: path.relative(repoRoot, ledgerPath),
      entryCount: generatedLedger.entries.length,
      coverage: generatedLedger.coverage.map((entry) => ({
        obligationFamily: entry.obligationFamily,
        cellFamily: entry.cellFamily,
        cellCount: entry.cellCount,
        exhaustive: entry.exhaustive,
        bound: entry.bound,
      })),
      ledgerBackedFamilyCount: dischargeFamilyCoveragePolicy.filter(
        (entry) => entry.mode === "ledgerBacked",
      ).length,
      proseOnlyFamilyCount: dischargeFamilyCoveragePolicy.filter(
        (entry) => entry.mode === "proseOnly",
      ).length,
      pins: generatedLedger.pins,
      defaultBuildSolverFree: true,
    },
    null,
    2,
  )}\n`,
);

function buildLedgerSource(): string {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade-proof",
      "--features",
      "smt-z3",
      "--bin",
      "omena-cascade-discharge-ledger",
      "--quiet",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );
  assert.equal(
    result.status,
    0,
    `discharge ledger generator failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result.stdout;
}

function parseLedger(source: string): DischargeLedger {
  return JSON.parse(source) as DischargeLedger;
}

function assertLedgerShape(ledger: DischargeLedger): void {
  assert.equal(ledger.schemaVersion, "1");
  assert.equal(ledger.product, "omena-cascade-proof.discharge-ledger");
  assert.match(ledger.pins.theorySignatureHash, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.specDigest, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.encoderContentHash, /^[a-f0-9]{64}$/u);
  assert.match(ledger.pins.solverVersion, /^z3-crate-\d+\.\d+\.\d+-gh-release$/u);
  assert.ok(ledger.entries.length > 0, "ledger must not be empty");

  const requiredCellFamilies = [
    "boxShorthandCombination",
    "longhandMerge",
    "scopeFlattenCandidate",
    "layerFlattenCandidate",
    "staticSupportsCondition",
    "layerFlattenCascadeInversion",
  ];
  assert.deepEqual(
    ledger.coverage.map((coverage) => coverage.cellFamily).toSorted(),
    requiredCellFamilies.toSorted(),
  );
  for (const family of requiredCellFamilies) {
    const coverage = ledger.coverage.find((entry) => entry.cellFamily === family);
    assert.ok(coverage, `${family} coverage must exist`);
    assert.ok(coverage.cellCount > 0, `${family} coverage must not be empty`);
    if (family === "longhandMerge") {
      assert.equal(
        coverage.exhaustive,
        false,
        "longhandMerge coverage must state its bounded subset",
      );
      assert.match(coverage.bound ?? "", /BOX_LONGHAND_MERGE_SHORTHAND_FAMILIES_V0/u);
    } else {
      assert.equal(coverage.exhaustive, true, `${family} coverage must be marked exhaustive`);
    }
  }
  assert.ok(
    ledger.coverage.find((entry) => entry.cellFamily === "layerFlattenCascadeInversion")?.bound,
    "bounded layer inversion coverage must state its bound",
  );

  const keys = new Set<string>();
  for (const entry of ledger.entries) {
    assert.match(entry.cellKey, /^[a-f0-9]{64}$/u);
    assert.ok(!keys.has(entry.cellKey), `${entry.cellKey} must be unique`);
    keys.add(entry.cellKey);
    assert.equal(entry.canonicalTermCount, entry.canonicalTerms.length);
    assert.equal(entry.solverKind, "z3");
    assert.equal(
      entry.referenceMatchesSolver,
      true,
      `${entry.cellFamily}/${entry.cellKey} reference and solver must agree`,
    );
    if (entry.cellFamily === "layerFlattenCascadeInversion") {
      assert.equal(entry.boundedness.kind, "boundedK");
      assert.equal(entry.boundedness.k, 3);
      assert.equal(entry.referenceKind, "boundedLayerOrderingPredicate");
    } else {
      assert.equal(entry.boundedness.kind, "exact");
      assert.equal(entry.referenceKind, "productStubBackend");
    }
  }
}

function assertFamilyCoverageClosure(ledger: DischargeLedger): void {
  const registeredFamilies = readRegisteredObligationFamilies();
  const policyFamilies = dischargeFamilyCoveragePolicy.map((policy) => policy.family);
  assert.equal(
    new Set(policyFamilies).size,
    policyFamilies.length,
    "discharge family coverage policy must not contain duplicate families",
  );
  assert.deepEqual(
    policyFamilies.toSorted(),
    registeredFamilies.toSorted(),
    "discharge family coverage policy must cover every registered obligation family",
  );

  const policyByFamily = new Map(
    dischargeFamilyCoveragePolicy.map((policy) => [policy.family, policy] as const),
  );
  const coverageByFamily = new Map<string, DischargeCoverage[]>();
  const coverageByKey = new Map<string, DischargeCoverage>();
  for (const coverage of ledger.coverage) {
    const policy = policyByFamily.get(coverage.obligationFamily);
    assert.ok(policy, `${coverage.obligationFamily} must be a registered obligation family`);
    assert.equal(
      policy.mode,
      "ledgerBacked",
      `${coverage.obligationFamily} must be marked as ledger-backed before it can carry cells`,
    );
    assert.ok(
      policy.cellFamilies?.includes(coverage.cellFamily),
      `${coverage.obligationFamily}/${coverage.cellFamily} must be declared by the coverage policy`,
    );
    const key = coverageKey(coverage.obligationFamily, coverage.cellFamily);
    assert.ok(!coverageByKey.has(key), `${key} coverage must not be duplicated`);
    coverageByKey.set(key, coverage);
    coverageByFamily.set(coverage.obligationFamily, [
      ...(coverageByFamily.get(coverage.obligationFamily) ?? []),
      coverage,
    ]);
  }

  for (const policy of dischargeFamilyCoveragePolicy) {
    const coverage = coverageByFamily.get(policy.family) ?? [];
    if (policy.mode === "proseOnly") {
      assert.equal(coverage.length, 0, `${policy.family} must stay prose-only in this ledger`);
      continue;
    }
    assert.deepEqual(
      coverage.map((entry) => entry.cellFamily).toSorted(),
      [...(policy.cellFamilies ?? [])].toSorted(),
      `${policy.family} must carry exactly its declared ledger cell families`,
    );
    for (const cellFamily of policy.cellFamilies ?? []) {
      const coverageEntry = coverageByKey.get(coverageKey(policy.family, cellFamily));
      assert.ok(coverageEntry, `${policy.family}/${cellFamily} coverage must exist`);
      assert.ok(
        coverageEntry.cellCount > 0,
        `${policy.family}/${cellFamily} coverage must not be empty`,
      );
      assert.equal(
        ledger.entries.filter(
          (entry) => entry.obligationFamily === policy.family && entry.cellFamily === cellFamily,
        ).length,
        coverageEntry.cellCount,
        `${policy.family}/${cellFamily} coverage count must match ledger entries`,
      );
    }
  }

  for (const entry of ledger.entries) {
    assert.ok(
      coverageByKey.has(coverageKey(entry.obligationFamily, entry.cellFamily)),
      `${entry.obligationFamily}/${entry.cellFamily}/${entry.cellKey} must have coverage`,
    );
  }
}

function assertLayerInversionPolarity(ledger: DischargeLedger): void {
  const entries = ledger.entries.filter(
    (entry) => entry.cellFamily === "layerFlattenCascadeInversion",
  );
  assert.ok(entries.length > 0, "layer inversion ledger coverage must be non-vacuous");
  let acceptedCount = 0;
  let rejectedCount = 0;
  for (const entry of entries) {
    const coordinates = entry.canonicalTerms.map((term) => {
      const match = /^decl:[^:]+:rank=(-?\d+):source=(-?\d+)$/u.exec(term);
      assert.ok(match, `invalid layer inversion term: ${term}`);
      return { rank: Number(match[1]), source: Number(match[2]) };
    });
    const inversionExists = coordinates.some((left, leftIndex) =>
      coordinates.some(
        (right, rightIndex) =>
          leftIndex !== rightIndex && left.rank > right.rank && right.source > left.source,
      ),
    );
    assert.equal(
      entry.verdict,
      inversionExists ? "rejected" : "accepted",
      `${entry.cellKey} polarity must follow the independently recomputed ordering predicate`,
    );
    if (entry.verdict === "accepted") acceptedCount += 1;
    if (entry.verdict === "rejected") rejectedCount += 1;
  }
  assert.ok(acceptedCount > 0 && rejectedCount > 0);
}

function readRegisteredObligationFamilies(): string[] {
  const source = readFileSync(
    path.join(repoRoot, "rust/crates/omena-evidence-graph/src/lib.rs"),
    "utf8",
  );
  const implIndex = source.indexOf("impl ObligationFamilyIdV0");
  const asStrIndex = source.indexOf("pub const fn as_str", implIndex);
  const descriptorIndex = source.indexOf("pub const fn descriptor", asStrIndex);
  assert.ok(implIndex >= 0 && asStrIndex >= 0 && descriptorIndex > asStrIndex);
  const asStrSource = source.slice(asStrIndex, descriptorIndex);
  return [...asStrSource.matchAll(/Self::[A-Za-z0-9]+ => "([^"]+)"/g)].map((match) => match[1]);
}

function coverageKey(obligationFamily: string, cellFamily: string): string {
  return `${obligationFamily}\0${cellFamily}`;
}

function assertLonghandMergeCoverageAuthority(): void {
  const cascadeSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-cascade/src/shorthand_authority.rs"),
    "utf8",
  );
  const runtimeSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-transform-passes/src/domains/shorthand.rs"),
    "utf8",
  );
  const generatorSource = readFileSync(
    path.join(
      repoRoot,
      "rust/crates/omena-cascade-proof/src/bin/omena-cascade-discharge-ledger.rs",
    ),
    "utf8",
  );
  const runtimeFamilies = rustStringArray(cascadeSource, "LONGHAND_MERGE_SHORTHAND_FAMILIES_V0");
  const boundedFamilies = rustStringArray(
    cascadeSource,
    "BOX_LONGHAND_MERGE_SHORTHAND_FAMILIES_V0",
  );

  assert.equal(runtimeFamilies.length, 38);
  assert.equal(boundedFamilies.length, 7);
  assert.ok(boundedFamilies.every((family) => runtimeFamilies.includes(family)));
  assert.match(runtimeSource, /LONGHAND_MERGE_SHORTHAND_FAMILIES_V0\s*\.iter\(\)/u);
  assert.match(generatorSource, /for shorthand in BOX_LONGHAND_MERGE_SHORTHAND_FAMILIES_V0/u);
  assert.doesNotMatch(generatorSource, /const LONGHAND_MERGE_SHORTHANDS/u);
}

function rustStringArray(source: string, name: string): string[] {
  const declaration = new RegExp(
    `pub const ${name}: \\[&str; \\d+\\] = \\[([\\s\\S]*?)\\n\\];`,
    "u",
  ).exec(source);
  assert.ok(declaration, `${name} must be a public Rust string-array authority`);
  return [...declaration[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
}

function assertRuntimePinConstants(ledger: DischargeLedger): void {
  const runtimeSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-cascade-proof/src/discharge_ledger.rs"),
    "utf8",
  );
  for (const [name, value] of Object.entries(ledger.pins)) {
    assert.ok(
      runtimeSource.includes(`"${value}"`),
      `runtime discharge ledger pin constant must include ${name}=${value}`,
    );
  }
}

function assertDefaultBuildSolverPurity(): void {
  const result = spawnSync(
    "cargo",
    [
      "tree",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade-proof",
      "--no-default-features",
      "--edges",
      "normal",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 16,
    },
  );
  assert.equal(
    result.status,
    0,
    `default dependency tree probe failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  for (const forbidden of ["omena-smt", "z3 v", "z3-sys"]) {
    assert.ok(
      !result.stdout.includes(forbidden),
      `default omena-cascade-proof build must stay solver-free; found ${forbidden}`,
    );
  }
}
