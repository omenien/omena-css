import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import ts from "../server/engine-core-ts/src/ts-facade";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type ShapeClass =
  | "identifierPath"
  | "lexicallyEnumerable"
  | "call"
  | "arithmetic"
  | "logicalOperator"
  | "computedNonLiteral"
  | "nestedTemplate"
  | "multiInterpolation"
  | "other";

type Disposition = "target" | "skipped" | "unrecorded";
type Population = "committed-corpus" | "shape-fixture";

interface ByteSpan {
  readonly start: number;
  readonly end: number;
}

interface ShapeFixture {
  readonly id: string;
  readonly shapeClass: ShapeClass;
  readonly source: string;
}

interface ShapeFixtureFile {
  readonly schemaVersion: "0";
  readonly product: "omena-bridge.source-type-fact-expression-shapes";
  readonly fixtures: readonly ShapeFixture[];
}

interface FarmManifest {
  readonly fixtures: readonly {
    readonly source: {
      readonly kind: string;
      readonly workspacePath?: string;
    };
  }[];
}

interface LintCensusReport {
  readonly policyDigest: string;
}

interface SourceInput {
  readonly id: string;
  readonly sourcePath: string;
  readonly source: string;
  readonly population: Population;
  readonly expectedShapeClass?: ShapeClass;
  readonly importedStyleBindings: readonly ImportedStyleBinding[];
  readonly classnamesBindBindings: readonly string[];
}

interface ImportedStyleBinding {
  readonly binding: string;
  readonly styleUri: string;
}

interface SyntaxSite {
  readonly byteSpan: ByteSpan;
  readonly shapeClass: ShapeClass;
}

interface RustFactCapture {
  readonly byteSpan: ByteSpan;
  readonly expressionId: string;
  readonly reason?: string;
}

interface RustAttemptCapture extends RustFactCapture {
  readonly targetStyleUri?: string;
  readonly shapeClass: ShapeClass;
  readonly lexicalDisposition: "resolved" | "typeProviderCandidate" | "unresolved";
}

interface RustCaptureFixture {
  readonly id: string;
  readonly syntax: {
    readonly typeFactTargets: readonly RustFactCapture[];
    readonly typeFactTargetSkipped: readonly RustFactCapture[];
    readonly typeFactAttempts: readonly RustAttemptCapture[];
  };
}

interface RustCaptureResponse {
  readonly fixtures: readonly RustCaptureFixture[];
}

interface CensusSite {
  readonly population: Population;
  readonly sourcePath: string;
  readonly byteSpan: ByteSpan;
  readonly shapeClass: ShapeClass;
  readonly disposition: Disposition;
  readonly lexicalDisposition: RustAttemptCapture["lexicalDisposition"];
  readonly expressionId?: string;
  readonly reason?: string;
  readonly attemptSpanRelation?: "coversSite";
}

