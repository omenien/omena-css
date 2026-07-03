import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import ts from "../server/engine-core-ts/src/ts-facade";
import { buildSourceBinder } from "../server/engine-core-ts/src/core/source-frontend/ts-source-binder-oracle";
import {
  composeSourceBindingGraph,
  type SourceBindingGraph,
} from "../server/engine-core-ts/src/core/binder/source-binding-graph";
import { cssModulesClassnamesBinderPluginV0 } from "../server/engine-core-ts/src/core/binder/binder-plugin";
import { AliasResolver } from "../server/engine-core-ts/src/core/cx/alias-resolver";
import { resolveFlowClassValues } from "../server/engine-core-ts/src/core/source-frontend/ts-flow-class-value-oracle";
import { buildSourceDocument } from "../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";
import type { SourceBinderResult } from "../server/engine-core-ts/src/core/binder/scope-types";
import type { SourceDocumentHIR } from "../server/engine-core-ts/src/core/hir/source-types";
import { makeStyleDocumentHIR } from "../server/engine-core-ts/src/core/hir/style-types";
import type {
  SelectorDeclHIR,
  StyleDocumentHIR,
} from "../server/engine-core-ts/src/core/hir/style-types";
import {
  captureTsSourceFrontendFactsV0,
  stringifyCanonicalSourceFrontendJsonV0,
  type CanonicalSourceFrontendCaptureV0,
} from "../server/engine-core-ts/src/core/source-frontend/canonical-capture";
import {
  projectRustSourceBindingIndexV0,
  type RustSourceBindingIndexV0,
} from "../server/engine-core-ts/src/core/source-frontend/rust-binding-index-projection";
import { UnresolvableTypeResolver } from "../server/engine-core-ts/src/core/ts/type-resolver";
import type { TypeFactControlFlowGraphV2 } from "../server/engine-core-ts/src/contracts";
import type { SourceLanguage } from "../server/engine-core-ts/src/core/hir/shared-types";

interface FrontendFixtureV0 {
  readonly id: string;
  readonly sourcePath: string;
  readonly stylePath: string;
  readonly selectorNames: readonly string[];
  readonly source: string;
  readonly sourceLanguage?: string;
  readonly cfgReferenceToken: string;
  readonly cfgVariableName: string;
}

type FixtureCaptureV0 = CanonicalSourceFrontendCaptureV0 & {
  readonly fixtureId: string;
  readonly rustRequest: unknown;
};

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
  readonly binding: {
    readonly bindingScopes: readonly CanonicalBindingScopeV0[];
    readonly scopeParentEdges: readonly CanonicalScopeParentEdgeV0[];
    readonly bindingDecls: readonly CanonicalBindingDeclV0[];
    readonly scopeContainsDecls: readonly CanonicalScopeContainsDeclV0[];
    readonly styleModules: readonly CanonicalStyleModuleNodeV0[];
    readonly classExpressionNodes: readonly CanonicalClassExpressionNodeV0[];
    readonly styleImportBindings: readonly CanonicalBindingStyleImportV0[];
    readonly declaresStyleImports: readonly CanonicalDeclaresStyleImportV0[];
    readonly styleImportResolvesModules: readonly CanonicalStyleImportResolvesModuleV0[];
    readonly expressionTargetsModules: readonly CanonicalExpressionTargetsModuleV0[];
    readonly classUtilBindings: readonly CanonicalClassUtilBindingV0[];
    readonly styleAccessUsesStyleImports: readonly CanonicalStyleAccessUsesStyleImportV0[];
    readonly symbolRefUsesDecls: readonly CanonicalSymbolRefUsesDeclV0[];
    readonly classnamesBindUtilityBindings: readonly CanonicalClassnamesBindUtilityBindingV0[];
    readonly declaresUtilityBindings: readonly CanonicalDeclaresUtilityBindingV0[];
    readonly utilityUsesStyleImports: readonly CanonicalUtilityUsesStyleImportV0[];
  };
  readonly cfgSnapshot: RustSourceControlFlowGraphCaptureV0 | null;
  readonly cfgProductContract: TypeFactControlFlowGraphV2 | null;
}

interface RustSourceControlFlowGraphCaptureV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-bridge.source-control-flow-graph";
  readonly binding: {
    readonly symbolOrdinal: number;
    readonly name: string;
  };
  readonly variableName: string;
  readonly referenceByteOffset: number;
  readonly snapshot: {
    readonly entryBlockId: string;
    readonly blocks: readonly RustFlowBlockSnapshotV0[];
  };
}

