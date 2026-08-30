import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface DeadReexportRow {
  readonly originCrate: string;
  readonly name: string;
  readonly disposition: "deprecatedFacade" | "existingDeprecation" | "renamedCompatibility";
}

interface DeadReexportExperiment {
  readonly schemaVersion: "0";
  readonly product: "omena-query.dead-reexport-experiment";
  readonly baseline: {
    readonly kind: "publishedRegistry";
    readonly version: "0.5.0";
    readonly allFeatures: true;
  };
  readonly authoredCandidateCount: 120;
  readonly rows: readonly DeadReexportRow[];
  readonly deletionExperiment: {
    readonly removedAllFeaturesPublicApiPathCount: 120;
    readonly cargoSemverCandidateFailureCount: 0;
    readonly cargoSemverObservedFailureLints: readonly string[];
    readonly candidateVerdictTable: {
      readonly rowSource: "rows";
      readonly rowCount: 120;
      readonly lint: "all_features_public_path_missing";
      readonly witnessTemplate: "omena_query::<name>";
      readonly witnessAuthority: "cargo-public-api 0.52.0 all-features differential";
      readonly cargoSemverChecksDisposition: "CANNOT-see-cross-crate-reexports";
      readonly upstreamIssue: "https://github.com/obi1kenobi/cargo-semver-checks/issues/638";
    };
  };
}

