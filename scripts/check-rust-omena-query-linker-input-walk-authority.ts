import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const injectSecondSourceWalk = process.argv.includes("--inject-second-source-walk");
const transformPath = "rust/crates/omena-query/src/style/transform.rs";
const testPath = "rust/crates/omena-query/src/tests/transform_facade.rs";
const fixturePaths = [
  "rust/crates/omena-query/tests/fixtures/bundle-code-split-dependencies/sass/entry.scss",
  "rust/crates/omena-query/tests/fixtures/bundle-code-split-dependencies/sass/admin.scss",
  "rust/crates/omena-query/tests/fixtures/bundle-code-split-dependencies/sass/_tokens.scss",
  "rust/crates/omena-query/tests/fixtures/bundle-code-split-dependencies/css/entry.css",
  "rust/crates/omena-query/tests/fixtures/bundle-code-split-dependencies/css/shared.css",
  "rust/crates/omena-query/tests/fixtures/bundle-package-exports/src/entry.css",
  "rust/crates/omena-query/tests/fixtures/bundle-package-exports/src/missing.css",
  "rust/crates/omena-query/tests/fixtures/bundle-package-exports/node_modules/@acme/theme/package.json",
  "rust/crates/omena-query/tests/fixtures/bundle-package-exports/node_modules/@acme/theme/dist/tokens.css",
] as const;

const originalTransformSource = read(transformPath);
const sourceWalkAnchor = "let modules = style_sources_to_transform_bundle_modules(style_sources);";
const transformSource = injectSecondSourceWalk
  ? replaceExactlyOnce(
      originalTransformSource,
      sourceWalkAnchor,
      `${sourceWalkAnchor}\n    let _duplicate_modules = style_sources_to_transform_bundle_modules(style_sources);`,
    )
  : originalTransformSource;
const testSource = read(testPath);

const dependencyProducer = functionSource(
  transformSource,
  "collect_omena_query_bundle_code_split_dependency_specifiers",
);
assert.equal(
  occurrences(dependencyProducer, "style_sources_to_transform_bundle_modules(style_sources)"),
  1,
  "code-split dependency facts must perform exactly one shared parsed-module walk",
);
assert.equal(
  occurrences(
    dependencyProducer,
    "project_omena_transform_bundle_linker_inputs_from_parsed_modules",
  ),
  1,
  "code-split dependency facts must project the shared parsed modules exactly once",
);
assert.equal(
  occurrences(dependencyProducer, "bundle_edge_is_module_dependency"),
  1,
  "code-split dependency facts must consume the shared module-edge authority",
);
assert.equal(
  occurrences(dependencyProducer, "collect_style_facts"),
  0,
  "code-split dependency collection must not introduce a second parser walk",
);
assert.equal(
  occurrences(dependencyProducer, "summarize_omena_transform_bundle_from_source"),
  0,
  "code-split dependency collection must not reopen the full source summary",
);

const reachabilityConsumer = functionSource(
  transformSource,
  "collect_omena_query_bundle_code_split_entry_reachability",
);
assert.ok(
  reachabilityConsumer.includes("dependency_specifiers_by_path"),
  "code-split reachability must consume the precomputed dependency producer",
);
for (const forbidden of [
  "collect_style_facts",
  "summarize_omena_transform_bundle_from_source",
  "style_sources_to_transform_bundle_modules",
]) {
  assert.equal(
    occurrences(reachabilityConsumer, forbidden),
    0,
    `code-split reachability must not start a source walk through ${forbidden}`,
  );
}

const parsedModuleProducer = functionSource(
  transformSource,
  "style_sources_to_transform_bundle_modules",
);
assert.equal(
  occurrences(parsedModuleProducer, "parse_omena_query_omena_parser_style_source"),
  1,
  "the shared parsed-module producer must parse once through the query parser facade",
);
assert.equal(
  occurrences(parsedModuleProducer, "omena_parser::facts_from_cst"),
  1,
  "the shared parsed-module producer must derive facts from the single parse result",
);
assert.equal(
  occurrences(parsedModuleProducer, "omena_parser::collect_style_facts"),
  0,
  "the shared parsed-module producer must not bypass the query parser facade",
);
assert.ok(
  parsedModuleProducer.includes("style_sources") &&
    parsedModuleProducer.includes(".iter()") &&
    parsedModuleProducer.includes(".collect()"),
  "the parsed-module producer must remain a single projection over input source paths",
);

