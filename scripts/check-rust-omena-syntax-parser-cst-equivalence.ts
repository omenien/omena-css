import { readdirSync, readFileSync } from "node:fs";
import { strict as assert } from "node:assert";

const syntaxSource = readFileSync("rust/crates/omena-syntax/src/lib.rs", "utf8");

// omena-parser's public surface is split across submodules (lib.rs re-export
// facade + parse.rs + summaries.rs + ...). Scan the whole crate src tree so the
// equivalence assertions track the symbols by content, not by file location —
// otherwise an internal module split silently drifts this gate stale.
const parserSrcDir = "rust/crates/omena-parser/src";
const parserSource = readdirSync(parserSrcDir, { recursive: true })
  .filter((entry): entry is string => typeof entry === "string" && entry.endsWith(".rs"))
  .map((entry) => readFileSync(`${parserSrcDir}/${entry}`, "utf8"))
  .join("\n");

assert.match(
  syntaxSource,
  /ready_surfaces:\s*vec!\[[\s\S]*"parserCstEquivalence"[\s\S]*\]/,
  "omena-syntax boundary must promote parserCstEquivalence to ready_surfaces",
);
assert.doesNotMatch(
  syntaxSource,
  /next_surfaces:\s*vec!\[[\s\S]*"parserCstEquivalence"[\s\S]*\]/,
  "omena-syntax boundary must not leave parserCstEquivalence in next_surfaces",
);

assert.match(
  parserSource,
  /use omena_syntax::SyntaxKind;/,
  "omena-parser must consume the shared omena-syntax SyntaxKind",
);
assert.match(
  parserSource,
  /pub fn syntax\(&self\) -> SyntaxNode<SyntaxKind>/,
  "ParseResult must expose cstree nodes typed by the shared SyntaxKind",
);
assert.match(
  parserSource,
  /pub struct ParserCstEquivalenceSummaryV0/,
  "omena-parser must expose a runtime CST equivalence summary",
);
assert.match(
  parserSource,
  /pub fn summarize_parser_cst_equivalence\(/,
  "omena-parser must expose the parser CST equivalence entrypoint",
);
assert.match(
  parserSource,
  /"parserUsesOmenaSyntaxKind"/,
  "parser CST equivalence summary must report shared SyntaxKind consumption",
);
assert.match(
  parserSource,
  /"typedCstWrapperEquivalence"/,
  "parser CST equivalence summary must report typed wrapper equivalence",
);

process.stdout.write(
  "validated omena-syntax parser CST equivalence: syntaxReady=true parserRuntimeSummary=true typedWrappers=true\n",
);
