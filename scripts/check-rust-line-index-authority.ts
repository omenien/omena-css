import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface LineIndexAuthorityBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-syntax.line-index-authority-baseline";
  readonly workspaceMemberManifestPaths: readonly string[];
}

interface DefinitionSite {
  readonly id: string;
  readonly relativePath: string;
  readonly functionName: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baselinePath = path.join(
  repoRoot,
  "rust/crates/omena-syntax/tests/snapshots/line-index-authority-baseline.json",
);
const publicApiSnapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
);
const writeBaseline = process.argv.includes("--write");
const injectedScanner = process.argv
  .find((argument) => argument.startsWith("--inject-scanner="))
  ?.slice("--inject-scanner=".length);
const injectWorkspaceMember = process.argv.includes("--inject-workspace-member");

const definitionSites: readonly DefinitionSite[] = [
  {
    id: "query-forward",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "parser_position_for_byte_offset",
  },
  {
    id: "query-reverse",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "byte_offset_for_parser_position",
  },
  {
    id: "semantic-forward",
    relativePath: "rust/crates/omena-semantic/src/lib.rs",
    functionName: "parser_position_for_byte_offset",
  },
  {
    id: "lsp-forward",
    relativePath: "rust/crates/omena-lsp-server/src/protocol.rs",
    functionName: "parser_position_for_byte_offset",
  },
  {
    id: "lsp-reverse",
    relativePath: "rust/crates/omena-lsp-server/src/protocol.rs",
    functionName: "byte_offset_for_parser_position",
  },
  {
    id: "cli-forward",
    relativePath: "rust/crates/omena-cli/src/text_edit.rs",
    functionName: "position_for_byte_offset",
  },
  {
    id: "cli-reverse",
    relativePath: "rust/crates/omena-cli/src/text_edit.rs",
    functionName: "byte_offset_for_position",
  },
  {
    id: "parser-forward",
    relativePath: "rust/crates/omena-parser/src/public_product.rs",
    functionName: "parser_range_for_byte_span",
  },
];

assert.equal(
  new Set(definitionSites.map((site) => site.id)).size,
  8,
  "the line-index census must enumerate all 5 forward and 3 reverse definition sites",
);
if (injectedScanner !== undefined) {
  assert.ok(
    definitionSites.some((site) => site.id === injectedScanner),
    `unknown scanner injection ${injectedScanner}`,
  );
}

const sourceByPath = new Map<string, string>();
for (const site of definitionSites) {
  if (!sourceByPath.has(site.relativePath)) {
    sourceByPath.set(
      site.relativePath,
      readFileSync(path.join(repoRoot, site.relativePath), "utf8"),
    );
  }
}

for (const site of definitionSites) {
  let body = extractFunctionBody(sourceByPath.get(site.relativePath) ?? "", site.functionName);
  if (site.id === injectedScanner) {
    body += "\nfor (_, value) in source.char_indices() { let _ = value; }";
  }
  assertNoIndependentSourceScan(site, body);
  assert.match(
    body,
    /(?:OmenaLineIndexV0|line_index|position_for_byte_offset_with_line_index|byte_offset_for_position_with_line_index)/,
    `${site.id} does not delegate to the shared line-index authority`,
  );
}

