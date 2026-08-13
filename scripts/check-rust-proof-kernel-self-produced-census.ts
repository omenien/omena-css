import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type Plane = "cascadeObligation" | "eGraph" | "evidenceMint";
type RequirementClassification = "IR_COMPUTED" | "PRODUCER_PASS_THROUGH" | "UNCLASSIFIED";

interface SourceFile {
  readonly sourcePath: string;
  readonly source: string;
}

interface Site {
  readonly sourcePath: string;
  readonly symbol: string;
  readonly lineHint: number;
  readonly lineSha256: string;
}

interface CensusRow {
  readonly siteKey: string;
  readonly site: Site;
  readonly plane: Plane;
  readonly decidingValue: string;
  readonly independentlyRecheckedBy: string;
  readonly owner: string | null;
  readonly reentryCondition: string | null;
  readonly entryStateObservation: string;
}

interface RequirementRow {
  readonly constructor: string;
  readonly name: string;
  readonly classification: RequirementClassification;
  readonly site: Site;
}

interface Snapshot {
  readonly sourceRef: string;
  readonly sourceFileSha256: string;
  readonly rowCount: number;
  readonly planeCounts: Readonly<Record<Plane, number>>;
  readonly rows: readonly CensusRow[];
  readonly requirementCountsByConstructor: Readonly<Record<string, number>>;
  readonly requirementClassificationCounts: Readonly<Record<RequirementClassification, number>>;
  readonly requirements: readonly RequirementRow[];
  readonly certificateConstructionCounts: Readonly<
    Record<string, { readonly production: number; readonly test: number }>
  >;
}

interface Artifact {
  readonly schemaVersion: "0";
  readonly product: "omena-proof-kernel.self-produced-evidence-census";
  readonly committedSourceFiles: readonly string[];
  readonly cannotCheck: readonly string[];
  readonly defaultBackendDisposition: string;
  readonly entry: Snapshot;
  readonly exit: Snapshot;
}

const repoRoot = process.cwd();
const artifactPath = "rust/omena-proof-kernel-self-produced-evidence-census.json";
const entrySourceRef = "0749a00707b07070ec0fed3342c836a674facd14";
const cascadeProofLibPath = "rust/crates/omena-cascade-proof/src/lib.rs";
const proofKernelPath = "rust/crates/omena-cascade-proof/src/proof_kernel.rs";
const cascadeProducerPath = "rust/crates/omena-transform-passes/src/runtime/cascade_proof.rs";
const executorPath = "rust/crates/omena-transform-passes/src/runtime/executor.rs";
const eggProducerPath = "rust/crates/omena-transform-egg/src/lib.rs";
const evidenceGraphPath = "rust/crates/omena-evidence-graph/src/lib.rs";
const sourcePaths = [
  cascadeProofLibPath,
  proofKernelPath,
  cascadeProducerPath,
  executorPath,
  eggProducerPath,
  evidenceGraphPath,
] as const;
const cannotCheck = [
  "rule catalog soundness",
  "whether an observer profile models browser behaviour",
  "a defect shared with an external side-condition source",
  "a defect that changes both comparison sides identically",
  "independent semantic strength for genuinely IR-computed requirements",
] as const;
const defaultBackendDisposition =
  "StubSmtBackendV0 re-reads require:<name>=<bool>; non-require audit labels are satisfied by unwrap_or(true) and do not contribute a checked formula.";

const args = new Set(process.argv.slice(2));
const writeMode = args.has("--write");
const assertEntryUnresolved = args.has("--assert-entry-unresolved");
const injectClassificationFlip = args.has("--inject-classification-flip");
const injectArtifactHandEdit = args.has("--inject-artifact-hand-edit");
const injectRecheckCallDeletion = args.has("--inject-recheck-call-deletion");
const injectNewSelfProducedSite = args.has("--inject-new-self-produced-site");
assert.ok(
  [
    assertEntryUnresolved,
    injectClassificationFlip,
    injectArtifactHandEdit,
    injectRecheckCallDeletion,
    injectNewSelfProducedSite,
  ].filter(Boolean).length <= 1,
  "only one falsifier mode may run at once",
);
assert.ok(!writeMode || process.argv.length === 3, "--write cannot be combined with falsifiers");