interface RustFlowBlockSnapshotV0 {
  readonly id: string;
  readonly kind:
    | "entry"
    | "assignment"
    | "branch"
    | "join"
    | "loopHeader"
    | "loopBody"
    | "loopExit"
    | "break"
    | "terminate"
    | "logicalOperand"
    | "logicalRhs"
    | "logicalJoin"
    | "exit";
  readonly transferKind:
    | "entry"
    | "assignFacts"
    | "branch"
    | "concatFacts"
    | "join"
    | "loop"
    | "break"
    | "terminate"
    | "exit";
  readonly successorBlockIds: readonly string[];
  readonly binding?: {
    readonly symbolOrdinal: number;
    readonly name: string;
  };
  readonly symbolOrdinal?: number;
  readonly variableName?: string;
  readonly expressionKind?: "logicalAnd" | "logicalOr" | "nullishCoalesce";
}

interface CanonicalImportedStyleBindingV0 {
  readonly binding: string;
  readonly styleUri: string;
}

interface CanonicalBindingScopeV0 {
  readonly kind: "sourceFile" | "function" | "block";
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
}

interface CanonicalScopeParentEdgeV0 {
  readonly childKind: "sourceFile" | "function" | "block";
  readonly childByteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly parentKind: "sourceFile" | "function" | "block";
  readonly parentByteSpan: {
    readonly start: number;
    readonly end: number;
  };
}

interface CanonicalBindingDeclV0 {
  readonly kind: "import" | "localVar" | "parameter";
  readonly name: string;
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly importPath?: string;
}

interface CanonicalScopeContainsDeclV0 {
  readonly scopeKind: "sourceFile" | "function" | "block";
  readonly scopeByteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly declKind: "import" | "localVar" | "parameter";
  readonly declName: string;
  readonly declByteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly importPath?: string;
}

interface CanonicalStyleModuleNodeV0 {
  readonly styleUri: string;
}

interface CanonicalClassExpressionNodeV0 {
  readonly kind: "literal" | "template" | "styleAccess" | "symbolRef";
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly targetStyleUri: string;
}

interface CanonicalClassnamesBindUtilityBindingV0 {
  readonly localName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
  readonly classnamesImportName: string;
}

interface CanonicalBindingStyleImportV0 {
  readonly localName: string;
  readonly styleUri: string;
}

