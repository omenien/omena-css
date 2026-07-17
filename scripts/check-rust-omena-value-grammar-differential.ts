import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import * as cssTree from "css-tree";

import { formatGeneratedJson } from "./generated-json";

type SeedCase = {
  id: string;
  property: string;
  value: string;
  expectedValid?: boolean;
  adjudication?: string;
  reason?: string;
  notComparableReason?: string;
};

type SeedManifest = {
  schemaVersion: string;
  product: string;
  cases: SeedCase[];
  differentialCases?: SeedCase[];
};

type MatcherCase = {
  id: string;
  verdict: "matched" | "unmatched" | "notMatchedWithinBudget" | "grammarDefect";
};

type MatcherReport = {
  cases: MatcherCase[];
};

type OutcomeKind = "agreeValid" | "agreeInvalid" | "disagree" | "notComparable";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const seedPath = join(
  repoRoot,
  "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-seeds.json",
);
const ledgerPath = join(
  repoRoot,
  "rust/crates/omena-spec-audit/data/value-grammar-differential.json",
);
const write = process.argv.includes("--write");
const injectDivergence = process.env.OMENA_VALUE_GRAMMAR_TEST_INJECT_MATCHER_DIVERGENCE === "1";

void main();

async function main(): Promise<void> {
  const manifest = JSON.parse(readFileSync(seedPath, "utf8")) as SeedManifest;
  assert.equal(manifest.schemaVersion, "0");

  const cases = [...manifest.cases, ...(manifest.differentialCases ?? [])];
  const tempRoot = mkdtempSync(join(tmpdir(), "omena-value-grammar-differential-"));
  const combinedPath = join(tempRoot, "cases.json");
  writeFileSync(
    combinedPath,
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: manifest.product,
        cases: cases.map((entry) => ({
          id: entry.id,
          property: entry.property,
          value: entry.value,
          expectedValid: entry.expectedValid ?? false,
        })),
      },
      null,
      2,
    )}\n`,
  );

  try {
    const matcher = runCargoExample("value_grammar_evidence", [combinedPath]);
    const matcherReport = JSON.parse(matcher.stdout) as MatcherReport;
    const matcherById = new Map(matcherReport.cases.map((entry) => [entry.id, entry]));
    const inputDigest = createHash("sha256").update(JSON.stringify(cases)).digest("hex");
    const witnessRun = runCargoExample("value_grammar_external_tool_evidence", [
      "css-tree",
      cssTree.version,
      inputDigest,
      "0",
    ]);
    const witness = JSON.parse(witnessRun.stdout) as {
      earnedVia: string;
      key: { inputIdentity: string };
      provenance: string[];
    };
    assert.equal(witness.earnedVia, "externalTool");
    assert.equal(witness.key.inputIdentity, inputDigest);
    assert.ok(witness.provenance.includes(`toolVersion:${cssTree.version}`));

    const violations: string[] = [];
    let injected = false;
    const outcomes = cases.map((entry) => {
      const matcherCase = matcherById.get(entry.id);
      assert.ok(matcherCase, `missing matcher output for ${entry.id}`);
      let omenaValid = verdictValidity(matcherCase.verdict);
      if (injectDivergence && !injected && omenaValid !== null && entry.expectedValid === true) {
        omenaValid = !omenaValid;
        injected = true;
      }
      const external = cssTreeValidity(entry.property, entry.value);
      let outcome: OutcomeKind;
      let notComparableReason: string | undefined;
      if (omenaValid === null) {
        outcome = "notComparable";
        notComparableReason = `omena:${matcherCase.verdict}`;
      } else if (external.valid === null) {
        outcome = "notComparable";
        notComparableReason = external.reason;
      } else if (omenaValid === external.valid) {
        outcome = omenaValid ? "agreeValid" : "agreeInvalid";
      } else {
        outcome = "disagree";
      }

      const adjudication =
        injected && entry.expectedValid === true ? undefined : entry.adjudication;
      if (outcome === "disagree" && !adjudication) {
        violations.push(`${entry.id}: unexplained disagreement`);
      }
      if (outcome !== "disagree" && entry.adjudication) {
        violations.push(`${entry.id}: stale disagreement adjudication`);
      }
      if (outcome === "notComparable" && !entry.notComparableReason && !notComparableReason) {
        violations.push(`${entry.id}: missing not-comparable reason`);
      }
      if (entry.notComparableReason && outcome !== "notComparable") {
        violations.push(`${entry.id}: expected a not-comparable outcome`);
      }

      return {
        id: entry.id,
        property: entry.property,
        value: entry.value,
        omenaVerdict: matcherCase.verdict,
        cssTreeValid: external.valid,
        outcome,
        ...(adjudication ? { adjudication } : {}),
        ...(entry.reason ? { reason: entry.reason } : {}),
        ...(outcome === "notComparable"
          ? { notComparableReason: entry.notComparableReason ?? notComparableReason }
          : {}),
      };
    });

    const counts = Object.fromEntries(
      (["agreeValid", "agreeInvalid", "disagree", "notComparable"] as const).map((kind) => [
        kind,
        outcomes.filter((outcome) => outcome.outcome === kind).length,
      ]),
    ) as Record<OutcomeKind, number>;
    for (const [kind, count] of Object.entries(counts)) {
      if (count === 0) violations.push(`outcome ${kind} is vacuous`);
    }
    if (injectDivergence && !injected) {
      violations.push("divergence injection did not reach a comparable valid case");
    }

    const ledger = {
      schemaVersion: "0",
      product: "rust.omena-value-grammar-differential",
      sourceProduct: manifest.product,
      oracle: { name: "css-tree", version: cssTree.version },
      inputDigest,
      caseCount: outcomes.length,
      counts,
      witness,
      outcomes,
    };
    const serialized = await formatGeneratedJson(ledgerPath, ledger);
    if (write) {
      writeFileSync(ledgerPath, serialized);
    } else if (!injectDivergence) {
      assert.equal(readFileSync(ledgerPath, "utf8"), serialized, "differential ledger drifted");
    }

    process.stdout.write(
      `${JSON.stringify({ ...ledger, violations, injectedDivergence: injected }, null, 2)}\n`,
    );
    assert.deepEqual(violations, []);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function runCargoExample(example: string, args: string[]) {
  const run = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "--quiet",
      "-p",
      "omena-abstract-value",
      "--example",
      example,
      "--",
      ...args,
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(run.status, 0, run.stderr || run.stdout);
  return run;
}

function verdictValidity(verdict: MatcherCase["verdict"]): boolean | null {
  if (verdict === "matched") return true;
  if (verdict === "unmatched") return false;
  return null;
}

function cssTreeValidity(
  property: string,
  value: string,
): {
  valid: boolean | null;
  reason?: string;
} {
  try {
    const result = cssTree.lexer.matchProperty(property, value);
    return { valid: result.matched !== null };
  } catch (error) {
    return {
      valid: null,
      reason: error instanceof Error ? error.message : String(error),
    };
  }
}
