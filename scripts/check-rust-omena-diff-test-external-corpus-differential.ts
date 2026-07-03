import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { parseCssWithZ5LightningCss } from "./check-rust-z5-external-comparator-readiness";

type SassLegacyDivergenceKind = "none" | "typeA" | "typeB" | "typeC";
type StaticLegacyDivergenceKind = "none" | "compilerError" | "nativeUnavailable" | "valueMismatch";
type StaticDialect = "scss" | "sass" | "less";
type StaticCompiler = "dart-sass" | "lessc";
type DifferentialVoiceName = "omena" | "dart-sass" | "lightningcss" | "lessc";
type PairOutcomeKind =
  | "match"
  | "external-compiled-reference-impossible"
  | "external-error-reference-clean"
  | "expected-sound-bail"
  | "external-compiler-error"
  | "reference-native-unavailable"
  | "value-mismatch"
  | "voice-parse-error";

interface SassDifferentialManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly fixtures: readonly SassDifferentialFixtureV0[];
}

interface SassDifferentialFixtureV0 {
  readonly id: string;
  readonly entrypoint: string;
  readonly sources: readonly string[];
  readonly expectedDivergence: SassLegacyDivergenceKind;
}

interface StaticDifferentialManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly lastVerified: string;
  readonly compilers: {
    readonly dartSassPackage: string;
    readonly dartSassVersion: string;
    readonly lesscPackage: string;
    readonly lesscVersion: string;
  };
  readonly fixtures: readonly StaticDifferentialFixtureV0[];
}

interface StaticDifferentialFixtureV0 {
  readonly id: string;
  readonly dialect: StaticDialect;
  readonly compiler: StaticCompiler;
  readonly source: string;
  readonly expectedDivergence: StaticLegacyDivergenceKind;
}

interface WptEnvelopeManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunks: readonly {
    readonly chunkId: string;
    readonly path: string;
  }[];
}

interface WptChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly fixtures: readonly {
    readonly id: string;
    readonly source: string;
  }[];
}

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
}

interface StyleDiagnosticsSummary {
  readonly diagnostics: readonly StyleDiagnostic[];
}

interface StyleDiagnostic {
  readonly code: string;
  readonly message: string;
}

interface OmenaBuildSummaryV0 {
  readonly execution?: {
    readonly cssModuleEvaluation?: {
      readonly productOutputSource?: string;
      readonly nativeEditOutput?: string;
    };
  };
}

interface DeclarationValuePairV0 {
  readonly property: string;
  readonly value: string;
}

interface DifferentialRecordV0 {
  readonly fixtureId: string;
  readonly corpus: string;
  readonly voicePair: string;
  readonly outcomeKind: PairOutcomeKind;
  readonly expectedOutcomeKind: PairOutcomeKind;
  readonly blocking: boolean;
  readonly allowlisted: boolean;
  readonly details: Record<string, unknown>;
}

interface VoiceVerdictV0 {
  readonly voice: DifferentialVoiceName;
  readonly corpus: string;
  readonly fixtureCount: number;
  readonly passedCount: number;
  readonly failedCount: number;
}

interface RunStatusResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

interface LedgerEntryV0 {
  readonly fixtureId: string;
  readonly voicePair: string;
  readonly outcomeKind: PairOutcomeKind;
  readonly reason: string;
  readonly since: string;
  readonly reviewAfter: string;
}

interface DisagreementLedgerV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly allowlistCount: number;
  readonly entries: readonly LedgerEntryV0[];
}

const repoRoot = process.cwd();
const sassCorpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-differential");
const staticCorpusRoot = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/static-stylesheet-external-differential",
);
const wptCorpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const ledgerPath = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/external-corpus-differential/disagreement-ledger.toml",
);

const packageJson = readJson<PackageJsonV0>(path.join(repoRoot, "package.json"));
const sassManifest = readJson<SassDifferentialManifestV0>(
  path.join(sassCorpusRoot, "manifest.json"),
);
const staticManifest = readJson<StaticDifferentialManifestV0>(
  path.join(staticCorpusRoot, "manifest.json"),
);
const ledger = readLedger(ledgerPath);

