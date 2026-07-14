import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { parseOmenaCliResponse } from "./lib/omena-cli-response";

interface MigrationPlan {
  readonly product: "omena-cli.migration-plan";
  readonly codemod: "cssModulesRename" | "sassImportToUse" | "tokenRename";
  readonly edits: readonly MigrationEdit[];
  readonly safeEdits: readonly string[];
  readonly reviewEdits: readonly string[];
  readonly blockers: readonly { readonly code: string }[];
  readonly evidence: readonly {
    readonly id: string;
    readonly kind: string;
    readonly source: string;
  }[];
  readonly rollback: {
    readonly receiptTyped: boolean;
    readonly receipt?: {
      readonly passId: string;
      readonly attemptedMutationCount: number;
      readonly restorable: { readonly kind: "inversePatch" };
    };
    readonly inverseEdits: readonly { readonly editId: string }[];
  };
}

interface MigrationEdit {
  readonly id: string;
  readonly uri: string;
  readonly expectedText: string;
  readonly replacementText: string;
  readonly evidence: { readonly primary: string; readonly supporting?: readonly string[] };
}

interface MigrationApplyReport {
  readonly product: "omena-cli.migration-apply-report";
  readonly appliedEditCount: number;
  readonly appliedFileCount: number;
  readonly writeReports: readonly {
    readonly writeKind: "migrationPlan";
    readonly wrote: boolean;
  }[];
  readonly rollback: MigrationPlan["rollback"];
}

interface SassMigrationFixture {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.sass-migration-fixtures";
  readonly source: { readonly repository: string; readonly pin: string };
  readonly cases: readonly {
    readonly id: string;
    readonly upstreamPath: string;
    readonly upstreamCase: string;
    readonly entry: string;
    readonly files: readonly { readonly path: string; readonly source: string }[];
  }[];
}

const repoRoot = path.resolve(import.meta.dirname, "..");
const workspace = mkdtempSync(path.join(os.tmpdir(), "omena-cli-migration-"));

try {
  buildCli();
  verifyPlanFirstRename();
  verifyDynamicRenameReview();
  verifySassOracle();
  verifyTokenReview();
  process.stdout.write(
    "validated omena-cli migrations: deterministic plans, guarded apply, query-owned occurrences, Dart Sass oracle, typed rollback\n",
  );
} finally {
  rmSync(workspace, { recursive: true, force: true });
}

function verifyPlanFirstRename(): void {
  const root = path.join(workspace, "css-modules-safe");
  const src = path.join(root, "src");
  mkdirSync(src, { recursive: true });
  const stylePath = path.join(src, "Button.module.css");
  const sourcePath = path.join(src, "Button.tsx");
  writeFileSync(stylePath, ".button { color: red; }\n");
  writeFileSync(
    sourcePath,
    [
      'import classNames from "classnames/bind";',
      'import styles from "./Button.module.css";',
      "const cx = classNames.bind(styles);",
      'export const view = <button className={cx("button")} />;',
      "",
    ].join("\n"),
  );
  const before = sourceHashes([stylePath, sourcePath]);
  const firstPath = path.join(root, "rename-plan.json");
  const secondPath = path.join(root, "rename-plan-repeat.json");
  const args = [
    "migrate",
    "css-modules-rename",
    "button",
    "control",
    "--root",
    root,
    "--target-style",
    stylePath,
  ];
  const first = plan([...args, "--plan", firstPath], firstPath);
  const second = plan([...args, "--plan", secondPath], secondPath);
  assert.deepEqual(sourceHashes([stylePath, sourcePath]), before, "planning mutated a source file");
  assert.equal(readFileSync(firstPath, "utf8"), readFileSync(secondPath, "utf8"));
  assert.deepEqual(first, second);
  assert.equal(first.product, "omena-cli.migration-plan");
  assert.equal(first.edits.length, 2);
  assert.equal(first.safeEdits.length, 2);
  assert.equal(first.reviewEdits.length, 0);
  assert.equal(first.blockers.length, 0);
  assertEvidenceAndRollback(first);

  const tamperedPath = path.join(root, "tampered-plan.json");
  const tampered = JSON.parse(readFileSync(firstPath, "utf8")) as MigrationPlan;
  writeFileSync(
    tamperedPath,
    `${JSON.stringify(
      {
        ...tampered,
        edits: tampered.edits.map((edit, index) => {
          if (index !== 0) return edit;
          return Object.assign({}, edit, { evidence: { primary: "unknown-evidence" } });
        }),
      },
      null,
      2,
    )}\n`,
  );
  const rejected = runCli(["migrate", "css-modules-rename", "--apply", tamperedPath, "--json"]);
  assert.notEqual(rejected.status, 0, "an edit with unbound evidence was accepted");
  assert.deepEqual(sourceHashes([stylePath, sourcePath]), before);

  const applied = parseOmenaCliResponse<MigrationApplyReport>(
    runCliOk(["migrate", "css-modules-rename", "--apply", firstPath, "--json"]),
    "omena-cli.migrate.apply",
  );
  assert.equal(applied.product, "omena-cli.migration-apply-report");
  assert.equal(applied.appliedEditCount, 2);
  assert.equal(applied.appliedFileCount, 2);
  assert(
    applied.writeReports.every((report) => report.writeKind === "migrationPlan" && report.wrote),
  );
  assertTypedRollback(applied.rollback, first.edits.length);
  assert.match(readFileSync(stylePath, "utf8"), /\.control/u);
  assert.match(readFileSync(sourcePath, "utf8"), /cx\("control"\)/u);
}

