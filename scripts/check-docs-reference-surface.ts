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
import { CONTRIBUTOR_RECIPE_TARGETS } from "../packages/check-orchestrator/src/manifest/documented-commands";

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

interface SdkWorkflowMatrix {
  readonly workflows: readonly string[];
  readonly surfaces: readonly string[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const writeMode = process.argv.includes("--write");
const generatedNotice = "<!-- Generated from product code. Do not edit by hand. -->";
const readmeLineBudget = 130;
const contributingLineBudget = 180;
const releasingLineBudget = 200;
const contributorPolicyContract = `## Commit Messages

Use plain imperative commit subjects:

\`\`\`text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
\`\`\`

Keep commit messages understandable without private planning documents. Do not
use internal planning labels, phase names, issue-triage shorthand, or private
catalog identifiers in public history.

## Verification

Run the smallest relevant check for the files you changed, then run the broader
gate before release-oriented changes. Prefer existing \`pnpm omena-check\` targets
when a target exists for the changed subsystem.`;
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
const sdkWorkflowMatrix = deriveSdkWorkflowMatrix();
const architectureCitations = verifyArchitectureCodemap();
const operationalGuideLines = verifyOperationalGuides();

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

const vscodeGuidePath = "docs/vscode-extension.md";
assertGeneratedFile(
  vscodeGuidePath,
  replaceGeneratedBlock(
    read(vscodeGuidePath),
    "OMENA PERSONA PRESETS",
    `${generatedNotice}\n\n${renderPersonaTable(personas)}`,
  ),
);
const sdkGuidePath = "docs/sdk.md";
assertGeneratedFile(
  sdkGuidePath,
  replaceGeneratedBlock(
    read(sdkGuidePath),
    "OMENA SDK WORKFLOWS",
    renderSdkWorkflowTable(sdkWorkflowMatrix),
  ),
);

verifyReadmeBudget();
const readmeLinks = verifyReadmeLinkMap();
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
      architectureCitations,
      operationalGuideLines,
      readmeLinks,
      generatedFragments: 2,
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

function verifyArchitectureCodemap(): number {
  const architecturePath = "ARCHITECTURE.md";
  assert.ok(existsSync(path.join(repoRoot, architecturePath)), `${architecturePath} must exist`);
  assert.ok(
    !existsSync(path.join(repoRoot, "docs/architecture-v3.md")),
    "the retired architecture document must not remain beside the current codemap",
  );

  const architecture = read(architecturePath);
  assert.ok(
    architecture.split("\n").length - 1 <= 220,
    `${architecturePath} must remain a navigable codemap of at most 220 lines`,
  );
  assert.ok(
    !architecture.includes("binder-builder.ts"),
    "the codemap must not cite binder-builder.ts",
  );
  assert.ok(
    architecture.includes("server/engine-core-ts/src/core/binder/source-binder.ts"),
    "the codemap must cite the current source binder",
  );

  const productPathMatrix = readJson<{
    readonly entries: readonly { readonly crate: string; readonly surface: string }[];
  }>("rust/omena-product-path-matrix.json");
  const excludedSurfaces = new Set(["check-evidence", "legacy-oracle", "research-fixture"]);
  const productCrates = productPathMatrix.entries
    .filter(({ surface }) => !excludedSurfaces.has(surface))
    .map(({ crate }) => crate)
    .toSorted();
  assert.equal(productCrates.length, 41, "the architecture product-path family count changed");
  for (const crateName of productCrates) {
    assert.ok(architecture.includes(`\`${crateName}\``), `${architecturePath} omits ${crateName}`);
  }

  for (const requiredDoc of [
    "docs/workspace-session-routing.md",
    "docs/governance/crate-boundary-review.md",
    "docs/engine-v2-contract-idl-decisions.md",
  ]) {
    assert.equal(
      architecture.split(requiredDoc).length - 1,
      1,
      `${architecturePath} must name ${requiredDoc} exactly once`,
    );
  }

  const citedPaths = new Set<string>();
  for (const match of architecture.matchAll(
    /`((?:client|contracts|packages|rust|scripts|server|test)\/[^`\s]+)`/gu,
  )) {
    citedPaths.add(match[1]);
  }
  for (const match of architecture.matchAll(/\]\((?!https?:)([^)#]+)(?:#[^)]+)?\)/gu)) {
    citedPaths.add(match[1].replace(/^\.\//u, ""));
  }
  for (const citedPath of citedPaths) {
    assert.ok(
      existsSync(path.join(repoRoot, citedPath)),
      `${architecturePath} cites missing ${citedPath}`,
    );
  }
  return citedPaths.size;
}

function derivePersonas(): PersonaManifest["presets"] {
  const manifest = readJson<PersonaManifest>("rust/crates/omena-cli/persona-presets.json");
  const presetDirectory = path.join(repoRoot, "rust/crates/omena-cli/persona-presets");
  const fileIds = readdirSync(presetDirectory)
    .filter((filename) => filename.endsWith(".toml"))
    .map((filename) => filename.slice(0, -".toml".length))
    .toSorted();
  const manifestIds = manifest.presets.map(({ id }) => id).toSorted();
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
  return manifest.presets.toSorted((left, right) => left.priority - right.priority);
}

function deriveSdkWorkflowMatrix(): SdkWorkflowMatrix {
  const contract = read("contracts/engine-sdk-workflow/main.tsp");
  const workflows = [...contract.matchAll(/model OmenaSdk([A-Z][A-Za-z0-9]+)RequestV0\s*\{/gu)]
    .map((match) => toCamelCase(match[1]))
    .filter((workflow) => workflow !== "errorEnvelope");
  const matrix = readJson<SdkWorkflowMatrix>("rust/omena-sdk-workflow-parity-matrix.json");
  assert.deepEqual(
    matrix.workflows,
    workflows,
    "SDK workflow parity must follow the TypeSpec request-model order",
  );
  assert.deepEqual(
    matrix.surfaces,
    ["napi", "wasm", "cli", "lsp"],
    "SDK documentation expects the four shipped parity surfaces",
  );
  return matrix;
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
  return rows.toSorted((left, right) => left.key.localeCompare(right.key));
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
  const contributorTable = renderMarkdownTable(
    ["Command", "Contract"],
    CONTRIBUTOR_RECIPE_TARGETS.map(({ target, purpose }) => [
      `\`pnpm omena-check run ${target}\``,
      purpose,
    ]),
  );
  return `${generatedNotice}

# Omena reference

These tables are rendered from the current product contracts and checked in CI.

- [CLI commands and product verbs](./cli.md)
- [Persona presets](./personas.md)
- [Configuration keys](./configuration.md)
- [LSP capabilities](./lsp-capabilities.md)

## Contributor checks

${contributorTable}
`;
}

