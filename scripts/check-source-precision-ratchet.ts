import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Tier =
  | "top"
  | "automaton"
  | "prefix"
  | "suffix"
  | "prefixSuffix"
  | "charInclusion"
  | "composite"
  | "finiteSet"
  | "exact"
  | "bottom";

interface ClassValue {
  readonly kind: Tier;
  readonly value?: string;
  readonly values?: readonly string[];
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLength?: number;
}

interface SourcePrecisionReference {
  readonly id: string;
  readonly resolvedTier: Tier;
  readonly precisionStratum: string;
  readonly resolvedValue: ClassValue;
  readonly topCause?: string;
}

interface SourcePrecisionBaseline {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.source-precision-baseline";
  readonly corpusManifestHash: string;
  readonly tierOrder: readonly Tier[];
  readonly totalReferenceCount: number;
  readonly nonTopReferenceCount: number;
  readonly topReferenceCount: number;
  readonly nonTopShareBasisPoints: number;
  readonly tierHistogram: Record<Tier, number>;
  readonly precisionStratumHistogram: Record<string, number>;
  readonly topCauseHistogram: Record<string, number>;
  readonly bottomWitnesses: Record<string, string>;
  readonly references: readonly SourcePrecisionReference[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baselinePath = path.join(repoRoot, "scripts", "source-precision-ratchet-baseline.json");
const tierOrder: readonly Tier[] = [
  "top",
  "automaton",
  "prefix",
  "suffix",
  "prefixSuffix",
  "charInclusion",
  "composite",
  "finiteSet",
  "exact",
  "bottom",
];

function readBaseline(): SourcePrecisionBaseline {
  assert.ok(
    existsSync(baselinePath),
    `source precision ratchet baseline is missing: ${path.relative(repoRoot, baselinePath)}`,
  );
  return parseBaseline(readFileSync(baselinePath, "utf8"), "committed baseline");
}

function readCurrentBaseline(): SourcePrecisionBaseline {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-diff-test",
      "--bin",
      "omena-source-precision-baseline",
      "--quiet",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 10 * 1024 * 1024,
    },
  );
  if (result.status !== 0) {
    throw new Error(
      `source precision baseline runner failed\n${result.stderr.trim()}\n${result.stdout.trim()}`,
    );
  }
  return parseBaseline(result.stdout, "current baseline");
}

function parseBaseline(source: string, label: string): SourcePrecisionBaseline {
  const parsed = JSON.parse(source) as SourcePrecisionBaseline;
  assert.equal(parsed.schemaVersion, "0", `${label} schemaVersion`);
  assert.equal(parsed.product, "omena-diff-test.source-precision-baseline", `${label} product`);
  assert.deepEqual(parsed.tierOrder, tierOrder, `${label} tier order`);
  assertNonVacuous(parsed, label);
  return parsed;
}

function assertNonVacuous(summary: SourcePrecisionBaseline, label: string): void {
  assert.ok(summary.totalReferenceCount > 0, `${label} must contain references`);
  assert.equal(
    summary.references.length,
    summary.totalReferenceCount,
    `${label} reference count must match entries`,
  );
  assert.ok(summary.nonTopReferenceCount > 0, `${label} must contain non-Top references`);
  assert.ok(summary.topReferenceCount > 0, `${label} must contain Top references`);

  const histogramTotal = tierOrder.reduce(
    (total, tier) => total + integerCount(summary.tierHistogram[tier], `${label}.${tier}`),
    0,
  );
  assert.equal(histogramTotal, summary.totalReferenceCount, `${label} histogram total`);
  const precisionStratumTotal = Object.values(summary.precisionStratumHistogram).reduce(
    (total, count) => total + integerCount(count, `${label}.precisionStratum`),
    0,
  );
  assert.equal(
    precisionStratumTotal,
    summary.totalReferenceCount,
    `${label} precision stratum total`,
  );
  assert.equal(
    summary.tierHistogram.top,
    summary.topReferenceCount,
    `${label} top histogram count`,
  );
  assert.equal(
    summary.totalReferenceCount - summary.topReferenceCount,
    summary.nonTopReferenceCount,
    `${label} non-Top count`,
  );
}

function assertNoRegression(
  baseline: SourcePrecisionBaseline,
  current: SourcePrecisionBaseline,
): void {
  assert.equal(
    current.corpusManifestHash,
    baseline.corpusManifestHash,
    "source precision corpus manifest changed; update the baseline with a recorded manifest transition",
  );
  assert.ok(
    current.topReferenceCount <= baseline.topReferenceCount,
    `Top references grew: baseline=${baseline.topReferenceCount} current=${current.topReferenceCount}`,
  );
  assert.ok(
    current.nonTopShareBasisPoints >= baseline.nonTopShareBasisPoints,
    `non-Top share fell: baseline=${baseline.nonTopShareBasisPoints} current=${current.nonTopShareBasisPoints}`,
  );

  assertBottomGrowthIsWitnessed(baseline, current);

  for (const tier of tierOrder) {
    if (tier === "top" || tier === "bottom") continue;
    assert.ok(
      current.tierHistogram[tier] >= baseline.tierHistogram[tier],
      `${tier} bucket regressed: baseline=${baseline.tierHistogram[tier]} current=${current.tierHistogram[tier]}`,
    );
  }

  assertPerReferenceMonotone(baseline, current);
}