assert.equal(sassManifest.schemaVersion, "0");
assert.equal(sassManifest.product, "omena-diff-test.sass-differential-corpus");
assert.equal(sassManifest.mode, "sass-compilability");
assert.ok(sassManifest.fixtures.length > 0, "sass differential corpus must not be empty");
assert.equal(staticManifest.schemaVersion, "0");
assert.equal(
  staticManifest.product,
  "omena-scss-eval.static-stylesheet-external-differential-corpus",
);
assert.equal(staticManifest.mode, "externalDifferential");
assert.ok(staticManifest.fixtures.length > 0, "external differential corpus must not be empty");
assert.equal(staticManifest.compilers.dartSassPackage, "sass");
assert.equal(staticManifest.compilers.dartSassVersion, "1.100.0");
assert.equal(staticManifest.compilers.lesscPackage, "less");
assert.equal(staticManifest.compilers.lesscVersion, "4.6.4");
assert.equal(packageJson.devDependencies?.sass, "1.100.0");
assert.equal(packageJson.devDependencies?.less, "4.6.4");
assert.equal(packageJson.devDependencies?.lightningcss, "1.32.0");

const dartSassVersion = run("pnpm", ["exec", "sass", "--version"]).stdout.trim();
const lesscVersion = run("pnpm", ["exec", "lessc", "--version"]).stdout.trim();
assert.match(
  dartSassVersion,
  /^1\.100\.0\b/u,
  `dart-sass oracle must resolve to 1.100.0, got ${dartSassVersion}`,
);
assert.match(
  lesscVersion,
  /^lessc 4\.6\.4\b/u,
  `lessc oracle must resolve to 4.6.4, got ${lesscVersion}`,
);

const sassRecords = sassManifest.fixtures.map(evaluateSassCompilabilityFixture);
const staticRecords = staticManifest.fixtures.map(evaluateStaticStylesheetFixture);
const lightningVerdict = evaluateLightningCssEnvelopeVoice();
const records = [...sassRecords, ...staticRecords];
const unallowlistedBlockingRecords = records.filter(
  (record) => record.blocking && !record.allowlisted,
);
assert.deepEqual(
  unallowlistedBlockingRecords,
  [],
  `blocking external-corpus disagreements require explicit ledger entries:\n${JSON.stringify(
    unallowlistedBlockingRecords,
    null,
    2,
  )}`,
);
assert.ok(
  records.some((record) => record.allowlisted),
  "external-corpus disagreement lane must exercise at least one committed ledger entry",
);

