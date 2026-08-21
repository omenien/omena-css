import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const catalogCratePath = path.join("rust/crates", ["omena", ["law", "vere"].join("")].join("-"));
const catalogPath = path.join(catalogCratePath, "src/lib.rs");
const proofKernelPath = "rust/crates/omena-cascade-proof/src/proof_kernel.rs";
const executorPath = "rust/crates/omena-transform-passes/src/runtime/executor.rs";
const eggPath = "rust/crates/omena-transform-egg/src/lib.rs";
const independencePath = path.join(catalogCratePath, "src/independence.rs");
const dataPath = path.join(catalogCratePath, "data/transform-catalog-independence-v0.json");
const args = new Set(process.argv.slice(2));
const injectRankHint = args.has("--inject-rank-hint");
const injectO1Bypass = args.has("--inject-o1-bypass");
const injectDependentPair = args.has("--inject-dependent-pair");
const injectEmptyIndependence = args.has("--inject-empty-independence");
const injectModuleExportTokenBypass = args.has("--inject-module-export-token-bypass");
const injectRewriteAssumptionParameter = args.has("--inject-rewrite-assumption-parameter");

assert.ok(
  [
    injectRankHint,
    injectO1Bypass,
    injectDependentPair,
    injectEmptyIndependence,
    injectModuleExportTokenBypass,
    injectRewriteAssumptionParameter,
  ].filter(Boolean).length <= 1,
  "only one proof-kernel falsifier may run at once",
);

let catalogSource = read(catalogPath);
let proofKernelSource = read(proofKernelPath);
let executorSource = read(executorPath);
const eggSource = read(eggPath);
const independenceSource = read(independencePath);
const independenceData = JSON.parse(read(dataPath)) as {
  entries: { leftPassId: string; rightPassId: string; disposition: string }[];
  profiles: readonly unknown[];
};
const injectedIndependenceData = structuredClone(independenceData);

if (injectRankHint) {
  catalogSource = catalogSource.replace(
    "rank_clusters: transform_catalog_independence_clusters_v0(requested),",
    "rank_clusters: transform_catalog_equation_clusters_v0(requested_pass_ids.as_slice()),",
  );
}
if (injectO1Bypass) {
  executorSource = executorSource.replace(
    "checked_token_ownership_admission_v0(census, module_instance, pass_kind).is_none()",
    "false",
  );
}
if (injectModuleExportTokenBypass) {
  executorSource = executorSource.replace(
    "require_dual_o1_tokens_v0(",
    "bypass_module_export_token_v0(",
  );
}
if (injectRewriteAssumptionParameter) {
  proofKernelSource += [
    "",
    "pub struct ExternalProofDataProbeV0;",
    "",
    "pub fn evaluate_external_rewrite_context_probe_v0(",
    "    before: &RewriteTermV0,",
    "    after: &RewriteTermV0,",
    "    rule_catalog: &RewriteRuleCatalogV0,",
    "    certificate: &RewriteCertificateEnvelopeV0,",
    "    external_data: &ExternalProofDataProbeV0,",
    ") -> Result<RewriteIssuanceTokenV0, CertificateRejectionV0> {",
    "    check_rewrite_certificate_v0(before, after, rule_catalog, certificate)",
    "}",
    "",
  ].join("\n");
}
if (injectDependentPair) {
  const dependent = injectedIndependenceData.entries.find(
    (entry) =>
      entry.leftPassId === "color-mix-lowering" && entry.rightPassId === "color-function-lowering",
  );
  assert.ok(dependent, "dependent-pair injection fixture is absent");
  dependent.disposition = "independent";
}
if (injectEmptyIndependence) {
  injectedIndependenceData.entries = [];
}

