import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { DECLARED_CHECK_GATES } from "../packages/check-orchestrator/src/manifest/declared.ts";
import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type EvidenceKind = "writerBacked" | "writerlessNumeric" | "authoringRequired";

interface RegistryEntry {
  readonly artifact: string;
  readonly kind: EvidenceKind;
  readonly checker: string;
  readonly rerunCommand: string;
}

interface Registry {
  readonly schemaVersion: "0";
  readonly product: "omena.release-print-sensitive-evidence";
  readonly entries: readonly RegistryEntry[];
}

interface DomainCensus {
  readonly exit: { readonly domain: { readonly trackedFileCount: number } };
}

interface BoundaryReviewFile {
  readonly measurementBase: string;
  readonly reviews: readonly {
    readonly id: string;
    readonly subject: { readonly measurementPaths: readonly string[] };
    readonly measurements: {
      readonly apiSurfaceStability: { readonly value: number; readonly response: string };
    };
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const registry = readJson<Registry>("rust/release-print-sensitive-evidence.json");
const mutation = process.argv.find((arg) => arg.startsWith("--inject-"));

assert.equal(registry.schemaVersion, "0");
assert.equal(registry.product, "omena.release-print-sensitive-evidence");
assert.deepEqual(
  registry.entries.map((entry) => entry.artifact),
  [
    "rust/omena-domain-claim-census.json",
    "rust/product-surface-boundary-reviews.json",
    "docs/releases/manifest.json",
  ],
  "print-sensitive registry membership and order are reviewed data",
);
assert.equal(
  new Set(registry.entries.map((entry) => entry.artifact)).size,
  registry.entries.length,
);
for (const entry of registry.entries) {
  assert.ok(["writerBacked", "writerlessNumeric", "authoringRequired"].includes(entry.kind));
  assert.ok(entry.checker.length > 0 && entry.rerunCommand.startsWith("pnpm "));
}

const releaseVerify = DECLARED_CHECK_GATES.find((gate) => gate.id === "release/release/verify");
assert(releaseVerify && releaseVerify.kind === "bundle");
const verifyDeps = new Set("deps" in releaseVerify ? releaseVerify.deps : []);
if (mutation === "--inject-bundle-member-drop") verifyDeps.delete(registry.entries[0]!.checker);
for (const entry of registry.entries) {
  assert.ok(
    verifyDeps.has(entry.checker),
    `release/release/verify must directly include print-sensitive checker ${entry.checker}`,
  );
}

const domain = readJson<DomainCensus>("rust/omena-domain-claim-census.json");
const trackedFiles = evidenceScanSurface
  .gitOutput(["ls-files", "-z"])
  .split("\0")
  .filter(Boolean).length;
const recordedTrackedFiles =
  domain.exit.domain.trackedFileCount + (mutation === "--inject-writer-staleness" ? 1 : 0);
assert.equal(
  recordedTrackedFiles,
  trackedFiles,
  "writer-backed domain census is stale; stage the complete print and rerun its writer",
);

const boundary = readJson<BoundaryReviewFile>("rust/product-surface-boundary-reviews.json");
for (const [index, review] of boundary.reviews.entries()) {
  const measured = Number.parseInt(
    execFileSync(
      "git",
      [
        "rev-list",
        "--count",
        `${boundary.measurementBase}..HEAD`,
        "--",
        ...review.subject.measurementPaths,
      ],
      { cwd: repoRoot, encoding: "utf8" },
    ).trim(),
    10,
  );
  const recorded =
    review.measurements.apiSurfaceStability.value +
    (mutation === "--inject-numeric-staleness" && index === 0 ? 1 : 0);
  assert.equal(recorded, measured, `${review.id} writer-less commit count is stale`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.print-sensitive-evidence",
      memberCount: registry.entries.length,
      kindCounts: Object.fromEntries(
        [...new Set(registry.entries.map((entry) => entry.kind))]
          .toSorted()
          .map((kind) => [kind, registry.entries.filter((entry) => entry.kind === kind).length]),
      ),
      releaseVerifyDirectCheckerCount: registry.entries.length,
      trackedFileCount: trackedFiles,
    },
    null,
    2,
  )}\n`,
);

function readJson<T>(relativePath: string): T {
  return JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8")) as T;
}
