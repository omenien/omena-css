import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  readonly scripts: Record<string, string>;
};
const readinessScript = requiredScript("check:rust-m4-axis-d-readiness");

const requiredBehaviorGates = [
  "rust/omena-resolver/fixture-suite",
  "rust/omena-bridge/boundary",
  "rust/omena-cascade/boundary",
  "rust/omena-query/transform-context",
  "rust/omena-query/transform-execute",
  "rust/omena-query/transform-differential",
  "rust/m4-axis-d-closure-audit",
] as const;

for (const target of requiredBehaviorGates) {
  assertIncludes(readinessScript, target, `M4 Axis D readiness must include ${target}`);
}

const resolverOrchestrator = read("rust/crates/omena-resolver/src/style_resolution.rs");
const resolverPathMappings = read(
  "rust/crates/omena-resolver/src/style_resolution/path_mappings.rs",
);
const resolverPackageResolution = read(
  "rust/crates/omena-resolver/src/style_resolution/package_resolution.rs",
);

for (const marker of ["mod package_resolution;", "mod path_mappings;"] as const) {
  assertIncludes(resolverOrchestrator, marker, `resolver orchestrator must retain ${marker}`);
}
for (const marker of [
  "tsconfig_style_module_base_candidates",
  "bundler_style_module_base_candidates",
] as const) {
  assertIncludes(resolverPathMappings, marker, `path mapping split must retain ${marker}`);
}
for (const marker of [
  "package_manifest_style_module_base_candidates",
  "package_import_style_module_base_candidates",
  "read_sass_node_package_export_entry",
] as const) {
  assertIncludes(
    resolverPackageResolution,
    marker,
    `package resolution split must retain ${marker}`,
  );
}

const queryTransform = read("rust/crates/omena-query/src/style/transform.rs");
const queryStaticStylesheet = read(
  "rust/crates/omena-query/src/style/transform/static_stylesheet.rs",
);
for (const moduleName of [
  "context",
  "css_modules",
  "design_tokens",
  "imports",
  "static_stylesheet",
] as const) {
  assertIncludes(queryTransform, `mod ${moduleName};`, `query transform must retain ${moduleName}`);
  assertFileExists(`rust/crates/omena-query/src/style/transform/${moduleName}.rs`);
}
assertIncludes(
  queryStaticStylesheet,
  "mod scss_variable_overrides;",
  "static stylesheet split must retain scss_variable_overrides",
);
assertFileExists(
  "rust/crates/omena-query/src/style/transform/static_stylesheet/scss_variable_overrides.rs",
);
const queryStaticScssVariableOverrides = read(
  "rust/crates/omena-query/src/style/transform/static_stylesheet/scss_variable_overrides.rs",
);
for (const marker of [
  "parse_static_scss_use_variable_override_list",
  "static_scss_matching_right_paren",
  "static_scss_top_level_colon_index",
  "canonical_static_scss_variable_name",
] as const) {
  assertIncludes(
    queryStaticScssVariableOverrides,
    marker,
    `scss variable override split must retain ${marker}`,
  );
}

const bridgeBundlerAlias = read("rust/crates/omena-bridge/src/bundler_config_alias.rs");
for (const moduleName of ["paths", "syntax"] as const) {
  assertIncludes(
    bridgeBundlerAlias,
    `mod ${moduleName};`,
    `bridge bundler alias must retain ${moduleName}`,
  );
  assertFileExists(`rust/crates/omena-bridge/src/bundler_config_alias/${moduleName}.rs`);
}

const cascadeOrchestrator = read("rust/crates/omena-cascade/src/lib.rs");
for (const moduleName of ["computed_value", "conformance", "fuzz", "ranking"] as const) {
  assertIncludes(
    cascadeOrchestrator,
    `mod ${moduleName};`,
    `cascade orchestrator must retain ${moduleName}`,
  );
  assertFileExists(`rust/crates/omena-cascade/src/${moduleName}.rs`);
}

const structuralSplits = [
  {
    family: "resolver",
    orchestrator: "rust/crates/omena-resolver/src/style_resolution.rs",
    modules: [
      "rust/crates/omena-resolver/src/style_resolution/path_mappings.rs",
      "rust/crates/omena-resolver/src/style_resolution/package_resolution.rs",
    ],
    behaviorGate: "rust/omena-resolver/fixture-suite",
  },
  {
    family: "query-transform",
    orchestrator: "rust/crates/omena-query/src/style/transform.rs",
    modules: [
      "rust/crates/omena-query/src/style/transform/context.rs",
      "rust/crates/omena-query/src/style/transform/css_modules.rs",
      "rust/crates/omena-query/src/style/transform/design_tokens.rs",
      "rust/crates/omena-query/src/style/transform/imports.rs",
      "rust/crates/omena-query/src/style/transform/static_stylesheet.rs",
      "rust/crates/omena-query/src/style/transform/static_stylesheet/scss_variable_overrides.rs",
    ],
    behaviorGate: "rust/omena-query/transform-differential",
  },
  {
    family: "bridge-bundler-alias",
    orchestrator: "rust/crates/omena-bridge/src/bundler_config_alias.rs",
    modules: [
      "rust/crates/omena-bridge/src/bundler_config_alias/paths.rs",
      "rust/crates/omena-bridge/src/bundler_config_alias/syntax.rs",
    ],
    behaviorGate: "rust/omena-bridge/boundary",
  },
  {
    family: "cascade-core",
    orchestrator: "rust/crates/omena-cascade/src/lib.rs",
    modules: [
      "rust/crates/omena-cascade/src/ranking.rs",
      "rust/crates/omena-cascade/src/computed_value.rs",
      "rust/crates/omena-cascade/src/conformance.rs",
      "rust/crates/omena-cascade/src/fuzz.rs",
    ],
    behaviorGate: "rust/omena-cascade/boundary",
  },
] as const;

process.stdout.write(
  JSON.stringify(
    {
      product: "rust.m4-axis-d-closure-audit",
      behaviorPreserving: true,
      readinessScript: "check:rust-m4-axis-d-readiness",
      behaviorGates: requiredBehaviorGates,
      structuralSplits: structuralSplits.map((split) => ({
        family: split.family,
        orchestrator: split.orchestrator,
        orchestratorLineCount: lineCount(read(split.orchestrator)),
        moduleCount: split.modules.length,
        modules: split.modules.map((modulePath) => ({
          path: modulePath,
          lineCount: lineCount(read(modulePath)),
        })),
        behaviorGate: split.behaviorGate,
      })),
      closedGates: [
        "resolverSplitCoveredByFixtureSuite",
        "queryTransformSplitCoveredByTransformGates",
        "queryStaticStylesheetScssOverrideSplitCoveredByTransformGates",
        "bridgeBundlerAliasSplitCoveredByBridgeBoundary",
        "cascadeCoreSplitCoveredByCascadeBoundary",
        "axisDReadinessComposesBehaviorGates",
      ],
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function requiredScript(name: string): string {
  const script = packageJson.scripts[name];
  assert.equal(typeof script, "string", `${name} must be declared in package.json`);
  return script;
}

function assertIncludes(source: string, marker: string, message: string): void {
  assert.ok(source.includes(marker), message);
}

function assertFileExists(relativePath: string): void {
  assert.ok(existsSync(path.join(root, relativePath)), `${relativePath} must exist`);
}

function lineCount(source: string): number {
  return source.split(/\r?\n/u).filter((line) => line.length > 0).length;
}
