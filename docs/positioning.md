# Positioning

omena-css is a semantic CSS-family platform for parser-owned facts,
cross-language CSS Modules evidence, cascade-aware diagnostics, conservative
transform planning, and editor/CI feedback. It also ships build, bundle, minify,
Vite, PostCSS compatibility, and Sass analysis surfaces that carry the same
typed evidence into build pipelines.

It is not positioned as a build-time replacement for established CSS tools.
Its build surfaces focus on evidence-aware decisions, fail-closed planning, and
shared semantics across CLI, SDK, editor, and CI consumers.

## How omena-css Compares

**Is omena-css a replacement for [Lightning CSS](https://lightningcss.dev/)?**
No. Lightning CSS is a fast parser, transformer, bundler, and minifier.
omena-css is a complementary build-pipeline tool that adds typed semantic
evidence: keep your transformer, and let omena-css gate what a transform may
safely change.

**Does omena-css replace [PostCSS](https://postcss.org/)?**
No. PostCSS is a JavaScript transformation and plugin ecosystem. omena-css is
an adjacent ecosystem that can feed semantic facts to PostCSS consumers, but it
is not a general PostCSS plugin host.

**Does omena-css compile Sass?**
No. [Dart Sass](https://sass-lang.com/dart-sass/) remains the compiler
reference. omena-css analyzes Sass/SCSS facts — module graphs, compatibility,
and provenance — and does not compile Sass.

**How does omena-css relate to [Biome](https://biomejs.dev/)?**
Biome CSS is a broad formatter/linter/assist toolchain and a neighbor rather
than a competitor. omena-css focuses on CSS Modules semantics, provenance, and
cascade evidence.

**Is omena-css faster than the tools above?**
omena-css publishes speed comparisons only with same-corpus, same-machine,
same-request evidence. External speed comparisons require same-corpus benchmark
runs before publication; the standard and current baselines live in
[performance evidence](performance.md).

## Scope And Non-Goals

Intentional non-goals:

- Compiling Sass or replacing the Dart Sass toolchain.
- Hosting or re-implementing the PostCSS plugin ecosystem.
- Competing as a drop-in build-time replacement for established CSS tools.
- Publishing speed rankings that do not meet the evidence standard above.

Current limitations, stated as facts rather than promises:

- The public Cargo API has no 1.0 freeze; crates follow the 0.x line and may
  re-key between minor trains.
- Research-facing semantic substrates remain bounded by their product paths and
  executable gates; their presence alone does not establish a stronger claim.