function renderCliReference(
  verbs: readonly ProductVerbRow[],
  commandRows: readonly CommandRow[],
): string {
  const verbTable = renderMarkdownTable(
    ["Command", "Status", "Dispatch owner"],
    verbs.map(({ verb, status, wiredBy }) => [
      `\`omena ${verb}\``,
      formatVerbStatus(status),
      wiredBy ? `\`${wiredBy}\`` : "-",
    ]),
  );
  return `${generatedNotice}

# CLI reference

## Product verbs

${verbTable}

## Complete command surface

${renderCommandTable(commandRows, verbs)}
`;
}

function renderPersonaReference(personaRows: PersonaManifest["presets"]): string {
  return `${generatedNotice}

# Persona presets

Use a built-in preset with \`extends = "omena:<preset-id>"\` in \`omena.toml\`.

${renderPersonaTable(personaRows).trimEnd()}
`;
}

function renderPersonaTable(personaRows: PersonaManifest["presets"]): string {
  return renderMarkdownTable(
    ["Preset", "Audience", "Product verbs"],
    personaRows.map(({ id, audience, verbs }) => [
      `\`${id}\``,
      `\`${audience}\``,
      verbs.map((verb) => `\`${verb}\``).join(", "),
    ]),
  );
}

function renderSdkWorkflowTable(matrix: SdkWorkflowMatrix): string {
  const surfaces = matrix.surfaces.map((surface) => `\`${surface}\``).join(", ");
  return `${generatedNotice}

${renderMarkdownTable(
  ["Workflow", "Covered surfaces"],
  matrix.workflows.map((workflow) => [`\`${workflow}\``, surfaces]),
)}
`;
}

function renderConfigReference(rows: readonly ConfigKeyRow[]): string {
  const table = renderMarkdownTable(
    ["Key path", "Typed owner"],
    rows.map(({ key, owner }) => [`\`${key}\``, `\`${owner}\``]),
  );
  return `${generatedNotice}

# Configuration reference

Canonical \`omena.toml\` keys use camelCase. Nested override entries are shown with \`[]\`.

${table}
`;
}

function renderLspReference(
  rows: readonly { readonly path: string; readonly value: unknown }[],
): string {
  const table = renderMarkdownTable(
    ["Capability path", "Value"],
    rows.map(({ path: capabilityPath, value }) => [
      `\`${capabilityPath}\``,
      `\`${formatValue(value)}\``,
    ]),
  );
  return `${generatedNotice}

# LSP capabilities

The table is rendered from the Rust server's serialized initialize capability contract.

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
  const rows = commandRows.map(({ command, summary }) => {
    const verb = byVerb.get(command);
    const role = verb ? formatVerbStatus(verb.status) : "Specialized command";
    const purpose =
      verb?.status === "reserved-alias"
        ? `Compatibility route through \`${verb.wiredBy}\`.`
        : summary;
    return [`\`omena ${command}\``, role, escapeTableCell(purpose)];
  });
  return renderMarkdownTable(["Command", "Role", "Purpose"], rows);
}

function formatVerbStatus(status: ProductVerbStatus): string {
  if (status === "wired") return "Product command";
  if (status === "reserved-alias") return "Compatibility alias";
  return "Reserved";
}