const linkerProjection = functionSource(
  transformSource,
  "prepare_transform_bundle_linker_projection",
);
assert.equal(
  occurrences(linkerProjection, "style_sources_to_transform_bundle_modules(style_sources)"),
  1,
  "linker projection must consume the shared parsed-module producer once",
);
assert.equal(
  occurrences(linkerProjection, "collect_style_facts"),
  0,
  "per-instance projection must not parse source text",
);
assert.ok(
  linkerProjection.includes(".with_configuration_hashes(configurations)"),
  "configuration instances must expand from parsed module facts",
);

const dependencyResolver = functionSource(
  transformSource,
  "resolve_transform_bundle_projection_dependency_templates",
);
assert.ok(
  dependencyResolver.includes("resolution_context: TransformResolutionContext<'_>"),
  "dependency resolution must receive one preconstructed resolution context",
);
assert.ok(
  dependencyResolver.includes("resolution_context.resolve_style_module("),
  "dependency resolution must consume the supplied context for each projected edge",
);
assert.equal(
  occurrences(dependencyResolver, "TransformResolutionContext::from_resolution_inputs"),
  0,
  "the per-edge loop must not reconstruct resolver state",
);
assert.equal(
  occurrences(dependencyResolver, "collect_style_facts"),
  0,
  "the per-edge resolver loop must not reopen source parsing",
);

const configuredInstanceTest = functionSource(
  transformSource,
  "configured_sass_edges_select_distinct_instances_without_reparsing",
);
assert.ok(
  configuredInstanceTest.includes("with_omena_parser_parse_instrumentation"),
  "configured-instance coverage must observe parser materialization",
);
assert.ok(
  configuredInstanceTest.includes("parser_snapshot.parse_invocation_count, 3"),
  "configured-instance coverage must pin three source paths to three parses",
);

for (const fixturePath of fixturePaths) {
  assert.ok(existsSync(path.join(repoRoot, fixturePath)), `missing fixture ${fixturePath}`);
  const includePath = fixturePath.replace("rust/crates/omena-query/", "../../");
  assert.ok(
    testSource.includes(includePath),
    `${fixturePath} must be consumed by the product-path tests`,
  );
}

const cargoTests = [
  runCargoTest("bundle_code_split_workspace_plan_traverses_module_dependencies_without_reparsing"),
  runCargoTest("bundle_paths_consume_package_export_resolution"),
  runCargoTest("dependency_resolution_tests"),
];

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-query.linker-input-walk-authority",
      producers: [
        {
          consumer: "codeSplitReachability",
          producer: "parsedModuleDependencyProjection",
          sourceWalkCount: 1,
        },
        {
          consumer: "configuredModuleInstantiation",
          producer: "parsedModuleFacts",
          sourceWalkCount: 1,
        },
        {
          consumer: "dependencyResolution",
          producer: "transformResolutionContext",
          perEdgeResolutionContextBuildCount: 0,
        },
      ],
      fixtureCount: fixturePaths.length,
      cargoTests,
    },
    null,
    2,
  )}\n`,
);

function runCargoTest(filter: string): { filter: string; passedCount: number } {
  const result = spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "--all-features",
      filter,
      "--",
      "--nocapture",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  if (result.status !== 0) {
    throw new Error(`cargo test ${filter} failed\n${output}`);
  }
  const passedCount = [...output.matchAll(/test result: ok\. ([0-9]+) passed/gu)].reduce(
    (largest, match) => Math.max(largest, Number(match[1])),
    0,
  );
  assert.ok(passedCount > 0, `cargo test ${filter} must execute at least one test`);
  return { filter, passedCount };
}

function functionSource(source: string, functionName: string): string {
  const pattern = new RegExp(
    `(?:pub(?:\\([^)]*\\))?\\s+)?fn\\s+${escapeRegExp(functionName)}\\s*(?:<[^>{}]*>)?\\s*\\(`,
    "u",
  );
  const match = pattern.exec(source);
  assert.ok(match?.index !== undefined, `${functionName} must exist`);
  const openingBrace = source.indexOf("{", match.index);
  assert.ok(openingBrace >= 0, `${functionName} must have a body`);
  let depth = 0;
  for (let index = openingBrace; index < source.length; index += 1) {
    if (source[index] === "{") {
      depth += 1;
    } else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(match.index, index + 1);
      }
    }
  }
  throw new Error(`${functionName} body was not closed`);
}

function occurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

function replaceExactlyOnce(source: string, needle: string, replacement: string): string {
  assert.equal(occurrences(source, needle), 1, `${needle} must occur exactly once`);
  return source.replace(needle, replacement);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
