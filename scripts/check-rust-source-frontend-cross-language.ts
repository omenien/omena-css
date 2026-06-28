import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import ts from "typescript";
import { buildSourceBinder } from "../server/engine-core-ts/src/core/binder/binder-builder";
import {
  buildSourceBindingGraph,
  type SourceBindingGraph,
} from "../server/engine-core-ts/src/core/binder/source-binding-graph";
import { cssModulesClassnamesBinderPluginV0 } from "../server/engine-core-ts/src/core/binder/binder-plugin";
import { AliasResolver } from "../server/engine-core-ts/src/core/cx/alias-resolver";
import { buildSourceDocument } from "../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";
import type { SourceBinderResult } from "../server/engine-core-ts/src/core/binder/scope-types";
import type { SourceDocumentHIR } from "../server/engine-core-ts/src/core/hir/source-types";
import {
  captureTsSourceFrontendFactsV0,
  stringifyCanonicalSourceFrontendJsonV0,
  type CanonicalSourceFrontendCaptureV0,
} from "../server/engine-core-ts/src/core/source-frontend/canonical-capture";

interface FrontendFixtureV0 {
  readonly id: string;
  readonly sourcePath: string;
  readonly source: string;
  readonly sourceLanguage?: string;
  readonly cfgReferenceToken: string;
  readonly cfgVariableName: string;
}

interface RustCaptureResponseV0 {
  readonly fixtures: readonly RustFixtureCaptureV0[];
}

interface RustFixtureCaptureV0 {
  readonly id: string;
  readonly syntax: {
    readonly importedStyleBindings: readonly CanonicalImportedStyleBindingV0[];
    readonly stylePropertyAccesses: readonly CanonicalStylePropertyAccessV0[];
    readonly selectorReferences: readonly CanonicalSelectorReferenceV0[];
  };
}

interface CanonicalImportedStyleBindingV0 {
  readonly binding: string;
  readonly styleUri: string;
}

interface CanonicalStylePropertyAccessV0 {
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly selectorName: string;
  readonly targetStyleUri: string | null;
}

interface CanonicalSelectorReferenceV0 {
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly selectorName: string | null;
  readonly matchKind: "exact" | "prefix";
  readonly targetStyleUri: string | null;
}

const workspaceRoot = "/fake/ws";
const aliasResolver = new AliasResolver(workspaceRoot, {});
const minimumCrossLanguageFixtureCount = 3;

const fixtures: readonly FrontendFixtureV0[] = [
  {
    id: "css-modules-cx-style-access-flow",
    sourcePath: "/fake/ws/src/Card.tsx",
    cfgReferenceToken: "size",
    cfgVariableName: "size",
    source: [
      'import classNames from "classnames/bind";',
      'import clsx from "clsx";',
      'import styles from "./Card.module.scss";',
      "const cx = classNames.bind(styles);",
      'export function Card({ enabled, tone }: { enabled: boolean; tone: "warm" | "cool" }) {',
      '  let size = "card";',
      "  if (enabled) {",
      '    size = "card--active";',
      "  }",
      '  return <div className={clsx(cx("card", `tone-${tone}`, size), styles.icon)} />;',
      "}",
      "",
    ].join("\n"),
  },
  {
    id: "css-modules-collection-arguments",
    sourcePath: "/fake/ws/src/Panel.tsx",
    cfgReferenceToken: "local",
    cfgVariableName: "local",
    source: [
      'import bind from "classnames/bind";',
      'import clsx from "clsx";',
      'import styles from "./Panel.module.scss";',
      "const cx = bind.bind(styles);",
      'export function Panel({ enabled, tone }: { enabled: boolean; tone: "info" | "warn" }) {',
      '  const local = "panel__local";',
      '  return <section className={clsx(styles.title, cx(["panel", enabled && "panel--enabled", `tone-${tone}`, local]))} />;',
      "}",
      "",
    ].join("\n"),
  },
  {
    id: "css-modules-object-arguments",
    sourcePath: "/fake/ws/src/Nav.tsx",
    cfgReferenceToken: "item",
    cfgVariableName: "item",
    source: [
      'import bind from "classnames/bind";',
      'import styles from "./Nav.module.scss";',
      "const cx = bind.bind(styles);",
      "export function Nav({ active }: { active: boolean }) {",
      '  const item = "nav__item";',
      '  return <nav className={cx({ nav: true, "nav--active": active, [item]: active }, styles["nav__label"])} />;',
      "}",
      "",
    ].join("\n"),
  },
];