const voiceVerdicts: VoiceVerdictV0[] = [
  summarizeVoice("omena", "sass-differential", sassRecords),
  summarizeVoice("dart-sass", "sass-differential", sassRecords),
  summarizeVoice("omena", "static-stylesheet-external-differential", staticRecords),
  summarizeVoice(
    "dart-sass",
    "static-stylesheet-external-differential",
    staticRecords.filter((record) => record.voicePair === "omena:dart-sass"),
  ),
  summarizeVoice(
    "lessc",
    "static-stylesheet-external-differential",
    staticRecords.filter((record) => record.voicePair === "omena:lessc"),
  ),
  lightningVerdict,
];
const voiceNames = new Set(voiceVerdicts.map((verdict) => verdict.voice));
assert.ok(voiceNames.has("omena"));
assert.ok(voiceNames.has("dart-sass"));
assert.ok(voiceNames.has("lightningcss"));
assert.ok(voiceNames.size >= 3, "external-corpus differential lane must run at least three voices");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-diff-test.external-corpus-differential",
      dartSassVersion,
      lesscVersion,
      voiceCount: voiceNames.size,
      voices: [...voiceNames].toSorted(),
      differentialRecordCount: records.length,
      blockingDisagreementCount: records.filter((record) => record.blocking).length,
      allowlistCount: ledger.allowlistCount,
      voiceVerdicts,
      outcomeCounts: countOutcomes(records),
      records,
    },
    null,
    2,
  )}\n`,
);

function evaluateSassCompilabilityFixture(
  fixture: SassDifferentialFixtureV0,
): DifferentialRecordV0 {
  assertFixturePath(sassCorpusRoot, fixture.entrypoint, `${fixture.id} entrypoint`);
  for (const source of fixture.sources) {
    assertFixturePath(sassCorpusRoot, source, `${fixture.id} source`);
  }

  const entrypoint = path.join(sassCorpusRoot, fixture.entrypoint);
  const sourcePaths = fixture.sources.map((source) => path.join(sassCorpusRoot, source));
  const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-external-corpus-sass-"));
  try {
    const sassResult = runStatus("pnpm", [
      "exec",
      "sass",
      "--no-source-map",
      entrypoint,
      path.join(outputRoot, `${fixture.id}.css`),
    ]);
    const dartSassCompiled = sassResult.status === 0;
    const summary = runStyleDiagnostics(entrypoint, sourcePaths);
    const missingSassSymbolCount = summary.diagnostics.filter(
      (diagnostic) => diagnostic.code === "missingSassSymbol",
    ).length;
    const outcomeKind = classifySassCompilabilityOutcome(dartSassCompiled, missingSassSymbolCount);
    const expectedOutcomeKind = legacySassOutcome(fixture.expectedDivergence);
    assert.equal(
      outcomeKind,
      expectedOutcomeKind,
      `${fixture.id} expected ${expectedOutcomeKind}, got ${outcomeKind}\nsass stderr=${sassResult.stderr}`,
    );
    return record(
      "sass-differential",
      fixture.id,
      "omena:dart-sass",
      outcomeKind,
      expectedOutcomeKind,
      {
        dartSassCompiled,
        missingSassSymbolCount,
      },
    );
  } finally {
    rmSync(outputRoot, { force: true, recursive: true });
  }
}

function evaluateStaticStylesheetFixture(
  fixture: StaticDifferentialFixtureV0,
): DifferentialRecordV0 {
  const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-external-corpus-static-"));
  const inputPath = path.join(outputRoot, `input.${fixture.dialect}`);
  const outputPath = path.join(outputRoot, "external.css");
  try {
    writeFileSync(inputPath, fixture.source);
    const compilerResult = runExternalCompiler(fixture, inputPath, outputPath);
    const compilerCompiled = compilerResult.status === 0;
    const compilerCss = compilerCompiled ? readFileSync(outputPath, "utf8") : "";
    const nativeOutput = readOmenaNativeEditOutput(inputPath);
    const nativeEvaluationAvailable = nativeOutput !== undefined;
    const compilerValuePairs = compilerCompiled ? collectDeclarationValuePairs(compilerCss) : [];
    const nativeValuePairs =
      nativeOutput === undefined ? [] : collectDeclarationValuePairs(nativeOutput);
    const outcomeKind = classifyStaticOutcome(
      compilerCompiled,
      nativeEvaluationAvailable,
      compilerValuePairs,
      nativeValuePairs,
    );
    const expectedOutcomeKind = legacyStaticOutcome(fixture.expectedDivergence);
    assert.equal(
      outcomeKind,
      expectedOutcomeKind,
      `${fixture.id} expected ${expectedOutcomeKind}, got ${outcomeKind}`,
    );
    return record(
      "static-stylesheet-external-differential",
      fixture.id,
      `omena:${fixture.compiler}`,
      outcomeKind,
      expectedOutcomeKind,
      {
        dialect: fixture.dialect,
        compilerCompiled,
        nativeEvaluationAvailable,
        compilerValuePairs,
        nativeValuePairs,
      },
    );
  } finally {
    rmSync(outputRoot, { force: true, recursive: true });
  }
}

function evaluateLightningCssEnvelopeVoice(): VoiceVerdictV0 {
  const manifest = readJson<WptEnvelopeManifestV0>(path.join(wptCorpusRoot, "manifest.json"));
  assert.equal(manifest.schemaVersion, "0");
  assert.equal(manifest.product, "omena-diff-test.wpt-seed-corpus.manifest");
  let fixtureCount = 0;
  let passedCount = 0;
  let failedCount = 0;
  for (const chunkRef of manifest.chunks) {
    const chunk = readJson<WptChunkV0>(path.join(wptCorpusRoot, chunkRef.path));
    assert.equal(chunk.schemaVersion, "0");
    assert.equal(chunk.product, "omena-diff-test.wpt-seed-corpus.chunk");
    for (const fixture of chunk.fixtures) {
      fixtureCount += 1;
      try {
        parseCssWithZ5LightningCss(`${chunkRef.path}#${fixture.id}`, fixture.source);
        passedCount += 1;
      } catch {
        failedCount += 1;
      }
    }
  }
  assert.ok(fixtureCount > 0, "WPT envelope corpus must expose CSS fixtures");
  assert.ok(passedCount > 0, "lightningcss voice must parse at least one WPT envelope fixture");
  return {
    voice: "lightningcss",
    corpus: "wpt-corpus",
    fixtureCount,
    passedCount,
    failedCount,
  };
}

