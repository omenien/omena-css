# Vitest CME

Internal test DSL seed for Omena CSS Modules.

This package is the incremental home for marker-driven test scenarios. Existing
`test/_fixtures` helpers remain valid; provider and protocol tests can migrate
one fixture at a time.

- `workspace()` fixture text parser
- marker syntax: `/*|*/`, `/*at:name*/`, `/*<range>*/.../*</range>*/`
- `documentFixture()` for provider-style document params from a workspace file
- `cursorFixture()` for provider-style cursor params from a marker
- `targetFixture()` for runtime query targets from a marker
- `textDocumentPositionFixture()`, `textDocumentRangeFixture()`, and
  `textDocumentRenameFixture()` for LSP-style params from markers
- `scenario()` wrapper actions: `hover`, `definition`, `prepareRename`,
  `codeAction`, `completion`, with action result types preserved
- `registerCmeMatchers()` with five domain-oriented matchers

The package is self-tested from `test/unit/vitest-cme`; migrated provider
fixtures use it directly from the relevant provider test.
