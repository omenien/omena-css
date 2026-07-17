import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  WEBREF_GRAMMAR_SNAPSHOT,
  type WebrefGrammarEntry,
  type WebrefGrammarSnapshot,
} from "./webref-grammar-extract";

export const PROPERTY_METADATA_OVERRIDES_PATH =
  "rust/crates/omena-spec-audit/data/property-metadata-overrides.json";
export const PROPERTY_METADATA_ADJUDICATION_PATH =
  "rust/crates/omena-spec-audit/data/property-metadata-adjudication.json";
const LEGACY_PROPERTY_METADATA_BASELINE = "98bddcd64eef89c5d5bbb63bf64a3c55ee0c35a6";

export interface DerivedPropertyMetadataRow {
  readonly propertyId: string;
  readonly canonicalName: string;
  readonly href: string;
  readonly syntax: string | null;
  readonly upstreamInherited: string | null;
  readonly upstreamInitial: string | null;
  readonly inherited: boolean | null;
  readonly initialValue: string | null;
  readonly appliesTo: string | null;
  readonly percentages: string | null;
  readonly computedValue: string | null;
  readonly animationType: string | null;
  readonly longhands: readonly string[];
  readonly legacyAliasOf: string | null;
  readonly boundaryClassification: "in-boundary" | "forward-tier" | "excluded-with-reason";
  readonly boundaryReason: string;
  readonly overrideReason: string | null;
}

interface PropertyMetadataOverride {
  readonly propertyId: string;
  readonly inherited?: boolean;
  readonly initialValue?: string;
  readonly reason: string;
}

interface PropertyMetadataOverrideFile {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.property-metadata-overrides";
  readonly reasonTaxonomy: readonly string[];
  readonly entries: readonly PropertyMetadataOverride[];
}

interface PropertyMetadataAdjudicationFile {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.property-metadata-adjudication";
  readonly sourceBaseline: string;
  readonly legacyRowCount: number;
  readonly entries: readonly {
    readonly propertyId: string;
    readonly decision: "upstream" | "override";
    readonly legacy: PropertyMetadataSemanticFields;
    readonly upstream: {
      readonly inherited: boolean | null;
      readonly initialValue: string | null;
    };
    readonly effective: PropertyMetadataSemanticFields;
  }[];
}

interface PropertyMetadataSemanticFields {
  readonly inherited: boolean | null;
  readonly initialValue: string | null;
}

const NON_EXECUTABLE_INITIAL_VALUES = new Set([
  "depends on user agent",
  "not defined for shorthand properties",
  "see individual properties",
]);

export function loadDerivedPropertyMetadataRows(
  repoRoot = process.cwd(),
): readonly DerivedPropertyMetadataRow[] {
  const snapshot = readJson<WebrefGrammarSnapshot>(path.join(repoRoot, WEBREF_GRAMMAR_SNAPSHOT));
  let overrides = readJson<PropertyMetadataOverrideFile>(
    path.join(repoRoot, PROPERTY_METADATA_OVERRIDES_PATH),
  );
  if (process.argv.includes("--inject-reasonless-override") && overrides.entries[0]) {
    overrides = {
      ...overrides,
      entries: [{ ...overrides.entries[0], reason: "" }, ...overrides.entries.slice(1)],
    };
  }
  const adjudication = readJson<PropertyMetadataAdjudicationFile>(
    path.join(repoRoot, PROPERTY_METADATA_ADJUDICATION_PATH),
  );
  validateSources(snapshot, overrides, adjudication);

  const propertyEntries = snapshot.categories.properties ?? [];
  const byName = new Map<string, WebrefGrammarEntry>();
  for (const entry of propertyEntries) {
    assert.ok(!byName.has(entry.name), `duplicate Webref property row: ${entry.name}`);
    assert.ok(entry.propertyMetadata, `property ${entry.name} lacks a metadata record`);
    byName.set(entry.name, entry);
  }
  const overrideByProperty = new Map<string, PropertyMetadataOverride>();
  for (const override of overrides.entries) {
    assert.ok(
      byName.has(override.propertyId),
      `override property is absent: ${override.propertyId}`,
    );
    assert.ok(
      !overrideByProperty.has(override.propertyId),
      `duplicate property metadata override: ${override.propertyId}`,
    );
    assert.ok(
      overrides.reasonTaxonomy.includes(override.reason),
      `${override.propertyId} reason is not registered`,
    );
    assert.ok(
      override.inherited !== undefined || override.initialValue !== undefined,
      `${override.propertyId} override must change at least one semantic field`,
    );
    overrideByProperty.set(override.propertyId, override);
  }

  const rows = [...byName.values()]
    .map((entry) => deriveRow(entry, overrideByProperty.get(entry.name)))
    .sort((left, right) => compareStrings(left.canonicalName, right.canonicalName));
  validateAdjudication(rows, overrideByProperty, adjudication);
  return rows;
}

