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
const sourcePaths = [
  "rust/crates/omena-cascade-proof/src/lib.rs",
  "rust/crates/omena-transform-passes/src/runtime/cascade_proof.rs",
  "rust/crates/omena-transform-passes/src/runtime/executor.rs",
  "rust/crates/omena-transform-egg/src/lib.rs",
  "rust/crates/omena-evidence-graph/src/lib.rs",
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
assert.ok(
  [assertEntryUnresolved, injectClassificationFlip, injectArtifactHandEdit].filter(Boolean)
    .length <= 1,
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

const exit = buildSnapshot(readWorkingTree(), "WORKTREE", false);
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

assert.deepEqual(observed, generated, "self-produced evidence census is stale or hand-edited");
validateExit(generated.exit);
process.stdout.write(renderReceipt(generated));

function buildSnapshot(
  files: readonly SourceFile[],
  sourceRef: string,
  entryMode: boolean,
): Snapshot {
  const byPath = new Map(files.map((file) => [file.sourcePath, file.source]));
  const cascade = requiredSource(byPath, sourcePaths[1]);
  const egg = requiredSource(byPath, sourcePaths[3]);
  const evidence = requiredSource(byPath, sourcePaths[4]);
  const rows = [
    ...scanCascadeObligationRows(cascade, entryMode),
    ...scanEggRows(egg, entryMode),
    scanEvidenceMintRow(evidence, entryMode),
  ].sort((left, right) => compareText(left.siteKey, right.siteKey));
  const requirements = scanRequirements(requiredSource(byPath, sourcePaths[0]));
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
  assert.deepEqual(planeCounts, { cascadeObligation: 12, eGraph: 3, evidenceMint: 1 });
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
  };
}

function scanCascadeObligationRows(source: string, entryMode: boolean): CensusRow[] {
  const collectors = [
    "collect_cascade_proof_obligations_for_pass_input",
    "collect_cascade_proof_obligations_for_ir_pass_input",
  ] as const;
  const helperPattern =
    /\b(shorthand_obligation|scope_obligation|layer_obligation|layer_inversion_obligation|layer_flatten_missing_bundle_obligation|supports_obligation|stale_prefix_removal_obligation)\s*\(/g;
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
      const disposition = entryMode ? unresolvedDisposition() : cascadeDisposition(helper);
      rows.push({
        siteKey,
        site: siteAt(sourcePaths[1], source, offset, `${collector} -> ${helper}`),
        plane: "cascadeObligation",
        decidingValue: cascadeDecidingValue(helper),
        ...disposition,
        entryStateObservation:
          "the producer emitted an accepted/provenance value before any census field named an outside recheck",
      });
    }
  }
  return rows;
}

function scanEggRows(source: string, entryMode: boolean): CensusRow[] {
  const producers = [
    "selector_rewrite_witnesses",
    "calc_rewrite_witnesses",
    "stale_prefix_removal_witnesses",
  ] as const;
  return producers.map((producer) => {
    const span = functionSpan(source, producer);
    const legacy = span.body.indexOf("EggRewriteProofV0::new(");
    const checked = span.body.indexOf("CheckedEggRewriteProofV0::");
    const proofOffset = checked >= 0 ? checked : legacy;
    assert.ok(proofOffset >= 0, `${producer} has no syntactic proof construction site`);
    const disposition = entryMode
      ? unresolvedDisposition()
      : producer === "selector_rewrite_witnesses"
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
      site: siteAt(sourcePaths[3], source, span.start + proofOffset, producer),
      plane: "eGraph" as const,
      decidingValue:
        producer === "selector_rewrite_witnesses"
          ? "selector rewrite admission"
          : `${producer} legacy admission disposition`,
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
      sourcePaths[4],
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

function cascadeDisposition(
  helper: string,
): Pick<CensusRow, "independentlyRecheckedBy" | "owner" | "reentryCondition"> {
  if (["scope_obligation", "layer_obligation", "layer_inversion_obligation"].includes(helper)) {
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

function cascadeDecidingValue(helper: string): string {
  switch (helper) {
    case "shorthand_obligation":
    case "scope_obligation":
    case "layer_obligation":
      return "accepted from producer-authored require:<name>=<bool> strings";
    case "layer_inversion_obligation":
      return "accepted from inversion backend verdict plus ledger-backed evidence";
    case "layer_flatten_missing_bundle_obligation":
      return "literal rejected missing-bundle obligation";
    case "supports_obligation":
      return "accepted from producer-owned static-supports verdict";
    case "stale_prefix_removal_obligation":
      return "literal accepted exact-peer obligation";
    default:
      assert.fail(`unknown cascade obligation helper: ${helper}`);
  }
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
}

function functionSpan(source: string, name: string): { start: number; body: string } {
  const signature = new RegExp(`(?:pub(?:\\([^)]*\\))?\\s+)?fn\\s+${name}\\b`, "g");
  const match = signature.exec(source);
  assert.ok(match, `missing function ${name}`);
  const brace = source.indexOf("{", match.index);
  assert.ok(brace >= 0, `missing body for ${name}`);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return { start: brace + 1, body: source.slice(brace + 1, index) };
      }
    }
  }
  assert.fail(`unterminated function ${name}`);
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
    `artifactSha256=${hash(JSON.stringify(artifact))}`,
    "",
  ].join("\n");
}
