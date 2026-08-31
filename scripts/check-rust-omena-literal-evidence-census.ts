import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

interface LiteralEvidenceAssignment {
  readonly sourcePath: string;
  readonly line: number;
  readonly fieldName: string;
  readonly literal: boolean;
}

interface LiteralEvidenceDisposition {
  readonly sourcePath: string;
  readonly fieldName: string;
  readonly disposition: "computed" | "renamed";
  readonly authority: string;
}

interface LiteralEvidenceCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.literal-evidence-census";
  readonly fieldNameClasses: readonly string[];
  readonly excludedTestSurfaces: readonly string[];
  readonly dispositions: readonly LiteralEvidenceDisposition[];
  readonly productionLiteralAssignments: readonly LiteralEvidenceAssignment[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-literal-evidence-census.json");
const writeMode = process.argv.includes("--write");
const injectIndirectLiteral = process.argv.includes("--inject-indirect-production-literal");
const injectConstantExpression = process.argv.includes("--inject-constant-expression-literal");
const fieldNamePattern = /(?:within_source|verified|parity)/u;

const productionLiteralAssignments: LiteralEvidenceAssignment[] = [];
const excludedTestAssignments: LiteralEvidenceAssignment[] = [];
for (const absoluteSourcePath of rustSourcePaths(path.join(repoRoot, "rust/crates"))) {
  const sourcePath = path.relative(repoRoot, absoluteSourcePath);
  const source = fs.readFileSync(absoluteSourcePath, "utf8");
  const allAssignments = collectLiteralEvidenceAssignments(sourcePath, source);
  if (!isProductionRustSource(sourcePath)) {
    excludedTestAssignments.push(...allAssignments);
    continue;
  }
  const productionAssignments = collectLiteralEvidenceAssignments(
    sourcePath,
    maskCfgTestItems(source),
  );
  productionLiteralAssignments.push(...productionAssignments);
  const productionKeys = new Set(productionAssignments.map(assignmentKey));
  excludedTestAssignments.push(
    ...allAssignments.filter((assignment) => !productionKeys.has(assignmentKey(assignment))),
  );
}
productionLiteralAssignments.sort(compareAssignments);
excludedTestAssignments.sort(compareAssignments);

const injected = collectLiteralEvidenceAssignments(
  "injected/literal-evidence.rs",
  "EvidenceReport {\n  precision_parity: true,\n}\n",
);
assert.deepEqual(
  injected.map(({ fieldName, literal }) => ({ fieldName, literal })),
  [{ fieldName: "precision_parity", literal: true }],
  "literal-evidence predicate must detect a newly introduced parity assertion",
);
const indirectInjected = collectLiteralEvidenceAssignments(
  "injected/indirect-literal-evidence.rs",
  "fn injected() {\n  let arbitrary_name = true;\n  let renamed = arbitrary_name;\n  let _ = EvidenceReport { precision_parity: renamed };\n}\n",
);
assert.deepEqual(
  indirectInjected.map(({ fieldName, literal }) => ({ fieldName, literal })),
  [{ fieldName: "precision_parity", literal: true }],
  "literal-evidence predicate must follow transitive local aliases with arbitrary names",
);
const constantExpressionInjected = collectLiteralEvidenceAssignments(
  "injected/constant-expression-literal-evidence.rs",
  "fn injected(condition: bool) {\n  let _ = EvidenceReport { precision_parity: true || condition };\n  let _ = EvidenceReport { proof_verified: [true][0] };\n}\n",
);
assert.deepEqual(
  constantExpressionInjected.map(({ fieldName, literal }) => ({ fieldName, literal })),
  [
    { fieldName: "precision_parity", literal: true },
    { fieldName: "proof_verified", literal: true },
  ],
  "literal-evidence predicate must detect the enumerated constant-true expression forms",
);
const commentDecoy = collectLiteralEvidenceAssignments(
  "injected/comment-decoy.rs",
  "/// let arbitrary_name = true; EvidenceReport { precision_parity: arbitrary_name }\nfn clean() {}\n",
);
assert.deepEqual(commentDecoy, [], "doc comments must not create literal-evidence assignments");
const cfgTestDecoy = maskCfgTestItems(
  "fn production() {}\n#[cfg(test)]\nmod tests { fn decoy() { let alias = true; let _ = EvidenceReport { precision_parity: alias }; } }\n",
);
assert.deepEqual(
  collectLiteralEvidenceAssignments("injected/cfg-test-decoy.rs", cfgTestDecoy),
  [],
  "cfg(test) attributed items must be excluded from production evidence",
);
if (
  process.env.OMENA_LITERAL_EVIDENCE_TEST_INJECT === "1" ||
  injectIndirectLiteral ||
  injectConstantExpression
) {
  productionLiteralAssignments.push(
    ...(injectConstantExpression
      ? constantExpressionInjected
      : injectIndirectLiteral
        ? indirectInjected
        : injected),
  );
  productionLiteralAssignments.sort(compareAssignments);
}

assert.deepEqual(
  productionLiteralAssignments,
  [],
  "production evidence fields must derive their value instead of receiving direct or aliased boolean literals",
);

const census: LiteralEvidenceCensus = {
  schemaVersion: "0",
  product: "omena.literal-evidence-census",
  fieldNameClasses: ["within_source", "verified", "parity"],
  excludedTestSurfaces: excludedTestAssignments.map(formatAssignment),
  dispositions: [
    {
      sourcePath: "rust/crates/omena-semantic/src/lib.rs",
      fieldName: "all_token_spans_within_source",
      disposition: "computed",
      authority: "ParsedCst token text ranges bounded by sourceByteLen",
    },
    {
      sourcePath: "rust/crates/omena-semantic/src/lib.rs",
      fieldName: "all_node_spans_within_source",
      disposition: "computed",
      authority: "ParsedCst node text ranges bounded by sourceByteLen",
    },
    {
      sourcePath: "rust/crates/omena-streaming-ifds/src/lib.rs",
      fieldName: "precision_parity_with_batch",
      disposition: "renamed",
      authority: "exactReachabilitySelected describes the selected exact SCC traversal",
    },
    {
      sourcePath: "rust/crates/omena-zk-audit/src/arkworks.rs",
      fieldName: "proof_verified",
      disposition: "computed",
      authority: "R1CS witness satisfaction result on the early rejection branch",
    },
    {
      sourcePath: "rust/crates/omena-cli/src/lock.rs",
      fieldName: "verified",
      disposition: "computed",
      authority: "Sigstore verification result after the fail-closed success check",
    },
  ],
  productionLiteralAssignments,
};

const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "literal-evidence census is stale",
  );
}

