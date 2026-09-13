import { readFileSync, writeFileSync, existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { runInNewContext } from "node:vm";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  CENSUS_ROW_IDS,
  assertCensusPopulation,
  assertNoLiteralRowSelection,
  checkerInventory,
} from "../../../scripts/lib/census-instrument-evidence";
import { compilerApi as ts } from "../../../server/engine-core-ts/src/ts-facade";
import type tsTypes from "../../../server/engine-core-ts/src/ts-facade";
import { hasRuntimeEvidence } from "../../../scripts/lib/rust-semver-intent";
import { banGateArgv } from "../../../scripts/lib/rust-write-authority";

const root = path.resolve(__dirname, "../../..");
const tick = String.fromCharCode(96);

describe("census evidence operands", () => {
  it("requires every externally declared row, naming each removal", () => {
    expect(() => assertCensusPopulation(CENSUS_ROW_IDS)).not.toThrow();
    for (const missing of CENSUS_ROW_IDS) {
      expect(() => assertCensusPopulation(CENSUS_ROW_IDS.filter((id) => id !== missing))).toThrow(
        "census row missing " + missing,
      );
    }
    expect(() => assertCensusPopulation([...CENSUS_ROW_IDS, "invented"])).toThrow(
      "unexpected census row invented",
    );
  });

  it.each([
    'row.id == "a1"',
    'row.id === "a1"',
    '"a1" === row.id',
    '(row.id) === ("a1")',
    '["a1"].includes(row.id)',
    'row.id.includes("a1")',
    "row.id === " + tick + "a1" + tick,
    tick + "a1" + tick + " === (row.id)",
    'row["id"] === "a1"',
    'row.expected.refusal === "binding does not exercise"',
    '"binding does not exercise" === (row.expected.refusal)',
    '["binding does not exercise"].includes(row.expected.refusal)',
    "row.expected.refusalPrefix === " + tick + "production reaches test constructor " + tick,
  ])("refuses literal selection through %s", (expression) => {
    expect(() =>
      assertNoLiteralRowSelection(
        "declare const inventory: any; function select(row) { return " +
          expression +
          "; } inventory.s0Rows.forEach(select);",
      ),
    ).toThrow("per-row literal selection is forbidden");
  });

  it("refuses switches and local aliases, while permitting typed mechanism dispatch", () => {
    for (const source of [
      'switch ((row.id)) { case "a1": break; }',
      'const prefix = row.expected.refusalPrefix; if (prefix === "unregistered") {}',
      'const id = (row.id); const alias = id; if ("f" == alias) {}',
      'const id = "a1"; if (row.id === id) {}',
      'const ids = ["a1"]; if (ids.includes(row.id)) {}',
      'const text = "binding does not exercise"; if (row.expected.refusal === text) {}',
      'const id = "a1"; const alias = id; if (alias === row.id) {}',
      'const id = "a1"; function nested(entry) { return entry.id === id; } nested(row);',
      'function nested(entry) { const id = "a1"; { return entry.id === id; } } nested(row);',
    ])
      expect(() =>
        assertNoLiteralRowSelection(
          "declare const inventory: any; for (const row of inventory.s0Rows) { " + source + " }",
        ),
      ).toThrow("per-row literal selection");
    expect(() =>
      assertNoLiteralRowSelection(
        'switch (row.gate.kind) { case "compiler": break; } const x = row.expected.refusal;',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'const id = "a1"; function nested(row, id) { return row.id === id; }',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'function nested(row, id) { return row.id === id; } const id = "a1";',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'const id = "a1"; function nested(row, input) { { const id = input; return row.id === id; } }',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        readFileSync(path.join(root, "scripts/check-rust-census-instrument-s0.ts"), "utf8"),
      ),
    ).not.toThrow();
  });

  it.each([
    'for (const spectralEntry of inventory.s0Rows) { if (spectralEntry.id === "a1") {} }',
    'for (const { id: token } of inventory.s0Rows) { if ("a1" !== token) {} }',
    'let needle = "a1"; for (const record of inventory.s0Rows) { if (record.id === needle) {} }',
    'let needle; needle = "off-menu"; for (const record of inventory.s0Rows) { if (record.id === needle) {} }',
    'const choices = new Set(["off-menu"]); for (const record of inventory.s0Rows) { if (choices.has(record.id)) {} }',
    'const choices = new Map([["off-menu", handler]]); for (const record of inventory.s0Rows) { if (choices.has(record.id)) {} }',
    'const choices = { "off-menu": handler }; for (const record of inventory.s0Rows) { if (record.id in choices) {} }',
    "const choices = { novel: handler }; const alias = choices; for (const record of inventory.s0Rows) { if (record.id in alias) {} }",
    'function choices() { return new Set(["off-menu"]); } for (const record of inventory.s0Rows) { if (choices().has(record.id)) {} }',
    'const choices = [dynamic, "off-menu"]; for (const record of inventory.s0Rows) { if (choices.includes(record.id)) {} }',
    'const options = { needle: "off-menu" }; for (const record of inventory.s0Rows) { if (record.id === options.needle) {} }',
    'const options = ["off-menu"]; const [needle] = options; for (const record of inventory.s0Rows) { if (record.id === needle) {} }',
    'const needle = () => "a1"; for (const record of inventory.s0Rows) { if (record.id === needle()) {} }',
    'function select(entry, needle) { return entry.id === needle; } for (const record of inventory.s0Rows) select(record, "a1");',
    "for (const record of inventory.s0Rows) { const normalized = record.id.toLowerCase(); if (/a/.test(normalized)) {} }",
    "for (const record of inventory.s0Rows) { const copy = `" +
      "${record.id}" +
      '`; if (copy === "a1") {} }',
    'for (const record of inventory.s0Rows) { const box = { nested: { item: record } }; const { nested: { item: alias } } = box; if (alias.id === "a1") {} }',
    'const boxes = inventory.s0Rows.map(entry => ({ payload: entry })); for (const { payload } of boxes) { if (payload.id.startsWith("a")) {} }',
    'for (const record of inventory.s0Rows) { const alias = opaqueIdentity(record); if (alias.id === "a1") {} }',
    'inventory.s0Rows.filter(({ id }) => id.startsWith("a"));',
    'inventory.s0Rows.some(record => record.id.endsWith("1"));',
    'inventory.s0Rows.forEach(record => record.id.indexOf("a"));',
    'inventory.s0Rows.find(record => record.id.lastIndexOf("a"));',
    "inventory.s0Rows.map(record => record.id.match(/a/));",
    "inventory.s0Rows.filter(record => record.id.search(/a/));",
    "inventory.s0Rows.filter(record => /a/.test(record.id));",
    "for (const record of inventory.s0Rows) { dispatch[record.id](); }",
    'const records = inventory.s0Rows; const renamed = records.filter(Boolean); for (const record of renamed) { if (["a1"].includes(record.id)) {} }',
    'const { s0Rows: records } = inventory; for (const record of records) { if ("a1" in record.id) {} }',
    'function select(entry) { return entry.id === "a1"; } inventory.s0Rows.filter(select);',
    'function select(entry) { return entry.id === "a1"; } for (const record of inventory.s0Rows) select(record);',
    'for (const record of inventory.s0Rows) { const { expected: { refusal: message } } = record; if (message === "failure") {} }',
  ])("authority-flow selection refuses %s", (body) => {
    expect(() => assertNoLiteralRowSelection("declare const inventory: any; " + body)).toThrow(
      "per-row literal selection is forbidden",
    );
  });

  it("authority-flow selection preserves unrelated bindings and mechanism dispatch", () => {
    for (const body of [
      'for (const record of inventory.s0Rows) { switch (record.gate.kind) { case "compiler": break; } }',
      "for (const record of inventory.s0Rows) { if (requested.has(record.id)) {} }",
      "const options = { value: external }; for (const record of inventory.s0Rows) { if (record.id === options.value) {} }",
      "const choices = new Set(external); for (const record of inventory.s0Rows) { if (choices.has(record.id)) {} }",
      'for (const record of inventory.s0Rows) { { const record = { id: "other" }; if (record.id === "other") {} } }',
      'function select(row) { return row.id === "unrelated"; }',
      'for (const record of inventory.s0Rows) { const { id } = external; if (id === "unrelated") {} }',
    ])
      expect(() =>
        assertNoLiteralRowSelection("declare const inventory: any; " + body),
      ).not.toThrow();
  });

  it.each(["map", "filter", "find", "some", "every", "forEach"])(
    "binds renamed and destructured selectors in %s callbacks",
    (method) => {
      for (const callback of [
        '(nebulaEntry) => { const alias = nebulaEntry; return alias.id === "off-menu"; }',
        "({ id: needle }) => /off-menu/.test(needle)",
        '({ refusal }) => refusal.includes("off-menu")',
        '(nebulaEntry) => nebulaEntry.refusalPrefix.startsWith("off-menu")',
      ])
        expect(() =>
          assertNoLiteralRowSelection("inventory.s0Rows." + method + "(" + callback + ");"),
        ).toThrow("per-row literal selection is forbidden");
    },
  );

  it("tracks loop aliases and refusal destructuring by their lexical declarations", () => {
    for (const body of [
      'for (const nebulaEntry of inventory.s0Rows) { const alias = nebulaEntry; const { refusal } = alias; if (refusal === "off-menu") {} }',
      'for (const { refusal: message } of inventory.s0Rows) { if (message.endsWith("off-menu")) {} }',
      "for (const { refusalPrefix } of inventory.s0Rows) { dispatch[refusalPrefix](); }",
      'for (const entry of inventory.s0Rows) { { var\n needle = entry.id; } if (needle === "off-menu") {} }',
      'const choose = function inspect(entry) { return entry.id === "off-menu"; }; inventory.s0Rows.some(choose);',
    ])
      expect(() => assertNoLiteralRowSelection(body)).toThrow(
        "per-row literal selection is forbidden",
      );
  });

  it("keeps loop, callback, catch and block shadowing independent of authority bindings", () => {
    for (const body of [
      'const entry = external; for (const entry of inventory.s0Rows) { use(entry.id); } if (entry.id === "unrelated") {}',
      'inventory.s0Rows.forEach(entry => use(entry.id)); others.forEach(entry => entry.id === "unrelated");',
      'for (const entry of inventory.s0Rows) { try {} catch (entry) { if (entry.id === "unrelated") {} } }',
      'for (const entry of inventory.s0Rows) { { const alias = external; if (alias.id === "unrelated") {} } const alias = entry; use(alias.id); }',
      'const entry = external; for (const entry of inventory.s0Rows) { function nested(entry) { return entry.id === "unrelated"; } nested(external); }',
      'const id = "unrelated"; for (const entry of inventory.s0Rows) { if (entry.id === external.id) {} }',
    ])
      expect(() => assertNoLiteralRowSelection(body)).not.toThrow();
  });

  it("keys actual calls and the helper definition by their complete source content", () => {
    const source =
      'function blockBody(x) { return x; }\nassert.match(value, /required/);\nblockBody("x");\n';
    const initial = checkerInventory("checker.ts", source);
    const changed = checkerInventory("checker.ts", source.replace("/required/", "/(?:)/"));
    expect(initial).toHaveLength(3);
    expect(initial.filter((row) => row.kind === "blockBody")).toHaveLength(2);
    expect(changed.find((row) => row.kind === "assert")!.identity).not.toEqual(
      initial.find((row) => row.kind === "assert")!.identity,
    );
    expect(
      checkerInventory("checker.ts", source + '// assert.ok(true); blockBody("decoy");\n'),
    ).toEqual(initial);
  });
});