const entry = buildSnapshot(readSourceRef(entrySourceRef), entrySourceRef, true);
if (assertEntryUnresolved) {
  const unresolved = entry.rows.filter(
    (row) =>
      row.independentlyRecheckedBy === "NONE" &&
      row.owner === null &&
      row.reentryCondition === null,
  );
  assert.equal(
    unresolved.length,
    0,
    [
      `entry census unresolved: ${unresolved.length}/${entry.rows.length} rows have independentlyRecheckedBy=NONE with no owner/re-entry`,
      `planeCounts=${JSON.stringify(entry.planeCounts)}`,
      `unclassifiedRequirements=${entry.requirementClassificationCounts.UNCLASSIFIED}`,
    ].join("\n"),
  );
  process.exit(0);
}

let exitFiles = readWorkingTree();
if (injectRecheckCallDeletion) {
  exitFiles = mutateRecheckCallDeletion(exitFiles);
}
if (injectNewSelfProducedSite) {
  exitFiles = mutateNewSelfProducedSite(exitFiles);
}
const exit = buildSnapshot(exitFiles, "WORKTREE", false);
const generated: Artifact = {
  schemaVersion: "0",
  product: "omena-proof-kernel.self-produced-evidence-census",
  committedSourceFiles: sourcePaths,
  cannotCheck,
  defaultBackendDisposition,
  entry,
  exit,
};

if (writeMode) {
  fs.writeFileSync(path.join(repoRoot, artifactPath), `${JSON.stringify(generated, null, 2)}\n`);
  process.stdout.write(renderReceipt(generated));
  process.exit(0);
}

const committed = readJson<Artifact>(artifactPath);
let observed = committed;
if (injectClassificationFlip) {
  assert.ok(committed.exit.requirements.length > 0, "classification injection needs a requirement");
  const [first, ...rest] = committed.exit.requirements;
  const flipped = first.classification === "IR_COMPUTED" ? "PRODUCER_PASS_THROUGH" : "IR_COMPUTED";
  observed = {
    ...committed,
    exit: {
      ...committed.exit,
      requirements: [{ ...first, classification: flipped }, ...rest],
    },
  };
}
if (injectArtifactHandEdit) {
  assert.ok(committed.exit.rows.length > 0, "hand-edit injection needs a census row");
  const [first, ...rest] = committed.exit.rows;
  observed = {
    ...committed,
    exit: {
      ...committed.exit,
      rows: [
        {
          ...first,
          site: { ...first.site, lineHint: first.site.lineHint + 1 },
        },
        ...rest,
      ],
    },
  };
}

if (injectRecheckCallDeletion) {
  validateExit(generated.exit);
}
if (injectNewSelfProducedSite) {
  assert.equal(
    generated.exit.rowCount,
    committed.exit.rowCount,
    "new self-produced constructor site was discovered; run the official updater",
  );
}
assert.deepEqual(observed, generated, "self-produced evidence census is stale or hand-edited");
validateExit(generated.exit);
process.stdout.write(renderReceipt(generated));

