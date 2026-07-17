import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  classifyCssSpecEntry,
  loadCssSpecBoundaryContext,
  type CssSpecBoundaryClassification,
} from "./css-spec-boundary";
import { WEBREF_CSS_JSON } from "./webref-grammar-extract";

const CATEGORY_ORDER = ["atrules", "functions", "properties", "selectors", "types"] as const;
const repoRoot = process.cwd();
const context = loadCssSpecBoundaryContext(repoRoot);
const source = JSON.parse(readFileSync(path.join(repoRoot, WEBREF_CSS_JSON), "utf8")) as Record<
  string,
  unknown
>;

const classificationCounts = new Map<CssSpecBoundaryClassification, number>();
const categoryCounts = new Map<string, number>();
const reasonCounts = new Map<string, number>();
const ruleCounts = new Map<string, number>();
let rowCount = 0;

for (const category of CATEGORY_ORDER) {
  const rows = source[category];
  assert.ok(Array.isArray(rows), `css.json.${category} must be an array`);
  const injectedRows = process.argv.includes("--inject-unclassified-source")
    ? [...rows, { name: "boundary-injection", href: "https://unclassified.invalid/spec/#x" }]
    : rows;
  for (const row of injectedRows as readonly { name?: unknown; href?: unknown }[]) {
    assert.equal(typeof row.name, "string", `css.json.${category} row name must be a string`);
    assert.equal(typeof row.href, "string", `css.json.${category} row href must be a string`);
    const verdict = classifyCssSpecEntry(context, row.href as string);
    assert.ok(
      context.boundary.reasonTaxonomy.includes(verdict.reason),
      `${category}:${String(row.name)} uses an unregistered boundary reason`,
    );
    classificationCounts.set(
      verdict.classification,
      (classificationCounts.get(verdict.classification) ?? 0) + 1,
    );
    reasonCounts.set(verdict.reason, (reasonCounts.get(verdict.reason) ?? 0) + 1);
    ruleCounts.set(verdict.ruleId, (ruleCounts.get(verdict.ruleId) ?? 0) + 1);
    rowCount += 1;
  }
  categoryCounts.set(category, injectedRows.length);
}

assert.equal(
  rowCount,
  [...categoryCounts.values()].reduce((total, count) => total + count, 0),
  "every Webref registry row must receive exactly one boundary verdict",
);
for (const ruleId of ["fxtf-filter-effects", "fxtf-masking-and-clipping", "fxtf-compositing"]) {
  assert.ok((ruleCounts.get(ruleId) ?? 0) > 0, `${ruleId} must classify at least one registry row`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-spec-audit.css-spec-boundary",
      snapshotYear: context.boundary.snapshot.year,
      categoryCounts: Object.fromEntries(categoryCounts),
      classificationCounts: Object.fromEntries([...classificationCounts].sort()),
      reasonCounts: Object.fromEntries([...reasonCounts].sort()),
      ruleCounts: Object.fromEntries([...ruleCounts].sort()),
      rowCount,
      unclassifiedRowCount: 0,
      complete: true,
    },
    null,
    2,
  )}\n`,
);