interface UseBlock {
  readonly attributes: string;
  readonly visibility: "public" | "crate";
  readonly path: string;
  readonly body: string;
  readonly names: readonly string[];
  readonly start: number;
  readonly end: number;
  readonly text: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tablePath = path.join(repoRoot, "rust/omena-query-dead-reexport-experiment.json");
const sourcePath = path.join(repoRoot, "rust/crates/omena-query/src/lib.rs");
const renameMapPath = path.join(repoRoot, "rust/omena-domain-claim-rename-map.json");
const table = JSON.parse(readFileSync(tablePath, "utf8")) as DeadReexportExperiment;
const source = readFileSync(sourcePath, "utf8");
const renameMap = readFileSync(renameMapPath, "utf8");

validateTable(table, source, renameMap);
runValidatorSelftests(table, source, renameMap);

if (process.argv.includes("--measure-removal")) {
  measureRemoval(table);
} else {
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "rust.omena-query.dead-reexports",
        candidateCount: table.rows.length,
        deprecatedFacadeCount: countDisposition(table.rows, "deprecatedFacade"),
        existingDeprecationCount: countDisposition(table.rows, "existingDeprecation"),
        renamedCompatibilityCount: countDisposition(table.rows, "renamedCompatibility"),
        baselineKind: table.baseline.kind,
        baselineVersion: table.baseline.version,
        selftestMutationCount: 3,
      },
      null,
      2,
    )}\n`,
  );
}

function validateTable(
  candidateTable: DeadReexportExperiment,
  candidateSource: string,
  candidateRenameMap: string,
): void {
  assert.equal(candidateTable.schemaVersion, "0");
  assert.equal(candidateTable.product, "omena-query.dead-reexport-experiment");
  assert.equal(candidateTable.baseline.kind, "publishedRegistry");
  assert.equal(candidateTable.baseline.version, "0.5.0");
  assert.equal(candidateTable.baseline.allFeatures, true);
  assert.equal(candidateTable.authoredCandidateCount, 120);
  assert.equal(candidateTable.rows.length, candidateTable.authoredCandidateCount);
  assert.equal(
    new Set(candidateTable.rows.map((row) => row.name)).size,
    candidateTable.rows.length,
    "dead re-export candidate names must be unique",
  );
  assert.equal(countDisposition(candidateTable.rows, "deprecatedFacade"), 115);
  assert.equal(countDisposition(candidateTable.rows, "existingDeprecation"), 2);
  assert.equal(countDisposition(candidateTable.rows, "renamedCompatibility"), 3);
  assert.equal(candidateTable.deletionExperiment.removedAllFeaturesPublicApiPathCount, 120);
  assert.equal(candidateTable.deletionExperiment.cargoSemverCandidateFailureCount, 0);
  assert.deepEqual(
    [...candidateTable.deletionExperiment.cargoSemverObservedFailureLints].toSorted(),
    ["enum_missing", "function_missing", "pub_module_level_const_missing", "struct_missing"],
  );
  assert.deepEqual(candidateTable.deletionExperiment.candidateVerdictTable, {
    rowSource: "rows",
    rowCount: 120,
    lint: "all_features_public_path_missing",
    witnessTemplate: "omena_query::<name>",
    witnessAuthority: "cargo-public-api 0.52.0 all-features differential",
    cargoSemverChecksDisposition: "CANNOT-see-cross-crate-reexports",
    upstreamIssue: "https://github.com/obi1kenobi/cargo-semver-checks/issues/638",
  });
  assert.equal(
    candidateTable.deletionExperiment.candidateVerdictTable.rowCount,
    candidateTable.rows.length,
    "the deletion verdict table must bind every candidate row",
  );

  const blocks = scanUseBlocks(candidateSource);
  for (const row of candidateTable.rows) {
    const block = blocks.find(
      (candidate) => candidate.visibility === "public" && candidate.names.includes(row.name),
    );
    assert.ok(block, `${row.name} must remain a public compatibility path`);
    assert.match(block.attributes, /#\[deprecated\(/u, `${row.name} must be deprecated`);
    if (row.disposition === "deprecatedFacade") {
      assert.match(block.attributes, /since = "0\.6\.0"/u);
      assert.match(
        block.attributes,
        /removal requires a separately reviewed pre-1\.0 semver decision/u,
      );
    }
    if (row.disposition === "existingDeprecation") {
      assert.match(block.attributes, /since = "0\.4\.0"/u);
    }
    if (row.disposition === "renamedCompatibility") {
      assert.match(block.attributes, /since = "0\.4\.0"/u);
      assert.ok(
        candidateRenameMap.includes(row.name),
        `${row.name} must remain named in the domain rename map`,
      );
    }
  }

  const mutation = removeDeadReexports(
    candidateSource,
    new Set(candidateTable.rows.map((row) => row.name)),
  );
  assert.equal(mutation.removedCount, candidateTable.rows.length);
  const mutatedNames = new Set(scanUseBlocks(mutation.source).flatMap((block) => block.names));
  for (const row of candidateTable.rows) {
    assert.ok(!mutatedNames.has(row.name), `${row.name} deletion mutation did not take effect`);
  }
}

function runValidatorSelftests(
  candidateTable: DeadReexportExperiment,
  candidateSource: string,
  candidateRenameMap: string,
): void {
  assert.throws(
    () =>
      validateTable(
        { ...candidateTable, rows: candidateTable.rows.slice(1) } as DeadReexportExperiment,
        candidateSource,
        candidateRenameMap,
      ),
    /120|candidate/u,
  );
  const first = candidateTable.rows[0];
  assert.ok(first);
  assert.throws(
    () =>
      validateTable(
        candidateTable,
        candidateSource.replaceAll(first.name, `${first.name}Mutation`),
        candidateRenameMap,
      ),
    /must remain a public compatibility path/u,
  );
  assert.throws(
    () =>
      validateTable(
        {
          ...candidateTable,
          deletionExperiment: {
            ...candidateTable.deletionExperiment,
            candidateVerdictTable: {
              ...candidateTable.deletionExperiment.candidateVerdictTable,
              rowCount: 0,
            },
          },
        } as DeadReexportExperiment,
        candidateSource,
        candidateRenameMap,
      ),
    /120|deletion verdict table/u,
  );
}

function scanUseBlocks(candidateSource: string): readonly UseBlock[] {
  const blockPattern =
    /((?:#\[[^\]]*\]\s*)*)pub(?:(\(crate\)))?\s+use\s+([A-Za-z0-9_:]+)::\{([\s\S]*?)\};/gu;
  const blocks: UseBlock[] = [];
  for (const match of candidateSource.matchAll(blockPattern)) {
    const text = match[0];
    const start = match.index ?? 0;
    const body = match[4] ?? "";
    blocks.push({
      attributes: match[1] ?? "",
      visibility: match[2] === "(crate)" ? "crate" : "public",
      path: match[3] ?? "",
      body,
      names: body
        .split(",")
        .map((item) => exportedName(item.trim()))
        .filter((name): name is string => name !== null),
      start,
      end: start + text.length,
      text,
    });
  }
  return blocks;
}

function removeDeadReexports(
  candidateSource: string,
  namesToRemove: ReadonlySet<string>,
): { readonly source: string; readonly removedCount: number } {
  const blocks = scanUseBlocks(candidateSource);
  let cursor = 0;
  let output = "";
  let removedCount = 0;
  for (const block of blocks) {
    const items = block.body
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item.length > 0);
    const retained = items.filter((item) => {
      const name = exportedName(item);
      if (name !== null && namesToRemove.has(name)) {
        removedCount += 1;
        return false;
      }
      return true;
    });
    if (retained.length === items.length) {
      continue;
    }
    const visibility = block.visibility === "crate" ? "pub(crate)" : "pub";
    const replacement =
      `${block.attributes}${visibility} use ${block.path}::{\n` +
      (retained.length > 0 ? `    ${retained.join(",\n    ")},\n` : "") +
      "};";
    output += candidateSource.slice(cursor, block.start) + replacement;
    cursor = block.end;
  }
  output += candidateSource.slice(cursor);
  return { source: output, removedCount };
}

function exportedName(item: string): string | null {
  if (item.length === 0) {
    return null;
  }
  const alias = item.match(/\bas\s+([A-Za-z0-9_]+)$/u)?.[1];
  return alias ?? item.match(/([A-Za-z0-9_]+)$/u)?.[1] ?? null;
}

function countDisposition(
  rows: readonly DeadReexportRow[],
  disposition: DeadReexportRow["disposition"],
): number {
  return rows.filter((row) => row.disposition === disposition).length;
}

function measureRemoval(candidateTable: DeadReexportExperiment): void {
  const parent = mkdtempSync(path.join(os.tmpdir(), "omena-query-dead-reexport-"));
  const worktree = path.join(parent, "worktree");
  const isolatedTarget = path.join(parent, "target");
  const environment = { ...process.env, CARGO_TARGET_DIR: isolatedTarget };
  try {
    execFileSync("git", ["worktree", "add", "--detach", worktree, "HEAD"], {
      cwd: repoRoot,
      stdio: "inherit",
    });
    const worktreeSourcePath = path.join(worktree, "rust/crates/omena-query/src/lib.rs");
    const worktreeSource = readFileSync(worktreeSourcePath, "utf8");
    const mutation = removeDeadReexports(
      worktreeSource,
      new Set(candidateTable.rows.map((row) => row.name)),
    );
    assert.equal(mutation.removedCount, candidateTable.rows.length);
    writeFileSync(worktreeSourcePath, mutation.source);

    execFileSync(
      "cargo",
      [
        "check",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-query",
        "--all-targets",
        "--all-features",
      ],
      { cwd: worktree, env: environment, stdio: "inherit" },
    );

    const currentApi = runPublicApi(repoRoot, environment);
    const deletedApi = runPublicApi(worktree, environment);
    const removedPaths = candidateTable.rows.filter(
      (row) => publicApiHasName(currentApi, row.name) && !publicApiHasName(deletedApi, row.name),
    );
    assert.equal(
      removedPaths.length,
      candidateTable.deletionExperiment.removedAllFeaturesPublicApiPathCount,
      "all-features public API deletion witness count drifted",
    );

    const semver = spawnSync(
      "cargo",
      [
        "semver-checks",
        "check-release",
        "--manifest-path",
        "rust/crates/omena-query/Cargo.toml",
        "--baseline-version",
        candidateTable.baseline.version,
        "--all-features",
        "--color",
        "never",
      ],
      { cwd: worktree, env: environment, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    );
    assert.equal(
      semver.status,
      1,
      "deletion semver experiment must report the known release delta",
    );
    const semverOutput = `${semver.stdout}\n${semver.stderr}`;
    const observedLints = [...semverOutput.matchAll(/^--- failure ([a-z0-9_]+):/gmu)].map(
      (match) => match[1],
    );
    assert.deepEqual(
      observedLints.toSorted(),
      [...candidateTable.deletionExperiment.cargoSemverObservedFailureLints].toSorted(),
      "deletion experiment cargo-semver lint set drifted",
    );
    const candidateWitnesses = semverOutput
      .split("\n")
      .filter((line) =>
        candidateTable.rows.some((row) => line.includes(`omena_query::${row.name}`)),
      );
    assert.equal(
      candidateWitnesses.length,
      candidateTable.deletionExperiment.cargoSemverCandidateFailureCount,
      "cargo-semver candidate witness count drifted",
    );
    const candidateVerdictRows = removedPaths.map((row) => ({
      originCrate: row.originCrate,
      name: row.name,
      lint: candidateTable.deletionExperiment.candidateVerdictTable.lint,
      witness: `omena_query::${row.name}`,
    }));
    assert.equal(
      candidateVerdictRows.length,
      candidateTable.deletionExperiment.candidateVerdictTable.rowCount,
      "the public-API deletion verdict table must remain non-empty and complete",
    );
    process.stdout.write(
      `${JSON.stringify(
        {
          schemaVersion: "0",
          product: "rust.omena-query.dead-reexport-removal-measurement",
          baselineKind: candidateTable.baseline.kind,
          baselineVersion: candidateTable.baseline.version,
          compile: "green",
          removedAllFeaturesPublicApiPathCount: removedPaths.length,
          cargoSemverFailureLints: observedLints,
          cargoSemverCandidateFailureCount: candidateWitnesses.length,
          candidateVerdictTable: {
            ...candidateTable.deletionExperiment.candidateVerdictTable,
            rows: candidateVerdictRows,
          },
        },
        null,
        2,
      )}\n`,
    );
  } finally {
    spawnSync("git", ["worktree", "remove", "--force", worktree], {
      cwd: repoRoot,
      stdio: "inherit",
    });
    rmSync(parent, { recursive: true, force: true });
  }
}

function runPublicApi(cwd: string, environment: NodeJS.ProcessEnv): string {
  return execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "--all-features",
      "-sss",
      "--color",
      "never",
    ],
    { cwd, env: environment, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
}

function publicApiHasName(publicApi: string, name: string): boolean {
  return new RegExp(`\\bomena_query::${name}\\b`, "u").test(publicApi);
}
