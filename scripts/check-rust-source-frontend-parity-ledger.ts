import assert from "node:assert/strict";
import { readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
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

const removeScopedSiblingInjection = installScopedSiblingInjection();
let importSyntaxProducerSites = -1;
let productSourceFrontendFileCount = -1;
try {
  productSourceFrontendFileCount = productSourceFrontendFiles().length;
  assertNoProductSourceFrontendFallbacks();
  importSyntaxProducerSites = assertNoImportSyntaxRegexProducers();
  assertTsSourceFrontendOracleIsNotProductPath();
} finally {
  removeScopedSiblingInjection();
}

console.log(
  JSON.stringify(
    {
      product: "omena.source-frontend-parity-ledger.check",
      entryGates: ledger.entryGates.length,
      sourceFrontendDefault: resolveSourceFrontendBackendKind({}),
      productSourceFrontendScope: "provider-import-dependencies-and-consumers",
      productSourceFrontendFileCount,
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
    "sourceFileCache",
    "entry.sourceFile.text",
    "ctx.entry.sourceFile.text",
    "cached?.sourceFile.fileName",
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
  const productFiles = productSourceFrontendFiles();
  const sources = new Map(productFiles.map((filePath) => [filePath, readRepoFile(filePath)]));
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
  if (process.argv.includes("--inject-import-regex-replacement-template")) {
    sources.set(
      "__injection__/source-import-regex-replacement-template.ts",
      'const source = String.raw`^\\s*impXrt\\s+(.+?)\\s+from\\s+[\'\"](.+?)[\'\"]`.replace("X", "o");\nconst pattern = new RegExp(source, "gm");\nexport function injected(input: string) { return pattern.exec(input); }\n',
    );
  }
  if (process.argv.includes("--inject-import-regex-aliased-string-raw")) {
    sources.set(
      "__injection__/source-import-regex-aliased-string-raw.ts",
      'const raw = String.raw;\nconst source = raw`^\\s*import\\s+(.+?)\\s+from\\s+[\'\"](.+?)[\'\"]`;\nconst pattern = new RegExp(source, "gm");\nexport function injected(input: string) { return pattern.test(input); }\n',
    );
  }
  if (process.argv.includes("--inject-import-regex-array-join")) {
    sources.set(
      "__injection__/source-import-regex-array-join.ts",
      'const source = ["^", "\\\\s*", "import", "\\\\s+(.+?)\\\\s+from\\\\s+[\'\\\"](.+?)[\'\\\"]"].join("");\nconst pattern = new RegExp(source, "gm");\nexport function injected(input: string) { return input.match(pattern); }\n',
    );
  }
  if (process.argv.includes("--inject-import-regex-helper-return")) {
    sources.set(
      "__injection__/source-import-regex-helper-return.ts",
      'function importPatternSource() { return String.raw`^\\s*import\\s+(.+?)\\s+from\\s+[\'\"](.+?)[\'\"]`; }\nconst pattern = new RegExp(importPatternSource(), "gm");\nexport function injected(input: string) { return input.search(pattern); }\n',
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
    const injectedPath = scopedSiblingInjectionPath();
    assert.ok(
      productFiles.includes(injectedPath),
      `import-graph scope did not include the real sibling probe ${injectedPath}`,
    );
  }
  appendScopedProducer(
    sources,
    productFiles,
    "--inject-source-text-offsets-import-regex-producer",
    "server/engine-core-ts/src/core/source-frontend/source-text-offsets.ts",
  );
  appendScopedProducer(
    sources,
    productFiles,
    "--inject-lsp-import-regex-producer",
    "server/lsp-server/src/providers/completion.ts",
  );

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

function scopedSiblingInjectionPath(): string {
  return "server/engine-core-ts/src/core/source-frontend/__source_frontend_scope_probe__.ts";
}

function installScopedSiblingInjection(): () => void {
  if (!process.argv.includes("--inject-sibling-import-regex-producer")) return () => {};
  const relativePath = scopedSiblingInjectionPath();
  const absolutePath = path.join(repoRoot, relativePath);
  assert.equal(
    readRepoFileOrNull(relativePath),
    null,
    `scope probe path must be absent before injection: ${relativePath}`,
  );
  const providerPath = "server/engine-host-node/src/source-frontend-analysis-provider.ts";
  let providerSpecifier = path
    .relative(path.dirname(relativePath), providerPath.slice(0, -3))
    .split(path.sep)
    .join("/");
  if (!providerSpecifier.startsWith(".")) providerSpecifier = `./${providerSpecifier}`;
  writeFileSync(
    absolutePath,
    `import ${JSON.stringify(providerSpecifier)};\n${scopedImportRegexProducerSource()}`,
  );
  return () => rmSync(absolutePath, { force: true });
}

function appendScopedProducer(
  sources: Map<string, string>,
  productFiles: readonly string[],
  flag: string,
  relativePath: string,
): void {
  if (!process.argv.includes(flag)) return;
  assert.ok(
    productFiles.includes(relativePath),
    `import-graph scope did not include the real producer probe ${relativePath}`,
  );
  const source = sources.get(relativePath);
  assert.ok(source !== undefined, `scoped producer source is missing: ${relativePath}`);
  sources.set(relativePath, `${source}\n${scopedImportRegexProducerSource()}`);
}

function scopedImportRegexProducerSource(): string {
  return String.raw`const scopedImportPattern = /^\s*import\s+(.+?)\s+from\s+['"](.+?)['"]/gm;
export function scopedImportProducer(source: string) { return source.search(scopedImportPattern); }
`;
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
  const staticRawTags = new Set<string>();
  const staticFunctions = new Map<string, string>();
  const regexValues = new Map<string, RegexValueV0>();
  const declarations: ts.VariableDeclaration[] = [];
  const functionDeclarations: ts.Node[] = [];

  const collect = (node: ts.Node): void => {
    if (ts.isVariableDeclaration(node)) {
      declarations.push(node);
    } else if (ts.isFunctionDeclaration(node) && node.name) {
      functionDeclarations.push(node);
    }
    ts.forEachChild(node, collect);
  };
  collect(sourceFile);

  const staticEnvironment: StaticStringEnvironmentV0 = {
    sourceFile,
    strings: staticStrings,
    rawTags: staticRawTags,
    functions: staticFunctions,
  };

  let changed = true;
  while (changed) {
    changed = false;
    for (const declaration of declarations) {
      if (!ts.isIdentifier(declaration.name) || !declaration.initializer) continue;
      const name = declaration.name.text;
      changed = bindStaticValue(name, declaration.initializer) || changed;
      if (isStaticStringRawTag(declaration.initializer, staticEnvironment)) {
        const previousSize = staticRawTags.size;
        staticRawTags.add(name);
        changed = staticRawTags.size !== previousSize || changed;
      }
      const returnExpression = staticFunctionReturnExpression(declaration.initializer);
      if (returnExpression) {
        changed = bindStaticFunction(name, returnExpression) || changed;
      }
      if (ts.isObjectLiteralExpression(declaration.initializer)) {
        for (const property of declaration.initializer.properties) {
          if (!ts.isPropertyAssignment(property)) continue;
          const propertyName = staticPropertyName(property.name, staticEnvironment);
          if (propertyName === null) continue;
          changed = bindStaticValue(`${name}.${propertyName}`, property.initializer) || changed;
        }
      }
    }
    for (const declaration of functionDeclarations) {
      const returnExpression = staticFunctionReturnExpression(declaration);
      if (returnExpression && declaration.name) {
        changed = bindStaticFunction(declaration.name.text, returnExpression) || changed;
      }
    }
  }

  function bindStaticValue(name: string, expression: ts.Expression): boolean {
    let bound = false;
    const staticValue = staticStringValue(expression, staticEnvironment);
    if (staticValue !== null && staticStrings.get(name) !== staticValue) {
      staticStrings.set(name, staticValue);
      bound = true;
    }
    const regexValue = regexValueForExpression(expression, staticEnvironment, regexValues);
    if (regexValue && regexValueKey(regexValues.get(name)) !== regexValueKey(regexValue)) {
      regexValues.set(name, regexValue);
      bound = true;
    }
    return bound;
  }

  function bindStaticFunction(name: string, returnExpression: ts.Expression): boolean {
    const value = staticStringValue(returnExpression, staticEnvironment);
    if (value === null || staticFunctions.get(name) === value) return false;
    staticFunctions.set(name, value);
    return true;
  }

  const producerSyntaxes = (node: ts.Expression | undefined) => {
    if (!node) return [];
    const regexValue = regexValueForExpression(node, staticEnvironment, regexValues);
    return regexValue ? sourceSyntaxesMatchedByRegex(regexValue) : [];
  };

  const sites: ImportSyntaxRegexProducerSiteV0[] = [];
  const inspect = (node: ts.Node): void => {
    if (ts.isCallExpression(node)) {
      const method = callMethod(node.expression, staticEnvironment);
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

interface StaticStringEnvironmentV0 {
  readonly sourceFile: ts.SourceFile;
  readonly strings: ReadonlyMap<string, string>;
  readonly rawTags: ReadonlySet<string>;
  readonly functions: ReadonlyMap<string, string>;
}

function regexValueForExpression(
  node: ts.Expression,
  environment: StaticStringEnvironmentV0,
  regexValues: ReadonlyMap<string, RegexValueV0>,
): RegexValueV0 | null {
  const unwrapped = unwrapExpression(node);
  const key = expressionKey(unwrapped, environment);
  if (key && regexValues.has(key)) return regexValues.get(key)!;
  if (ts.isRegularExpressionLiteral(unwrapped)) {
    const literal = unwrapped.getText(environment.sourceFile);
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
    const source = pattern ? staticStringValue(pattern, environment) : null;
    const flagValue = flags ? staticStringValue(flags, environment) : "";
    return source === null || flagValue === null ? null : { source, flags: flagValue };
  }
  return null;
}

function staticStringValue(
  node: ts.Expression,
  environment: StaticStringEnvironmentV0,
): string | null {
  const unwrapped = unwrapExpression(node);
  if (ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)) {
    return unwrapped.text;
  }
  if (
    ts.isTaggedTemplateExpression(unwrapped) &&
    isStaticStringRawTag(unwrapped.tag, environment) &&
    ts.isNoSubstitutionTemplateLiteral(unwrapped.template)
  ) {
    const text = unwrapped.template.getText(environment.sourceFile);
    return text.slice(1, -1);
  }
  if (ts.isTemplateExpression(unwrapped)) {
    let value = unwrapped.head.text;
    for (const span of unwrapped.templateSpans) {
      const expression = staticStringValue(span.expression, environment);
      if (expression === null) return null;
      value += expression + span.literal.text;
    }
    return value;
  }
  const key = expressionKey(unwrapped, environment);
  if (key && environment.strings.has(key)) return environment.strings.get(key)!;
  if (
    ts.isBinaryExpression(unwrapped) &&
    unwrapped.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    const left = staticStringValue(unwrapped.left, environment);
    const right = staticStringValue(unwrapped.right, environment);
    return left === null || right === null ? null : left + right;
  }
  if (ts.isCallExpression(unwrapped)) {
    const callKey = expressionKey(unwrapped.expression, environment);
    if (callKey && unwrapped.arguments.length === 0 && environment.functions.has(callKey)) {
      return environment.functions.get(callKey)!;
    }
    const method = callMethod(unwrapped.expression, environment);
    const receiver = callReceiver(unwrapped.expression);
    if (method === "join" && receiver && ts.isArrayLiteralExpression(receiver)) {
      const separator = unwrapped.arguments[0]
        ? staticStringValue(unwrapped.arguments[0], environment)
        : ",";
      if (separator === null) return null;
      const values = receiver.elements.map((element) =>
        ts.isSpreadElement(element) ? null : staticStringValue(element, environment),
      );
      return values.some((value) => value === null) ? null : (values as string[]).join(separator);
    }
    if ((method === "replace" || method === "replaceAll") && receiver) {
      const receiverValue = staticStringValue(receiver, environment);
      const searchValue = unwrapped.arguments[0]
        ? staticStringValue(unwrapped.arguments[0], environment)
        : null;
      const replacementValue = unwrapped.arguments[1]
        ? staticStringValue(unwrapped.arguments[1], environment)
        : null;
      if (receiverValue === null || searchValue === null || replacementValue === null) return null;
      if (method === "replaceAll") return receiverValue.split(searchValue).join(replacementValue);
      const index = receiverValue.indexOf(searchValue);
      return index < 0
        ? receiverValue
        : `${receiverValue.slice(0, index)}${replacementValue}${receiverValue.slice(index + searchValue.length)}`;
    }
  }
  return null;
}

function isStaticStringRawTag(
  node: ts.Expression,
  environment: StaticStringEnvironmentV0,
): boolean {
  const unwrapped = unwrapExpression(node);
  if (
    ts.isPropertyAccessExpression(unwrapped) &&
    ts.isIdentifier(unwrapped.expression) &&
    unwrapped.expression.text === "String" &&
    unwrapped.name.text === "raw"
  ) {
    return true;
  }
  const key = expressionKey(unwrapped, environment);
  return key !== null && environment.rawTags.has(key);
}

function staticFunctionReturnExpression(node: ts.Node): ts.Expression | null {
  if (ts.isArrowFunction(node) && !ts.isBlock(node.body)) return node.body;
  if (
    !ts.isFunctionDeclaration(node) &&
    !ts.isFunctionExpression(node) &&
    !ts.isArrowFunction(node)
  ) {
    return null;
  }
  const body = node.body;
  if (!body || !ts.isBlock(body)) return null;
  const returnStatements = body.statements.filter(ts.isReturnStatement);
  return returnStatements.length === 1 ? (returnStatements[0]?.expression ?? null) : null;
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

function expressionKey(node: ts.Expression, environment: StaticStringEnvironmentV0): string | null {
  if (ts.isIdentifier(node)) return node.text;
  if (ts.isPropertyAccessExpression(node)) {
    const receiver = expressionKey(node.expression, environment);
    return receiver ? `${receiver}.${node.name.text}` : null;
  }
  if (ts.isElementAccessExpression(node) && node.argumentExpression) {
    const receiver = expressionKey(node.expression, environment);
    const property = staticStringValue(node.argumentExpression, environment);
    return receiver && property !== null ? `${receiver}.${property}` : null;
  }
  return null;
}

function staticPropertyName(
  name: ts.PropertyName,
  environment: StaticStringEnvironmentV0,
): string | null {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
    return name.text;
  }
  if (ts.isComputedPropertyName(name)) {
    return staticStringValue(name.expression, environment);
  }
  return null;
}

function callMethod(
  expression: ts.LeftHandSideExpression,
  environment: StaticStringEnvironmentV0,
): string | null {
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  if (ts.isElementAccessExpression(expression) && expression.argumentExpression) {
    return staticStringValue(expression.argumentExpression, environment);
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
    regexCapturesRequiredSyntax(value, 'import styles from "./Probe.module.scss";', "import", [
      "styles",
      "./Probe.module.scss",
    ]) ||
    regexCapturesRequiredSyntax(value, "import styles from './Probe.module.scss';", "import", [
      "styles",
      "./Probe.module.scss",
    ]) ||
    regexCapturesRequiredSyntax(
      value,
      'const styles = require("./Probe.module.scss");',
      "require",
      ["styles", "./Probe.module.scss"],
    )
  ) {
    syntaxes.push("import");
  }
  if (
    regexCapturesRequiredSyntax(value, "const cx = classNames.bind(styles);", "classNames.bind", [
      "cx",
      "styles",
    ])
  ) {
    syntaxes.push("classnamesBind");
  }
  return syntaxes;
}

function regexCapturesRequiredSyntax(
  value: RegexValueV0,
  sample: string,
  requiredMatchText: string,
  requiredCaptures: readonly string[],
): boolean {
  try {
    const match = new RegExp(value.source, value.flags).exec(sample);
    return (
      match !== null &&
      match[0]?.includes(requiredMatchText) === true &&
      requiredCaptures.every((required) =>
        match.slice(1).some((capture) => capture?.includes(required)),
      )
    );
  } catch {
    return false;
  }
}

function productSourceFrontendFiles(): readonly string[] {
  const providerPath = "server/engine-host-node/src/source-frontend-analysis-provider.ts";
  const serverFiles = listRepoFiles("server").filter(isTypeScriptProductSource).toSorted();
  const serverFileSet = new Set(serverFiles);
  assert.ok(
    serverFileSet.has(providerPath),
    `source frontend provider is missing: ${providerPath}`,
  );

  const dependencies = new Map(serverFiles.map((filePath) => [filePath, new Set<string>()]));
  const consumers = new Map(serverFiles.map((filePath) => [filePath, new Set<string>()]));
  for (const filePath of serverFiles) {
    const source = readRepoFile(filePath);
    for (const specifier of relativeModuleSpecifiers(filePath, source)) {
      const dependency = resolveTypeScriptModule(filePath, specifier, serverFileSet);
      if (!dependency) continue;
      dependencies.get(filePath)?.add(dependency);
      consumers.get(dependency)?.add(filePath);
    }
  }

  const scope = new Set([
    ...transitiveGraphClosure(providerPath, dependencies),
    ...transitiveGraphClosure(providerPath, consumers),
  ]);
  for (const requiredPath of [
    "server/engine-core-ts/src/core/source-frontend/source-text-offsets.ts",
    "server/lsp-server/src/providers/completion.ts",
  ]) {
    assert.ok(scope.has(requiredPath), `provider import-graph scope must include ${requiredPath}`);
  }
  return [...scope].toSorted();
}

function isTypeScriptProductSource(filePath: string): boolean {
  return (
    /\.(?:c|m)?tsx?$/u.test(filePath) &&
    !filePath.endsWith(".d.ts") &&
    !filePath.includes("/dist/") &&
    !/(?:^|\/)(?:test|tests|__tests__)(?:\/|$)/u.test(filePath) &&
    !/\.(?:spec|test)\.(?:c|m)?tsx?$/u.test(filePath)
  );
}

function relativeModuleSpecifiers(filePath: string, source: string): readonly string[] {
  const sourceFile = ts.createSourceFile(
    filePath,
    source,
    ts.ScriptTarget.Latest,
    true,
    filePath.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const specifiers = new Set<string>();
  const add = (value: ts.Expression | undefined): void => {
    if (value && ts.isStringLiteral(value) && value.text.startsWith(".")) {
      specifiers.add(value.text);
    }
  };
  const visit = (node: ts.Node): void => {
    if (ts.isImportDeclaration(node)) {
      add(node.moduleSpecifier);
    } else if (node.kind === ts.SyntaxKind.ExportDeclaration) {
      add((node as { readonly moduleSpecifier?: ts.Expression }).moduleSpecifier);
    } else if (
      node.kind === ts.SyntaxKind.ImportEqualsDeclaration &&
      (node as { readonly moduleReference: ts.Node }).moduleReference.kind ===
        ts.SyntaxKind.ExternalModuleReference
    ) {
      add(
        (
          (node as { readonly moduleReference: ts.Node }).moduleReference as ts.Node & {
            readonly expression?: ts.Expression;
          }
        ).expression,
      );
    } else if (
      ts.isCallExpression(node) &&
      (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
        (ts.isIdentifier(node.expression) && node.expression.text === "require"))
    ) {
      add(node.arguments[0]);
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return [...specifiers];
}

function resolveTypeScriptModule(
  importerPath: string,
  specifier: string,
  serverFiles: ReadonlySet<string>,
): string | null {
  const base = path.normalize(path.join(path.dirname(importerPath), specifier));
  const withoutRuntimeExtension = base.replace(/\.(?:c|m)?jsx?$/u, "");
  const candidates = [base, withoutRuntimeExtension];
  for (const stem of [...candidates]) {
    for (const extension of [".ts", ".tsx", ".mts", ".cts"]) {
      candidates.push(`${stem}${extension}`);
      candidates.push(path.join(stem, `index${extension}`));
    }
  }
  return candidates.find((candidate) => serverFiles.has(candidate)) ?? null;
}

function transitiveGraphClosure(
  root: string,
  edges: ReadonlyMap<string, ReadonlySet<string>>,
): ReadonlySet<string> {
  const visited = new Set([root]);
  const pending = [root];
  while (pending.length > 0) {
    const current = pending.shift();
    if (!current) continue;
    for (const adjacent of edges.get(current) ?? []) {
      if (visited.has(adjacent)) continue;
      visited.add(adjacent);
      pending.push(adjacent);
    }
  }
  return visited;
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
