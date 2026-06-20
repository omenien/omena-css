import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

// Release gate for the SIF resolve loop (omenien/rfcs #39).
//
// The static SIF generator, the lock author/verify chain, and the external-SIF
// resolution path all ship through the real `omena-cli` surface, but no
// release-boundary check proved they compose into a usable end-to-end loop: a
// user authoring a SIF for a previously-unresolved external Sass `@use` must see
// the external missing-symbol diagnostic disappear, and that authored lock must
// frozen-verify against the generated artifact.
//
// This gate exercises the chain through the shipped CLI commands only — no editor
// transport, no resolver generator — and asserts the decisive behavioral signal:
//   * baseline (external sif mode, no SIF): the external boundary diagnostic
//     `missingExternalSif` fires, so the reference is genuinely unresolved;
//   * with the generated SIF in scope: BOTH `missingExternalSif` and
//     `missingSassSymbol` are gone, proving the SIF actually resolved the token.
// A regression in the generator or the external-SIF resolution path flips one of
// those assertions and fails the release.

const CARGO_MANIFEST = "rust/Cargo.toml";
const BINARY_PATH = "rust/target/debug/omena-cli";
const CANONICAL_URL = "https://cdn.example/tokens.scss";

interface StyleDiagnosticsResult {
  readonly diagnostics?: ReadonlyArray<{ readonly code: string }>;
}

interface LifExportsResult {
  readonly lessVariables?: ReadonlyArray<{ readonly name: string; readonly valueRepr?: string }>;
  readonly lessMixins?: ReadonlyArray<{ readonly name: string; readonly guarded?: boolean }>;
  readonly lessDetachedRulesets?: ReadonlyArray<{
    readonly name: string;
    readonly memberNames?: readonly string[];
  }>;
}

function runChecked(
  label: string,
  command: string,
  args: readonly string[],
  options: { readonly cwd?: string } = {},
): string {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd ?? process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 32,
  });
  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `${label} failed (status=${result.status})\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result.stdout;
}

function diagnosticCodes(json: string, label: string): string[] {
  const parsed = JSON.parse(json) as StyleDiagnosticsResult;
  assert.ok(
    Array.isArray(parsed.diagnostics),
    `${label} must emit a diagnostics array, got: ${json}`,
  );
  return parsed.diagnostics.map((diagnostic) => diagnostic.code);
}

// 1. Build the real CLI surface that ships the generator + lock + resolution path.
runChecked("build omena-cli", "cargo", [
  "build",
  "--manifest-path",
  CARGO_MANIFEST,
  "-p",
  "omena-cli",
  "--quiet",
]);

const binary = path.resolve(process.cwd(), BINARY_PATH);
const workspace = mkdtempSync(path.join(tmpdir(), "omena-sif-e2e-"));

try {
  const tokensPath = path.join(workspace, "tokens.scss");
  const lessTokensPath = path.join(workspace, "tokens.less");
  const consumerPath = path.join(workspace, "consumer.module.scss");
  const sifPath = path.join(workspace, "tokens.sif.json");
  const lifExportsPath = path.join(workspace, "tokens.lif-exports.json");
  const lockPath = path.join(workspace, "omena.lock");

  // A package that references an external Sass symbol through an `@use` URL.
  writeFileSync(tokensPath, "$brand: red !default;\n");
  writeFileSync(
    lessTokensPath,
    [
      "@brand: #fff;",
      "@tokens: { primary: @brand; @gap: 2px; };",
      ".button(@gap: 1rem) when (@gap > 0) { color: @brand; }",
    ].join("\n"),
  );
  writeFileSync(
    consumerPath,
    `@use "${CANONICAL_URL}" as remote;\n.button { color: remote.$brand; }\n`,
  );

  // 2. Baseline: external sif mode with no SIF must flag the unresolved boundary.
  const baselineCodes = diagnosticCodes(
    runChecked("baseline style-diagnostics (no sif)", binary, [
      "style-diagnostics",
      consumerPath,
      "--source",
      consumerPath,
      "--external",
      "sif",
      "--json",
    ]),
    "baseline style-diagnostics",
  );
  assert.ok(
    baselineCodes.includes("missingExternalSif"),
    `baseline must report the unresolved external boundary, got: ${baselineCodes.join(",") || "(none)"}`,
  );

  // 3. Generate a SIF from the external module's tokens file.
  runChecked("sif generate", binary, [
    "sif",
    "generate",
    tokensPath,
    "--canonical-url",
    CANONICAL_URL,
    "--output",
    sifPath,
  ]);
  runChecked("sif generate-lif-exports", binary, [
    "sif",
    "generate-lif-exports",
    lessTokensPath,
    "--syntax",
    "less",
    "--output",
    lifExportsPath,
  ]);
  const lifExports = JSON.parse(readFileSync(lifExportsPath, "utf8")) as LifExportsResult;
  assert.deepEqual(
    lifExports.lessVariables?.map((variable) => variable.name),
    ["@brand"],
    "LIF export generation must expose Less variables through the shipped CLI",
  );
  assert.deepEqual(
    lifExports.lessMixins?.map((mixin) => [mixin.name, mixin.guarded]),
    [[".button", true]],
    "LIF export generation must expose guarded Less mixins through the shipped CLI",
  );
  assert.deepEqual(
    lifExports.lessDetachedRulesets?.map((ruleset) => [ruleset.name, ruleset.memberNames]),
    [["@tokens", ["@gap", "primary"]]],
    "LIF export generation must expose Less detached rulesets through the shipped CLI",
  );

  // 4. Author the lock from the generated SIF, then frozen-verify it.
  runChecked("lock update", binary, [
    "lock",
    "update",
    "--lockfile",
    lockPath,
    "--sif",
    sifPath,
    "--json",
  ]);
  const verifyReport = JSON.parse(
    runChecked("lock verify --frozen", binary, [
      "lock",
      "verify",
      "--lockfile",
      lockPath,
      "--frozen",
      "--json",
    ]),
  ) as { readonly verified?: boolean; readonly entriesChecked?: number };
  assert.equal(
    verifyReport.verified,
    true,
    "frozen lock verification must pass for the authored lock",
  );
  assert.equal(
    verifyReport.entriesChecked,
    1,
    "frozen lock verification must check the single authored SIF entry",
  );

  // 5. Decisive end-to-end assertion: with the generated SIF in scope, the
  //    external boundary AND the missing-symbol diagnostic are both gone.
  const resolvedCodes = diagnosticCodes(
    runChecked("resolved style-diagnostics (--sif)", binary, [
      "style-diagnostics",
      consumerPath,
      "--source",
      consumerPath,
      "--sif",
      sifPath,
      "--external",
      "sif",
      "--json",
    ]),
    "resolved style-diagnostics",
  );
  assert.ok(
    !resolvedCodes.includes("missingSassSymbol"),
    `generated SIF must suppress missingSassSymbol, got: ${resolvedCodes.join(",") || "(none)"}`,
  );
  assert.ok(
    !resolvedCodes.includes("missingExternalSif"),
    `generated SIF must resolve the external boundary, got: ${resolvedCodes.join(",") || "(none)"}`,
  );

  process.stdout.write(
    [
      "validated SIF end-to-end resolve loop:",
      "generator=sif-generate",
      "lif=generate-lif-exports",
      "lock=update+verify-frozen",
      `baseline=${baselineCodes.join("|") || "(none)"}`,
      `resolved=${resolvedCodes.join("|") || "(none)"}`,
      "assert=missingSassSymbol-absent+missingExternalSif-absent",
    ].join(" ") + "\n",
  );
} finally {
  rmSync(workspace, { recursive: true, force: true });
}
