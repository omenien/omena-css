import { strict as assert } from "node:assert";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

interface ReadViewFunction {
  readonly file: string;
  readonly name: string;
  readonly signature: string;
  body: string;
}

const repoRoot = process.cwd();
const sourceRoot = path.join(repoRoot, "rust/crates/omena-lsp-server/src");
const injectDirectShellAccess = process.argv.includes("--inject-direct-shell-access");
const sources = new Map(
  rustFiles(sourceRoot).map((absolutePath) => [
    path.relative(sourceRoot, absolutePath).replaceAll(path.sep, "/"),
    readFileSync(absolutePath, "utf8"),
  ]),
);
const stateSource = requiredSource("state.rs");
const messageLoopSource = requiredSource("message_loop.rs");

const copiedFields = copiedQuerySnapshotFields(stateSource);
const viewFields = queryReadViewFields(stateSource);
assert.deepEqual(
  viewFields,
  copiedFields,
  "LspQueryReadView storage methods must exactly match the query snapshot copied-field partition",
);

const shellFields = lspShellStateFields(stateSource);
const loopOwnedFields = shellFields.filter((field) => !copiedFields.includes(field));
assert.ok(
  loopOwnedFields.length > 0,
  "query snapshot must leave loop-owned fields outside the view",
);

const readViewFunctions = [...sources].flatMap(([file, source]) =>
  readViewFunctionsInSource(file, source),
);
assert.ok(readViewFunctions.length > 0, "query read view consumer scan must be non-empty");

const scannedModules = [...new Set(readViewFunctions.map(({ file }) => file))].toSorted();
assert.deepEqual(
  scannedModules,
  [
    "color_provider.rs",
    "deferred_notification.rs",
    "disk_cache.rs",
    "document_links.rs",
    "external_sif_symbols.rs",
    "foreign_style_identity.rs",
    "lib.rs",
    "message_loop.rs",
    "open_document_inputs.rs",
    "parallel_style_wave.rs",
    "provider_tier_feedback.rs",
    "query_reuse.rs",
    "source_domain_hover.rs",
    "source_occurrence_cache.rs",
    "source_selector_provider.rs",
    "style_diagnostics.rs",
    "style_diagnostics_snapshot.rs",
    "style_symbol_monikers.rs",
    "style_symbol_occurrence_cache.rs",
    "style_symbol_provider.rs",
    "workspace_occurrences.rs",
    "workspace_resolution.rs",
    "workspace_symbols.rs",
  ],
  "the committed worker read-view module census must track the scan-derived set",
);

if (injectDirectShellAccess) {
  readViewFunctions[0].body += "\nlet _ = state.shutdown_requested;\n";
}

for (const readViewFunction of readViewFunctions) {
  assert.ok(
    readViewFunction.signature.includes("&dyn LspQueryReadView"),
    `${readViewFunction.file}:${readViewFunction.name} must accept the query read view`,
  );
  for (const field of loopOwnedFields) {
    assert.ok(
      !new RegExp(`\\b(?:state|view)\\s*\\.\\s*${escapeRegex(field)}\\b`).test(
        readViewFunction.body,
      ),
      `${readViewFunction.file}:${readViewFunction.name} reads loop-owned field ${field}`,
    );
  }
}

for (const [file, source] of sources) {
  assert.ok(
    !source.includes("dispatch.snapshot.shell_state") && !source.includes("snapshot.shell_state()"),
    `${file} must not recover LspShellState from a dispatched query snapshot`,
  );
}

const dispatchBody = functionBody(messageLoopSource, "resolve_dispatched_query_response");
assert.match(
  dispatchBody,
  /let state:\s*&dyn LspQueryReadView\s*=\s*&dispatch\.snapshot;/,
  "the dispatched query resolver must enter through LspQueryReadView",
);
const dispatchedEntryFunctions = [
  ...new Set(
    [...dispatchBody.matchAll(/(?:crate::[a-z_]+::)?(resolve_lsp_[a-z_]+)\(state\b/g)].map(
      (match) => match[1],
    ),
  ),
].toSorted();
assert.deepEqual(
  dispatchedEntryFunctions,
  [
    "resolve_lsp_code_lens",
    "resolve_lsp_definition",
    "resolve_lsp_document_color",
    "resolve_lsp_document_links",
    "resolve_lsp_hover",
    "resolve_lsp_workspace_symbols",
  ],
  "the dispatched provider entry census must stay explicit and scan-derived",
);
for (const entryFunction of dispatchedEntryFunctions) {
  assert.ok(
    readViewFunctions.some(({ name }) => name === entryFunction),
    `${entryFunction} must resolve to a function compiled against LspQueryReadView`,
  );
}

console.log(
  [
    "validated omena-lsp query read boundary",
    `copiedFields=${copiedFields.length}`,
    `loopOwnedFields=${loopOwnedFields.length}`,
    `modules=${scannedModules.length}`,
    `functions=${readViewFunctions.length}`,
    `dispatchEntries=${dispatchedEntryFunctions.length}`,
  ].join(" "),
);

function rustFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) return rustFiles(absolutePath);
    if (entry.isFile() && entry.name.endsWith(".rs") && statSync(absolutePath).isFile()) {
      return [absolutePath];
    }
    return [];
  });
}

