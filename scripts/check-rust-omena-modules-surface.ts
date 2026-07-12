import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const interfaceSource = read("rust/crates/omena-query/src/style/module_interface.rs");
const queryFacadeSource = read("rust/crates/omena-query/src/style.rs");
const parserFacadeSource = read("rust/crates/omena-query/src/style/parser_facade.rs");
const cliSource = read("rust/crates/omena-cli/src/modules.rs");
const dispatchSource = read("rust/crates/omena-cli/src/dispatch.rs");
const configSource = read("rust/crates/omena-cli/src/config/schema.rs");

for (const forbidden of ["omena_parser", "parse_omena", "tokenize", "Lexer", "Parser"]) {
  assert.equal(
    interfaceSource.includes(forbidden),
    false,
    `module interface VIEW must not depend directly on parser machinery: ${forbidden}`,
  );
}

for (const forbidden of ["Mutex<", "RwLock<", "RefCell<", "OnceLock<", "CacheV0", "StoreV0"]) {
  assert.equal(
    interfaceSource.includes(forbidden),
    false,
    `module interface VIEW must not own a parallel cache or store: ${forbidden}`,
  );
}

assert.ok(
  queryFacadeSource.includes("summarize_omena_query_css_modules_interface_bundle("),
  "omena-query must expose the canonical module interface VIEW",
);
assert.ok(
  queryFacadeSource.includes("collect_omena_query_style_facts_with_icss_values_raw("),
  "the interface VIEW must collect parser facts and ICSS values from one parsed CST",
);
for (const required of [
  "parse(style_source, dialect)",
  "facts_from_cst(style_source, &parsed)",
  "collect_icss_export_values_from_cst(style_source, &parsed)",
]) {
  assert.ok(
    parserFacadeSource.includes(required),
    `the parser facade must share one CST across facts and ICSS values: ${required}`,
  );
}
assert.ok(
  interfaceSource.includes("OmenaQueryModuleIdV0::new("),
  "module interfaces must re-key the existing module identity",
);
assert.ok(
  interfaceSource.includes("summarize_omena_query_css_modules_interface_summary_view("),
  "module interface counts must be projected through the existing summary plane",
);
assert.ok(
  cliSource.includes("render_omena_query_css_modules_interface_json("),
  "the CLI must consume the query-owned interface renderer",
);
assert.ok(
  cliSource.includes("write_module_artifact("),
  "modules emit must use its classified artifact writer",
);
assert.ok(
  dispatchSource.includes("Command::Modules { command } => modules_command(command)"),
  "the modules product verb must route directly to its implementation",
);
for (const field of ["include", "declaration_dir", "interface_file"]) {
  assert.ok(configSource.includes(`pub(crate) ${field}:`), `[modules].${field} must be typed`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-modules-surface",
      parserDirectReferences: 0,
      dedicatedStores: 0,
      productVerbWired: true,
      summaryPlaneView: true,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
