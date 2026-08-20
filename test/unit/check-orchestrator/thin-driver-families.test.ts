import { describe, expect, it } from "vitest";
import { RUST_SHADOW_FAMILY } from "../../../scripts/lib/rust-shadow-family";
import { QUERY_CONSUMER_FAMILY } from "../../../scripts/lib/query-consumer-family";
import { CONTRACT_PARITY_SMOKE_FAMILY } from "../../../scripts/lib/contract-parity-smoke-family";
import { CONTRACT_PARITY_GOLDEN_FAMILY } from "../../../scripts/lib/contract-parity-golden-family";

// g131-S6: per-family invariants — table rows == former member count, and
// the id set equals the enumerated former single-file drivers (a dropped or
// renamed row is a silent gate-surface change; this arm makes it loud).
describe("thin-driver families (g131-S6)", () => {
  it("rust-shadow family carries exactly the 42 former drivers (14 shared + 28 own corpus)", () => {
    const rows = Object.entries(RUST_SHADOW_FAMILY);
    expect(rows.length).toBe(42);
    expect(rows.filter(([, row]) => row.corpus === "shared").length).toBe(14);
    expect(rows.filter(([, row]) => row.corpus === "own").length).toBe(28);
    for (const [slug, row] of rows) {
      expect(slug).toMatch(/^rust-[a-z0-9-]+$/u);
      expect(typeof row.run).toBe("function");
    }
  });

  it("query-consumer family carries exactly the 6 former drivers", () => {
    expect(Object.keys(QUERY_CONSUMER_FAMILY).toSorted()).toEqual([
      "code-action-query-consumer",
      "completion-query-consumer",
      "explain-expression-query-consumer",
      "rename-query-consumer",
      "source-diagnostics-query-consumer",
      "style-diagnostics-query-consumer",
    ]);
  });

  it("contract-parity families carry the 4 former drivers across 2 tables", () => {
    expect(Object.keys(CONTRACT_PARITY_SMOKE_FAMILY).toSorted()).toEqual([
      "contract-parity-v1-smoke",
      "contract-parity-v2-smoke",
    ]);
    expect(Object.keys(CONTRACT_PARITY_GOLDEN_FAMILY).toSorted()).toEqual([
      "contract-parity-v1-golden",
      "contract-parity-v2-golden",
    ]);
  });
});
