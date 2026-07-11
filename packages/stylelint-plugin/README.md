# @omena/stylelint-plugin

Stylelint compatibility surface for Omena CSS Modules. It keeps convention-oriented Stylelint
workflows available while Omena provides semantic and source-aware diagnostics from its shared
workspace graph; it is not a claim that every Stylelint rule has an Omena equivalent.

Omena supports two compatibility directions:

- `omena lint --stylelint-config <path>` reads JSON or YAML `.stylelintrc` rule settings. The eight
  rules below map to native Omena checker rules, and every unsupported rule is listed in the lint
  compatibility report instead of being silently dropped.
- This package lets an existing Stylelint process consume the same eight Omena diagnostic families.

Current rules:

- `omena/unused-selector`
- `omena/missing-composed-module`
- `omena/missing-composed-selector`
- `omena/missing-value-module`
- `omena/missing-imported-value`
- `omena/missing-keyframes`
- `omena/missing-custom-property`
- `omena/missing-sass-symbol`

Recommended config:

```js
export default {
  extends: ["@omena/stylelint-plugin/recommended"],
};
```

Direct rule config:

```js
export default {
  plugins: ["@omena/stylelint-plugin"],
  rules: {
    "omena/unused-selector": [true],
    "omena/missing-composed-module": [true],
    "omena/missing-composed-selector": [true],
    "omena/missing-value-module": [true],
    "omena/missing-imported-value": [true],
    "omena/missing-keyframes": [true],
    "omena/missing-custom-property": [true],
    "omena/missing-sass-symbol": [true],
  },
};
```

Current boundaries:

- first cut is focused on `.module.css` / `.module.scss` / `.module.less`
- rules outside the explicit eight-rule compatibility table remain owned by Stylelint and are
  reported as unsupported by `omena lint`
- `omena/unused-selector`,
  `omena/missing-composed-module`,
  `omena/missing-composed-selector`,
  `omena/missing-value-module`,
  `omena/missing-imported-value`,
  `omena/missing-custom-property`, and
  `omena/missing-keyframes` read the `omena-query` style diagnostics surface
  through `omena-cli`
- `omena/missing-sass-symbol` also supports that direct
  `omena-cli` path for same-file unresolved Sass symbols
- In external projects, set `OMENA_CLI_BIN=/path/to/omena` to a built CLI binary.
