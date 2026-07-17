import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parse } from "parse5";
import type { DefaultTreeAdapterTypes } from "parse5";
import ts from "typescript";

type StaticValue = string | readonly string[] | Readonly<Record<string, string>>;

interface TierZeroModuleV0 {
  readonly moduleId: string;
  readonly wptPath: string;
}

interface TierZeroTupleV0 {
  readonly id: string;
  readonly moduleId: string;
  readonly wptPath: string;
  readonly wptSourceLine: number;
  readonly subtest: string;
  readonly sourceTextSha256: string;
  readonly helperClass: string;
  readonly helperCall: string;
  readonly subject: "property" | "selector" | "rule";
  readonly expectedValidity: "valid" | "invalid";
  readonly property: string;
  readonly value: string;
  readonly expectedValues: readonly string[];
  readonly specLinks: readonly string[];
}

interface SkippedCallV0 {
  readonly wptPath: string;
  readonly wptSourceLine: number;
  readonly helperCall: string;
  readonly reason: string;
}

interface ModuleCoverageV0 {
  readonly moduleId: string;
  readonly wptPath: string;
  readonly htmlFileCount: number;
  readonly eligibleTierZeroFileCount: number;
  readonly nonTierZeroFileCount: number;
  readonly excludedTentativeFileCount: number;
  readonly excludedOptionalFileCount: number;
  readonly extractedSubtestCount: number;
  readonly skippedDynamicCallCount: number;
  readonly skippedDynamicReasons: Readonly<Record<string, number>>;
}

interface ExtractionResultV0 {
  readonly tuples: readonly TierZeroTupleV0[];
  readonly coverage: readonly ModuleCoverageV0[];
  readonly skippedCalls: readonly SkippedCallV0[];
}

interface ScriptFragmentV0 {
  readonly source: string;
  readonly startLine: number;
}

const modules: readonly TierZeroModuleV0[] = [
  { moduleId: "selectors", wptPath: "css/selectors" },
  { moduleId: "css-values", wptPath: "css/css-values" },
  { moduleId: "css-color", wptPath: "css/css-color" },
  { moduleId: "css-typed-om", wptPath: "css/css-typed-om" },
  { moduleId: "css-variables", wptPath: "css/css-variables" },
  { moduleId: "cssom", wptPath: "css/cssom" },
  { moduleId: "css-cascade", wptPath: "css/css-cascade" },
];

const helperImports = new Map([
  ["parsing-testcommon.js", "parsing"],
  ["serialize-testcommon.js", "serialization"],
  ["numeric-testcommon.js", "numeric"],
  ["color-testcommon.js", "color"],
]);

const helperCallClasses = new Map([
  ["test_valid_value", "parsing"],
  ["test_invalid_value", "parsing"],
  ["test_valid_selector", "parsing"],
  ["test_invalid_selector", "parsing"],
  ["test_valid_rule", "parsing"],
  ["test_invalid_rule", "parsing"],
  ["test_specified_serialization", "serialization"],
  ["test_serialization", "serialization"],
  ["test_math_specified", "numeric"],
  ["fuzzy_test_valid_color", "color"],
  ["fuzzy_test_valid_color_property", "color"],
]);

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const extractedRoot = path.join(corpusRoot, "extracted");
const tuplesPath = path.join(extractedRoot, "tier-zero-tuples.json");
const coveragePath = path.join(extractedRoot, "tier-zero-coverage.json");
const sourcePin = "web-platform-tests/wpt@27d2ee72d025342dd435074bc5f9b454d9d7191b";

const wptRoot = readArgument("--wpt-root");
const write = process.argv.includes("--write");
const check = process.argv.includes("--check");
const selfTest = process.argv.includes("--self-test");

assert.ok(!(write && check), "choose either --write or --check");