function buildSnapshot(
  files: readonly SourceFile[],
  sourceRef: string,
  entryMode: boolean,
): Snapshot {
  const byPath = new Map(files.map((file) => [file.sourcePath, file.source]));
  const proofKernel = requiredSource(byPath, proofKernelPath);
  const cascade = requiredSource(byPath, cascadeProducerPath);
  const executor = requiredSource(byPath, executorPath);
  const egg = requiredSource(byPath, eggProducerPath);
  const evidence = requiredSource(byPath, evidenceGraphPath);
  const rows = [
    ...scanKernelIssuanceRows(proofKernel, entryMode),
    ...scanCascadeObligationRows(cascade, executor, entryMode),
    ...scanEggRows(egg, entryMode),
    scanEvidenceMintRow(evidence, entryMode),
  ].sort((left, right) => compareText(left.siteKey, right.siteKey));
  const requirements = scanRequirements(requiredSource(byPath, cascadeProofLibPath));
  const planeCounts = countBy<Plane>(
    rows.map((row) => row.plane),
    ["cascadeObligation", "eGraph", "evidenceMint"],
  );
  const requirementCountsByConstructor = Object.fromEntries(
    [...new Set(requirements.map((row) => row.constructor))]
      .sort()
      .map((constructor) => [
        constructor,
        requirements.filter((row) => row.constructor === constructor).length,
      ]),
  );
  const requirementClassificationCounts = countBy<RequirementClassification>(
    requirements.map((row) => row.classification),
    ["IR_COMPUTED", "PRODUCER_PASS_THROUGH", "UNCLASSIFIED"],
  );
  assert.deepEqual(requirementCountsByConstructor, {
    canonical_box_shorthand_combination_input_v0: 5,
    canonical_layer_flatten_candidate_input_v0: 4,
    canonical_longhand_merge_input_v0: 5,
    canonical_scope_flatten_candidate_input_v0: 5,
    canonical_static_supports_condition_input_v0: 2,
    canonical_transform_rewrite_candidate_input_v0: 5,
  });
  return {
    sourceRef,
    sourceFileSha256: hash(
      files
        .map((file) => `${file.sourcePath}\0${hash(file.source)}`)
        .sort()
        .join("\n"),
    ),
    rowCount: rows.length,
    planeCounts,
    rows,
    requirementCountsByConstructor,
    requirementClassificationCounts,
    requirements,
    certificateConstructionCounts: scanCertificateConstructionCounts(proofKernel),
  };
}

