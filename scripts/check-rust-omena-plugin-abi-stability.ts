import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

interface AbiConditionV0 {
  readonly id:
    | "abi-signature-frozen"
    | "outcome-fields-mandatory"
    | "consumption-law-closed"
    | "decision-evidence-precision-bound";
  readonly sourceKind: "source-derived";
  readonly authorityPaths: readonly string[];
}

interface PluginAbiStabilityContractV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.plugin-abi-stability-contract";
  readonly abiVersion: string;
  readonly externalPluginAbiStable: false;
  readonly traitSignatures: readonly string[];
  readonly outcomeFields: Readonly<Record<string, string>>;
  readonly conditions: readonly AbiConditionV0[];
}

const repoRoot = process.cwd();
const runnerRoot = path.join(repoRoot, "rust/crates/omena-query-transform-runner");
const apiRelativePath = "rust/crates/omena-query-transform-runner/src/plugin_api.rs";
const bridgeRelativePath = "rust/crates/omena-bridge/src/lib.rs";
const consumptionLawRelativePath = "scripts/check-rust-omena-plugin-consumption-law.ts";
const censusRelativePath =
  "rust/crates/omena-query-transform-runner/plugin-consumption-law-census.json";
const contractPath = path.join(runnerRoot, "plugin-abi-stability-contract.json");
const writeMode = process.argv.includes("--write");

let apiSource = fs.readFileSync(path.join(repoRoot, apiRelativePath), "utf8");
let bridgeSource = fs.readFileSync(path.join(repoRoot, bridgeRelativePath), "utf8");
if (process.argv.includes("--inject-abi-signature-drift")) {
  apiSource = apiSource.replace("fn metadata(&self)", "fn metadata_changed(&self)");
}
if (process.argv.includes("--inject-optional-outcome")) {
  apiSource = apiSource.replaceAll(
    "pub evidence_reference: EvidenceNodeKeyV0,",
    "pub evidence_reference: Option<EvidenceNodeKeyV0>,",
  );
}
if (process.argv.includes("--inject-decision-bypass")) {
  apiSource = apiSource.replace(
    "let execution_outcome = outcome.decision.compatibility_outcome();",
    "let execution_outcome = synthetic_outcome();",
  );
}
if (process.argv.includes("--simulate-early-flip")) {
  bridgeSource = bridgeSource.replace(
    "external_plugin_abi_stable: false,",
    "external_plugin_abi_stable: true,",
  );
}

const abiVersion = requiredCapture(
  apiSource,
  /pub const OMENA_PLUGIN_ABI_VERSION_V0: &str = "([^"]+)";/u,
  "plugin ABI version",
);
const traitBlock = extractNamedBlock(apiSource, "pub trait OmenaPlugin");
const traitSignatures = collectTraitSignatures(traitBlock);
assert.deepEqual(
  traitSignatures,
  [
    "fn metadata(&self) -> &'static PluginMetadataV0;",
    "fn analyze(&self, snapshot: &PluginWorkspaceSnapshotV0<'_>) -> PluginAnalysisV0;",
    "fn transform(&self, ir: &mut PluginTransformIrV0<'_>, context: PluginTransformContextV0) -> PluginOutcomeV0;",
  ],
  "OmenaPlugin ABI signature changed without an explicit compatibility decision",
);

const outcomeBlock = extractNamedBlock(apiSource, "pub struct PluginOutcomeV0");
const outcomeFields = collectPublicFieldTypes(outcomeBlock);
assert.deepEqual(
  outcomeFields,
  {
    change_summary: "PluginChangeSummaryV0",
    decision: "TransformDecision",
    evidence_reference: "EvidenceNodeKeyV0",
    precision: "FactPrecision",
  },
  "plugin outcomes must carry mandatory change, evidence, precision, and decision fields",
);
assert.ok(
  Object.values(outcomeFields).every((fieldType) => !fieldType.startsWith("Option<")),
  "plugin outcome obligations must not be optional",
);

const injectConstantReady = process.argv.includes("--inject-constant-ready");
const injectConsumptionViolation =
  process.argv.includes("--inject-consumption-violation") || injectConstantReady;
