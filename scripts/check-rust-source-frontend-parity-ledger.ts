import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import ts from "../server/engine-core-ts/src/ts-facade";
import { resolveSourceFrontendBackendKind } from "../server/engine-host-node/src/source-frontend-analysis-provider";

const repoRoot = process.cwd();
const ledgerPath = path.join(repoRoot, "rust/omena-source-frontend-parity-ledger.json");

interface SymbolEvidence {
  readonly path: string;
  readonly symbols: readonly string[];
}

interface EntryGate {
  readonly id: string;
  readonly status: "met" | "blocked";
  readonly evidence: readonly SymbolEvidence[];
}

interface Component {
  readonly id: "syntax" | "binding" | "sparse-cfg";
  readonly status: "TS_OWNED" | "RETIRED";
  readonly rustAuthority: string;
  readonly oracle: string;
  readonly oracleStatus: "not-built" | "partial-green" | "green";
  readonly tsLiveSurfaces: readonly SymbolEvidence[];
}

interface Survivor {
  readonly id: string;
  readonly status: "TYPE_ORACLE_SURVIVOR";
  readonly boundary: string;
  readonly surfaces: readonly SymbolEvidence[];
}

interface SourceFrontendParityLedger {
  readonly schemaVersion: 0;
  readonly product: "omena.source-frontend-parity-ledger";
  readonly entryGates: readonly EntryGate[];
  readonly components: readonly Component[];
  readonly survivors: readonly Survivor[];
  readonly forbiddenBeforeOracleGreen: readonly string[];
}

const ledger = JSON.parse(readFileSync(ledgerPath, "utf8")) as SourceFrontendParityLedger;

assert.equal(resolveSourceFrontendBackendKind({}), "rust-source-frontend");
assert.throws(
  () => resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "typescript-current" }),
  /Unknown source frontend backend: typescript-current/u,
);

assert.equal(ledger.schemaVersion, 0);
assert.equal(ledger.product, "omena.source-frontend-parity-ledger");
assert.deepEqual(ledger.entryGates.map((gate) => gate.id).toSorted(), [
  "g5-source-symbol-identity",
  "g7-session-transaction-semantic-graph",
  "g8-tsgo-type-oracle-capabilities",
]);

for (const gate of ledger.entryGates) {
  assert.equal(gate.status, "met", `${gate.id} must be met before G11 enters committed scope`);
  assert.ok(gate.evidence.length > 0, `${gate.id} must carry file/symbol evidence`);
  assertEvidence(gate.evidence, `entry gate ${gate.id}`);
}

assert.deepEqual(
  ledger.components.map((component) => component.id),
  ["syntax", "binding", "sparse-cfg"],
);

for (const component of ledger.components) {
  assert.ok(
    component.rustAuthority.includes("::"),
    `${component.id} needs a Rust authority anchor`,
  );
  if (component.status === "TS_OWNED") {
    assertEvidence(component.tsLiveSurfaces, `component ${component.id}`);
  } else {
    assert.equal(
      component.oracleStatus,
      "green",
      `${component.id} cannot be RETIRED before its oracle is green`,
    );
    assert.equal(
      component.tsLiveSurfaces.length,
      0,
      `${component.id} is RETIRED and must not list TypeScript live surfaces`,
    );
    assertNoLiveEvidence(component.tsLiveSurfaces, `component ${component.id}`);
  }
}

assert.equal(ledger.survivors.length, 1);
const [tsgo] = ledger.survivors;
assert.equal(tsgo?.id, "tsgo-type-oracle");
assert.equal(tsgo?.status, "TYPE_ORACLE_SURVIVOR");
assert.ok(
  tsgo?.boundary.includes("type-query provider"),
  "tsgo survivor boundary must stay type-query-only",
);
assertEvidence(tsgo?.surfaces ?? [], "tsgo survivor");

const bridgeSource = readRepoFile("rust/crates/omena-bridge/src/source_syntax.rs");
for (const forbidden of [
  "target.binding == identifier.name.as_str()",
  "binding.binding == argument.binding",
  "target.binding == style_binding",
  "recipe.local_name == binding",
]) {
  assert.equal(
    bridgeSource.includes(forbidden),
    false,
    `G5 identity gate regressed to name matching: ${forbidden}`,
  );
}