process.stdout.write(
  `Omena literal evidence census OK: dispositions=${census.dispositions.length} productionLiterals=0\n`,
);

function collectLiteralEvidenceAssignments(
  sourcePath: string,
  source: string,
): LiteralEvidenceAssignment[] {
  const assignments: LiteralEvidenceAssignment[] = [];
  const aliases = new Map<string, boolean>();
  const lexicalSource = maskRustCommentsAndLiterals(source);
  for (const [index, line] of lexicalSource.split("\n").entries()) {
    const bindingPattern =
      /\b(?:let|const|static)\s+(?:mut\s+)?([a-z][a-z0-9_]*)\s*(?::\s*bool\s*)?=\s*([^;]+)\s*;/gu;
    for (const match of line.matchAll(bindingPattern)) {
      const resolved = resolveBooleanValue(match[2], aliases);
      if (resolved !== undefined) aliases.set(match[1], resolved);
    }
    const explicitAssignmentPattern = /\b([a-z][a-z0-9_]*)\s*:\s*([^,}]+)\s*(?=[,}])/gu;
    for (const match of line.matchAll(explicitAssignmentPattern)) {
      if (!fieldNamePattern.test(match[1])) continue;
      const literal = resolveBooleanValue(match[2], aliases);
      if (literal === undefined) continue;
      assignments.push({ sourcePath, line: index + 1, fieldName: match[1], literal });
    }
    const shorthandPattern = /^\s*([a-z][a-z0-9_]*)\s*,\s*$/u;
    const shorthand = shorthandPattern.exec(line);
    if (shorthand && fieldNamePattern.test(shorthand[1])) {
      const literal = aliases.get(shorthand[1]);
      if (literal !== undefined) {
        assignments.push({ sourcePath, line: index + 1, fieldName: shorthand[1], literal });
      }
    }
  }
  return assignments;
}

function resolveBooleanValue(
  value: string,
  aliases: ReadonlyMap<string, boolean>,
): boolean | undefined {
  const normalized = value.trim();
  if (normalized === "true") return true;
  if (normalized === "false") return false;
  if (/^true\s*\|\|/u.test(normalized)) return true;
  if (/^false\s*&&/u.test(normalized)) return false;
  const singletonIndex = /^\[\s*(true|false)\s*\]\s*\[\s*0\s*\]$/u.exec(normalized);
  if (singletonIndex?.[1] === "true") return true;
  if (singletonIndex?.[1] === "false") return false;
  return aliases.get(normalized);
}

function assignmentKey(assignment: LiteralEvidenceAssignment): string {
  return `${assignment.sourcePath}:${assignment.line}:${assignment.fieldName}=${assignment.literal}`;
}

