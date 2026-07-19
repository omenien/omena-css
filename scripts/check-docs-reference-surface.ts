import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

type ProductVerbStatus = "stub" | "reserved-alias" | "wired";

interface ProductVerbRow {
  readonly verb: string;
  readonly status: ProductVerbStatus;
  readonly wiredBy: string | null;
}

interface ProductVerbManifest {
  readonly verbs: readonly ProductVerbRow[];
}

interface PersonaManifest {
  readonly consumption: {
    readonly syntax: string;
  };
  readonly presets: readonly {
    readonly id: string;
    readonly priority: number;
    readonly audience: string;
    readonly configPath: string;
    readonly verbs: readonly string[];
  }[];
}

interface LspBoundarySummary {
  readonly capabilities: Readonly<Record<string, unknown>>;
}

interface CommandRow {
  readonly variant: string;
  readonly command: string;
  readonly summary: string;
}

interface ConfigKeyRow {
  readonly key: string;
  readonly owner: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const writeMode = process.argv.includes("--write");
const generatedNotice = "<!-- Generated from product code. Do not edit by hand. -->";
const readmeLineBudget = 593;
const maximumUnwiredChecks = new Set([
  "check-rust-m6-dimensional-refinement.ts",
  "check-rust-m4-gamma-readiness.ts",
]);
const expectedUnwiredChecks = [
  "check-rust-m4-gamma-readiness.ts",
  "check-rust-m6-dimensional-refinement.ts",
] as const;

assert.ok(
  expectedUnwiredChecks.every((filename) => maximumUnwiredChecks.has(filename)),
  "the known unwired-check ledger may only shrink from its reviewed maximum",
);
assert.ok(expectedUnwiredChecks.length <= 2, "the unwired-check ledger may not grow");

verifyProductVerbManifest();
const productVerbs = readJson<ProductVerbManifest>("rust/crates/omena-cli/verb-census.json").verbs;
const commands = extractCommandRows(read("rust/crates/omena-cli/src/commands.rs"));
const productVerbNames = extractEnumVariants(
  read("rust/crates/omena-cli/src/product_verb.rs"),
  "ProductVerb",
).map(toKebabCase);
assert.deepEqual(
  productVerbs.map(({ verb }) => verb),
  productVerbNames,
  "product verb rendering must follow ProductVerb source order",
);
assert.deepEqual(
  commands
    .filter(({ command }) => productVerbNames.includes(command))
    .map(({ command }) => command),
  productVerbNames,
  "the Command enum must retain every product verb",
);

const personas = derivePersonas();
const configKeys = deriveConfigKeys(read("rust/crates/omena-cli/src/config/schema.rs"));
const lspCapabilities = flattenObject(readLspBoundary().capabilities);

const renderedFiles = new Map<string, string>([
  ["docs/reference/README.md", renderReferenceIndex()],
  ["docs/reference/cli.md", renderCliReference(productVerbs, commands)],
  ["docs/reference/personas.md", renderPersonaReference(personas)],
  ["docs/reference/configuration.md", renderConfigReference(configKeys)],
  ["docs/reference/lsp-capabilities.md", renderLspReference(lspCapabilities)],
]);

for (const [relativePath, content] of renderedFiles) {
  assertGeneratedFile(relativePath, content);
}

const cliReadmePath = "rust/crates/omena-cli/README.md";
const renderedCliReadme = renderCliReadme(read(cliReadmePath), productVerbs, commands);
assertGeneratedFile(cliReadmePath, renderedCliReadme);

verifyReadmeBudget();
verifyCheckScriptReachability();
verifyExecutableTomlExamples();

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.reference-surface",
      productVerbs: productVerbs.length,
      cliCommands: commands.length,
      personas: personas.length,
      configKeys: configKeys.length,
      lspCapabilityLeaves: lspCapabilities.length,
      generatedFiles: renderedFiles.size,
      readmeLineBudget,
      knownUnwiredChecks: expectedUnwiredChecks.length,
      mode: writeMode ? "write" : "check",
    },
    null,
    2,
  )}\n`,
);

function verifyProductVerbManifest(): void {
  const result = spawnSync(
    process.execPath,
    ["--import", "tsx", "./scripts/check-rust-omena-cli-verb-census.ts"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(
    result.status,
    0,
    ["product verb source census failed", result.stdout.trim(), result.stderr.trim()]
      .filter(Boolean)
      .join("\n"),
  );
}

function derivePersonas(): PersonaManifest["presets"] {
  const manifest = readJson<PersonaManifest>("rust/crates/omena-cli/persona-presets.json");
  const presetDirectory = path.join(repoRoot, "rust/crates/omena-cli/persona-presets");
  const fileIds = readdirSync(presetDirectory)
    .filter((filename) => filename.endsWith(".toml"))
    .map((filename) => filename.slice(0, -".toml".length))
    .sort();
  const manifestIds = manifest.presets.map(({ id }) => id).sort();
  assert.deepEqual(
    manifestIds,
    fileIds,
    "persona reference must be derived from the TOML file set",
  );
  for (const preset of manifest.presets) {
    assert.equal(
      preset.configPath,
      `persona-presets/${preset.id}.toml`,
      `persona ${preset.id} must point to its source TOML file`,
    );
    assert.ok(
      existsSync(path.join(repoRoot, "rust/crates/omena-cli", preset.configPath)),
      `persona ${preset.id} is missing its source TOML file`,
    );
  }
  assert.equal(manifest.consumption.syntax, 'extends = "omena:<preset-id>"');
  return [...manifest.presets].sort((left, right) => left.priority - right.priority);
}

function deriveConfigKeys(source: string): readonly ConfigKeyRow[] {
  const rows: ConfigKeyRow[] = [];
  const visit = (structName: string, prefix: string, seen: readonly string[]): void => {
    assert.ok(!seen.includes(structName), `config schema recursion detected at ${structName}`);
    for (const field of extractStructFields(source, structName)) {
      const key = prefix ? `${prefix}.${toCamelCase(field.name)}` : toCamelCase(field.name);
      const nested = field.type.match(/\b(Omena(?:[A-Za-z0-9]+Config|ConfigOverride))\b/u)?.[1];
      if (nested) {
        const collectionKey = field.type.startsWith("Vec<") ? `${key}[]` : key;
        visit(nested, collectionKey, [...seen, structName]);
        continue;
      }
      rows.push({ key, owner: structName });
    }
  };
  visit("OmenaConfig", "", []);
  return rows.sort((left, right) => left.key.localeCompare(right.key));
}

function readLspBoundary(): LspBoundarySummary {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-lsp-server",
      "--bin",
      "omena-lsp-server-boundary",
      "--quiet",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(
    result.status,
    0,
    ["omena-lsp-server boundary binary failed", result.stdout.trim(), result.stderr.trim()]
      .filter(Boolean)
      .join("\n"),
  );
  return JSON.parse(result.stdout) as LspBoundarySummary;
}

function renderReferenceIndex(): string {
  return `${generatedNotice}
