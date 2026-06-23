# @omena/stylelint-plugin

First-cut Stylelint consumer for Omena CSS Modules.

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

Current limitations:

- first cut is focused on `.module.css` / `.module.scss` / `.module.less`
- current package is still a local repo package, not a published artifact
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