function formatAssignment(assignment: LiteralEvidenceAssignment): string {
  return assignmentKey(assignment);
}

function compareAssignments(
  left: LiteralEvidenceAssignment,
  right: LiteralEvidenceAssignment,
): number {
  if (left.sourcePath < right.sourcePath) return -1;
  if (left.sourcePath > right.sourcePath) return 1;
  return left.line - right.line || left.fieldName.localeCompare(right.fieldName);
}

function isProductionRustSource(sourcePath: string): boolean {
  return (
    sourcePath.endsWith(".rs") &&
    !/(?:^|\/)tests?(?:\/|\.rs$)/u.test(sourcePath) &&
    !sourcePath.endsWith("_test.rs")
  );
}

function maskCfgTestItems(source: string): string {
  const masked = [...source];
  const lexicalSource = maskRustCommentsAndLiterals(source);
  const attribute = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of lexicalSource.matchAll(attribute)) {
    const start = match.index;
    const end = findAttributedRustItemEnd(source, start + match[0].length);
    assert.ok(end > start, "cfg(test) item must have a balanced item boundary");
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n" && masked[index] !== "\r") masked[index] = " ";
    }
  }
  return masked.join("");
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = [...source];
  let index = 0;
  while (index < source.length) {
    if (/\s/u.test(source[index] ?? "")) {
      index += 1;
      continue;
    }
    const end = skipRustTriviaOrLiteral(source, index);
    if (end <= index) {
      index += 1;
      continue;
    }
    for (let cursor = index; cursor < end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") masked[cursor] = " ";
    }
    index = end;
  }
  return masked.join("");
}

function findAttributedRustItemEnd(source: string, start: number): number {
  let index = start;
  while (index < source.length) {
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    if (source[index] === ";") return index + 1;
    if (source[index] === "{") return findBalancedRustBraceEnd(source, index);
    index += 1;
  }
  return source.length;
}

function findBalancedRustBraceEnd(source: string, openingBrace: number): number {
  let depth = 0;
  let index = openingBrace;
  while (index < source.length) {
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index + 1;
    }
    index += 1;
  }
  throw new Error(`unbalanced cfg(test) Rust item near byte ${openingBrace}`);
}

function skipRustTriviaOrLiteral(source: string, index: number): number {
  if (/\s/u.test(source[index] ?? "")) {
    let cursor = index + 1;
    while (cursor < source.length && /\s/u.test(source[cursor] ?? "")) cursor += 1;
    return cursor;
  }
  if (source.startsWith("//", index)) {
    const newline = source.indexOf("\n", index + 2);
    return newline < 0 ? source.length : newline + 1;
  }
  if (source.startsWith("/*", index)) {
    let cursor = index + 2;
    let depth = 1;
    while (cursor < source.length && depth > 0) {
      if (source.startsWith("/*", cursor)) {
        depth += 1;
        cursor += 2;
      } else if (source.startsWith("*/", cursor)) {
        depth -= 1;
        cursor += 2;
      } else cursor += 1;
    }
    return cursor;
  }
  const charQuoteOffset = source.startsWith("b'", index) ? 1 : 0;
  if (source[index + charQuoteOffset] === "'") {
    let cursor = index + charQuoteOffset + 1;
    let escaped = false;
    while (cursor < source.length && cursor - index <= 12 && source[cursor] !== "\n") {
      if (!escaped && source[cursor] === "'") return cursor + 1;
      if (!escaped && source[cursor] === "\\") escaped = true;
      else escaped = false;
      cursor += 1;
    }
  }
  const raw = source.slice(index).match(/^(?:b?r)(#+)?"/u);
  if (raw) {
    const hashes = raw[1] ?? "";
    const terminator = `"${hashes}`;
    const end = source.indexOf(terminator, index + raw[0].length);
    return end < 0 ? source.length : end + terminator.length;
  }
  const quoteOffset = source.startsWith('b"', index) ? 1 : 0;
  if (source[index + quoteOffset] === '"') {
    let cursor = index + quoteOffset + 1;
    while (cursor < source.length) {
      if (source[cursor] === "\\") cursor += 2;
      else if (source[cursor] === '"') return cursor + 1;
      else cursor += 1;
    }
    return source.length;
  }
  return index;
}

function rustSourcePaths(root: string): string[] {
  const paths: string[] = [];
  for (const entry of evidenceScanSurface.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      paths.push(...rustSourcePaths(entryPath));
      continue;
    }
    if (entry.name.endsWith(".rs")) {
      paths.push(entryPath);
    }
  }
  return paths;
}
