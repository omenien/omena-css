import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadCheckManifest } from "../packages/check-orchestrator/src/manifest/index.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const checkerPath = "scripts/check-rust-omena-debt-clock.ts";
const ledgerPath = "rust/omena-debt-ledger.json";
const clockPath = "scripts/omena-debt-reference-clock.json";
const matrixPath = "rust/omena-product-path-matrix.json";
const codeownersPath = ".github/CODEOWNERS";
const packageJsonPath = "package.json";

const validKinds = new Set(["span-shim", "dual-arm-oracle", "whitelist", "compat-window"]);
const expectedGateId = "rust/omena-debt-clock";
const expectedReceiptClients = [
  "obligation-family-carrier-retirement",
  "dialect-seed-dual-arm-date-window",
  "windows-required-promotion-window",
  "stable-node-key-string-arm",
  "transform-ir-span-shim",
  "z3-discharge-compat-window",
  "napi-json-compat-window",
  "omena-cli-facts-alias-window",
] as const;

interface ReferenceClockManifest {
  readonly schemaVersion: string;
  readonly reference_date: string;
}

interface DebtLedgerManifest {
  readonly schemaVersion: string;
  readonly product: string;
  readonly staleness_policy: string;
  readonly entries: readonly DebtEntry[];
  readonly ratchets: readonly RatchetRegistration[];
  readonly client_receipts: readonly ClientReceipt[];
  readonly fences: readonly DebtFence[];
}

interface DebtEntry {
  readonly id: string;
  readonly mechanism: string;
  readonly kind: string;
  readonly introduced_in: string;
  readonly expiry: {
    readonly after_reference_date: string;
  };
  readonly on_expiry: string;
  readonly renewals?: readonly DebtRenewal[];
}

interface DebtRenewal {
  readonly after_reference_date: string;
  readonly reason: string;
}

interface RatchetRegistration {
  readonly ratchet_id: string;
  readonly manifest_path: string;
  readonly owning_check: string;
}

interface ClientReceipt {
  readonly client: string;
  readonly accepted_kinds: readonly string[];
  readonly authority: string;
}

interface DebtFence {
  readonly id: string;
  readonly status: string;
  readonly reason: string;
}

const clock = readJson<ReferenceClockManifest>(clockPath);
const ledger = readJson<DebtLedgerManifest>(ledgerPath);