const tsgoClient = readRepoFile("rust/crates/omena-tsgo-client/src/lib.rs");
assert.match(tsgoClient, /ProviderUnresolvedDisciplineV0::UnknownNotGuess/);
assert.match(tsgoClient, /TSGO_TYPE_ORACLE_PROVIDER_KIND_V0:\s*&str\s*=\s*"type-oracle"/);

const typeFactCfgProvider = readRepoFile(
  "server/engine-host-node/src/type-fact-control-flow-graph.ts",
);
assert.equal(
  typeFactCfgProvider.includes("resolveFlowClassValues"),
  false,
  "sparse CFG product path must not fall back to the TypeScript flow analyzer",
);
assert.equal(
  typeFactCfgProvider.includes("TypescriptFallback"),
  false,
  "sparse CFG product path must not expose a TypeScript fallback entrypoint",
);

assertNoProductSourceFrontendFallbacks();
const importSyntaxProducerSites = assertNoImportSyntaxRegexProducers();
assertTsSourceFrontendOracleIsNotProductPath();

console.log(
  JSON.stringify(
    {
      product: "omena.source-frontend-parity-ledger.check",
      entryGates: ledger.entryGates.length,
      sourceFrontendDefault: resolveSourceFrontendBackendKind({}),
      components: ledger.components.map(({ id, status, oracleStatus }) => ({
        id,
        status,
        oracleStatus,
      })),
      survivors: ledger.survivors.map(({ id, status }) => ({ id, status })),
      importSyntaxProducerSites,
    },
    null,
    2,
  ),
);

function assertEvidence(evidence: readonly SymbolEvidence[], label: string): void {
  for (const item of evidence) {
    const content = readRepoFile(item.path);
    assert.ok(item.symbols.length > 0, `${label} evidence ${item.path} has no symbols`);
    for (const symbol of item.symbols) {
      assert.ok(
        content.includes(symbol),
        `${label} evidence missing ${JSON.stringify(symbol)} in ${item.path}`,
      );
    }
  }
}

function assertNoLiveEvidence(evidence: readonly SymbolEvidence[], label: string): void {
  for (const item of evidence) {
    const content = readRepoFileOrNull(item.path);
    if (content === null) continue;
    for (const symbol of item.symbols) {
      assert.equal(
        content.includes(symbol),
        false,
        `${label} retired surface still contains ${JSON.stringify(symbol)} in ${item.path}`,
      );
    }
  }
}

function assertNoProductSourceFrontendFallbacks(): void {
  const productFiles = productSourceFrontendFiles();
  const forbiddenPatterns = [
    "buildSourceBinder(",
    "buildSourceBindingGraph(",
    "resolveFlowClassValues(",
    "buildFlowSlice(",
    "buildFlowNodes(",
    "TypescriptFallback",
    "../engine-core-ts/src/core/binder/binder-builder",
    "../engine-core-ts/src/core/source-frontend/ts-source-binder-oracle",
    "../engine-core-ts/src/core/source-frontend/ts-flow-class-value-oracle",
    "../engine-core-ts/src/core/source-frontend/ts-flow-slice-oracle",
    "../engine-core-ts/src/core/source-frontend/ts-source-cfg-oracle",
    "../engine-core-ts/src/core/flow/class-value-analysis",
    "../engine-core-ts/src/core/flow/flow-slice",
    "../engine-core-ts/src/core/flow/cfg",
    "SourceFileCache",
    "sourceFileCache",
    "entry.sourceFile.text",
    "ctx.entry.sourceFile.text",
    "cached?.sourceFile.fileName",
    "readonly sourceFile: ts.SourceFile",
    "IMPORT_FROM_PATTERN",
    "CLASSNAMES_BIND_INITIALIZER_PATTERN",
    "bindingIndexWithImportFallbacks",
    "importedStyleBindingsJson",
    "classnamesBindBindingsJson",
  ];

  for (const filePath of productFiles) {
    const content = readRepoFile(filePath);
    for (const pattern of forbiddenPatterns) {
      assert.equal(
        content.includes(pattern),
        false,
        `product source frontend path must not reintroduce ${JSON.stringify(pattern)} in ${filePath}`,
      );
    }
  }

  const analysisCache = readRepoFile(
    "server/engine-core-ts/src/core/indexing/document-analysis-cache.ts",
  );
  assert.match(
    analysisCache,
    /Rust source frontend analysis is required/u,
    "DocumentAnalysisCache must keep Rust source frontend analysis as a required product-path dependency",
  );
  assert.doesNotMatch(
    analysisCache,
    /sourceFrontendAnalysis\?:/u,
    "DocumentAnalysisCache must not make sourceFrontendAnalysis optional",
  );
}