if (selfTest) {
  runSelfTest();
  process.stdout.write(
    `${JSON.stringify({ schemaVersion: "0", product: "rust.omena-wpt-tier-zero-extractor-self-test", passed: true }, null, 2)}\n`,
  );
} else {
  assert.ok(wptRoot, "--wpt-root is required outside --self-test mode");
  assert.ok(write || check, "choose --write or --check");
  const actualPin = readWptPin(wptRoot);
  assert.equal(actualPin, sourcePin, "WPT checkout must match the committed extraction pin");
  const result = extractCorpus(wptRoot);
  const tupleArtifact = {
    schemaVersion: "0",
    product: "omena-diff-test.wpt-tier-zero-tuples",
    source: {
      repository: "https://github.com/web-platform-tests/wpt",
      pin: sourcePin,
      extractionMode: "static-helper-call-sites",
      testharnessExecuted: false,
    },
    modules,
    tuples: result.tuples,
  } as const;
  const coverageArtifact = {
    schemaVersion: "0",
    product: "omena-diff-test.wpt-tier-zero-coverage",
    sourcePin,
    moduleCount: modules.length,
    extractedSubtestCount: result.tuples.length,
    skippedDynamicCallCount: result.skippedCalls.length,
    modules: result.coverage,
    skippedDynamicCalls: result.skippedCalls,
  } as const;
  const tupleSource = stableJson(tupleArtifact);
  const coverageSource = stableJson(coverageArtifact);

  if (write) {
    mkdirSync(extractedRoot, { recursive: true });
    writeFileSync(tuplesPath, tupleSource);
    writeFileSync(coveragePath, coverageSource);
  } else {
    assert.equal(readFileSync(tuplesPath, "utf8"), tupleSource, "tier-zero tuples are stale");
    assert.equal(readFileSync(coveragePath, "utf8"), coverageSource, "tier-zero coverage is stale");
  }

  process.stdout.write(
    stableJson({
      schemaVersion: "0",
      product: "rust.omena-wpt-tier-zero-extractor",
      mode: write ? "write" : "check",
      sourcePin,
      tupleCount: result.tuples.length,
      skippedDynamicCallCount: result.skippedCalls.length,
      moduleCoverage: result.coverage,
      tupleSha256: sha256(tupleSource),
      coverageSha256: sha256(coverageSource),
    }),
  );
}

function extractCorpus(root: string): ExtractionResultV0 {
  const tuples: TierZeroTupleV0[] = [];
  const coverage: ModuleCoverageV0[] = [];
  const skippedCalls: SkippedCallV0[] = [];

  for (const module of modules) {
    const moduleRoot = path.join(root, module.wptPath);
    const htmlFiles = collectFiles(moduleRoot).filter((filePath) => /\.html?$/u.test(filePath));
    let eligibleTierZeroFileCount = 0;
    let nonTierZeroFileCount = 0;
    let excludedTentativeFileCount = 0;
    let excludedOptionalFileCount = 0;
    const moduleTuples: TierZeroTupleV0[] = [];
    const moduleSkipped: SkippedCallV0[] = [];

    for (const filePath of htmlFiles) {
      const wptPath = slash(path.relative(root, filePath));
      if (/\.tentative\./u.test(filePath)) {
        excludedTentativeFileCount += 1;
        continue;
      }
      if (/\.optional\./u.test(filePath)) {
        excludedOptionalFileCount += 1;
        continue;
      }
      const source = readFileSync(filePath, "utf8");
      const document = parse(source, { sourceCodeLocationInfo: true });
      const documentFacts = readDocumentFacts(document);
      if (documentFacts.helperClasses.size === 0) {
        nonTierZeroFileCount += 1;
        continue;
      }
      eligibleTierZeroFileCount += 1;
      for (const fragment of documentFacts.scriptFragments) {
        const extracted = extractScriptCalls({
          module,
          wptPath,
          fragment,
          helperClasses: documentFacts.helperClasses,
          specLinks: documentFacts.specLinks,
        });
        moduleTuples.push(...extracted.tuples);
        moduleSkipped.push(...extracted.skippedCalls);
      }
    }

    moduleTuples.sort(compareTuple);
    moduleSkipped.sort(compareSkippedCall);
    tuples.push(...moduleTuples);
    skippedCalls.push(...moduleSkipped);
    const reasonCounts = new Map<string, number>();
    for (const skipped of moduleSkipped) {
      reasonCounts.set(skipped.reason, (reasonCounts.get(skipped.reason) ?? 0) + 1);
    }
    coverage.push({
      moduleId: module.moduleId,
      wptPath: module.wptPath,
      htmlFileCount: htmlFiles.length,
      eligibleTierZeroFileCount,
      nonTierZeroFileCount,
      excludedTentativeFileCount,
      excludedOptionalFileCount,
      extractedSubtestCount: moduleTuples.length,
      skippedDynamicCallCount: moduleSkipped.length,
      skippedDynamicReasons: Object.fromEntries(
        [...reasonCounts].sort(([left], [right]) => left.localeCompare(right)),
      ),
    });
  }

  tuples.sort(compareTuple);
  skippedCalls.sort(compareSkippedCall);
  assert.equal(
    new Set(tuples.map((tuple) => tuple.id)).size,
    tuples.length,
    "tuple ids must be unique",
  );
  return { tuples, coverage, skippedCalls };
}

