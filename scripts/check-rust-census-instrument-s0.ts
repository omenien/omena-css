import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { DECLARED_CHECK_GATES } from "../packages/check-orchestrator/src/manifest/declared";
import { banGateArgv, rustNamedFunctions } from "./lib/rust-write-authority";

type Home = "compiler" | "clippy" | "instrument";

type FailureMechanism = "cargo-check" | "deny-clippy" | "node-assertion";

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
  readonly withinFunction?: string;
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
  readonly expected: Expected;
  readonly gate: Gate;
  readonly mutations: readonly Mutation[];
}

interface Authority {
  readonly s0Rows: readonly S0Row[];
  readonly retiredInstrument: {
    readonly path: string;
    readonly baselineSha256: string;
    readonly firstFailureWitness: string;
  };
  readonly countedResidue: readonly {
    readonly identity: string;
    readonly kind: string;
    readonly owner: string;
  }[];
  readonly precision: {
    readonly familyCallSites: readonly RegisteredCallSite[];
    readonly deserializationContainers: readonly {
      readonly identity: string;
      readonly owner: string;
    }[];
    readonly birthFloors: {
      readonly familyCallSites: number;
      readonly deserializationContainers: number;
    };
  };
  readonly baselineWriteSafety: {
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
  readonly argv: readonly string[];
  readonly cwd: ".";
  readonly environment: {
    readonly CARGO_INCREMENTAL: "0";
    readonly CARGO_TARGET_DIR: "<REPO>/rust/target";
    readonly CARGO_TERM_COLOR: "never";
    readonly NO_COLOR: "1";
  };
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
}

interface CommandEvidence {
  readonly argv: readonly string[];
  readonly cwd: ".";
  readonly environment: CommandReceipt["environment"];
  readonly exitCode: number;
  readonly stdoutSha256: string;
  readonly stderrSha256: string;
}

interface RegisteredCallSite {
  readonly crate: string;
  readonly file: string;
  readonly item: string;
  readonly member: string;
  readonly ordinal: number;
}

interface FamilyPopulationReceipt {
  readonly command: CommandReceipt;
  readonly newCallSites: readonly RegisteredCallSite[];
}

interface AuthorityValidationReceipt {
  readonly retiredInstrument: {
    readonly path: string;
    readonly sha256: string;
    readonly assertCount: number;
    readonly blockBodyCount: number;
    readonly firstFailureWitness: string;
    readonly command: CommandEvidence;
  };
  readonly demotion: {
    readonly id: "rust/precision-floor";
    readonly ciTier: "manual";
    readonly cadence: "manual";
    readonly strength: "advisory";
    readonly cadenceSource: "derived";
    readonly strengthSource: "derived";
    readonly manifestErrorCount: 0;
    readonly ciWorkflowDirectReferenceCount: 0;
    readonly listCommand: CommandEvidence;
    readonly doctorCommand: CommandEvidence;
  };
  readonly precision: {
    readonly familyCallSiteCount: number;
    readonly deserializationContainerCount: number;
    readonly command: CommandEvidence;
  };
}

interface RowExecutionReceipt {
  readonly rowId: string;
  readonly replay: number;
  readonly home: Home;
  readonly inputFiles: readonly { readonly path: string; readonly sha256: string }[];
  readonly inputTreeDigest: string;
  readonly familyPopulation: FamilyPopulationReceipt | null;
  readonly command: CommandReceipt;
  readonly observedSignature: string;
  readonly baselineWriteSafety: null | {
    readonly checkerSha256: string;
    readonly verdict: "GREEN" | "RED";
    readonly assertion: string | null;
    readonly command: CommandReceipt;
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authorityPath = path.join(repoRoot, "rust/census-instrument-s0.json");

function sha256(input: string | Buffer): string {
  return createHash("sha256").update(input).digest("hex");
}

function compareCodePoint(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
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

function failureMechanism(gate: Gate): FailureMechanism {
  switch (gate.kind) {
    case "compiler":
      return "cargo-check";
    case "clippy-ban":
      return "deny-clippy";
    case "precision-authority":
    case "fs-acquisition-census":
    case "write-safety":
      return "node-assertion";
  }
}

function acceptedExitCodes(mechanism: FailureMechanism): ReadonlySet<number> {
  switch (mechanism) {
    case "cargo-check":
    case "deny-clippy":
      return new Set([101]);
    case "node-assertion":
      return new Set([1]);
  }
}

function validateExitCode(receipt: CommandReceipt, gate: Gate): void {
  const mechanism = failureMechanism(gate);
  const accepted = acceptedExitCodes(mechanism);
  assert.ok(
    accepted.has(receipt.exitCode),
    `${mechanism} exited ${receipt.exitCode}; accepted ${[...accepted].join(",")}`,
  );
}

function validateAuthority(authority: Authority): void {
  assert.ok(authority.s0Rows.length >= 1, "census row table is empty");
  const authorityIds = authority.s0Rows.map(({ id }) => id).toSorted(compareCodePoint);
  assert.equal(new Set(authorityIds).size, authorityIds.length, "duplicate census row id");
  for (const row of authority.s0Rows) {
    assert.match(row.id, /^[a-z][a-z0-9-]*$/u, `invalid census row id ${row.id}`);
    assert.equal(
      expectedHome(row.expected),
      gateHome(row.gate),
      `expected kind and gate disagree ${row.id}`,
    );
    assert.ok(row.mutations.length >= 1, `injection recipe missing ${row.id}`);
    if (row.gate.kind === "compiler") {
      assert.ok(row.gate.package, `compiler package missing ${row.id}`);
    } else {
      assert.equal(row.gate.package, undefined, `non-compiler package present ${row.id}`);
    }
  }
  assert.ok(authority.retiredInstrument.path.length >= 1, "retired instrument path is empty");
  assert.match(
    authority.retiredInstrument.baselineSha256,
    /^[0-9a-f]{64}$/u,
    "retired instrument digest is invalid",
  );
  assert.ok(
    authority.retiredInstrument.firstFailureWitness.length >= 1,
    "retired instrument witness is empty",
  );
  assert.ok(authority.countedResidue.length >= 1, "counted residue is empty");
  for (const row of authority.countedResidue) {
    assert.ok(row.identity.length >= 1, "counted residue identity is empty");
    assert.ok(row.owner.trim().length >= 1, `counted residue owner missing ${row.identity}`);
  }
  assert.equal(
    new Set(authority.countedResidue.map(({ identity }) => identity)).size,
    authority.countedResidue.length,
    "counted residue identity is duplicated",
  );
  for (const [name, floor] of Object.entries(authority.precision.birthFloors)) {
    assert.ok(Number.isSafeInteger(floor) && floor >= 1, `precision birth floor invalid ${name}`);
  }
}

function needsBaselineWriteSafety(row: S0Row): boolean {
  return (
    row.gate.kind === "clippy-ban" ||
    row.gate.kind === "fs-acquisition-census" ||
    row.gate.kind === "write-safety"
  );
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
        if (mutation.withinFunction) {
          const enclosing = rustNamedFunctions(source, "crate").filter(
            ({ start, end }) => start <= offset && offset < end,
          );
          assert.ok(
            enclosing.some(({ shortName }) => shortName === mutation.withinFunction),
            `replacement is outside function ${mutation.file}#${mutation.withinFunction}`,
          );
        }
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
  const roots = [
    [realpathSync(treeRoot), "<TREE>"],
    [treeRoot, "<TREE>"],
    [realpathSync(repoRoot), "<REPO>"],
    [repoRoot, "<REPO>"],
  ] as const;
  let normalized = value;
  for (const [root, replacement] of [...roots].sort(
    ([left], [right]) => right.length - left.length,
  )) {
    normalized = normalized.replaceAll(root, replacement);
  }
  return normalized.replaceAll(os.homedir(), "<USER_HOME>");
}

function executeCommand(root: string, spec: CommandSpec): CommandReceipt {
  const cargoTargetDir = path.join(repoRoot, "rust/target");
  const result = spawnSync(spec.executable, spec.args, {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_INCREMENTAL: "0",
      CARGO_TARGET_DIR: cargoTargetDir,
      CARGO_TERM_COLOR: "never",
      NO_COLOR: "1",
    },
    maxBuffer: 256 * 1024 * 1024,
  });
  const stdout = normalizedOutput(result.stdout ?? "", root);
  const stderr = normalizedOutput(result.stderr ?? "", root);
  const exitCode = result.status ?? 127;
  return {
    argv: spec.argv,
    cwd: ".",
    environment: {
      CARGO_INCREMENTAL: "0",
      CARGO_TARGET_DIR: "<REPO>/rust/target",
      CARGO_TERM_COLOR: "never",
      NO_COLOR: "1",
    },
    exitCode,
    stdout,
    stderr,
  };
}

function commandEvidence(receipt: CommandReceipt): CommandEvidence {
  return {
    argv: receipt.argv,
    cwd: receipt.cwd,
    environment: receipt.environment,
    exitCode: receipt.exitCode,
    stdoutSha256: sha256(receipt.stdout),
    stderrSha256: sha256(receipt.stderr),
  };
}

function callIdentity(site: RegisteredCallSite): string {
  return [site.crate, site.file, site.item, site.member, String(site.ordinal)].join("|");
}

function familyPopulationReceipt(
  root: string,
  row: S0Row,
  authority: Authority,
): FamilyPopulationReceipt | null {
  const prefix = "refusalPrefix" in row.expected ? row.expected.refusalPrefix : null;
  const isUnregisteredCall = prefix === "unregistered sealed-family call site ";
  const isProductionTestCall = prefix === "production reaches test constructor ";
  if (!isUnregisteredCall && !isProductionTestCall) return null;

  const args = ["--import", "tsx", "scripts/check-rust-precision-authority.ts", "--print-baseline"];
  const command = executeCommand(root, {
    executable: process.execPath,
    args,
    argv: ["node", ...args],
  });
  assert.equal(command.exitCode, 0, "family population derivation failed");
  const population = JSON.parse(command.stdout) as {
    familyCallSites: readonly RegisteredCallSite[];
  };
  assert.ok(Array.isArray(population.familyCallSites), "family population is absent");
  const registered = new Set(authority.precision.familyCallSites.map(callIdentity));
  const newCallSites = population.familyCallSites.filter(
    (site) => !registered.has(callIdentity(site)),
  );
  const relevantCallSites = isProductionTestCall
    ? newCallSites.filter(({ member }) => member === "from_axes_for_tests")
    : newCallSites.filter(({ member }) => member !== "from_axes_for_tests");
  assert.ok(relevantCallSites.length >= 1, "injected family call site is absent");
  const touchedRustFiles = new Set(
    row.mutations.map(({ file }) => file).filter((file) => file.endsWith(".rs")),
  );
  const observedFiles = new Set(relevantCallSites.map(({ file }) => file));
  assert.deepEqual(
    [...observedFiles].toSorted(compareCodePoint),
    [...touchedRustFiles].toSorted(compareCodePoint),
    "injected family call coverage diverged",
  );
  return { command, newCallSites };
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

function validateExpected(receipt: CommandReceipt, expected: Expected, gate: Gate): string {
  validateExitCode(receipt, gate);
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

function checkerInventory(
  checkerPath: string,
  source: string,
): Array<{ readonly identity: string; readonly kind: "assert" | "blockBody" }> {
  const rows: Array<{ identity: string; kind: "assert" | "blockBody" }> = [];
  for (const [kind, pattern] of [
    ["assert", /\bassert(?:\.[A-Za-z_$][\w$]*)?\s*\(/gu],
    ["blockBody", /\bblockBody\s*\(/gu],
  ] as const) {
    for (const match of source.matchAll(pattern)) {
      const line = source.slice(0, match.index).split("\n").length;
      rows.push({ identity: `${checkerPath}:${line}`, kind });
    }
  }
  return rows.toSorted((left, right) =>
    compareCodePoint(`${left.kind}\0${left.identity}`, `${right.kind}\0${right.identity}`),
  );
}

function validateDemotion(): AuthorityValidationReceipt["demotion"] {
  const gateId = "rust/precision-floor" as const;
  const declarations = DECLARED_CHECK_GATES.filter(({ id }) => id === gateId);
  assert.equal(declarations.length, 1, `demotion declared not derived ${gateId}`);
  const declaration = declarations[0]!;
  assert.ok(
    declaration.ciTier === "manual" &&
      typeof declaration.ciReason === "string" &&
      declaration.ciReason.trim().length >= 1 &&
      !Object.hasOwn(declaration, "strength") &&
      !Object.hasOwn(declaration, "cadence") &&
      !Object.hasOwn(declaration, "ciGroup"),
    `demotion declared not derived ${gateId}`,
  );

  const cli = "packages/check-orchestrator/src/cli/main.ts";
  const listArgs = ["--import", "tsx", cli, "list", "--json"];
  const listCommand = executeCommand(repoRoot, {
    executable: process.execPath,
    args: listArgs,
    argv: ["node", ...listArgs],
  });
  assert.equal(listCommand.exitCode, 0, `demotion manifest list failed:\n${listCommand.stderr}`);
  const listed = JSON.parse(listCommand.stdout) as readonly Record<string, unknown>[];
  const lifecycleRows = listed.filter(({ id }) => id === gateId);
  assert.equal(lifecycleRows.length, 1, `demotion declared not derived ${gateId}`);
  const lifecycle = lifecycleRows[0]!;
  assert.ok(
    lifecycle.ciTier === "manual" &&
      lifecycle.cadence === "manual" &&
      lifecycle.strength === "advisory" &&
      lifecycle.cadenceSource === "derived" &&
      lifecycle.strengthSource === "derived",
    `demotion declared not derived ${gateId}`,
  );

  const ciWorkflow = readFileSync(path.join(repoRoot, ".github/workflows/ci.yml"), "utf8");
  const directReferences = [
    ...ciWorkflow.matchAll(/(?:^|[\s"'])rust\/precision-floor(?=$|[\s"'])/gmu),
    ...ciWorkflow.matchAll(/(?:^|[\s"'])check:rust-precision-floor(?=$|[\s"'])/gmu),
  ];
  assert.equal(
    directReferences.length,
    0,
    `demotion declared not derived ${gateId}: ci workflow reference`,
  );

  const doctorArgs = ["--import", "tsx", cli, "doctor", "--json"];
  const doctorCommand = executeCommand(repoRoot, {
    executable: process.execPath,
    args: doctorArgs,
    argv: ["node", ...doctorArgs],
  });
  assert.equal(
    doctorCommand.exitCode,
    0,
    `demotion manifest doctor failed:\n${doctorCommand.stderr}`,
  );
  const doctor = JSON.parse(doctorCommand.stdout) as { readonly errorCount: number };
  assert.equal(doctor.errorCount, 0, `demotion manifest errors ${doctor.errorCount}`);

  return {
    id: gateId,
    ciTier: "manual",
    cadence: "manual",
    strength: "advisory",
    cadenceSource: "derived",
    strengthSource: "derived",
    manifestErrorCount: 0,
    ciWorkflowDirectReferenceCount: 0,
    listCommand: commandEvidence(listCommand),
    doctorCommand: commandEvidence(doctorCommand),
  };
}

function validateCurrentAuthority(authority: Authority): AuthorityValidationReceipt {
  const checker = safeRelativeFile(repoRoot, authority.retiredInstrument.path);
  const checkerBytes = readFileSync(checker);
  const checkerDigest = sha256(checkerBytes);
  assert.equal(
    checkerDigest,
    authority.retiredInstrument.baselineSha256,
    `retired instrument edited ${authority.retiredInstrument.path}`,
  );

  const inventory = checkerInventory(authority.retiredInstrument.path, checkerBytes.toString());
  const declaredInventory = authority.countedResidue
    .filter(({ kind }) => kind === "assert" || kind === "blockBody")
    .map(({ identity, kind }) => ({ identity, kind }))
    .toSorted((left, right) =>
      compareCodePoint(`${left.kind}\0${left.identity}`, `${right.kind}\0${right.identity}`),
    );
  const inventoryKeys = inventory.map(({ identity, kind }) => `${kind}:${identity}`);
  const declaredKeys = declaredInventory.map(({ identity, kind }) => `${kind}:${identity}`);
  const inventorySet = new Set(inventoryKeys);
  const declaredSet = new Set(declaredKeys);
  const missing = inventoryKeys.find((key) => !declaredSet.has(key));
  const extra = declaredKeys.find((key) => !inventorySet.has(key));
  assert.ok(
    !missing && !extra && inventoryKeys.length === declaredKeys.length,
    `counted residue not enumerated ${missing ?? extra ?? "row-count"}`,
  );

  const checkerArgs = ["--import", "tsx", authority.retiredInstrument.path];
  const checkerCommand = executeCommand(repoRoot, {
    executable: process.execPath,
    args: checkerArgs,
    argv: ["node", ...checkerArgs],
  });
  assert.equal(checkerCommand.exitCode, 1, "retired instrument witness did not assert");
  const witness = firstAssertion(checkerCommand);
  assert.equal(
    witness,
    authority.retiredInstrument.firstFailureWitness,
    "retired instrument first failure witness drifted",
  );

  const demotion = validateDemotion();
  const precisionArgs = ["--import", "tsx", "scripts/check-rust-precision-authority.ts"];
  const precisionCommand = executeCommand(repoRoot, {
    executable: process.execPath,
    args: precisionArgs,
    argv: ["node", ...precisionArgs],
  });
  assert.equal(
    precisionCommand.exitCode,
    0,
    `precision authority baseline failed:\n${precisionCommand.stderr}`,
  );
  const precision = JSON.parse(precisionCommand.stdout) as {
    readonly familyCallSiteCount: number;
    readonly deserializationContainerCount: number;
  };
  assert.equal(
    precision.familyCallSiteCount,
    authority.precision.familyCallSites.length,
    "precision family receipt diverged",
  );
  assert.equal(
    precision.deserializationContainerCount,
    authority.precision.deserializationContainers.length,
    "precision container receipt diverged",
  );

  return {
    retiredInstrument: {
      path: authority.retiredInstrument.path,
      sha256: checkerDigest,
      assertCount: inventory.filter(({ kind }) => kind === "assert").length,
      blockBodyCount: inventory.filter(({ kind }) => kind === "blockBody").length,
      firstFailureWitness: witness,
      command: commandEvidence(checkerCommand),
    },
    demotion,
    precision: {
      familyCallSiteCount: precision.familyCallSiteCount,
      deserializationContainerCount: precision.deserializationContainerCount,
      command: commandEvidence(precisionCommand),
    },
  };
}

function expectedStrings(expected: Expected): string[] {
  if ("code" in expected) return [expected.code, expected.type];
  if ("methods" in expected)
    return expected.methods.map((method) => `use of a disallowed method \`${method}\``);
  return ["refusal" in expected ? expected.refusal : expected.refusalPrefix];
}

function baselineReceipt(
  root: string,
  authority: Authority,
  expected: Expected,
): NonNullable<RowExecutionReceipt["baselineWriteSafety"]> {
  const checker = safeRelativeFile(root, authority.baselineWriteSafety.path);
  const current = readFileSync(checker);
  const historical = Buffer.from(
    git(repoRoot, [
      "show",
      `${authority.baselineWriteSafety.revision}:${authority.baselineWriteSafety.path}`,
    ]),
  );
  assert.equal(
    sha256(historical),
    authority.baselineWriteSafety.sha256,
    "baseline checker digest drifted",
  );
  let receipt: CommandReceipt;
  try {
    writeFileSync(checker, historical);
    const args = ["--import", "tsx", authority.baselineWriteSafety.path];
    receipt = executeCommand(root, {
      executable: process.execPath,
      args,
      argv: ["node", ...args],
    });
  } finally {
    writeFileSync(checker, current);
  }
  const assertion = firstAssertion(receipt);
  if (receipt.exitCode !== 0) {
    assert.ok(assertion, "baseline write-safety RED has no machine-recorded assertion");
  }
  for (const expectedText of expectedStrings(expected)) {
    assert.notEqual(
      assertion,
      expectedText,
      "baseline assertion equals the row's own expected text",
    );
  }
  return {
    checkerSha256: authority.baselineWriteSafety.sha256,
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
  replay: number,
  scratchParent: string,
  authority: Authority,
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
    const familyPopulation = familyPopulationReceipt(root, row, authority);
    const command = executeCommand(root, spec);
    const observedSignature = validateExpected(command, row.expected, row.gate);
    assert.deepEqual(inputFiles(root, touched), files, `gate mutated injected inputs ${row.id}`);
    const baselineWriteSafety = needsBaselineWriteSafety(row)
      ? baselineReceipt(root, authority, row.expected)
      : null;
    assert.deepEqual(
      changedPaths(root),
      touched,
      `baseline replay mutated injected inputs ${row.id}`,
    );
    return {
      rowId: row.id,
      replay,
      home: expectedHome(row.expected),
      inputFiles: files,
      inputTreeDigest,
      familyPopulation,
      command,
      observedSignature,
      baselineWriteSafety,
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
const authorityBytes = readFileSync(authorityPath);
const authority = JSON.parse(authorityBytes.toString("utf8")) as Authority;
validateAuthority(authority);
const authorityValidation = validateCurrentAuthority(authority);
if (process.argv.includes("--authority-only")) {
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "rust.census-instrument-authority",
        authorityValidation,
      },
      null,
      2,
    )}\n`,
  );
  process.exit(0);
}
const authorityIds = new Set(authority.s0Rows.map(({ id }) => id));
const requested = new Set(argumentValues("--row"));
for (const id of requested) assert.ok(authorityIds.has(id), `unknown requested row ${id}`);
const selectedRows = authority.s0Rows.filter(({ id }) => requested.size === 0 || requested.has(id));
assert.ok(selectedRows.length >= 1, "no S0 rows selected");
const scratchParent = mkdtempSync(path.join(os.tmpdir(), "omena-census-s0-"));
const receipts: RowExecutionReceipt[] = [];
try {
  for (const row of selectedRows) {
    const first = runRow(row, 1, scratchParent, authority);
    process.stderr.write(`S0 ${row.id} replay 1/2 ${first.observedSignature}\n`);
    const second = runRow(row, 2, scratchParent, authority);
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
    assert.deepEqual(
      second.command.argv,
      first.command.argv,
      `clean replay argv drifted ${row.id}`,
    );
    assert.deepEqual(
      second.command.environment,
      first.command.environment,
      `clean replay environment drifted ${row.id}`,
    );
    assert.equal(
      second.observedSignature,
      first.observedSignature,
      `clean replay signature drifted ${row.id}`,
    );
    assert.deepEqual(
      second.familyPopulation?.newCallSites ?? null,
      first.familyPopulation?.newCallSites ?? null,
      `clean replay family population drifted ${row.id}`,
    );
    assert.deepEqual(
      second.familyPopulation?.command.argv ?? null,
      first.familyPopulation?.command.argv ?? null,
      `clean replay family population argv drifted ${row.id}`,
    );
    assert.deepEqual(
      second.familyPopulation?.command.environment ?? null,
      first.familyPopulation?.command.environment ?? null,
      `clean replay family population environment drifted ${row.id}`,
    );
    assert.deepEqual(
      second.baselineWriteSafety && {
        verdict: second.baselineWriteSafety.verdict,
        assertion: second.baselineWriteSafety.assertion,
        exitCode: second.baselineWriteSafety.command.exitCode,
        argv: second.baselineWriteSafety.command.argv,
        environment: second.baselineWriteSafety.command.environment,
      },
      first.baselineWriteSafety && {
        verdict: first.baselineWriteSafety.verdict,
        assertion: first.baselineWriteSafety.assertion,
        exitCode: first.baselineWriteSafety.command.exitCode,
        argv: first.baselineWriteSafety.command.argv,
        environment: first.baselineWriteSafety.command.environment,
      },
      `clean replay baseline verdict drifted ${row.id}`,
    );
    receipts.push(first, second);
  }
} finally {
  rmSync(scratchParent, { recursive: true, force: true });
}

const primaryReceipts = receipts.filter(({ replay }) => replay === 1);
assert.equal(
  new Set(primaryReceipts.map(({ inputTreeDigest }) => inputTreeDigest)).size,
  selectedRows.length,
  "row injection tree digests are not unique",
);
const executionEvidenceDigests = primaryReceipts.map((receipt) =>
  sha256(
    JSON.stringify({
      inputFiles: receipt.inputFiles,
      inputTreeDigest: receipt.inputTreeDigest,
      argv: receipt.command.argv,
      environment: receipt.command.environment,
      exitCode: receipt.command.exitCode,
      stdout: receipt.command.stdout,
      stderr: receipt.command.stderr,
      observedSignature: receipt.observedSignature,
    }),
  ),
);
assert.equal(
  new Set(executionEvidenceDigests).size,
  selectedRows.length,
  "row execution evidence is not unique",
);

const fullReceipt = {
  schemaVersion: "0",
  product: "rust.census-instrument-s0-receipt",
  authoritySha256: sha256(authorityBytes),
  executorSha256: sha256(readFileSync(fileURLToPath(import.meta.url))),
  bindingRowCount: authority.s0Rows.length,
  executedRowCount: selectedRows.length,
  executionReceiptCount: receipts.length,
  cleanReplayCount: receipts.filter(({ replay }) => replay === 2).length,
  authorityValidation,
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
          newFamilyCallSites:
            first.familyPopulation?.newCallSites.map(callIdentity).toSorted(compareCodePoint) ?? [],
          preGoalVerdict: first.baselineWriteSafety?.verdict ?? null,
          preGoalAssertion: first.baselineWriteSafety?.assertion ?? null,
        };
      }),
    },
    null,
    2,
  )}\n`,
);