interface ShapeDispositionCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.source-type-fact.shape-disposition-census";
  readonly generatedBy: "scripts/check-rust-omena-source-type-fact-shape-census.ts";
  readonly authorities: {
    readonly syntacticPopulation: "typescript-ast";
    readonly recordedDisposition: "omena-source-frontend-rust-capture";
    readonly joinKey: "source-path-and-byte-span";
    readonly attemptJoin: "exact-span-or-same-start-covering-span";
  };
  readonly corpus: {
    readonly manifestPath: string;
    readonly manifestSha256: string;
    readonly lintReportPath: string;
    readonly lintReportSha256: string;
    readonly lintPolicyDigest: string;
    readonly localWorkspacePaths: readonly string[];
    readonly sourceFileCount: number;
    readonly sourceSiteCount: number;
    readonly nonEnumerableSiteCount: number;
    readonly interpretation: "scale-starved";
  };
  readonly shapeFixtures: {
    readonly path: string;
    readonly sha256: string;
    readonly siteCount: number;
  };
  readonly shapeCounts: Readonly<Record<ShapeClass, number>>;
  readonly fixtureShapeCounts: Readonly<Record<ShapeClass, number>>;
  readonly dispositionCounts: Readonly<Record<Disposition, number>>;
  readonly structuralEntailments: readonly {
    readonly disposition: "unrecorded";
    readonly owner: string;
    readonly reentry: string;
  }[];
  readonly sites: readonly CensusSite[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestRelativePath = "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json";
const lintReportRelativePath =
  "rust/crates/omena-diff-test/oss-corpus-farm/lint-census-report.json";
const fixtureRelativePath =
  "rust/crates/omena-bridge/tests/fixtures/source-type-fact-expression-shapes.json";
const censusRelativePath = "rust/omena-source-type-fact-shape-census.json";
const sourceExtensions = new Set([".cjs", ".js", ".jsx", ".mjs", ".ts", ".tsx"]);
const skippedDirectoryNames = new Set([
  ".git",
  ".next",
  "build",
  "coverage",
  "dist",
  "node_modules",
  "target",
]);
const shapeClasses: readonly ShapeClass[] = [
  "identifierPath",
  "lexicallyEnumerable",
  "call",
  "arithmetic",
  "logicalOperator",
  "computedNonLiteral",
  "nestedTemplate",
  "multiInterpolation",
  "other",
];
const nonEnumerableShapeClasses = new Set<ShapeClass>([
  "call",
  "arithmetic",
  "logicalOperator",
  "computedNonLiteral",
  "nestedTemplate",
  "multiInterpolation",
  "other",
]);
const writeMode = process.argv.includes("--write");
const requireRecorded = process.argv.includes("--assert-all-recorded");

const manifestSource = readFileSync(path.join(repoRoot, manifestRelativePath), "utf8");
const manifest = JSON.parse(manifestSource) as FarmManifest;
const lintReportSource = readFileSync(path.join(repoRoot, lintReportRelativePath), "utf8");
const lintReport = JSON.parse(lintReportSource) as LintCensusReport;
const fixtureSource = readFileSync(path.join(repoRoot, fixtureRelativePath), "utf8");
const fixtureFile = JSON.parse(fixtureSource) as ShapeFixtureFile;
const localWorkspacePaths = manifest.fixtures
  .filter((entry) => entry.source.kind === "local-workspace")
  .map((entry) => entry.source.workspacePath)
  .filter((workspacePath): workspacePath is string => workspacePath !== undefined)
  .toSorted();

assert.ok(localWorkspacePaths.length > 0, "farm manifest must expose local workspace entries");
assert.equal(
  new Set(localWorkspacePaths).size,
  localWorkspacePaths.length,
  "local workspace roots must be unique",
);

const sourcePaths = localWorkspacePaths
  .flatMap((workspacePath) => sourceFilesUnder(path.join(repoRoot, workspacePath)))
  .map((sourcePath) => path.relative(repoRoot, sourcePath))
  .toSorted();
assert.equal(
  new Set(sourcePaths).size,
  sourcePaths.length,
  "local workspace entries must not walk the same source file twice",
);

const sourceInputs: SourceInput[] = sourcePaths.map((sourcePath) => {
  const source = readFileSync(path.join(repoRoot, sourcePath), "utf8");
  return {
    id: `corpus:${sourcePath}`,
    sourcePath,
    source,
    population: "committed-corpus",
    importedStyleBindings: importedStyleBindingsForSource(sourcePath, source),
    classnamesBindBindings: classnamesBindImportNames(sourcePath, source),
  };
});
for (const fixture of fixtureFile.fixtures) {
  sourceInputs.push({
    id: `fixture:${fixture.id}`,
    sourcePath: `source-type-fact-shapes/${fixture.id}.tsx`,
    source: fixture.source,
    population: "shape-fixture",
    expectedShapeClass: fixture.shapeClass,
    importedStyleBindings: importedStyleBindingsForSource(
      `source-type-fact-shapes/${fixture.id}.tsx`,
      fixture.source,
    ),
    classnamesBindBindings: classnamesBindImportNames(
      `source-type-fact-shapes/${fixture.id}.tsx`,
      fixture.source,
    ),
  });
}

const observedSyntaxSites = new Map<string, readonly SyntaxSite[]>();
for (const input of sourceInputs) {
  const inputSites = scanClassNameExpressionSites(input.sourcePath, input.source);
  observedSyntaxSites.set(input.id, inputSites);
  if (input.expectedShapeClass !== undefined) {
    assert.equal(inputSites.length, 1, `${input.id} must expose exactly one shape site`);
    assert.equal(
      inputSites[0]?.shapeClass,
      input.expectedShapeClass,
      `${input.id} shape classification drifted`,
    );
  }
}

const recordedIndexFacts = captureRustIndex(sourceInputs);
const censusSites = joinAuthorities(sourceInputs, observedSyntaxSites, recordedIndexFacts);
const shapeCounts = countByShape(censusSites);
const fixtureShapeCounts = countByShape(
  censusSites.filter((site) => site.population === "shape-fixture"),
);
const dispositionCounts = countByDisposition(censusSites);
for (const shapeClass of shapeClasses) {
  assert.ok(
    fixtureShapeCounts[shapeClass] > 0,
    `shape fixture population is empty for ${shapeClass}`,
  );
}
assert.ok(
  Object.hasOwn(dispositionCounts, "unrecorded"),
  "the unrecorded disposition must remain an explicit census row",
);

const corpusSites = censusSites.filter((site) => site.population === "committed-corpus");
const corpusNonEnumerableSiteCount = corpusSites.filter((site) =>
  nonEnumerableShapeClasses.has(site.shapeClass),
).length;
assert.equal(
  corpusNonEnumerableSiteCount,
  0,
  `the committed corpus is expected to remain scale-starved for non-enumerable shapes: ${JSON.stringify(
    corpusSites.filter((site) => nonEnumerableShapeClasses.has(site.shapeClass)),
  )}`,
);

const census: ShapeDispositionCensus = {
  schemaVersion: "0",
  product: "omena.source-type-fact.shape-disposition-census",
  generatedBy: "scripts/check-rust-omena-source-type-fact-shape-census.ts",
  authorities: {
    syntacticPopulation: "typescript-ast",
    recordedDisposition: "omena-source-frontend-rust-capture",
    joinKey: "source-path-and-byte-span",
    attemptJoin: "exact-span-or-same-start-covering-span",
  },
  corpus: {
    manifestPath: manifestRelativePath,
    manifestSha256: sha256(manifestSource),
    lintReportPath: lintReportRelativePath,
    lintReportSha256: sha256(lintReportSource),
    lintPolicyDigest: lintReport.policyDigest,
    localWorkspacePaths,
    sourceFileCount: sourcePaths.length,
    sourceSiteCount: corpusSites.length,
    nonEnumerableSiteCount: corpusNonEnumerableSiteCount,
    interpretation: "scale-starved",
  },
  shapeFixtures: {
    path: fixtureRelativePath,
    sha256: sha256(fixtureSource),
    siteCount: censusSites.length - corpusSites.length,
  },
  shapeCounts,
  fixtureShapeCounts,
  dispositionCounts,
  structuralEntailments: [
    {
      disposition: "unrecorded",
      owner: "omena-bridge-source-type-fact-collection",
      reentry: "className-site-bypasses-target-and-skipped-recording",
    },
  ],
  sites: censusSites,
};

assert.equal(
  census.dispositionCounts.unrecorded,
  0,
  "every className expression site must have a recorded index disposition",
);

const serialized = `${JSON.stringify(census, null, 2)}\n`;
const censusPath = path.join(repoRoot, censusRelativePath);
if (writeMode) {
  writeFileSync(censusPath, serialized);
} else {
  assert.ok(existsSync(censusPath), `${censusRelativePath} must exist; regenerate with --write`);
  assert.equal(
    readFileSync(censusPath, "utf8"),
    serialized,
    `${censusRelativePath} drifted; regenerate with --write`,
  );
}

console.log(
  JSON.stringify(
    {
      product: census.product,
      corpus: census.corpus,
      shapeCounts: census.shapeCounts,
      dispositionCounts: census.dispositionCounts,
      strictRecordingRequested: requireRecorded,
    },
    null,
    2,
  ),
);

function sourceFilesUnder(root: string): string[] {
  const files: string[] = [];
  for (const entry of evidenceScanSurface.readdirSync(root, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) {
      continue;
    }
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!skippedDirectoryNames.has(entry.name)) {
        files.push(...sourceFilesUnder(entryPath));
      }
      continue;
    }
    if (entry.isFile() && sourceExtensions.has(path.extname(entry.name))) {
      files.push(entryPath);
    }
  }
  return files;
}

