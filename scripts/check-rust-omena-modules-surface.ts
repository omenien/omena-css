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
  "facts_from_cst(style_source, parsed)",
  "collect_icss_export_values_from_cst(style_source, parsed)",
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
assertModulesEmitUsesTransactionAuthority(cliSource);

const transactionBypassMutation = replaceRequired(
  cliSource,
  "transaction.commit()",
  "write_module_artifact()",
  "modules transaction commit",
);
assert.throws(
  () => assertModulesEmitUsesTransactionAuthority(transactionBypassMutation),
  /modules emit must commit its transaction authority/u,
  "modules emit bypassing WorkspaceEditTransaction must remain RED",
);

const transactionPreservingControl = replaceRequired(
  cliSource,
  "WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::EvidenceRequired)",
  `WorkspaceEditTransaction::new(
            None,
            WorkspaceEditSafetyClassV0::EvidenceRequired,
        )`,
  "modules transaction constructor",
);
assert.doesNotThrow(
  () => assertModulesEmitUsesTransactionAuthority(transactionPreservingControl),
  "equivalent transaction formatting must remain GREEN",
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
      moduleEmitTransactionAuthority: true,
      moduleEmitTransactionBypassMutation: "red",
      moduleEmitTransactionPreservingControl: "green",
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function assertModulesEmitUsesTransactionAuthority(source: string): void {
  const body = functionRegion(source, "apply_or_check_module_artifacts", "compile_include_globs");
  assert.match(
    body,
    /WorkspaceEditTransaction::new\(\s*None,\s*WorkspaceEditSafetyClassV0::EvidenceRequired,?\s*\)/u,
    "modules emit must construct WorkspaceEditTransaction with its evidence-required safety class",
  );
  assert.match(
    body,
    /transaction\s*=\s*transaction\.expect\(expected\)\.edit\(/u,
    "modules emit must stage artifacts through its transaction authority",
  );
  assert.match(
    body,
    /transaction\.commit\(\)/u,
    "modules emit must commit its transaction authority",
  );
  assert.doesNotMatch(
    body,
    /\bwrite_module_artifact\s*\(/u,
    "modules emit must not bypass WorkspaceEditTransaction through the retired artifact writer",
  );
}

function functionRegion(source: string, name: string, nextName: string): string {
  const startMarker = `fn ${name}(`;
  const endMarker = `\nfn ${nextName}(`;
  const start = source.indexOf(startMarker);
  assert.notEqual(start, -1, `missing function: ${name}`);
  const end = source.indexOf(endMarker, start);
  assert.notEqual(end, -1, `missing function boundary after ${name}: ${nextName}`);
  return source.slice(start, end);
}

function replaceRequired(
  source: string,
  needle: string,
  replacement: string,
  label: string,
): string {
  assert.equal(source.split(needle).length, 2, `${label} must appear exactly once`);
  return source.replace(needle, replacement);
}
