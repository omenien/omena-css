import { strict as assert } from "node:assert";
import { readdirSync, readFileSync } from "node:fs";

const lspManifest = read("rust/crates/omena-lsp-server/Cargo.toml");
assert.ok(
  lspManifest.includes("omena-query"),
  "omena-lsp-server must consume parser-backed facts through omena-query",
);
assert.ok(
  !lspManifest.includes("engine-style-parser"),
  "omena-lsp-server must not depend on engine-style-parser",
);
assert.ok(
  !lspManifest.includes("omena-bridge"),
  "omena-lsp-server must not bypass omena-query with a direct omena-bridge dependency",
);
assert.ok(
  !lspManifest.includes("omena-parser"),
  "omena-lsp-server must not bypass omena-query with a direct omena-parser dependency",
);

const queryManifest = read("rust/crates/omena-query/Cargo.toml");
assert.ok(
  queryManifest.includes("omena-parser"),
  "omena-query must own the parser-facing dependency for LSP consumers",
);
assert.ok(
  queryManifest.includes("omena-bridge"),
  "omena-query must own the bridge-facing dependency for LSP consumers",
);
assert.ok(
  !queryManifest.includes("engine-style-parser"),
  "omena-query must not expose legacy parser coupling to LSP consumers",
);

const lspBoundary = read("rust/crates/omena-lsp-server/src/boundary.rs");
assert.ok(
  lspBoundary.includes('candidate_owner: "omena-query/sourceSyntaxIndex"') &&
    lspBoundary.includes('style_definition_owner: "omena-query/styleHoverCandidates"'),
  "LSP source provider adapter must route source syntax and style hover data through omena-query",
);
assert.ok(
  lspBoundary.includes("consumeQueryStyleHoverCandidates") &&
    lspBoundary.includes("consumeQuerySassModuleSources"),
  "LSP request policy must consume parser-backed query style candidates and Sass module sources",
);
assert.ok(
  lspBoundary.includes("query_reuse: rust_query_reuse_contract()"),
  "LSP boundary must delegate query reuse policy to the Rust query reuse contract",
);

const lspQueryReuse = read("rust/crates/omena-lsp-server/src/query_reuse.rs");
assert.ok(
  lspQueryReuse.includes("cached_surfaces: vec![") &&
    lspQueryReuse.includes('"styleHoverCandidates"') &&
    lspQueryReuse.includes('"sourceSyntaxIndex"'),
  "LSP query reuse must cache parser-backed query surfaces",
);

const queryBoundary = read("rust/crates/omena-query/src/boundary.rs");
assert.ok(
  queryBoundary.includes('style_document_summary_source: "omena-parser.style-facts"') &&
    queryBoundary.includes('output_product: "omena-parser.style-facts"'),
  "omena-query boundary must advertise omena-parser style facts as the style document source",
);

const queryStyle = read("rust/crates/omena-query/src/style.rs");
const queryParserFacade = read("rust/crates/omena-query/src/style/parser_facade.rs");
assert.ok(
  queryParserFacade.includes("collect_style_facts(style_source, dialect)") &&
    queryParserFacade.includes('product: "omena-query.omena-parser-style-facts"'),
  "omena-query parser facade must collect style facts from omena-parser",
);
assert.ok(
  queryStyle.includes('product: "omena-query.style-hover-candidates"') &&
    queryStyle.includes("collect_style_selector_hover_candidates_from_omena_parser_facts") &&
    queryStyle.includes("collect_custom_property_hover_candidates_from_omena_parser_facts"),
  "omena-query hover candidates must be derived from omena-parser facts",
);
const querySourcePaths = [
  "rust/crates/omena-query/src/style.rs",
  "rust/crates/omena-query/src/style/stylesheet_evaluation.rs",
  "rust/crates/omena-query/src/style/transform.rs",
  ...readdirSync("rust/crates/omena-query/src/style/diagnostics")
    .filter((entry) => entry.endsWith(".rs"))
    .map((entry) => `rust/crates/omena-query/src/style/diagnostics/${entry}`),
] as const;

for (const querySourcePath of querySourcePaths) {
  const querySource = read(querySourcePath);
  assert.ok(
    !querySource.includes("collect_style_facts("),
    `${querySourcePath} must collect parser facts through the query parser facade`,
  );
  assert.ok(
    !/\b(?:lex|parse)\(/.test(querySource),
    `${querySourcePath} must lex/parse through the query parser facade`,
  );
}

const lspStyleProviderParity = read("scripts/check-rust-omena-lsp-server-style-provider-parity.ts");
assert.ok(
  lspStyleProviderParity.includes("styleHoverCandidatesRequest") &&
    lspStyleProviderParity.includes(
      'response.result.product, "omena-lsp-server.style-hover-candidates"',
    ),
  "LSP provider parity must exercise the parser-backed style hover candidate request",
);

const packageJson = read("package.json");
assert.ok(
  packageJson.includes('"check:rust-omena-lsp-server-boundary"') &&
    packageJson.includes("rust/omena-lsp-server/parser-consumer"),
  "rust/omena-lsp-server/boundary must include parser-consumer integration",
);
assert.ok(
  packageJson.includes('"check:rust-omena-lsp-server-lane"') &&
    packageJson.includes("rust/omena-lsp-server/provider-parity") &&
    packageJson.includes("rust/omena-lsp-server/runtime-loop"),
  "Rust LSP lane must keep provider parity and runtime loop canary checks",
);

process.stdout.write(
  "validated omena-lsp-server parser consumer: queryFacade=true parserFacts=true providerParity=true runtimeLane=true\n",
);

function read(filePath: string): string {
  return readFileSync(filePath, "utf8");
}
