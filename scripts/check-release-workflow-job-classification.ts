import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  releaseWorkflowJobDigest,
  releaseWorkflowJobFacts,
  type ReleaseWorkflowJobFact,
} from "./lib/release-workflow-classification.ts";

interface AllowlistEntry {
  readonly workflow: string;
  readonly job: string;
  readonly semanticDigest: string;
}

interface Allowlist {
  readonly schemaVersion: "0";
  readonly product: "release.non-publishing-workflow-jobs";
  readonly expectedEnvironmentProtection: boolean;
  readonly entryCount: number;
  readonly entries: readonly AllowlistEntry[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const allowlistPath = path.join(repoRoot, "docs/releases/release-workflow-job-allowlist.json");
const allowlist = JSON.parse(readFileSync(allowlistPath, "utf8")) as Allowlist;
const mutation = process.argv.find((arg) => arg.startsWith("--inject-"));
const facts = releaseWorkflowJobFacts(repoRoot).map(cloneFact);
const entries = allowlist.entries.map((entry) => ({ ...entry }));

if (mutation === "--inject-unused-entry") {
  entries.push({ workflow: "missing.yml", job: "missing", semanticDigest: "0".repeat(64) });
}
if (mutation === "--inject-unlisted-push-job") {
  const target = requiredFact(facts, "_publish-crate-train.yml:publish");
  Reflect.set(target, "events", ["push"]);
}
if (mutation === "--inject-action-sha-bump") {
  mutateFirstUses(
    requiredListedFact(facts, entries),
    (uses) => `${uses.split("@")[0]}@${"f".repeat(40)}`,
  );
}
if (mutation === "--inject-publishing-step") {
  const target = requiredListedFact(facts, entries);
  const steps = Array.isArray(target.job.steps) ? [...target.job.steps] : [];
  Reflect.set(target.job, "steps", [...steps, { name: "publish mutation", run: "npm publish" }]);
}
if (mutation === "--inject-guard-deletion") {
  const target = facts.find(
    (fact) => entries.some((entry) => entryKey(entry) === fact.key) && deepHasKey(fact.job, "if"),
  );
  assert(target, "guard-deletion mutation target is absent");
  assert(deleteFirstKey(target.job, "if"), "guard-deletion mutation did not delete a guard");
}
if (mutation === "--inject-with-value-flip") {
  const target = facts.find(
    (fact) => entries.some((entry) => entryKey(entry) === fact.key) && deepHasKey(fact.job, "with"),
  );
  assert(target, "with-value mutation target is absent");
  assert(flipFirstWithValue(target.job), "with-value mutation did not change a value");
}
if (mutation === "--inject-registry-token-env") {
  const target = requiredListedFact(facts, entries);
  Reflect.set(target.job, "env", {
    ...asRecord(target.job.env),
    NPM_TOKEN: "${{ secrets.NPM_TOKEN }}",
  });
}

assert.equal(allowlist.schemaVersion, "0");
assert.equal(allowlist.product, "release.non-publishing-workflow-jobs");
assert.equal(typeof allowlist.expectedEnvironmentProtection, "boolean");
assert.equal(
  allowlist.entryCount,
  allowlist.entries.length,
  "allowlist entryCount must pin the hand-reviewed entry set",
);
assert.equal(
  new Set(entries.map(entryKey)).size,
  entries.length,
  "allowlist entries must be unique",
);

const factByKey = new Map(facts.map((fact) => [fact.key, fact]));
for (const entry of entries) {
  const key = entryKey(entry);
  const fact = factByKey.get(key);
  assert(fact, `unused release workflow allowlist entry: ${key}`);
  assert.equal(
    releaseWorkflowJobDigest(fact),
    entry.semanticDigest,
    `${key} semantic job body drifted; review publication capability before updating the digest`,
  );
}

const listedKeys = new Set(entries.map(entryKey));
const unlisted = facts.filter((fact) => !listedKeys.has(fact.key));
for (const fact of unlisted) {
  assert.ok(
    looksPublicationCapable(fact),
    `${fact.key} is neither a reviewed non-publishing job nor recognizably publication-capable`,
  );
  assert.ok(
    fact.events.length > 0 &&
      fact.events.every((event) => event === "workflow_dispatch" || event === "workflow_call"),
    `${fact.key} publication-capable job must be reachable only by workflow_dispatch/workflow_call`,
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.workflow-job-classification",
      workflowJobCount: facts.length,
      nonPublishingAllowlistCount: entries.length,
      dispatchOnlyPublicationJobCount: unlisted.length,
      expectedEnvironmentProtection: allowlist.expectedEnvironmentProtection,
      usesRefDigestNeutrality: true,
    },
    null,
    2,
  )}\n`,
);

function cloneFact(fact: ReleaseWorkflowJobFact): ReleaseWorkflowJobFact {
  return structuredClone(fact);
}

function entryKey(entry: AllowlistEntry): string {
  return `${entry.workflow}:${entry.job}`;
}

function requiredFact(
  factsToSearch: readonly ReleaseWorkflowJobFact[],
  key: string,
): ReleaseWorkflowJobFact {
  const fact = factsToSearch.find((candidate) => candidate.key === key);
  assert(fact, `workflow job fixture is absent: ${key}`);
  return fact;
}

function requiredListedFact(
  factsToSearch: readonly ReleaseWorkflowJobFact[],
  listedEntries: readonly AllowlistEntry[],
): ReleaseWorkflowJobFact {
  const listed = new Set(listedEntries.map(entryKey));
  const fact = factsToSearch.find(
    (candidate) => listed.has(candidate.key) && deepHasKey(candidate.job, "uses"),
  );
  assert(fact, "listed workflow job with an action use is absent");
  return fact;
}

function looksPublicationCapable(fact: ReleaseWorkflowJobFact): boolean {
  const source = JSON.stringify(fact.job).toLowerCase();
  return [
    "cargo publish",
    "npm publish",
    "vsce publish",
    "action-gh-release",
    "gh release",
    "trusted-publish",
    "attest-build-provenance",
    "marketplace",
  ].some((marker) => source.includes(marker));
}

function mutateFirstUses(value: unknown, mutate: (uses: string) => string): boolean {
  if (Array.isArray(value)) {
    return value.some((item) => mutateFirstUses(item, mutate));
  }
  if (value === null || typeof value !== "object") return false;
  for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
    if (key === "uses" && typeof child === "string" && child.includes("@")) {
      Reflect.set(value as object, key, mutate(child));
      return true;
    }
    if (mutateFirstUses(child, mutate)) return true;
  }
  return false;
}

function deepHasKey(value: unknown, target: string): boolean {
  if (Array.isArray(value)) return value.some((item) => deepHasKey(item, target));
  if (value === null || typeof value !== "object") return false;
  return Object.entries(value as Record<string, unknown>).some(
    ([key, child]) => key === target || deepHasKey(child, target),
  );
}

function deleteFirstKey(value: unknown, target: string): boolean {
  if (Array.isArray(value)) return value.some((item) => deleteFirstKey(item, target));
  if (value === null || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  if (Object.hasOwn(record, target)) return Reflect.deleteProperty(record, target);
  return Object.values(record).some((child) => deleteFirstKey(child, target));
}

function flipFirstWithValue(value: unknown): boolean {
  if (Array.isArray(value)) return value.some(flipFirstWithValue);
  if (value === null || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  const withMap = asRecord(record.with);
  const firstKey = Object.keys(withMap)[0];
  if (firstKey) {
    const before = withMap[firstKey];
    withMap[firstKey] = typeof before === "boolean" ? !before : `${String(before)}-mutated`;
    record.with = withMap;
    return true;
  }
  return Object.values(record).some(flipFirstWithValue);
}

function asRecord(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}
