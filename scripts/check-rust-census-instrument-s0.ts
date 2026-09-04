import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { banGateArgv } from "./lib/rust-write-authority";

type Home = "compiler" | "clippy" | "instrument";

type Expected =
  | { readonly code: string; readonly type: string }
  | { readonly methods: readonly string[] }
  | { readonly refusal: string }
  | { readonly refusalPrefix: string };

interface Gate {
  readonly kind:
    | "compiler"
    | "clippy-ban"
    | "precision-authority"
    | "fs-acquisition-census"
    | "write-safety";
  readonly package?: string;
}

interface TextMutation {
  readonly kind: "append" | "create";
  readonly file: string;
  readonly text: string;
}

interface ReplaceMutation {
  readonly kind: "replace";
  readonly file: string;
  readonly match: string;
  readonly replacement: string;
  readonly expectedMatches?: number;
  readonly matchIndex?: number;
}

interface JsonSetMutation {
  readonly kind: "json-set";
  readonly file: string;
  readonly valuePath: readonly string[];
  readonly value: unknown;
}

interface JsonSetMatchingMutation {
  readonly kind: "json-set-matching";
  readonly file: string;
  readonly arrayPath: readonly string[];
  readonly match: Readonly<Record<string, unknown>>;
  readonly valuePath: readonly string[];
  readonly value: unknown;
}

type Mutation = TextMutation | ReplaceMutation | JsonSetMutation | JsonSetMatchingMutation;

interface S0Row {
  readonly id: string;
  readonly home: Home;
  readonly expected: Expected;
  readonly gate: Gate;
  readonly mutations: readonly Mutation[];
}

interface Authority {
  readonly bindingRowCount: number;
  readonly s0Rows: readonly S0Row[];
  readonly preGoalWriteSafety: {
    readonly revision: string;
    readonly path: string;
    readonly sha256: string;
  };
}

interface CommandSpec {
  readonly executable: string;
  readonly args: readonly string[];
  readonly argv: readonly string[];
}

interface CommandReceipt {
  readonly executionId: string;
  readonly argv: readonly string[];
  readonly cwd: ".";
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
}

