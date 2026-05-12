# omena-cli

`omena-cli` is the command-line consumer surface for the Omena CSS workspace.

Current commands:

- `omena check <file>` parses a CSS-family file and reports parser-owned facts.
- `omena build <file>` runs the conservative transform pipeline and writes CSS
  output.
- `omena passes` lists the transform pass ids accepted by `omena build`.

Install the published CLI with Cargo:

```sh
cargo install omena-cli
omena check path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena passes
```

The CLI intentionally consumes the same public parser and transform crates that
library users consume. Checker-grade diagnostics can be layered in through the
query and checker crates as those crates become part of the standalone surface.
