# Plugin Consumers Example

Minimal repo-local example that wires both lint consumers against one clean CSS
Modules workspace.

Files:

- `eslint.config.mjs`
- `stylelint.config.mjs`
- `src/App.jsx`
- `src/App.module.scss`

Run from the repo root:

```bash
pnpm check:plugin-consumer-example
```

That command verifies:

- ESLint can load `@omena/eslint-plugin`
- Stylelint can load `@omena/stylelint-plugin`
- the example workspace stays clean under both consumer surfaces
