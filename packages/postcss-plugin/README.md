# @omena/postcss-plugin

PostCSS integration for running Omena CSS transforms in production builds.

It targets CSS Modules and Sass/CSS module files while preserving Omena's
source-map provenance. Use this package when your build already runs PostCSS
and you want Omena's CSS-Modules/Sass transform pipeline without adopting the
Vite plugin.

## Install

```sh
npm install -D @omena/postcss-plugin@0.2.1 @omena/napi@0.2.1 postcss@^8.5
```

`@omena/napi` is the preferred runtime. The plugin can fall back to
`@omena/wasm` when native bindings are unavailable and `wasmFallback` is not
disabled.

## PostCSS Config

```js
const { omenaPostcss } = require("@omena/postcss-plugin");

module.exports = {
  plugins: [
    omenaPostcss({
      passes: ["scss-module-evaluate"],
      minify: true,
      sourceMap: true,
    }),
  ],
};
```

ESM default export is also available:

```js
import omenaPostcss from "@omena/postcss-plugin";

export default {
  plugins: [
    omenaPostcss({
      bundle: true,
      sources: ["src/styles/tokens.module.css"],
    }),
  ],
};
```

## Scope

- The published `0.2.1` package transforms `.module.css` and `.module.scss`
  files by default.
- `minify`, `treeShake`, and `bundle` compose built-in pass presets.
- `sources` and `packageManifests` provide additional workspace context for
  bundle-oriented transforms.
- `omena.toml` can provide canonical `[build]` defaults;
  `omena.config.{ts,js,mjs,cjs,json,toml}` remains compatible. Explicit plugin
  options override config values.
- The plugin records an `omena-css` PostCSS message containing the transform
  summary and source-map provenance details.

Use `include` to opt into a wider path set after the downstream build pipeline
is ready for Omena-owned preprocessing.

Repository source also recognizes `.module.less`, but that behavior is not part
of the published `0.2.1` contract.