function replaceGeneratedBlock(source: string, label: string, body: string): string {
  const markerStart = `<!-- BEGIN GENERATED: ${label} -->`;
  const markerEnd = `<!-- END GENERATED: ${label} -->`;
  const start = source.indexOf(markerStart);
  const end = source.indexOf(markerEnd, start);
  assert.notEqual(start, -1, `${label} start marker is missing`);
  assert.notEqual(end, -1, `${label} end marker is missing`);
  const block = `${markerStart}\n${body.trimEnd()}\n\n${markerEnd}`;
  return `${source.slice(0, start)}${block}${source.slice(end + markerEnd.length)}`;
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
  const readmeLines = readme.endsWith("\n")
    ? readme.split("\n").length - 1
    : readme.split("\n").length;
  assert.ok(
    readmeLines <= readmeLineBudget,
    `README.md has ${readmeLines} lines; the current public-front-door budget is ${readmeLineBudget}`,
  );
}

function verifyOperationalGuides(): { readonly contributing: number; readonly releasing: number } {
  const contributing = read("CONTRIBUTING.md");
  const releasing = read("RELEASING.md");
  assert.ok(
    contributing.includes(contributorPolicyContract),
    "CONTRIBUTING.md must preserve the commit-message and verification policy verbatim",
  );
  assert.ok(
    contributing.includes("[release runbook](RELEASING.md)"),
    "CONTRIBUTING.md must link maintainers to RELEASING.md",
  );
  for (const heading of ["### crates.io", "### npm", "### VS Code Marketplace", "### Open VSX"]) {
    assert.ok(releasing.includes(heading), `RELEASING.md must retain ${heading}`);
  }
  const contributingLines = lineCount(contributing);
  const releasingLines = lineCount(releasing);
  assert.ok(
    contributingLines <= contributingLineBudget,
    `CONTRIBUTING.md has ${contributingLines} lines; budget is ${contributingLineBudget}`,
  );
  assert.ok(
    releasingLines <= releasingLineBudget,
    `RELEASING.md has ${releasingLines} lines; budget is ${releasingLineBudget}`,
  );
  return { contributing: contributingLines, releasing: releasingLines };
}

function lineCount(source: string): number {
  return source.endsWith("\n") ? source.split("\n").length - 1 : source.split("\n").length;
}

function verifyReadmeLinkMap(): number {
  const readme = read("README.md");
  const targets = [...readme.matchAll(/\[[^\]]+\]\(([^)]+)\)/gu)].map((match) => match[1]);
  const requiredTargets = [
    "https://marketplace.visualstudio.com/items?itemName=omena.omena-css",
    "rust/crates/omena-cli/README.md",
    "docs/sdk.md",
    "https://docs.rs/omena-lsp-server",
    "docs/clients/zed.md",
    "docs/clients/neovim.md",
    "rust/crates/omena-bundler/README.md",
    "packages/vite-plugin/README.md",
    "packages/eslint-plugin/README.md",
    "packages/stylelint-plugin/README.md",
    "docs/sass-compat.md",
    "docs/migrate-verb.md",
    "docs/vscode-extension.md",
    "docs/positioning.md",
    "ARCHITECTURE.md",
    "docs/performance.md",
    "CHANGELOG.md",
    "CONTRIBUTING.md",
    "docs/reference/README.md",
  ] as const;
  for (const requiredTarget of requiredTargets) {
    assert.ok(targets.includes(requiredTarget), `README.md link map omits ${requiredTarget}`);
  }
  for (const target of targets) {
    if (/^(?:https?:|mailto:)/u.test(target)) continue;
    const localTarget = target.split("#", 1)[0].replace(/^\.\//u, "");
    assert.ok(localTarget, `README.md contains an empty local link target: ${target}`);
    assert.ok(existsSync(path.join(repoRoot, localTarget)), `README.md links missing ${target}`);
  }
  return targets.length;
}

function verifyCheckScriptReachability(): void {
  const checkScripts = readdirSync(path.join(repoRoot, "scripts"))
    .filter((filename) => /^check-.*\.(?:ts|mjs)$/u.test(filename))
    .toSorted();
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
  return files.toSorted();
}

function countCharacter(value: string, character: string): number {
  return [...value].filter((candidate) => candidate === character).length;
}

function formatValue(value: unknown): string {
  if (Array.isArray(value)) return JSON.stringify(value);
  return String(value);
}

function renderMarkdownTable(
  headers: readonly string[],
  rows: readonly (readonly string[])[],
): string {
  assert.ok(headers.length > 0, "generated Markdown tables require at least one column");
  assert.ok(
    rows.every((row) => row.length === headers.length),
    "generated Markdown table rows must match the header width",
  );
  const widths = headers.map((header, index) =>
    Math.max(3, header.length, ...rows.map((row) => row[index]!.length)),
  );
  const renderRow = (cells: readonly string[]): string =>
    `| ${cells.map((cell, index) => cell.padEnd(widths[index]!)).join(" | ")} |`;
  return [
    renderRow(headers),
    renderRow(widths.map((width) => "-".repeat(width))),
    ...rows.map(renderRow),
  ].join("\n");
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