function verifyDynamicRenameReview(): void {
  const root = path.join(workspace, "css-modules-dynamic");
  mkdirSync(root, { recursive: true });
  const stylePath = path.join(root, "Button.module.css");
  const sourcePath = path.join(root, "Button.tsx");
  const planPath = path.join(root, "migration-plan.json");
  writeFileSync(stylePath, ".button-primary { color: red; }\n.button-secondary { color: blue; }\n");
  writeFileSync(
    sourcePath,
    [
      'import classNames from "classnames/bind";',
      'import styles from "./Button.module.css";',
      "const cx = classNames.bind(styles);",
      'const exact = cx("button-primary");',
      "const dynamic = cx(`button-${variant}`);",
      "",
    ].join("\n"),
  );
  const planValue = plan(
    [
      "migrate",
      "css-modules-rename",
      "button-primary",
      "control-primary",
      "--root",
      root,
      "--target-style",
      stylePath,
      "--plan",
      planPath,
    ],
    planPath,
  );
  assert.equal(planValue.safeEdits.length, 2);
  assert.equal(planValue.reviewEdits.length, 1);
  const review = planValue.edits.find((edit) => planValue.reviewEdits.includes(edit.id));
  assert.equal(review?.expectedText, "button-");
  const before = sourceHashes([stylePath, sourcePath]);
  const rejected = runCli(["migrate", "css-modules-rename", "--apply", planPath, "--json"]);
  assert.notEqual(rejected.status, 0, "review edits were applied without approval");
  assert.deepEqual(sourceHashes([stylePath, sourcePath]), before);
}

function verifySassOracle(): void {
  const fixtures = JSON.parse(
    readFileSync(path.join(repoRoot, "rust/crates/omena-cli/fixtures/sass-migration.json"), "utf8"),
  ) as SassMigrationFixture;
  const corpusManifest = JSON.parse(
    readFileSync(
      path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus/manifest.json"),
      "utf8",
    ),
  ) as { readonly source: { readonly repository: string; readonly pin: string } };
  assert.equal(fixtures.schemaVersion, "0");
  assert.equal(fixtures.product, "omena-cli.sass-migration-fixtures");
  assert.deepEqual(fixtures.source, {
    repository: corpusManifest.source.repository,
    pin: corpusManifest.source.pin,
  });

  const equivalentFixture = sassFixture(fixtures, "scss-precedes-css");
  const root = path.join(workspace, "sass");
  writeSassFixture(root, equivalentFixture.files);
  const sourcePaths = equivalentFixture.files.map((file) => path.join(root, file.path));
  const entryPath = path.join(root, equivalentFixture.entry);
  const planPath = path.join(root, "migration-plan.json");
  const before = sourceHashes(sourcePaths);
  const planValue = plan(
    ["migrate", "sass-import-to-use", "--root", entryPath, "--plan", planPath],
    planPath,
  );
  assert.deepEqual(sourceHashes(sourcePaths), before);
  assert.equal(planValue.safeEdits.length, 1);
  assert.equal(planValue.blockers.length, 0);
  assert(
    planValue.evidence.some(
      (item) => item.kind === "dartSassCompileEquivalence" && item.source === "dart-sass@1.101.0",
    ),
  );
  assertEvidenceAndRollback(planValue);

  const transitiveFixture = sassFixture(fixtures, "transitive-forwarded-variable");
  const transitiveRoot = path.join(workspace, "sass-transitive");
  writeSassFixture(transitiveRoot, transitiveFixture.files);
  const transitivePlanPath = path.join(transitiveRoot, "migration-plan.json");
  const transitivePlan = plan(
    ["migrate", "sass-import-to-use", "--root", transitiveRoot, "--plan", transitivePlanPath],
    transitivePlanPath,
  );
  assert(
    transitivePlan.blockers.some((blocker) => blocker.code === "sassOracleMismatch"),
    "a downstream visibility regression was not blocked",
  );
  assert.equal(transitivePlan.edits.length, 0);
  assert.equal(transitivePlan.safeEdits.length, 0);
}

