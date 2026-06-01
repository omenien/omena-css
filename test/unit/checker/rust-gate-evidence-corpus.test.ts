import { describe, expect, it } from "vitest";
import {
  RUST_GATE_EVIDENCE_CORPUS,
  RUST_GATE_EVIDENCE_VARIANTS,
} from "../../../scripts/rust-gate-evidence-corpus";

describe("rust gate evidence corpus", () => {
  it("uses unique labels", () => {
    const labels = RUST_GATE_EVIDENCE_CORPUS.map((entry) => entry.label);
    expect(new Set(labels).size).toBe(labels.length);
  });

  it("targets routable pnpm commands only", () => {
    for (const entry of RUST_GATE_EVIDENCE_CORPUS) {
      expect(entry.argv.length).toBeGreaterThan(0);
      expect(isRoutablePnpmCommand(entry.argv)).toBe(true);
    }
  });

  it("uses unique variant labels", () => {
    const labels = RUST_GATE_EVIDENCE_VARIANTS.map((variant) => variant.label);
    expect(new Set(labels).size).toBe(labels.length);
  });

  it("references only declared variants", () => {
    const labels = new Set(RUST_GATE_EVIDENCE_VARIANTS.map((variant) => variant.label));
    for (const entry of RUST_GATE_EVIDENCE_CORPUS) {
      for (const variant of entry.variants ?? []) {
        expect(labels.has(variant)).toBe(true);
      }
    }
  });
});

function isRoutablePnpmCommand(argv: readonly string[]): boolean {
  if (argv[0]?.startsWith("check:")) return true;
  return argv[0] === "omena-check" && (argv[1] === "run" || argv[1] === "bundle");
}