const planBody = functionBody(catalogSource, "plan_transform_catalog_parallel_layers_v0");
const layerBody = functionBody(catalogSource, "transform_catalog_independence_clusters_v0");
const o1Body = functionBody(executorSource, "closed_world_admission_o1_reasons");
const tokenConsumerBody = functionBody(executorSource, "checked_token_ownership_admission_v0");
const executionBody = functionBody(
  executorSource,
  "execute_transform_passes_on_source_with_active_lex_cache",
);
const moduleExportConsumerBody = functionBody(
  executorSource,
  "checked_module_export_preservation_admission_v0",
);
const dualTokenConsumerBody = functionBody(executorSource, "require_dual_o1_tokens_v0");
const moduleExportCheckerBody = functionBody(
  proofKernelSource,
  "check_module_export_preservation_v0",
);
const selectorDecisionBody = functionBody(eggSource, "decide_checked_egg_rewrite");
const rewriteCheckerApis = [
  { name: "check_rewrite_certificate_v0", argumentCount: 4 },
  { name: "check_optional_rewrite_certificate_v0", argumentCount: 4 },
  { name: "check_serialized_rewrite_certificate_v0", argumentCount: 4 },
] as const;
const trackedRustSources = trackedRustSourceFiles().map((sourceFile) =>
  sourceFile.sourcePath === proofKernelPath
    ? { ...sourceFile, source: proofKernelSource }
    : sourceFile,
);
const rewriteCheckerCallers = rewriteCheckerApis.flatMap((api) =>
  trackedRustSources.flatMap((sourceFile) =>
    scanFunctionCalls(sourceFile.sourcePath, sourceFile.source, api.name).map((call) => ({
      ...call,
      api: api.name,
      expectedArgumentCount: api.argumentCount,
    })),
  ),
);
const publicFunctionContracts = scanPublicFunctionContracts(proofKernelSource);
const publicFunctionInputs = publicFunctionContracts.flatMap((contract) =>
  contract.parameters.map((parameter) => ({
    functionName: contract.functionName,
    parameterName: parameter.name,
    parameterType: parameter.type,
    consumed: parameterIsConsumed(parameter.name, contract.body),
  })),
);
const nonConsumedPublicInputs = publicFunctionInputs.filter((input) => !input.consumed);
assert.equal(
  nonConsumedPublicInputs.length,
  0,
  "public proof-kernel inputs must be consumed independent of identifier spelling: " +
    JSON.stringify(nonConsumedPublicInputs),
);
const assumptionShapedInputs = publicFunctionContracts.flatMap((contract) =>
  contract.parameters
    .filter((parameter) => assumptionShapedParameter(parameter))
    .map((parameter) => ({
      functionName: contract.functionName,
      parameterName: parameter.name,
      parameterType: parameter.type,
      consumed: parameterIsConsumed(parameter.name, contract.body),
    })),
);
const nonConsumedAssumptionInputs = assumptionShapedInputs.filter((input) => !input.consumed);
assert.equal(
  nonConsumedAssumptionInputs.length,
  0,
  "public assumption-shaped inputs must be semantically consumed: " +
    JSON.stringify(nonConsumedAssumptionInputs),
);
assert.doesNotMatch(
  proofKernelSource,
  /CanonicalRewriteAssumptions?V0|CANONICAL_REWRITE_ASSUMPTIONS_SCHEMA_VERSION_V0|RewriteCheckInputV0::Assumptions|DuplicateAssumption/u,
  "orphan rewrite-assumption vocabulary must stay removed from the public kernel surface",
);