function classnamesBindImportNames(sourcePath: string, source: string): string[] {
  const sourceFile = createSourceFile(sourcePath, source);
  const names = new Set<string>();
  sourceFile.forEachChild((node) => {
    if (
      !ts.isImportDeclaration(node) ||
      !ts.isStringLiteral(node.moduleSpecifier) ||
      node.moduleSpecifier.text !== "classnames/bind"
    ) {
      return;
    }
    const clause = node.importClause;
    if (clause?.name !== undefined) {
      names.add(clause.name.text);
    }
    const bindings = clause?.namedBindings;
    if (bindings !== undefined && ts.isNamedImports(bindings)) {
      for (const element of bindings.elements) {
        names.add(element.name.text);
      }
    }
  });
  return [...names].toSorted();
}

function importedStyleBindingsForSource(
  sourcePath: string,
  source: string,
): ImportedStyleBinding[] {
  const sourceFile = createSourceFile(sourcePath, source);
  const bindings = new Map<string, string>();
  sourceFile.forEachChild((node) => {
    if (
      !ts.isImportDeclaration(node) ||
      !ts.isStringLiteral(node.moduleSpecifier) ||
      !/\.module\.(?:css|less|sass|scss)$/u.test(node.moduleSpecifier.text)
    ) {
      return;
    }
    const relativeStylePath = path.posix.normalize(
      path.posix.join("/workspace", path.posix.dirname(sourcePath), node.moduleSpecifier.text),
    );
    const styleUri = `file://${relativeStylePath}`;
    const clause = node.importClause;
    if (clause?.name !== undefined) {
      bindings.set(clause.name.text, styleUri);
    }
    const namedBindings = clause?.namedBindings;
    if (namedBindings !== undefined && ts.isNamespaceImport(namedBindings)) {
      bindings.set(namedBindings.name.text, styleUri);
    }
  });
  return [...bindings]
    .map(([binding, styleUri]) => ({ binding, styleUri }))
    .toSorted((left, right) => left.binding.localeCompare(right.binding));
}