function classifySassCompilabilityOutcome(
  dartSassCompiled: boolean,
  missingSassSymbolCount: number,
): PairOutcomeKind {
  if (dartSassCompiled && missingSassSymbolCount === 0) {
    return "match";
  }
  if (dartSassCompiled) {
    return "external-compiled-reference-impossible";
  }
  if (missingSassSymbolCount === 0) {
    return "external-error-reference-clean";
  }
  return "expected-sound-bail";
}

function classifyStaticOutcome(
  compilerCompiled: boolean,
  nativeEvaluationAvailable: boolean,
  compilerValuePairs: readonly DeclarationValuePairV0[],
  nativeValuePairs: readonly DeclarationValuePairV0[],
): PairOutcomeKind {
  if (!compilerCompiled) {
    return "external-compiler-error";
  }
  if (!nativeEvaluationAvailable) {
    return "reference-native-unavailable";
  }
  if (JSON.stringify(compilerValuePairs) !== JSON.stringify(nativeValuePairs)) {
    return "value-mismatch";
  }
  return "match";
}

function legacySassOutcome(kind: SassLegacyDivergenceKind): PairOutcomeKind {
  switch (kind) {
    case "none":
      return "match";
    case "typeA":
      return "external-compiled-reference-impossible";
    case "typeB":
      return "external-error-reference-clean";
    case "typeC":
      return "expected-sound-bail";
  }
}

function legacyStaticOutcome(kind: StaticLegacyDivergenceKind): PairOutcomeKind {
  switch (kind) {
    case "none":
      return "match";
    case "compilerError":
      return "external-compiler-error";
    case "nativeUnavailable":
      return "reference-native-unavailable";
    case "valueMismatch":
      return "value-mismatch";
  }
}

function record(
  corpus: string,
  fixtureId: string,
  voicePair: string,
  outcomeKind: PairOutcomeKind,
  expectedOutcomeKind: PairOutcomeKind,
  details: Record<string, unknown>,
): DifferentialRecordV0 {
  const allowlisted = ledger.entries.some(
    (entry) =>
      entry.fixtureId === fixtureId &&
      entry.voicePair === voicePair &&
      entry.outcomeKind === outcomeKind,
  );
  return {
    fixtureId,
    corpus,
    voicePair,
    outcomeKind,
    expectedOutcomeKind,
    blocking: outcomeKind === "external-compiled-reference-impossible",
    allowlisted,
    details,
  };
}

function summarizeVoice(
  voice: DifferentialVoiceName,
  corpus: string,
  records: readonly DifferentialRecordV0[],
): VoiceVerdictV0 {
  return {
    voice,
    corpus,
    fixtureCount: records.length,
    passedCount: records.filter((record) => record.outcomeKind === record.expectedOutcomeKind)
      .length,
    failedCount: records.filter((record) => record.outcomeKind !== record.expectedOutcomeKind)
      .length,
  };
}

function countOutcomes(records: readonly DifferentialRecordV0[]): Record<PairOutcomeKind, number> {
  const counts: Record<PairOutcomeKind, number> = {
    match: 0,
    "external-compiled-reference-impossible": 0,
    "external-error-reference-clean": 0,
    "expected-sound-bail": 0,
    "external-compiler-error": 0,
    "reference-native-unavailable": 0,
    "value-mismatch": 0,
    "voice-parse-error": 0,
  };
  for (const record of records) {
    counts[record.outcomeKind] += 1;
  }
  return counts;
}

function runExternalCompiler(
  fixture: StaticDifferentialFixtureV0,
  inputPath: string,
  outputPath: string,
): RunStatusResult {
  if (fixture.compiler === "lessc") {
    return runStatus("pnpm", ["exec", "lessc", inputPath, outputPath]);
  }
  return runStatus("pnpm", [
    "exec",
    "sass",
    "--no-source-map",
    "--style",
    "expanded",
    inputPath,
    outputPath,
  ]);
}

function runStyleDiagnostics(
  entrypoint: string,
  sourcePaths: readonly string[],
): StyleDiagnosticsSummary {
  const args = [
    "style-diagnostics",
    entrypoint,
    ...sourcePaths.flatMap((sourcePath) => ["--source", sourcePath]),
    "--json",
  ];
  return JSON.parse(
    run(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-cli",
        "--bin",
        "omena-cli",
        "--",
        ...args,
      ],
      1024 * 1024 * 64,
    ).stdout,
  ) as StyleDiagnosticsSummary;
}