describe("acquisition population bookkeeping", () => {
  it("prints population operands without asserting their shared-input identity", () => {
    const source = readFileSync(
      path.join(root, "scripts/check-rust-fs-acquisition-census.ts"),
      "utf8",
    );
    const file = ts.createSourceFile("acquisition.ts", source, ts.ScriptTarget.Latest, true);
    const counts = new Set([
      "observedExpressionCount",
      "birthExpressionCount",
      "removedExpressionCount",
      "admittedExpressionCount",
    ]);
    const assertions: string[] = [];
    const visit = (node: tsTypes.Node): void => {
      if (
        ts.isCallExpression(node) &&
        ts.isPropertyAccessExpression(node.expression) &&
        ts.isIdentifier(node.expression.expression) &&
        node.expression.expression.text === "assert"
      ) {
        const operands = (child: tsTypes.Node): void => {
          if (ts.isIdentifier(child) && counts.has(child.text)) assertions.push(child.text);
          ts.forEachChild(child, operands);
        };
        node.arguments.forEach(operands);
      }
      ts.forEachChild(node, visit);
    };
    visit(file);
    expect(assertions).toEqual([]);
    expect(source).toContain('kind: "bookkeeping-only"');
  });
});

describe("semver declaration evidence", () => {
  const needle = "provider_completeness: ProviderCompletenessV1";
  const declared = "pub struct AnalysisPrecisionV1 {\n    " + needle + ",\n}";
  const decoys = [
    "fn from_axes(" + needle + ") {}",
    "/// " + needle,
    'const DECOY: &str = "' + needle + '";',
    "struct DifferentType {\n    " + needle + ",\n}",
  ].join("\n");
  it("requires the field in the certified struct even when every decoy survives", () => {
    expect(hasRuntimeEvidence(declared + "\n" + decoys, needle, "AnalysisPrecisionV1")).toBe(true);
    expect(
      hasRuntimeEvidence(
        declared.replace(needle + ",", "") + "\n" + decoys,
        needle,
        "AnalysisPrecisionV1",
      ),
    ).toBe(false);
    expect(() => hasRuntimeEvidence(declared, needle)).toThrow("struct declaration operand");
  });
  it("does not accept a comment or string transcription as runtime evidence", () => {
    const value = "WorldAssumptionV1::Open";
    expect(hasRuntimeEvidence("// " + value + '\nconst X: &str = "' + value + '";', value)).toBe(
      false,
    );
    expect(hasRuntimeEvidence("let world = " + value + ";", value)).toBe(true);
    expect(
      hasRuntimeEvidence(
        '#[serde(rename = "k-cfa")]\nKLimitedCallSite,',
        '#[serde(rename = "k-cfa")]',
      ),
    ).toBe(true);
  });
});

