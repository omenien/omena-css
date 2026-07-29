---
title: PostCSS integration
description: Insert the published Omena plugin into a PostCSS pipeline with explicit file and source-map ownership.
kind: how-to
status: stable
products: [postcss, bundler]
owner: bundler
sourceOfTruth: authored
---

# PostCSS integration

```sh
npm install --save-dev @omena/postcss-plugin@0.2.1
```

```js
import omena from "@omena/postcss-plugin";

export default {
  plugins: [
    omena({
      minify: true,
      sourceMap: true,
    }),
  ],
};
```

The published `0.2.1` default scope is `.module.css` and `.module.scss`.
Repository source also recognizes `.module.less`, but that change is not a
published-package guarantee.

Omena replaces the PostCSS root with the engine output and attaches structured
build information to the result. Place it where one plugin clearly owns the
final CSS bytes. If an earlier plugin changes module syntax, verify source maps
and pass the complete workspace context needed by bundle operations.
