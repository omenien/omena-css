import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const args = new Set(process.argv.slice(2));
const injectUnqualifiedSpeedClaim = args.has("--inject-unqualified-speed-claim");
const injectSecondOrderSite = args.has("--inject-second-order-site");
const injectResponseLeak = args.has("--inject-response-leak");
assert.ok(
  [injectUnqualifiedSpeedClaim, injectSecondOrderSite, injectResponseLeak].filter(Boolean).length <=
    1,
  "only one guarded-cascade falsifier may run at once",
);

const firstWitnessPath = "rust/crates/omena-cascade/src/first_witness.rs";
let firstWitness = fs.readFileSync(path.join(repoRoot, firstWitnessPath), "utf8");
const atRuleCall = ["VariableOrderRegistrationV0::at_rule_", "nesting_dfs("].join("");
const siteCall = ["VariableOrderRegistrationV0::site_", "first_appearance("].join("");
const trackedRustSources = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
  cwd: repoRoot,
  encoding: "utf8",
})
  .split("\0")
  .filter(Boolean)
  .filter(isProductionRustSource);
const rivalPath = "rust/crates/omena-query/src/style/cascade_checker/runtime_state.rs";
const atRuleSites: string[] = [];
let siteSiblingCount = 0;
for (const sourcePath of trackedRustSources) {
  let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
  if (injectSecondOrderSite && sourcePath === rivalPath) {
    const injection = `\nfn injected_guarded_order_rival() { let _ = omena_cascade::${atRuleCall}[]); }\n`;
    const testModule = source.search(/\n#\[cfg\(test\)\]\s*\nmod\s+/u);
    source =
      testModule < 0
        ? source + injection
        : source.slice(0, testModule) + injection + source.slice(testModule);
  }
  const production = productionRustPrefix(source);
  siteSiblingCount += countOccurrences(production, siteCall);
  for (const [lineIndex, line] of production.split(/\r?\n/u).entries()) {
    if (line.includes(atRuleCall)) {
      atRuleSites.push(`${sourcePath}:${lineIndex + 1}`);
    }
  }
}
const atRuleDerivationCount = atRuleSites.length;
assert.equal(
  atRuleDerivationCount,
  1,
  `atRuleNestingDfs must have exactly one git-ls-files Rust production derivation site; observed ${atRuleDerivationCount}: ${atRuleSites.join(", ")}`,
);
assert.ok(
  siteSiblingCount > 0,
  "siteFirstAppearance must remain registered as a sibling domain and is not an at-rule violation",
);
assert.ok(
  firstWitness.includes('AT_RULE_NESTING_DFS_ORDERING_DOMAIN_V0: &str = "atRuleNestingDfs"'),
  "at-rule ordering domain must remain explicitly named",
);
assert.ok(
  firstWitness.includes('SITE_FIRST_APPEARANCE_ORDERING_DOMAIN_V0: &str = "siteFirstAppearance"'),
  "site ordering sibling domain must remain explicitly named",
);

const trackedSources = execFileSync("git", ["ls-files", "-z"], {
  cwd: repoRoot,
  encoding: "utf8",
})
  .split("\0")
  .filter(Boolean)
  .filter(isHonestClaimSurface);
const treeWord = ["tr", "ee"].join("");
const speedWord = ["fas", "ter"].join("");
const unqualifiedPhrase = ["the", treeWord, "is", speedWord].join(" ");
const totalCostWord = ["poly", "logarithmic"].join("");
const broadTreeClaim = new RegExp(
  `\\b(?:incremental|augmented)?[ -]*${treeWord}\\b.{0,60}\\b${speedWord}\\b`,
  "iu",
);
const violations: string[] = [];
for (const sourcePath of trackedSources) {
  let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
  if (injectUnqualifiedSpeedClaim && sourcePath === firstWitnessPath) {
    source += `\n// ${unqualifiedPhrase}\n`;
  }
  for (const [lineIndex, line] of source.split(/\r?\n/u).entries()) {
    const lower = line.toLocaleLowerCase("en-US");
    if (
      lower.includes(unqualifiedPhrase) ||
      lower.includes(totalCostWord) ||
      broadTreeClaim.test(line)
    ) {
      violations.push(`${sourcePath}:${lineIndex + 1}:${line.trim()}`);
    }
  }
}
assert.deepEqual(
  violations,
  [],
  `guarded cascade performance claim lacks its apply-cache condition:\n${violations.join("\n")}`,
);
assert.ok(
  firstWitness.includes("wall-clock benefit is conditional on the apply-cache budget"),
  "the measured tree claim must retain its apply-cache-budget condition",
);

const responseEnvironment = { ...process.env };
if (injectResponseLeak) {
  responseEnvironment.OMENA_RESPONSE_SPLIT_TEST_INJECT_INTERNAL = "1";
}
execFileSync(
  process.execPath,
  ["--import", "tsx", "scripts/check-rust-omena-response-surface-split-census.ts"],
  {
    cwd: repoRoot,
    env: responseEnvironment,
    stdio: "inherit",
  },
);

// FALSIFIER: id=guarded-cascade-unqualified-speed-claim class=accounting via=--inject-unqualified-speed-claim producer=can-fail owner=guarded-cascade-winner-closure entry=apply-cache-conditioned-claim
// FALSIFIER: id=guarded-cascade-second-at-rule-order-site class=structuralEntailment via=--inject-second-order-site producer=can-fail owner=guarded-cascade-winner-closure entry=git-ls-files-rust-production-order-authority
// FALSIFIER: id=guarded-cascade-response-surface-leak class=accounting via=--inject-response-leak producer=can-fail owner=guarded-cascade-winner-closure entry=public-response-split
process.stdout.write(
  `guarded cascade winner closure OK: scope=gitLsFilesRustProduction orderDomain=atRuleNestingDfs derivationSites=${atRuleDerivationCount} siblingSiteFirstAppearance=${siteSiblingCount} honestClaimFiles=${trackedSources.length} responseSplit=checked\n`,
);

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

function isHonestClaimSurface(sourcePath: string): boolean {
  if (sourcePath.startsWith(".personal_docs/") || sourcePath.startsWith("vendor/")) return false;
  return /(?:^|\/)(?:[^/]+\.)?(?:rs|ts|tsx|js|mjs|md|json|toml|ya?ml)$/u.test(sourcePath);
}

function isProductionRustSource(sourcePath: string): boolean {
  return (
    sourcePath.endsWith(".rs") &&
    !/(?:^|\/)tests?(?:\/|\.rs$)/u.test(sourcePath) &&
    !sourcePath.endsWith("_test.rs")
  );
}

function productionRustPrefix(source: string): string {
  return source.split(/\n#\[cfg\(test\)\]\s*\nmod\s+/u)[0] ?? source;
}
