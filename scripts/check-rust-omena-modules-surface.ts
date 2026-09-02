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

const compositeFilesystemBypassMutation = replaceRequired(
  cliSource,
  "    for plan in plans {",
  '    for plan in plans {\n        let _ = fs::write(plan.path.as_path(), b"mutation control");',
  "modules artifact loop",
);
assert.throws(
  () => assertModulesEmitUsesTransactionAuthority(compositeFilesystemBypassMutation),
  /modules emit must not contain a bare filesystem write/u,
  "a bare filesystem write inside the modules emit loop must remain RED",
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
      moduleEmitCompositeFilesystemBypassMutation: "red",
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
  const body = functionRegion(source, "apply_or_check_module_artifacts");
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
  assert.doesNotMatch(
    maskRustCommentsAndLiterals(body),
    /\b(?:(?:std::)?fs::(?:write|copy|rename|hard_link|remove_file)|std::os::[a-z_]+::fs::symlink)\s*\(|\b(?:std::fs::)?(?:File|OpenOptions)::(?:create|create_new|options|new)\s*\(|\.write(?:_all)?\s*\(/gu,
    "modules emit must not contain a bare filesystem write",
  );
}

function functionRegion(source: string, name: string): string {
  const structural = maskRustCommentsAndLiterals(source);
  const match = new RegExp(`\\bfn\\s+${name}\\s*\\(`, "u").exec(structural);
  assert.ok(match?.index !== undefined, `missing function: ${name}`);
  const open = structural.indexOf("{", match.index);
  assert.notEqual(open, -1, `missing function body: ${name}`);
  return source.slice(match.index, matchingBrace(structural, open, `function ${name}`) + 1);
}

function matchingBrace(source: string, open: number, label: string): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  assert.fail(`${label} has an unterminated body`);
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n" && masked[index] !== "\r") masked[index] = " ";
    }
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      blank(index, end < 0 ? source.length : end);
      index = end < 0 ? source.length : end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else index += 1;
      }
      blank(start, index);
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (raw) {
      const start = index;
      const terminator = `"${raw[1] ?? ""}`;
      index += raw[0].length;
      const end = source.indexOf(terminator, index);
      index = end < 0 ? source.length : end + terminator.length;
      blank(start, index);
      continue;
    }
    if (source[index] === '"') {
      const start = index;
      index += 1;
      let escaped = false;
      while (index < source.length) {
        const current = source[index++]!;
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      blank(start, index);
      continue;
    }
    index += 1;
  }
  return masked.join("");
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
