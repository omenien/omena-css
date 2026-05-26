import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const TARGETS = [
  {
    path: "rust/crates/omena-categorical/src/lib.rs",
    required: ["claim_level:", "product-wired additive evidence", "not a completed categorical"],
  },
  {
    path: "rust/crates/omena-smt/src/lib.rs",
    required: ["claim_level:", "opt-in solver-backed checking", "not default build SMT"],
  },
  {
    path: "rust/crates/omena-variational/src/lib.rs",
    required: ["claim_level:", "product-wired posterior inference", "not a corpus-calibrated"],
  },
  {
    path: "rust/crates/omena-zk-audit/src/lib.rs",
    required: ["claim_level:", "opt-in arkworks proof round-trip", "default build stays"],
  },
  {
    path: "rust/crates/omena-zk-circuit/src/lib.rs",
    required: ["claim_level:", "constraint-generation substrate", "not a standalone proving"],
  },
  {
    path: "rust/crates/omena-rg-flow/src/lib.rs",
    required: ["claim_level:", "Jacobian-spectrum approximation", "not a full"],
  },
  {
    path: "rust/crates/omena-lawvere/src/lib.rs",
    required: ["claim_level:", "differential commutativity witness", "not a global"],
  },
  {
    path: "rust/crates/omena-streaming-ifds/src/lib.rs",
    required: ["claim_level:", "exact default live-analysis mechanism", "not an asymptotic"],
  },
  {
    path: "rust/crates/omena-ensemble/src/lib.rs",
    required: ["claim_level:", "opt-in replica-overlap substrate", "not a default product"],
  },
  {
    path: "rust/crates/omena-refinement/src/lib.rs",
    required: ["claim_level:", "cascade refinement bridge substrate", "not Liquid-Haskell"],
  },
] as const;

for (const target of TARGETS) {
  const source = readFileSync(target.path, "utf8");
  const moduleDocs = source
    .split("\n")
    .filter((line) => line.startsWith("//!"))
    .map((line) => line.replace(/^\/\/!\s?/u, "").trim())
    .join(" ");
  for (const required of target.required) {
    assert.ok(
      moduleDocs.includes(required),
      `${target.path} must include claim-level rustdoc token: ${required}`,
    );
  }
}

process.stdout.write(
  `validated theory claim_level rustdoc: crateCount=${TARGETS.length} marker=claim_level\n`,
);