interface ImportSyntaxRegexProducerSiteV0 {
  readonly path: string;
  readonly line: number;
  readonly method: string;
  readonly syntax: "import" | "classnamesBind";
}

function assertNoImportSyntaxRegexProducers(): number {
  const sources = new Map(
    productSourceFrontendFiles().map((filePath) => [filePath, readRepoFile(filePath)]),
  );
  if (process.argv.includes("--inject-import-regex-producer")) {
    sources.set(
      "__injection__/source-import-regex-producer.ts",
      String.raw`const importKeyword = "import";
const pattern = "^\\s*" + importKeyword + "\\s+(.+?)\\s+from\\s+['\"](.+?)['\"]";
const producer = new RegExp(pattern, "gm");
const renamedProducer = producer;
export function injected(source: string) { return [...source.matchAll(renamedProducer)]; }
`,
    );
  }
  if (process.argv.includes("--inject-import-regex-replace-callback")) {
    sources.set(
      "__injection__/source-import-regex-replace-callback.ts",
      String.raw`const importPattern = /^\s*imp[o]rt\s+(.+?)\s+from\s+['"](.+?)['"]/gm;
export function injected(source: string) { return source.replace(importPattern, (_match, binding, specifier) => binding + specifier); }
`,
    );
  }
  if (process.argv.includes("--inject-import-regex-property-exec")) {
    sources.set(
      "__injection__/source-import-regex-property-exec.ts",
      String.raw`const syntax = { importPattern: /^\s*import\s+(.+?)\s+from\s+['"](.+?)['"]/gm };
export function injected(source: string) { return syntax.importPattern.exec(source); }
`,
    );
  }
  if (process.argv.includes("--inject-import-regex-split")) {
    sources.set(
      "__injection__/source-import-regex-split.ts",
      String.raw`const importPattern = /^\s*import\s+(.+?)\s+from\s+['"](.+?)['"]/gm;
export function injected(source: string) { return source.split(importPattern); }
`,
    );
  }
  if (process.argv.includes("--inject-import-regex-character-class")) {
    sources.set(
      "__injection__/source-import-regex-character-class.ts",
      String.raw`const pattern = /^\s*imp[o]rt\s+(.+?)\s+from\s+['"](.+?)['"]/gm;
export function injected(source: string) { return pattern.test(source); }
`,
    );
  }
  if (process.argv.includes("--inject-import-regex-string-raw")) {
    sources.set(
      "__injection__/source-import-regex-string-raw.ts",
      'const pattern = new RegExp(String.raw`^\\s*import\\s+(.+?)\\s+from\\s+[\'"](.+?)[\'"]`, "gm");\nexport function injected(source: string) { return pattern.exec(source); }\n',
    );
  }
  if (process.argv.includes("--inject-classnames-bind-regex-producer")) {
    sources.set(
      "__injection__/source-classnames-bind-regex-producer.ts",
      String.raw`const bindPattern = /const\s+(\w+)\s*=\s*classNames\.bind\((\w+)\)/g;
export function injected(source: string) { return [...source.matchAll(bindPattern)]; }
`,
    );
  }
  if (process.argv.includes("--inject-sibling-import-regex-producer")) {
    sources.set(
      "server/engine-core-ts/src/core/indexing/__injected-source-import-regex-producer.ts",
      String.raw`const siblingPattern = /^\s*import\s+(.+?)\s+from\s+['"](.+?)['"]/gm;
export function injected(source: string) { return source.search(siblingPattern); }
`,
    );
  }

  const sites = [...sources].flatMap(([filePath, source]) =>
    importSyntaxRegexProducerSites(filePath, source),
  );
  assert.deepEqual(
    sites,
    [],
    `source frontend host contains import-syntax regex producers: ${sites
      .map((site) => `${site.path}:${site.line} via ${site.method} (${site.syntax})`)
      .join(", ")}`,
  );
  return sites.length;
}

