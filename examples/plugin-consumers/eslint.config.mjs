import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("eslint-plugin-css-module-explainer");

export default [
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
  },
  ...omena.configs.recommended,
];