assertClock(clock);
assertLedger(ledger, clock.reference_date);
assertCheckerDoesNotUseRuntimeClock();
assertCodeowners();
assertProductMatrix();

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-debt-clock",
      referenceDate: clock.reference_date,
      expiryEntries: ledger.entries.length,
      ratchetRegistrations: ledger.ratchets.length,
      receiptClients: ledger.client_receipts.length,
      fences: ledger.fences.length,
    },
    null,
    2,
  )}\n`,
);

function readJson<T>(relativePath: string): T {
  const absolutePath = path.join(repoRoot, relativePath);
  assert.ok(existsSync(absolutePath), `${relativePath} must exist`);
  return JSON.parse(readFileSync(absolutePath, "utf8")) as T;
}

function assertClock(manifest: ReferenceClockManifest): void {
  assert.equal(manifest.schemaVersion, "0", `${clockPath} schemaVersion must be 0`);
  assertIsoDate(manifest.reference_date, `${clockPath} reference_date`);
}

function assertLedger(manifest: DebtLedgerManifest, referenceDate: string): void {
  assert.equal(manifest.schemaVersion, "0", `${ledgerPath} schemaVersion must be 0`);
  assert.equal(manifest.product, "omena-css.debt-ledger", `${ledgerPath} product mismatch`);
  assertSemanticToken(manifest.staleness_policy, `${ledgerPath} staleness_policy`);
  assert.ok(
    manifest.staleness_policy.includes("reference date") &&
      manifest.staleness_policy.includes("stale reference date"),
    `${ledgerPath} must state how committed-clock staleness is handled`,
  );
  assert.ok(Array.isArray(manifest.entries), `${ledgerPath} entries must be an array`);
  assert.ok(Array.isArray(manifest.ratchets), `${ledgerPath} ratchets must be an array`);
  assert.ok(
    Array.isArray(manifest.client_receipts),
    `${ledgerPath} client_receipts must be an array`,
  );
  assert.ok(Array.isArray(manifest.fences), `${ledgerPath} fences must be an array`);

  assert.ok(manifest.entries.length > 0, `${ledgerPath} must contain a real expiry entry`);
  assert.ok(manifest.ratchets.length > 0, `${ledgerPath} must contain a real ratchet registration`);
  assert.ok(
    manifest.entries.some(
      (entry) => entry.mechanism === "retiring-engine-style-parser" && entry.kind === "whitelist",
    ),
    `${ledgerPath} must seed the retiring style parser whitelist entry`,
  );
  assert.ok(
    manifest.ratchets.some(
      (ratchet) =>
        ratchet.ratchet_id === "god-file-loc-ceilings" &&
        ratchet.manifest_path === "scripts/rust-godfile-ceilings.json" &&
        ratchet.owning_check === "check:rust-core-layer-hygiene",
    ),
    `${ledgerPath} must register the god-file LOC ceiling ratchet`,
  );

  for (const entry of manifest.entries) {
    assertDebtEntry(entry, referenceDate);
  }
  assertRegisteredClientExpiries(manifest.entries);
  assertNoExpiredEntries(manifest.entries, referenceDate);
  assertExpiryPredicateSelfTest();

  for (const ratchet of manifest.ratchets) {
    assertRatchetRegistration(ratchet);
  }
  assertClientReceipts(manifest.client_receipts);
  assertFences(manifest.fences);
}

function assertRegisteredClientExpiries(entries: readonly DebtEntry[]): void {
  const byId = new Map(entries.map((entry) => [entry.id, entry]));
  const stableNodeKey = byId.get("stable-node-key-string-arm");
  const spanShim = byId.get("transform-ir-span-shim");
  const dialectSeeds = byId.get("dialect-seed-known-failure-review");
  assert.ok(stableNodeKey, "stable node key build-script expiry must be registered");
  assert.ok(spanShim, "transform IR span shim expiry must be registered");
  assert.ok(dialectSeeds, "dialect seed review date must be registered");

  const stableNodeBuild = readFileSync(
    path.join(repoRoot, "rust/crates/omena-transform-cst/build.rs"),
    "utf8",
  );
  const unixDay = stableNodeBuild.match(
    /STABLE_NODE_KEY_STRING_ARM_EXPIRY_UNIX_DAY: u64 = ([\d_]+);/u,
  );
  assert.equal(unixDay?.[1]?.replaceAll("_", ""), "20727");
  assert.equal(stableNodeKey.expiry.after_reference_date, "2026-10-01");

  const spanBaseline = readJson<{
    readonly expiry: { readonly notAfterUtcDate: string };
  }>("scripts/transform-ir-span-shim-baseline.json");
  assert.equal(spanShim.expiry.after_reference_date, spanBaseline.expiry.notAfterUtcDate);

  const seedReviewDates = [
    "rust/crates/omena-diff-test/known-failures/sass-spec-seed-policy.toml",
    "rust/crates/omena-diff-test/known-failures/less-seed-policy.toml",
  ].map((policyPath) => {
    const policy = readFileSync(path.join(repoRoot, policyPath), "utf8");
    const match = policy.match(/^review_after = "(\d{4}-\d{2}-\d{2})"$/mu);
    assert.ok(match, `${policyPath} must declare review_after`);
    return match[1];
  });
  assert.deepEqual(new Set(seedReviewDates), new Set([dialectSeeds.expiry.after_reference_date]));
}

function assertDebtEntry(entry: DebtEntry, referenceDate: string): void {
  assertExactKeys(entry, [
    "expiry",
    "id",
    "introduced_in",
    "kind",
    "mechanism",
    "on_expiry",
    ...(entry.renewals === undefined ? [] : ["renewals"]),
  ]);
  assertSemanticToken(entry.id, "entry id");
  assertSemanticToken(entry.mechanism, `${entry.id} mechanism`);
  assertSemanticToken(entry.introduced_in, `${entry.id} introduced_in`);
  assertSemanticToken(entry.on_expiry, `${entry.id} on_expiry`);
  assert.ok(validKinds.has(entry.kind), `${entry.id} has unsupported debt kind ${entry.kind}`);
  assertIntroducedIn(entry.introduced_in, `${entry.id} introduced_in`);
  assert.ok(entry.expiry && typeof entry.expiry === "object", `${entry.id} must contain expiry`);
  assertExactKeys(entry.expiry, ["after_reference_date"]);
  assertIsoDate(entry.expiry.after_reference_date, `${entry.id} expiry.after_reference_date`);
  assert.ok(
    entry.expiry.after_reference_date >= referenceDate || hasActiveRenewal(entry, referenceDate),
    `${entry.mechanism} expired on ${entry.expiry.after_reference_date}; ${entry.on_expiry}`,
  );

  if (entry.renewals !== undefined) {
    assert.ok(Array.isArray(entry.renewals), `${entry.id} renewals must be an array`);
    for (const renewal of entry.renewals) {
      assertExactKeys(renewal, ["after_reference_date", "reason"]);
      assertIsoDate(renewal.after_reference_date, `${entry.id} renewal.after_reference_date`);
      assertSemanticToken(renewal.reason, `${entry.id} renewal reason`);
    }
  }
}

function assertNoExpiredEntries(entries: readonly DebtEntry[], referenceDate: string): void {
  const expired = entries.filter(
    (entry) =>
      entry.expiry.after_reference_date < referenceDate && !hasActiveRenewal(entry, referenceDate),
  );
  assert.equal(
    expired.length,
    0,
    `expired debt entries remain present:\n${expired
      .map(
        (entry) => `  ${entry.id}: ${entry.mechanism} expired ${entry.expiry.after_reference_date}`,
      )
      .join("\n")}`,
  );
}

function assertExpiryPredicateSelfTest(): void {
  const fixture: DebtEntry = {
    id: "expired-whitelist-fixture",
    mechanism: "expired-whitelist-fixture",
    kind: "whitelist",
    introduced_in: "2026-07-01",
    expiry: { after_reference_date: "2026-07-06" },
    on_expiry: "Remove the expired fixture.",
  };
  assert.throws(
    () => assertNoExpiredEntries([fixture], "2026-07-07"),
    /expired debt entries/u,
    "expiry predicate self-test must flag a past-expiry entry",
  );
  assert.doesNotThrow(() =>
    assertNoExpiredEntries(
      [
        {
          ...fixture,
          renewals: [{ after_reference_date: "2026-07-08", reason: "fixture renewal" }],
        },
      ],
      "2026-07-07",
    ),
  );
}

function hasActiveRenewal(entry: DebtEntry, referenceDate: string): boolean {
  return (entry.renewals ?? []).some((renewal) => renewal.after_reference_date >= referenceDate);
}

function assertRatchetRegistration(ratchet: RatchetRegistration): void {
  assertExactKeys(ratchet, ["manifest_path", "owning_check", "ratchet_id"]);
  assertSemanticToken(ratchet.ratchet_id, "ratchet_id");
  assertSemanticToken(ratchet.manifest_path, `${ratchet.ratchet_id} manifest_path`);
  assertSemanticToken(ratchet.owning_check, `${ratchet.ratchet_id} owning_check`);
  assert.ok(
    existsSync(path.join(repoRoot, ratchet.manifest_path)),
    `${ratchet.ratchet_id} manifest_path does not exist: ${ratchet.manifest_path}`,
  );

  const packageJson = readJson<{ readonly scripts?: Record<string, string> }>(packageJsonPath);
  assert.ok(
    packageJson.scripts?.[ratchet.owning_check],
    `${ratchet.ratchet_id} owning_check is not a package script: ${ratchet.owning_check}`,
  );

  const manifest = loadCheckManifest(repoRoot);
  assert.ok(
    manifest.gates.some((gate) => gate.scriptName === ratchet.owning_check),
    `${ratchet.ratchet_id} owning_check is not registered in the check orchestrator`,
  );
}

function assertClientReceipts(receipts: readonly ClientReceipt[]): void {
  const clients = new Set(receipts.map((receipt) => receipt.client));
  assert.deepEqual(
    [...clients].toSorted(),
    [...expectedReceiptClients].toSorted(),
    "client receipt registry must enumerate every deferred debt-clock client",
  );
  for (const receipt of receipts) {
    assertExactKeys(receipt, ["accepted_kinds", "authority", "client"]);
    assertSemanticToken(receipt.client, `${receipt.client} client`);
    assertSemanticToken(receipt.authority, `${receipt.client} authority`);
    assert.ok(receipt.accepted_kinds.length > 0, `${receipt.client} needs accepted kinds`);
    for (const kind of receipt.accepted_kinds) {
      assert.ok(validKinds.has(kind), `${receipt.client} accepts unsupported debt kind ${kind}`);
    }
  }
}

function assertFences(fences: readonly DebtFence[]): void {
  const byId = new Map(fences.map((fence) => [fence.id, fence]));
  assert.equal(byId.get("v-suffix-mass-rename-sweep")?.status, "rejected");
  assert.equal(byId.get("train-ordinal-expiry")?.status, "parked");
  for (const fence of fences) {
    assertExactKeys(fence, ["id", "reason", "status"]);
    assertSemanticToken(fence.id, `${fence.id} fence`);
    assertSemanticToken(fence.status, `${fence.id} status`);
    assertSemanticToken(fence.reason, `${fence.id} reason`);
  }
}

function assertCodeowners(): void {
  const codeowners = readFileSync(path.join(repoRoot, codeownersPath), "utf8");
  for (const governedPath of [ledgerPath, clockPath]) {
    assert.ok(
      codeowners.split("\n").some((line) => line.trim().startsWith(`${governedPath} `)),
      `${codeownersPath} must govern ${governedPath}`,
    );
  }
}

function assertProductMatrix(): void {
  const matrix = readJson<{
    readonly entries: readonly { readonly crate: string; readonly gates?: readonly string[] }[];
  }>(matrixPath);
  assert.ok(
    matrix.entries.some((entry) => entry.gates?.includes(expectedGateId)),
    `${matrixPath} must include ${expectedGateId}`,
  );

  const manifest = loadCheckManifest(repoRoot);
  assert.ok(
    manifest.gates.some((gate) => gate.id === expectedGateId),
    `${expectedGateId} must be inventory-derived from a registered package script`,
  );
}

function assertCheckerDoesNotUseRuntimeClock(): void {
  const source = readFileSync(path.join(repoRoot, checkerPath), "utf8");
  for (const forbidden of [
    ["Date", ".now"].join(""),
    ["new ", "Date", "("].join(""),
    ["get", "Time", "("].join(""),
  ]) {
    assert.equal(
      source.includes(forbidden),
      false,
      `${checkerPath} must not use runtime clock API ${forbidden}`,
    );
  }
}

function assertIsoDate(value: string, label: string): void {
  assert.equal(typeof value, "string", `${label} must be a string`);
  assert.ok(/^\d{4}-\d{2}-\d{2}$/u.test(value), `${label} must be YYYY-MM-DD`);
}

function assertIntroducedIn(value: string, label: string): void {
  if (/^\d{4}-\d{2}-\d{2}$/u.test(value)) {
    return;
  }
  assert.ok(
    /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/u.test(value),
    `${label} must be an ISO date or a kebab-case mechanism name`,
  );
}

function assertSemanticToken(value: string, label: string): void {
  assert.equal(typeof value, "string", `${label} must be a string`);
  assert.ok(value.trim().length > 0, `${label} must not be empty`);
  const disallowedWords = [["wa", "ve"].join(""), ["north", "star"].join("")];
  const disallowedPrefix = ["go", "al", "-"].join("");
  assert.ok(!/\bg\d+\b/u.test(value), `${label} contains private planning vocabulary`);
  assert.ok(!value.includes(disallowedPrefix), `${label} contains private planning vocabulary`);
  for (const word of disallowedWords) {
    assert.ok(
      !new RegExp(`\\b${escapeRegExp(word)}\\b`, "u").test(value),
      `${label} contains private planning vocabulary`,
    );
  }
}

function assertExactKeys(value: object, keys: readonly string[]): void {
  assert.deepEqual(Object.keys(value).toSorted(), [...keys].toSorted());
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