function assertBottomGrowthIsWitnessed(
  baseline: SourcePrecisionBaseline,
  current: SourcePrecisionBaseline,
): void {
  const baselineBottomIds = new Set(
    baseline.references
      .filter((reference) => reference.resolvedTier === "bottom")
      .map((reference) => reference.id),
  );
  const newBottomReferences = current.references.filter(
    (reference) => reference.resolvedTier === "bottom" && !baselineBottomIds.has(reference.id),
  );
  if (newBottomReferences.length === 0) return;

  for (const reference of newBottomReferences) {
    assert.ok(
      current.bottomWitnesses[reference.id]?.length > 0,
      `new Bottom reference lacks a reachability or contradiction witness: ${reference.id}`,
    );
  }
}

function assertPerReferenceMonotone(
  baseline: SourcePrecisionBaseline,
  current: SourcePrecisionBaseline,
): void {
  const baselineById = new Map(baseline.references.map((reference) => [reference.id, reference]));
  for (const reference of current.references) {
    const previous = baselineById.get(reference.id);
    assert.ok(previous, `new reference ${reference.id} requires a baseline transition`);
    assert.ok(
      isSubsetOrEqual(reference.resolvedValue, previous.resolvedValue),
      `reference ${reference.id} is not a monotone refinement`,
    );
  }
}

function isSubsetOrEqual(left: ClassValue, right: ClassValue): boolean {
  if (JSON.stringify(left) === JSON.stringify(right)) return true;
  if (left.kind === "bottom") return true;
  if (right.kind === "top") return true;
  if (left.kind === "top") return right.kind === "top";

  if (left.kind === "exact" && typeof left.value === "string") {
    return matchesClassValue(right, left.value);
  }
  if (left.kind === "finiteSet" && Array.isArray(left.values)) {
    return left.values.every((value) => matchesClassValue(right, value));
  }
  if (left.kind === "prefix" && right.kind === "prefix") {
    return String(left.prefix ?? "").startsWith(String(right.prefix ?? ""));
  }
  if (left.kind === "suffix" && right.kind === "suffix") {
    return String(left.suffix ?? "").endsWith(String(right.suffix ?? ""));
  }
  if (left.kind === "prefixSuffix" && right.kind === "prefixSuffix") {
    return (
      String(left.prefix ?? "").startsWith(String(right.prefix ?? "")) &&
      String(left.suffix ?? "").endsWith(String(right.suffix ?? "")) &&
      Number(left.minLength ?? 0) >= Number(right.minLength ?? 0)
    );
  }
  return false;
}

function matchesClassValue(value: ClassValue, candidate: string): boolean {
  switch (value.kind) {
    case "top":
      return true;
    case "bottom":
      return false;
    case "exact":
      return value.value === candidate;
    case "finiteSet":
      return value.values?.includes(candidate) ?? false;
    case "prefix":
      return candidate.startsWith(String(value.prefix ?? ""));
    case "suffix":
      return candidate.endsWith(String(value.suffix ?? ""));
    case "prefixSuffix":
      return (
        candidate.startsWith(String(value.prefix ?? "")) &&
        candidate.endsWith(String(value.suffix ?? "")) &&
        candidate.length >= Number(value.minLength ?? 0)
      );
    default:
      return false;
  }
}

function integerCount(value: number, label: string): number {
  assert.ok(Number.isInteger(value) && value >= 0, `${label} must be a non-negative integer`);
  return value;
}

const baseline = readBaseline();
const current = readCurrentBaseline();
assertNoRegression(baseline, current);

if (
  current.topReferenceCount < baseline.topReferenceCount ||
  current.nonTopReferenceCount > baseline.nonTopReferenceCount
) {
  console.warn("source precision improved; review and update the committed baseline");
}

console.log(
  JSON.stringify(
    {
      product: "rust.source-precision-ratchet",
      corpusManifestHash: current.corpusManifestHash,
      totalReferenceCount: current.totalReferenceCount,
      nonTopReferenceCount: current.nonTopReferenceCount,
      topReferenceCount: current.topReferenceCount,
      nonTopShareBasisPoints: current.nonTopShareBasisPoints,
      tierHistogram: current.tierHistogram,
      precisionStratumHistogram: current.precisionStratumHistogram,
      topCauseHistogram: current.topCauseHistogram,
      passed: true,
    },
    null,
    2,
  ),
);