function sassFixture(
  fixtures: SassMigrationFixture,
  id: string,
): SassMigrationFixture["cases"][number] {
  const fixture = fixtures.cases.find((candidate) => candidate.id === id);
  assert(fixture, `missing Sass migration fixture ${id}`);
  assert.match(fixture.upstreamPath, /^spec\//u);
  assert.notEqual(fixture.upstreamCase, "");
  return fixture;
}

function writeSassFixture(
  root: string,
  files: SassMigrationFixture["cases"][number]["files"],
): void {
  for (const file of files) {
    const outputPath = path.join(root, file.path);
    mkdirSync(path.dirname(outputPath), { recursive: true });
    writeFileSync(outputPath, file.source);
  }
}

function verifyTokenReview(): void {
  const root = path.join(workspace, "tokens");
  mkdirSync(root, { recursive: true });
  const sourcePath = path.join(root, "tokens.css");
  const planPath = path.join(root, "migration-plan.json");
  writeFileSync(
    sourcePath,
    [
      '@property --brand { syntax: "<color>"; inherits: true; initial-value: red; }',
      ":root { --brand: red; }",
      ".plain { color: var(--brand); }",
      ".fallback { color: var(--brand, blue); }",
      "",
    ].join("\n"),
  );
  const planValue = plan(
    ["migrate", "token-rename", "brand", "accent", "--root", root, "--plan", planPath],
    planPath,
  );
  assert.equal(planValue.edits.length, 4);
  assert.equal(planValue.safeEdits.length, 3);
  assert.equal(planValue.reviewEdits.length, 1);
  assert.equal(
    planValue.edits.find((edit) => planValue.reviewEdits.includes(edit.id))?.expectedText,
    "--brand",
  );
  assertEvidenceAndRollback(planValue);
}

function assertEvidenceAndRollback(planValue: MigrationPlan): void {
  const evidenceIds = new Set(planValue.evidence.map((item) => item.id));
  assert(
    planValue.edits.every(
      (edit) =>
        evidenceIds.has(edit.evidence.primary) &&
        (edit.evidence.supporting ?? []).every((id) => evidenceIds.has(id)),
    ),
  );
  assert.equal(planValue.rollback.receiptTyped, false);
  assert.equal(planValue.rollback.receipt, undefined);
  assert.equal(planValue.rollback.inverseEdits.length, planValue.edits.length);
  assert.deepEqual(
    planValue.rollback.inverseEdits.map((edit) => edit.editId).toSorted(),
    planValue.edits.map((edit) => edit.id).toSorted(),
  );
}

function assertTypedRollback(rollback: MigrationPlan["rollback"], editCount: number): void {
  assert.equal(rollback.receiptTyped, true);
  assert.match(rollback.receipt?.passId ?? "", /^source\.migration\./u);
  assert.equal(rollback.receipt?.attemptedMutationCount, editCount);
  assert.deepEqual(rollback.receipt?.restorable, { kind: "inversePatch" });
  assert.equal(rollback.inverseEdits.length, editCount);
}

function plan(args: readonly string[], artifactPath: string): MigrationPlan {
  const payload = parseOmenaCliResponse<MigrationPlan>(
    runCliOk([...args, "--json"]),
    "omena-cli.migrate.plan",
  );
  assert.deepEqual(JSON.parse(readFileSync(artifactPath, "utf8")), payload);
  return payload;
}

function buildCli(): void {
  const result = spawnSync(
    "cargo",
    ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli", "--bin", "omena"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, [result.stdout, result.stderr].join("\n"));
}

function runCliOk(args: readonly string[]): string {
  const result = runCli(args);
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, [result.stdout, result.stderr].join("\n"));
  return result.stdout;
}

function runCli(args: readonly string[]): ReturnType<typeof spawnSync> {
  const executable = path.join(
    repoRoot,
    "rust",
    "target",
    "debug",
    process.platform === "win32" ? "omena.exe" : "omena",
  );
  return spawnSync(executable, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function sourceHashes(paths: readonly string[]): readonly string[] {
  return paths.map((filePath) => createHash("sha256").update(readFileSync(filePath)).digest("hex"));
}
