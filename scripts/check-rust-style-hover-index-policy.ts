import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
let source = readFileSync(
  path.join(repoRoot, "rust/crates/omena-lsp-server/src/query_reuse.rs"),
  "utf8",
);
if (process.argv.includes("--inject-remove-guard")) {
  source = source.replace(
    "source_bytes > STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0",
    "false && source_bytes > STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0",
  );
}

assert.match(
  source,
  /pub const STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0:\s*usize\s*=\s*64\s*\*\s*1024;/,
  "the hover-index policy must stay pinned to 64 KiB below the measured crossover",
);
const guardBody = extractFunctionBody(source, "style_hover_index_is_oversized");
assert.match(
  guardBody,
  /^\s*source_bytes\s*>\s*STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0\s*$/u,
  "oversized hover indexing guard was removed or weakened",
);
const refreshBody = extractFunctionBody(source, "refresh_document_reusable_indexes");
assert.match(refreshBody, /style_hover_index_is_oversized\(document\.text\.len\(\)\)/);
assert.match(refreshBody, /record_oversized_style_hover_index_skip/);
assert.match(refreshBody, /Vec::new\(\)/);
const embeddedBody = extractFunctionBody(source, "collect_vue_embedded_module_style_indexes");
assert.match(embeddedBody, /style_hover_index_is_oversized\(embedded\.style_source\.len\(\)\)/);
assert.match(embeddedBody, /return Some\(\(summary, Vec::new\(\)\)\)/);

for (const needle of [
  '"product": "omena-lsp-server.style-hover-index-policy"',
  '"outcome": "skipped"',
  '"reason": "source-byte-limit"',
  '"sourceBytes": source_bytes',
  '"maximumSourceBytes": STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0',
  '"oversizedSkipCount": skip_count',
]) {
  assert.ok(source.includes(needle), `typed hover-index skip log is missing ${needle}`);
}
assert.match(source, /fn one_megabyte_style_source_skips_hover_indexing_with_a_counted_policy/);
assert.match(source, /fn one_megabyte_vue_module_style_skips_hover_indexing_with_a_counted_policy/);
assert.match(source, /assert!\(document\.style_candidates\.is_empty\(\)\)/);
assert.match(source, /OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT\.load/);

const measurementReceipts = runMeasurementTests();
const measuredFixtureKinds = measurementReceipts.map((receipt) => receipt.fixtureKind).toSorted();
assert.deepEqual(measuredFixtureKinds, ["styleDocument", "vueEmbeddedStyleModule"]);
for (const receipt of measurementReceipts) {
  const measuredSourceBytes =
    receipt.fixtureKind === "vueEmbeddedStyleModule"
      ? receipt.embeddedSourceBytes
      : receipt.sourceBytes;
  assert.ok(measuredSourceBytes >= 1_048_576);
  assert.equal(receipt.maximumSourceBytes, 65_536);
  assert.equal(receipt.hoverIndexAttempted, false);
  assert.equal(receipt.oversizedSkipCount, 1);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-lsp-server.style-hover-index-policy-check",
      maximumSourceBytes: 65_536,
      oversizedFixtureMinimumBytes: 1_048_576,
      policy: "skip-hover-index-and-emit-typed-log",
      skipCounter: "OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT",
      measurementReceiptCount: measurementReceipts.length,
      measuredFixtureKinds,
      embeddedStyleCovered: measuredFixtureKinds.includes("vueEmbeddedStyleModule"),
    },
    null,
    2,
  )}\n`,
);

interface MeasurementReceipt {
  readonly schemaVersion: "0";
  readonly product: "omena-lsp-server.style-hover-index-measurement";
  readonly fixtureKind: "styleDocument" | "vueEmbeddedStyleModule";
  readonly sourceBytes?: number;
  readonly embeddedSourceBytes?: number;
  readonly maximumSourceBytes: number;
  readonly hoverIndexAttempted: false;
  readonly oversizedSkipCount: 1;
}

function runMeasurementTests(): readonly MeasurementReceipt[] {
  const run = spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-lsp-server",
      "oversized_style_hover_index_tests",
      "--lib",
      "--",
      "--nocapture",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  assert.equal(
    run.status,
    0,
    `style-hover index measurement tests failed\nstdout:\n${run.stdout}\nstderr:\n${run.stderr}`,
  );
  const receipts: MeasurementReceipt[] = [];
  for (const line of `${run.stdout}\n${run.stderr}`.split("\n")) {
    const start = line.indexOf("{");
    const end = line.lastIndexOf("}");
    if (start < 0 || end <= start) continue;
    try {
      const candidate = JSON.parse(line.slice(start, end + 1)) as Partial<MeasurementReceipt>;
      if (candidate.product === "omena-lsp-server.style-hover-index-measurement") {
        receipts.push(candidate as MeasurementReceipt);
      }
    } catch {
      // Cargo can place non-JSON test text around braces; only complete measurement
      // receipts are relevant to this derivation.
    }
  }
  assert.equal(receipts.length, 2, "both oversized style measurement receipts must execute");
  return receipts;
}

function extractFunctionBody(rustSource: string, functionName: string): string {
  const match = new RegExp(`\\bfn\\s+${functionName}\\b`).exec(rustSource);
  assert.ok(match, `missing function ${functionName}`);
  const open = rustSource.indexOf("{", match.index);
  assert.ok(open >= 0, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = open; index < rustSource.length; index += 1) {
    if (rustSource[index] === "{") depth += 1;
    if (rustSource[index] === "}") {
      depth -= 1;
      if (depth === 0) return rustSource.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated body for ${functionName}`);
}
