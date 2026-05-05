# stylelint-plugin-css-module-explainer

First-cut Stylelint consumer for CSS Module Explainer.

Current rules:

- `css-module-explainer/unused-selector`
- `css-module-explainer/missing-composed-module`
- `css-module-explainer/missing-composed-selector`
- `css-module-explainer/missing-value-module`
- `css-module-explainer/missing-imported-value`
- `css-module-explainer/missing-keyframes`
- `css-module-explainer/missing-custom-property`
- `css-module-explainer/missing-sass-symbol`

Recommended config:

```js
export default {
  extends: ["stylelint-plugin-css-module-explainer/recommended"],
};
```

Direct rule config:

```js
export default {
  plugins: ["stylelint-plugin-css-module-explainer"],
  rules: {
    "css-module-explainer/unused-selector": [true],
    "css-module-explainer/missing-composed-module": [true],
    "css-module-explainer/missing-composed-selector": [true],
    "css-module-explainer/missing-value-module": [true],
    "css-module-explainer/missing-imported-value": [true],
    "css-module-explainer/missing-keyframes": [true],
    "css-module-explainer/missing-custom-property": [true],
    "css-module-explainer/missing-sass-symbol": [true],
  },
};
```

Current limitations:

- first cut is focused on `.module.css` / `.module.scss` / `.module.less`
- current package is still a local repo package, not a published artifact