function requiredSource(file: string): string {
  const source = sources.get(file);
  assert.ok(source, `missing Rust source ${file}`);
  return source;
}

function copiedQuerySnapshotFields(source: string): string[] {
  const functionSource = functionBody(source, "query_snapshot");
  const marker = "state: LspShellState";
  const markerIndex = functionSource.indexOf(marker);
  assert.notEqual(markerIndex, -1, "query_snapshot must retain its existing snapshot carrier");
  const openingBrace = functionSource.indexOf("{", markerIndex + marker.length);
  const block = rustBlock(functionSource, openingBrace);
  return [...block.matchAll(/^\s*([a-z_][a-z0-9_]*)\s*:/gm)]
    .map((match) => match[1])
    .filter((field) => field !== "state")
    .toSorted();
}

function queryReadViewFields(source: string): string[] {
  const marker = "pub trait LspQueryReadView";
  const markerIndex = source.indexOf(marker);
  assert.notEqual(markerIndex, -1, "LspQueryReadView trait must exist");
  const openingBrace = source.indexOf("{", markerIndex + marker.length);
  return [...rustBlock(source, openingBrace).matchAll(/\bfn query_([a-z0-9_]+)\s*\(/g)]
    .map((match) => match[1])
    .toSorted();
}

function lspShellStateFields(source: string): string[] {
  const marker = "pub struct LspShellState";
  const markerIndex = source.indexOf(marker);
  assert.notEqual(markerIndex, -1, "LspShellState must exist");
  const openingBrace = source.indexOf("{", markerIndex + marker.length);
  return [
    ...rustBlock(source, openingBrace).matchAll(/^\s*pub(?:\(crate\))?\s+([a-z_][a-z0-9_]*)\s*:/gm),
  ]
    .map((match) => match[1])
    .toSorted();
}

function readViewFunctionsInSource(file: string, source: string): ReadViewFunction[] {
  const functions: ReadViewFunction[] = [];
  const seen = new Set<number>();
  let cursor = 0;
  while ((cursor = source.indexOf("&dyn LspQueryReadView", cursor)) !== -1) {
    const fnIndex = source.lastIndexOf("fn ", cursor);
    const openingBrace = source.indexOf("{", cursor);
    if (fnIndex !== -1 && openingBrace !== -1 && !seen.has(fnIndex)) {
      const signature = source.slice(fnIndex, openingBrace);
      const name = /^fn\s+([a-z0-9_]+)/.exec(signature)?.[1];
      if (name) {
        functions.push({ file, name, signature, body: rustBlock(source, openingBrace) });
        seen.add(fnIndex);
      }
    }
    cursor += "&dyn LspQueryReadView".length;
  }
  return functions;
}

function functionBody(source: string, name: string): string {
  const match = new RegExp(`\\bfn\\s+${escapeRegex(name)}\\b`).exec(source);
  assert.ok(match, `missing function ${name}`);
  const openingBrace = source.indexOf("{", match.index + match[0].length);
  assert.notEqual(openingBrace, -1, `missing body for ${name}`);
  return rustBlock(source, openingBrace);
}

function rustBlock(source: string, openingBrace: number): string {
  assert.equal(source[openingBrace], "{", "Rust block must start at an opening brace");
  let depth = 0;
  let inString = false;
  let inLineComment = false;
  let blockCommentDepth = 0;
  let escaped = false;
  for (let index = openingBrace; index < source.length; index += 1) {
    const current = source[index];
    const next = source[index + 1];
    if (inLineComment) {
      if (current === "\n") inLineComment = false;
      continue;
    }
    if (blockCommentDepth > 0) {
      if (current === "/" && next === "*") {
        blockCommentDepth += 1;
        index += 1;
      } else if (current === "*" && next === "/") {
        blockCommentDepth -= 1;
        index += 1;
      }
      continue;
    }
    if (inString) {
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === '"') inString = false;
      continue;
    }
    if (current === "/" && next === "/") {
      inLineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      blockCommentDepth = 1;
      index += 1;
      continue;
    }
    if (current === '"') {
      inString = true;
      continue;
    }
    if (current === "{") depth += 1;
    if (current === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(openingBrace, index + 1);
    }
  }
  throw new Error("unterminated Rust block");
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