interface CanonicalDeclaresStyleImportV0 {
  readonly declName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

interface CanonicalStyleImportResolvesModuleV0 {
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

interface CanonicalExpressionTargetsModuleV0 {
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly targetStyleUri: string;
}

interface CanonicalClassUtilBindingV0 {
  readonly localName: string;
}

interface CanonicalStyleAccessUsesStyleImportV0 {
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly declName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

interface CanonicalSymbolRefUsesDeclV0 {
  readonly byteSpan: {
    readonly start: number;
    readonly end: number;
  };
  readonly rawReference: string;
  readonly rootName: string;
  readonly declName: string;
  readonly styleUri: string;
}

interface CanonicalUtilityUsesStyleImportV0 {
  readonly utilityLocalName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
}

interface CanonicalDeclaresUtilityBindingV0 {
  readonly declName: string;
  readonly utilityLocalName: string;
  readonly utilityKind: "classnamesBind" | "classUtil";
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
const minimumCrossLanguageFixtureCount = 4;

const fixtures: readonly FrontendFixtureV0[] = [
  {
    id: "css-modules-cx-style-access-flow",
    sourcePath: "/fake/ws/src/Card.tsx",
    stylePath: "/fake/ws/src/Card.module.scss",
    selectorNames: ["card", "card--active", "tone-warm", "tone-cool", "icon"],
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
    stylePath: "/fake/ws/src/Panel.module.scss",
    selectorNames: ["panel", "panel--enabled", "panel__local", "tone-info", "tone-warn", "title"],
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
    stylePath: "/fake/ws/src/Nav.module.scss",
    selectorNames: ["nav", "nav--active", "nav__item", "nav__label"],
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
  {
    id: "css-modules-loop-logical-flow",
    sourcePath: "/fake/ws/src/Toolbar.tsx",
    stylePath: "/fake/ws/src/Toolbar.module.scss",
    selectorNames: ["toolbar", "toolbar--active", "toolbar__icon"],
    cfgReferenceToken: "size",
    cfgVariableName: "size",
    source: [
      'import bind from "classnames/bind";',
      'import clsx from "clsx";',
      'import styles from "./Toolbar.module.scss";',
      "const cx = bind.bind(styles);",
      "export function Toolbar({ active }: { active: boolean }) {",
      '  let size = "toolbar";',
      "  while (active) {",
      '    size = active && "toolbar--active";',
      "    break;",
      "  }",
      '  return <div className={clsx(cx("toolbar", size), styles.toolbar__icon)} />;',
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
  reports.every((report) => report.syntax.status === "green"),
  "syntax oracle must be green before retiring the TypeScript syntax path",
);
assert.ok(
  reports.every((report) => report.syntax.allFieldsMatch),
  `syntax oracle must match all covered fields without recorded gaps: ${JSON.stringify(reports, null, 2)}`,
);
assert.ok(
  reports.some((report) =>
    report.syntax.coveredFields.some((field) => field.field === "symbolRefSelectorReferences"),
  ),
  "at least one fixture must promote symbolRef selector projection into covered fields",
);
assert.ok(
  reports.every((report) => report.binding.coveredFieldsMatch),
  `covered source binding fields must match: ${JSON.stringify(reports, null, 2)}`,
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "classnamesBindUtilityBindings"),
  ),
  "at least one fixture must promote classnames/bind utility projection into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "classUtilBindings"),
  ),
  "at least one fixture must promote class utility binding nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "styleImportBindings"),
  ),
  "at least one fixture must promote style import binding nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "graphNodeKeys"),
  ),
  "at least one fixture must promote canonical binding graph node keys into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "graphEdgeKeys"),
  ),
  "at least one fixture must promote canonical binding graph edge keys into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "bindingDecls"),
  ),
  "at least one fixture must promote declaration nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "bindingScopes"),
  ),
  "at least one fixture must promote scope nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "scopeParentEdges"),
  ),
  "at least one fixture must promote scope parent edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "scopeContainsDecls"),
  ),
  "at least one fixture must promote scope declaration containment edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "styleModules"),
  ),
  "at least one fixture must promote style module nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "classExpressionNodes"),
  ),
  "at least one fixture must promote class expression nodes into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "declaresStyleImports"),
  ),
  "at least one fixture must promote style import declaration edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "styleImportResolvesModules"),
  ),
  "at least one fixture must promote style import resolution edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "expressionTargetsModules"),
  ),
  "at least one fixture must promote expression target module edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "styleAccessUsesStyleImports"),
  ),
  "at least one fixture must promote style access declaration edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "symbolRefUsesDecls"),
  ),
  "at least one fixture must promote symbol reference declaration edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "utilityUsesStyleImports"),
  ),
  "at least one fixture must promote utility-to-style-import edges into covered binding fields",
);
assert.ok(
  reports.some((report) =>
    report.binding.coveredFields.some((field) => field.field === "declaresUtilityBindings"),
  ),
  "at least one fixture must promote utility declaration edges into covered binding fields",
);
assert.ok(
  reports.every((report) => report.binding.status === "green"),
  "binding graph oracle must be green before retiring the TypeScript binding path",
);
assert.ok(
  reports.every((report) => report.binding.allFieldsMatch),
  `binding graph oracle must match all covered fields: ${JSON.stringify(reports, null, 2)}`,
);
assert.ok(
  reports.every((report) => report.cfg.status === "green"),
  "CFG oracle must be green before retiring the TypeScript flow path",
);
assert.ok(
  reports.every((report) => report.cfg.coveredFieldsMatch),
  `covered CFG fields must match: ${JSON.stringify(reports, null, 2)}`,
);
assert.ok(
  reports.some((report) =>
    report.cfg.coveredFields.some((field) => field.field === "blockGraphSnapshot"),
  ),
  "at least one fixture must promote Rust CFG block graph snapshots into covered fields",
);
assert.ok(
  reports.every((report) =>
    report.cfg.coveredFields.some(
      (field) => field.field === "typeFactControlFlowGraphContract" && field.matches,
    ),
  ),
  "every fixture must prove Rust CFG projection matches the product TypeFact CFG contract",
);
assert.ok(
  captures.some((capture) =>
    capture.cfgSnapshot?.snapshot.blocks.some((block) => block.kind === "loopHeader"),
  ),
  "cross-language CFG corpus must include a loop header",
);
assert.ok(
  captures.some((capture) =>
    capture.cfgSnapshot?.snapshot.blocks.some((block) => block.kind === "break"),
  ),
  "cross-language CFG corpus must include a break block",
);
assert.ok(
  captures.some((capture) =>
    capture.cfgSnapshot?.snapshot.blocks.some((block) => block.kind === "logicalOperand"),
  ),
  "cross-language CFG corpus must include a short-circuit operand block",
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

function captureFixture(fixture: FrontendFixtureV0): FixtureCaptureV0 {
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
  const sourceBindingGraph = composeSourceBindingGraph(sourceDocument, sourceBinder);
  const styleDocument = styleDocumentForFixture(fixture);
  const cfgReferenceRange = rangeForToken(sourceFile, fixture.cfgReferenceToken);
  const capture = captureTsSourceFrontendFactsV0({
    sourceFile,
    sourceBinder,
    sourceDocument,
    sourceBindingGraph,
    semantic: {
      styleDocumentForPath: (path) => (path === fixture.stylePath ? styleDocument : null),
      typeResolver: new UnresolvableTypeResolver(),
      filePath: fixture.sourcePath,
      workspaceRoot,
      resolveSymbolValues: (expression) =>
        resolveFlowClassValues(sourceFile, expression.range, expression.rootName),
    },
    cfg: {
      variableName: fixture.cfgVariableName,
      referenceRange: cfgReferenceRange,
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
      cfgVariableName: capture.cfgSnapshot?.variableName,
      cfgReferenceByteOffset: capture.cfgSnapshot?.referenceByteOffset,
    },
  };
}

function captureRustSyntax(fixtureCaptures: readonly FixtureCaptureV0[]): RustCaptureResponseV0 {
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

function compareFixture(tsCapture: FixtureCaptureV0, rustCaptureResponse: RustCaptureResponseV0) {
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
  const symbolSelectorField = fieldReport(
    "symbolRefSelectorReferences",
    tsCapture.syntax.symbolSelectorReferences,
    rustSymbolSelectorReferences,
  );
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
    ...(symbolSelectorField.matches ? [symbolSelectorField] : []),
  ];
  const recordedGaps = symbolSelectorField.matches
    ? []
    : [
        {
          field: "symbolRefSelectorReferences",
          status: "recorded-red",
          reason:
            "Rust source syntax does not yet match the TS semantic selector projection for this symbol reference.",
          tsJson: stringifyCanonicalSourceFrontendJsonV0(tsCapture.syntax.symbolSelectorReferences),
          rustJson: stringifyCanonicalSourceFrontendJsonV0(rustSymbolSelectorReferences),
        },
      ];
  return {
    fixture: fixtureId(tsCapture),
    syntax: {
      status:
        fields.every((field) => field.matches) && recordedGaps.length === 0
          ? "green"
          : "partial-green",
      coveredFields: fields,
      coveredFieldsMatch: fields.every((field) => field.matches),
      allFieldsMatch: fields.every((field) => field.matches) && recordedGaps.length === 0,
      recordedGaps,
    },
    binding: compareBindingProjection(tsCapture, rustCapture),
    cfg: compareCfgProjection(tsCapture, rustCapture),
  };
}

function compareCfgProjection(tsCapture: FixtureCaptureV0, rustCapture: RustFixtureCaptureV0) {
  const fields = [
    fieldReport(
      "reference",
      cfgReferenceForTsCapture(tsCapture),
      cfgReferenceForRustCapture(rustCapture),
    ),
    fieldReport(
      "blockGraphSnapshot",
      canonicalCfgSnapshot(tsCapture.cfgSnapshot?.snapshot ?? null),
      canonicalCfgSnapshot(rustCapture.cfgSnapshot?.snapshot ?? null),
    ),
    fieldReport(
      "typeFactControlFlowGraphContract",
      canonicalCfgSnapshot(rustCapture.cfgSnapshot?.snapshot ?? null),
      canonicalCfgSnapshot(rustCapture.cfgProductContract),
    ),
    fieldReport(
      "typeFactControlFlowGraphSymbolOrdinals",
      cfgSymbolOrdinals(rustCapture.cfgSnapshot?.snapshot ?? null),
      cfgSymbolOrdinals(rustCapture.cfgProductContract),
    ),
  ];
  const allFieldsMatch = fields.every((field) => field.matches);
  return {
    status: allFieldsMatch ? "green" : "partial-green",
    coveredFields: fields,
    coveredFieldsMatch: allFieldsMatch,
    allFieldsMatch,
    recordedGaps: allFieldsMatch
      ? []
      : fields
          .filter((field) => !field.matches)
          .map((field) => ({
            field: field.field,
            status: "recorded-red",
            reason: "Rust CFG projection does not match the captured TypeScript CFG field.",
          })),
  };
}

function compareBindingProjection(tsCapture: FixtureCaptureV0, rustCapture: RustFixtureCaptureV0) {
  const projectedRustCapture = captureProjectedRustBindingGraph(tsCapture, rustCapture);
  const fields = [
    fieldReport(
      "graphNodeKeys",
      tsCapture.bindingGraph.graphNodeKeys,
      rustBindingGraphNodeKeys(rustCapture.binding),
    ),
    fieldReport(
      "graphEdgeKeys",
      tsCapture.bindingGraph.graphEdgeKeys,
      rustBindingGraphEdgeKeys(rustCapture.binding),
    ),
    fieldReport(
      "projectedGraphNodeKeys",
      tsCapture.bindingGraph.graphNodeKeys,
      projectedRustCapture.bindingGraph.graphNodeKeys,
    ),
    fieldReport(
      "projectedGraphEdgeKeys",
      tsCapture.bindingGraph.graphEdgeKeys,
      projectedRustCapture.bindingGraph.graphEdgeKeys,
    ),
    fieldReport(
      "bindingScopes",
      tsCapture.bindingGraph.bindingScopes,
      rustCapture.binding.bindingScopes.toSorted(compareByStableJson),
    ),
    fieldReport(
      "scopeParentEdges",
      tsCapture.bindingGraph.scopeParentEdges,
      rustCapture.binding.scopeParentEdges.toSorted(compareByStableJson),
    ),
    fieldReport(
      "bindingDecls",
      tsCapture.bindingGraph.bindingDecls,
      rustCapture.binding.bindingDecls.toSorted(compareByStableJson),
    ),
    fieldReport(
      "scopeContainsDecls",
      tsCapture.bindingGraph.scopeContainsDecls,
      rustCapture.binding.scopeContainsDecls.toSorted(compareByStableJson),
    ),
    fieldReport(
      "styleModules",
      tsCapture.bindingGraph.styleModules,
      rustCapture.binding.styleModules.toSorted(compareByStableJson),
    ),
    fieldReport(
      "classExpressionNodes",
      tsCapture.bindingGraph.classExpressionNodes,
      rustCapture.binding.classExpressionNodes.toSorted(compareByStableJson),
    ),
    fieldReport(
      "styleImportBindings",
      styleImportBindingsForTsCapture(tsCapture),
      rustCapture.binding.styleImportBindings.toSorted(compareByStableJson),
    ),
    fieldReport(
      "declaresStyleImports",
      declaresStyleImportsForTsCapture(tsCapture),
      rustCapture.binding.declaresStyleImports.toSorted(compareByStableJson),
    ),
    fieldReport(
      "styleImportResolvesModules",
      styleImportResolvesModulesForTsCapture(tsCapture),
      rustCapture.binding.styleImportResolvesModules.toSorted(compareByStableJson),
    ),
    fieldReport(
      "expressionTargetsModules",
      tsCapture.bindingGraph.expressionTargetsModules,
      rustCapture.binding.expressionTargetsModules.toSorted(compareByStableJson),
    ),
    fieldReport(
      "styleAccessUsesStyleImports",
      tsCapture.bindingGraph.styleAccessUsesStyleImports,
      rustCapture.binding.styleAccessUsesStyleImports.toSorted(compareByStableJson),
    ),
    fieldReport(
      "symbolRefUsesDecls",
      tsCapture.bindingGraph.symbolRefUsesDecls,
      rustCapture.binding.symbolRefUsesDecls.toSorted(compareByStableJson),
    ),
    fieldReport(
      "classnamesBindUtilityBindings",
      classnamesBindUtilityBindingsForTsCapture(tsCapture),
      rustCapture.binding.classnamesBindUtilityBindings.toSorted(compareByStableJson),
    ),
    fieldReport(
      "classUtilBindings",
      tsCapture.bindingGraph.classUtilBindings,
      rustCapture.binding.classUtilBindings.toSorted(compareByStableJson),
    ),
    fieldReport(
      "declaresUtilityBindings",
      declaresUtilityBindingsForTsCapture(tsCapture),
      rustCapture.binding.declaresUtilityBindings.toSorted(compareByStableJson),
    ),
    fieldReport(
      "utilityUsesStyleImports",
      utilityUsesStyleImportsForTsCapture(tsCapture),
      rustCapture.binding.utilityUsesStyleImports.toSorted(compareByStableJson),
    ),
  ];
  const allFieldsMatch = fields.every((field) => field.matches);
  return {
    status: allFieldsMatch ? "green" : "partial-green",
    coveredFields: fields,
    coveredFieldsMatch: allFieldsMatch,
    allFieldsMatch,
    recordedGaps: allFieldsMatch
      ? []
      : fields
          .filter((field) => !field.matches)
          .map((field) => ({
            field: field.field,
            status: "recorded-red",
            reason:
              "Rust binding graph projection does not match the captured TypeScript binding field.",
            tsNodeCount: tsCapture.bindingGraph.nodes.length,
            tsEdgeCount: tsCapture.bindingGraph.edges.length,
          })),
  };
}

function captureProjectedRustBindingGraph(
  tsCapture: FixtureCaptureV0,
  rustCapture: RustFixtureCaptureV0,
): CanonicalSourceFrontendCaptureV0 {
  const fixture = fixtures.find((candidate) => candidate.id === fixtureId(tsCapture));
  assert.ok(fixture, `missing fixture for ${fixtureId(tsCapture)}`);
  const sourceFile = ts.createSourceFile(
    fixture.sourcePath,
    fixture.source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const projected = projectRustSourceBindingIndexV0({
    filePath: fixture.sourcePath,
    source: fixture.source,
    language: sourceLanguageForFixture(fixture),
    index: rustCapture.binding satisfies RustSourceBindingIndexV0,
  });
  return captureTsSourceFrontendFactsV0({
    sourceFile,
    sourceBinder: projected.sourceBinder,
    sourceDocument: projected.sourceDocument,
    sourceBindingGraph: projected.sourceBindingGraph,
  });
}

function sourceLanguageForFixture(fixture: FrontendFixtureV0): SourceLanguage {
  return fixture.sourceLanguage === "typescript" || fixture.sourcePath.endsWith(".ts")
    ? "typescript"
    : "typescriptreact";
}

function rustBindingGraphNodeKeys(binding: RustFixtureCaptureV0["binding"]): readonly string[] {
  return [
    ...binding.bindingScopes.map((scope) =>
      graphKey({
        kind: "scope",
        scopeKind: scope.kind,
        byteSpan: scope.byteSpan,
      }),
    ),
    ...binding.bindingDecls.map(rustDeclKey),
    ...binding.styleImportBindings.map((styleImport) =>
      rustStyleImportKey(styleImport.localName, styleImport.styleUri),
    ),
    ...binding.classnamesBindUtilityBindings.map(rustClassnamesBindUtilityKey),
    ...binding.classUtilBindings.map(rustClassUtilKey),
    ...binding.classExpressionNodes.map(rustExpressionKey),
    ...binding.styleModules.map((styleModule) => rustStyleModuleKey(styleModule.styleUri)),
  ].toSorted();
}

function rustBindingGraphEdgeKeys(binding: RustFixtureCaptureV0["binding"]): readonly string[] {
  const declByName = new Map(binding.bindingDecls.map((decl) => [decl.name, rustDeclKey(decl)]));
  const expressionBySpanAndUri = new Map(
    binding.classExpressionNodes.map((expression) => [
      `${spanKey(expression.byteSpan)}:${expression.targetStyleUri}`,
      rustExpressionKey(expression),
    ]),
  );
  const classnamesBindByLocalName = new Map(
    binding.classnamesBindUtilityBindings.map((utility) => [
      utility.localName,
      rustClassnamesBindUtilityKey(utility),
    ]),
  );
  const classUtilByLocalName = new Map(
    binding.classUtilBindings.map((utility) => [utility.localName, rustClassUtilKey(utility)]),
  );
  const edges = [
    ...binding.scopeParentEdges.map((edge) =>
      graphEdgeKey(
        "scopeParent",
        rustScopeKey(edge.childKind, edge.childByteSpan),
        rustScopeKey(edge.parentKind, edge.parentByteSpan),
      ),
    ),
    ...binding.scopeContainsDecls.map((edge) =>
      graphEdgeKey(
        "scopeContainsDecl",
        rustScopeKey(edge.scopeKind, edge.scopeByteSpan),
        rustDeclKey({
          kind: edge.declKind,
          name: edge.declName,
          byteSpan: edge.declByteSpan,
          importPath: edge.importPath,
        }),
      ),
    ),
    ...binding.declaresStyleImports.map((edge) =>
      graphEdgeKey(
        "declaresStyleImport",
        declByName.get(edge.declName) ?? missingGraphNodeKey("decl", edge.declName),
        rustStyleImportKey(edge.stylesLocalName, edge.styleUri),
      ),
    ),
    ...binding.styleImportResolvesModules.map((edge) =>
      graphEdgeKey(
        "styleImportResolvesModule",
        rustStyleImportKey(edge.stylesLocalName, edge.styleUri),
        rustStyleModuleKey(edge.styleUri),
      ),
    ),
    ...binding.expressionTargetsModules.map((edge) => {
      const expression =
        expressionBySpanAndUri.get(`${spanKey(edge.byteSpan)}:${edge.targetStyleUri}`) ??
        missingGraphNodeKey("expression", `${spanKey(edge.byteSpan)}:${edge.targetStyleUri}`);
      return graphEdgeKey(
        "expressionTargetsModule",
        expression,
        rustStyleModuleKey(edge.targetStyleUri),
      );
    }),
    ...binding.styleAccessUsesStyleImports.map((edge) => {
      const expression =
        expressionBySpanAndUri.get(`${spanKey(edge.byteSpan)}:${edge.styleUri}`) ??
        missingGraphNodeKey("expression", `${spanKey(edge.byteSpan)}:${edge.styleUri}`);
      return graphEdgeKey(
        "expressionUsesDecl",
        expression,
        declByName.get(edge.declName) ?? missingGraphNodeKey("decl", edge.declName),
      );
    }),
    ...binding.symbolRefUsesDecls.map((edge) => {
      const expression =
        expressionBySpanAndUri.get(`${spanKey(edge.byteSpan)}:${edge.styleUri}`) ??
        missingGraphNodeKey("expression", `${spanKey(edge.byteSpan)}:${edge.styleUri}`);
      return graphEdgeKey(
        "expressionUsesDecl",
        expression,
        declByName.get(edge.declName) ?? missingGraphNodeKey("decl", edge.declName),
      );
    }),
    ...binding.declaresUtilityBindings.map((edge) => {
      const utility =
        edge.utilityKind === "classnamesBind"
          ? classnamesBindByLocalName.get(edge.utilityLocalName)
          : classUtilByLocalName.get(edge.utilityLocalName);
      return graphEdgeKey(
        "declaresUtilityBinding",
        declByName.get(edge.declName) ?? missingGraphNodeKey("decl", edge.declName),
        utility ?? missingGraphNodeKey("utilityBinding", edge.utilityLocalName),
      );
    }),
    ...binding.utilityUsesStyleImports.map((edge) =>
      graphEdgeKey(
        "utilityUsesStyleImport",
        classnamesBindByLocalName.get(edge.utilityLocalName) ??
          missingGraphNodeKey("utilityBinding", edge.utilityLocalName),
        rustStyleImportKey(edge.stylesLocalName, edge.styleUri),
      ),
    ),
  ];
  return edges.toSorted();
}

function cfgReferenceForTsCapture(capture: CanonicalSourceFrontendCaptureV0) {
  assert.ok(capture.cfgSnapshot, `${fixtureId(capture)} must have a TS CFG snapshot`);
  return {
    variableName: capture.cfgSnapshot.variableName,
    referenceByteOffset: capture.cfgSnapshot.referenceByteOffset,
  };
}

function cfgReferenceForRustCapture(capture: RustFixtureCaptureV0) {
  assert.ok(capture.cfgSnapshot, `${capture.id} must have a Rust CFG snapshot`);
  return {
    variableName: capture.cfgSnapshot.variableName,
    referenceByteOffset: capture.cfgSnapshot.referenceByteOffset,
  };
}

function canonicalCfgSnapshot(
  snapshot:
    | NonNullable<CanonicalSourceFrontendCaptureV0["cfgSnapshot"]>["snapshot"]
    | RustSourceControlFlowGraphCaptureV0["snapshot"]
    | TypeFactControlFlowGraphV2
    | null,
) {
  if (!snapshot) return null;
  return {
    entryBlockId: snapshot.entryBlockId,
    blocks: snapshot.blocks.map(canonicalCfgBlock),
  };
}

function canonicalCfgBlock(
  block:
    | NonNullable<CanonicalSourceFrontendCaptureV0["cfgSnapshot"]>["snapshot"]["blocks"][number]
    | RustFlowBlockSnapshotV0
    | TypeFactControlFlowGraphV2["blocks"][number],
) {
  return {
    id: block.id,
    kind: block.kind,
    transferKind: block.transferKind,
    successorBlockIds: [...block.successorBlockIds],
    ...(block.variableName ? { variableName: block.variableName } : {}),
    ...(block.expressionKind ? { expressionKind: block.expressionKind } : {}),
  };
}

function cfgSymbolOrdinals(
  snapshot:
    | RustSourceControlFlowGraphCaptureV0["snapshot"]
    | TypeFactControlFlowGraphV2
    | null
    | undefined,
) {
  if (!snapshot) return null;
  return snapshot.blocks.map((block) => ({
    id: block.id,
    symbolOrdinal: block.symbolOrdinal ?? null,
  }));
}

function rustScopeKey(kind: string, byteSpan: CanonicalBindingScopeV0["byteSpan"]): string {
  return graphKey({
    kind: "scope",
    scopeKind: kind,
    byteSpan,
  });
}

function rustDeclKey(decl: {
  readonly kind: string;
  readonly name: string;
  readonly byteSpan: CanonicalBindingDeclV0["byteSpan"];
  readonly importPath?: string;
}): string {
  return graphKey({
    kind: "decl",
    declKind: decl.kind,
    name: decl.name,
    byteSpan: decl.byteSpan,
    ...(decl.importPath ? { importPath: decl.importPath } : {}),
  });
}

function rustStyleImportKey(localName: string, styleUri: string): string {
  return graphKey({
    kind: "styleImport",
    localName,
    styleUri,
  });
}

function rustStyleModuleKey(styleUri: string): string {
  return graphKey({
    kind: "styleModule",
    styleUri,
  });
}

function rustExpressionKey(expression: CanonicalClassExpressionNodeV0): string {
  return graphKey({
    kind: "expression",
    expressionKind: expression.kind,
    byteSpan: expression.byteSpan,
    targetStyleUri: expression.targetStyleUri,
  });
}

function rustClassnamesBindUtilityKey(binding: CanonicalClassnamesBindUtilityBindingV0): string {
  return graphKey({
    kind: "utilityBinding",
    utilityKind: "classnamesBind",
    localName: binding.localName,
    stylesLocalName: binding.stylesLocalName,
    styleUri: binding.styleUri,
    classnamesImportName: binding.classnamesImportName,
  });
}

function rustClassUtilKey(binding: CanonicalClassUtilBindingV0): string {
  return graphKey({
    kind: "utilityBinding",
    utilityKind: "classUtil",
    localName: binding.localName,
  });
}

function graphEdgeKey(kind: string, from: string, to: string): string {
  return `${kind}:${from}->${to}`;
}

function missingGraphNodeKey(kind: string, value: string): string {
  return graphKey({
    kind: "missing",
    nodeKind: kind,
    value,
  });
}

function graphKey(value: Record<string, unknown>): string {
  return stringifyCanonicalSourceFrontendJsonV0(value);
}

function styleImportBindingsForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalBindingStyleImportV0[] {
  return capture.bindingGraph.nodes
    .flatMap((node) =>
      node.kind === "styleImport"
        ? [
            {
              localName: node.styleImport.localName,
              styleUri: styleImportUri(node.styleImport.resolved),
            },
          ]
        : [],
    )
    .toSorted(compareByStableJson);
}

function declaresStyleImportsForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalDeclaresStyleImportV0[] {
  const nodes = new Map(capture.bindingGraph.nodes.map((node) => [node.id, node]));
  return capture.bindingGraph.edges
    .flatMap((edge) => {
      if (edge.kind !== "declaresStyleImport") return [];
      const declNode = nodes.get(edge.from);
      const styleImportNode = nodes.get(edge.to);
      if (declNode?.kind !== "decl" || styleImportNode?.kind !== "styleImport") return [];
      return [
        {
          declName: declNode.decl.name,
          stylesLocalName: styleImportNode.styleImport.localName,
          styleUri: styleImportUri(styleImportNode.styleImport.resolved),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function styleImportResolvesModulesForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalStyleImportResolvesModuleV0[] {
  const nodes = new Map(capture.bindingGraph.nodes.map((node) => [node.id, node]));
  return capture.bindingGraph.edges
    .flatMap((edge) => {
      if (edge.kind !== "styleImportResolvesModule") return [];
      const styleImportNode = nodes.get(edge.from);
      const styleModuleNode = nodes.get(edge.to);
      if (styleImportNode?.kind !== "styleImport" || styleModuleNode?.kind !== "styleModule") {
        return [];
      }
      return [
        {
          stylesLocalName: styleImportNode.styleImport.localName,
          styleUri: fileUriForAbsolutePath(styleModuleNode.scssModulePath),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function classnamesBindUtilityBindingsForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalClassnamesBindUtilityBindingV0[] {
  return capture.syntax.utilityBindings
    .flatMap((binding) =>
      binding.kind === "classnamesBind"
        ? [
            {
              localName: binding.localName,
              stylesLocalName: binding.stylesLocalName,
              styleUri: fileUriForAbsolutePath(binding.scssModulePath),
              classnamesImportName: binding.classNamesImportName,
            },
          ]
        : [],
    )
    .toSorted(compareByStableJson);
}

function declaresUtilityBindingsForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalDeclaresUtilityBindingV0[] {
  const nodes = new Map(capture.bindingGraph.nodes.map((node) => [node.id, node]));
  return capture.bindingGraph.edges
    .flatMap((edge) => {
      if (edge.kind !== "declaresUtilityBinding") return [];
      const declNode = nodes.get(edge.from);
      const utilityNode = nodes.get(edge.to);
      if (declNode?.kind !== "decl" || utilityNode?.kind !== "utilityBinding") {
        return [];
      }
      return [
        {
          declName: declNode.decl.name,
          utilityLocalName: utilityNode.utilityBinding.localName,
          utilityKind: utilityNode.utilityBinding.kind,
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function utilityUsesStyleImportsForTsCapture(
  capture: CanonicalSourceFrontendCaptureV0,
): readonly CanonicalUtilityUsesStyleImportV0[] {
  const nodes = new Map(capture.bindingGraph.nodes.map((node) => [node.id, node]));
  return capture.bindingGraph.edges
    .flatMap((edge) => {
      if (edge.kind !== "utilityUsesStyleImport") return [];
      const utilityNode = nodes.get(edge.from);
      const styleImportNode = nodes.get(edge.to);
      if (
        utilityNode?.kind !== "utilityBinding" ||
        styleImportNode?.kind !== "styleImport" ||
        utilityNode.utilityBinding.kind !== "classnamesBind"
      ) {
        return [];
      }
      return [
        {
          utilityLocalName: utilityNode.utilityBinding.localName,
          stylesLocalName: styleImportNode.styleImport.localName,
          styleUri: styleImportUri(styleImportNode.styleImport.resolved),
        },
      ];
    })
    .toSorted(compareByStableJson);
}

function styleImportUri(
  styleImport: SourceDocumentHIR["styleImports"][number]["resolved"],
): string {
  return styleImport.kind === "resolved"
    ? fileUriForAbsolutePath(styleImport.absolutePath)
    : `missing:${styleImport.specifier}`;
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

function styleDocumentForFixture(fixture: FrontendFixtureV0): StyleDocumentHIR {
  return makeStyleDocumentHIR(
    fixture.stylePath,
    fixture.selectorNames.map((name, index) => selectorForFixture(name, index)),
  );
}

function selectorForFixture(name: string, index: number): SelectorDeclHIR {
  const line = index + 1;
  return {
    kind: "selector",
    id: `selector:${index}:${name}`,
    range: {
      start: { line, character: 1 },
      end: { line, character: 1 + name.length },
    },
    name,
    canonicalName: name,
    viewKind: "canonical",
    fullSelector: `.${name}`,
    declarations: "color: red",
    ruleRange: {
      start: { line, character: 0 },
      end: { line, character: name.length + 12 },
    },
    composes: [],
    nestedSafety: "flat",
  };
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

function fileUriForAbsolutePath(path: string): string {
  return path.startsWith("file://") ? path : `file://${path}`;
}