interface RowExecutionReceipt {
  readonly rowId: string;
  readonly replay: number;
  readonly home: Home;
  readonly inputFiles: readonly { readonly path: string; readonly sha256: string }[];
  readonly inputTreeDigest: string;
  readonly command: CommandReceipt;
  readonly observedSignature: string;
  readonly preGoalWriteSafety: null | {
    readonly checkerSha256: string;
    readonly verdict: "GREEN" | "RED";
    readonly assertion: string | null;
    readonly command: CommandReceipt;
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authorityPath = path.join(repoRoot, "rust/census-instrument-s0.json");

// Reviewer-owned table clause, transcribed independently from the census JSON.
// The parser below derives both the exact row set and each expected home/kind.
const REVIEWED_BINDING_CLAUSE = `
a1|instrument|prefix-space|unregistered sealed-family call site
a2|instrument|prefix-space|production reaches test constructor
b1|compiler|compiler|E0116,AnalysisPrecisionV1
b2|compiler|compiler|E0451,AnalysisPrecisionV1
c|compiler|compiler|E0616,AnalysisPrecisionV1
d|compiler|compiler|E0451,AnalysisPrecisionV1
e|compiler|compiler|E0451,AnalysisPrecisionV1
f|instrument|prefix-space|unregistered sealed-family call site
g1|instrument|exact|binding does not exercise sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_global_class_fallthrough_diagnostic:1
g2|instrument|exact|gated exclusion unverified-source-reference not exercised by missing-selector-context-drift
s1|instrument|exact|unregistered deserialization container omena-cli::crate::diagnostics::DiagnosticPrecisionEnvelopeV0
s2|instrument|exact|unregistered deserialization container omena-query::crate::style::module_interface::FlattenedPrecisionEnvelopeV0
s3|instrument|exact|unregistered deserialization container omena-query::crate::style::stylesheet_evaluation::PrecisionWireBundleV0
j|clippy|methods|std::fs::write
k|clippy|methods|std::fs::write
l|clippy|methods|std::fs::write
m|clippy|methods|std::fs::write
n1|clippy|methods|std::fs::remove_dir,std::fs::DirBuilder::create
n2|instrument|prefix|unregistered acquisition site omena-bridge::
o|instrument|prefix-space|destination class not derived from a sanctioned-module call
p|clippy|methods|std::fs::write
q|clippy|methods|std::fs::write
t1|instrument|prefix-space|suppression above fn granularity
t2|instrument|prefix-space|suppression below fn granularity
t3|instrument|exact|crate omena-lsp-server does not inherit the workspace lint table
`;

const REVIEWED_WRITE_ROW_CLAUSE = "j k l m n1 n2 o p q t1 t2 t3";

function sha256(input: string | Buffer): string {
  return createHash("sha256").update(input).digest("hex");
}

function compareCodePoint(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function bindingRows(): Map<string, { home: Home; expected: Expected }> {
  const rows = new Map<string, { home: Home; expected: Expected }>();
  for (const line of REVIEWED_BINDING_CLAUSE.trim().split("\n")) {
    const [id, rawHome, mode, payload] = line.split("|");
    assert.ok(
      id && rawHome && mode && payload !== undefined,
      `binding clause row malformed ${line}`,
    );
    assert.ok(
      rawHome === "compiler" || rawHome === "clippy" || rawHome === "instrument",
      `binding clause home malformed ${line}`,
    );
    let expected: Expected;
    switch (mode) {
      case "compiler": {
        const [code, type] = payload.split(",");
        assert.ok(code && type, `binding compiler expectation malformed ${line}`);
        expected = { code, type };
        break;
      }
      case "methods":
        expected = { methods: payload.split(",") };
        break;
      case "exact":
        expected = { refusal: payload };
        break;
      case "prefix":
        expected = { refusalPrefix: payload };
        break;
      case "prefix-space":
        expected = { refusalPrefix: `${payload} ` };
        break;
      default:
        assert.fail(`binding expectation mode malformed ${line}`);
    }
    assert.ok(!rows.has(id), `duplicate binding clause row ${id}`);
    rows.set(id, { home: rawHome, expected });
  }
  return rows;
}

function expectedHome(expected: Expected): Home {
  if ("code" in expected) return "compiler";
  if ("methods" in expected) return "clippy";
  return "instrument";
}

function gateHome(gate: Gate): Home {
  if (gate.kind === "compiler") return "compiler";
  if (gate.kind === "clippy-ban") return "clippy";
  return "instrument";
}

function validateAuthority(authority: Authority): Map<string, { home: Home; expected: Expected }> {
  const bindings = bindingRows();
  assert.equal(bindings.size, 25, "binding clause row count changed");
  assert.equal(authority.bindingRowCount, bindings.size, "binding row count diverged");
  assert.equal(authority.s0Rows.length, bindings.size, "census row count diverged");
  const authorityIds = authority.s0Rows.map(({ id }) => id).toSorted(compareCodePoint);
  assert.deepEqual(
    authorityIds,
    [...bindings.keys()].toSorted(compareCodePoint),
    "census row ids diverged",
  );
  assert.equal(new Set(authorityIds).size, authorityIds.length, "duplicate census row id");
  for (const row of authority.s0Rows) {
    const binding = bindings.get(row.id);
    assert.ok(binding, `binding row missing ${row.id}`);
    assert.equal(row.home, expectedHome(binding.expected), `expected home diverged ${row.id}`);
    assert.equal(row.home, binding.home, `declared home diverged ${row.id}`);
    assert.equal(row.home, gateHome(row.gate), `gate home diverged ${row.id}`);
    assert.deepEqual(row.expected, binding.expected, `expected kind diverged ${row.id}`);
    assert.ok(row.mutations.length >= 1, `injection recipe missing ${row.id}`);
    if (row.gate.kind === "compiler") {
      assert.ok(row.gate.package, `compiler package missing ${row.id}`);
    } else {
      assert.equal(row.gate.package, undefined, `non-compiler package present ${row.id}`);
    }
  }
  const writeRows = REVIEWED_WRITE_ROW_CLAUSE.split(/\s+/u).filter(Boolean);
  assert.equal(writeRows.length, 12, "write-row binding count changed");
  assert.equal(new Set(writeRows).size, writeRows.length, "duplicate write-row binding");
  for (const id of writeRows) assert.ok(bindings.has(id), `write-row binding missing ${id}`);
  return bindings;
}

function assertNoPerRowControlFlow(): void {
  const source = readFileSync(fileURLToPath(import.meta.url), "utf8");
  const literalBranch = new RegExp(
    "\\b(?:if|else\\s+if)\\s*\\([^\\n]*(?:row\\s*\\.\\s*id|rowId)\\s*={2,3}\\s*[\"']",
    "u",
  );
  const literalSwitch = new RegExp("\\bswitch\\s*\\(\\s*(?:row\\s*\\.\\s*id|rowId)\\s*\\)", "u");
  assert.doesNotMatch(source, literalBranch, "per-row literal branch is forbidden");
  assert.doesNotMatch(source, literalSwitch, "per-row switch is forbidden");
}

function safeRelativeFile(root: string, relative: string): string {
  assert.ok(
    relative.length > 0 && !path.isAbsolute(relative),
    `injection path is not relative ${relative}`,
  );
  const absolute = path.resolve(root, relative);
  assert.ok(absolute.startsWith(`${root}${path.sep}`), `injection path escapes tree ${relative}`);
  return absolute;
}

function jsonValueAt(root: unknown, segments: readonly string[]): unknown {
  let current = root;
  for (const segment of segments) {
    assert.ok(
      current && typeof current === "object",
      `JSON path is not an object ${segments.join("/")}`,
    );
    current = (current as Record<string, unknown>)[segment];
  }
  return current;
}

function setJsonValue(root: unknown, segments: readonly string[], value: unknown): void {
  assert.ok(segments.length >= 1, "JSON value path is empty");
  let current = root;
  for (const segment of segments.slice(0, -1)) {
    assert.ok(
      current && typeof current === "object",
      `JSON path is not an object ${segments.join("/")}`,
    );
    const record = current as Record<string, unknown>;
    const next = record[segment] ?? {};
    assert.ok(next && typeof next === "object", `JSON path is not an object ${segments.join("/")}`);
    record[segment] = next;
    current = next;
  }
  assert.ok(current && typeof current === "object");
  (current as Record<string, unknown>)[segments.at(-1)!] = value;
}

function recordMatches(value: unknown, expected: Readonly<Record<string, unknown>>): boolean {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return Object.entries(expected).every(
    ([key, expectedValue]) => JSON.stringify(record[key]) === JSON.stringify(expectedValue),
  );
}

function matchOffsets(source: string, needle: string): number[] {
  assert.ok(needle.length >= 1, "replacement match is empty");
  const offsets: number[] = [];
  let cursor = 0;
  while (cursor <= source.length - needle.length) {
    const offset = source.indexOf(needle, cursor);
    if (offset < 0) break;
    offsets.push(offset);
    cursor = offset + needle.length;
  }
  return offsets;
}

function applyMutations(root: string, mutations: readonly Mutation[]): string[] {
  const touched = new Set<string>();
  for (const mutation of mutations) {
    const file = safeRelativeFile(root, mutation.file);
    switch (mutation.kind) {
      case "append": {
        assert.ok(existsSync(file), `append target missing ${mutation.file}`);
        writeFileSync(file, readFileSync(file, "utf8") + mutation.text);
        break;
      }
      case "create": {
        assert.ok(!existsSync(file), `create target already exists ${mutation.file}`);
        mkdirSync(path.dirname(file), { recursive: true });
        writeFileSync(file, mutation.text);
        break;
      }
      case "replace": {
        const source = readFileSync(file, "utf8");
        const offsets = matchOffsets(source, mutation.match);
        assert.equal(
          offsets.length,
          mutation.expectedMatches ?? 1,
          `replacement match count changed ${mutation.file}`,
        );
        const selected = mutation.matchIndex ?? 0;
        assert.ok(
          selected >= 0 && selected < offsets.length,
          `replacement index invalid ${mutation.file}`,
        );
        const offset = offsets[selected]!;
        writeFileSync(
          file,
          source.slice(0, offset) +
            mutation.replacement +
            source.slice(offset + mutation.match.length),
        );
        break;
      }
      case "json-set": {
        const value = JSON.parse(readFileSync(file, "utf8")) as unknown;
        setJsonValue(value, mutation.valuePath, mutation.value);
        writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
        break;
      }
      case "json-set-matching": {
        const value = JSON.parse(readFileSync(file, "utf8")) as unknown;
        const collection = jsonValueAt(value, mutation.arrayPath);
        assert.ok(Array.isArray(collection), `JSON match path is not an array ${mutation.file}`);
        const matches = collection.filter((entry) => recordMatches(entry, mutation.match));
        assert.equal(matches.length, 1, `JSON matching row count changed ${mutation.file}`);
        setJsonValue(matches[0], mutation.valuePath, mutation.value);
        writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
        break;
      }
    }
    touched.add(mutation.file);
  }
  return [...touched].toSorted(compareCodePoint);
}

function git(root: string, args: readonly string[]): string {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  });
  assert.equal(result.status, 0, `git ${args.join(" ")} failed:\n${result.stderr}`);
  return result.stdout;
}

function changedPaths(root: string): string[] {
  return git(root, ["status", "--porcelain", "--untracked-files=all"])
    .split("\n")
    .filter(Boolean)
    .map((line) => line.slice(3).split(" -> ").at(-1)!)
    .toSorted(compareCodePoint);
}

function inputFiles(
  root: string,
  touched: readonly string[],
): Array<{ path: string; sha256: string }> {
  return touched.map((relative) => ({
    path: relative,
    sha256: sha256(readFileSync(safeRelativeFile(root, relative))),
  }));
}

function commandFor(root: string, row: S0Row): CommandSpec {
  switch (row.gate.kind) {
    case "compiler": {
      assert.ok(row.gate.package);
      const args = [
        "check",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        row.gate.package,
        "--lib",
        "--message-format=json",
      ];
      return { executable: "cargo", args, argv: ["cargo", ...args] };
    }
    case "clippy-ban": {
      const args = [...banGateArgv(root).cargoArgs];
      return { executable: "cargo", args, argv: ["cargo", ...args] };
    }
    case "precision-authority": {
      const args = ["--import", "tsx", "scripts/check-rust-precision-authority.ts"];
      return { executable: process.execPath, args, argv: ["node", ...args] };
    }
    case "fs-acquisition-census": {
      const args = ["--import", "tsx", "scripts/check-rust-fs-acquisition-census.ts"];
      return { executable: process.execPath, args, argv: ["node", ...args] };
    }
    case "write-safety": {
      const args = ["--import", "tsx", "scripts/check-rust-omena-write-safety.ts"];
      return { executable: process.execPath, args, argv: ["node", ...args] };
    }
  }
}

function normalizedOutput(value: string, treeRoot: string): string {
  return value.replaceAll(treeRoot, "<TREE>").replaceAll(repoRoot, "<REPO>");
}

function executeCommand(root: string, spec: CommandSpec, inputDigest: string): CommandReceipt {
  const result = spawnSync(spec.executable, spec.args, {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_INCREMENTAL: "0",
      CARGO_TARGET_DIR: path.join(repoRoot, "rust/target"),
      CARGO_TERM_COLOR: "never",
      NO_COLOR: "1",
    },
    maxBuffer: 256 * 1024 * 1024,
  });
  const stdout = normalizedOutput(result.stdout ?? "", root);
  const stderr = normalizedOutput(result.stderr ?? "", root);
  const exitCode = result.status ?? 127;
  return {
    executionId: sha256(
      `${JSON.stringify(spec.argv)}\0${inputDigest}\0${exitCode}\0${stdout}\0${stderr}`,
    ),
    argv: spec.argv,
    cwd: ".",
    exitCode,
    stdout,
    stderr,
  };
}

