import { strict as assert } from "node:assert";
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
assert.match(source, /assert!\(document\.style_candidates\.is_empty\(\)\)/);
assert.match(source, /OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT\.load/);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-lsp-server.style-hover-index-policy-check",
      maximumSourceBytes: 65_536,
      oversizedFixtureMinimumBytes: 1_048_576,
      policy: "skip-hover-index-and-emit-typed-log",
      skipCounter: "OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT",
      embeddedStyleCovered: true,
    },
    null,
    2,
  )}\n`,
);

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