function deriveRow(
  entry: WebrefGrammarEntry,
  override: PropertyMetadataOverride | undefined,
): DerivedPropertyMetadataRow {
  const metadata = entry.propertyMetadata;
  assert.ok(metadata, `property ${entry.name} lacks metadata`);
  const upstreamInherited = normalizeInherited(metadata.inherited);
  const upstreamInitial = executableInitialValue(metadata.initial);
  const inherited = override?.inherited ?? upstreamInherited;
  const initialValue = override?.initialValue ?? upstreamInitial;
  if (override?.inherited !== undefined) {
    assert.notEqual(
      override.inherited,
      upstreamInherited,
      `${entry.name} inherited override is redundant`,
    );
  }
  if (override?.initialValue !== undefined) {
    assert.notEqual(
      override.initialValue,
      upstreamInitial,
      `${entry.name} initial override is redundant`,
    );
  }
  return {
    propertyId: entry.name,
    canonicalName: entry.name,
    href: entry.href,
    syntax: entry.syntax,
    upstreamInherited: metadata.inherited,
    upstreamInitial: metadata.initial,
    inherited,
    initialValue,
    appliesTo: metadata.appliesTo,
    percentages: metadata.percentages,
    computedValue: metadata.computedValue,
    animationType: metadata.animationType,
    longhands: metadata.longhands ?? [],
    legacyAliasOf: metadata.legacyAliasOf,
    boundaryClassification: entry.boundary.classification,
    boundaryReason: entry.boundary.reason,
    overrideReason: override?.reason ?? null,
  };
}

function normalizeInherited(value: string | null): boolean | null {
  if (value?.toLowerCase() === "yes") return true;
  if (value?.toLowerCase() === "no") return false;
  return null;
}

function executableInitialValue(value: string | null): string | null {
  if (value === null || NON_EXECUTABLE_INITIAL_VALUES.has(value.toLowerCase())) return null;
  return value;
}

function validateSources(
  snapshot: WebrefGrammarSnapshot,
  overrides: PropertyMetadataOverrideFile,
  adjudication: PropertyMetadataAdjudicationFile,
): void {
  assert.equal(snapshot.schemaVersion, "1");
  assert.equal(snapshot.product, "omena-spec-audit.webref-grammar");
  assert.equal(overrides.schemaVersion, "0");
  assert.equal(overrides.product, "omena-spec-audit.property-metadata-overrides");
  assert.equal(new Set(overrides.reasonTaxonomy).size, overrides.reasonTaxonomy.length);
  assert.equal(adjudication.schemaVersion, "0");
  assert.equal(adjudication.product, "omena-spec-audit.property-metadata-adjudication");
  assert.equal(
    adjudication.sourceBaseline,
    LEGACY_PROPERTY_METADATA_BASELINE,
    "legacy property rows must remain tied to their recorded source revision",
  );
  assert.equal(adjudication.legacyRowCount, 29, "legacy property row count must remain explicit");
  assert.equal(
    adjudication.entries.length,
    adjudication.legacyRowCount,
    "every legacy property row must be adjudicated",
  );
}

function validateAdjudication(
  rows: readonly DerivedPropertyMetadataRow[],
  overrides: ReadonlyMap<string, PropertyMetadataOverride>,
  adjudication: PropertyMetadataAdjudicationFile,
): void {
  const rowNames = new Set(rows.map((row) => row.propertyId));
  const rowsByName = new Map(rows.map((row) => [row.propertyId, row]));
  const seen = new Set<string>();
  for (const entry of adjudication.entries) {
    assert.ok(
      rowNames.has(entry.propertyId),
      `adjudicated property is absent: ${entry.propertyId}`,
    );
    assert.ok(!seen.has(entry.propertyId), `duplicate adjudication: ${entry.propertyId}`);
    seen.add(entry.propertyId);
    const row = rowsByName.get(entry.propertyId);
    assert.ok(row, `adjudicated property is absent: ${entry.propertyId}`);
    const upstreamFields: PropertyMetadataSemanticFields = {
      inherited: normalizeInherited(row.upstreamInherited),
      initialValue: executableInitialValue(row.upstreamInitial),
    };
    assert.deepEqual(
      entry.upstream,
      { inherited: upstreamFields.inherited, initialValue: row.upstreamInitial },
      `${entry.propertyId} upstream evidence drifted`,
    );
    assert.deepEqual(
      entry.effective,
      { inherited: row.inherited, initialValue: row.initialValue },
      `${entry.propertyId} effective metadata drifted`,
    );
    assert.deepEqual(
      entry.legacy,
      entry.effective,
      `${entry.propertyId} no longer reproduces the legacy runtime row`,
    );
    const upstreamMatchesLegacy =
      upstreamFields.inherited === entry.legacy.inherited &&
      upstreamFields.initialValue === entry.legacy.initialValue;
    assert.equal(
      entry.decision,
      upstreamMatchesLegacy ? "upstream" : "override",
      `${entry.propertyId} decision does not describe the legacy/upstream delta`,
    );
    assert.equal(
      overrides.has(entry.propertyId),
      entry.decision === "override",
      `${entry.propertyId} adjudication disagrees with the override authority`,
    );
  }
  assert.equal(
    [...overrides.keys()].filter((property) => !seen.has(property)).length,
    0,
    "every compatibility override must be adjudicated",
  );
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}
