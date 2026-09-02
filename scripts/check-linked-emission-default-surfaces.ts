import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

interface SurfaceContractV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-default-surfaces";
  readonly defaultEmissionOrdering: "importOrderPreserving";
  readonly defaultDependencyAuthority: "resolvedDependencies";
  readonly targetReleaseVersion: "0.5.0";
  readonly legacyRemovalLedgerPath: string;
  readonly surfaces: ReadonlyArray<{
    readonly id: string;
    readonly sourcePath: string;
    readonly defaultNeedles: readonly string[];
    readonly legacyOptIn: string;
  }>;
  readonly legacyConstructorCallSites: ReadonlyArray<{
    readonly sourcePath: string;
    readonly constructor: string;
    readonly occurrenceCount: number;
  }>;
}

interface LegacyRemovalLedgerV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-legacy-removal-ledger";
  readonly entries: ReadonlyArray<{
    readonly id: string;
    readonly surface: string;
    readonly status: "deprecated";
    readonly introducedVersion: "0.5.0";
    readonly removalHorizon: "before-1.0";
    readonly successor: string;
    readonly reason: string;
  }>;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const injectSilentLegacyConstruction = process.argv.includes("--inject-silent-legacy-construction");
const injectAdapterLegacyBinding = process.argv.includes("--inject-adapter-legacy-binding");
const injectAdapterEmissionEnvironmentDrop = process.argv.includes(
  "--inject-adapter-emission-environment-drop",
);
const injectDefaultDrift = process.argv.includes("--inject-default-drift");
const injectViteEffectiveOptionsDrop = process.argv.includes(
  "--inject-vite-effective-options-drop",
);
const knownArguments = new Set([
  "--inject-silent-legacy-construction",
  "--inject-adapter-legacy-binding",
  "--inject-adapter-emission-environment-drop",
  "--inject-default-drift",
  "--inject-vite-effective-options-drop",
]);
for (const argument of process.argv.slice(2)) {
  assert.ok(knownArguments.has(argument), `unknown argument: ${argument}`);
}

const contract = readJson<SurfaceContractV0>("rust/omena-linked-emission-default-surfaces.json");
const ledger = readJson<LegacyRemovalLedgerV0>(contract.legacyRemovalLedgerPath);

assert.equal(contract.schemaVersion, "0");
assert.equal(contract.product, "omena.linked-emission-default-surfaces");
assert.equal(contract.defaultEmissionOrdering, "importOrderPreserving");
assert.equal(contract.defaultDependencyAuthority, "resolvedDependencies");
assert.equal(contract.targetReleaseVersion, "0.5.0");
assert.deepEqual(
  contract.surfaces.map((surface) => surface.id),
  [
    "rust-bundle-options",
    "rust-query-options",
    "command-line",
    "command-line-bundle-subcommand",
    "node-api",
    "webassembly",
    "javascript-adapter",
    "vite-plugin",
    "postcss-plugin",
  ],
);

for (const surface of contract.surfaces) {
  assert.ok(surface.legacyOptIn.length > 0, `${surface.id} has no explicit legacy opt-in`);
  const source = readSource(surface.sourcePath);
  for (const needle of surface.defaultNeedles) {
    assert.ok(source.includes(needle), `${surface.id} is missing default construction: ${needle}`);
  }
}

assert.equal(ledger.schemaVersion, "0");
assert.equal(ledger.product, "omena.linked-emission-legacy-removal-ledger");
assert.deepEqual(
  ledger.entries.map((entry) => entry.id),
  [
    "rust-link-options-legacy-compatibility",
    "rust-query-emission-path-legacy-compatibility",
    "cli-legacy-emission",
    "napi-legacy-bundle-export",
    "wasm-legacy-bundle-export",
    "js-adapter-legacy-emission-option",
    "vite-legacy-emission-option",
    "postcss-legacy-emission-option",
  ],
);
for (const entry of ledger.entries) {
  assert.equal(entry.status, "deprecated");
  assert.equal(entry.introducedVersion, "0.5.0");
  assert.equal(entry.removalHorizon, "before-1.0");
  assert.ok(entry.surface.length > 0);
  assert.ok(entry.successor.length > 0);
  assert.ok(entry.reason.length > 0);
}

