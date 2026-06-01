export interface RustGateEvidenceEntry {
  readonly label: string;
  readonly argv: readonly string[];
  readonly variants?: readonly string[];
}

export interface RustGateEvidenceVariant {
  readonly label: string;
  readonly env?: Readonly<Record<string, string>>;
}

export const RUST_GATE_EVIDENCE_VARIANTS: readonly RustGateEvidenceVariant[] = [
  {
    label: "tsgo",
    env: {
      OMENA_TYPE_FACT_BACKEND: "tsgo",
      OMENA_TYPECHECK_VARIANT: "tsgo",
    },
  },
] as const;

export const RUST_GATE_EVIDENCE_CORPUS: readonly RustGateEvidenceEntry[] = [
  {
    label: "release-batch",
    argv: ["check:release-batch"],
    variants: ["tsgo"],
  },
  {
    label: "real-project-corpus",
    argv: ["check:real-project-corpus"],
    variants: ["tsgo"],
  },
  {
    label: "core-build",
    argv: ["omena-check", "run", "core/build"],
    variants: ["tsgo"],
  },
  {
    label: "lsp-server-smoke",
    argv: ["check:lsp-server-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "eslint-plugin-smoke",
    argv: ["check:eslint-plugin-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "stylelint-plugin-smoke",
    argv: ["check:stylelint-plugin-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "contract-parity-v2-smoke",
    argv: ["check:contract-parity-v2-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "rust-shadow-smoke",
    argv: ["check:rust-shadow-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "rust-type-fact-compare",
    argv: ["check:rust-type-fact-compare"],
    variants: ["tsgo"],
  },
  {
    label: "rust-query-plan-compare",
    argv: ["check:rust-query-plan-compare"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-compare",
    argv: ["check:rust-expression-domain-compare"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-fragments",
    argv: ["check:rust-expression-domain-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-candidates",
    argv: ["check:rust-expression-domain-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-canonical-candidate",
    argv: ["check:rust-expression-domain-canonical-candidate"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-evaluator-candidates",
    argv: ["check:rust-expression-domain-evaluator-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-reduced-evaluator",
    argv: ["check:rust-expression-domain-reduced-evaluator"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-flow-analysis",
    argv: ["check:rust-expression-domain-flow-analysis"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-domain-canonical-producer",
    argv: ["check:rust-expression-domain-canonical-producer"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-fragments",
    argv: ["check:rust-expression-semantics-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-candidates",
    argv: ["check:rust-expression-semantics-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-query-fragments",
    argv: ["check:rust-expression-semantics-query-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-match-fragments",
    argv: ["check:rust-expression-semantics-match-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-canonical-candidate",
    argv: ["check:rust-expression-semantics-canonical-candidate"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-evaluator-candidates",
    argv: ["check:rust-expression-semantics-evaluator-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-expression-semantics-canonical-producer",
    argv: ["check:rust-expression-semantics-canonical-producer"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-side-canonical-candidate",
    argv: ["check:rust-source-side-canonical-candidate"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-side-evaluator-candidates",
    argv: ["check:rust-source-side-evaluator-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-side-canonical-producer",
    argv: ["check:rust-source-side-canonical-producer"],
    variants: ["tsgo"],
  },
  {
    label: "rust-semantic-canonical-candidate",
    argv: ["check:rust-semantic-canonical-candidate"],
    variants: ["tsgo"],
  },
  {
    label: "rust-semantic-evaluator-candidates",
    argv: ["check:rust-semantic-evaluator-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-semantic-canonical-producer",
    argv: ["check:rust-semantic-canonical-producer"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-fragments",
    argv: ["check:rust-source-resolution-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-candidates",
    argv: ["check:rust-source-resolution-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-evaluator-candidates",
    argv: ["check:rust-source-resolution-evaluator-candidates"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-match-fragments",
    argv: ["check:rust-source-resolution-match-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-canonical-candidate",
    argv: ["check:rust-source-resolution-canonical-candidate"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-canonical-producer",
    argv: ["check:rust-source-resolution-canonical-producer"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-query-fragments",
    argv: ["check:rust-source-resolution-query-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-selector-usage-plan-compare",
    argv: ["check:rust-selector-usage-plan-compare"],
    variants: ["tsgo"],
  },
  {
    label: "rust-selector-usage-fragments",
    argv: ["check:rust-selector-usage-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-selector-usage-query-fragments",
    argv: ["check:rust-selector-usage-query-fragments"],
    variants: ["tsgo"],
  },
  {
    label: "rust-source-resolution-plan-compare",
    argv: ["check:rust-source-resolution-plan-compare"],
    variants: ["tsgo"],
  },
  {
    label: "rust-shadow-compare",
    argv: ["check:rust-shadow-compare"],
    variants: ["tsgo"],
  },
  {
    label: "backend-typecheck-smoke",
    argv: ["check:backend-typecheck-smoke"],
    variants: ["tsgo"],
  },
  {
    label: "type-fact-backend-parity",
    argv: ["check:type-fact-backend-parity"],
    variants: ["tsgo"],
  },
] as const;
