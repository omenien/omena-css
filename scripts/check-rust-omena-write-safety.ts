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

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const checkerRoot = "rust/crates/omena-checker/src";
const cliRoot = "rust/crates/omena-cli/src";
const manifestPath = "rust/crates/omena-cli/write-safety-census.json";
const manifest = readJson<WriteSafetyManifest>(manifestPath);
const fixSafetySource = read("rust/crates/omena-checker/src/fix_safety.rs");
const writeGateSource = read(manifest.sourceMutationGate.path);
const queryRunnerSource = read("rust/crates/omena-query-transform-runner/src/lib.rs");
const queryFacadeSource = read("rust/crates/omena-query/src/lib.rs");
const productionWritePrimitive =
  /\b(?:std::)?fs::write\s*\(|\bFile::create\s*\(|\bOpenOptions::new\s*\(|\.write_all\s*\(/gu;

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

  for (const file of rustSourceFiles(cliRoot)) {
    const source = stripCfgTestModules(read(file), file);
    const functions = topLevelFunctions(source);
    for (const match of source.matchAll(productionWritePrimitive)) {
      const offset = match.index ?? -1;
      const owner = functions.findLast(({ start }) => start < offset);
      assert.ok(owner, `${file} contains fs::write outside a named function`);
      const key = `${file}#${owner.name}`;
      const current = derived.get(key) ?? { path: file, function: owner.name, writeCount: 0 };
      current.writeCount += 1;
      derived.set(key, current);
    }
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
  const declarations: { name: string; variants: string[] }[] = [];
  for (const match of source.matchAll(/\benum\s+([A-Z][A-Za-z0-9_]*)\s*\{/gu)) {
    const bodyStart = (match.index ?? 0) + match[0].length;
    const bodyEnd = matchingBrace(source, bodyStart - 1);
    const variants = source
      .slice(bodyStart, bodyEnd)
      .split("\n")
      .flatMap((line) => line.match(/^\s*([A-Z][A-Za-z0-9_]*)\s*(?:,|\{|\()/u)?.slice(1) ?? []);
    declarations.push({ name: match[1]!, variants });
  }
  return declarations;
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
