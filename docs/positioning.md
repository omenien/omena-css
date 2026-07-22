# Positioning

omena-css is a semantic CSS-family platform for parser-owned facts,
cross-language CSS Modules evidence, cascade-aware diagnostics, conservative
transform planning, and editor/CI feedback. It also ships build, bundle, minify,
Vite, PostCSS compatibility, and Sass analysis surfaces that carry the same
typed evidence into build pipelines.

It is not positioned as a build-time replacement for established CSS tools.
Its build surfaces focus on evidence-aware decisions, fail-closed planning, and
shared semantics across CLI, SDK, editor, and CI consumers.

## Role Comparison

Role source anchors:

- Lightning CSS: https://lightningcss.dev/
- PostCSS: https://postcss.org/
- Dart Sass: https://sass-lang.com/dart-sass/
- Biome: https://biomejs.dev/

| Tool          | Public role                                              | omena-css relationship                                                                                                        |
| ------------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Lightning CSS | Fast parser, transformer, bundler, and minifier          | Complementary build-pipeline tool. omena-css should compare against it only with same-corpus benchmark evidence.              |
| PostCSS       | JavaScript CSS transformation and plugin ecosystem       | Adjacent ecosystem. omena-css can feed semantic facts to consumers, but it is not a general PostCSS plugin replacement claim. |
| Dart Sass     | Primary Sass implementation and compiler reference path  | Compiler reference. omena-css analyzes Sass/SCSS facts but does not claim Sass compiler replacement.                          |
| Biome CSS     | Broad formatter/linter/assist toolchain with CSS support | Broad toolchain neighbor. omena-css focuses on CSS Modules semantics, provenance, and cascade evidence.                       |

## Evidence-Backed Claims

- Parser, cascade, transform, benchmark, and standalone workspace surfaces have
  versioned gates in the source monorepo.
- External speed comparisons require same-corpus, same-machine, same-request
  evidence before publication.
- Research-facing semantic substrates remain bounded by their product paths and
  executable gates; their presence alone does not establish a stronger claim.

## Current Non-Claims

- No direct speed ranking against Lightning CSS, PostCSS, Dart Sass, or Biome.
- No Sass compiler replacement claim.
- No PostCSS ecosystem replacement claim.
- No theorem-complete cascade, sheaf/cosheaf, modal, Datalog, egglog, or
  perceptual claim.
- No public Cargo 1.0 API freeze claim.
