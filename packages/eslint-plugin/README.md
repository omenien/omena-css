# eslint-plugin-css-module-explainer

First-cut ESLint consumer for CSS Module Explainer.

Current rules:

- `css-module-explainer/missing-module`
- `css-module-explainer/missing-static-class`
- `css-module-explainer/missing-template-prefix`
- `css-module-explainer/missing-resolved-class-values`
- `css-module-explainer/missing-resolved-class-domain`
- `css-module-explainer/invalid-class-reference`
- `css-module-explainer/no-unknown-dynamic-class`
- `css-module-explainer/no-impossible-selector`
- `css-module-explainer/no-imprecise-value`
- `css-module-explainer/source-check`

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
const cssModuleExplainer = require("eslint-plugin-css-module-explainer");

export default [...cssModuleExplainer.configs.recommended];
```

Focused variant:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const cssModuleExplainer = require("eslint-plugin-css-module-explainer");

export default [...cssModuleExplainer.configs.focused];
```

Supported options:

- `workspaceRoot`
- `classnameTransform`
- `pathAlias`
- `includeMissingModule`

Current limitations:

- source-side rules only
- style-side rules are not exposed yet
- this package is still a local workspace package, not a published artifact

Optional dynamic moat:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const cssModuleExplainer = require("eslint-plugin-css-module-explainer");

export default [...cssModuleExplainer.configs.dynamicMoat];
```

This rule targets dynamic class expressions whose resolved values or domains do
not map to any known selector in the referenced CSS Module.

Manual focused dynamic variants:

```js
"css-module-explainer/missing-resolved-class-values": "error"
"css-module-explainer/missing-resolved-class-domain": "error"
```

Precise M-tier split:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const cssModuleExplainer = require("eslint-plugin-css-module-explainer");

export default [...cssModuleExplainer.configs.mTier];
```

Equivalent manual rules:

```js
"css-module-explainer/no-impossible-selector": "error"
"css-module-explainer/no-imprecise-value": "error"
```
