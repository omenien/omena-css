# Paper Draft Outline

This is the initial public outline for the research track behind omena-css.
It is not a submitted paper; it records the external-facing argument and the
evidence that must exist before publication.

## Candidate 1: Cascade-Proven CSS Transforms

- Problem: many CSS transforms are syntactically simple but semantically unsafe
  without cascade, layer, scope, or selector evidence.
- Contribution: proof-carrying transform helpers that reject unsafe rewrites
  unless the caller provides closed-world evidence.
- Evaluation: compare accepted and rejected transform candidates across real
  CSS Modules, SCSS, and Less projects.

## Candidate 2: Incremental CSS-Family Analysis

- Problem: editor latency depends on reusing parser, cascade, and transform
  facts across small edits.
- Contribution: incremental fact boundaries for style analysis and conservative
  transform planning.
- Evaluation: measure cold and warm editor request latency across project-size
  buckets.

## Candidate 3: Parser-Owned Style Facts

- Problem: editor integrations often duplicate style parsing in ad hoc request
  handlers.
- Contribution: parser-owned canonical fact production for CSS Modules and
  CSS-family dialects.
- Evaluation: compare diagnostics, hover, definition, references, and transform
  results before and after request handlers consume parser-owned facts.
