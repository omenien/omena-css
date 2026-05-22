import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
  readonly scripts: Record<string, string>;
};

const readinessScript = requiredScript("check:rust-m4-axis-b-readiness");
const packageScript = requiredScript("package");
const packagedTypeFactProtocolScript = requiredScript(
  "check:packaged-omena-lsp-server-type-fact-protocol",
);
const fixtureSuiteScript = readFileSync(
  "scripts/check-rust-omena-resolver-fixture-suite.ts",
  "utf8",
);
const bundlerAliasSource = readFileSync(
  "rust/crates/omena-bridge/src/bundler_config_alias.rs",
  "utf8",
);
const bundlerAliasTests = readFileSync(
  "rust/crates/omena-bridge/src/bundler_config_alias/tests.rs",
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
const packagedTypeFactProtocolFile = readFileSync(
  "scripts/check-packaged-omena-lsp-server-type-fact-protocol.ts",
  "utf8",
);
const resolverPackageTests = readFileSync(
  "rust/crates/omena-resolver/src/tests/package.rs",
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
assertIncludes(
  packageScript,
  "release/check/packaged-omena-lsp-server-type-fact-protocol",
  "release package must run the packaged #38 type-fact protocol gate",
);
assertIncludes(
  packagedTypeFactProtocolScript,
  "check-packaged-omena-lsp-server-type-fact-protocol.ts",
  "packaged #38 gate must point at its implementation file",
);
assertIncludes(
  packagedTypeFactProtocolFile,
  "check-rust-omena-lsp-server-type-fact-protocol.ts",
  "packaged #38 gate must reuse the normal type-fact protocol fixture",
);

const resolverFixtureRequirements = [
  "resolver package exports/imports and Sass pkg URLs",
  "resolver TypeScript path mappings",
  "resolver Vite/Webpack-style bundler aliases",
  "bridge product style-resolution inputs",
  "LSP product TypeScript path mappings",
  "LSP product Vite/Webpack bundler aliases",
  "LSP product package manifests and imports",
  "package=exports-imports-conditions-patterns",
  "package=null-blocking-private-subpaths",
  "sass=node-package-importer-pkg-url-ordering",
  "bundler=vite-webpack-aliases",
] as const;
for (const marker of resolverFixtureRequirements) {
  assertIncludes(fixtureSuiteScript, marker, `resolver fixture suite must cover ${marker}`);
}

const resolverPackageNullBlockingRequirements = [
  "package_export_null_subpath_blocks_pattern_and_file_fallback",
  "package_import_null_exact_entry_blocks_pattern_fallback",
  '"./private/*":null',
  '"#theme/private":null',
] as const;
for (const marker of resolverPackageNullBlockingRequirements) {
  assertIncludes(
    resolverPackageTests,
    marker,
    `resolver package fixture suite must cover null-blocking package maps: ${marker}`,
  );
}

const bundlerAliasExtractionRequirements = [
  "use oxc_parser::{Parser, ParserReturn}",
  "Parser::new(&allocator, config_source, source_type).parse()",
  "collect_bundler_aliases_from_program",
] as const;
for (const marker of bundlerAliasExtractionRequirements) {
  assertIncludes(
    bundlerAliasSource,
    marker,
    `bundler alias extraction must use the OXC config AST path: ${marker}`,
  );
}

const bundlerAliasFixtureRequirements = [
  "extracts_vite_object_aliases_from_define_config",
  "extracts_webpack_array_aliases_from_module_exports",
  "preserves_webpack_array_alias_declaration_order",
  "marks_dynamic_alias_entries_unrecognized",
  "marks_dynamic_exported_config_unrecognized_without_top_level_fallback",
  "marks_dynamic_module_exports_config_unrecognized",
  "regex-alias-find",
  "dynamic-config-export",
] as const;
for (const marker of bundlerAliasFixtureRequirements) {
  assertIncludes(
    bundlerAliasTests,
    marker,
    `bundler alias fixture suite must cover literal extraction and dynamic markers: ${marker}`,
  );
}

const resolverCacheRequirements = [
  "cachedWorkspaceResolutionInputCount",
  "package.json(root+package),tsconfig.json,tsconfig.base.json,vite.config.ts,webpack.config.js",
  "root package imports initial definition",
  "root package imports refreshed definition",
  "tsconfig refreshed definition",
  "tsconfig.base refreshed definition",
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
for (const marker of ["CME_OMENA_LSP_SERVER_PATH", "CME_OMENA_LSP_SERVER_CWD"] as const) {
  assertIncludes(typeFactProtocolScript, marker, `#38 protocol gate must support ${marker}`);
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
          "package-null-blocking-private-subpaths",
          "sass-node-package-importer-pkg-url-ordering",
          "vite-webpack-aliases",
          "lsp-product-paths",
        ],
      },
      bundlerConfigAliasExtraction: {
        owner: "omena-bridge",
        parser: "oxc_parser-js-ts-config-only",
        covers: [
          "vite-define-config-object-aliases",
          "webpack-module-exports-array-aliases",
          "webpack-array-declaration-order",
          "dynamic-alias-entry-markers",
          "dynamic-config-export-markers",
        ],
      },
      cacheAndInvalidation: {
        gate: "rust/omena-lsp-server/resolver-cache-runtime",
        covers: [
          "package.json",
          "root-package-imports",
          "tsconfig.json",
          "tsconfig.base.json",
          "vite.config.ts",
          "webpack.config.js",
        ],
        compares: ["refreshBaseline", "hotDefinition"],
      },
      issue38: {
        gate: "rust/omena-lsp-server/type-fact-protocol",
        packagedGate: "release/check/packaged-omena-lsp-server-type-fact-protocol",
        covers: [
          "unopened-style-disk-fallback",
          "utf16-positioning",
          "nullish-union-soft-skip",
          "tsx-first-style-later-refresh",
          "hover-definition-references-rename",
          "packaged-vsix-lsp-protocol",
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
