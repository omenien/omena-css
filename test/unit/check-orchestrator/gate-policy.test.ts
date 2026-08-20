import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { loadCheckManifest } from "../../../packages/check-orchestrator/src/manifest/index";

const CI_YML = `name: CI
on:
  push:
    branches: [master]
jobs:
  leaf:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run docs/site
  ci-required:
    needs:
      - leaf
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
`;

function policyRoot(policy: object | null): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "omena-gate-policy-"));
  writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({
      name: "omena-css",
      scripts: {
        "omena-check": "node ./check.js",
        "check:docs-site": "node ./scripts/check-docs-site.ts",
      },
    }),
  );
  mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
  writeFileSync(path.join(root, ".github/workflows/ci.yml"), CI_YML);
  mkdirSync(path.join(root, "packages/check-orchestrator"), { recursive: true });
  if (policy) {
    writeFileSync(
      path.join(root, "packages/check-orchestrator/gate-policy.json"),
      JSON.stringify(policy, null, 2),
    );
  }
  return root;
}

const BASE_POLICY = {
  schemaVersion: "0",
  product: "omena.check-orchestrator.gate-policy",
  template: { reviewIntervalDays: 90, advisoryHoldReasons: ["demotion-staged"] },
  costSource: "static-enumeration-2026-08-18",
  escapeHatch: {
    maxGateCount: 156,
    owner: "check-orchestrator maintainers",
    reviewedAt: "2026-08-20",
    reviewAfter: "2026-11-18",
  },
  lanes: { bundles: [], singles: ["docs/site"], ciJobs: [] },
  // The fixture's live escape-hatch population is exactly tooling/omena-check
  // (docs/site is tier-unclassified in this root); the digest pins that pair.
  governedLeafCriteria: { "manual-tool-with-named-consumer": 1 },
  governedLeafCriteriaDigest: "18776702947c2a0560f5e35f0112e387d0f09a6994a446858ea6d3cc8483e32e",
  records: [
    {
      gateId: "docs/site",
      stage: "blocking",
      reviewedAt: "2026-08-20",
      reviewAfter: "2026-11-18",
      evidence: "fixture",
    },
  ],
};

function diagnosticsFor(policy: object | null, today = "2026-08-20") {
  process.env.OMENA_GATE_POLICY_TODAY = today;
  const manifest = loadCheckManifest(policyRoot(policy), { declaredGates: [] });
  return manifest.diagnostics.filter((diagnostic) => diagnostic.code.startsWith("gate-policy"));
}

afterEach(() => {
  delete process.env.OMENA_GATE_POLICY_TODAY;
});

describe("gate policy registry (promotion policy)", () => {
  it("is quiet on a coherent policy", () => {
    expect(diagnosticsFor(BASE_POLICY)).toEqual([]);
  });

  it("RED-PROOF: an expired review REDs with the recertification path in the message", () => {
    const diagnostics = diagnosticsFor(BASE_POLICY, "2026-11-19");
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "gate-policy-review-expired",
          message: expect.stringContaining("set reviewedAt=today and reviewAfter=today+90d"),
        }),
      ]),
    );
  });

  it("RED-PROOF: interval drift REDs", () => {
    const drifted = structuredClone(BASE_POLICY);
    drifted.records[0]!.reviewAfter = "2026-11-17";
    expect(diagnosticsFor(drifted)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-policy-interval-drift" })]),
    );
  });

  it("RED-PROOF: a staged value contradicting the derived strength REDs unless a vocabulary holdReason carries it", () => {
    const mismatched = structuredClone(BASE_POLICY);
    mismatched.records[0]!.stage = "advisory";
    expect(diagnosticsFor(mismatched)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-policy-stage-mismatch" })]),
    );

    const held = structuredClone(BASE_POLICY);
    held.records[0]!.stage = "advisory";
    (held.records[0] as { holdReason?: string }).holdReason = "demotion-staged";
    expect(diagnosticsFor(held)).toEqual([]);

    const badReason = structuredClone(BASE_POLICY);
    badReason.records[0]!.stage = "advisory";
    (badReason.records[0] as { holdReason?: string }).holdReason = "because";
    expect(diagnosticsFor(badReason)).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ code: "gate-policy-unknown-hold-reason" }),
      ]),
    );
  });

  it("RED-PROOF: a record naming a retired gate REDs (same-commit retirement duty)", () => {
    const orphan = structuredClone(BASE_POLICY);
    orphan.records[0]!.gateId = "docs/retired-gate";
    expect(diagnosticsFor(orphan)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-policy-unknown-gate" })]),
    );
  });

  it("RED-PROOF: deleting the policy file from an orchestrator-bearing root REDs", () => {
    expect(diagnosticsFor(null)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-policy-missing" })]),
    );
  });
});
