# @omena/vite-plugin

Vite consumer surface for the in-process Omena CSS native/wasm build API.

```js
import { omenaCss } from "@omena/vite-plugin";

export default {
  plugins: [
    omenaCss({
      passes: ["comment-strip", "whitespace-strip"],
      minify: true,
      treeShake: true,
      bundle: true,
      sources: ["src/styles/tokens.module.css"],
    }),
  ],
};
```

Default scope is intentionally conservative:

- `.module.css` and `.module.scss` files are transformed by default
- the hot path calls `@omena/napi` directly and falls back to `@omena/wasm`
  when native bindings are unavailable
- CLI and `cargo run` fallback are intentionally not used in Vite transforms
- Vite dev serves an Omena-owned CSS Modules runtime with `import.meta.hot`
  acceptance so style edits update without a full page reload
- `treeShake`, `bundle`, and `minify` compose built-in pass presets; provide
  `sources`/`packageManifests` when bundle context needs additional workspace
  files
- `omena.config.{ts,js,mjs,cjs,json,toml}` can provide defaults; explicit
  plugin options override config values

Use `include` to opt into a wider path set after the downstream build pipeline
is ready for omena-owned preprocessing.
