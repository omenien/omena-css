import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  buildAffectedCheckPlan,
  CI_PROBE_PROFILES,
  loadCheckManifest,
  resolveGateTarget,
} from "../../../packages/check-orchestrator/src";

const repoRoot = path.resolve(import.meta.dirname, "../../..");

describe("affected check planning", () => {
  it("selects the focused CLI profile for CLI-only changes", () => {
    const plan = buildAffectedCheckPlan([
      "rust/crates/omena-cli/src/main.rs",
      "scripts/check-rust-omena-cli-trace.ts",
    ]);

    expect(plan.profiles).toEqual(["rust-cli"]);
    expect(plan.requiresFullCi).toBe(false);
  });

  it("adds Linux performance evidence for performance-sensitive code", () => {
    const plan = buildAffectedCheckPlan(["rust/crates/omena-streaming-ifds/src/demand.rs"]);

    expect(plan.profiles).toEqual(["rust-workspace", "linux-benchmark"]);
    expect(plan.requiresFullCi).toBe(false);
  });

  it("fails closed for workflow topology and unknown paths", () => {
    const plan = buildAffectedCheckPlan([".github/workflows/ci.yml", "schema/new-format.json"]);

    expect(plan.profiles).toEqual(["orchestrator"]);
    expect(plan.requiresFullCi).toBe(true);
    expect(plan.reasons.filter((reason) => reason.requiresFullCi)).toHaveLength(2);
  });

  it("ignores local planning documents", () => {
    const plan = buildAffectedCheckPlan([".personal_docs/codex/goal.md"]);

    expect(plan.profiles).toEqual([]);
    expect(plan.reasons).toEqual([]);
    expect(plan.requiresFullCi).toBe(false);
  });
});

describe("CI probe profiles", () => {
  it("resolve to manual, declared check targets", () => {
    const manifest = loadCheckManifest(repoRoot);

    for (const profile of CI_PROBE_PROFILES) {
      const target = resolveGateTarget(manifest, profile.target);
      expect(target, profile.id).not.toBeNull();
      expect(target?.ciTier, profile.id).toBe("manual");
    }
  });

  it("keeps workflow choices synchronized with the profile registry", () => {
    const workflow = readFileSync(path.join(repoRoot, ".github/workflows/ci-probe.yml"), "utf8");
    const optionsBlock = workflow.match(/        options:\n(?<options>(?:          - .+\n)+)/)
      ?.groups?.options;
    const workflowProfiles =
      optionsBlock
        ?.split("\n")
        .map((line) => line.match(/^          - (.+)$/)?.[1])
        .filter((value): value is string => Boolean(value)) ?? [];

    expect(workflowProfiles).toEqual(CI_PROBE_PROFILES.map((profile) => profile.id));
  });
});