function validateCompiler(
  receipt: CommandReceipt,
  expected: Extract<Expected, { code: string }>,
): string {
  const diagnostics = receipt.stdout
    .split("\n")
    .filter(Boolean)
    .flatMap((line) => {
      try {
        return [JSON.parse(line) as Record<string, unknown>];
      } catch {
        return [];
      }
    })
    .filter((entry) => entry.reason === "compiler-message")
    .map((entry) => entry.message as Record<string, unknown>)
    .filter((message) => {
      const code = message.code as { code?: string } | null;
      return code?.code === expected.code && JSON.stringify(message).includes(expected.type);
    });
  assert.ok(receipt.exitCode !== 0, `compiler row unexpectedly compiled ${expected.code}`);
  assert.ok(
    diagnostics.length >= 1,
    `compiler diagnostic absent ${expected.code} ${expected.type}`,
  );
  return `${expected.code}:${expected.type}`;
}

function validateClippy(
  receipt: CommandReceipt,
  expected: Extract<Expected, { methods: readonly string[] }>,
): string {
  assert.ok(receipt.exitCode !== 0, "clippy row unexpectedly passed");
  const output = `${receipt.stdout}\n${receipt.stderr}`;
  for (const method of expected.methods) {
    assert.ok(
      output.includes(`use of a disallowed method \`${method}\``),
      `clippy DefId diagnostic absent ${method}`,
    );
  }
  return expected.methods.map((method) => `clippy:${method}`).join(",");
}

