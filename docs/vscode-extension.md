# VS Code Extension

The Omena CSS Modules extension connects VS Code to the Rust
`omena-lsp-server`. It owns editor integration and process lifecycle; CSS
semantics remain in the shared Rust engine.

## Install

Install from the
[VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=omena.omena-css),
or build a local VSIX:

```bash
pnpm install
pnpm package
code --install-extension omena-css-*.vsix
```

## Editor Features

- Definition, hover, completion, references, rename, diagnostics, and code actions.
- `classnames/bind`, `classnames`, `clsx`, and direct `styles.*` source patterns.
- CSS, SCSS, and Less module selectors, nested BEM names, `composes`, `@value`,
  `@keyframes`, custom properties, Sass symbols, and style contexts.
- Multi-root workspaces, watched-file refresh, path aliases, and source/style
  identity across TypeScript, JavaScript, Vue, Svelte, Astro, HTML, and Markdown
  projections supported by the server.

The extension activates for TypeScript, TSX, JavaScript, JSX, CSS, SCSS, Less,
Vue, Svelte, Astro, HTML, Markdown/MDX, Liquid, Twig, Nunjucks, Handlebars, ERB,
EJS, Django/Jinja templates, and EEx/HEEx. Embedded-language support is negotiated
by the Rust server and host projection path rather than by a second style parser
in the extension.

## Runtime Selection

`omena.lspServerRuntime` defaults to `auto`, which selects the bundled Rust
server and fails fast if no configured or packaged binary can be found.
Use `omena-lsp-server` to require that path explicitly. The public setting has
no Node runtime and no automatic semantic fallback.

`omena.typeFactBackend` defaults to `tsgo`; `tsgo-workspace` uses the workspace
binary. `typescript-current` remains a repository-internal comparison path, not
an editor setting.

## Configuration

All extension settings use the `omena.*` namespace. The generated references
are the current sources for the complete surface:

- [Configuration keys](reference/configuration.md)
- [LSP capabilities](reference/lsp-capabilities.md)
- [CLI commands](reference/cli.md)

Frequently adjusted settings include:

| Setting                         | Purpose                                         |
| ------------------------------- | ----------------------------------------------- |
| `omena.features.*`              | Enable editor feature families.                 |
| `omena.diagnostics.*`           | Control diagnostic severity and selected rules. |
| `omena.scss.classnameTransform` | Mirror css-loader `localsConvention`.           |
| `omena.pathAlias`               | Add resource-scoped import aliases.             |
| `omena.hover.maxCandidates`     | Bound dynamic candidates shown in hover.        |

For a selector such as `.btn-primary`, `asIs` exposes the original name;
`camelCase`/`dashes` expose original and camelized names; `camelCaseOnly` and
`dashesOnly` expose only the alias. Rename is rejected where reverse mapping is
lossy.

## CLI Persona Presets

The extension and CLI share workspace semantics, while CLI automation can start
from a built-in persona. The following table is generated from the persona
manifest rather than copied by hand.

<!-- BEGIN GENERATED: OMENA PERSONA PRESETS -->
<!-- Generated from product code. Do not edit by hand. -->
| Preset | Audience | Product verbs |
| --- | --- | --- |
| `workspace-maintenance` | `largeFrontendMonorepo` | `lint`, `fmt`, `modules`, `sass`, `intel`, `migrate`, `verify`, `ci`, `explain` |
| `design-system-governance` | `designSystemPlatform` | `lint`, `fmt`, `modules`, `sass`, `verify`, `ci`, `explain` |
| `build-integration` | `frameworkBundlerPluginAuthor` | `minify`, `bundle`, `modules`, `verify`, `ci`, `explain` |
| `migration-safety` | `migrationToolAuthor` | `lint`, `fmt`, `modules`, `sass`, `migrate`, `verify`, `explain` |
| `assurance-gates` | `highAssuranceCi` | `lint`, `modules`, `verify`, `ci`, `explain` |
| `semantic-research` | `cssSassCompilerResearch` | `lint`, `intel`, `verify`, `ci`, `explain` |
<!-- END GENERATED: OMENA PERSONA PRESETS -->

Use a preset from `omena.toml`; local values override it:

```toml
extends = "omena:workspace-maintenance"

[lint]
profile = "strict"
```

See the [generated persona reference](reference/personas.md) for the same table
as a standalone surface.

## Status And Logs

The `Omena CSS Modules` status item reports starting, indexing, settling, ready,
and not-running states. Select it to open the `Omena CSS Modules` output channel.
Indexing status includes admitted documents and pending files; ready status
includes indexed documents and external token sources.

If startup fails, first check the output channel, then verify the runtime setting
and any explicit `OMENA_LSP_SERVER_PATH` or `OMENA_LSP_SERVER_COMMAND` value.
The extension does not silently switch engines when an explicit Rust runtime is
missing.

## Other Editors

The language server is distributed independently of VS Code. See the
[Zed guide](clients/zed.md) and [Neovim guide](clients/neovim.md) for standalone
installation and client configuration.
