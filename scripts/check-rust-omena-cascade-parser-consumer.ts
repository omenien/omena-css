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
  semanticLib.includes("let parsed = parse(style_source, dialect);") &&
    semanticLib.includes("let facts = facts_from_cst(style_source, &parsed);"),
  "semantic boundary must derive omena-parser style facts from a single CST parse",
);
assert.ok(
  semanticLib.includes(
    "let design_token_semantics = summarize_design_token_semantics(&parser_facts, &semantic_facts);",
  ),
  "semantic boundary must feed parser facts into design-token cascade semantics",
);

const designTokens = read("rust/crates/omena-semantic/src/design_tokens.rs");
const cascadeRanking = read("rust/crates/omena-cascade/src/ranking.rs");
assert.ok(
  designTokens.includes("use omena_cascade::{") &&
    designTokens.includes("select_open_world_cascade_winner"),
  "design token semantics must call omena-cascade open-world winner selection",
);
assert.equal(
  countOccurrences(cascadeRanking, "module_rank.cmp("),
  1,
  "omena-cascade must contain one open-world module-rank comparison",
);
assert.equal(
  countOccurrences(designTokens, "ModuleRank::ZERO"),
  0,
  "design-token cascade keys must not manufacture a zero module rank",
);
assert.equal(
  countOccurrences(designTokens, "select_open_world_cascade_winner("),
  1,
  "design token semantics must use one shared open-world winner selection",
);
assert.ok(
  designTokens.includes(".map(DesignTokenCandidateDeclaration::Local)") &&
    designTokens.includes(".map(DesignTokenCandidateDeclaration::Workspace)") &&
    !designTokens.includes("local_winner.or(workspace_winner)"),
  "design token semantics must rank same-file and workspace candidates in one domain",
);
assert.ok(
  designTokens.includes("source_order_cascade_ranking_ready") &&
    designTokens.includes("workspace_cascade_candidate_signal_ready"),
  "design token cascade surface must expose same-file and workspace readiness signals",
);

const queryStyle = [
  read("rust/crates/omena-query/src/style.rs"),
  read("rust/crates/omena-query/src/style/cascade_position.rs"),
].join("\n");
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
  "validated omena-cascade parser consumer: semanticParserFacts=true unifiedCandidateDomain=true openWorldSelection=true queryReadSurface=true\n",
);

function read(filePath: string): string {
  return readFileSync(filePath, "utf8");
}

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