for (const api of rewriteCheckerApis) {
  const signature = functionSignature(proofKernelSource, api.name);
  assert.doesNotMatch(
    signature,
    /CanonicalRewriteAssumptionsV0|\bassumptions?\b/u,
    `${api.name} must not imply that arbitrary assumptions constrain its verdict`,
  );
}
assert.doesNotMatch(
  proofKernelSource,
  /\bfn\s+validate_assumptions\s*\(/u,
  "the inert rewrite-assumption validation helper must stay removed",
);
for (const caller of rewriteCheckerCallers) {
  assert.equal(
    caller.argumentCount,
    caller.expectedArgumentCount,
    `${caller.api} caller ${caller.sourcePath}:${caller.line} has an undeclared argument`,
  );
  assert.doesNotMatch(
    caller.argumentsText,
    /CanonicalRewriteAssumptionsV0|\bassumptions?\b/u,
    `${caller.api} caller ${caller.sourcePath}:${caller.line} passes a semantically inert assumption`,
  );
}
assert.ok(rewriteCheckerCallers.length > 0, "rewrite-checker caller census is empty");

assert.match(planBody, /transform_catalog_independence_clusters_v0\(requested\)/);
assert.doesNotMatch(planBody, /transform_catalog_equation_clusters_v0/);
assert.doesNotMatch(planBody, /transform_catalog_execution_rank_hint/);
assert.doesNotMatch(layerBody, /transform_catalog_equation_clusters_v0/);
assert.doesNotMatch(layerBody, /transform_catalog_execution_rank_hint/);
assert.match(planBody, /scheduler_status: "independenceDataReady"/);
assert.match(planBody, /executor_consumes_plan: false/);
assert.match(
  catalogSource,
  /TRANSFORM_CATALOG_PLAN_NON_CONSUMPTION_REASON_V0:[\s\S]*executorKeepsValidatedSerialDagUntilParallelApplicationSemanticsLand/,
);
assert.match(o1Body, /checked_token_ownership_admission_v0/);
assert.match(o1Body, /proofKernelToken:/);
assert.match(tokenConsumerBody, /check_rewrite_certificate_v0/);
assert.match(tokenConsumerBody, /matches_endpoints_v0/);
assert.match(tokenConsumerBody, /matches_catalog_v0/);
assert.match(executionBody, /checked_module_export_preservation_admission_v0/);
assert.match(executionBody, /require_dual_o1_tokens_v0/);
assert.match(moduleExportConsumerBody, /ModuleExportPreservationCertV0::new/);
assert.match(moduleExportConsumerBody, /matches_endpoints_v0/);
assert.match(moduleExportConsumerBody, /matches_catalog_v0/);
assert.match(dualTokenConsumerBody, /ownership_token/);
assert.match(dualTokenConsumerBody, /preservation/);
assert.match(moduleExportCheckerBody, /module_export_token_support_v0/);
assert.match(proofKernelSource, /OrderedTokenWordV0/);
assert.match(proofKernelSource, /TokenSupportV0/);
assert.match(proofKernelSource, /token_support_v0/);
assert.match(selectorDecisionBody, /selector_rewrite_rule_catalog_v0/);
assert.match(selectorDecisionBody, /matches_catalog_v0/);
assert.match(proofKernelSource, /catalog_content_digest:\s*\[u8; 32\]/);
assert.match(independenceSource, /default_transform_observation_matrix_v0\(\)/);
assert.match(independenceSource, /checked_adjacent_swap_token_v0/);
assert.match(independenceSource, /canonicalize_transform_catalog_schedule_v0/);
assert.equal(independenceData.profiles.length, 1);
assert.equal(
  independenceData.entries.filter((entry) => entry.disposition === "independent").length,
  1,
);

const kernelOutput = cargoTest(
  "omena-cascade-proof",
  "proof_kernel::tests::transform_independence_requires_observation_and_precondition_halves",
);
const catalogDigestOutput = cargoTest(
  "omena-cascade-proof",
  "proof_kernel::tests::catalog_digest_is_order_independent_and_content_sensitive",
);
const spoofedCatalogOutput = cargoTest(
  "omena-transform-egg",
  "tests::checked_selector_rejects_token_issued_by_spoofed_catalog",
);
const ownershipOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::proof_kernel_token_closes_favourable_ownership_count_bypass",
  true,
);
const moduleExportForgeOutput = cargoTest(
  "omena-cascade-proof",
  "proof_kernel::tests::module_export_preservation_rejects_drop_identity_swap_and_digest_tamper",
);
const moduleExportProductionOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::four_export_preserving_passes_construct_checked_tokens_on_the_production_path",
);
const moduleExportHashingOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::class_hashing_rolls_back_a_non_bijective_module_export_rotation",
);
const dualO1Output = cargoTest(
  "omena-transform-passes",
  "runtime::executor::dispatch_table_tests::destructive_o1_admission_requires_ownership_and_preservation_tokens",
);
const r15ObservationOutput = cargoTest(
  "omena-transform-passes",
  "runtime::observation_projection::tests::executed_whitespace_pass_is_selector_equivalent_but_not_raw_equivalent",
);
const r16ObservationOutput = cargoTest(
  "omena-transform-passes",
  "runtime::observation_projection::tests::executed_pass_baseline_detects_cascade_winner_corruption",
);
const executableTruthOutput = cargoTest(
  "omena-transform-passes",
  "runtime::observation_projection::tests::executable_truth_table_runs_product_pass_and_authority_projections",
);
const r18Output = cargoTest(
  "omena-lawvere",
  "independence::tests::canonical_schedule_agrees_with_independent_bounded_oracle",
  true,
);
const productRowsOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::committed_independence_rows_match_product_transform_outputs",
  true,
);
const r19ProductOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::nested_color_lowering_conflict_has_swapped_order_output_divergence",
  true,
);
const r19CheckerOutput = cargoTest(
  "omena-lawvere",
  "independence::tests::dependent_pair_injection_is_rejected_by_data_and_s1_checker",
  true,
);
const r20Output = cargoTest(
  "omena-lawvere",
  "independence::tests::empty_independence_data_collapses_parallel_width_to_one",
  true,
);
const injectedDataOutput = cargoTest(
  "omena-lawvere",
  "independence::tests::gate_supplied_independence_data_satisfies_product_invariants",
  true,
  {
    OMENA_PROOF_KERNEL_INDEPENDENCE_JSON: JSON.stringify(injectedIndependenceData),
  },
);

