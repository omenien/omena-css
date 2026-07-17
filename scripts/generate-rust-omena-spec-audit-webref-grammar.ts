import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  GENERATOR_TOOL,
  WEBREF_GRAMMAR_SNAPSHOT,
  extractWebrefGrammarSnapshot,
  serializeWebrefGrammarSnapshot,
} from "./webref-grammar-extract";

// Vendor the `@webref/css` value-definition-syntax grammar snapshot from the
// installed pinned package. `--check` asserts the on-disk snapshot is current
// (idempotent regeneration); no flag rewrites it. Mirrors the WPT corpus
// generator discipline.

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const snapshotPath = path.join(repoRoot, WEBREF_GRAMMAR_SNAPSHOT);
const deltaReportRelativePath = "rust/crates/omena-spec-audit/data/webref-registry-delta.json";
const deltaReportPath = path.join(repoRoot, deltaReportRelativePath);

const snapshot = extractWebrefGrammarSnapshot(repoRoot);
const snapshotSource = serializeWebrefGrammarSnapshot(snapshot);
const targetFingerprint = sha256(snapshotSource);

if (checkOnly) {
  assert.equal(
    readFileSync(snapshotPath, "utf8"),
    snapshotSource,
    `${WEBREF_GRAMMAR_SNAPSHOT} is stale; run \`node --import tsx ./${GENERATOR_TOOL}\``,
  );
  const deltaReport = JSON.parse(readFileSync(deltaReportPath, "utf8")) as RegistryDeltaReport;
  if (process.argv.includes("--inject-stale-delta-report")) {
    deltaReport.target.fingerprint = "injected-stale-fingerprint";
  }
  assert.equal(deltaReport.schemaVersion, "0", "registry delta report schema drifted");
  assert.equal(deltaReport.product, "omena-spec-audit.webref-registry-delta");
  assert.equal(
    deltaReport.target.fingerprint,
    targetFingerprint,
    "registry snapshot changed without a reviewed added/changed/removed delta report",
  );
  assert.equal(deltaReport.target.version, snapshot.source.version);
  assert.equal(deltaReport.target.gitHead, snapshot.source.gitHead);
  assert.ok(deltaReport.humanReviewRequired, "registry deltas require human review");
  assert.ok(Array.isArray(deltaReport.added));
  assert.ok(Array.isArray(deltaReport.changed));
  assert.ok(Array.isArray(deltaReport.removed));
} else {
  const previousSource = readFileSync(snapshotPath, "utf8");
  if (previousSource !== snapshotSource) {
    const previous = JSON.parse(previousSource) as UnknownGrammarSnapshot;
    const deltaReport = buildDeltaReport(previous, snapshot, previousSource, snapshotSource);
    writeFileSync(deltaReportPath, `${JSON.stringify(deltaReport, null, 2)}\n`);
  }
  writeFileSync(snapshotPath, snapshotSource);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-spec-audit.webref-grammar-generator",
      mode: checkOnly ? "check" : "write",
      version: snapshot.source.version,
      gitHead: snapshot.source.gitHead,
      entryCount: snapshot.entryCount,
      categoryCounts: Object.fromEntries(
        Object.entries(snapshot.categories).map(([category, entries]) => [
          category,
          entries.length,
        ]),
      ),
      generatedFiles: [WEBREF_GRAMMAR_SNAPSHOT, deltaReportRelativePath],
    },
    null,
    2,
  )}\n`,
);

interface RegistryDeltaReport {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.webref-registry-delta";
  readonly humanReviewRequired: true;
  readonly source: RegistryDeltaEndpoint;
  readonly target: RegistryDeltaEndpoint;
  readonly added: readonly string[];
  readonly changed: readonly string[];
  readonly removed: readonly string[];
}

interface RegistryDeltaEndpoint {
  schemaVersion: string;
  version: string;
  gitHead: string;
  fingerprint: string;
  entryCount: number;
}

interface UnknownGrammarSnapshot {
  readonly schemaVersion?: string;
  readonly source?: { readonly version?: string; readonly gitHead?: string };
  readonly entryCount?: number;
  readonly categories?: Readonly<Record<string, readonly unknown[]>>;
}

function buildDeltaReport(
  previous: UnknownGrammarSnapshot,
  next: typeof snapshot,
  previousSource: string,
  nextSource: string,
): RegistryDeltaReport {
  const previousRows = flattenRows(previous.categories ?? {});
  const nextRows = flattenRows(next.categories);
  const previousByKey = new Map(previousRows.map((row) => [row.key, row.value]));
  const nextByKey = new Map(nextRows.map((row) => [row.key, row.value]));
  const added = [...nextByKey.keys()].filter((key) => !previousByKey.has(key));
  const removed = [...previousByKey.keys()].filter((key) => !nextByKey.has(key));
  const changed = [...nextByKey.keys()].filter(
    (key) =>
      previousByKey.has(key) &&
      JSON.stringify(previousByKey.get(key)) !== JSON.stringify(nextByKey.get(key)),
  );
  return {
    schemaVersion: "0",
    product: "omena-spec-audit.webref-registry-delta",
    humanReviewRequired: true,
    source: {
      schemaVersion: previous.schemaVersion ?? "unknown",
      version: previous.source?.version ?? "unknown",
      gitHead: previous.source?.gitHead ?? "unknown",
      fingerprint: sha256(previousSource),
      entryCount: previous.entryCount ?? previousRows.length,
    },
    target: {
      schemaVersion: next.schemaVersion,
      version: next.source.version,
      gitHead: next.source.gitHead,
      fingerprint: sha256(nextSource),
      entryCount: next.entryCount,
    },
    added: added.sort(),
    changed: changed.sort(),
    removed: removed.sort(),
  };
}

function flattenRows(categories: Readonly<Record<string, readonly unknown[]>>): Array<{
  key: string;
  value: unknown;
}> {
  const rows: Array<{ key: string; value: unknown }> = [];
  for (const [category, entries] of Object.entries(categories).sort()) {
    const occurrences = new Map<string, number>();
    for (const value of entries) {
      const entry = value as { name?: unknown; syntax?: unknown };
      const base = `${category}:${String(entry.name)}:${String(entry.syntax ?? "")}`;
      const occurrence = occurrences.get(base) ?? 0;
      occurrences.set(base, occurrence + 1);
      rows.push({ key: `${base}#${occurrence}`, value });
    }
  }
  return rows;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}