const syntaxIndexPath = "rust/crates/omena-syntax/src/line_index.rs";
const syntaxIndexSource = readFileSync(path.join(repoRoot, syntaxIndexPath), "utf8");
assert.equal(
  countMatches(syntaxIndexSource, /pub struct OmenaLineIndexV0\b/g),
  1,
  "OmenaLineIndexV0 must have exactly one definition",
);
assert.match(syntaxIndexSource, /line_starts:\s*Vec<u32>/);
assert.match(syntaxIndexSource, /\.partition_point\(/);
assert.match(syntaxIndexSource, /pub fn position_for_byte_offset\b/);
assert.match(syntaxIndexSource, /pub fn byte_offset_for_position\b/);

const parserSource = sourceByPath.get("rust/crates/omena-parser/src/public_product.rs") ?? "";
assert.match(
  parserSource,
  /type SourceLineIndex\s*=\s*OmenaLineIndexV0\s*;/,
  "omena-parser must delegate its prior-art SourceLineIndex name to OmenaLineIndexV0",
);

const querySource = sourceByPath.get("rust/crates/omena-query/src/style.rs") ?? "";
const hoverBody = extractFunctionBody(querySource, "summarize_omena_query_style_hover_candidates");
assert.equal(
  countMatches(hoverBody, /omena_query_line_index\s*\(/g),
  1,
  "the hover candidate pass must build one shared line index per source",
);
for (const collector of [
  "collect_style_selector_hover_candidates_from_omena_parser_facts",
  "collect_custom_property_hover_candidates_from_omena_parser_facts",
  "collect_sass_symbol_hover_candidates_from_omena_parser_facts",
  "collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts",
]) {
  const collectorBody = extractFunctionBody(querySource, collector);
  assert.doesNotMatch(
    collectorBody,
    /OmenaLineIndexV0::new|omena_query_line_index\s*\(/,
    `${collector} rebuilds the line index per candidate family`,
  );
  assert.match(collectorBody, /parser_range_for_byte_span_with_line_index\s*\(/);
}

const implementationSources = trackedRustSources();
if (!implementationSources.includes(syntaxIndexPath)) {
  implementationSources.push(syntaxIndexPath);
}
const implementationCount = implementationSources
  .map((relativePath) => readFileSync(path.join(repoRoot, relativePath), "utf8"))
  .reduce((count, source) => count + countMatches(source, /impl\s+OmenaLineIndexV0\b/g), 0);
assert.equal(implementationCount, 1, "line-index scanner implementation census must equal one");

const workspaceMemberManifestPaths = cargoWorkspaceMemberManifestPaths();
if (injectWorkspaceMember) {
  workspaceMemberManifestPaths.push("rust/crates/injected-line-index-member/Cargo.toml");
  workspaceMemberManifestPaths.sort();
}
const publicApi = cargoPublicApi();
assert.match(publicApi, /pub struct omena_syntax::OmenaLineIndexV0/);
assert.match(publicApi, /pub fn omena_syntax::OmenaLineIndexV0::position_for_byte_offset/);
assert.match(publicApi, /pub fn omena_syntax::OmenaLineIndexV0::byte_offset_for_position/);

if (writeBaseline) {
  mkdirSync(path.dirname(baselinePath), { recursive: true });
  const baseline: LineIndexAuthorityBaselineV0 = {
    schemaVersion: "0",
    product: "omena-syntax.line-index-authority-baseline",
    workspaceMemberManifestPaths,
  };
  writeFileSync(baselinePath, `${JSON.stringify(baseline, null, 2)}\n`);
  writeFileSync(publicApiSnapshotPath, publicApi);
} else {
  assert.ok(existsSync(baselinePath), "line-index authority baseline is missing; run with --write");
  assert.ok(
    existsSync(publicApiSnapshotPath),
    "omena-syntax public API snapshot is missing; run with --write",
  );
  const baseline = JSON.parse(readFileSync(baselinePath, "utf8")) as LineIndexAuthorityBaselineV0;
  assert.deepEqual(
    workspaceMemberManifestPaths,
    baseline.workspaceMemberManifestPaths,
    "Cargo workspace membership changed; line-index authority forbids adding a crate in this slice",
  );
  assert.equal(
    publicApi,
    normalizeOutput(readFileSync(publicApiSnapshotPath, "utf8")),
    "omena-syntax public API changed without an explicit line-index snapshot update",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-syntax.line-index-authority",
      forwardDefinitionSiteCount: 5,
      reverseDefinitionSiteCount: 3,
      implementationCount,
      hoverLineIndexBuildsPerSource: 1,
      workspaceMemberCount: workspaceMemberManifestPaths.length,
      publicHelper: "OmenaLineIndexV0",
    },
    null,
    2,
  )}\n`,
);

function assertNoIndependentSourceScan(site: DefinitionSite, body: string): void {
  const forbidden = [
    /\.char_indices\s*\(/,
    /\.bytes\s*\(\)\s*\.filter/,
    /\.rfind\s*\(\s*['"]\\n['"]\s*\)/,
    /\.encode_utf16\s*\(\)\s*\.count\s*\(/,
  ];
  for (const pattern of forbidden) {
    assert.doesNotMatch(body, pattern, `${site.id} restored an independent source scanner`);
  }
}

function extractFunctionBody(source: string, functionName: string): string {
  const signature = new RegExp(`\\bfn\\s+${functionName}\\b`);
  const match = signature.exec(source);
  assert.ok(match, `missing function ${functionName}`);
  const open = source.indexOf("{", match.index);
  assert.ok(open >= 0, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated body for ${functionName}`);
}

function trackedRustSources(): string[] {
  return execFileSync("git", ["ls-files", "rust/crates/*/src/*.rs", "rust/crates/*/src/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
    .trim()
    .split("\n")
    .filter(Boolean);
}

function cargoWorkspaceMemberManifestPaths(): string[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    ),
  ) as {
    readonly workspace_members: readonly string[];
    readonly packages: readonly { readonly id: string; readonly manifest_path: string }[];
  };
  const members = new Set(metadata.workspace_members);
  return metadata.packages
    .filter((entry) => members.has(entry.id))
    .map((entry) => path.relative(repoRoot, entry.manifest_path).replaceAll(path.sep, "/"))
    .sort();
}

function cargoPublicApi(): string {
  const output = execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-syntax",
      "-sss",
      "--color",
      "never",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  return normalizeOutput(output);
}

function normalizeOutput(value: string): string {
  return `${value.replaceAll("\r\n", "\n").trimEnd()}\n`;
}

function countMatches(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length;
}
