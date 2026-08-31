import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import ts from "../server/engine-core-ts/src/ts-facade";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type SourceEntry = { readonly relativePath: string; source: string };
type MergeSite = { readonly relativePath: string; readonly offset: number; readonly kind: string };

const repoRoot = path.resolve(import.meta.dirname, "..");
const sourceDirectoryScopes = [
  "packages/css-build-adapter/",
  "packages/vite-plugin/",
  "rust/crates/omena-napi/src/",
  "rust/crates/omena-query/src/",
  "rust/crates/omena-wasm/src/",
] as const;
const exactSourcePaths = new Set([
  "contracts/engine-sdk-workflow/main.tsp",
  "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
]);
const targets = gitTrackedFiles().filter(isAuditedExportPlaneSource).toSorted();
const sources: SourceEntry[] = targets.map((relativePath) => ({
  relativePath,
  source: fs.readFileSync(path.join(repoRoot, relativePath), "utf8"),
}));

assert.ok(
  targets.includes("rust/crates/omena-query/src/style/module_interface.rs"),
  "the CSS Module interface producer must be inside the audited source domain",
);

if (process.argv.includes("--inject-regex-classmap")) {
  sources.push({
    relativePath: "packages/css-build-adapter/injected-helper.cjs",
    source: "const classMap = Object.fromEntries(emittedCss.matchAll(/\\.([a-z-]+)/gu));",
  });
}
if (process.argv.includes("--inject-merged-export-namespace")) {
  sources.push({
    relativePath: "packages/css-build-adapter/injected-spread-merge.cjs",
    source: "const mergedExports = { ...classExports, ...valueExports };",
  });
}
if (process.argv.includes("--inject-object-assign-export-namespace")) {
  sources.push({
    relativePath: "packages/css-build-adapter/injected-object-assign-merge.cjs",
    source: "const mergedExports = Object.assign({}, classExports, valueExports);",
  });
}
if (process.argv.includes("--inject-entries-export-namespace")) {
  sources.push({
    relativePath: "packages/css-build-adapter/injected-entries-merge.cjs",
    source:
      "const mergedExports = Object.fromEntries([...Object.entries(classExports), ...Object.entries(valueExports)]);",
  });
}
if (process.argv.includes("--inject-query-module-interface-merged-namespace")) {
  mutateSource("rust/crates/omena-query/src/style/module_interface.rs", (source) =>
    source.concat(
      "\nfn injected_merge(class_exports: BTreeMap<String, String>, value_exports: BTreeMap<String, String>) {\n",
      "    let merged_exports = class_exports.into_iter().chain(value_exports).collect::<BTreeMap<_, _>>();\n",
      "    drop(merged_exports);\n",
      "}\n",
    ),
  );
}

const semanticTransportAnchors = [
  {
    id: "query-style-module-link",
    relativePath: "rust/crates/omena-query/src/style.rs",
    needles: ["mod module_interface;", "pub use module_interface::{"],
  },
  {
    id: "query-module-interface-families",
    relativePath: "rust/crates/omena-query/src/style/module_interface.rs",
    needles: [
      "pub class_exports:",
      "pub icss_exports:",
      "declare const styles: typeof classExports;",
    ],
  },
  {
    id: "query-bundler-host-families",
    relativePath: "rust/crates/omena-query/src/bundler_host.rs",
    needles: [
      "let mut class_exports",
      "let mut value_exports",
      "class_exports.insert",
      "value_exports.insert",
    ],
  },
  {
    id: "typespec-contract-families",
    relativePath: "contracts/engine-sdk-workflow/main.tsp",
    needles: ["classExports: Record<string>;", "valueExports: Record<string>;"],
  },
  {
    id: "rust-generated-contract-families",
    relativePath: "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs",
    needles: [
      "pub class_exports: BTreeMap<String, String>",
      "pub value_exports: BTreeMap<String, String>",
    ],
  },
  {
    id: "typescript-generated-contract-families",
    relativePath: "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
    needles: ["readonly classExports: RecordStringJson", "readonly valueExports: RecordStringJson"],
  },
  {
    id: "adapter-generated-contract-families",
    relativePath: "packages/css-build-adapter/bundler-host-contract.generated.d.ts",
    needles: ["readonly classExports:", "readonly valueExports:"],
  },
  {
    id: "adapter-runtime-families",
    relativePath: "packages/css-build-adapter/index.cjs",
    needles: [
      "classExports: moduleInterface.classExports",
      "valueExports: moduleInterface.valueExports",
    ],
  },
  {
    id: "adapter-public-output-families",
    relativePath: "packages/css-build-adapter/index.d.ts",
    needles: ["readonly classExports?:", "readonly valueExports?:"],
  },
  {
    id: "vite-runtime-families",
    relativePath: "packages/vite-plugin/index.cjs",
    needles: [
      "const classExports = ${JSON.stringify(classExports)}",
      "const valueExports = ${JSON.stringify(valueExports)}",
    ],
  },
  {
    id: "napi-bundler-host-boundary",
    relativePath: "rust/crates/omena-napi/src/lib.rs",
    needles: [
      "resolve_css_module_for_bundler_host_json",
      "resolve_omena_bundler_host_module_v0(request)",
    ],
  },
  {
    id: "wasm-bundler-host-boundary",
    relativePath: "rust/crates/omena-wasm/src/lib.rs",
    needles: [
      "resolve_css_module_for_bundler_host(request",
      "resolve_omena_bundler_host_module_v0(request)",
    ],
  },
] as const;