assert.match(kernelOutput, /test result: ok/);
assert.match(catalogDigestOutput, /test result: ok/);
assert.match(spoofedCatalogOutput, /test result: ok/);
assert.match(ownershipOutput, /test result: ok/);
assert.match(moduleExportForgeOutput, /test result: ok/);
assert.match(moduleExportProductionOutput, /test result: ok/);
assert.match(moduleExportHashingOutput, /test result: ok/);
assert.match(dualO1Output, /test result: ok/);
assert.match(
  r15ObservationOutput,
  /selectorProjection passExecuted=true pass=whitespace-strip selectorMatching=true rawBytes=false/,
);
assert.match(
  r16ObservationOutput,
  /cascadeWinnerProjection passExecuted=true pass=whitespace-strip passCascadeWinner=true corruptedCascadeWinner=false disjointSelectorMatching=true/,
);
assert.match(executableTruthOutput, /test result: ok/);
assert.match(r18Output, /bound=4 permutations=24[\s\S]*canonicalOracleAgreement=true/);
assert.match(productRowsOutput, /test result: ok/);
assert.match(r19ProductOutput, /equal=false/);
assert.match(r19CheckerOutput, /dataValidation=Err/);
assert.match(r19CheckerOutput, /S1 checker rejected reorder certificate/);
assert.match(r20Output, /entries=0 layers=2 maxParallelWidth=1/);
assert.match(injectedDataOutput, /injectedDataValidation=Ok independentPairs=1 maxParallelWidth=2/);

process.stdout.write(
  [
    "proof-kernel gate: ok",
    "checkerSideConditions=tokenOwnershipSeparability,transformIndependence,moduleExportPreservation",
    "catalogBinding=schemaId+contentDigest trustedCatalogSelectedByConsumer",
    "observationProfiles=1 independentPairs=1 dependentPairs=1",
    "executableObservationKinds=selectorMatching,cascadeWinner",
    "scheduleOracleBound=4 schedulePermutations=24",
    "executorConsumesPlan=false nonConsumptionReason=executorKeepsValidatedSerialDagUntilParallelApplicationSemanticsLand",
    "o1Consumer=closed_world_admission_o1_reasons+require_dual_o1_tokens_v0",
    `rewriteCheckerApis=${rewriteCheckerApis.length} rewriteCheckerCallers=${rewriteCheckerCallers.length} trackedRustSourceFiles=${trackedRustSources.length}`,
    `rewriteCheckerCallerCensus=${rewriteCheckerCallers.map((caller) => `${caller.api}@${caller.sourcePath}:${caller.line}`).join(",")}`,
    "publicFunctionContracts=" +
      publicFunctionContracts.length +
      " publicFunctionInputs=" +
      publicFunctionInputs.length +
      " nonConsumedPublicInputs=0" +
      " assumptionShapedInputs=" +
      assumptionShapedInputs.length +
      " nonConsumedAssumptionInputs=0",
    "orphanRewriteAssumptionVocabulary=0 removedHelper=validate_assumptions",
    "",
  ].join("\n"),
);

