# omena-vite-plugin

Thin Vite consumer surface for `omena build`.

```js
import { omenaCss } from "omena-vite-plugin";

export default {
  plugins: [
    omenaCss({
      passes: ["comment-strip", "whitespace-strip"],
    }),
  ],
};
```

Default scope is intentionally conservative:

- only `.module.css` files are transformed by default
- SCSS/Less preprocessor replacement is not enabled here yet
- the plugin uses `CME_OMENA_CLI_BIN` when set, otherwise it falls back to
  `cargo run -p omena-cli`

Use `include` to opt into a wider path set after the downstream build pipeline
is ready for omena-owned preprocessing.