# Omena reference

These tables are rendered from the current product contracts and checked in CI.

- [CLI commands and product verbs](./cli.md)
- [Persona presets](./personas.md)
- [Configuration keys](./configuration.md)
- [LSP capabilities](./lsp-capabilities.md)
`;
}

function renderCliReference(
  verbs: readonly ProductVerbRow[],
  commandRows: readonly CommandRow[],
): string {
  const verbRows = verbs
    .map(
      ({ verb, status, wiredBy }) =>
        `| \`omena ${verb}\` | ${formatVerbStatus(status)} | ${wiredBy ? `\`${wiredBy}\`` : "-"} |`,
    )
    .join("\n");
  return `${generatedNotice}
# CLI reference

## Product verbs

| Command | Status | Dispatch owner |
| --- | --- | --- |
${verbRows}

## Complete command surface

${renderCommandTable(commandRows, verbs)}
`;
}

function renderPersonaReference(personaRows: PersonaManifest["presets"]): string {
  const rows = personaRows
    .map(
      ({ id, audience, verbs }) =>
        `| \`${id}\` | \`${audience}\` | ${verbs.map((verb) => `\`${verb}\``).join(", ")} |`,
    )
    .join("\n");
  return `${generatedNotice}
# Persona presets

Use a built-in preset with \`extends = "omena:<preset-id>"\` in \`omena.toml\`.

| Preset | Audience | Product verbs |
| --- | --- | --- |
${rows}
`;
}