function cargoTest(
  packageName: string,
  testName: string,
  allFeatures = false,
  extraEnv: Readonly<Record<string, string>> = {},
): string {
  const cargoArgs = ["test", "--manifest-path", "rust/Cargo.toml", "-p", packageName];
  if (allFeatures) cargoArgs.push("--all-features");
  cargoArgs.push(testName, "--", "--exact", "--nocapture");
  const result = spawnSync("cargo", cargoArgs, {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, ...extraEnv },
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  if (result.status !== 0) process.stderr.write(output);
  assert.equal(result.status, 0, `${packageName} ${testName} failed`);
  return output;
}

function read(sourcePath: string): string {
  return fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
}

interface RustSourceFile {
  readonly sourcePath: string;
  readonly source: string;
}

interface FunctionCallSite {
  readonly sourcePath: string;
  readonly line: number;
  readonly argumentCount: number;
  readonly argumentsText: string;
}

interface PublicFunctionParameter {
  readonly name: string;
  readonly type: string;
}

interface PublicFunctionContract {
  readonly functionName: string;
  readonly parameters: readonly PublicFunctionParameter[];
  readonly body: string;
}

function scanPublicFunctionContracts(source: string): readonly PublicFunctionContract[] {
  const masked = maskRustCommentsAndStrings(source);
  const pattern =
    /\bpub(?:\s*\([^)]*\))?\s+(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gu;
  const contracts: PublicFunctionContract[] = [];
  for (const match of masked.matchAll(pattern)) {
    const functionName = match[1];
    const anchor = match.index ?? 0;
    const openParen = masked.indexOf("(", anchor + match[0].length);
    assert.notEqual(openParen, -1, "missing parameter list for public function " + functionName);
    const closeParen = matchingDelimiterEnd(masked, openParen, "(", ")");
    const openBrace = masked.indexOf("{", closeParen + 1);
    assert.notEqual(openBrace, -1, "missing body for public function " + functionName);
    const closeBrace = matchingDelimiterEnd(masked, openBrace, "{", "}");
    const parameters = splitTopLevel(masked.slice(openParen + 1, closeParen), ",")
      .map((parameter) => parameter.trim())
      .filter(Boolean)
      .filter((parameter) => !/^(?:&\s*(?:mut\s+)?|mut\s+)?self$/u.test(parameter))
      .map((parameter) => {
        const colon = topLevelSeparatorIndex(parameter, ":");
        assert.notEqual(
          colon,
          -1,
          "unsupported public parameter shape in " + functionName + ": " + parameter,
        );
        const name = parameter
          .slice(0, colon)
          .trim()
          .replace(/^mut\s+/u, "")
          .replace(/^ref\s+/u, "");
        return { name, type: parameter.slice(colon + 1).trim() };
      });
    contracts.push({
      functionName,
      parameters,
      body: source.slice(openBrace + 1, closeBrace),
    });
  }
  assert.ok(contracts.length > 0, "public proof-kernel function census is empty");
  return contracts;
}

function assumptionShapedParameter(parameter: PublicFunctionParameter): boolean {
  return /assum|premis|hypothes|side_?condition|context/iu.test(
    parameter.name + " " + parameter.type,
  );
}

function parameterIsConsumed(parameterName: string, body: string): boolean {
  const maskedBody = maskRustCommentsAndStrings(body);
  return new RegExp("\\b" + escapeRegExp(parameterName) + "\\b", "u").test(maskedBody);
}

function splitTopLevel(source: string, separator: string): readonly string[] {
  const parts: string[] = [];
  let start = 0;
  let parentheses = 0;
  let brackets = 0;
  let braces = 0;
  let angles = 0;
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    if (character === "(") parentheses += 1;
    if (character === ")") parentheses -= 1;
    if (character === "[") brackets += 1;
    if (character === "]") brackets -= 1;
    if (character === "{") braces += 1;
    if (character === "}") braces -= 1;
    if (character === "<") angles += 1;
    if (character === ">") angles = Math.max(0, angles - 1);
    if (
      character === separator &&
      parentheses === 0 &&
      brackets === 0 &&
      braces === 0 &&
      angles === 0
    ) {
      parts.push(source.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(source.slice(start));
  return parts;
}

function topLevelSeparatorIndex(source: string, separator: string): number {
  const parts = splitTopLevel(source, separator);
  return parts.length > 1 ? parts[0].length : -1;
}

function matchingDelimiterEnd(
  source: string,
  open: number,
  openCharacter: string,
  closeCharacter: string,
): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === openCharacter) depth += 1;
    if (source[index] === closeCharacter) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  assert.fail("unterminated " + openCharacter + closeCharacter + " delimiter");
}

function escapeRegExp(value: string): string {
  return value.replace(/[|\\{}()[\]^$+*?.-]/gu, "\\$&");
}

function trackedRustSourceFiles(): readonly RustSourceFile[] {
  const sourcePaths = execFileSync("git", ["ls-files", ":(glob)rust/crates/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  })
    .trim()
    .split("\n")
    .filter(Boolean);
  assert.ok(sourcePaths.length > 0, "tracked Rust source census is empty");
  return sourcePaths.map((sourcePath) => ({ sourcePath, source: read(sourcePath) }));
}

function functionSignature(source: string, functionName: string): string {
  const anchor = source.indexOf(`fn ${functionName}`);
  assert.notEqual(anchor, -1, `missing function ${functionName}`);
  const brace = source.indexOf("{", anchor);
  assert.notEqual(brace, -1, `missing signature terminator for ${functionName}`);
  return source.slice(anchor, brace);
}

function scanFunctionCalls(
  sourcePath: string,
  source: string,
  functionName: string,
): readonly FunctionCallSite[] {
  const masked = maskRustCommentsAndStrings(source);
  const pattern = new RegExp(`\\b${functionName}\\s*\\(`, "gu");
  const calls: FunctionCallSite[] = [];
  for (const match of masked.matchAll(pattern)) {
    const anchor = match.index ?? 0;
    const prefix = masked.slice(Math.max(0, anchor - 24), anchor);
    if (/\bfn\s*$/u.test(prefix)) continue;
    const openParen = masked.indexOf("(", anchor + functionName.length);
    const parsed = parseCallArguments(source, masked, openParen);
    calls.push({
      sourcePath,
      line: source.slice(0, anchor).split("\n").length,
      argumentCount: parsed.argumentCount,
      argumentsText: parsed.argumentsText,
    });
  }
  return calls;
}

function parseCallArguments(
  source: string,
  masked: string,
  openParen: number,
): { readonly argumentCount: number; readonly argumentsText: string } {
  assert.notEqual(openParen, -1, "function call has no opening parenthesis");
  let parentheses = 0;
  let brackets = 0;
  let braces = 0;
  const commaOffsets: number[] = [];
  for (let index = openParen + 1; index < masked.length; index += 1) {
    const character = masked[index];
    if (character === "(") parentheses += 1;
    if (character === ")") {
      if (parentheses === 0 && brackets === 0 && braces === 0) {
        const argumentsText = source.slice(openParen + 1, index);
        const trailingArgumentStart = (commaOffsets.at(-1) ?? openParen) + 1;
        const hasTrailingArgument = masked.slice(trailingArgumentStart, index).trim().length > 0;
        const argumentCount =
          argumentsText.trim().length === 0 ? 0 : commaOffsets.length + Number(hasTrailingArgument);
        return { argumentCount, argumentsText };
      }
      parentheses -= 1;
    }
    if (character === "[") brackets += 1;
    if (character === "]") brackets -= 1;
    if (character === "{") braces += 1;
    if (character === "}") braces -= 1;
    if (character === "," && parentheses === 0 && brackets === 0 && braces === 0) {
      commaOffsets.push(index);
    }
  }
  assert.fail("unterminated function call");
}

function maskRustCommentsAndStrings(source: string): string {
  const masked = [...source];
  let blockCommentDepth = 0;
  let inLineComment = false;
  let inString = false;
  let escaped = false;
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    const next = source[index + 1];
    if (inLineComment) {
      if (character === "\n") inLineComment = false;
      else masked[index] = " ";
      continue;
    }
    if (blockCommentDepth > 0) {
      if (character === "/" && next === "*") {
        masked[index] = " ";
        masked[index + 1] = " ";
        blockCommentDepth += 1;
        index += 1;
      } else if (character === "*" && next === "/") {
        masked[index] = " ";
        masked[index + 1] = " ";
        blockCommentDepth -= 1;
        index += 1;
      } else if (character !== "\n") {
        masked[index] = " ";
      }
      continue;
    }
    if (inString) {
      if (character !== "\n") masked[index] = " ";
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') inString = false;
      continue;
    }
    if (character === "/" && next === "/") {
      masked[index] = " ";
      masked[index + 1] = " ";
      inLineComment = true;
      index += 1;
      continue;
    }
    if (character === "/" && next === "*") {
      masked[index] = " ";
      masked[index + 1] = " ";
      blockCommentDepth = 1;
      index += 1;
      continue;
    }
    if (character === '"') {
      masked[index] = " ";
      inString = true;
    }
  }
  return masked.join("");
}

function functionBody(source: string, functionName: string): string {
  const anchor = source.indexOf(`fn ${functionName}`);
  assert.notEqual(anchor, -1, `missing function ${functionName}`);
  const brace = source.indexOf("{", anchor);
  assert.notEqual(brace, -1, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(brace + 1, index);
    }
  }
  assert.fail(`unterminated function ${functionName}`);
}