function validateInstrument(
  receipt: CommandReceipt,
  expected: Exclude<Expected, { code: string } | { methods: readonly string[] }>,
): string {
  assert.ok(receipt.exitCode !== 0, "instrument row unexpectedly passed");
  const output = `${receipt.stdout}\n${receipt.stderr}`;
  if ("refusal" in expected) {
    assert.ok(output.includes(expected.refusal), `instrument refusal absent ${expected.refusal}`);
    return expected.refusal;
  }
  const line = output.split("\n").find((candidate) => candidate.includes(expected.refusalPrefix));
  assert.ok(
    line,
    `instrument refusal prefix absent ${expected.refusalPrefix}\n${output.slice(-4_000)}`,
  );
  return line.slice(line.indexOf(expected.refusalPrefix)).trim();
}

function validateExpected(receipt: CommandReceipt, expected: Expected): string {
  if ("code" in expected) return validateCompiler(receipt, expected);
  if ("methods" in expected) return validateClippy(receipt, expected);
  return validateInstrument(receipt, expected);
}

function firstAssertion(receipt: CommandReceipt): string | null {
  if (receipt.exitCode === 0) return null;
  const lines = `${receipt.stderr}\n${receipt.stdout}`.split("\n");
  const index = lines.findIndex((line) => /AssertionError(?:\s+\[[^\]]+\])?:/u.test(line));
  if (index < 0) return null;
  const line = lines[index]!;
  const inline = line.slice(line.indexOf(":") + 1).trim();
  if (inline) return inline;
  return (
    lines
      .slice(index + 1)
      .find((candidate) => candidate.trim().length > 0)
      ?.trim() ?? null
  );
}