if (process.argv.includes("--inject-dropped-semantic-transport-anchor")) {
  mutateSource("packages/css-build-adapter/index.cjs", (source) =>
    source.replace("valueExports: moduleInterface.valueExports", "valueExports: {}"),
  );
}

const regexClassMapSites = sources.flatMap(findRegexClassMapSites);
const mergedNamespaceMaps = sources.flatMap(findMergedNamespaceMaps);
const emptyClassMapNegativeControl = findMergedNamespaceMaps({
  relativePath: "packages/css-build-adapter/empty-class-map-negative-control.cjs",
  source: "const classMap = {};",
});
const missingSemanticTransportAnchors = semanticTransportAnchors
  .filter(({ relativePath, needles }) => {
    const source = sources.find((entry) => entry.relativePath === relativePath)?.source;
    return source === undefined || needles.some((needle) => !source.includes(needle));
  })
  .map(({ id }) => id);

assert.deepEqual(
  regexClassMapSites,
  [],
  `CSS-text class-map extraction sites found: ${JSON.stringify(regexClassMapSites)}`,
);
assert.deepEqual(
  mergedNamespaceMaps,
  [],
  `class and ICSS value export namespaces were merged: ${JSON.stringify(mergedNamespaceMaps)}`,
);
assert.deepEqual(
  emptyClassMapNegativeControl,
  [],
  "an empty class-only map must not be classified as an export-family merge",
);
assert.deepEqual(
  missingSemanticTransportAnchors,
  [],
  `semantic transport anchors missing: ${missingSemanticTransportAnchors.join(", ")}`,
);

process.stdout.write(
  `${JSON.stringify({
    schemaVersion: "0",
    product: "js-bundler-host.no-regex-classmap",
    exportPlaneFiles: sources.length,
    semanticTransportAnchors:
      semanticTransportAnchors.length - missingSemanticTransportAnchors.length,
    regexClassMapSites: regexClassMapSites.length,
    mergedNamespaceMaps: mergedNamespaceMaps.length,
    negativeControls: 1,
  })}\n`,
);

