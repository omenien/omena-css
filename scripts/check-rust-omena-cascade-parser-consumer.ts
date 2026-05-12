import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const semanticManifest = read("rust/crates/omena-semantic/Cargo.toml");
assert.ok(
  semanticManifest.includes("omena-cascade"),
  "omena-semantic must depend on omena-cascade for cascade ranking",
);
assert.ok(
  semanticManifest.includes("omena-parser"),
  "omena-semantic must consume omena-parser facts before cascade ranking",
);
assert.ok(
  !semanticManifest.includes("engine-style-parser"),
  "omena-semantic must not route cascade ranking through engine-style-parser",
);

const semanticLib = read("rust/crates/omena-semantic/src/lib.rs");
assert.ok(
  semanticLib.includes("let facts = collect_style_facts(style_source, dialect);"),
  "semantic boundary must collect omena-parser style facts from source",
);
assert.ok(
  semanticLib.includes(
    "let design_token_semantics = summarize_design_token_semantics(&parser_facts, &semantic_facts);",
  ),
  "semantic boundary must feed parser facts into design-token cascade semantics",
);

const designTokens = read("rust/crates/omena-semantic/src/design_tokens.rs");
assert.ok(
  designTokens.includes("use omena_cascade::{") &&
    designTokens.includes("select_cascade_winner"),
  "design token semantics must call omena-cascade winner selection",
);
assert.ok(
  countOccurrences(designTokens, "select_cascade_winner(") >= 2,
  "design token semantics must rank both same-file and workspace candidate sets",
);
assert.ok(
  designTokens.includes("source_order_cascade_ranking_ready") &&
    designTokens.includes("workspace_cascade_candidate_signal_ready"),
  "design token cascade surface must expose same-file and workspace readiness signals",
);

const queryStyle = read("rust/crates/omena-query/src/style.rs");
assert.ok(
  queryStyle.includes("read_omena_query_cascade_at_position") &&
    queryStyle.includes('cascade_engine: "omena-cascade"'),
  "omena-query must expose the omena-cascade-backed read-cascade-at-position surface",
);

const packageJson = read("package.json");
assert.ok(
  packageJson.includes('"check:rust-omena-cascade-boundary"') &&
    packageJson.includes("rust/omena-cascade/parser-consumer"),
  "rust/omena-cascade/boundary must include parser-consumer integration",
);

process.stdout.write(
  "validated omena-cascade parser consumer: semanticParserFacts=true sameFileRanking=true workspaceRanking=true queryReadSurface=true\n",
);

function read(filePath: string): string {
  return readFileSync(filePath, "utf8");
}

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
