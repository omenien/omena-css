import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/feature-resolved-product-reachability
 *
 * Witnesses, per product root, which of omena-query's THEORETICAL-RIGOR features
 * are actually active in the shipped binary — resolved via `cargo tree -e features`
 * (metadata-only, no compile). This is the only mechanism that can witness
 * "theory ships OFF": omena-categorical is a NON-optional R1 dep, so there is no
 * optional EDGE for the layer/matrix gates to flag — only feature resolution
 * distinguishes "crate linked" from "analysis feature active".
 *
 * REPORT-ONLY by default (always exits 0, emits the per-root table + any drift).
 * The hard-fail variant (`OMENA_FEATURE_REACHABILITY_HARDFAIL=1`) is a surfaced,
 * user-gated decision (deferral #79-90) — NOT enabled by default.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rustDir = path.join(repoRoot, "rust");
const allowListPath = path.join(rustDir, "feature-resolved-product-reachability.json");

interface ReachabilityRoot {
  readonly root: string;
  readonly declaredReachable: readonly string[];
  readonly declaredDormant: readonly string[];
}

interface ReachabilityAllowList {
  readonly schemaVersion: string;
  readonly product: string;
  readonly queryCrate: string;
  readonly theoryRigorFeatures: readonly string[];
  readonly roots: readonly ReachabilityRoot[];
}

const allowList = JSON.parse(readFileSync(allowListPath, "utf8")) as ReachabilityAllowList;

assert.equal(allowList.schemaVersion, "0", `${allowListPath} must use schemaVersion "0"`);
assert.ok(
  Array.isArray(allowList.roots) && allowList.roots.length > 0,
  `${allowListPath} must declare at least one product root`,
);

const queryCrate = allowList.queryCrate;
const theoryRigorFeatures = new Set(allowList.theoryRigorFeatures);
const featureLinePattern = new RegExp(`${escapeRegExp(queryCrate)} feature "([^"]+)"`);

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function resolveActiveTheoryFeatures(root: string): string[] {
  const output = execFileSync("cargo", ["tree", "-e", "features", "-p", root, "-i", queryCrate], {
    cwd: rustDir,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 64,
  });
  const active = new Set<string>();
  for (const line of output.split("\n")) {
    const match = line.match(featureLinePattern);
    if (match && theoryRigorFeatures.has(match[1])) {
      active.add(match[1]);
    }
  }
  return [...active].toSorted();
}

interface RootResult {
  readonly root: string;
  readonly activeTheoryFeatures: readonly string[];
  readonly declaredReachable: readonly string[];
  readonly declaredDormant: readonly string[];
  readonly missingReachable: readonly string[];
  readonly leakedDormant: readonly string[];
}

const results: RootResult[] = allowList.roots.map((entry) => {
  const active = resolveActiveTheoryFeatures(entry.root);
  const activeSet = new Set(active);
  return {
    root: entry.root,
    activeTheoryFeatures: active,
    declaredReachable: entry.declaredReachable,
    declaredDormant: entry.declaredDormant,
    missingReachable: entry.declaredReachable.filter((feature) => !activeSet.has(feature)),
    leakedDormant: entry.declaredDormant.filter((feature) => activeSet.has(feature)),
  };
});

const drift = results.filter(
  (result) => result.missingReachable.length > 0 || result.leakedDormant.length > 0,
);

const hardFail = process.env.OMENA_FEATURE_REACHABILITY_HARDFAIL === "1";

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: allowList.product,
      queryCrate,
      mode: hardFail ? "hard-fail" : "report-only",
      resolution: "cargo tree -e features -p <root> -i " + queryCrate,
      roots: results,
      driftRootCount: drift.length,
    },
    null,
    2,
  )}\n`,
);

if (drift.length > 0) {
  const summary = drift
    .map(
      (result) =>
        `  ${result.root}: missingReachable=[${result.missingReachable.join(", ")}] leakedDormant=[${result.leakedDormant.join(", ")}]`,
    )
    .join("\n");
  if (hardFail) {
    assert.fail(`feature-resolved product-reachability drift (hard-fail mode):\n${summary}`);
  }
  process.stderr.write(
    `warning: feature-resolved product-reachability drift (report-only; theory feature reachability changed vs the allow-list):\n${summary}\n`,
  );
}