function readOmenaNativeEditOutput(inputPath: string): string | undefined {
  const summary = JSON.parse(
    run(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-cli",
        "--bin",
        "omena-cli",
        "--",
        "build",
        inputPath,
        "--json",
      ],
      1024 * 1024 * 32,
    ).stdout,
  ) as OmenaBuildSummaryV0;
  const evaluation = summary.execution?.cssModuleEvaluation;
  assert.equal(evaluation?.productOutputSource, "nativeEditOutput");
  return evaluation.nativeEditOutput;
}

function collectDeclarationValuePairs(css: string): DeclarationValuePairV0[] {
  const pairs: DeclarationValuePairV0[] = [];
  const declarationPattern = /(?<property>[-_a-zA-Z][-_a-zA-Z0-9]*)\s*:\s*(?<value>[^;{}]+);/gu;
  for (const match of css.matchAll(declarationPattern)) {
    const property = match.groups?.property;
    const value = match.groups?.value;
    assert.ok(property, `missing declaration property in ${match[0]}`);
    assert.ok(value, `missing declaration value in ${match[0]}`);
    pairs.push({
      property,
      value: value.trim().replace(/\s+/gu, " "),
    });
  }
  return pairs.sort((left, right) => {
    const propertyOrder = left.property.localeCompare(right.property);
    if (propertyOrder !== 0) {
      return propertyOrder;
    }
    return left.value.localeCompare(right.value);
  });
}

function readLedger(filePath: string): DisagreementLedgerV0 {
  const source = readFileSync(filePath, "utf8");
  const schemaVersion = requiredTomlString(source, "schema_version");
  const product = requiredTomlString(source, "product");
  const allowlistCount = Number(requiredTomlNumber(source, "allowlist_count"));
  const entries = source
    .split(/\n\[\[allowlist\]\]\n/gu)
    .slice(1)
    .map((entrySource) => {
      const entry = {
        fixtureId: requiredTomlString(entrySource, "fixture_id"),
        voicePair: requiredTomlString(entrySource, "voice_pair"),
        outcomeKind: requiredTomlString(entrySource, "outcome_kind") as PairOutcomeKind,
        reason: requiredTomlString(entrySource, "reason"),
        since: requiredTomlString(entrySource, "since"),
        reviewAfter: requiredTomlString(entrySource, "review_after"),
      };
      assert.ok(entry.reason.length >= 12, `${entry.fixtureId} ledger reason is too short`);
      assert.match(entry.since, /^\d{4}-\d{2}-\d{2}$/u);
      assert.match(entry.reviewAfter, /^\d{4}-\d{2}-\d{2}$/u);
      return entry;
    });
  assert.equal(schemaVersion, "0");
  assert.equal(product, "omena-diff-test.external-corpus-disagreement-ledger");
  assert.equal(allowlistCount, entries.length, "ledger allowlist_count must match entries");
  return { schemaVersion, product, allowlistCount, entries };
}

function requiredTomlString(source: string, key: string): string {
  const match = new RegExp(`^${key}\\s*=\\s*"(?<value>[^"]*)"`, "mu").exec(source);
  assert.ok(match?.groups?.value !== undefined, `missing TOML string key ${key}`);
  return match.groups.value;
}

function requiredTomlNumber(source: string, key: string): number {
  const match = new RegExp(`^${key}\\s*=\\s*(?<value>\\d+)`, "mu").exec(source);
  assert.ok(match?.groups?.value !== undefined, `missing TOML number key ${key}`);
  return Number(match.groups.value);
}

function assertFixturePath(root: string, relativePath: string, label: string): void {
  assert.ok(!path.isAbsolute(relativePath), `${label} must be relative`);
  assert.ok(!relativePath.includes(".."), `${label} must stay inside the corpus root`);
  assert.ok(existsSync(path.join(root, relativePath)), `${label} does not exist: ${relativePath}`);
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function run(
  command: string,
  args: readonly string[],
  maxBuffer = 1024 * 1024,
): { readonly stdout: string } {
  const result = runStatus(command, args, maxBuffer);
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}

function runStatus(
  command: string,
  args: readonly string[],
  maxBuffer = 1024 * 1024,
): RunStatusResult {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer,
  });
  if (result.error) {
    throw result.error;
  }
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}
