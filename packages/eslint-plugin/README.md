# @omena/eslint-plugin

First-cut ESLint consumer for Omena CSS Modules.

Current rules:

- `omena/missing-module`
- `omena/missing-static-class`
- `omena/missing-template-prefix`
- `omena/missing-resolved-class-values`
- `omena/missing-resolved-class-domain`
- `omena/invalid-class-reference`
- `omena/no-unknown-dynamic-class`
- `omena/no-impossible-selector`
- `omena/no-imprecise-value`
- `omena/source-check`

Config variants:

- `configs.recommended`
  - aggregate source-side diagnostics through `source-check`
- `configs.focused`
  - explicit focused rules for missing module/static/template/dynamic findings
- `configs.dynamicMoat`
  - optional moat rule for unresolved dynamic class expressions
- `configs.mTier`
  - precise M-tier split rules for impossible finite selector values and imprecise dynamic value domains

Recommended flat config:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.recommended];
```

Focused variant:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.focused];
```

Supported options:

- `workspaceRoot`
- `classnameTransform`
- `pathAlias`
- `includeMissingModule`

Optional direct Omena backend:

- Set `OMENA_ESLINT_QUERY_BACKEND=omena-cli` to route focused source-side rules
  through `omena source-diagnostics`.
- Set `OMENA_CLI_BIN=/path/to/omena` to use a prebuilt CLI binary instead
  of `cargo run`.
- Without those variables, the plugin keeps using the existing TypeScript
  checker host path.

Current limitations:

- source-side rules only
- style-side rules are not exposed yet
- this package is still a local workspace package, not a published artifact

Optional dynamic moat:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.dynamicMoat];
```

This rule targets dynamic class expressions whose resolved values or domains do
not map to any known selector in the referenced CSS Module.

Manual focused dynamic variants:

```js
"omena/missing-resolved-class-values": "error"
"omena/missing-resolved-class-domain": "error"
```

Precise M-tier split:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.mTier];
```

Equivalent manual rules:

```js
"omena/no-impossible-selector": "error"
"omena/no-imprecise-value": "error"
```