const consumptionResult = spawnSync(
  process.execPath,
  [
    "--import",
    "tsx",
    path.join(repoRoot, consumptionLawRelativePath),
    ...(injectConsumptionViolation ? ["--inject-forbidden-symbol"] : []),
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
const consumptionLawClosed = consumptionResult.status === 0;

const decisionEvidencePrecisionBound = [
  "outcome.decision.compatibility_outcome()",
  "PluginOutcomeValidationErrorV0::DecisionMismatch",
  "PluginOutcomeValidationErrorV0::EvidenceReferenceMismatch",
  "outcome.precision.satisfies(context.required_precision)",
].every((binding) => apiSource.includes(binding));

const externalPluginAbiStable =
  requiredCapture(
    bridgeSource,
    /external_plugin_abi_stable:\s*(true|false),/u,
    "external plugin ABI stability value",
  ) === "true";
assert.equal(
  externalPluginAbiStable,
  false,
  "external plugin loading must remain disabled until a separate release decision flips the boundary",
);
assert.ok(
  bridgeSource.includes("!boundary.external_plugin_abi_stable"),
  "the bridge boundary must continue to reject an early external ABI flip",
);

const conditions = [
  {
    id: "abi-signature-frozen",
    sourceKind: "source-derived",
    authorityPaths: [apiRelativePath],
    ready: traitSignatures.length === 3,
    observedReady: traitSignatures.length === 3,
  },
  {
    id: "outcome-fields-mandatory",
    sourceKind: "source-derived",
    authorityPaths: [apiRelativePath],
    ready: Object.keys(outcomeFields).length === 4,
    observedReady: Object.keys(outcomeFields).length === 4,
  },
  {
    id: "consumption-law-closed",
    sourceKind: "source-derived",
    authorityPaths: [consumptionLawRelativePath, censusRelativePath],
    ready: injectConstantReady ? true : consumptionLawClosed,
    observedReady: consumptionLawClosed,
  },
  {
    id: "decision-evidence-precision-bound",
    sourceKind: "source-derived",
    authorityPaths: [apiRelativePath, "rust/crates/omena-transform-passes/src/model.rs"],
    ready: decisionEvidencePrecisionBound,
    observedReady: decisionEvidencePrecisionBound,
  },
] as const;

assert.deepEqual(
  conditions
    .filter((condition) => condition.ready !== condition.observedReady)
    .map((condition) => condition.id),
  [],
  "reported plugin ABI readiness must equal independently observed source evidence",
);

assert.deepEqual(
  conditions.filter((condition) => !condition.ready).map((condition) => condition.id),
  [],
  `plugin ABI readiness condition failed${consumptionResult.stderr ? `: ${consumptionResult.stderr.trim()}` : ""}`,
);

const contract: PluginAbiStabilityContractV0 = {
  schemaVersion: "0",
  product: "omena.plugin-abi-stability-contract",
  abiVersion,
  externalPluginAbiStable: false,
  traitSignatures,
  outcomeFields,
  conditions: conditions.map(
    ({ ready: _ready, observedReady: _observedReady, ...condition }) => condition,
  ),
};
const serialized = `${JSON.stringify(contract, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(contractPath, serialized);
  formatJsonFile(contractPath);
} else {
  assert.ok(fs.existsSync(contractPath), "missing plugin ABI stability contract; run with --write");
  assert.deepEqual(
    JSON.parse(fs.readFileSync(contractPath, "utf8")),
    contract,
    "plugin ABI stability contract is stale; regenerate and review the compatibility surface",
  );
}

process.stdout.write(
  `Omena plugin ABI readiness OK: conditions=${conditions.length} externalStable=false abi=${abiVersion}\n`,
);

function requiredCapture(source: string, pattern: RegExp, label: string): string {
  const match = pattern.exec(source);
  assert.ok(match, `missing ${label}`);
  return match[1];
}

function extractNamedBlock(source: string, marker: string): string {
  const markerIndex = source.indexOf(marker);
  assert.ok(markerIndex >= 0, `missing Rust surface marker: ${marker}`);
  const openIndex = source.indexOf("{", markerIndex);
  assert.ok(openIndex >= 0, `missing Rust block for marker: ${marker}`);
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(markerIndex, index + 1);
  }
  throw new Error(`unterminated Rust block for marker: ${marker}`);
}

function collectTraitSignatures(traitBlock: string): string[] {
  return [
    ...traitBlock.matchAll(/\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*\([^;]+?\)\s*(?:->\s*[^;]+)?;/gsu),
  ].map((match) =>
    match[0]
      .replace(/\s+/gu, " ")
      .replace(/\(\s+/gu, "(")
      .replace(/,\s*\)/gu, ")")
      .trim(),
  );
}

function collectPublicFieldTypes(structBlock: string): Record<string, string> {
  return Object.fromEntries(
    [...structBlock.matchAll(/^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,]+),/gmu)]
      .map((match) => [match[1], match[2].trim()] as const)
      .toSorted(([left], [right]) => left.localeCompare(right)),
  );
}

function formatJsonFile(filePath: string): void {
  execFileSync(process.execPath, [path.join(repoRoot, "node_modules/oxfmt/bin/oxfmt"), filePath], {
    cwd: repoRoot,
    stdio: "inherit",
  });
}
