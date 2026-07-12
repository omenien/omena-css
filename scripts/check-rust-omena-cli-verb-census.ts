import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type VerbStatus = "stub" | "reserved-alias" | "wired";

interface VerbRow {
  readonly verb: string;
  readonly status: VerbStatus;
  readonly wiredBy: string | null;
}

interface VerbManifest {
  readonly schemaVersion: string;
  readonly product: string;
  readonly verbs: readonly VerbRow[];
}

interface DebtLedger {
  readonly entries: readonly {
    readonly id: string;
    readonly mechanism: string;
    readonly kind: string;
    readonly expiry: { readonly after_reference_date: string };
  }[];
}

interface ConfigSchemaManifest {
  readonly tables: readonly { readonly table: string; readonly verb: string | null }[];
  readonly subTableLessVerbs: readonly string[];
}

const requiredVerbs = [
  "check",
  "lint",
  "fmt",
  "minify",
  "bundle",
  "modules",
  "sass",
  "intel",
  "migrate",
  "verify",
  "ci",
  "explain",
] as const;

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const commandsSource = read("rust/crates/omena-cli/src/commands.rs");
const dispatchSource = read("rust/crates/omena-cli/src/dispatch.rs");
const productVerbSource = read("rust/crates/omena-cli/src/product_verb.rs");
const manifest = JSON.parse(read("rust/crates/omena-cli/verb-census.json")) as VerbManifest;
const debtLedger = JSON.parse(read("rust/omena-debt-ledger.json")) as DebtLedger;
const configSchema = JSON.parse(
  read("rust/crates/omena-cli/config-schema-census.json"),
) as ConfigSchemaManifest;

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.product-verb-census");

const productVariants = extractEnumVariants(productVerbSource, "ProductVerb");
const commandVariants = new Set(extractEnumVariants(commandsSource, "Command"));
const productNames = productVariants.map(toKebabCase);
assert.deepEqual(
  productNames,
  [...requiredVerbs],
  "ProductVerb must preserve the product contract",
);

for (const variant of productVariants) {
  assert.ok(commandVariants.has(variant), `Command is missing product variant ${variant}`);
}

const derivedRows = productVariants.map(deriveDispatchRow);
assert.deepEqual(
  manifest.verbs,
  derivedRows,
  "verb manifest must match command and dispatch source-derived state",
);
assert.equal(
  derivedRows.filter((row) => row.status === "stub").length,
  8,
  "router must retain eight unwired product slots after modules is connected",
);
assert.deepEqual(
  derivedRows.filter((row) => row.status === "wired").map((row) => row.verb),
  ["lint", "fmt", "modules"],
  "lint, formatting, and modules must be directly wired product verbs",
);
assert.deepEqual(
  derivedRows.filter((row) => row.status === "reserved-alias").map((row) => row.verb),
  ["check"],
  "only check may be a reserved compatibility alias",
);
assertFactsAliasExpiry(debtLedger);
const configMappedVerbs = configSchema.tables.flatMap(({ verb }) => (verb ? [verb] : []));
assert.deepEqual(
  [...new Set([...configMappedVerbs, ...configSchema.subTableLessVerbs])].sort(),
  [...requiredVerbs].sort(),
  "the shared config partial-map must classify every product verb",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-cli-verb-census",
      verbs: derivedRows.length,
      stub: derivedRows.filter((row) => row.status === "stub").length,
      reservedAlias: derivedRows.filter((row) => row.status === "reserved-alias").length,
      wired: derivedRows.filter((row) => row.status === "wired").length,
    },
    null,
    2,
  )}\n`,
);

function deriveDispatchRow(variant: string): VerbRow {
  const arm = extractCommandArm(dispatchSource, variant);
  if (arm.includes("run_reserved_facts_alias")) {
    assert.equal(variant, "Check", "facts compatibility alias may only occupy check");
    return { verb: toKebabCase(variant), status: "reserved-alias", wiredBy: "facts_file" };
  }

  const stubMarker = `CliExit::not_yet_wired(ProductVerb::${variant})`;
  if (arm.includes(stubMarker)) {
    return { verb: toKebabCase(variant), status: "stub", wiredBy: null };
  }

  const handler = arm.match(/=>\s*(?:return\s+)?([a-z][a-z0-9_]*)\s*\(/u)?.[1];
  assert.ok(handler, `wired product command ${variant} must expose a direct handler call`);
  return { verb: toKebabCase(variant), status: "wired", wiredBy: handler };
}

function assertFactsAliasExpiry(ledger: DebtLedger): void {
  const entry = ledger.entries.find((candidate) => candidate.id === "omena-cli-facts-alias-window");
  assert.ok(entry, "the parser-facts compatibility alias must retain an expiry entry");
  assert.equal(entry.mechanism, "omena-cli-facts-command-alias");
  assert.equal(entry.kind, "compat-window");
  assert.ok(
    entry.expiry.after_reference_date > "2026-07-11",
    "the parser-facts compatibility alias expiry must follow its introduction date",
  );
}

function extractCommandArm(source: string, variant: string): string {
  const marker = `        Command::${variant}`;
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, `dispatch is missing Command::${variant}`);
  const next = source.indexOf("\n        Command::", start + marker.length);
  const matchEnd = source.indexOf("\n    };", start + marker.length);
  const end = next === -1 || (matchEnd !== -1 && matchEnd < next) ? matchEnd : next;
  assert.ok(end > start, `could not delimit dispatch arm for ${variant}`);
  return source.slice(start, end);
}

function extractEnumVariants(source: string, enumName: string): readonly string[] {
  const declaration = `enum ${enumName} {`;
  const declarationStart = source.indexOf(declaration);
  assert.notEqual(declarationStart, -1, `missing enum ${enumName}`);
  const bodyStart = source.indexOf("{", declarationStart) + 1;
  let depth = 1;
  let cursor = bodyStart;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated enum ${enumName}`);
  return source
    .slice(bodyStart, cursor - 1)
    .split("\n")
    .flatMap((line) => line.match(/^    ([A-Z][A-Za-z0-9_]*)(?:,|\s*\{|\s*\()/u)?.slice(1) ?? []);
}

function toKebabCase(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase();
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