const captures = fixtures.map(captureFixture);
for (const capture of captures) {
  const again = captureFixture(fixtures.find((fixture) => fixture.id === capture.fixtureId)!);
  assert.equal(
    stringifyCanonicalSourceFrontendJsonV0(capture),
    stringifyCanonicalSourceFrontendJsonV0(again),
    `${capture.id} TS frontend capture must be deterministic`,
  );
}

const rustResponse = captureRustSyntax(captures);
const reports = captures.map((capture) => compareFixture(capture, rustResponse));

assert.ok(
  reports.length >= minimumCrossLanguageFixtureCount,
  `cross-language source frontend corpus must include at least ${minimumCrossLanguageFixtureCount} fixtures`,
);
assert.ok(
  reports.every((report) => report.syntax.coveredFieldsMatch),
  `covered source syntax fields must match: ${JSON.stringify(reports, null, 2)}`,
);
assert.ok(
  reports.every((report) => report.binding.status === "recorded-red"),
  "binding graph must remain an explicit gap until the Rust binding oracle is built",
);
assert.ok(
  reports.every((report) => report.cfg.status === "recorded-red"),
  "CFG must remain an explicit gap until the Rust CFG oracle is built",
);

console.log(
  JSON.stringify(
    {
      product: "omena.source-frontend-cross-language-capture.check",
      fixtureCount: reports.length,
      reports,
    },
    null,
    2,
  ),
);

function captureFixture(fixture: FrontendFixtureV0): CanonicalSourceFrontendCaptureV0 & {
  readonly fixtureId: string;
  readonly rustRequest: unknown;
} {
  const sourceFile = ts.createSourceFile(
    fixture.sourcePath,
    fixture.source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const sourceBinder = buildSourceBinder(sourceFile);
  const pluginAnalysis = cssModulesClassnamesBinderPluginV0.analyzeSource({
    sourceFile,
    filePath: fixture.sourcePath,
    sourceBinder,
    fileExists: () => true,
    aliasResolver,
  });
  const sourceDocument = buildSourceDocument({
    filePath: fixture.sourcePath,
    cxBindings: pluginAnalysis.cxBindings,
    stylesBindings: pluginAnalysis.stylesBindings,
    classUtilNames: pluginAnalysis.classUtilNames,
    sourceBinder,
    classExpressions: pluginAnalysis.classExpressions,
    domainClassReferences: pluginAnalysis.domainClassReferences,
  });
  const sourceBindingGraph = buildSourceBindingGraph(sourceDocument, sourceBinder);
  const capture = captureTsSourceFrontendFactsV0({
    sourceFile,
    sourceBinder,
    sourceDocument,
    sourceBindingGraph,
    cfg: {
      variableName: fixture.cfgVariableName,
      referenceRange: rangeForToken(sourceFile, fixture.cfgReferenceToken),
    },
  });

  assertCaptureHasLoadBearingFacts(capture, sourceBinder, sourceDocument, sourceBindingGraph);

  return {
    ...capture,
    fixtureId: fixture.id,
    rustRequest: {
      id: fixture.id,
      sourcePath: fixture.sourcePath,
      source: fixture.source,
      sourceLanguage: fixture.sourceLanguage,
      importedStyleBindings: capture.syntax.importedStyleBindings,
      classnamesBindBindings: pluginAnalysis.rawCxBindings
        .map((binding) => binding.classNamesImportName)
        .toSorted(),
    },
  };
}

function captureRustSyntax(
  fixtureCaptures: readonly (CanonicalSourceFrontendCaptureV0 & {
    readonly rustRequest: unknown;
  })[],
): RustCaptureResponseV0 {
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
      cwd: process.cwd(),
      input: JSON.stringify({ fixtures: fixtureCaptures.map((capture) => capture.rustRequest) }),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 16,
    },
  );
  assert.equal(
    child.status,
    0,
    `Rust source frontend capture failed\nstdout:\n${child.stdout}\nstderr:\n${child.stderr}`,
  );
  return JSON.parse(child.stdout) as RustCaptureResponseV0;
}