function gitTrackedFiles(): string[] {
  const result = evidenceScanSurface.spawnSync("git", ["ls-files", "-z"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout.split("\0").filter(Boolean);
}

function isAuditedExportPlaneSource(relativePath: string): boolean {
  if (exactSourcePaths.has(relativePath)) return true;
  if (!sourceDirectoryScopes.some((prefix) => relativePath.startsWith(prefix))) return false;
  return /\.(?:cjs|mjs|js|cts|mts|ts|tsx|rs)$/u.test(relativePath);
}

function mutateSource(relativePath: string, mutate: (source: string) => string): void {
  const entry = sources.find((candidate) => candidate.relativePath === relativePath);
  assert.ok(entry, `${relativePath} must be discovered before it can be mutated`);
  const mutated = mutate(entry.source);
  assert.notEqual(
    mutated,
    entry.source,
    `${relativePath} mutation must change the discovered source`,
  );
  entry.source = mutated;
}

function findRegexClassMapSites(entry: SourceEntry): MergeSite[] {
  const sites: MergeSite[] = [];
  for (const needle of ["extractCssModuleClassMap", ".matchAll("] as const) {
    let offset = entry.source.indexOf(needle);
    while (offset !== -1) {
      sites.push({ relativePath: entry.relativePath, offset, kind: needle });
      offset = entry.source.indexOf(needle, offset + needle.length);
    }
  }
  return sites;
}

function findMergedNamespaceMaps(entry: SourceEntry): MergeSite[] {
  if (/\.(?:cjs|mjs|js|cts|mts|ts|tsx)$/u.test(entry.relativePath)) {
    return findJavaScriptMergedNamespaceMaps(entry);
  }
  if (entry.relativePath.endsWith(".rs")) {
    return findRustMergedNamespaceMaps(entry);
  }
  return [];
}

function findJavaScriptMergedNamespaceMaps(entry: SourceEntry): MergeSite[] {
  const sourceFile = ts.createSourceFile(
    entry.relativePath,
    entry.source,
    ts.ScriptTarget.Latest,
    true,
    scriptKindFor(entry.relativePath),
  );
  const sites: MergeSite[] = [];
  const visit = (node: ts.Node): void => {
    const kind = mergeConstructKind(node);
    if (kind) {
      sites.push({ relativePath: entry.relativePath, offset: node.getStart(sourceFile), kind });
      return;
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return sites;
}

function mergeOperandFamilies(node: ts.Node): Set<"class" | "value"> {
  const families = new Set<"class" | "value">();
  const visit = (candidate: ts.Node): void => {
    if (candidate !== node && isNestedFunctionBoundary(candidate)) return;
    if (ts.isObjectLiteralExpression(candidate)) {
      for (const property of candidate.properties) {
        if (property.kind === ts.SyntaxKind.SpreadAssignment) {
          visit((property as { readonly expression: ts.Expression }).expression);
        }
      }
      return;
    }
    if (ts.isArrayLiteralExpression(candidate)) {
      for (const element of candidate.elements) {
        visit(ts.isSpreadElement(element) ? element.expression : element);
      }
      return;
    }
    if (ts.isIdentifier(candidate)) {
      if (candidate.text === "classExports" || candidate.text === "class_exports") {
        families.add("class");
      }
      if (candidate.text === "valueExports" || candidate.text === "value_exports") {
        families.add("value");
      }
    }
    ts.forEachChild(candidate, visit);
  };
  visit(node);
  return families;
}

function mergeConstructKind(node: ts.Node): string | null {
  if (ts.isObjectLiteralExpression(node)) {
    const spreads = node.properties.flatMap((property) =>
      property.kind === ts.SyntaxKind.SpreadAssignment
        ? [(property as { readonly expression: ts.Expression }).expression]
        : [],
    );
    if (spreads.length > 0 && hasBothFamilies(spreads)) {
      return "object-spread";
    }
  }
  if (ts.isArrayLiteralExpression(node)) {
    const spreads = node.elements.filter(ts.isSpreadElement);
    if (spreads.length > 0 && hasBothFamilies(spreads.map(({ expression }) => expression))) {
      return "array-spread";
    }
  }
  if (ts.isCallExpression(node)) {
    const callee = node.expression.getText();
    if (hasBothFamilies(node.arguments)) {
      if (callee === "Object.assign") return "Object.assign";
      if (callee === "Object.fromEntries") return "Object.fromEntries";
      if (callee.endsWith(".concat")) return "Array.concat";
    }
  }
  if (
    ts.isNewExpression(node) &&
    node.expression.getText() === "Map" &&
    hasBothFamilies(node.arguments ?? [])
  ) {
    return "Map";
  }
  return null;
}

function isNestedFunctionBoundary(node: ts.Node): boolean {
  return (
    ts.isArrowFunction(node) ||
    ts.isFunctionDeclaration(node) ||
    ts.isFunctionExpression(node) ||
    ts.isMethodDeclaration(node) ||
    ts.isConstructorDeclaration(node) ||
    ts.isGetAccessorDeclaration(node) ||
    ts.isSetAccessorDeclaration(node)
  );
}

function hasBothFamilies(nodes: readonly ts.Node[]): boolean {
  const families = new Set<"class" | "value">();
  for (const node of nodes) {
    for (const family of mergeOperandFamilies(node)) families.add(family);
  }
  return families.has("class") && families.has("value");
}

function findRustMergedNamespaceMaps(entry: SourceEntry): MergeSite[] {
  const sites: MergeSite[] = [];
  const statementPattern =
    /[^;]*(?:class_exports[^;]*value_exports|value_exports[^;]*class_exports)[^;]*;/gu;
  for (const match of entry.source.matchAll(statementPattern)) {
    if (!/\.(?:chain|extend)\s*\(|collect\s*::<[^>]*(?:Map|map)|from_iter\s*\(/u.test(match[0])) {
      continue;
    }
    sites.push({
      relativePath: entry.relativePath,
      offset: match.index,
      kind: "rust-map-composition",
    });
  }
  return sites;
}

function scriptKindFor(relativePath: string): ts.ScriptKind {
  if (relativePath.endsWith(".tsx")) return ts.ScriptKind.TSX;
  if (relativePath.endsWith(".jsx")) return ts.ScriptKind.JSX;
  if (/\.(?:cjs|mjs|js)$/u.test(relativePath)) return ts.ScriptKind.JS;
  return ts.ScriptKind.TS;
}
