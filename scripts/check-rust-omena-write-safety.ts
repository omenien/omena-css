import { strict as assert } from "node:assert";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type WriteClassification = "artifact" | "bookkeeping" | "source-mutation-gate";

interface WriteSite {
  readonly path: string;
  readonly function: string;
  readonly writeCount: number;
  readonly classification: WriteClassification;
  readonly owner: string;
}

interface WriteSafetyManifest {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.write-safety-census";
  readonly sourceMutationGate: { readonly path: string; readonly function: string };
  readonly productSourceWriteCallers: number;
  readonly writeSites: readonly WriteSite[];
  readonly consumerContracts: readonly {
    readonly surface: string;
    readonly writeKind: string;
    readonly additionalRequirement: string;
    readonly defaultPosture: string;
  }[];
  readonly namedWaits: readonly {
    readonly surface: string;
    readonly condition: string;
    readonly owner: string;
  }[];
}

interface NonFilesystemWriteSink {
  readonly path: string;
  readonly function: string;
  readonly writeCount: number;
  readonly evidence: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cliRoot = "rust/crates/omena-cli/src";
const manifestPath = "rust/crates/omena-cli/write-safety-census.json";
const manifest = readJson<WriteSafetyManifest>(manifestPath);
const fixSafetySource = read("rust/crates/omena-checker/src/fix_safety.rs");
const writeGateSource = read(manifest.sourceMutationGate.path);
const queryRunnerSource = read("rust/crates/omena-query-transform-runner/src/lib.rs");
const queryFacadeSource = read("rust/crates/omena-query/src/lib.rs");
const productionWritePrimitive =
  /\b(?:std::)?fs::write\s*\(|\bFile::create\s*\(|\bOpenOptions::new\s*\(|\.write_all\s*\(/gu;
const nonFilesystemWriteSinks: readonly NonFilesystemWriteSink[] = [
  {
    path: "rust/crates/omena-cli/src/daemon.rs",
    function: "emit_watch_result",
    writeCount: 1,
    evidence: "std::io::stdout()",
  },
  {
    path: "rust/crates/omena-cli/src/daemon.rs",
    function: "write_wire_bytes",
    writeCount: 2,
    evidence: "TcpStream",
  },
];

assert.deepEqual(
  enumDeclarations(`
const DECOY: &str = "pub enum StringLiteral { Safe, Conservative, ManualReview }";
const RAW_DECOY: &str = r#"pub enum RawLiteral { Safe, Conservative, ManualReview }"#;
const LEFT_BRACE: char = '{';
// pub enum LineComment { Safe, Conservative, ManualReview }
/* pub enum BlockComment { Safe, Conservative, ManualReview } */
enum SourceDeclaration {
  Safe,
  Conservative,
  ManualReview,
}
`),
  [{ name: "SourceDeclaration", variants: ["Safe", "Conservative", "ManualReview"] }],
  "write-safety census must inspect Rust declarations rather than literal or comment text",
);

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.write-safety-census");
assert.deepEqual(extractEnumVariants(fixSafetySource, "FixSafetyV0"), [
  "Safe",
  "Conservative",
  "ManualReview",
]);
assert.doesNotMatch(
  fixSafetySource,
  /OmenaCheckerRuleCodeV0/u,
  "fix safety must be derived from evidence rather than a rule-code table",
);
for (const signal of [
  "syntax_preserving",
  "local_semantics_required",
  "local_semantics_ready",
  "closed_world_required",
  "closed_world_ready",
  "reference_precision_required",
  "reference_precision",
]) {
  assert.ok(fixSafetySource.includes(`pub ${signal}:`), `missing evidence signal ${signal}`);
}
for (const precision of ["Exact", "Conservative", "Heuristic", "Unknown"]) {
  assert.ok(
    fixSafetySource.includes(`FactPrecision::${precision}`),
    `FactPrecision::${precision} must affect classification`,
  );
}
assert.ok(fixSafetySource.includes('rationale.push("syntaxSafe")'));
assert.ok(fixSafetySource.includes('rationale.push("localSemanticSafe")'));
assert.ok(fixSafetySource.includes('rationale.push("workspaceClosedWorldSafe")'));

assert.ok(queryRunnerSource.includes("RollbackReceiptV0"));
assert.ok(queryRunnerSource.includes("TransformDecision"));
assert.ok(queryFacadeSource.includes("TransformDecision as OmenaQueryTransformDecisionV0"));
for (const variant of ["Applied", "NoChange", "Blocked", "Rejected"]) {
  assert.ok(
    writeGateSource.includes(`OmenaQueryTransformDecisionV0::${variant}`),
    `write gate must consume TransformDecision::${variant}`,
  );
}

const allRustFiles = rustSourceFiles("rust/crates");
const safetyAuthorities = allRustFiles.filter((file) => read(file).includes("enum FixSafetyV0"));
assert.deepEqual(safetyAuthorities, ["rust/crates/omena-checker/src/fix_safety.rs"]);
assertNoSemanticSafetyCopies(allRustFiles);
assertNoTypeScriptSafetyCopies();

const derivedWriteSites = deriveProductionWriteSites();
assert.deepEqual(
  manifest.writeSites.map(siteIdentity).toSorted(),
  derivedWriteSites.map(siteIdentity).toSorted(),
  "every production filesystem write must have an owned classification",
);
assert.equal(
  manifest.writeSites.filter(({ classification }) => classification === "source-mutation-gate")
    .length,
  1,
  "source mutation must have one filesystem gate",
);
assert.deepEqual(
  manifest.writeSites.find(({ classification }) => classification === "source-mutation-gate"),
  {
    ...manifest.sourceMutationGate,
    writeCount: 1,
    classification: "source-mutation-gate",
    owner: "classified product source edits",
  },
);

const productionGateSource = stripCfgTestModules(writeGateSource, manifest.sourceMutationGate.path);
const gateOccurrenceCount = [...productionGateSource.matchAll(/\bapply_write_with_safety\s*\(/gu)]
  .length;
assert.equal(gateOccurrenceCount, 1, "write gate must have one definition and no hidden self-call");
const cliProductionSources = rustSourceFiles(cliRoot).map((file) =>
  stripCfgTestModules(read(file), file),
);
const allGateOccurrences = cliProductionSources.reduce(
  (count, source) => count + [...source.matchAll(/\bapply_write_with_safety\s*\(/gu)].length,
  0,
);
assert.equal(
  allGateOccurrences - 1,
  manifest.productSourceWriteCallers,
  "product source-write caller count must remain explicit until a real consumer lands",
);

assert.deepEqual(
  manifest.consumerContracts.map(({ surface }) => surface),
  ["lint", "format", "minify", "migrate"],
);
assert.deepEqual(
  manifest.consumerContracts.map(({ writeKind }) => writeKind),
  ["lintFix", "formatting", "transform", "migrationPlan"],
);
assert.deepEqual(
  manifest.consumerContracts.map(({ additionalRequirement }) => additionalRequirement),
  [
    "sharedSafetyAssessment",
    "observedIdempotence",
    "appliedTransformDecisionWithoutBlockedOrRejected",
    "reviewedPlan",
  ],
);
assert.deepEqual(
  manifest.namedWaits.map(({ surface, condition }) => `${surface}:${condition}`),
  [
    "lint:routedSourceFix",
    "check:integratedCheckComposition",
    "source-edit:structuralSharingRevalidation",
  ],
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-write-safety",
      safetyAuthorityCount: safetyAuthorities.length,
      productionWriteSiteCount: derivedWriteSites.reduce(
        (count, { writeCount }) => count + writeCount,
        0,
      ),
      classifiedFunctionCount: derivedWriteSites.length,
      sourceMutationGateCount: 1,
      productSourceWriteCallers: manifest.productSourceWriteCallers,
      consumerContractCount: manifest.consumerContracts.length,
      namedWaitCount: manifest.namedWaits.length,
    },
    null,
    2,
  )}\n`,
);

function deriveProductionWriteSites(): WriteSite[] {
  const manifestByKey = new Map(
    manifest.writeSites.map((site) => [`${site.path}#${site.function}`, site]),
  );
  const derived = new Map<string, { path: string; function: string; writeCount: number }>();
  const observedNonFilesystemWrites = new Map<string, number>();
  const nonFilesystemByKey = new Map(
    nonFilesystemWriteSinks.map((sink) => [`${sink.path}#${sink.function}`, sink]),
  );

  for (const file of rustSourceFiles(cliRoot)) {
    const source = stripCfgTestModules(read(file), file);
    const functions = topLevelFunctions(source);
    for (const match of source.matchAll(productionWritePrimitive)) {
      const offset = match.index ?? -1;
      const owner = functions.findLast(({ start }) => start < offset);
      assert.ok(owner, `${file} contains fs::write outside a named function`);
      const key = `${file}#${owner.name}`;
      if (nonFilesystemByKey.has(key)) {
        observedNonFilesystemWrites.set(key, (observedNonFilesystemWrites.get(key) ?? 0) + 1);
        continue;
      }
      const current = derived.get(key) ?? { path: file, function: owner.name, writeCount: 0 };
      current.writeCount += 1;
      derived.set(key, current);
    }
  }

  for (const sink of nonFilesystemWriteSinks) {
    const key = `${sink.path}#${sink.function}`;
    assert.equal(
      observedNonFilesystemWrites.get(key),
      sink.writeCount,
      `non-filesystem write sink changed: ${key}`,
    );
    const source = stripCfgTestModules(read(sink.path), sink.path);
    const functions = topLevelFunctions(source);
    const index = functions.findIndex(({ name }) => name === sink.function);
    assert.ok(index >= 0, `non-filesystem write sink is missing: ${key}`);
    const end = functions[index + 1]?.start ?? source.length;
    assert.ok(
      source.slice(functions[index]!.start, end).includes(sink.evidence),
      `non-filesystem write sink lost its ${sink.evidence} evidence: ${key}`,
    );
  }

  return [...derived.values()].map((site) => {
    const registered = manifestByKey.get(`${site.path}#${site.function}`);
    assert.ok(registered, `unclassified production write: ${site.path}#${site.function}`);
    return { ...site, classification: registered.classification, owner: registered.owner };
  });
}

function assertNoSemanticSafetyCopies(files: readonly string[]): void {
  const auto = new Set(["safe", "automatic", "autoapply", "autowrite"]);
  const optIn = new Set(["conservative", "optin", "explicitapproval"]);
  const manual = new Set(["manualreview", "manual", "reviewonly"]);
  const copies: string[] = [];

  for (const file of files) {
    const source = read(file);
    for (const declaration of enumDeclarations(source)) {
      const normalized = declaration.variants.map((variant) => variant.toLowerCase());
      if (
        normalized.some((variant) => auto.has(variant)) &&
        normalized.some((variant) => optIn.has(variant)) &&
        normalized.some((variant) => manual.has(variant)) &&
        !(declaration.name === "FixSafetyV0" && file.endsWith("/omena-checker/src/fix_safety.rs"))
      ) {
        copies.push(`${file}:${declaration.name}`);
      }
    }
  }
  assert.deepEqual(
    copies,
    [],
    `semantic write-safety enum copies are forbidden: ${copies.join(", ")}`,
  );
}

function assertNoTypeScriptSafetyCopies(): void {
  const copies = sourceFiles(
    ["packages", "server", "client"],
    [".ts", ".tsx", ".js", ".cjs", ".mjs"],
  ).filter((file) => {
    const source = read(file);
    return (
      /["']safe["']/u.test(source) &&
      /["']conservative["']/u.test(source) &&
      /["']manualReview["']/u.test(source)
    );
  });
  assert.deepEqual(
    copies,
    [],
    `TypeScript write-safety copies are forbidden: ${copies.join(", ")}`,
  );
}

function enumDeclarations(source: string): { name: string; variants: string[] }[] {
  const code = maskRustCommentsAndLiterals(source);
  const declarations: { name: string; variants: string[] }[] = [];
  for (const match of code.matchAll(/\benum\s+([A-Z][A-Za-z0-9_]*)\s*\{/gu)) {
    const bodyStart = (match.index ?? 0) + match[0].length;
    const bodyEnd = matchingBrace(code, bodyStart - 1, `enum ${match[1]}`);
    const variants = code
      .slice(bodyStart, bodyEnd)
      .split("\n")
      .flatMap((line) => line.match(/^\s*([A-Z][A-Za-z0-9_]*)\s*(?:,|\{|\()/u)?.slice(1) ?? []);
    declarations.push({ name: match[1]!, variants });
  }
  return declarations;
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = source.split("");
  let index = 0;

  const blank = (start: number, end: number): void => {
    for (let cursor = start; cursor < end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") masked[cursor] = " ";
    }
  };

  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end < 0 ? source.length : end;
      blank(index, stop);
      index = stop;
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
        } else {
          index += 1;
        }
      }
      blank(start, index);
      continue;
    }

    const rawPrefix = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (rawPrefix) {
      const start = index;
      const terminator = `"${rawPrefix[1] ?? ""}`;
      index += rawPrefix[0].length;
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
        const current = source[index]!;
        index += 1;
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      blank(start, index);
      continue;
    }

    const characterLiteral = source
      .slice(index)
      .match(/^'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|.)|[^'\\\r\n])'/u);
    if (characterLiteral) {
      const start = index;
      index += characterLiteral[0].length;
      blank(start, index);
      continue;
    }

    index += 1;
  }

  return masked.join("");
}

function extractEnumVariants(source: string, name: string): string[] {
  const declaration = enumDeclarations(source).find((candidate) => candidate.name === name);
  assert.ok(declaration, `missing enum ${name}`);
  return declaration.variants;
}

function stripCfgTestModules(source: string, _label: string): string {
  const marker = /#\[cfg\(test\)\]\s*mod\s+[a-zA-Z0-9_]+\s*\{/gu;
  const match = marker.exec(source);
  if (!match) return source;
  assert.deepEqual(
    topLevelFunctions(source.slice(match.index + match[0].length)),
    [],
    `${_label} must not place production functions after its cfg(test) module`,
  );
  return source.slice(0, match.index);
}

function matchingBrace(source: string, open: number, label = "source"): number {
  let depth = 0;
  let quote: string | null = null;
  let escaped = false;
  let lineComment = false;
  let blockCommentDepth = 0;

  for (let index = open; index < source.length; index += 1) {
    const current = source[index]!;
    const next = source[index + 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
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
    if (quote) {
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === quote) quote = null;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
    } else if (current === "/" && next === "*") {
      blockCommentDepth = 1;
      index += 1;
    } else if (current === '"') {
      quote = current;
    } else if (current === "{") {
      depth += 1;
    } else if (current === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  assert.fail(`${label} has an unterminated brace-delimited block`);
}

function topLevelFunctions(source: string): { name: string; start: number }[] {
  return [...source.matchAll(/^(?:pub(?:\([^)]*\))?\s+)?fn\s+([a-z][a-z0-9_]*)/gmu)].map(
    (match) => ({ name: match[1]!, start: match.index ?? 0 }),
  );
}

function siteIdentity(site: WriteSite): string {
  return [site.path, site.function, site.writeCount, site.classification, site.owner].join("|");
}

function rustSourceFiles(root: string): string[] {
  return sourceFiles([root], [".rs"]).filter(
    (file) => !file.endsWith("/tests.rs") && !file.includes("/tests/"),
  );
}

function sourceFiles(roots: readonly string[], extensions: readonly string[]): string[] {
  return roots
    .flatMap((root) => walk(root))
    .filter((file) => extensions.some((extension) => file.endsWith(extension)))
    .toSorted();
}

function walk(relativeRoot: string): string[] {
  const absoluteRoot = path.join(repoRoot, relativeRoot);
  return readdirSync(absoluteRoot, { withFileTypes: true }).flatMap((entry) => {
    const relativePath = path.posix.join(relativeRoot, entry.name);
    return entry.isDirectory() ? walk(relativePath) : [relativePath];
  });
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}
