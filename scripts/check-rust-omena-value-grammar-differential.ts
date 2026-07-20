import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import * as cssTree from "css-tree";

import { formatGeneratedJson } from "./generated-json";

type SeedCase = {
  id: string;
  property: string;
  value: string;
  expectedValid?: boolean;
  adjudication?: AdjudicationKind;
  reason?: string;
  owner?: string;
  notComparableReason?: string;
  source?: DeclarationSource;
};

type SeedManifest = {
  schemaVersion: string;
  product: string;
  cases: SeedCase[];
  differentialCases?: SeedCase[];
};

type MatcherCase = {
  id: string;
  verdict: "matched" | "unmatched" | "notMatchedWithinBudget" | "grammarDefect";
};

type MatcherReport = {
  cases: MatcherCase[];
};

type OutcomeKind = "agreeValid" | "agreeInvalid" | "disagree" | "notComparable";
type AdjudicationKind = "omenaMatcherDefect" | "cssTreeDefect" | "grammarSourceDivergence";

type DeclarationSource = {
  repository: string;
  pin: string;
  path: string;
  line: number;
  dialect: "css" | "scss" | "less";
};

type RealDeclarationCorpus = {
  schemaVersion: string;
  product: string;
  generatedBy: string;
  sourceManifest: string;
  maxCaseCount: number;
  scannedFileCount: number;
  harvestedDeclarationCount: number;
  uniqueDeclarationCount: number;
  caseCount: number;
  sourcePins: {
    repository: string;
    pin: string;
    sparsePaths: string[];
  }[];
  cases: SeedCase[];
};

type CorpusFarmManifest = {
  schemaVersion: string;
  product: string;
  fixtures: {
    source: {
      repository: string;
      pin: string;
      sparsePaths: string[];
    };
  }[];
};

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const seedPath = join(
  repoRoot,
  "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-seeds.json",
);
const realDeclarationCorpusPath = join(
  repoRoot,
  "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-real-declarations.json",
);
const corpusFarmManifestPath = join(
  repoRoot,
  "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json",
);
const ledgerPath = join(
  repoRoot,
  "rust/crates/omena-spec-audit/data/value-grammar-differential.json",
);
const MINIMUM_REAL_DECLARATION_CASE_COUNT = 113;
const REQUIRED_VALID_CASE_IDS = [
  "padding-unitless-zero-valid",
  "margin-unitless-zero-auto-valid",
  "border-width-mixed-zero-valid",
  "box-shadow-none-valid",
  "background-transparent-valid",
  "webkit-background-clip-text-valid",
] as const;
const ADJUDICATION_KINDS = new Set<AdjudicationKind>([
  "omenaMatcherDefect",
  "cssTreeDefect",
  "grammarSourceDivergence",
]);
const write = process.argv.includes("--write");
const injectDivergence = process.env.OMENA_VALUE_GRAMMAR_TEST_INJECT_MATCHER_DIVERGENCE === "1";
const injectAdjudicationContradiction =
  process.env.OMENA_VALUE_GRAMMAR_TEST_INJECT_ADJUDICATION_CONTRADICTION === "1";

assert.ok(
  !(write && (injectDivergence || injectAdjudicationContradiction)),
  "fault injection cannot update the committed differential ledger",
);

void main();