function renderConfigReference(rows: readonly ConfigKeyRow[]): string {
  const table = rows.map(({ key, owner }) => `| \`${key}\` | \`${owner}\` |`).join("\n");
  return `${generatedNotice}
# Configuration reference

Canonical \`omena.toml\` keys use camelCase. Nested override entries are shown with \`[]\`.

| Key path | Typed owner |
| --- | --- |
${table}
`;
}

function renderLspReference(
  rows: readonly { readonly path: string; readonly value: unknown }[],
): string {
  const table = rows
    .map(
      ({ path: capabilityPath, value }) => `| \`${capabilityPath}\` | \`${formatValue(value)}\` |`,
    )
    .join("\n");
  return `${generatedNotice}
# LSP capabilities

The table is rendered from the Rust server's serialized initialize capability contract.

| Capability path | Value |
| --- | --- |
${table}
`;
}

function renderCliReadme(
  source: string,
  verbs: readonly ProductVerbRow[],
  commandRows: readonly CommandRow[],
): string {
  const markerStart = "<!-- BEGIN GENERATED: OMENA CLI COMMANDS -->";
  const markerEnd = "<!-- END GENERATED: OMENA CLI COMMANDS -->";
  const generatedBlock = `${markerStart}
${generatedNotice}
${renderCommandTable(commandRows, verbs)}
${markerEnd}`;

  if (source.includes(markerStart)) {
    const start = source.indexOf(markerStart);
    const end = source.indexOf(markerEnd, start);
    assert.notEqual(end, -1, "CLI README command-list end marker is missing");
    return `${source.slice(0, start)}${generatedBlock}${source.slice(end + markerEnd.length)}`;
  }

  const startMarker = "Current commands:";
  const endMarker = "Install the published CLI with Cargo:";
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker, start);
  assert.notEqual(start, -1, "CLI README current-command list is missing");
  assert.notEqual(end, -1, "CLI README install section is missing");
  return `${source.slice(0, start)}## Commands

${generatedBlock}

${source.slice(end)}`;
}

function renderCommandTable(
  commandRows: readonly CommandRow[],
  verbs: readonly ProductVerbRow[],
): string {
  const byVerb = new Map(verbs.map((row) => [row.verb, row]));
  const rows = commandRows
    .map(({ command, summary }) => {
      const verb = byVerb.get(command);
      const role = verb ? formatVerbStatus(verb.status) : "Specialized command";
      const purpose =
        verb?.status === "reserved-alias"
          ? `Compatibility route through \`${verb.wiredBy}\`.`
          : summary;
      return `| \`omena ${command}\` | ${role} | ${escapeTableCell(purpose)} |`;
    })
    .join("\n");
  return `| Command | Role | Purpose |
| --- | --- | --- |
${rows}`;
}

function formatVerbStatus(status: ProductVerbStatus): string {
  if (status === "wired") return "Product command";
  if (status === "reserved-alias") return "Compatibility alias";
  return "Reserved";
}

function assertGeneratedFile(relativePath: string, expected: string): void {
  const absolutePath = path.join(repoRoot, relativePath);
  if (writeMode) {
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, expected);
    return;
  }
  assert.ok(
    existsSync(absolutePath),
    `${relativePath} is missing; regenerate the reference surface`,
  );
  assert.equal(
    readFileSync(absolutePath, "utf8"),
    expected,
    `${relativePath} is stale; regenerate the reference surface`,
  );
}

