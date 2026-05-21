import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
  readonly scripts: Record<string, string>;
};

const readinessScript = requiredScript("check:rust-m4-axis-b-readiness");
const fixtureSuiteScript = readFileSync(
  "scripts/check-rust-omena-resolver-fixture-suite.ts",
  "utf8",
);
const resolverCacheScript = readFileSync(
  "scripts/check-rust-omena-lsp-server-resolver-cache-runtime.ts",
  "utf8",
);
const typeFactProtocolScript = readFileSync(
  "scripts/check-rust-omena-lsp-server-type-fact-protocol.ts",
  "utf8",
);

const requiredReadinessTargets = [
  "rust/omena-resolver/fixture-suite",
  "rust/omena-lsp-server/resolver-cache-runtime",
  "rust/omena-lsp-server/type-fact-protocol",
  "rust/m4-axis-b-closure-audit",
] as const;
for (const target of requiredReadinessTargets) {
  assertIncludes(readinessScript, target, `M4 Axis B readiness must include ${target}`);
}

const resolverFixtureRequirements = [
  "resolver package exports/imports and Sass pkg URLs",
  "resolver TypeScript path mappings",
  "resolver Vite/Webpack-style bundler aliases",
  "bridge product style-resolution inputs",
  "LSP product TypeScript path mappings",
  "LSP product Vite/Webpack bundler aliases",
  "LSP product package manifests and imports",
  "package=exports-imports-conditions-patterns",
  "sass=node-package-importer-pkg-url-ordering",
  "bundler=vite-webpack-aliases",
] as const;
for (const marker of resolverFixtureRequirements) {
  assertIncludes(fixtureSuiteScript, marker, `resolver fixture suite must cover ${marker}`);
}

const resolverCacheRequirements = [
  "cachedWorkspaceResolutionInputCount",
  "package.json(root+package),tsconfig.json,vite.config.ts,webpack.config.js",
  "root package imports initial definition",
  "root package imports refreshed definition",
  "tsconfig refreshed definition",
  "webpack refreshed definition",
  "package refreshed definition",
  "refreshBaseline",
  "hotDefinition",
] as const;
for (const marker of resolverCacheRequirements) {
  assertIncludes(resolverCacheScript, marker, `resolver cache runtime must cover ${marker}`);
}

const issue38RegressionRequirements = [
  "diskFallback=unopened-style",
  "unicodePosition=utf16",
  "union=nullish-soft-skip",
  "projection=omena-query",
  "late style didOpen did not retrigger source projection",
  "textDocument/rename",
  "textDocument/references",
  "font-size-\\${fontSize}",
] as const;
for (const marker of issue38RegressionRequirements) {
  assertIncludes(typeFactProtocolScript, marker, `#38 protocol gate must cover ${marker}`);
}

process.stdout.write(
  JSON.stringify(
    {
      product: "rust.m4-axis-b-closure-audit",
      resolverPerimeter: {
        fixtureSuite: "rust/omena-resolver/fixture-suite",
        covers: [
          "node-package-resolution",
          "typescript-paths-and-extends",
          "package-exports-imports-conditions-patterns",
          "sass-node-package-importer-pkg-url-ordering",
          "vite-webpack-aliases",
          "lsp-product-paths",
        ],
      },
      cacheAndInvalidation: {
        gate: "rust/omena-lsp-server/resolver-cache-runtime",
        covers: [
          "package.json",
          "root-package-imports",
          "tsconfig.json",
          "vite.config.ts",
          "webpack.config.js",
        ],
        compares: ["refreshBaseline", "hotDefinition"],
      },
      issue38: {
        gate: "rust/omena-lsp-server/type-fact-protocol",
        covers: [
          "unopened-style-disk-fallback",
          "utf16-positioning",
          "nullish-union-soft-skip",
          "tsx-first-style-later-refresh",
          "hover-definition-references-rename",
        ],
        externalAcceptanceStillRequired: true,
      },
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function requiredScript(name: string): string {
  const script = packageJson.scripts[name];
  assert.equal(typeof script, "string", `${name} must be declared in package.json`);
  return script;
}

function assertIncludes(source: string, marker: string, message: string): void {
  assert.ok(source.includes(marker), message);
}