function importSyntaxRegexProducerSites(
  filePath: string,
  source: string,
): readonly ImportSyntaxRegexProducerSiteV0[] {
  const sourceFile = ts.createSourceFile(
    filePath,
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const staticStrings = new Map<string, string>();
  const regexValues = new Map<string, RegexValueV0>();
  const declarations: ts.VariableDeclaration[] = [];

  const collect = (node: ts.Node): void => {
    if (ts.isVariableDeclaration(node)) {
      declarations.push(node);
    }
    ts.forEachChild(node, collect);
  };
  collect(sourceFile);

  let changed = true;
  while (changed) {
    changed = false;
    for (const declaration of declarations) {
      if (!ts.isIdentifier(declaration.name) || !declaration.initializer) continue;
      const name = declaration.name.text;
      changed = bindStaticValue(name, declaration.initializer) || changed;
      if (ts.isObjectLiteralExpression(declaration.initializer)) {
        for (const property of declaration.initializer.properties) {
          if (!ts.isPropertyAssignment(property)) continue;
          const propertyName = staticPropertyName(property.name, sourceFile, staticStrings);
          if (propertyName === null) continue;
          changed = bindStaticValue(`${name}.${propertyName}`, property.initializer) || changed;
        }
      }
    }
  }

  function bindStaticValue(name: string, expression: ts.Expression): boolean {
    let bound = false;
    const staticValue = staticStringValue(expression, sourceFile, staticStrings);
    if (staticValue !== null && staticStrings.get(name) !== staticValue) {
      staticStrings.set(name, staticValue);
      bound = true;
    }
    const regexValue = regexValueForExpression(expression, sourceFile, staticStrings, regexValues);
    if (regexValue && regexValueKey(regexValues.get(name)) !== regexValueKey(regexValue)) {
      regexValues.set(name, regexValue);
      bound = true;
    }
    return bound;
  }

  const producerSyntaxes = (node: ts.Expression | undefined) => {
    if (!node) return [];
    const regexValue = regexValueForExpression(node, sourceFile, staticStrings, regexValues);
    return regexValue ? sourceSyntaxesMatchedByRegex(regexValue) : [];
  };

  const sites: ImportSyntaxRegexProducerSiteV0[] = [];
  const inspect = (node: ts.Node): void => {
    if (ts.isCallExpression(node)) {
      const method = callMethod(node.expression, sourceFile, staticStrings);
      const receiver = callReceiver(node.expression);
      const producer =
        method && ["match", "matchAll", "search", "replace", "replaceAll", "split"].includes(method)
          ? node.arguments[0]
          : method && ["exec", "test"].includes(method)
            ? receiver
            : undefined;
      for (const syntax of producerSyntaxes(producer)) {
        const { line } = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
        sites.push({ path: filePath, line: line + 1, method: method!, syntax });
      }
    }
    ts.forEachChild(node, inspect);
  };
  inspect(sourceFile);
  return sites;
}

interface RegexValueV0 {
  readonly source: string;
  readonly flags: string;
}

function regexValueForExpression(
  node: ts.Expression,
  sourceFile: ts.SourceFile,
  staticStrings: ReadonlyMap<string, string>,
  regexValues: ReadonlyMap<string, RegexValueV0>,
): RegexValueV0 | null {
  const unwrapped = unwrapExpression(node);
  const key = expressionKey(unwrapped, sourceFile, staticStrings);
  if (key && regexValues.has(key)) return regexValues.get(key)!;
  if (ts.isRegularExpressionLiteral(unwrapped)) {
    const literal = unwrapped.getText(sourceFile);
    const finalSlash = literal.lastIndexOf("/");
    if (!literal.startsWith("/") || finalSlash <= 0) return null;
    return { source: literal.slice(1, finalSlash), flags: literal.slice(finalSlash + 1) };
  }
  if (
    ts.isNewExpression(unwrapped) &&
    ts.isIdentifier(unwrapped.expression) &&
    unwrapped.expression.text === "RegExp" &&
    unwrapped.arguments?.length
  ) {
    const [pattern, flags] = unwrapped.arguments;
    const source = pattern ? staticStringValue(pattern, sourceFile, staticStrings) : null;
    const flagValue = flags ? staticStringValue(flags, sourceFile, staticStrings) : "";
    return source === null || flagValue === null ? null : { source, flags: flagValue };
  }
  return null;
}

function staticStringValue(
  node: ts.Expression,
  sourceFile: ts.SourceFile,
  staticStrings: ReadonlyMap<string, string>,
): string | null {
  const unwrapped = unwrapExpression(node);
  if (ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)) {
    return unwrapped.text;
  }
  if (
    ts.isTaggedTemplateExpression(unwrapped) &&
    ts.isPropertyAccessExpression(unwrapped.tag) &&
    ts.isIdentifier(unwrapped.tag.expression) &&
    unwrapped.tag.expression.text === "String" &&
    unwrapped.tag.name.text === "raw" &&
    ts.isNoSubstitutionTemplateLiteral(unwrapped.template)
  ) {
    const text = unwrapped.template.getText(sourceFile);
    return text.slice(1, -1);
  }
  const key = expressionKey(unwrapped, sourceFile, staticStrings);
  if (key && staticStrings.has(key)) return staticStrings.get(key)!;
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    const left = staticStringValue(unwrapped.left, sourceFile, staticStrings);
    const right = staticStringValue(unwrapped.right, sourceFile, staticStrings);
    return left === null || right === null ? null : left + right;
  }
  return null;
}