function readDocumentFacts(document: DefaultTreeAdapterTypes.Document): {
  readonly helperClasses: ReadonlySet<string>;
  readonly scriptFragments: readonly ScriptFragmentV0[];
  readonly specLinks: readonly string[];
} {
  const helperClasses = new Set<string>();
  const scriptFragments: ScriptFragmentV0[] = [];
  const specLinks = new Set<string>();

  visitHtmlNode(document, (node) => {
    if (!("tagName" in node)) return;
    const attributes = new Map(node.attrs.map((attribute) => [attribute.name, attribute.value]));
    if (node.tagName === "link") {
      const relations = (attributes.get("rel") ?? "").split(/\s+/u);
      const href = attributes.get("href");
      if (relations.includes("help") && href) specLinks.add(href);
      return;
    }
    if (node.tagName !== "script") return;
    const src = attributes.get("src");
    if (src) {
      const helperClass = helperClassForImport(src);
      if (helperClass) helperClasses.add(helperClass);
      return;
    }
    for (const child of node.childNodes) {
      if (child.nodeName !== "#text" || child.value.trim() === "") continue;
      scriptFragments.push({
        source: child.value,
        startLine: child.sourceCodeLocation?.startLine ?? 1,
      });
    }
  });

  return {
    helperClasses,
    scriptFragments,
    specLinks: [...specLinks].sort(),
  };
}

function extractScriptCalls(input: {
  readonly module: TierZeroModuleV0;
  readonly wptPath: string;
  readonly fragment: ScriptFragmentV0;
  readonly helperClasses: ReadonlySet<string>;
  readonly specLinks: readonly string[];
}): { readonly tuples: TierZeroTupleV0[]; readonly skippedCalls: SkippedCallV0[] } {
  const sourceFile = ts.createSourceFile(
    input.wptPath,
    input.fragment.source,
    ts.ScriptTarget.ESNext,
    true,
    ts.ScriptKind.JS,
  );
  const environment = new Map<string, StaticValue>();
  const tuples: TierZeroTupleV0[] = [];
  const skippedCalls: SkippedCallV0[] = [];

  function visit(node: ts.Node, insideFunction: boolean): void {
    if (
      !insideFunction &&
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.initializer
    ) {
      const value = staticValue(node.initializer, environment);
      if (value !== undefined) environment.set(node.name.text, value);
    }
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const helperCall = node.expression.text;
      const helperClass = helperCallClasses.get(helperCall);
      if (helperClass && input.helperClasses.has(helperClass)) {
        const location = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
        const wptSourceLine = input.fragment.startLine + location.line;
        const subtest = node.getText(sourceFile).trim();
        if (insideFunction) {
          skippedCalls.push({
            wptPath: input.wptPath,
            wptSourceLine,
            helperCall,
            reason: "dynamic-function-context",
          });
          return;
        }
        const extracted = tupleFromCall(
          helperCall,
          node.arguments,
          environment,
          input.module,
          input.wptPath,
          wptSourceLine,
          subtest,
          helperClass,
          input.specLinks,
        );
        if (typeof extracted === "string") {
          skippedCalls.push({
            wptPath: input.wptPath,
            wptSourceLine,
            helperCall,
            reason: extracted,
          });
        } else {
          tuples.push(extracted);
        }
      }
    }
    const childInsideFunction = insideFunction || ts.isFunctionLike(node);
    ts.forEachChild(node, (child) => visit(child, childInsideFunction));
  }

  visit(sourceFile, false);
  return { tuples, skippedCalls };
}