function expectedStrings(expected: Expected): string[] {
  if ("code" in expected) return [expected.code, expected.type];
  if ("methods" in expected)
    return expected.methods.map((method) => `use of a disallowed method \`${method}\``);
  return ["refusal" in expected ? expected.refusal : expected.refusalPrefix];
}

function preGoalReceipt(
  root: string,
  authority: Authority,
  expected: Expected,
  inputDigest: string,
): NonNullable<RowExecutionReceipt["preGoalWriteSafety"]> {
  const checker = safeRelativeFile(root, authority.preGoalWriteSafety.path);
  const current = readFileSync(checker);
  const historical = Buffer.from(
    git(repoRoot, [
      "show",
      `${authority.preGoalWriteSafety.revision}:${authority.preGoalWriteSafety.path}`,
    ]),
  );
  assert.equal(
    sha256(historical),
    authority.preGoalWriteSafety.sha256,
    "pre-goal checker digest drifted",
  );
  let receipt: CommandReceipt;
  try {
    writeFileSync(checker, historical);
    const args = ["--import", "tsx", authority.preGoalWriteSafety.path];
    receipt = executeCommand(
      root,
      { executable: process.execPath, args, argv: ["node", ...args] },
      sha256(`${inputDigest}\0${authority.preGoalWriteSafety.sha256}`),
    );
  } finally {
    writeFileSync(checker, current);
  }
  const assertion = firstAssertion(receipt);
  if (receipt.exitCode !== 0) {
    assert.ok(assertion, "pre-goal write-safety RED has no machine-recorded assertion");
  }
  for (const expectedText of expectedStrings(expected)) {
    assert.notEqual(
      assertion,
      expectedText,
      "pre-goal assertion equals the row's own expected text",
    );
  }
  return {
    checkerSha256: authority.preGoalWriteSafety.sha256,
    verdict: receipt.exitCode === 0 ? "GREEN" : "RED",
    assertion,
    command: receipt,
  };
}

