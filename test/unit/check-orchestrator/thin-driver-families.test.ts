import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { RUST_SHADOW_FAMILY } from "../../../scripts/lib/rust-shadow-family";
import { QUERY_CONSUMER_FAMILY } from "../../../scripts/lib/query-consumer-family";
import { CONTRACT_PARITY_SMOKE_FAMILY } from "../../../scripts/lib/contract-parity-smoke-family";
import { CONTRACT_PARITY_GOLDEN_FAMILY } from "../../../scripts/lib/contract-parity-golden-family";

const repoRoot = path.resolve(__dirname, "../../..");

// The enumerated former single-file drivers (goal S6 分解: 14 shared + 28 own).
const RUST_SHADOW_MEMBER_SLUGS = [
  "rust-checker-source-missing-canonical-candidate",
  "rust-checker-source-missing-canonical-producer",
  "rust-checker-style-recovery-canonical-candidate",
  "rust-checker-style-recovery-canonical-producer",
  "rust-checker-style-unused-canonical-candidate",
  "rust-checker-style-unused-canonical-producer",
  "rust-checker-style-unused-consumer-boundary",
  "rust-expression-domain-candidates",
  "rust-expression-domain-canonical-candidate",
  "rust-expression-domain-canonical-producer",
  "rust-expression-domain-compare",
  "rust-expression-domain-evaluator-candidates",
  "rust-expression-domain-fragments",
  "rust-expression-domain-reduced-evaluator",
  "rust-expression-semantics-candidates",
  "rust-expression-semantics-canonical-candidate",
  "rust-expression-semantics-canonical-producer",
  "rust-expression-semantics-evaluator-candidates",
  "rust-expression-semantics-fragments",
  "rust-expression-semantics-match-fragments",
  "rust-expression-semantics-query-fragments",
  "rust-query-plan-compare",
  "rust-selector-usage-fragments",
  "rust-selector-usage-plan-compare",
  "rust-selector-usage-query-fragments",
  "rust-semantic-canonical-candidate",
  "rust-semantic-canonical-producer",
  "rust-semantic-evaluator-candidates",
  "rust-shadow-compare",
  "rust-shadow-smoke",
  "rust-source-resolution-candidates",
  "rust-source-resolution-canonical-candidate",
  "rust-source-resolution-canonical-producer",
  "rust-source-resolution-evaluator-candidates",
  "rust-source-resolution-fragments",
  "rust-source-resolution-match-fragments",
  "rust-source-resolution-plan-compare",
  "rust-source-resolution-query-fragments",
  "rust-source-side-canonical-candidate",
  "rust-source-side-canonical-producer",
  "rust-source-side-evaluator-candidates",
  "rust-type-fact-compare",
] as const;

// g131-S6: per-family invariants — table rows == former member count, and
// the id set equals the enumerated former single-file drivers (a dropped or
// renamed row is a silent gate-surface change; this arm makes it loud).
describe("thin-driver families (g131-S6)", () => {
  it("rust-shadow family carries exactly the 42 former drivers (14 shared + 28 own corpus)", () => {
    const rows = Object.entries(RUST_SHADOW_FAMILY);
    // EXACT id set (stage-5 R2): a dropped or renamed row must be loud, so
    // the pin is the enumerated set, not a count.
    expect(Object.keys(RUST_SHADOW_FAMILY).toSorted()).toEqual(RUST_SHADOW_MEMBER_SLUGS);
    expect(rows.filter(([, row]) => row.corpus === "shared").length).toBe(14);
    expect(rows.filter(([, row]) => row.corpus === "own").length).toBe(28);
    for (const [slug, row] of rows) {
      expect(slug).toMatch(/^rust-[a-z0-9-]+$/u);
      expect(typeof row.run).toBe("function");
    }
  });

  it("SLUG-BODY BINDING (stage-5 R2): every registry row binds run_<slug> — a swapped or repointed body is loud", () => {
    // The lens proved a silent hole: swapping two members' run functions left
    // every surface green. The binding contract is structural in the table
    // source: each row must reference the function named after its own slug.
    const source = readFileSync(path.join(repoRoot, "scripts/lib/rust-shadow-family.ts"), "utf8");
    for (const slug of Object.keys(RUST_SHADOW_FAMILY)) {
      const fn = `run_${slug.replaceAll("-", "_")}`;
      // oxfmt renders rows either inline or expanded — accept both shapes,
      // but the run: binding must name the slug's own function.
      expect(source, `row "${slug}" must bind ${fn}`).toMatch(
        new RegExp(`"${slug}": \\{\\s*corpus: "(?:shared|own)",\\s*run: ${fn},?\\s*\\}`, "u"),
      );
      expect(source, `${fn} must be defined`).toContain(`async function ${fn}(`);
    }
    const qcSource = readFileSync(
      path.join(repoRoot, "scripts/lib/query-consumer-family.ts"),
      "utf8",
    );
    for (const slug of Object.keys(QUERY_CONSUMER_FAMILY)) {
      const fn = `run_${slug.replaceAll("-", "_")}`;
      expect(qcSource, `row "${slug}" must bind ${fn}`).toMatch(
        new RegExp(`"${slug}": ${fn},`, "u"),
      );
    }
    for (const [file, family] of [
      ["scripts/lib/contract-parity-smoke-family.ts", CONTRACT_PARITY_SMOKE_FAMILY],
      ["scripts/lib/contract-parity-golden-family.ts", CONTRACT_PARITY_GOLDEN_FAMILY],
    ] as const) {
      const parity = readFileSync(path.join(repoRoot, file), "utf8");
      for (const slug of Object.keys(family)) {
        const fn = `run_${slug.replaceAll("-", "_")}`;
        expect(parity, `row "${slug}" must bind ${fn}`).toMatch(
          new RegExp(`"${slug}": ${fn},`, "u"),
        );
      }
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