function scanKernelIssuanceRows(source: string, entryMode: boolean): CensusRow[] {
  const productionSource = source.slice(0, testModuleStart(source));
  const rows = functionSpans(productionSource).flatMap((span) => {
    const pattern = /\bRewriteIssuanceTokenV0::issue\s*\(/gu;
    return [...span.body.matchAll(pattern)].map((match, index) => ({ span, match, index }));
  });
  assert.ok(rows.length > 0, "checker issuance constructor is absent");
  return rows.map(({ span, match, index }) => ({
    siteKey: `eGraph:${span.name}:issuance:${index + 1}`,
    site: siteAt(
      proofKernelPath,
      source,
      span.start + (match.index ?? 0),
      `${span.name} -> RewriteIssuanceTokenV0::issue`,
    ),
    plane: "eGraph" as const,
    decidingValue:
      "sealed witness-issuance token bound to catalog identity and endpoints; transform application is not decided here",
    ...(entryMode
      ? unresolvedDisposition()
      : {
          independentlyRecheckedBy: "NONE",
          owner: "proof-kernel consumers",
          reentryCondition:
            "every consuming gate must compare the sealed catalog identity and endpoints with its internally selected trusted catalog before using the token",
        }),
    entryStateObservation:
      "the checker issued an endpoint token from a caller-supplied catalog without sealing catalog identity",
  }));
}

function scanCascadeObligationRows(
  source: string,
  executorSource: string,
  entryMode: boolean,
): CensusRow[] {
  const collectors = [
    "collect_cascade_proof_obligations_for_pass_input",
    "collect_cascade_proof_obligations_for_ir_pass_input",
  ] as const;
  const helperNames = discoverCascadeProducerHelpers(source);
  assert.ok(helperNames.length > 0, "no cascade obligation producer helpers were discovered");
  const helperPattern = new RegExp(`\\b(${helperNames.map(escapeRegExp).join("|")})\\s*\\(`, "g");
  const rows: CensusRow[] = [];
  for (const collector of collectors) {
    const span = functionSpan(source, collector);
    let match: RegExpExecArray | null;
    const ordinals = new Map<string, number>();
    while ((match = helperPattern.exec(span.body)) !== null) {
      const helper = match[1];
      const ordinal = (ordinals.get(helper) ?? 0) + 1;
      ordinals.set(helper, ordinal);
      const offset = span.start + match.index;
      const siteKey = `cascadeObligation:${collector}:${helper}:${ordinal}`;
      const disposition = entryMode
        ? unresolvedDisposition()
        : cascadeDispositionFromCalls(source, executorSource, helper);
      rows.push({
        siteKey,
        site: siteAt(cascadeProducerPath, source, offset, `${collector} -> ${helper}`),
        plane: "cascadeObligation",
        decidingValue: cascadeDecidingValue(source, helper),
        ...disposition,
        entryStateObservation:
          "the producer emitted an accepted/provenance value before any census field named an outside recheck",
      });
    }
  }
  return rows;
}

function scanEggRows(source: string, entryMode: boolean): CensusRow[] {
  const productionSource = source.slice(0, testModuleStart(source));
  const producers = functionSpans(productionSource).filter((span) =>
    /\b(?:CheckedEggRewriteProofV0|EggRewriteProofV0)::new\s*\(/u.test(span.body),
  );
  return producers.map((span) => {
    const producer = span.name;
    const legacy = span.body.indexOf("EggRewriteProofV0::new(");
    const checked = span.body.indexOf("CheckedEggRewriteProofV0::");
    const proofOffset = checked >= 0 ? checked : legacy;
    assert.ok(proofOffset >= 0, `${producer} has no syntactic proof construction site`);
    const disposition = entryMode
      ? unresolvedDisposition()
      : functionCallClosureContains(productionSource, producer, "check_rewrite_certificate_v0")
        ? {
            independentlyRecheckedBy: "omena_cascade_proof::check_rewrite_certificate_v0",
            owner: null,
            reentryCondition: null,
          }
        : {
            independentlyRecheckedBy: "NONE",
            owner: "omena-transform-egg maintainers",
            reentryCondition:
              "replace the declared structural disposition with a matching S1 certificate before tightening this producer",
          };
    return {
      siteKey: `eGraph:${producer}`,
      site: siteAt(eggProducerPath, source, span.start + proofOffset, producer),
      plane: "eGraph" as const,
      decidingValue: functionCallClosureContains(
        productionSource,
        producer,
        "check_rewrite_certificate_v0",
      )
        ? "post-transform witness emission guarded by a derivability-only checker token; transform application already occurred"
        : `${producer} post-transform witness emission under a declared structural gap`,
      ...disposition,
      entryStateObservation:
        "EggRewriteProofV0 accepted caller-owned specificity/provenance/witness values without an outside endpoint derivation",
    };
  });
}

function scanEvidenceMintRow(source: string, entryMode: boolean): CensusRow {
  const needle = "pub fn from_discharge_cell_key_v0";
  const offset = source.indexOf(needle);
  assert.ok(offset >= 0, "LedgerDischargeWitnessV0 shape-mint constructor is absent");
  return {
    siteKey: "evidenceMint:LedgerDischargeWitnessV0::from_discharge_cell_key_v0",
    site: siteAt(
      evidenceGraphPath,
      source,
      offset,
      "LedgerDischargeWitnessV0::from_discharge_cell_key_v0",
    ),
    plane: "evidenceMint",
    decidingValue: "64-character ASCII-hex key shape",
    ...(entryMode
      ? unresolvedDisposition()
      : {
          independentlyRecheckedBy: "NONE",
          owner: "omena-evidence-graph maintainers",
          reentryCondition:
            "replace shape-only minting with a matched accepted discharge-ledger lookup before treating this witness as semantic evidence",
        }),
    entryStateObservation:
      "a correctly shaped missing ledger key could mint a witness without a ledger verdict",
  };
}

function scanRequirements(source: string): RequirementRow[] {
  const constructors = [
    "canonical_box_shorthand_combination_input_v0",
    "canonical_longhand_merge_input_v0",
    "canonical_scope_flatten_candidate_input_v0",
    "canonical_layer_flatten_candidate_input_v0",
    "canonical_static_supports_condition_input_v0",
    "canonical_transform_rewrite_candidate_input_v0",
  ] as const;
  const functionNames = [
    ["smt_ir_computed_requirement_v0", "IR_COMPUTED"],
    ["smt_producer_pass_through_requirement_v0", "PRODUCER_PASS_THROUGH"],
    ["smt_require_term_v0", "UNCLASSIFIED"],
  ] as const;
  const rows: RequirementRow[] = [];
  for (const constructor of constructors) {
    const span = functionSpan(source, constructor);
    for (const [functionName, classification] of functionNames) {
      const pattern = new RegExp(`${functionName}\\s*\\(\\s*"([^"]+)"`, "g");
      let match: RegExpExecArray | null;
      while ((match = pattern.exec(span.body)) !== null) {
        rows.push({
          constructor,
          name: match[1],
          classification,
          site: siteAt(
            sourcePaths[0],
            source,
            span.start + match.index,
            `${constructor}:${match[1]}`,
          ),
        });
      }
    }
  }
  return rows.sort((left, right) =>
    compareText(`${left.constructor}:${left.name}`, `${right.constructor}:${right.name}`),
  );
}

function cascadeDispositionFromCalls(
  source: string,
  executorSource: string,
  helper: string,
): Pick<CensusRow, "independentlyRecheckedBy" | "owner" | "reentryCondition"> {
  const producerReachesLedgerEvidence =
    functionCallClosureContains(source, helper, "ledger_backed_discharge_evidence") ||
    functionCallClosureContains(source, helper, "ledger_backed_inversion_discharge_evidence");
  const consumerCallsIndependentRecheck = functionCallClosureContains(
    executorSource,
    "flatten_discharge_precondition_failure",
    "has_matching_discharge_evidence",
  );
  if (producerReachesLedgerEvidence && consumerCallsIndependentRecheck) {
    return {
      independentlyRecheckedBy:
        "omena_transform_passes::runtime::executor::has_matching_discharge_evidence",
      owner: null,
      reentryCondition: null,
    };
  }
  return {
    independentlyRecheckedBy: "NONE",
    owner: "omena-transform-passes maintainers",
    reentryCondition:
      "add a certificate or independently produced ledger cell for this obligation before strengthening its acceptance claim",
  };
}

function cascadeDecidingValue(source: string, helper: string): string {
  const body = functionSpan(source, helper).body;
  if (body.includes("check_layer_flatten_inversion(")) {
    return "accepted from inversion backend verdict plus ledger-backed evidence";
  }
  if (body.includes("classify_canonical_requirements_v0(")) {
    return "accepted from producer-authored require:<name>=<bool> strings";
  }
  if (body.includes("accepted: false")) {
    return "literal rejected missing-bundle obligation";
  }
  if (body.includes("witness.verdict != StaticSupportsEvalVerdictV0::Unknown")) {
    return "accepted from producer-owned static-supports verdict";
  }
  if (/proof_obligation\([\s\S]*?\btrue\s*,/u.test(body)) {
    return "literal accepted producer-owned structural obligation";
  }
  return "producer-owned cascade obligation value";
}

function unresolvedDisposition(): Pick<
  CensusRow,
  "independentlyRecheckedBy" | "owner" | "reentryCondition"
> {
  return { independentlyRecheckedBy: "NONE", owner: null, reentryCondition: null };
}

function validateExit(snapshot: Snapshot): void {
  const unresolved = snapshot.rows.filter(
    (row) =>
      row.independentlyRecheckedBy === "NONE" &&
      (!row.owner?.trim() || !row.reentryCondition?.trim()),
  );
  assert.deepEqual(
    unresolved,
    [],
    "every NONE disposition must carry a non-empty owner and re-entry condition",
  );
  assert.equal(snapshot.requirementClassificationCounts.UNCLASSIFIED, 0);
  assert.equal(snapshot.requirementClassificationCounts.IR_COMPUTED, 24);
  assert.equal(snapshot.requirementClassificationCounts.PRODUCER_PASS_THROUGH, 2);
  const selector = snapshot.rows.find((row) => row.siteKey === "eGraph:selector_rewrite_witnesses");
  assert.equal(
    selector?.independentlyRecheckedBy,
    "omena_cascade_proof::check_rewrite_certificate_v0",
    "selector witness producer lost its independently rechecked call path",
  );
  for (const [certificate, counts] of Object.entries(snapshot.certificateConstructionCounts)) {
    assert.equal(
      counts.production,
      0,
      `${certificate} unexpectedly acquired a production construction site`,
    );
  }
}

function discoverCascadeProducerHelpers(source: string): string[] {
  const productionSource = source.slice(0, testModuleStart(source));
  return functionSpans(productionSource)
    .filter(
      (span) =>
        span.name !== "proof_obligation" &&
        (span.body.includes("proof_obligation(") ||
          span.body.includes("TransformCascadeProofObligationV0 {")),
    )
    .map((span) => span.name)
    .sort(compareText);
}

function functionCallClosureContains(source: string, root: string, target: string): boolean {
  const spans = new Map(functionSpans(source).map((span) => [span.name, span]));
  const pending = [root];
  const visited = new Set<string>();
  const targetPattern = new RegExp(`\\b${escapeRegExp(target)}\\s*\\(`, "u");
  while (pending.length > 0) {
    const current = pending.pop();
    if (current === undefined || visited.has(current)) continue;
    visited.add(current);
    const span = spans.get(current);
    if (span === undefined) continue;
    if (targetPattern.test(span.body)) return true;
    for (const call of span.body.matchAll(/\b([a-z][a-z0-9_]*)\s*\(/gu)) {
      if (spans.has(call[1]) && !visited.has(call[1])) pending.push(call[1]);
    }
  }
  return false;
}

function scanCertificateConstructionCounts(
  source: string,
): Record<string, { production: number; test: number }> {
  const testStart = testModuleStart(source);
  const sections = {
    production: source.slice(0, testStart),
    test: source.slice(testStart),
  };
  return Object.fromEntries(
    ["CascadeWinnerEqualityCertV0", "ComputedValueEqualityCertV0", "SourceMapTraceCertV0"].map(
      (certificate) => [
        certificate,
        {
          production: countStructConstructions(sections.production, certificate),
          test: countStructConstructions(sections.test, certificate),
        },
      ],
    ),
  );
}

function countStructConstructions(source: string, typeName: string): number {
  const pattern = new RegExp(`\\b${escapeRegExp(typeName)}\\s*\\{`, "gu");
  return [...source.matchAll(pattern)].filter((match) => {
    const prefix = source.slice(Math.max(0, match.index - 32), match.index);
    return !/\b(?:pub\s+)?struct\s*$/u.test(prefix);
  }).length;
}

interface FunctionSourceSpan {
  readonly name: string;
  readonly start: number;
  readonly body: string;
}

function functionSpans(source: string): FunctionSourceSpan[] {
  const masked = maskRustCommentsAndLiterals(source);
  const signature = /(?:pub(?:\([^)]*\))?\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\b/g;
  const spans: FunctionSourceSpan[] = [];
  let match: RegExpExecArray | null;
  while ((match = signature.exec(masked)) !== null) {
    const brace = masked.indexOf("{", match.index);
    assert.ok(brace >= 0, `missing body for ${match[1]}`);
    let depth = 0;
    let end = -1;
    for (let index = brace; index < masked.length; index += 1) {
      if (masked[index] === "{") depth += 1;
      if (masked[index] === "}") {
        depth -= 1;
        if (depth === 0) {
          end = index;
          break;
        }
      }
    }
    assert.ok(end >= 0, `unterminated function ${match[1]}`);
    spans.push({ name: match[1], start: brace + 1, body: source.slice(brace + 1, end) });
    signature.lastIndex = end + 1;
  }
  return spans;
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = source.split("");
  let index = 0;
  const blank = (start: number, end: number): void => {
    for (let cursor = start; cursor < end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") masked[cursor] = " ";
    }
  };
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end < 0 ? source.length : end;
      blank(index, stop);
      index = stop;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      blank(start, index);
      continue;
    }
    const rawPrefix = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (rawPrefix !== null) {
      const start = index;
      const terminator = `"${rawPrefix[1] ?? ""}`;
      index += rawPrefix[0].length;
      const end = source.indexOf(terminator, index);
      index = end < 0 ? source.length : end + terminator.length;
      blank(start, index);
      continue;
    }
    if (source[index] === '"') {
      const start = index;
      index += 1;
      let escaped = false;
      while (index < source.length) {
        const current = source[index];
        index += 1;
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      blank(start, index);
      continue;
    }
    const characterLiteral = source
      .slice(index)
      .match(/^'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|.)|[^'\\\r\n])'/u);
    if (characterLiteral !== null) {
      const start = index;
      index += characterLiteral[0].length;
      blank(start, index);
      continue;
    }
    index += 1;
  }
  return masked.join("");
}

function testModuleStart(source: string): number {
  const match = /#\[cfg\(test\)\]\s*mod\s+tests\s*\{/u.exec(source);
  return match?.index ?? source.length;
}

function mutateRecheckCallDeletion(files: readonly SourceFile[]): SourceFile[] {
  return files.map((file) => {
    if (file.sourcePath !== eggProducerPath) return file;
    const span = functionSpan(file.source, "selector_rewrite_issuance_token_v0");
    const relative = span.body.indexOf("check_rewrite_certificate_v0(");
    assert.ok(relative >= 0, "recheck deletion injection could not find the checker call");
    const offset = span.start + relative;
    return {
      sourcePath: file.sourcePath,
      source:
        file.source.slice(0, offset) +
        "removed_recheck_call_v0(" +
        file.source.slice(offset + "check_rewrite_certificate_v0(".length),
    };
  });
}

function mutateNewSelfProducedSite(files: readonly SourceFile[]): SourceFile[] {
  return files.map((file) => {
    if (file.sourcePath !== proofKernelPath) return file;
    const insertion = [
      "fn injected_self_produced_issuance_v0() {",
      "    let _ = RewriteIssuanceTokenV0::issue(todo!(), todo!(), todo!(), Vec::new());",
      "}",
      "",
    ].join("\n");
    const offset = testModuleStart(file.source);
    return {
      sourcePath: file.sourcePath,
      source: `${file.source.slice(0, offset)}${insertion}${file.source.slice(offset)}`,
    };
  });
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function functionSpan(source: string, name: string): { start: number; body: string } {
  const span = functionSpans(source).find((candidate) => candidate.name === name);
  assert.ok(span, `missing function ${name}`);
  return { start: span.start, body: span.body };
}

function siteAt(sourcePath: string, source: string, offset: number, symbol: string): Site {
  const lineStart = source.lastIndexOf("\n", Math.max(0, offset - 1)) + 1;
  const lineEndIndex = source.indexOf("\n", offset);
  const lineEnd = lineEndIndex < 0 ? source.length : lineEndIndex;
  const line = source.slice(lineStart, lineEnd);
  return {
    sourcePath,
    symbol,
    lineHint: source.slice(0, offset).split("\n").length,
    lineSha256: hash(line),
  };
}

function readSourceRef(sourceRef: string): SourceFile[] {
  return sourcePaths.map((sourcePath) => ({
    sourcePath,
    source: execFileSync("git", ["show", `${sourceRef}:${sourcePath}`], {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    }),
  }));
}

function readWorkingTree(): SourceFile[] {
  return sourcePaths.map((sourcePath) => ({
    sourcePath,
    source: fs.readFileSync(path.join(repoRoot, sourcePath), "utf8"),
  }));
}

function requiredSource(byPath: ReadonlyMap<string, string>, sourcePath: string): string {
  const source = byPath.get(sourcePath);
  assert.notEqual(source, undefined, `missing committed source ${sourcePath}`);
  return source;
}

function countBy<T extends string>(values: readonly T[], keys: readonly T[]): Record<T, number> {
  return Object.fromEntries(
    keys.map((key) => [key, values.filter((value) => value === key).length]),
  ) as Record<T, number>;
}

function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function readJson<T>(sourcePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, sourcePath), "utf8")) as T;
}

function renderReceipt(artifact: Artifact): string {
  return [
    "proof-kernel self-produced evidence census: ok",
    `entry rows=${artifact.entry.rowCount} planes=${JSON.stringify(artifact.entry.planeCounts)} unclassifiedRequirements=${artifact.entry.requirementClassificationCounts.UNCLASSIFIED}`,
    `exit rows=${artifact.exit.rowCount} planes=${JSON.stringify(artifact.exit.planeCounts)} requirements=${artifact.exit.requirementClassificationCounts.IR_COMPUTED}/${artifact.exit.requirementClassificationCounts.PRODUCER_PASS_THROUGH}/${artifact.exit.requirementClassificationCounts.UNCLASSIFIED}`,
    `certificateProductionSites=${Object.entries(artifact.exit.certificateConstructionCounts)
      .map(([certificate, counts]) => `${certificate}:${counts.production}`)
      .join(",")}`,
    `artifactSha256=${hash(JSON.stringify(artifact))}`,
    "",
  ].join("\n");
}