function tupleFromCall(
  helperCall: string,
  args: ts.NodeArray<ts.Expression>,
  environment: ReadonlyMap<string, StaticValue>,
  module: TierZeroModuleV0,
  wptPath: string,
  wptSourceLine: number,
  subtest: string,
  helperClass: string,
  specLinks: readonly string[],
): TierZeroTupleV0 | string {
  let subject: TierZeroTupleV0["subject"] = "property";
  let expectedValidity: TierZeroTupleV0["expectedValidity"] = "valid";
  let property: string | undefined;
  let value: string | undefined;
  let expectedValues: readonly string[] | undefined;

  if (helperCall === "test_valid_value" || helperCall === "test_invalid_value") {
    property = staticString(args[0], environment);
    value = staticString(args[1], environment);
    expectedValidity = helperCall === "test_valid_value" ? "valid" : "invalid";
    expectedValues =
      expectedValidity === "valid" ? staticStringSet(args[2], environment, value) : [];
  } else if (helperCall === "test_valid_selector" || helperCall === "test_invalid_selector") {
    subject = "selector";
    property = "@selector";
    value = staticString(args[0], environment);
    expectedValidity = helperCall === "test_valid_selector" ? "valid" : "invalid";
    expectedValues =
      expectedValidity === "valid" ? staticStringSet(args[1], environment, value) : [];
  } else if (helperCall === "test_valid_rule" || helperCall === "test_invalid_rule") {
    subject = "rule";
    property = "@rule";
    value = staticString(args[0], environment);
    expectedValidity = helperCall === "test_valid_rule" ? "valid" : "invalid";
    expectedValues =
      expectedValidity === "valid" ? staticStringSet(args[1], environment, value) : [];
  } else if (helperCall === "test_specified_serialization") {
    property = staticString(args[0], environment);
    value = staticString(args[1], environment);
    expectedValues = staticStringSet(args[2], environment);
  } else if (helperCall === "test_serialization") {
    value = staticString(args[0], environment);
    expectedValues = staticStringSet(args[1], environment);
    property = staticObject(args[4], environment)?.prop;
    if (!property) return "missing-static-property";
  } else if (helperCall === "test_math_specified") {
    value = staticString(args[0], environment);
    expectedValues = staticStringSet(args[1], environment);
    const options = staticObject(args[2], environment) ?? {};
    property = options.prop ?? numericPropertyForType(options.type ?? "length");
  } else if (helperCall === "fuzzy_test_valid_color_property") {
    property = staticString(args[0], environment);
    value = staticString(args[1], environment);
    expectedValues = staticStringSet(args[2], environment);
  } else if (helperCall === "fuzzy_test_valid_color") {
    property = "color";
    value = staticString(args[0], environment);
    expectedValues = staticStringSet(args[1], environment);
  }

  if (!property) return "dynamic-property";
  if (!value) return "dynamic-value";
  if (!expectedValues) return "dynamic-expected-serialization";
  if (expectedValidity === "valid" && expectedValues.length === 0) {
    return "empty-expected-serialization-set";
  }
  const normalizedExpectedValues = [...new Set(expectedValues)];
  const callHash = sha256(subtest);
  return {
    id: `${module.moduleId}:${slash(path.relative(module.wptPath, wptPath))}:${wptSourceLine}:${callHash.slice(0, 12)}`,
    moduleId: module.moduleId,
    wptPath,
    wptSourceLine,
    subtest,
    sourceTextSha256: callHash,
    helperClass,
    helperCall,
    subject,
    expectedValidity,
    property,
    value,
    expectedValues: normalizedExpectedValues,
    specLinks,
  };
}

function staticValue(
  expression: ts.Expression | undefined,
  environment: ReadonlyMap<string, StaticValue>,
): StaticValue | undefined {
  if (!expression) return undefined;
  if (ts.isStringLiteralLike(expression)) return expression.text;
  if (ts.isIdentifier(expression)) return environment.get(expression.text);
  if (ts.isParenthesizedExpression(expression))
    return staticValue(expression.expression, environment);
  if (
    ts.isBinaryExpression(expression) &&
    expression.operatorToken.kind === ts.SyntaxKind.PlusToken
  ) {
    const left = staticValue(expression.left, environment);
    const right = staticValue(expression.right, environment);
    return typeof left === "string" && typeof right === "string" ? left + right : undefined;
  }
  if (ts.isArrayLiteralExpression(expression)) {
    const values = expression.elements.map((element) => staticValue(element, environment));
    return values.every((value): value is string => typeof value === "string") ? values : undefined;
  }
  if (ts.isObjectLiteralExpression(expression)) {
    const entries: [string, string][] = [];
    for (const property of expression.properties) {
      if (!ts.isPropertyAssignment(property)) return undefined;
      const name = propertyName(property.name);
      const value = staticValue(property.initializer, environment);
      if (!name || typeof value !== "string") return undefined;
      entries.push([name, value]);
    }
    return Object.fromEntries(entries);
  }
  return undefined;
}

function staticString(
  expression: ts.Expression | undefined,
  environment: ReadonlyMap<string, StaticValue>,
): string | undefined {
  const value = staticValue(expression, environment);
  return typeof value === "string" ? value : undefined;
}