function verifyReadmeBudget(): void {
  const readme = read("README.md");
  const lineCount = readme.endsWith("\n")
    ? readme.split("\n").length - 1
    : readme.split("\n").length;
  assert.ok(
    lineCount <= readmeLineBudget,
    `README.md has ${lineCount} lines; the current public-front-door budget is ${readmeLineBudget}`,
  );
}

function verifyCheckScriptReachability(): void {
  const checkScripts = readdirSync(path.join(repoRoot, "scripts"))
    .filter((filename) => /^check-.*\.(?:ts|mjs)$/u.test(filename))
    .sort();
  const referenceFiles = [
    "package.json",
    "packages/check-orchestrator/src/manifest/declared.ts",
    ...walkFiles(path.join(repoRoot, ".github/workflows"), (file) => file.endsWith(".yml")),
    ...walkFiles(path.join(repoRoot, "scripts"), (file) => /tsconfig.*\.json$/u.test(file)),
  ];
  const references = referenceFiles
    .map((file) => readFileSync(path.isAbsolute(file) ? file : path.join(repoRoot, file), "utf8"))
    .join("\n");
  const unwired = checkScripts.filter((filename) => !references.includes(filename));
  assert.deepEqual(
    unwired,
    [...expectedUnwiredChecks],
    "every check script must be wired or remain in the reviewed shrink-only ledger",
  );
}

function verifyExecutableTomlExamples(): void {
  const markdownFiles = [
    path.join(repoRoot, "README.md"),
    path.join(repoRoot, "CONTRIBUTING.md"),
    path.join(repoRoot, "RELEASING.md"),
    path.join(repoRoot, "rust/crates/omena-cli/README.md"),
    ...walkFiles(path.join(repoRoot, "docs"), (file) => file.endsWith(".md")),
  ];
  const examples = markdownFiles.flatMap((file) => extractTomlFences(file));
  assert.ok(examples.length > 0, "the public docs must retain executable omena.toml examples");
  for (const example of examples) verifyTomlExample(example);
}

