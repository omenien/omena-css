import { strict as assert } from "node:assert";
import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

type TableKind = "global" | "verb" | "alias" | "legacy";

interface ConfigTableRow {
  readonly table: string;
  readonly kind: TableKind;
  readonly verb: string | null;
}

interface ConfigSchemaManifest {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.config-schema-census";
  readonly canonicalFileName: string;
  readonly compatibilityFileNames: readonly string[];
  readonly compatibilityDebtId: string;
  readonly tables: readonly ConfigTableRow[];
  readonly subTableLessVerbs: readonly string[];
  readonly translationValidation: readonly {
    readonly value: string;
    readonly reportKind: string | null;
    readonly engineArm: string | null;
  }[];
}

interface VerbManifest {
  readonly verbs: readonly { readonly verb: string }[];
}

interface DebtLedgerManifest {
  readonly entries: readonly {
    readonly id: string;
    readonly mechanism: string;
    readonly kind: string;
    readonly introduced_in: string;
    readonly expiry: { readonly after_reference_date: string };
  }[];
  readonly client_receipts: readonly {
    readonly client: string;
    readonly accepted_kinds: readonly string[];
    readonly authority: string;
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = readJson<ConfigSchemaManifest>("rust/crates/omena-cli/config-schema-census.json");
const verbs = readJson<VerbManifest>("rust/crates/omena-cli/verb-census.json");
const debtLedger = readJson<DebtLedgerManifest>("rust/omena-debt-ledger.json");
const schemaSource = read("rust/crates/omena-cli/src/config/schema.rs");
const resolutionSource = read("rust/crates/omena-cli/src/config/resolution.rs");
const loaderSource = read("rust/crates/omena-cli/src/config/loader.rs");
const conformanceSource = read(
  "rust/crates/omena-diff-test/src/transform_pass_cascade_conformance.rs",
);

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.config-schema-census");
assert.equal(manifest.canonicalFileName, "omena.toml");
assert.deepEqual(manifest.compatibilityFileNames, ["omena.config.toml", "omena.config.json"]);
assert.equal(manifest.compatibilityDebtId, "omena-cli-build-config-alias-window");

const verbNames = new Set(verbs.verbs.map(({ verb }) => verb));
const tableNames = manifest.tables.map(({ table }) => table);
assert.equal(new Set(tableNames).size, tableNames.length, "config tables must be unique");

const rootFields = extractStructFields(schemaSource, "OmenaConfig").filter(
  (field) => !["extends", "overrides"].includes(field),
);
const derivedTables = rootFields.flatMap((field) => {
  if (field !== "intelligence") return [toCamelCase(field)];
  const intelligenceFields = extractStructFields(schemaSource, "OmenaIntelligenceConfig");
  assert.deepEqual(intelligenceFields, ["tailwind"]);
  return intelligenceFields.map((nested) => `intelligence.${toCamelCase(nested)}`);
});
assert.deepEqual(
  [...derivedTables].sort(),
  [...tableNames].sort(),
  "typed config tables must match the committed table census",
);

for (const row of manifest.tables) {
  if (row.kind === "verb" || row.kind === "alias") {
    assert.ok(row.verb, `${row.table} must name its product verb`);
    assert.ok(verbNames.has(row.verb), `${row.table} maps to unknown verb ${row.verb}`);
  } else {
    assert.equal(row.verb, null, `${row.table} must not claim a product verb`);
  }
}
assert.deepEqual(
  manifest.tables.filter(({ kind }) => kind === "legacy"),
  [{ table: "build", kind: "legacy", verb: null }],
  "the legacy build table must remain the only time-boxed schema alias",
);
assertCompatibilityDebt();
assert.deepEqual([...manifest.subTableLessVerbs].sort(), ["bundle", "check", "explain", "migrate"]);
for (const verb of manifest.subTableLessVerbs) {
  assert.ok(verbNames.has(verb), `sub-table-less verb ${verb} must exist`);
  assert.ok(!manifest.tables.some((row) => row.verb === verb));
}

const translationModes = extractEnumVariants(schemaSource, "OmenaTranslationValidationMode").map(
  toCamelCase,
);
assert.deepEqual(
  manifest.translationValidation.map(({ value }) => value),
  translationModes,
  "translation-validation values must be enum-derived",
);
assert.deepEqual(manifest.translationValidation, [
  { value: "off", reportKind: null, engineArm: null },
  { value: "staged", reportKind: "notYetConsumed", engineArm: "modelConformant" },
]);
const engineConformanceArms = extractEnumVariants(
  conformanceSource,
  "TransformPassCascadeConformanceVerdictV0",
).map(toCamelCase);
assert.ok(
  engineConformanceArms.includes("modelConformant"),
  "staged translation validation must reference the engine-owned model-conformance arm",
);
assert.ok(
  resolutionSource.includes('"verify.translationValidation"'),
  "the staged value must reach the real config report renderer",
);
assert.ok(
  resolutionSource.includes("engine-owned observation-equality report arm"),
  "the unavailable engine-owned arm must remain an explicit named wait",
);

const schemaVocabulary = [...extractAllStructFields(schemaSource), ...translationModes].filter(
  (word) => !["verify", "translation_validation"].includes(word),
);
for (const word of schemaVocabulary) {
  assert.doesNotMatch(
    word,
    /(proven|certified|validated|verified)/iu,
    `config schema introduced unsupported guarantee vocabulary: ${word}`,
  );
}

const parserCallCount = [...resolutionSource.matchAll(/\bparse_config_document\s*\(/gu)].length;
assert.equal(
  parserCallCount,
  2,
  "config parser must have one definition and one recursive-resolution call site",
);
assert.equal(
  [...loaderSource.matchAll(/\bresolve_config_document\s*\(/gu)].length,
  1,
  "the memoized loader must remain the only resolved-document call site",
);
assert.ok(loaderSource.includes("CONFIG_LOADER"));
assert.ok(loaderSource.includes("Arc::clone"));

verifyRenderedReports();

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-cli-config-schema-census",
      tableCount: manifest.tables.length,
      productVerbCount: verbNames.size,
      subTableLessVerbCount: manifest.subTableLessVerbs.length,
      translationValidationModeCount: translationModes.length,
      parserCallSiteCount: parserCallCount - 1,
    },
    null,
    2,
  )}\n`,
);

function verifyRenderedReports(): void {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), "omena-config-census-"));
  try {
    mkdirSync(path.join(fixtureRoot, "src"));
    writeFileSync(path.join(fixtureRoot, "src", "input.css"), ".card { color: red; }\n");
    writeFileSync(
      path.join(fixtureRoot, "omena.toml"),
      [
        "[lint]",
        'profile = "recommended"',
        "profileTypo = true",
        "[verify]",
        'translationValidation = "staged"',
        "",
      ].join("\n"),
    );
    const run = spawnSync(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-cli",
        "--bin",
        "omena",
        "--",
        "build",
        path.join(fixtureRoot, "src", "input.css"),
        "--json",
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stderr, /omena config \[unknownKey\].*profileTypo/u);
    assert.doesNotMatch(run.stderr, /omena config \[notYetConsumed\] lint/u);
    assert.match(run.stderr, /omena config \[notYetConsumed\] verify\.translationValidation/u);
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
}

function assertCompatibilityDebt(): void {
  const debt = debtLedger.entries.find(({ id }) => id === manifest.compatibilityDebtId);
  assert.ok(debt, `${manifest.compatibilityDebtId} must be registered in the debt ledger`);
  assert.equal(debt.mechanism, "omena-cli-build-config-alias");
  assert.equal(debt.kind, "compat-window");
  assert.match(debt.introduced_in, /^\d{4}-\d{2}-\d{2}$/u);
  assert.match(debt.expiry.after_reference_date, /^\d{4}-\d{2}-\d{2}$/u);
  assert.ok(
    debt.expiry.after_reference_date > debt.introduced_in,
    "the legacy build config alias must have a future expiry",
  );

  const receipt = debtLedger.client_receipts.find(
    ({ client }) => client === manifest.compatibilityDebtId,
  );
  assert.ok(receipt, `${manifest.compatibilityDebtId} must have a client receipt`);
  assert.deepEqual(receipt.accepted_kinds, ["compat-window"]);
  assert.equal(receipt.authority, "omena CLI config schema census");
}

function extractStructFields(source: string, name: string): string[] {
  const body = extractBlock(source, `struct ${name}`);
  return body
    .split("\n")
    .flatMap((line) => line.match(/^\s+pub\(crate\)\s+([a-z][a-z0-9_]*):/u)?.slice(1) ?? []);
}

function extractAllStructFields(source: string): string[] {
  return [...source.matchAll(/pub\(crate\)\s+([a-z][a-z0-9_]*):/gu)].map((match) => match[1]!);
}

function extractEnumVariants(source: string, name: string): string[] {
  const body = extractBlock(source, `enum ${name}`);
  return body
    .split("\n")
    .flatMap((line) => line.match(/^\s+([A-Z][A-Za-z0-9_]*),/u)?.slice(1) ?? []);
}

function extractBlock(source: string, declaration: string): string {
  const start = source.indexOf(declaration);
  assert.notEqual(start, -1, `missing ${declaration}`);
  const bodyStart = source.indexOf("{", start) + 1;
  let depth = 1;
  let cursor = bodyStart;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated ${declaration}`);
  return source.slice(bodyStart, cursor - 1);
}

function toCamelCase(value: string): string {
  const lower = value.replace(/^[A-Z]/u, (character) => character.toLowerCase());
  return lower.replace(/_([a-z])/gu, (_, character: string) => character.toUpperCase());
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