describe("S0 refusal evidence and child environment", () => {
  const source = readFileSync(
    path.join(root, "scripts/check-rust-census-instrument-s0.ts"),
    "utf8",
  );
  const file = ts.createSourceFile("runner.ts", source, ts.ScriptTarget.Latest, true);
  function harness(directory = root) {
    const journal: { command: { environment: Record<string, string>; stdout: string } }[] = [];
    const functions = [
      "commandFor",
      "executeCommand",
      "validateClippy",
      "writeReceipt",
      "argumentValues",
      "sha256",
      "compareCodePoint",
    ];
    const declarations = file.statements.filter(
      (node) => ts.isFunctionDeclaration(node) && node.name && functions.includes(node.name.text),
    );
    expect(declarations).toHaveLength(functions.length);
    const context = {
      assert,
      spawnSync,
      banGateArgv,
      path,
      createHash,
      writeFileSync,
      mkdirSync,
      repoRoot: directory,
      process,
      normalizedOutput: (value: string) => value,
      safeRelativeFile: (base: string, file: string) => path.join(base, file),
      executionJournal: journal,
      currentAttempt: { kind: "row", id: "fixture", replay: 1 },
    };
    const js = ts.transpileModule(declarations.map((node) => node.getText(file)).join("\n"), {
      compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
    }).outputText;
    const api = runInNewContext(
      js + "\n({commandFor,executeCommand,validateClippy,writeReceipt})",
      context,
    ) as {
      commandFor(
        root: string,
        row: { gate: { kind: "clippy-ban" } },
      ): { executable: string; args: string[]; argv: string[] };
      executeCommand(
        root: string,
        spec: { executable: string; args: string[]; argv: string[] },
      ): { stdout: string; environment: Record<string, string> };
      validateClippy(
        receipt: { exitCode: number; stdout: string; stderr: string },
        expected: { methods: string[] },
      ): string;
      writeReceipt(receipt: unknown, failure?: boolean): void;
    };
    return { api, journal, context };
  }
  const method = "std::fs::write";
  const diagnostic = {
    reason: "compiler-message",
    message: {
      code: { code: "clippy::disallowed_methods" },
      level: "error",
      message: "use of a disallowed method `" + method + "`",
    },
  };
  it("requests JSON diagnostics without changing the canonical clippy gate", () => {
    const { api } = harness();
    const command = api.commandFor(root, { gate: { kind: "clippy-ban" } });
    expect(command.executable).toBe("cargo");
    expect(command.args).toContain("--message-format=json");
    expect(command.args.indexOf("--message-format=json")).toBeLessThan(command.args.indexOf("--"));
    expect(command.args.filter((arg) => arg !== "--message-format=json")).toEqual(
      banGateArgv(root).cargoArgs,
    );
    expect(command.argv).toEqual(["cargo", ...command.args]);
  });
  it("accepts only the fired JSON lint with cargo exit 101", () => {
    const { api } = harness();
    const good = { exitCode: 101, stdout: JSON.stringify(diagnostic), stderr: "" };
    expect(api.validateClippy(good, { methods: [method] })).toBe("clippy:" + method);
    for (const bad of [
      { ...good, exitCode: 1 },
      { ...good, exitCode: 0 },
      { ...good, stdout: "", stderr: diagnostic.message.message },
      { ...good, stdout: JSON.stringify({ ...diagnostic, reason: "build-script-executed" }) },
      {
        ...good,
        stdout: JSON.stringify({
          ...diagnostic,
          message: { ...diagnostic.message, code: { code: "E0425" } },
        }),
      },
      {
        ...good,
        stdout: JSON.stringify({
          ...diagnostic,
          message: { ...diagnostic.message, level: "warning" },
        }),
      },
    ])
      expect(() => api.validateClippy(bad, { methods: [method] })).toThrow();
    expect(() =>
      api.validateClippy(
        { ...good, stdout: "", stderr: "DISTINCT_BUILD_FAILURE_PAYLOAD" },
        { methods: [method] },
      ),
    ).toThrow("DISTINCT_BUILD_FAILURE_PAYLOAD");
  });
  it("records the actual inherited child values and makes environment drift observable", () => {
    const { api, journal } = harness();
    const keys = [
      "RUSTC_WRAPPER",
      "SCCACHE_BUCKET",
      "CARGO_BUILD_JOBS",
      "CARGO_REGISTRIES_FIXTURE_TOKEN",
      "DEVELOPER_DIR",
      "SDKROOT",
      "RUSTUP_TOOLCHAIN",
    ];
    const prior = keys.map((key) => process.env[key]);
    try {
      Object.assign(process.env, {
        RUSTC_WRAPPER: "/tmp/evidence-wrapper",
        SCCACHE_BUCKET: "fixture-bucket",
        CARGO_BUILD_JOBS: "7",
        CARGO_REGISTRIES_FIXTURE_TOKEN: "fixture-secret",
        DEVELOPER_DIR: "/tmp/fixture-developer",
        SDKROOT: "/tmp/fixture-sdk",
        RUSTUP_TOOLCHAIN: "fixture-toolchain",
      });
      const args = [
        "-e",
        'process.stdout.write(JSON.stringify(Object.fromEntries(["RUSTC_WRAPPER","SCCACHE_BUCKET","CARGO_BUILD_JOBS","DEVELOPER_DIR","SDKROOT","RUSTUP_TOOLCHAIN"].map(k=>[k,process.env[k]]))))',
      ];
      const first = api.executeCommand(root, {
        executable: process.execPath,
        args,
        argv: ["node", ...args],
      });
      expect(first.environment).toMatchObject(JSON.parse(first.stdout));
      expect(first.environment.CARGO_REGISTRIES_FIXTURE_TOKEN).toMatch(
        /^<REDACTED_SHA256:[a-f0-9]{64}>$/u,
      );
      process.env.CARGO_BUILD_JOBS = "8";
      const second = api.executeCommand(root, {
        executable: process.execPath,
        args,
        argv: ["node", ...args],
      });
      expect(second.environment).toMatchObject(JSON.parse(second.stdout));
      expect(() =>
        assert.deepEqual(second.environment, first.environment, "clean replay environment drifted"),
      ).toThrow("clean replay environment drifted");
      expect(journal).toHaveLength(2);
    } finally {
      keys.forEach((key, index) => {
        if (prior[index] === undefined) delete process.env[key];
        else process.env[key] = prior[index];
      });
    }
  });
  it("persists the failing command through the actual runner catch and the always-uploaded path", () => {
    const directory = mkdtempSync(path.join(os.tmpdir(), "census-partial-"));
    try {
      const { api, context } = harness(directory);
      const finalTry = file.statements.at(-1)!;
      expect(ts.isTryStatement(finalTry)).toBe(true);
      const js = ts
        .transpileModule(finalTry.getText(file), {
          compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
        })
        .outputText.replaceAll("import.meta.url", '"file:///fixture/runner.ts"');
      expect(() =>
        runInNewContext(js, {
          ...context,
          exports: {},
          main() {
            const args = ["-e", 'process.stderr.write("CHILD_FAILURE_WITNESS");process.exit(101)'];
            const command = api.executeCommand(directory, {
              executable: process.execPath,
              args,
              argv: ["node", ...args],
            });
            api.validateClippy(command as never, { methods: [method] });
          },
          writeReceipt: api.writeReceipt,
          existsSync,
          authorityPath: "missing-authority",
          sha256: (bytes: string) => createHash("sha256").update(bytes).digest("hex"),
          fileURLToPath: () => "fixture",
          readFileSync: () => source,
          receipts: [],
          controlReceipts: [],
        }),
      ).toThrow("CHILD_FAILURE_WITNESS");
      const partial = JSON.parse(
        readFileSync(path.join(directory, ".omena-ci/census-instrument-receipt.json"), "utf8"),
      );
      expect(partial.status).toBe("failed");
      expect(partial.executionJournal[0].command.stderr).toBe("CHILD_FAILURE_WITNESS");
      expect(partial.executionJournal[0].command.exitCode).toBe(101);
      expect(partial.currentAttempt).toMatchObject({ kind: "row", id: "fixture", replay: 1 });
      const workflow = readFileSync(
        path.join(root, ".github/workflows/census-instrument.yml"),
        "utf8",
      );
      expect(workflow).toMatch(
        /if: always\(\)[\s\S]*uses: actions\/upload-artifact@[\s\S]*\.omena-ci\/census-instrument-receipt\.json/u,
      );
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
});