function verifyTomlExample(example: {
  readonly file: string;
  readonly line: number;
  readonly body: string;
}): void {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), "omena-doc-config-"));
  try {
    mkdirSync(path.join(fixtureRoot, "src"));
    mkdirSync(path.join(fixtureRoot, "dist"));
    writeFileSync(path.join(fixtureRoot, "src/input.css"), ".card { color: red; }\n");
    writeFileSync(path.join(fixtureRoot, "omena.toml"), example.body);
    materializeRelativeExtends(fixtureRoot, example.body);
    const result = spawnSync(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-cli",
        "--bin",
        "omena",
        "--",
        "build",
        path.join(fixtureRoot, "src/input.css"),
        "--json",
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(
      result.status,
      0,
      `${path.relative(repoRoot, example.file)}:${example.line} is not accepted by the real config parser\n${result.stderr}`,
    );
    assert.doesNotMatch(
      result.stderr,
      /omena config \[unknownKey\]/u,
      `${path.relative(repoRoot, example.file)}:${example.line} contains an unknown config key`,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
}

function materializeRelativeExtends(root: string, source: string): void {
  for (const line of source.split("\n")) {
    if (!/^\s*extends\s*=/u.test(line)) continue;
    for (const match of line.matchAll(/"([^"]+)"/gu)) {
      const value = match[1]!;
      if (value.startsWith("omena:") || value.includes("://") || value.includes("${")) continue;
      const destination = path.resolve(root, value);
      assert.ok(
        destination === root || destination.startsWith(`${root}${path.sep}`),
        `documented extends path escapes its fixture root: ${value}`,
      );
      mkdirSync(path.dirname(destination), { recursive: true });
      if (!existsSync(destination)) writeFileSync(destination, "");
    }
  }
}

function extractTomlFences(file: string): readonly {
  readonly file: string;
  readonly line: number;
  readonly body: string;
}[] {
  const source = readFileSync(file, "utf8");
  return [...source.matchAll(/^```toml[^\n]*\n([\s\S]*?)^```\s*$/gmu)].map((match) => ({
    file,
    line: source.slice(0, match.index).split("\n").length,
    body: match[1]!.endsWith("\n") ? match[1]! : `${match[1]}\n`,
  }));
}

function extractCommandRows(source: string): readonly CommandRow[] {
  const body = extractBlock(source, "enum Command");
  const rows: CommandRow[] = [];
  let depth = 0;
  let docs: string[] = [];
  for (const line of body.split("\n")) {
    if (depth === 0) {
      const doc = line.match(/^\s{4}\/\/\/\s?(.*)$/u)?.[1];
      if (doc !== undefined) {
        docs.push(doc);
      } else {
        const variant = line.match(/^\s{4}([A-Z][A-Za-z0-9_]*)(?:\s*\{|\s*\(|,)\s*/u)?.[1];
        if (variant) {
          rows.push({
            variant,
            command: toKebabCase(variant),
            summary: docs.join(" ").replace(/\s+/gu, " ").trim() || "CLI command.",
          });
          docs = [];
        } else if (line.trim() && !line.trimStart().startsWith("#[")) {
          docs = [];
        }
      }
    }
    depth += countCharacter(line, "{") - countCharacter(line, "}");
  }
  assert.ok(rows.length > 0, "Command enum must expose at least one command");
  assert.equal(new Set(rows.map(({ command }) => command)).size, rows.length);
  return rows;
}

function extractStructFields(
  source: string,
  name: string,
): readonly { readonly name: string; readonly type: string }[] {
  const body = extractBlock(source, `struct ${name}`);
  return body.split("\n").flatMap((line) => {
    const match = line.match(/^\s+pub\(crate\)\s+([a-z][a-z0-9_]*):\s*(.+),$/u);
    return match ? [{ name: match[1]!, type: match[2]!.trim() }] : [];
  });
}

function extractEnumVariants(source: string, name: string): readonly string[] {
  return extractBlock(source, `enum ${name}`)
    .split("\n")
    .flatMap((line) => line.match(/^\s+([A-Z][A-Za-z0-9_]*),/u)?.slice(1) ?? []);
}

function extractBlock(source: string, declaration: string): string {
  const start = source.indexOf(declaration);
  assert.notEqual(start, -1, `missing ${declaration}`);
  const bodyStart = source.indexOf("{", start) + 1;
  let depth = 1;
  let cursor = bodyStart;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated ${declaration}`);
  return source.slice(bodyStart, cursor - 1);
}

function flattenObject(
  value: Readonly<Record<string, unknown>>,
  prefix = "",
): readonly { readonly path: string; readonly value: unknown }[] {
  return Object.entries(value).flatMap(([key, entry]) => {
    const entryPath = prefix ? `${prefix}.${key}` : key;
    if (entry !== null && typeof entry === "object" && !Array.isArray(entry)) {
      return flattenObject(entry as Readonly<Record<string, unknown>>, entryPath);
    }
    return [{ path: entryPath, value: entry }];
  });
}

function walkFiles(directory: string, predicate: (file: string) => boolean): string[] {
  if (!existsSync(directory)) return [];
  const files: string[] = [];
  for (const entry of readdirSync(directory)) {
    const absolutePath = path.join(directory, entry);
    if (statSync(absolutePath).isDirectory()) files.push(...walkFiles(absolutePath, predicate));
    else if (predicate(absolutePath)) files.push(absolutePath);
  }
  return files.sort();
}

function countCharacter(value: string, character: string): number {
  return [...value].filter((candidate) => candidate === character).length;
}

function formatValue(value: unknown): string {
  if (Array.isArray(value)) return JSON.stringify(value);
  return String(value);
}

function escapeTableCell(value: string): string {
  return value.replace(/\|/gu, "\\|");
}

function toCamelCase(value: string): string {
  const lower = value.replace(/^[A-Z]/u, (character) => character.toLowerCase());
  return lower.replace(/_([a-z])/gu, (_, character: string) => character.toUpperCase());
}

function toKebabCase(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase();
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