function unwrapExpression(node: ts.Expression): ts.Expression {
  if (
    ts.isParenthesizedExpression(node) ||
    ts.isAsExpression(node) ||
    ts.isTypeAssertionExpression(node) ||
    ts.isNonNullExpression(node)
  ) {
    return unwrapExpression(node.expression);
  }
  return node;
}

function expressionKey(
  node: ts.Expression,
  sourceFile: ts.SourceFile,
  staticStrings: ReadonlyMap<string, string>,
): string | null {
  if (ts.isIdentifier(node)) return node.text;
  if (ts.isPropertyAccessExpression(node)) {
    const receiver = expressionKey(node.expression, sourceFile, staticStrings);
    return receiver ? `${receiver}.${node.name.text}` : null;
  }
  if (ts.isElementAccessExpression(node) && node.argumentExpression) {
    const receiver = expressionKey(node.expression, sourceFile, staticStrings);
    const property = staticStringValue(node.argumentExpression, sourceFile, staticStrings);
    return receiver && property !== null ? `${receiver}.${property}` : null;
  }
  return null;
}

function staticPropertyName(
  name: ts.PropertyName,
  sourceFile: ts.SourceFile,
  staticStrings: ReadonlyMap<string, string>,
): string | null {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  if (ts.isComputedPropertyName(name)) {
    return staticStringValue(name.expression, sourceFile, staticStrings);
  }
  return null;
}

function callMethod(
  expression: ts.LeftHandSideExpression,
  sourceFile: ts.SourceFile,
  staticStrings: ReadonlyMap<string, string>,
): string | null {
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  if (ts.isElementAccessExpression(expression) && expression.argumentExpression) {
    return staticStringValue(expression.argumentExpression, sourceFile, staticStrings);
  }
  return null;
}

function callReceiver(expression: ts.LeftHandSideExpression): ts.Expression | undefined {
  if (ts.isPropertyAccessExpression(expression) || ts.isElementAccessExpression(expression)) {
    return expression.expression;
  }
  return undefined;
}

function regexValueKey(value: RegexValueV0 | undefined): string {
  return value ? `${value.source}/${value.flags}` : "";
}

function sourceSyntaxesMatchedByRegex(
  value: RegexValueV0,
): readonly ("import" | "classnamesBind")[] {
  const syntaxes: ("import" | "classnamesBind")[] = [];
  if (
    regexMatchesRequiredText(
      value,
      'import styles from "./Probe.module.scss";',
      "./Probe.module.scss",
    ) ||
    regexMatchesRequiredText(
      value,
      'const styles = require("./Probe.module.scss");',
      "./Probe.module.scss",
    )
  ) {
    syntaxes.push("import");
  }
  if (
    regexMatchesRequiredText(
      value,
      "const cx = classNames.bind(styles);",
      "classNames.bind(styles)",
    )
  ) {
    syntaxes.push("classnamesBind");
  }
  return syntaxes;
}

