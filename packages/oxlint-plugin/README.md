# @omena/oxlint-plugin

Thin Oxlint JS-plugin adapter for omena source diagnostics.

Oxlint currently loads custom JavaScript plugins through `jsPlugins`. This
adapter exposes focused source diagnostics by calling `omena source-diagnostics`
directly, so the source of truth remains `omena-query`.

```jsonc
{
  "jsPlugins": [
    {
      "name": "omena",
      "specifier": "@omena/oxlint-plugin",
    },
  ],
  "rules": {
    "omena/missing-static-class": "error",
    "omena/missing-template-prefix": "error",
  },
}
```

Set `OMENA_CLI_BIN=/path/to/omena`, or the rule-level `omenaBin` option, to
use a prebuilt CLI binary instead of the local Cargo fallback.