function staticStringSet(
  expression: ts.Expression | undefined,
  environment: ReadonlyMap<string, StaticValue>,
  defaultValue?: string,
): readonly string[] | undefined {
  if (!expression) return defaultValue === undefined ? undefined : [defaultValue];
  const value = staticValue(expression, environment);
  if (typeof value === "string") return [value];
  return Array.isArray(value) && value.every((entry) => typeof entry === "string")
    ? value
    : undefined;
}

function staticObject(
  expression: ts.Expression | undefined,
  environment: ReadonlyMap<string, StaticValue>,
): Readonly<Record<string, string>> | undefined {
  const value = staticValue(expression, environment);
  return value !== undefined && !Array.isArray(value) && typeof value === "object"
    ? value
    : undefined;
}

function propertyName(name: ts.PropertyName): string | undefined {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
  return undefined;
}

function numericPropertyForType(type: string): string | undefined {
  const properties: Readonly<Record<string, string>> = {
    number: "scale",
    integer: "z-index",
    length: "flex-basis",
    angle: "rotate",
    time: "transition-delay",
    resolution: "image-resolution",
    flex: "grid-template-rows",
  };
  return properties[type];
}

function runSelfTest(): void {
  const source = `<!doctype html>
<link rel="help" href="https://drafts.csswg.org/css-values/#calc-func">
<script src="/css/support/parsing-testcommon.js"></script>
<script>
const alternatives = ["rgb(255, 0, 0)", "red"];
test_valid_value("color", "rgb(255 0 0)", alternatives);
test_invalid_value("color", generatedValue);
function emitDynamicTest() {
  test_valid_value("color", "blue");
}
</script>`;
  const document = parse(source, { sourceCodeLocationInfo: true });
  const facts = readDocumentFacts(document);
  const extracted = facts.scriptFragments.flatMap((fragment) => {
    const result = extractScriptCalls({
      module: { moduleId: "fixture", wptPath: "css/fixture" },
      wptPath: "css/fixture/static-and-dynamic.html",
      fragment,
      helperClasses: facts.helperClasses,
      specLinks: facts.specLinks,
    });
    return [result];
  });
  const tuples = extracted.flatMap((result) => result.tuples);
  const skipped = extracted.flatMap((result) => result.skippedCalls);
  assert.equal(tuples.length, 1);
  assert.deepEqual(tuples[0]?.expectedValues, ["rgb(255, 0, 0)", "red"]);
  assert.deepEqual(tuples[0]?.specLinks, ["https://drafts.csswg.org/css-values/#calc-func"]);
  assert.deepEqual(
    skipped.map((entry) => entry.reason),
    ["dynamic-value", "dynamic-function-context"],
    "dynamic helper calls must be reported rather than disappearing",
  );
}

function readWptPin(root: string): string {
  const result = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" });
  return `web-platform-tests/wpt@${result.trim()}`;
}

function helperClassForImport(src: string): string | undefined {
  for (const [filename, helperClass] of helperImports) {
    if (src.endsWith(`/${filename}`) || src === filename) return helperClass;
  }
  return undefined;
}

function visitHtmlNode(
  node: DefaultTreeAdapterTypes.Node,
  visitor: (node: DefaultTreeAdapterTypes.Node) => void,
): void {
  visitor(node);
  if ("childNodes" in node) {
    for (const child of node.childNodes) visitHtmlNode(child, visitor);
  }
  if ("content" in node) visitHtmlNode(node.content, visitor);
}

function collectFiles(root: string): string[] {
  if (!statSync(root).isDirectory()) return [];
  const files: string[] = [];
  for (const entry of readdirSync(root).sort()) {
    const filePath = path.join(root, entry);
    if (statSync(filePath).isDirectory()) files.push(...collectFiles(filePath));
    else files.push(filePath);
  }
  return files;
}

function readArgument(name: string): string | undefined {
  const inline = process.argv.find((argument) => argument.startsWith(`${name}=`));
  if (inline) return inline.slice(name.length + 1);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function compareTuple(left: TierZeroTupleV0, right: TierZeroTupleV0): number {
  return (
    left.wptPath.localeCompare(right.wptPath) ||
    left.wptSourceLine - right.wptSourceLine ||
    left.id.localeCompare(right.id)
  );
}

function compareSkippedCall(left: SkippedCallV0, right: SkippedCallV0): number {
  return (
    left.wptPath.localeCompare(right.wptPath) ||
    left.wptSourceLine - right.wptSourceLine ||
    left.helperCall.localeCompare(right.helperCall)
  );
}

function stableJson(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function slash(value: string): string {
  return value.split(path.sep).join("/");
}