function scanClassNameExpressionSites(sourcePath: string, source: string): SyntaxSite[] {
  const sourceFile = createSourceFile(sourcePath, source);
  const boundUtilityNames = collectBoundUtilityNames(sourceFile);
  const sites: SyntaxSite[] = [];
  const visit = (node: ts.Node): void => {
    if (
      ts.isJsxAttribute(node) &&
      node.name.getText(sourceFile) === "className" &&
      node.initializer !== undefined &&
      ts.isJsxExpression(node.initializer) &&
      node.initializer.expression !== undefined
    ) {
      collectSitesFromClassExpression(
        unwrapExpression(node.initializer.expression),
        source,
        sourceFile,
        boundUtilityNames,
        sites,
      );
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return sites
    .toSorted((left, right) => {
      return (
        left.byteSpan.start - right.byteSpan.start ||
        left.byteSpan.end - right.byteSpan.end ||
        left.shapeClass.localeCompare(right.shapeClass)
      );
    })
    .filter((site, index, all) => {
      const previous = all[index - 1];
      return (
        previous === undefined ||
        previous.byteSpan.start !== site.byteSpan.start ||
        previous.byteSpan.end !== site.byteSpan.end ||
        previous.shapeClass !== site.shapeClass
      );
    });
}

function collectSitesFromClassExpression(
  expression: ts.Expression,
  source: string,
  sourceFile: ts.SourceFile,
  boundUtilityNames: ReadonlySet<string>,
  sites: SyntaxSite[],
): void {
  const unwrapped = unwrapExpression(expression);
  if (ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)) {
    return;
  }
  if (ts.isCallExpression(unwrapped) && isClassUtilityCall(unwrapped, boundUtilityNames)) {
    for (const argument of unwrapped.arguments) {
      if (!ts.isSpreadElement(argument)) {
        collectSitesFromClassExpression(argument, source, sourceFile, boundUtilityNames, sites);
      }
    }
    return;
  }
  if (ts.isArrayLiteralExpression(unwrapped)) {
    for (const element of unwrapped.elements) {
      if (!ts.isSpreadElement(element)) {
        collectSitesFromClassExpression(element, source, sourceFile, boundUtilityNames, sites);
      }
    }
    return;
  }
  if (ts.isObjectLiteralExpression(unwrapped)) {
    for (const property of unwrapped.properties) {
      if (property.kind === ts.SyntaxKind.SpreadAssignment) {
        collectSitesFromClassExpression(
          property.expression,
          source,
          sourceFile,
          boundUtilityNames,
          sites,
        );
      } else if (
        (ts.isPropertyAssignment(property) ||
          ts.isShorthandPropertyAssignment(property) ||
          ts.isMethodDeclaration(property)) &&
        ts.isComputedPropertyName(property.name)
      ) {
        collectSitesFromClassExpression(
          property.name.expression,
          source,
          sourceFile,
          boundUtilityNames,
          sites,
        );
      }
    }
    return;
  }
  if (ts.isConditionalExpression(unwrapped)) {
    collectSitesFromClassExpression(
      unwrapped.whenTrue,
      source,
      sourceFile,
      boundUtilityNames,
      sites,
    );
    collectSitesFromClassExpression(
      unwrapped.whenFalse,
      source,
      sourceFile,
      boundUtilityNames,
      sites,
    );
    return;
  }
  if (
    ts.isBinaryExpression(unwrapped) &&
    (unwrapped.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken ||
      unwrapped.operatorToken.kind === ts.SyntaxKind.BarBarToken)
  ) {
    collectSitesFromClassExpression(unwrapped.right, source, sourceFile, boundUtilityNames, sites);
    return;
  }
  if (ts.isTemplateExpression(unwrapped)) {
    if (unwrapped.templateSpans.length > 1) {
      sites.push(siteForNode(unwrapped, "multiInterpolation", source, sourceFile));
      return;
    }
    const interpolation = unwrapped.templateSpans[0]?.expression;
    if (interpolation !== undefined) {
      const interpolationExpression = unwrapExpression(interpolation);
      sites.push(
        siteForNode(
          interpolationExpression,
          classifyShape(interpolationExpression),
          source,
          sourceFile,
        ),
      );
    }
    return;
  }
  sites.push(siteForNode(unwrapped, classifyShape(unwrapped), source, sourceFile));
}

function collectBoundUtilityNames(sourceFile: ts.SourceFile): ReadonlySet<string> {
  const names = new Set(["classnames", "classNames", "clsx", "cn"]);
  const classnamesBindImports = new Set<string>();
  const visit = (node: ts.Node): void => {
    if (
      ts.isImportDeclaration(node) &&
      ts.isStringLiteral(node.moduleSpecifier) &&
      ["classnames", "classnames/bind", "clsx", "clsx/lite"].includes(node.moduleSpecifier.text)
    ) {
      const clause = node.importClause;
      if (clause?.name !== undefined) {
        names.add(clause.name.text);
        if (node.moduleSpecifier.text === "classnames/bind") {
          classnamesBindImports.add(clause.name.text);
        }
      }
      const bindings = clause?.namedBindings;
      if (bindings !== undefined && ts.isNamedImports(bindings)) {
        for (const element of bindings.elements) {
          names.add(element.name.text);
          if (node.moduleSpecifier.text === "classnames/bind") {
            classnamesBindImports.add(element.name.text);
          }
        }
      }
    }
    if (
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.initializer !== undefined &&
      ts.isCallExpression(node.initializer) &&
      ts.isPropertyAccessExpression(node.initializer.expression) &&
      node.initializer.expression.name.text === "bind" &&
      ts.isIdentifier(node.initializer.expression.expression) &&
      classnamesBindImports.has(node.initializer.expression.expression.text)
    ) {
      names.add(node.name.text);
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return names;
}

function isClassUtilityCall(
  expression: ts.CallExpression,
  boundUtilityNames: ReadonlySet<string>,
): boolean {
  return (
    ts.isIdentifier(expression.expression) && boundUtilityNames.has(expression.expression.text)
  );
}

function classifyShape(expression: ts.Expression): ShapeClass {
  if (isIdentifierPath(expression)) {
    return "identifierPath";
  }
  if (ts.isConditionalExpression(expression)) {
    return "lexicallyEnumerable";
  }
  if (ts.isCallExpression(expression)) {
    return "call";
  }
  if (
    ts.isElementAccessExpression(expression) &&
    !isStaticElementKey(expression.argumentExpression)
  ) {
    return "computedNonLiteral";
  }
  if (ts.isTemplateExpression(expression)) {
    return "nestedTemplate";
  }
  if (ts.isBinaryExpression(expression)) {
    if (
      expression.operatorToken.kind === ts.SyntaxKind.AmpersandAmpersandToken ||
      expression.operatorToken.kind === ts.SyntaxKind.BarBarToken ||
      expression.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken
    ) {
      return "logicalOperator";
    }
    if (
      [
        ts.SyntaxKind.PlusToken,
        ts.SyntaxKind.MinusToken,
        ts.SyntaxKind.AsteriskToken,
        ts.SyntaxKind.AsteriskAsteriskToken,
        ts.SyntaxKind.SlashToken,
        ts.SyntaxKind.PercentToken,
      ].includes(expression.operatorToken.kind)
    ) {
      return "arithmetic";
    }
  }
  return "other";
}

function isIdentifierPath(expression: ts.Expression): boolean {
  const unwrapped = unwrapExpression(expression);
  if (ts.isIdentifier(unwrapped)) {
    return true;
  }
  if (ts.isPropertyAccessExpression(unwrapped)) {
    return isIdentifierPath(unwrapped.expression);
  }
  if (ts.isElementAccessExpression(unwrapped) && isStaticElementKey(unwrapped.argumentExpression)) {
    return isIdentifierPath(unwrapped.expression);
  }
  return false;
}

function isStaticElementKey(expression: ts.Expression | undefined): boolean {
  if (expression === undefined) {
    return false;
  }
  const unwrapped = unwrapExpression(expression);
  return ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped);
}

function unwrapExpression(expression: ts.Expression): ts.Expression {
  let current = expression;
  for (;;) {
    if (
      ts.isParenthesizedExpression(current) ||
      ts.isAsExpression(current) ||
      ts.isTypeAssertionExpression(current) ||
      ts.isNonNullExpression(current) ||
      ts.isSatisfiesExpression(current)
    ) {
      current = current.expression;
      continue;
    }
    return current;
  }
}

function siteForNode(
  node: ts.Node,
  shapeClass: ShapeClass,
  source: string,
  sourceFile: ts.SourceFile,
): SyntaxSite {
  const start = node.getStart(sourceFile);
  const end = node.getEnd();
  return {
    byteSpan: {
      start: Buffer.byteLength(source.slice(0, start), "utf8"),
      end: Buffer.byteLength(source.slice(0, end), "utf8"),
    },
    shapeClass,
  };
}

function createSourceFile(sourcePath: string, source: string): ts.SourceFile {
  const extension = path.extname(sourcePath);
  const scriptKind =
    extension === ".js" || extension === ".mjs" || extension === ".cjs"
      ? ts.ScriptKind.JS
      : extension === ".jsx"
        ? ts.ScriptKind.JSX
        : extension === ".tsx"
          ? ts.ScriptKind.TSX
          : ts.ScriptKind.TS;
  return ts.createSourceFile(sourcePath, source, ts.ScriptTarget.Latest, true, scriptKind);
}

function captureRustIndex(inputs: readonly SourceInput[]): RustCaptureResponse {
  const child = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-diff-test",
      "--bin",
      "omena-source-frontend-rust-capture",
    ],
    {
      cwd: repoRoot,
      input: JSON.stringify({
        fixtures: inputs.map((input) => ({
          id: input.id,
          sourcePath: input.sourcePath,
          source: input.source,
          sourceLanguage: null,
          importedStyleBindings: input.importedStyleBindings,
          classnamesBindBindings: input.classnamesBindBindings,
          cfgVariableName: "__source_type_fact_census__",
          cfgReferenceByteOffset: 0,
        })),
      }),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );
  assert.equal(
    child.status,
    0,
    `Rust source index capture failed\nstdout:\n${child.stdout}\nstderr:\n${child.stderr}`,
  );
  return JSON.parse(child.stdout) as RustCaptureResponse;
}

function joinAuthorities(
  inputs: readonly SourceInput[],
  syntaxSitesById: ReadonlyMap<string, readonly SyntaxSite[]>,
  rustCapture: RustCaptureResponse,
): CensusSite[] {
  const rustById = new Map(rustCapture.fixtures.map((fixture) => [fixture.id, fixture]));
  const sites: CensusSite[] = [];
  for (const input of inputs) {
    const syntaxSites = syntaxSitesById.get(input.id) ?? [];
    const capture = rustById.get(input.id);
    assert.ok(capture, `Rust capture is missing ${input.id}`);
    for (const syntaxSite of syntaxSites) {
      const targets = capture.syntax.typeFactTargets.filter((target) =>
        spansEqual(target.byteSpan, syntaxSite.byteSpan),
      );
      const skipped = capture.syntax.typeFactTargetSkipped.filter((fact) =>
        spansEqual(fact.byteSpan, syntaxSite.byteSpan),
      );
      const exactAttempts = capture.syntax.typeFactAttempts.filter((fact) =>
        spansEqual(fact.byteSpan, syntaxSite.byteSpan),
      );
      const attempts =
        exactAttempts.length > 0
          ? exactAttempts
          : capture.syntax.typeFactAttempts.filter(
              (fact) =>
                fact.byteSpan.start === syntaxSite.byteSpan.start &&
                fact.byteSpan.end >= syntaxSite.byteSpan.end,
            );
      assert.ok(
        targets.length <= 1 && skipped.length <= 1 && targets.length + skipped.length <= 1,
        `${input.id}:${syntaxSite.byteSpan.start}-${syntaxSite.byteSpan.end} has conflicting index dispositions`,
      );
      assert.equal(
        attempts.length,
        1,
        `${input.id}:${syntaxSite.byteSpan.start}-${syntaxSite.byteSpan.end} must have exactly one type-fact attempt`,
      );
      const attempt = attempts[0];
      assert.ok(attempt);
      assert.equal(
        attempt.shapeClass,
        syntaxSite.shapeClass,
        `${input.id}:${syntaxSite.byteSpan.start}-${syntaxSite.byteSpan.end} shape authorities disagree`,
      );
      const fact = targets[0] ?? skipped[0];
      sites.push({
        population: input.population,
        sourcePath: input.sourcePath,
        byteSpan: syntaxSite.byteSpan,
        shapeClass: syntaxSite.shapeClass,
        lexicalDisposition: attempt.lexicalDisposition,
        disposition:
          targets.length === 1 ? "target" : skipped.length === 1 ? "skipped" : "unrecorded",
        ...(fact === undefined ? {} : { expressionId: fact.expressionId }),
        ...(fact?.reason === undefined ? {} : { reason: fact.reason }),
        ...(exactAttempts.length > 0 ? {} : { attemptSpanRelation: "coversSite" }),
      });
    }
  }
  return sites.toSorted((left, right) => {
    return (
      left.population.localeCompare(right.population) ||
      left.sourcePath.localeCompare(right.sourcePath) ||
      left.byteSpan.start - right.byteSpan.start ||
      left.byteSpan.end - right.byteSpan.end ||
      left.shapeClass.localeCompare(right.shapeClass)
    );
  });
}

function spansEqual(left: ByteSpan, right: ByteSpan): boolean {
  return left.start === right.start && left.end === right.end;
}

function countByShape(sites: readonly CensusSite[]): Record<ShapeClass, number> {
  return Object.fromEntries(
    shapeClasses.map((shapeClass) => [
      shapeClass,
      sites.filter((site) => site.shapeClass === shapeClass).length,
    ]),
  ) as Record<ShapeClass, number>;
}

function countByDisposition(sites: readonly CensusSite[]): Record<Disposition, number> {
  return {
    target: sites.filter((site) => site.disposition === "target").length,
    skipped: sites.filter((site) => site.disposition === "skipped").length,
    unrecorded: sites.filter((site) => site.disposition === "unrecorded").length,
  };
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}