function treeDigest(
  files: readonly { path: string; sha256: string }[],
  argv: readonly string[],
): string {
  return sha256(`${JSON.stringify(files)}\0${JSON.stringify(argv)}`);
}

function runRow(
  row: S0Row,
  expected: Expected,
  replay: number,
  scratchParent: string,
  authority: Authority,
  writeRows: ReadonlySet<string>,
): RowExecutionReceipt {
  const root = path.join(scratchParent, `${String(replay).padStart(2, "0")}-${row.id}`);
  git(repoRoot, ["worktree", "add", "--detach", "--quiet", root, "HEAD"]);
  try {
    assert.deepEqual(changedPaths(root), [], `scratch baseline is dirty ${row.id}`);
    const nodeModules = path.join(repoRoot, "node_modules");
    assert.ok(existsSync(nodeModules), "workspace node_modules is unavailable");
    symlinkSync(
      nodeModules,
      path.join(root, "node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
    const touched = applyMutations(root, row.mutations);
    assert.deepEqual(changedPaths(root), touched, `injected file set diverged ${row.id}`);
    const files = inputFiles(root, touched);
    const spec = commandFor(root, row);
    const inputTreeDigest = treeDigest(files, spec.argv);
    const command = executeCommand(root, spec, inputTreeDigest);
    const observedSignature = validateExpected(command, expected);
    assert.deepEqual(inputFiles(root, touched), files, `gate mutated injected inputs ${row.id}`);
    const preGoalWriteSafety = writeRows.has(row.id)
      ? preGoalReceipt(root, authority, expected, inputTreeDigest)
      : null;
    assert.deepEqual(
      changedPaths(root),
      touched,
      `pre-goal replay mutated injected inputs ${row.id}`,
    );
    return {
      rowId: row.id,
      replay,
      home: row.home,
      inputFiles: files,
      inputTreeDigest,
      command,
      observedSignature,
      preGoalWriteSafety,
    };
  } finally {
    git(repoRoot, ["worktree", "remove", "--force", root]);
  }
}

function argumentValues(name: string): string[] {
  const values: string[] = [];
  for (let index = 2; index < process.argv.length; index += 1) {
    if (process.argv[index] !== name) continue;
    const value = process.argv[index + 1];
    assert.ok(value && !value.startsWith("--"), `${name} requires a value`);
    values.push(value);
    index += 1;
  }
  return values;
}

assertNoPerRowControlFlow();
const authority = JSON.parse(readFileSync(authorityPath, "utf8")) as Authority;
const bindings = validateAuthority(authority);
const requested = new Set(argumentValues("--row"));
for (const id of requested) assert.ok(bindings.has(id), `unknown requested row ${id}`);
const selectedRows = authority.s0Rows.filter(({ id }) => requested.size === 0 || requested.has(id));
assert.ok(selectedRows.length >= 1, "no S0 rows selected");
const writeRows = new Set(REVIEWED_WRITE_ROW_CLAUSE.split(/\s+/u).filter(Boolean));
const scratchParent = mkdtempSync(path.join(os.tmpdir(), "omena-census-s0-"));
const receipts: RowExecutionReceipt[] = [];
try {
  for (const row of selectedRows) {
    const expected = bindings.get(row.id)!.expected;
    const first = runRow(row, expected, 1, scratchParent, authority, writeRows);
    process.stderr.write(`S0 ${row.id} replay 1/2 ${first.observedSignature}\n`);
    const second = runRow(row, expected, 2, scratchParent, authority, writeRows);
    process.stderr.write(`S0 ${row.id} replay 2/2 ${second.observedSignature}\n`);
    assert.equal(
      second.inputTreeDigest,
      first.inputTreeDigest,
      `clean replay input drifted ${row.id}`,
    );
    assert.equal(
      second.command.exitCode,
      first.command.exitCode,
      `clean replay exit drifted ${row.id}`,
    );
    assert.equal(
      second.observedSignature,
      first.observedSignature,
      `clean replay signature drifted ${row.id}`,
    );
    assert.deepEqual(
      second.preGoalWriteSafety && {
        verdict: second.preGoalWriteSafety.verdict,
        assertion: second.preGoalWriteSafety.assertion,
        exitCode: second.preGoalWriteSafety.command.exitCode,
      },
      first.preGoalWriteSafety && {
        verdict: first.preGoalWriteSafety.verdict,
        assertion: first.preGoalWriteSafety.assertion,
        exitCode: first.preGoalWriteSafety.command.exitCode,
      },
      `clean replay pre-goal verdict drifted ${row.id}`,
    );
    receipts.push(first, second);
  }
} finally {
  rmSync(scratchParent, { recursive: true, force: true });
}

const fullReceipt = {
  schemaVersion: "0",
  product: "rust.census-instrument-s0-receipt",
  bindingRowCount: bindings.size,
  executedRowCount: selectedRows.length,
  executionReceiptCount: receipts.length,
  cleanReplayCount: receipts.filter(({ replay }) => replay === 2).length,
  rows: receipts,
};
const writeTargets = argumentValues("--write");
assert.ok(writeTargets.length <= 1, "--write may be specified at most once");
if (writeTargets[0]) {
  const target = safeRelativeFile(repoRoot, writeTargets[0]);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, `${JSON.stringify(fullReceipt, null, 2)}\n`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: fullReceipt.schemaVersion,
      product: fullReceipt.product,
      bindingRowCount: fullReceipt.bindingRowCount,
      executedRowCount: fullReceipt.executedRowCount,
      executionReceiptCount: fullReceipt.executionReceiptCount,
      cleanReplayCount: fullReceipt.cleanReplayCount,
      receiptSha256: sha256(JSON.stringify(fullReceipt)),
      rows: selectedRows.map(({ id }) => {
        const first = receipts.find((receipt) => receipt.rowId === id && receipt.replay === 1)!;
        return {
          id,
          home: first.home,
          inputTreeDigest: first.inputTreeDigest,
          observedSignature: first.observedSignature,
          preGoalVerdict: first.preGoalWriteSafety?.verdict ?? null,
          preGoalAssertion: first.preGoalWriteSafety?.assertion ?? null,
        };
      }),
    },
    null,
    2,
  )}\n`,
);
