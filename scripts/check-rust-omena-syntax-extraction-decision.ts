import { strict as assert } from "node:assert";
import { readdirSync, readFileSync } from "node:fs";

const packageJson = readFileSync("package.json", "utf8");
const syntaxSource = readFileSync("rust/crates/omena-syntax/src/lib.rs", "utf8");

// omena-parser's public surface is split across submodules (lib.rs facade +
// parse.rs + summaries.rs + ...). Scan the whole crate src tree so these
// assertions track symbols by content, not by file location — and so the
// "no local SyntaxKind enum" guard also covers submodules, not just lib.rs.
const parserSrcDir = "rust/crates/omena-parser/src";
const parserSource = readdirSync(parserSrcDir, { recursive: true })
  .filter((entry): entry is string => typeof entry === "string" && entry.endsWith(".rs"))
  .map((entry) => readFileSync(`${parserSrcDir}/${entry}`, "utf8"))
  .join("\n");
const parserManifest = readFileSync("rust/crates/omena-parser/Cargo.toml", "utf8");
const syntaxReadme = readFileSync("rust/crates/omena-syntax/README.md", "utf8");

assert.ok(
  packageJson.includes("check:rust-omena-syntax-extraction-decision"),
  "package.json must expose the syntax extraction decision gate",
);
assert.ok(
  packageJson.includes("rust/omena-syntax/extraction-decision"),
  "the omena-syntax boundary must include the extraction decision gate",
);
assert.ok(
  syntaxSource.includes("syntax_kind_extraction_decision: keep `SyntaxKind` extracted"),
  "omena-syntax rustdoc must record the keep-extracted decision",
);
assert.ok(
  syntaxSource.includes('syntax_kind_owner_crate: "omena-syntax"'),
  "omena-syntax boundary summary must name omena-syntax as SyntaxKind owner",
);
assert.ok(
  syntaxSource.includes('parser_consumer_policy: "parserConsumesOmenaSyntaxKindNoLocalTaxonomy"'),
  "omena-syntax boundary summary must record the parser consumer policy",
);
assert.ok(
  /^\s*omena-syntax\s*=/m.test(parserManifest),
  "omena-parser must depend on omena-syntax for shared SyntaxKind ownership",
);
assert.ok(
  parserSource.includes("use omena_syntax::SyntaxKind;"),
  "omena-parser must import SyntaxKind from omena-syntax",
);
assert.ok(
  !/\benum\s+SyntaxKind\b/u.test(parserSource),
  "omena-parser must not re-declare a local SyntaxKind enum",
);
assert.ok(
  parserSource.includes('"parserUsesOmenaSyntaxKind"'),
  "parser CST equivalence summary must advertise omena-syntax consumption",
);
assert.ok(
  syntaxReadme.includes("must consume instead of inventing their own local node/token taxonomies"),
  "omena-syntax README must document the no-local-taxonomy consumer rule",
);

process.stdout.write(
  "validated omena-syntax extraction decision: owner=omena-syntax parserPolicy=parserConsumesOmenaSyntaxKindNoLocalTaxonomy\n",
);