function compareFixture(
  tsCapture: CanonicalSourceFrontendCaptureV0,
  rustCaptureResponse: RustCaptureResponseV0,
) {
  const rustCapture = rustCaptureResponse.fixtures.find(
    (fixture) => fixture.id === fixtureId(tsCapture),
  );
  assert.ok(rustCapture, `missing Rust capture for ${fixtureId(tsCapture)}`);
  const rustStyleAccessSpans = new Set(
    rustCapture.syntax.stylePropertyAccesses.map((access) => spanKey(access.byteSpan)),
  );
  const tsSymbolReferenceSpans = new Set(
    tsCapture.syntax.symbolReferences.map((reference) => spanKey(reference.byteSpan)),
  );
  const rustDirectSelectorReferences = rustCapture.syntax.selectorReferences
    .filter(
      (reference) =>
        reference.targetStyleUri !== null &&
        !rustStyleAccessSpans.has(spanKey(reference.byteSpan)) &&
        !tsSymbolReferenceSpans.has(spanKey(reference.byteSpan)),
    )
    .toSorted(compareByStableJson);
  const rustSymbolSelectorReferences = rustCapture.syntax.selectorReferences
    .filter(
      (reference) =>
        reference.targetStyleUri !== null &&
        tsSymbolReferenceSpans.has(spanKey(reference.byteSpan)),
    )
    .toSorted(compareByStableJson);
  const fields = [
    fieldReport(
      "importedStyleBindings",
      tsCapture.syntax.importedStyleBindings,
      rustCapture.syntax.importedStyleBindings,
    ),
    fieldReport(
      "stylePropertyAccesses",
      tsCapture.syntax.stylePropertyAccesses,
      rustCapture.syntax.stylePropertyAccesses.toSorted(compareByStableJson),
    ),
    fieldReport(
      "selectorReferences",
      tsCapture.syntax.selectorReferences,
      rustDirectSelectorReferences,
    ),
  ];
  const recordedGaps = [
    {
      field: "symbolRefSelectorReferences",
      status: "recorded-red",
      reason:
        "Rust currently records local class-value selector projections before the Rust binding/CFG oracle is built.",
      tsJson: stringifyCanonicalSourceFrontendJsonV0(tsCapture.syntax.symbolReferences),
      rustJson: stringifyCanonicalSourceFrontendJsonV0(rustSymbolSelectorReferences),
    },
  ];
  return {
    fixture: fixtureId(tsCapture),
    syntax: {
      status: "partial-green",
      coveredFields: fields,
      coveredFieldsMatch: fields.every((field) => field.matches),
      allFieldsMatch: fields.every((field) => field.matches) && recordedGaps.length === 0,
      recordedGaps,
    },
    binding: {
      status: "recorded-red",
      reason: "Rust binding graph projection is not built yet.",
      tsNodeCount: tsCapture.bindingGraph.nodes.length,
      tsEdgeCount: tsCapture.bindingGraph.edges.length,
    },
    cfg: {
      status: "recorded-red",
      reason: "Rust sparse CFG projection is not built yet.",
      tsBlockCount: tsCapture.cfgSnapshot?.snapshot.blocks.length ?? 0,
    },
  };
}

function fieldReport(field: string, tsValue: unknown, rustValue: unknown) {
  const tsJson = stringifyCanonicalSourceFrontendJsonV0(tsValue);
  const rustJson = stringifyCanonicalSourceFrontendJsonV0(rustValue);
  return {
    field,
    matches: tsJson === rustJson,
    tsJson,
    rustJson,
  };
}

function assertCaptureHasLoadBearingFacts(
  capture: CanonicalSourceFrontendCaptureV0,
  sourceBinder: SourceBinderResult,
  sourceDocument: SourceDocumentHIR,
  sourceBindingGraph: SourceBindingGraph,
): void {
  assert.ok(capture.syntax.importedStyleBindings.length > 0, "fixture must include style imports");
  assert.ok(capture.syntax.stylePropertyAccesses.length > 0, "fixture must include style access");
  assert.ok(capture.syntax.selectorReferences.length > 0, "fixture must include selector refs");
  assert.ok(capture.syntax.symbolReferences.length > 0, "fixture must include symbol refs");
  assert.ok(sourceBinder.decls.length > 0, "fixture must include binder declarations");
  assert.ok(sourceDocument.classExpressions.length > 0, "fixture must include class expressions");
  assert.ok(sourceBindingGraph.edges.length > 0, "fixture must include binding graph edges");
  assert.ok(capture.cfgSnapshot !== null, "fixture must include a CFG snapshot");
}

function rangeForToken(sourceFile: ts.SourceFile, token: string) {
  const start = sourceFile.text.lastIndexOf(token);
  assert.notEqual(start, -1, `missing token ${token}`);
  const end = start + token.length;
  return {
    start: sourceFile.getLineAndCharacterOfPosition(start),
    end: sourceFile.getLineAndCharacterOfPosition(end),
  };
}

function fixtureId(
  capture: CanonicalSourceFrontendCaptureV0 & { readonly fixtureId?: string },
): string {
  return capture.fixtureId ?? capture.sourcePath;
}

function spanKey(span: { readonly start: number; readonly end: number }): string {
  return `${span.start}:${span.end}`;
}

function compareByStableJson(left: unknown, right: unknown): number {
  return stringifyCanonicalSourceFrontendJsonV0(left).localeCompare(
    stringifyCanonicalSourceFrontendJsonV0(right),
  );
}