const bundler = readSource("rust/crates/omena-bundler/src/lib.rs");
const emissionOrder = readSource("rust/crates/omena-bundler/src/emission_order.rs");
const queryTypes = readSource("rust/crates/omena-query/src/types.rs");
const cliCommands = readSource("rust/crates/omena-cli/src/commands.rs");
const cliBuild = readSource("rust/crates/omena-cli/src/build.rs");
const napi = readSource("rust/crates/omena-napi/src/lib.rs");
const wasm = readSource("rust/crates/omena-wasm/src/lib.rs");
const adapter = readSource("packages/css-build-adapter/index.cjs");
const adapterTypes = readSource("packages/css-build-adapter/index.d.ts");
const vite = readSource("packages/vite-plugin/index.cjs");
const viteTypes = readSource("packages/vite-plugin/index.d.ts");
const postcss = readSource("packages/postcss-plugin/index.cjs");
const postcssTypes = readSource("packages/postcss-plugin/index.d.ts");

assert.match(
  emissionOrder,
  /pub enum EmissionOrderingPolicyV0\s*\{[\s\S]*?#\[deprecated\([\s\S]*?since = "0\.5\.0"[\s\S]*?before 1\.0[\s\S]*?\)\]\s*ModuleIdLegacy,[\s\S]*?#\[default\]\s*ImportOrderPreserving,/u,
);
assert.match(
  bundler,
  /pub enum BundleResolutionAuthorityV0\s*\{[\s\S]*?#\[default\]\s*Resolved,[\s\S]*?#\[deprecated\([\s\S]*?since = "0\.5\.0"[\s\S]*?before 1\.0[\s\S]*?\)\]\s*LegacyPathInferred,/u,
);
assert.match(
  queryTypes,
  /pub enum OmenaQueryBundleEmissionPathV0\s*\{\s*#\[deprecated\([\s\S]*?since = "0\.5\.0"[\s\S]*?before 1\.0[\s\S]*?\)\]\s*ImportInlineLegacy,\s*#\[default\]\s*LinkedOrder,/u,
);
assert.match(cliCommands, /deprecated import-inline bundle emission path/u);
assert.match(cliCommands, /#\[arg\(long = "legacy-emission"\)\]/u);
assert.match(
  cliBuild,
  /if legacy_emission\s*\{[\s\S]*?OmenaQueryBundleEmissionPathV0::legacy_compatibility\(\)[\s\S]*?\}\s*else\s*\{\s*OmenaQueryBundleEmissionPathV0::default\(\)/u,
);

assertDefaultExecutionScopeSignature(napi, "bundle_style_sources_with_context_execution_scope");
assert.doesNotMatch(
  napi,
  /#\[napi\(js_name = "bundleStyleSourcesWithContextExecutionScopeJson"\)\]/u,
);
assert.match(napi, /\/\/\/ @deprecated Legacy import-inline bundle emission/u);
assertDefaultExecutionScopeSignature(wasm, "bundle_style_sources_with_context_execution_scope");
assert.doesNotMatch(
  wasm,
  /#\[wasm_bindgen\(js_name = bundleStyleSourcesWithContextExecutionScope\)\]/u,
);
assert.match(
  wasm,
  /#\[deprecated\([\s\S]*?legacy import-inline bundle emission is scheduled for removal before 1\.0/u,
);

assert.ok(
  adapter.includes(
    "const buildBundleSources = options.legacyEmission\n      ? engine.buildBundleSourcesLegacy\n      : engine.buildBundleSources;",
  ),
  "JS adapter no longer selects linked emission unless legacyEmission is explicit",
);
assert.equal(
  countOccurrences(adapter, "legacyEmission: Boolean(options.legacyEmission),"),
  1,
  "JS adapter cache identity must bind the selected emission mode exactly once",
);
const defaultAdapterBuilders = [
  ...adapter.matchAll(/async buildBundleSources\(input\) \{([\s\S]*?)\n\s{12}\},/gu),
];
assert.equal(defaultAdapterBuilders.length, 2, "expected one N-API and one WASM default builder");
for (const match of defaultAdapterBuilders) {
  const body = match[1] ?? "";
  assert.doesNotMatch(body, /Legacy/u);
  assert.doesNotMatch(body, /\bfalse\b/u);
}
assert.equal(countOccurrences(adapter, "async buildBundleSourcesLegacy(input)"), 2);
assert.match(adapterTypes, /@deprecated Legacy import-inline (?:bundle )?emission/u);
assert.match(
  vite,
  /async transform\(code, id\) \{[\s\S]*?const effectiveOptions = await resolveEffectiveOptions\(options, state\);[\s\S]*?const output = await rebuildViteSource\(\s*fileId,\s*code,\s*diskSource,\s*upstreamMap,\s*effectiveOptions,\s*state,\s*\);/u,
  "Vite transform must delegate the resolved emission option through the provenance-aware build path",
);
assert.match(
  vite,
  /async function rebuildViteSource\(fileId, source, diskSource, upstreamMap, effectiveOptions, state\) \{[\s\S]*?const identityContext = isDiskBacked[\s\S]*?const output = await rebuildAndCache\(fileId, source, effectiveOptions, state, identityContext\);/u,
  "Vite provenance-aware builds must preserve resolved emission options at the shared adapter boundary",
);
assert.match(viteTypes, /@deprecated Legacy import-inline (?:bundle )?emission/u);
assert.match(
  postcss,
  /const effectiveOptions = \{[\s\S]*?\.\.\.\(await resolveEffectiveOptions\(options, state\)\)[\s\S]*?const output = await rebuildAndCache\(filePath, source, effectiveOptions, state\);/u,
  "PostCSS must delegate the resolved emission option to the shared adapter build",
);
assert.match(postcssTypes, /@deprecated Legacy import-inline (?:bundle )?emission/u);

const productionRustFiles = gitFiles("rust/crates/**/*.rs").filter(
  (sourcePath) => !sourcePath.includes("/tests/"),
);
const productionRust = productionRustFiles.map((sourcePath) => ({
  sourcePath,
  source: readSource(sourcePath),
}));
const directQueryLegacySelections = productionRust.flatMap(({ sourcePath, source }) =>
  matchesWithLocations(
    sourcePath,
    source,
    /bundle_emission_path\s*[:=]\s*OmenaQueryBundleEmissionPathV0::ImportInlineLegacy\b/gu,
  ),
);
assert.deepEqual(
  directQueryLegacySelections,
  [],
  `legacy query selection bypasses legacy_compatibility(): ${directQueryLegacySelections.join(", ")}`,
);
assert.equal(
  countOccurrences(bundler, "emission_ordering_policy: EmissionOrderingPolicyV0::ModuleIdLegacy"),
  1,
  "ModuleIdLegacy must appear in exactly the named bundler compatibility constructor",
);
assert.equal(
  countOccurrences(
    bundler,
    "dependency_resolution_authority: BundleResolutionAuthorityV0::LegacyPathInferred",
  ),
  1,
  "LegacyPathInferred must appear in exactly the named bundler compatibility constructor",
);

const actualLegacyCallSites = productionRust
  .flatMap(({ sourcePath, source }) =>
    [
      "TransformBundleLinkOptionsV0::legacy_compatibility()",
      "OmenaQueryBundleEmissionPathV0::legacy_compatibility()",
    ].flatMap((constructor) => {
      const occurrenceCount = countOccurrences(source, constructor);
      return occurrenceCount === 0 ? [] : [{ sourcePath, constructor, occurrenceCount }];
    }),
  )
  .toSorted((left, right) =>
    compareCodePoints(
      `${left.sourcePath}\0${left.constructor}`,
      `${right.sourcePath}\0${right.constructor}`,
    ),
  );
const expectedLegacyCallSites = [...contract.legacyConstructorCallSites].toSorted((left, right) =>
  compareCodePoints(
    `${left.sourcePath}\0${left.constructor}`,
    `${right.sourcePath}\0${right.constructor}`,
  ),
);
assert.deepEqual(actualLegacyCallSites, expectedLegacyCallSites);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena.linked-emission-default-surface-evaluation",
      decision: "green",
      surfaceCount: contract.surfaces.length,
      defaultEmissionOrdering: contract.defaultEmissionOrdering,
      defaultDependencyAuthority: contract.defaultDependencyAuthority,
      directLegacyConstructionSiteCount: directQueryLegacySelections.length,
      explicitLegacyConstructorSiteCount: actualLegacyCallSites.length,
      legacyRemovalRowCount: ledger.entries.length,
      targetReleaseVersion: contract.targetReleaseVersion,
    },
    null,
    2,
  )}\n`,
);

function readJson<T>(sourcePath: string): T {
  return JSON.parse(readSource(sourcePath)) as T;
}

function readSource(sourcePath: string): string {
  let source = readFileSync(path.join(repoRoot, sourcePath), "utf8");
  if (injectSilentLegacyConstruction && sourcePath === "rust/crates/omena-napi/src/lib.rs") {
    source +=
      "\nconst _INJECTED: OmenaQueryConsumerBuildOptionsV0 = OmenaQueryConsumerBuildOptionsV0 { bundle_emission_path: OmenaQueryBundleEmissionPathV0::ImportInlineLegacy };\n";
  }
  if (injectAdapterLegacyBinding && sourcePath === "packages/css-build-adapter/index.cjs") {
    source = source.replace(": engine.buildBundleSources;", ": engine.buildBundleSourcesLegacy;");
  }
  if (
    injectAdapterEmissionEnvironmentDrop &&
    sourcePath === "packages/css-build-adapter/index.cjs"
  ) {
    source = source.replace("    legacyEmission: Boolean(options.legacyEmission),\n", "");
  }
  if (injectDefaultDrift && sourcePath === "rust/crates/omena-query/src/types.rs") {
    source = source
      .replace(
        "pub enum OmenaQueryBundleEmissionPathV0 {\n",
        "pub enum OmenaQueryBundleEmissionPathV0 {\n    #[default]\n",
      )
      .replace(
        "    ImportInlineLegacy,\n    #[default]\n    LinkedOrder,",
        "    ImportInlineLegacy,\n    LinkedOrder,",
      );
  }
  if (injectViteEffectiveOptionsDrop && sourcePath === "packages/vite-plugin/index.cjs") {
    const mutated = source.replace(
      "        effectiveOptions,\n        state,\n      );",
      "        {},\n        state,\n      );",
    );
    assert.notEqual(mutated, source, "Vite effective-options mutation did not match the source");
    source = mutated;
  }
  return source;
}

function assertDefaultExecutionScopeSignature(source: string, functionName: string): void {
  const signature = source.match(new RegExp(`pub fn ${functionName}\\([\\s\\S]*?\\) ->`, "u"))?.[0];
  assert.ok(signature, `missing ${functionName} signature`);
  assert.doesNotMatch(signature, /legacy_emission|linked_emission|:\s*bool/u);
}

function gitFiles(glob: string): string[] {
  const result = evidenceScanSurface.spawnSync("git", ["ls-files", glob], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.split("\n").filter(Boolean);
}

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

function matchesWithLocations(sourcePath: string, source: string, pattern: RegExp): string[] {
  return [...source.matchAll(pattern)].map((match) => {
    const line = source.slice(0, match.index).split("\n").length;
    return `${sourcePath}:${line}`;
  });
}

function compareCodePoints(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
