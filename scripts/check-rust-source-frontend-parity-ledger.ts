import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const ledgerPath = path.join(repoRoot, "rust/omena-source-frontend-parity-ledger.json");

interface SymbolEvidence {
  readonly path: string;
  readonly symbols: readonly string[];
}

interface EntryGate {
  readonly id: string;
  readonly status: "met" | "blocked";
  readonly evidence: readonly SymbolEvidence[];
}

interface Component {
  readonly id: "syntax" | "binding" | "sparse-cfg";
  readonly status: "TS_OWNED" | "RETIRED";
  readonly rustAuthority: string;
  readonly oracle: string;
  readonly oracleStatus: "not-built" | "partial-green" | "green";
  readonly tsLiveSurfaces: readonly SymbolEvidence[];
}

interface Survivor {
  readonly id: string;
  readonly status: "TYPE_ORACLE_SURVIVOR";
  readonly boundary: string;
  readonly surfaces: readonly SymbolEvidence[];
}

interface SourceFrontendParityLedger {
  readonly schemaVersion: 0;
  readonly product: "omena.source-frontend-parity-ledger";
  readonly entryGates: readonly EntryGate[];
  readonly components: readonly Component[];
  readonly survivors: readonly Survivor[];
  readonly forbiddenBeforeOracleGreen: readonly string[];
}

const ledger = JSON.parse(readFileSync(ledgerPath, "utf8")) as SourceFrontendParityLedger;

assert.equal(ledger.schemaVersion, 0);
assert.equal(ledger.product, "omena.source-frontend-parity-ledger");
assert.deepEqual(ledger.entryGates.map((gate) => gate.id).toSorted(), [
  "g5-source-symbol-identity",
  "g7-session-transaction-semantic-graph",
  "g8-tsgo-type-oracle-capabilities",
]);

for (const gate of ledger.entryGates) {
  assert.equal(gate.status, "met", `${gate.id} must be met before G11 enters committed scope`);
  assert.ok(gate.evidence.length > 0, `${gate.id} must carry file/symbol evidence`);
  assertEvidence(gate.evidence, `entry gate ${gate.id}`);
}

assert.deepEqual(
  ledger.components.map((component) => component.id),
  ["syntax", "binding", "sparse-cfg"],
);

for (const component of ledger.components) {
  assert.ok(
    component.rustAuthority.includes("::"),
    `${component.id} needs a Rust authority anchor`,
  );
  if (component.status === "TS_OWNED") {
    assertEvidence(component.tsLiveSurfaces, `component ${component.id}`);
  } else {
    assert.equal(
      component.oracleStatus,
      "green",
      `${component.id} cannot be RETIRED before its oracle is green`,
    );
    assertNoLiveEvidence(component.tsLiveSurfaces, `component ${component.id}`);
  }
}

assert.equal(ledger.survivors.length, 1);
const [tsgo] = ledger.survivors;
assert.equal(tsgo?.id, "tsgo-type-oracle");
assert.equal(tsgo?.status, "TYPE_ORACLE_SURVIVOR");
assert.ok(
  tsgo?.boundary.includes("type-query provider"),
  "tsgo survivor boundary must stay type-query-only",
);
assertEvidence(tsgo?.surfaces ?? [], "tsgo survivor");

const bridgeSource = readRepoFile("rust/crates/omena-bridge/src/source_syntax.rs");
for (const forbidden of [
  "target.binding == identifier.name.as_str()",
  "binding.binding == argument.binding",
  "target.binding == style_binding",
  "recipe.local_name == binding",
]) {
  assert.equal(
    bridgeSource.includes(forbidden),
    false,
    `G5 identity gate regressed to name matching: ${forbidden}`,
  );
}

const tsgoClient = readRepoFile("rust/crates/omena-tsgo-client/src/lib.rs");
assert.match(tsgoClient, /ProviderUnresolvedDisciplineV0::UnknownNotGuess/);
assert.match(tsgoClient, /TSGO_TYPE_ORACLE_PROVIDER_KIND_V0:\s*&str\s*=\s*"type-oracle"/);

console.log(
  JSON.stringify(
    {
      product: "omena.source-frontend-parity-ledger.check",
      entryGates: ledger.entryGates.length,
      components: ledger.components.map(({ id, status, oracleStatus }) => ({
        id,
        status,
        oracleStatus,
      })),
      survivors: ledger.survivors.map(({ id, status }) => ({ id, status })),
    },
    null,
    2,
  ),
);

function assertEvidence(evidence: readonly SymbolEvidence[], label: string): void {
  for (const item of evidence) {
    const content = readRepoFile(item.path);
    assert.ok(item.symbols.length > 0, `${label} evidence ${item.path} has no symbols`);
    for (const symbol of item.symbols) {
      assert.ok(
        content.includes(symbol),
        `${label} evidence missing ${JSON.stringify(symbol)} in ${item.path}`,
      );
    }
  }
}

function assertNoLiveEvidence(evidence: readonly SymbolEvidence[], label: string): void {
  for (const item of evidence) {
    const content = readRepoFileOrNull(item.path);
    if (content === null) continue;
    for (const symbol of item.symbols) {
      assert.equal(
        content.includes(symbol),
        false,
        `${label} retired surface still contains ${JSON.stringify(symbol)} in ${item.path}`,
      );
    }
  }
}

function readRepoFile(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readRepoFileOrNull(relativePath: string): string | null {
  try {
    return readRepoFile(relativePath);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
}
