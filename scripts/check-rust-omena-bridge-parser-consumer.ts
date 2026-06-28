import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const bridgeManifest = read("rust/crates/omena-bridge/Cargo.toml");
assert.ok(
  bridgeManifest.includes("omena-parser"),
  "omena-bridge must depend on omena-parser for shared parser spans/contracts",
);
assert.ok(
  bridgeManifest.includes("omena-semantic"),
  "omena-bridge must consume parser-backed style semantics through omena-semantic",
);
assert.ok(
  !bridgeManifest.includes("engine-style-parser"),
  "omena-bridge must not depend on engine-style-parser in the product bridge lane",
);

const bridgeLib = read("rust/crates/omena-bridge/src/lib.rs");
assert.ok(
  countOccurrences(bridgeLib, "summarize_omena_parser_style_semantic_boundary_from_source") >= 2,
  "bridge from-source paths must assemble style graphs from omena-parser-backed semantic boundaries",
);
assert.ok(
  bridgeLib.includes("collect_omena_bridge_design_token_workspace_declarations_from_source") &&
    bridgeLib.includes(
      "collect_design_token_workspace_declarations(style_path, &boundary.parser_facts)",
    ),
  "bridge design-token workspace declarations must be fed by parser boundary facts",
);
assert.ok(
  bridgeLib.includes("styleSemanticGraphFromSource") &&
    bridgeLib.includes("omenaParserBackedStyleSemanticBoundaryFromSource"),
  "bridge public boundary must advertise the parser-backed from-source graph path",
);

const sourceSyntax = read("rust/crates/omena-bridge/src/source_syntax.rs");
assert.ok(
  sourceSyntax.includes("use omena_parser::ParserByteSpanV0;"),
  "source syntax facts must use the shared omena-parser byte-span contract",
);
assert.ok(
  !sourceSyntax.includes("engine_style_parser") && !sourceSyntax.includes("engine-style-parser"),
  "source syntax indexing must not reintroduce engine-style-parser coupling",
);
assert.ok(
  sourceSyntax.includes("SourceImportedStyleBindingV0") &&
    sourceSyntax.includes("target_style_uri"),
  "source syntax references must preserve target-aware CSS Module bindings for the bridge join",
);
assert.ok(
  sourceSyntax.includes("canonicalize_source_selector_references"),
  "source syntax references must dedupe generic and target-aware bridge candidates",
);

const sourceEvidence = read("rust/crates/omena-bridge/src/source_evidence.rs");
const promotionEvidence = read("rust/crates/omena-bridge/src/promotion_evidence.rs");
assert.ok(
  sourceEvidence.includes("pub use omena_semantic::") &&
    sourceEvidence.includes("omena_semantic::summarize_source_input_evidence(input)") &&
    !sourceEvidence.includes("pub struct SourceInputPromotionEvidenceSummaryV0"),
  "bridge source evidence must delegate to omena-semantic instead of duplicating evidence DTOs",
);
assert.ok(
  promotionEvidence.includes("pub use omena_semantic::") &&
    promotionEvidence.includes("omena_semantic::summarize_semantic_promotion_evidence(") &&
    promotionEvidence.includes(
      "omena_semantic::summarize_semantic_promotion_evidence_with_source_input(",
    ) &&
    !promotionEvidence.includes("pub struct SemanticPromotionEvidenceSummaryV0"),
  "bridge promotion evidence must delegate to omena-semantic instead of duplicating promotion DTOs",
);

const semanticLib = read("rust/crates/omena-semantic/src/lib.rs");
assert.ok(
  semanticLib.includes("let parsed = parse(style_source, dialect);") &&
    semanticLib.includes("let facts = facts_from_cst(style_source, &parsed);") &&
    semanticLib.includes("let cst = parsed.cst();") &&
    semanticLib.includes("summarize_omena_parser_contract_facts("),
  "bridge's delegated semantic boundary must derive omena-parser facts from the source CST",
);
assert.ok(
  semanticLib.includes("summarize_omena_parser_contract_facts") &&
    semanticLib.includes("summarize_omena_parser_semantic_facts"),
  "bridge's delegated semantic boundary must expose parser contract and semantic facts",
);

const packageJson = read("package.json");
assert.ok(
  packageJson.includes('"check:rust-omena-bridge-boundary"') &&
    packageJson.includes("rust/omena-bridge/parser-consumer"),
  "rust/omena-bridge/boundary must include parser-consumer integration",
);

process.stdout.write(
  "validated omena-bridge parser consumer: parserSpans=true parserBackedStyleGraph=true targetAwareSourceJoin=true semanticEvidenceDelegated=true\n",
);

function read(filePath: string): string {
  return readFileSync(filePath, "utf8");
}

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