async function main(): Promise<void> {
  const manifest = JSON.parse(readFileSync(seedPath, "utf8")) as SeedManifest;
  assert.equal(manifest.schemaVersion, "0");
  const realDeclarationCorpus = JSON.parse(
    readFileSync(realDeclarationCorpusPath, "utf8"),
  ) as RealDeclarationCorpus;
  const corpusFarmManifest = JSON.parse(
    readFileSync(corpusFarmManifestPath, "utf8"),
  ) as CorpusFarmManifest;
  validateRealDeclarationCorpus(realDeclarationCorpus, corpusFarmManifest);

  const seedCases = [...manifest.cases, ...(manifest.differentialCases ?? [])];
  const cases = [...seedCases, ...realDeclarationCorpus.cases];
  assert.equal(new Set(cases.map((entry) => entry.id)).size, cases.length, "duplicate case id");
  const tempRoot = mkdtempSync(join(tmpdir(), "omena-value-grammar-differential-"));
  const combinedPath = join(tempRoot, "cases.json");
  writeFileSync(
    combinedPath,
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: manifest.product,
        cases: cases.map((entry) => ({
          id: entry.id,
          property: entry.property,
          value: entry.value,
          expectedValid: entry.expectedValid ?? false,
        })),
      },
      null,
      2,
    )}\n`,
  );

  try {
    const matcher = runCargoExample("value_grammar_evidence", [combinedPath]);
    const matcherReport = JSON.parse(matcher.stdout) as MatcherReport;
    const matcherById = new Map(matcherReport.cases.map((entry) => [entry.id, entry]));
    const inputDigest = createHash("sha256").update(JSON.stringify(cases)).digest("hex");
    const witnessRun = runCargoExample("value_grammar_external_tool_evidence", [
      "css-tree",
      cssTree.version,
      inputDigest,
      "0",
    ]);
    const witness = JSON.parse(witnessRun.stdout) as {
      earnedVia: string;
      key: { inputIdentity: string };
      provenance: string[];
    };
    assert.equal(witness.earnedVia, "externalTool");
    assert.equal(witness.key.inputIdentity, inputDigest);
    assert.ok(witness.provenance.includes(`toolVersion:${cssTree.version}`));

    const violations: string[] = [];
    let injectedDivergenceId: string | undefined;
    let injectedContradictionId: string | undefined;
    const outcomes = cases.map((entry) => {
      const matcherCase = matcherById.get(entry.id);
      assert.ok(matcherCase, `missing matcher output for ${entry.id}`);
      let omenaValid = verdictValidity(matcherCase.verdict);
      if (
        injectDivergence &&
        injectedDivergenceId === undefined &&
        omenaValid !== null &&
        entry.expectedValid === true
      ) {
        omenaValid = !omenaValid;
        injectedDivergenceId = entry.id;
      }
      const external = cssTreeValidity(entry.property, entry.value);
      let outcome: OutcomeKind;
      let notComparableReason: string | undefined;
      if (omenaValid === null) {
        outcome = "notComparable";
        notComparableReason = `omena:${matcherCase.verdict}`;
      } else if (external.valid === null) {
        outcome = "notComparable";
        notComparableReason = external.reason;
      } else if (omenaValid === external.valid) {
        outcome = omenaValid ? "agreeValid" : "agreeInvalid";
      } else {
        outcome = "disagree";
      }

      let adjudication = injectedDivergenceId === entry.id ? undefined : entry.adjudication;
      if (
        injectAdjudicationContradiction &&
        injectedContradictionId === undefined &&
        outcome === "disagree" &&
        adjudication
      ) {
        adjudication =
          adjudication === "omenaMatcherDefect" ? "cssTreeDefect" : "omenaMatcherDefect";
        injectedContradictionId = entry.id;
      }
      validateOutcomeAdjudication({
        entry,
        outcome,
        omenaValid,
        cssTreeValid: external.valid,
        adjudication,
        violations,
      });

      return {
        id: entry.id,
        property: entry.property,
        value: entry.value,
        omenaVerdict: matcherCase.verdict,
        omenaValid,
        cssTreeValid: external.valid,
        outcome,
        ...(adjudication ? { adjudication } : {}),
        ...(entry.reason ? { reason: entry.reason } : {}),
        ...(entry.owner ? { owner: entry.owner } : {}),
        ...(entry.source ? { source: entry.source } : {}),
        ...(outcome === "notComparable"
          ? { notComparableReason: entry.notComparableReason ?? notComparableReason }
          : {}),
      };
    });

    const counts = Object.fromEntries(
      (["agreeValid", "agreeInvalid", "disagree", "notComparable"] as const).map((kind) => [
        kind,
        outcomes.filter((outcome) => outcome.outcome === kind).length,
      ]),
    ) as Record<OutcomeKind, number>;
    for (const [kind, count] of Object.entries(counts)) {
      if (count === 0) violations.push(`outcome ${kind} is vacuous`);
    }
    if (injectDivergence && injectedDivergenceId === undefined) {
      violations.push("divergence injection did not reach a comparable valid case");
    }
    if (injectAdjudicationContradiction && injectedContradictionId === undefined) {
      violations.push("adjudication contradiction injection did not reach a disagreement");
    }
    for (const id of REQUIRED_VALID_CASE_IDS) {
      const outcome = outcomes.find((entry) => entry.id === id);
      if (outcome?.outcome !== "agreeValid") {
        violations.push(`${id}: required repaired declaration is not agreeValid`);
      }
    }

    const unadjudicatedDisagreementCount = outcomes.filter(
      (entry) => entry.outcome === "disagree" && !entry.adjudication,
    ).length;
    const wrongDefiniteUnownedCount = outcomes.filter(
      (entry) =>
        entry.outcome === "disagree" &&
        entry.omenaValid === false &&
        entry.cssTreeValid === true &&
        (entry.adjudication !== "omenaMatcherDefect" || !entry.owner),
    ).length;
    if (unadjudicatedDisagreementCount !== 0) {
      violations.push(`${unadjudicatedDisagreementCount} disagreements are unadjudicated`);
    }
    if (wrongDefiniteUnownedCount !== 0) {
      violations.push(`${wrongDefiniteUnownedCount} wrong-definite rows lack a matcher owner`);
    }

    const ledger = {
      schemaVersion: "0",
      product: "rust.omena-value-grammar-differential",
      sourceProduct: manifest.product,
      oracle: { name: "css-tree", version: cssTree.version },
      inputDigest,
      caseCount: outcomes.length,
      sources: {
        seedManifest: {
          product: manifest.product,
          caseCount: seedCases.length,
        },
        realDeclarations: {
          product: realDeclarationCorpus.product,
          sourceManifest: realDeclarationCorpus.sourceManifest,
          generatedBy: realDeclarationCorpus.generatedBy,
          scannedFileCount: realDeclarationCorpus.scannedFileCount,
          harvestedDeclarationCount: realDeclarationCorpus.harvestedDeclarationCount,
          uniqueDeclarationCount: realDeclarationCorpus.uniqueDeclarationCount,
          caseCount: realDeclarationCorpus.caseCount,
          sourcePins: realDeclarationCorpus.sourcePins,
        },
      },
      counts,
      unadjudicatedDisagreementCount,
      wrongDefiniteUnownedCount,
      witness,
      outcomes,
    };
    const serialized = await formatGeneratedJson(ledgerPath, ledger);
    if (write) {
      writeFileSync(ledgerPath, serialized);
    } else if (!injectDivergence && !injectAdjudicationContradiction) {
      assert.equal(readFileSync(ledgerPath, "utf8"), serialized, "differential ledger drifted");
    }

    process.stdout.write(
      `${JSON.stringify(
        {
          ...ledger,
          violations,
          injectedDivergenceId,
          injectedContradictionId,
        },
        null,
        2,
      )}\n`,
    );
    assert.deepEqual(violations, []);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function validateRealDeclarationCorpus(
  corpus: RealDeclarationCorpus,
  manifest: CorpusFarmManifest,
): void {
  assert.equal(corpus.schemaVersion, "0");
  assert.equal(corpus.product, "omena-abstract-value.value-grammar-real-declarations");
  assert.equal(corpus.generatedBy, "scripts/generate-rust-omena-value-grammar-corpus.ts");
  assert.equal(corpus.sourceManifest, "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json");
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.oss-corpus-farm.manifest");
  assert.equal(corpus.caseCount, corpus.cases.length);
  assert.ok(corpus.caseCount >= MINIMUM_REAL_DECLARATION_CASE_COUNT);
  assert.ok(corpus.caseCount <= corpus.maxCaseCount);
  assert.ok(corpus.scannedFileCount > 0);
  assert.ok(corpus.harvestedDeclarationCount >= corpus.uniqueDeclarationCount);
  assert.ok(corpus.uniqueDeclarationCount >= corpus.caseCount);
  assert.deepEqual(corpus.sourcePins, sourcePinsFromFarmManifest(manifest));
  assert.equal(
    new Set(corpus.cases.map((entry) => `${entry.property}\0${entry.value}`)).size,
    corpus.cases.length,
    "real declaration corpus contains duplicate property/value tuples",
  );

  const pinByRepository = new Map(corpus.sourcePins.map((entry) => [entry.repository, entry]));
  for (const entry of corpus.cases) {
    assert.equal(
      entry.id,
      `oss-${createHash("sha256")
        .update(`${entry.property}\0${entry.value}`)
        .digest("hex")
        .slice(0, 20)}`,
    );
    assert.ok(entry.property.length > 0);
    assert.ok(entry.value.length > 0);
    assert.equal(
      entry.expectedValid,
      undefined,
      `${entry.id}: harvested declarations cannot carry an expected-validity filter`,
    );
    assert.ok(entry.source, `${entry.id}: missing source provenance`);
    const pin = pinByRepository.get(entry.source.repository);
    assert.ok(pin, `${entry.id}: source repository is not in the pinned farm manifest`);
    assert.equal(entry.source.pin, pin.pin);
    assert.ok(entry.source.line > 0);
    assert.ok(
      pin.sparsePaths.some(
        (sparsePath) =>
          entry.source?.path === sparsePath || entry.source?.path.startsWith(`${sparsePath}/`),
      ),
      `${entry.id}: source path is outside the pinned sparse paths`,
    );
  }
}

function sourcePinsFromFarmManifest(manifest: CorpusFarmManifest) {
  const groups = new Map<string, { repository: string; pin: string; sparsePaths: Set<string> }>();
  for (const fixture of manifest.fixtures) {
    const key = `${fixture.source.repository}\0${fixture.source.pin}`;
    const group = groups.get(key) ?? {
      repository: fixture.source.repository,
      pin: fixture.source.pin,
      sparsePaths: new Set<string>(),
    };
    for (const sparsePath of fixture.source.sparsePaths) group.sparsePaths.add(sparsePath);
    groups.set(key, group);
  }
  return [...groups.values()]
    .sort((left, right) =>
      `${left.repository}\0${left.pin}`.localeCompare(`${right.repository}\0${right.pin}`, "en"),
    )
    .map((entry) => ({
      repository: entry.repository,
      pin: entry.pin,
      sparsePaths: [...entry.sparsePaths].sort((left, right) => left.localeCompare(right, "en")),
    }));
}

function validateOutcomeAdjudication(options: {
  entry: SeedCase;
  outcome: OutcomeKind;
  omenaValid: boolean | null;
  cssTreeValid: boolean | null;
  adjudication: AdjudicationKind | undefined;
  violations: string[];
}): void {
  const { entry, outcome, omenaValid, cssTreeValid, adjudication, violations } = options;
  if (adjudication && !ADJUDICATION_KINDS.has(adjudication)) {
    violations.push(`${entry.id}: unknown adjudication ${adjudication}`);
  }
  if (outcome === "disagree") {
    if (!adjudication) violations.push(`${entry.id}: unexplained disagreement`);
    if (!entry.reason?.trim()) violations.push(`${entry.id}: disagreement has no reviewed reason`);
    if (!entry.owner?.trim()) violations.push(`${entry.id}: disagreement has no follow-up owner`);
    if (omenaValid === false && cssTreeValid === true && adjudication !== "omenaMatcherDefect") {
      violations.push(`${entry.id}: wrong-definite disagreement is not a matcher defect`);
    }
    if (omenaValid === true && cssTreeValid === false && adjudication === "omenaMatcherDefect") {
      violations.push(`${entry.id}: matcher-defect adjudication contradicts an accepted value`);
    }
  } else if (adjudication) {
    violations.push(`${entry.id}: stale disagreement adjudication`);
  }

  if (outcome === "notComparable") {
    if (!entry.notComparableReason?.trim()) {
      violations.push(`${entry.id}: missing committed not-comparable reason`);
    }
  } else if (entry.notComparableReason) {
    violations.push(`${entry.id}: expected a not-comparable outcome`);
  }
}

function runCargoExample(example: string, args: string[]) {
  const run = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "--quiet",
      "-p",
      "omena-abstract-value",
      "--example",
      example,
      "--",
      ...args,
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(run.status, 0, run.stderr || run.stdout);
  return run;
}

function verdictValidity(verdict: MatcherCase["verdict"]): boolean | null {
  if (verdict === "matched") return true;
  if (verdict === "unmatched") return false;
  return null;
}

function cssTreeValidity(
  property: string,
  value: string,
): {
  valid: boolean | null;
  reason?: string;
} {
  try {
    const result = cssTree.lexer.matchProperty(property, value);
    return { valid: result.matched !== null };
  } catch (error) {
    return {
      valid: null,
      reason: error instanceof Error ? error.message : String(error),
    };
  }
}