function regexMatchesRequiredText(
  value: RegexValueV0,
  sample: string,
  requiredText: string,
): boolean {
  try {
    const match = new RegExp(value.source, value.flags).exec(sample);
    return match !== null && match.some((part) => part?.includes(requiredText));
  } catch {
    return false;
  }
}

function productSourceFrontendFiles(): readonly string[] {
  return listRepoFiles("server/engine-host-node/src")
    .filter((filePath) => filePath.endsWith(".ts"))
    .concat([
      "server/engine-core-ts/src/core/indexing/document-analysis-cache.ts",
      "server/engine-core-ts/src/core/source-frontend/rust-binding-index-projection.ts",
    ]);
}

function assertTsSourceFrontendOracleIsNotProductPath(): void {
  assert.equal(
    readRepoFileOrNull("server/engine-core-ts/src/core/binder/binder-builder.ts"),
    null,
    "TypeScript source binder builder must not remain in the product binder directory",
  );
  for (const retiredFlowPath of [
    "server/engine-core-ts/src/core/flow/class-value-analysis.ts",
    "server/engine-core-ts/src/core/flow/flow-slice.ts",
    "server/engine-core-ts/src/core/flow/cfg.ts",
  ]) {
    assert.equal(
      readRepoFileOrNull(retiredFlowPath),
      null,
      `TypeScript sparse CFG oracle must not remain in the product flow directory: ${retiredFlowPath}`,
    );
  }

  const bindingGraph = readRepoFile(
    "server/engine-core-ts/src/core/binder/source-binding-graph.ts",
  );
  assert.equal(
    bindingGraph.includes("buildSourceBindingGraph("),
    false,
    "source-binding-graph must expose graph contract/helpers, not the retired TS frontend graph builder",
  );

  const oracleOnlyFiles = new Set([
    "server/engine-core-ts/src/core/source-frontend/canonical-capture.ts",
    "server/engine-core-ts/src/core/source-frontend/ts-flow-class-value-oracle.ts",
    "server/engine-core-ts/src/core/source-frontend/ts-flow-slice-oracle.ts",
    "server/engine-core-ts/src/core/source-frontend/ts-source-binder-oracle.ts",
    "server/engine-core-ts/src/core/source-frontend/ts-source-cfg-oracle.ts",
  ]);
  const forbiddenOraclePatterns = [
    "ts-source-binder-oracle",
    "ts-flow-class-value-oracle",
    "ts-flow-slice-oracle",
    "ts-source-cfg-oracle",
    "buildSourceBinder(",
    "buildSourceBindingGraph(",
    "resolveFlowClassValues(",
    "buildFlowSlice(",
    "buildFlowNodes(",
    "buildFlowBlockGraphSnapshot(",
  ];
  const productFiles = listRepoFiles("server")
    .filter((filePath) => filePath.endsWith(".ts"))
    .filter((filePath) => !filePath.includes("/dist/"))
    .filter((filePath) => !oracleOnlyFiles.has(filePath));
  for (const filePath of productFiles) {
    const content = readRepoFile(filePath);
    for (const pattern of forbiddenOraclePatterns) {
      assert.equal(
        content.includes(pattern),
        false,
        `TS source frontend oracle must not be imported or called by product code: ${pattern} in ${filePath}`,
      );
    }
  }
}

function listRepoFiles(relativeDir: string): readonly string[] {
  const absoluteDir = path.join(repoRoot, relativeDir);
  return readdirSync(absoluteDir).flatMap((entry) => {
    const relativePath = path.join(relativeDir, entry);
    const absolutePath = path.join(repoRoot, relativePath);
    const stats = statSync(absolutePath);
    if (stats.isDirectory()) return listRepoFiles(relativePath);
    return stats.isFile() ? [relativePath] : [];
  });
}

function readRepoFile(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readRepoFileOrNull(relativePath: string): string | null {
  try {
    return readRepoFile(relativePath);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
}
