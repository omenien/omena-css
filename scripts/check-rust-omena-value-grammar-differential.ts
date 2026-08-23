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
  specUrl?: string;
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
  validationClass: "valid" | "invalid" | "notValidatable";
  validationReason: string;
};

type MatcherReport = {
  cases: MatcherCase[];
};

type KeywordClosurePair = {
  id: string;
  property: string;
  value: string;
};

type ClosedWorldTokenKind =
  | "ident"
  | "hash"
  | "dimension"
  | "number"
  | "percentage"
  | "functionName"
  | "string"
  | "url";

type BuiltinTokenProfileSpec = {
  name: string;
  cssTreeType?: string;
  registryDerived?: true;
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
    source:
      | {
          kind: "pinned-repository";
          repository: string;
          pin: string;
          sparsePaths: string[];
        }
      | {
          kind: "local-workspace";
          workspacePath: string;
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
const keywordClosureCertificatePath = join(
  repoRoot,
  "rust/crates/omena-abstract-value/data/closed-world-keyword-closure-certificate.json",
);
const builtinTokenProfilePath = join(
  repoRoot,
  "rust/crates/omena-abstract-value/data/closed-world-builtin-token-profiles.json",
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
  const keywordClosure = keywordClosurePairs();
  const allMatcherCases = [...cases, ...keywordClosure.pairs];
  assert.equal(
    new Set(allMatcherCases.map((entry) => entry.id)).size,
    allMatcherCases.length,
    "keyword-closure ids must not collide with the differential corpus",
  );
  const tempRoot = mkdtempSync(join(tmpdir(), "omena-value-grammar-differential-"));
  const combinedPath = join(tempRoot, "cases.json");
  writeFileSync(
    combinedPath,
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: manifest.product,
        cases: allMatcherCases.map((entry) => ({
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
        ...(entry.specUrl ? { specUrl: entry.specUrl } : {}),
        validationClass: matcherCase.validationClass,
        validationReason: matcherCase.validationReason,
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
        entry.validationClass === "invalid" &&
        entry.cssTreeValid === true &&
        (entry.adjudication !== "omenaMatcherDefect" || !entry.owner || !entry.specUrl),
    ).length;
    if (unadjudicatedDisagreementCount !== 0) {
      violations.push(`${unadjudicatedDisagreementCount} disagreements are unadjudicated`);
    }
    if (wrongDefiniteUnownedCount !== 0) {
      violations.push(`${wrongDefiniteUnownedCount} wrong-definite rows lack a matcher owner`);
    }
    const keywordClosureOutcomes = keywordClosure.pairs.map((entry) => {
      const matcherCase = matcherById.get(entry.id);
      assert.ok(matcherCase, `missing keyword-closure matcher output for ${entry.id}`);
      return {
        property: entry.property,
        value: entry.value,
        validationClass: matcherCase.validationClass,
        validationReason: matcherCase.validationReason,
      };
    });
    const keywordClosureDefiniteRejections = keywordClosureOutcomes.filter(
      (entry) => entry.validationClass === "invalid",
    );
    const keywordClosureMatcherGaps = keywordClosure.pairs
      .map((entry) => {
        const matcherCase = matcherById.get(entry.id);
        assert.ok(matcherCase, `missing keyword-closure matcher output for ${entry.id}`);
        return matcherCase.verdict === "matched"
          ? undefined
          : {
              property: entry.property,
              value: entry.value,
              verdict: matcherCase.verdict,
            };
      })
      .filter((entry): entry is NonNullable<typeof entry> => entry !== undefined);
    const incompleteProperties = new Set(keywordClosureMatcherGaps.map((entry) => entry.property));
    const certifiedProperties = keywordClosure.properties.filter(
      (property) => !incompleteProperties.has(property),
    );
    if (keywordClosureDefiniteRejections.length !== 0) {
      violations.push(
        `${keywordClosureDefiniteRejections.length} css-tree accepted single-keyword pairs were definitely rejected: ${JSON.stringify(keywordClosureDefiniteRejections)}`,
      );
    }
    const keywordClosureSummary = {
      oracle: `css-tree@${cssTree.version}`,
      propertyCount: keywordClosure.propertyCount,
      candidatePairCount: keywordClosure.candidatePairCount,
      acceptedPairCount: keywordClosure.pairs.length,
      matchedPairCount: keywordClosure.pairs.length - keywordClosureMatcherGaps.length,
      matcherGapCount: keywordClosureMatcherGaps.length,
      definiteRejectionCount: keywordClosureDefiniteRejections.length,
      acceptedPairDigest: createHash("sha256")
        .update(
          JSON.stringify(keywordClosure.pairs.map(({ property, value }) => ({ property, value }))),
        )
        .digest("hex"),
    };
    const keywordClosureCertificate = {
      schemaVersion: "0",
      product: "omena-abstract-value.closed-world-keyword-closure-certificate",
      oracle: { name: "css-tree", version: cssTree.version },
      source: "cssTree.lexer.properties.directKeywordAcceptance",
      propertyCount: keywordClosure.propertyCount,
      candidatePairCount: keywordClosure.candidatePairCount,
      acceptedPairCount: keywordClosure.pairs.length,
      matchedPairCount: keywordClosure.pairs.length - keywordClosureMatcherGaps.length,
      matcherGapCount: keywordClosureMatcherGaps.length,
      acceptedPairDigest: keywordClosureSummary.acceptedPairDigest,
      certifiedProperties,
      matcherGaps: keywordClosureMatcherGaps,
    };
    const builtinTokenProfiles = buildBuiltinTokenProfiles();

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
      keywordClosure: keywordClosureSummary,
      witness,
      outcomes,
    };
    const serialized = await formatGeneratedJson(ledgerPath, ledger);
    const serializedKeywordClosureCertificate = await formatGeneratedJson(
      keywordClosureCertificatePath,
      keywordClosureCertificate,
    );
    const serializedBuiltinTokenProfiles = await formatGeneratedJson(
      builtinTokenProfilePath,
      builtinTokenProfiles,
    );
    if (write) {
      writeFileSync(ledgerPath, serialized);
      writeFileSync(keywordClosureCertificatePath, serializedKeywordClosureCertificate);
      writeFileSync(builtinTokenProfilePath, serializedBuiltinTokenProfiles);
    } else if (!injectDivergence && !injectAdjudicationContradiction) {
      assert.equal(readFileSync(ledgerPath, "utf8"), serialized, "differential ledger drifted");
      assert.equal(
        readFileSync(keywordClosureCertificatePath, "utf8"),
        serializedKeywordClosureCertificate,
        "keyword-closure certificate drifted",
      );
      assert.equal(
        readFileSync(builtinTokenProfilePath, "utf8"),
        serializedBuiltinTokenProfiles,
        "closed-world builtin token profiles drifted",
      );
    }

    process.stdout.write(
      `${JSON.stringify(
        {
          ...ledger,
          builtinTokenProfiles: {
            profileCount: builtinTokenProfiles.profiles.length,
            witnessDigest: builtinTokenProfiles.witnessDigest,
          },
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

function buildBuiltinTokenProfiles() {
  const tokenSamples: Record<ClosedWorldTokenKind, string[]> = {
    ident: ["fixture", "--fixture"],
    hash: ["#abc"],
    dimension: ["12px", "12deg", "12s", "2dppx", "1fr"],
    number: ["12"],
    percentage: ["12%"],
    functionName: ["calc(1px + 2px)", "foo("],
    string: ['"fixture"'],
    url: ["url(fixture.png)"],
  };
  const profileSpecs: BuiltinTokenProfileSpec[] = [
    "declaration-value",
    "any-value",
    "whole-value",
    "custom-ident",
    "ident",
    "ident-token",
    "dashed-ident",
    "custom-property-name",
    "string",
    "string-token",
    "url",
    "url-token",
    "number",
    "number-token",
    "integer",
    "length",
    "angle",
    "time",
    "resolution",
    "flex",
    "dimension-token",
    "percentage",
    "percentage-token",
    "length-percentage",
    "alpha-value",
    "hex-color",
    "hash-token",
    "function-token",
    "zero",
    "comma-token",
  ].map((name) => ({ name, cssTreeType: name }));
  profileSpecs.push(
    { name: "named-color", registryDerived: true },
    { name: "image", registryDerived: true },
    { name: "transform-function", registryDerived: true },
  );

  const profiles = profileSpecs.map((spec) => {
    if (spec.registryDerived) {
      return {
        name: spec.name,
        authority: "registryDerived" as const,
        openTokenKinds: [],
        allowedValues: {},
        witnesses: [],
      };
    }
    assert.ok(spec.cssTreeType);
    const witnesses = Object.entries(tokenSamples).flatMap(([tokenKind, samples]) =>
      samples.map((sample) => ({
        tokenKind: tokenKind as ClosedWorldTokenKind,
        sample,
        accepted: cssTreeTypeAccepts(spec.cssTreeType!, sample),
      })),
    );
    const cssTreeTypeExists = witnesses.every((witness) => witness.accepted !== null);
    const openTokenKinds = cssTreeTypeExists
      ? (Object.keys(tokenSamples) as ClosedWorldTokenKind[]).filter((tokenKind) =>
          witnesses.some((witness) => witness.tokenKind === tokenKind && witness.accepted === true),
        )
      : (Object.keys(tokenSamples) as ClosedWorldTokenKind[]);
    const zeroAccepted = cssTreeTypeExists
      ? cssTreeTypeAccepts(spec.cssTreeType, "0") === true
      : true;
    const allowedValues =
      zeroAccepted && !openTokenKinds.includes("number") ? { number: ["0"] } : {};
    return {
      name: spec.name,
      authority: cssTreeTypeExists ? ("cssTreeWitness" as const) : ("defaultOpen" as const),
      cssTreeType: spec.cssTreeType,
      openTokenKinds,
      allowedValues,
      witnesses,
    };
  });
  const witnessDigest = createHash("sha256").update(JSON.stringify(profiles)).digest("hex");
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.closed-world-builtin-token-profiles",
    oracle: { name: "css-tree", version: cssTree.version },
    policy:
      "A builtin token kind is closed only when css-tree rejects every representative; unknown types default every token kind open.",
    tokenSamples,
    structuralSamples: [{ tokenKind: "comma", sample: "," }],
    profileCount: profiles.length,
    witnessDigest,
    profiles,
  };
}

function cssTreeTypeAccepts(type: string, value: string): boolean | null {
  try {
    const result = cssTree.lexer.matchType(type, value);
    if (result.error?.message.includes(`Unknown type \`${type}\``)) return null;
    return result.matched !== null;
  } catch (error) {
    if (error instanceof Error && error.message.includes(`Unknown type \`${type}\``)) return null;
    throw error;
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
    if (fixture.source.kind !== "pinned-repository") continue;
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
    if (!entry.specUrl?.startsWith("https://")) {
      violations.push(`${entry.id}: disagreement has no specification citation`);
    }
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

function keywordClosurePairs(): {
  propertyCount: number;
  candidatePairCount: number;
  properties: string[];
  pairs: KeywordClosurePair[];
} {
  const properties = cssTree.lexer.properties as unknown as Record<string, { syntax: unknown }>;
  const pairs: KeywordClosurePair[] = [];
  let candidatePairCount = 0;
  const propertyNames = Object.keys(properties).sort(codePointCompare);
  for (const property of propertyNames) {
    const keywords = new Set<string>();
    collectDirectGrammarKeywords(properties[property]?.syntax, keywords);
    candidatePairCount += keywords.size;
    for (const value of [...keywords].sort(codePointCompare)) {
      if (cssTreeValidity(property, value).valid !== true) continue;
      const digest = createHash("sha256")
        .update(`${property}\0${value}`)
        .digest("hex")
        .slice(0, 20);
      pairs.push({
        id: `keyword-closure-${digest}`,
        property,
        value,
      });
    }
  }
  return {
    propertyCount: Object.keys(properties).length,
    candidatePairCount,
    properties: propertyNames,
    pairs,
  };
}

function collectDirectGrammarKeywords(node: unknown, keywords: Set<string>): void {
  if (Array.isArray(node)) {
    for (const entry of node) collectDirectGrammarKeywords(entry, keywords);
    return;
  }
  if (node === null || typeof node !== "object") return;
  const record = node as Record<string, unknown>;
  if (record.type === "Keyword" && typeof record.name === "string") {
    keywords.add(record.name.toLowerCase());
    return;
  }
  if (record.type === "Type" || record.type === "Property") return;
  for (const value of Object.values(record)) collectDirectGrammarKeywords(value, keywords);
}

function codePointCompare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
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
