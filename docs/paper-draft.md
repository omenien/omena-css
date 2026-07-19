# Paper Draft Outline

This is the initial public outline for the research track behind omena-css.
It is not a submitted paper; it records the external-facing argument and the
evidence that must exist before publication.

## Candidate 1: Evidence-Gated CSS Transforms

- Problem: many CSS transforms are syntactically simple but semantically unsafe
  without cascade, layer, scope, or selector evidence.
- Contribution: proof-carrying transform helpers that reject unsafe rewrites
  unless the caller provides closed-world evidence.
- Evaluation: compare accepted and rejected transform candidates across real
  CSS Modules, SCSS, and Less projects.
- Current evidence: cascade/value-family, dimensional/refinement, and transform
  planning gates are staged evidence. They are not a sheaf/cosheaf theorem,
  Liquid-Haskell-style inference, or a global correctness proof.

## Candidate 2: Incremental CSS-Family Analysis

- Problem: editor latency depends on reusing parser, cascade, and transform
  facts across small edits.
- Contribution: incremental fact boundaries for style analysis and conservative
  transform planning.
- Evaluation: measure cold and warm editor request latency across project-size
  buckets.
- Current evidence: the incremental layer has real invalidation and reuse
  summaries. DBSP, Z-set, and external Datalog-host claims are later work.

## Candidate 3: Parser-Owned Style Facts

- Problem: editor integrations often duplicate style parsing in ad hoc request
  handlers.
- Contribution: parser-owned canonical fact production for CSS Modules and
  CSS-family dialects.
- Evaluation: compare diagnostics, hover, definition, references, and transform
  results before and after request handlers consume parser-owned facts.

## Current Evidence Boundary

The current research track is evidence-backed only at the substrate level:

- The Vue SFC source-language bridge covers script-side `useCssModule()` and
  embedded `<style module>` behavior.
- Cascade-family work is framing-neutral substrate, not a sheaf or cosheaf
  theorem.
- Dimensional/refinement work bridges cascade-family values into refinement
  predicate witnesses. It does not fork a unit system, complete SMT refinement,
  or claim Liquid-Haskell-style inference.
- Edit-distance and cascade-margin work is fixture-witness substrate, not a
  calibrated Lipschitz theorem.
- Contextual equality saturation is scaffold-only over the optional `egg`
  boundary. It does not claim an egglog binding or full three-view fusion.
- `perceptual-check` is a downstream CLI/schema surface over omena facts with a
  fixture-witnessed WCAG contrast bound for exact sRGB color/background pairs.
  It does not implement APCA, OKLab, a full perceptual algorithm, or a
  public-safety claim.

## Publication Requirement

Before submission or public benchmarking, every claim must cite one of:

- a source-controlled gate command,
- a release artifact,
- a fixture matrix,
- a benchmark corpus and machine record,
- an issue disposition,
- or a workspace publish dry-run (cargo publish --workspace --locked --dry-run).
