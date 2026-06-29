import { describe, expect, it, vi } from "vitest";
import type ts from "typescript";
import { DiagnosticSeverity, type Diagnostic } from "vscode-languageserver-protocol/node";
import type { Range } from "@omena/shared";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import type { ResolvedCxBinding } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { computeDiagnostics } from "../../../server/lsp-server/src/providers/diagnostics";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import { SELECTED_QUERY_RUNNER_COMMANDS } from "../../../server/engine-host-node/src/selected-query-backend";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../server/engine-host-node/src/selected-query-backend";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";
import { documentFixture, workspace, type CmeWorkspace } from "../../../packages/vitest-cme/src";

const SOURCE_PATH = "/fake/ws/src/Button.tsx";
const SOURCE_URI = "file:///fake/ws/src/Button.tsx";
const TSX_WORKSPACE = workspace({
  [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const a = cx('/*<indicator>*/indicator/*</indicator>*/');
const b = cx('/*<missing>*/unknonw/*</missing>*/');
`,
});
const CX_BINDING_RANGE = TSX_WORKSPACE.range("binding", SOURCE_PATH).range;
const INDICATOR_RANGE = TSX_WORKSPACE.range("indicator", SOURCE_PATH).range;
const MISSING_CLASS_RANGE = TSX_WORKSPACE.range("missing", SOURCE_PATH).range;
type TestClassExpression = Parameters<typeof buildTestClassExpressions>[0]["expressions"][number];

function stableDiagnosticSnapshot(diagnostics: readonly Diagnostic[]): string {
  return JSON.stringify(
    [...diagnostics]
      .map((diagnostic) => ({
        code: diagnostic.code,
        severity: diagnostic.severity,
        source: diagnostic.source,
        message: diagnostic.message,
        range: diagnostic.range,
        tags: diagnostic.tags,
        data: diagnostic.data,
      }))
      .toSorted((left, right) => String(left.code).localeCompare(String(right.code))),
    null,
    2,
  );
}

const SOURCE_QUERY_PROVENANCE = [
  "omena-query.source-syntax-index",
  "omena-query.style-selector-definitions",
  "omena-query-checker-orchestrator.product-diagnostic-gate",
  "omena-checker.rule-registry",
] as const;

function sourceQueryDiagnosticMessage(
  code:
    | "missingModule"
    | "missingStaticClass"
    | "missingTemplatePrefix"
    | "missingResolvedClassValues"
    | "missingResolvedClassDomain",
): string {
  switch (code) {
    case "missingModule":
      return "Cannot resolve CSS Module './Missing.module.scss'. The file does not exist.";
    case "missingStaticClass":
      return "Class '.unknonw' not found in target CSS Module. Did you mean 'unknown'?";
    case "missingTemplatePrefix":
      return "No class starting with 'unknonw' found in target CSS Module.";
    case "missingResolvedClassValues":
      return "Missing class for possible value: 'unknonw'.";
    case "missingResolvedClassDomain":
      return "No class matched resolved prefix 'unknonw'.";
  }
}

function sourceQueryDiagnosticPrecision(
  code:
    | "missingModule"
    | "missingStaticClass"
    | "missingTemplatePrefix"
    | "missingResolvedClassValues"
    | "missingResolvedClassDomain",
) {
  return {
    product: "omena-query.analysis-precision",
    valueDomain: code === "missingModule" ? "styleModuleResolution" : "classValueResolution",
    flowSensitivity:
      code === "missingModule" ? "sourceImportResolution" : "sourceSelectorReference",
    contextSensitivity:
      code === "missingResolvedClassValues" || code === "missingResolvedClassDomain"
        ? "resolvedClassValueDomain"
        : code === "missingModule"
          ? "perImportSpecifier"
          : "perSourceReference",
    revisionAxis: "OmenaQuerySourceDiagnosticsForFileV0.input",
  };
}

function selectedQuerySourceMergedOutputReferenceDiagnostics(
  codes: readonly (
    | "missingModule"
    | "missingStaticClass"
    | "missingTemplatePrefix"
    | "missingResolvedClassValues"
    | "missingResolvedClassDomain"
  )[],
): readonly Diagnostic[] {
  return codes.map((code) => ({
    range: MISSING_CLASS_RANGE,
    severity: DiagnosticSeverity.Warning,
    code,
    source: "omena-css",
    message: sourceQueryDiagnosticMessage(code),
    data: {
      querySeverity: "warning",
      provenance: SOURCE_QUERY_PROVENANCE,
      precision: sourceQueryDiagnosticPrecision(code),
      ...(code === "missingStaticClass"
        ? {
            createSelector: {
              uri: "file:///fake/ws/src/Button.module.scss",
              range: {
                start: { line: 1, character: 0 },
                end: { line: 1, character: 0 },
              },
              newText: "\n\n.unknonw {\n}\n",
              selectorName: "unknonw",
            },
          }
        : {}),
    },
  }));
}

const detectCxBindings = (_sourceFile: ts.SourceFile): CxBinding[] => [
  {
    cxVarName: "cx",
    stylesVarName: "styles",
    scssModulePath: "/fake/ws/src/Button.module.scss",
    classNamesImportName: "classNames",
    bindingRange: CX_BINDING_RANGE,
  },
];

const parseClassExpressions = (_sf: ts.SourceFile, bindings: readonly ResolvedCxBinding[]) =>
  buildTestClassExpressions({
    filePath: SOURCE_PATH,
    bindings,
    expressions:
      bindings.length === 0
        ? []
        : [
            {
              kind: "literal",
              origin: "cxCall",
              className: "indicator",
              range: INDICATOR_RANGE,
              scssModulePath: bindings[0]!.scssModulePath,
            },
            {
              kind: "literal",
              origin: "cxCall",
              className: "unknonw",
              range: MISSING_CLASS_RANGE,
              scssModulePath: bindings[0]!.scssModulePath,
            },
          ],
  });

function styleDocumentForSelectors(selectors: ReadonlyMap<string, ReturnType<typeof info>>) {
  return () => buildStyleDocumentFromSelectorMap("/fake/ws/src/Button.module.scss", selectors);
}

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf) }),
    parseClassExpressions,
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator")],
        ["unknown", info("unknown")], // nearby typo target
      ]),
    ...overrides,
  });
}

function diagnosticParams(testWorkspace: CmeWorkspace = TSX_WORKSPACE) {
  return documentFixture({
    workspace: testWorkspace,
    filePath: SOURCE_PATH,
    documentUri: SOURCE_URI,
  });
}

function makeExpressionDeps(
  expressionFactory: (bindings: readonly ResolvedCxBinding[]) => TestClassExpression[],
  selectors: ReadonlyMap<string, ReturnType<typeof info>>,
  typeResolver: TypeResolver = new FakeTypeResolver(),
): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({
      stylesBindings: new Map(),
      bindings: detectCxBindings(sf),
    }),
    parseClassExpressions: (_sf: ts.SourceFile, bindings: readonly ResolvedCxBinding[]) =>
      buildTestClassExpressions({
        filePath: SOURCE_PATH,
        bindings,
        expressions: bindings.length === 0 ? [] : expressionFactory(bindings),
      }),
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    styleDocumentForPath: styleDocumentForSelectors(selectors),
    typeResolver,
  });
}

describe("computeDiagnostics", () => {
  const baseParams = diagnosticParams();

  it("returns an empty array when all classes resolve", () => {
    const deps = makeDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", info("indicator")],
          ["unknonw", info("unknonw")],
        ]),
    });
    const result = computeDiagnostics(baseParams, deps);
    expect(result).toEqual([]);
  });

  it("does not fall back to checker diagnostics when selected-query has no runner", () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    try {
      expect(computeDiagnostics(baseParams, makeDeps())).toEqual([]);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("warns on a missing static class with a did-you-mean hint", () => {
    const result = computeDiagnostics(baseParams, makeDeps());
    expect(result).toHaveLength(1);
    const d = result[0]!;
    expect(d.severity).toBe(DiagnosticSeverity.Warning);
    expect(d.message).toContain("'.unknonw'");
    expect(d.message).toContain("Did you mean 'unknown'?");
    expect(d.data).toMatchObject({
      suggestion: "unknown",
      createSelector: {
        uri: "file:///fake/ws/src/Button.module.scss",
        newText: "\n\n.unknonw {\n}\n",
      },
    });
  });

  it("uses omena-query source diagnostics with Rust-owned quick-fix data", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    const BUFFER_SCSS = ".indicator {}\n";
    const DISK_SCSS = ".indicator {}\n.legacy {}\n";
    const commands: string[] = [];
    const runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync = async (
      command,
      input,
    ) => {
      commands.push(command);
      expect(command).toBe(SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile);
      expect(input).toMatchObject({
        sourcePath: SOURCE_PATH,
        sourceSource: baseParams.content,
        styles: [{ stylePath: "/fake/ws/src/Button.module.scss", styleSource: BUFFER_SCSS }],
      });
      return {
        product: "omena-query.diagnostics-for-file",
        fileKind: "source",
        diagnostics: [
          {
            code: "missingStaticClass",
            severity: "warning",
            provenance: [
              "omena-query.source-syntax-index",
              "omena-query.style-selector-definitions",
            ],
            range: MISSING_CLASS_RANGE,
            message: "Class '.unknonw' not found in target CSS Module. Did you mean 'unknown'?",
            suggestion: "unknown",
            createSelector: {
              uri: "/fake/ws/src/Button.module.scss",
              range: {
                start: { line: 1, character: 0 },
                end: { line: 1, character: 0 },
              },
              newText: "\n\n.unknonw {\n}\n",
              selectorName: "unknonw",
            },
          },
          {
            code: "missingResolvedClassValues",
            severity: "warning",
            provenance: [
              "omena-query.source-syntax-index",
              "omena-query.style-selector-definitions",
            ],
            range: INDICATOR_RANGE,
            message: "Missing class for possible value: 'indicator'.",
          },
          {
            code: "missingStaticClass",
            severity: "warning",
            provenance: [
              "omena-query.source-syntax-index",
              "omena-query.style-selector-definitions",
            ],
            range: {
              start: { line: 99, character: 0 },
              end: { line: 99, character: 10 },
            },
            message: "Class '.directOnly' not found in target CSS Module.",
            createSelector: {
              uri: "/fake/ws/src/Button.module.scss",
              range: {
                start: { line: 1, character: 0 },
                end: { line: 1, character: 0 },
              },
              newText: "\n\n.directOnly {\n}\n",
              selectorName: "directOnly",
            },
          },
        ],
      };
    };
    const deps = {
      ...makeDeps({
        readOpenDocumentText: (filePath) =>
          filePath === "/fake/ws/src/Button.module.scss" ? BUFFER_SCSS : null,
        readStyleFile: () => DISK_SCSS,
      }),
      runRustSelectedQueryBackendJsonAsync,
    };

    try {
      const result = await computeDiagnostics(baseParams, deps);
      expect(commands).toEqual([SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile]);
      expect(result).toHaveLength(2);
      const missingStaticClass = result.find(
        (diagnostic) => diagnostic.code === "missingStaticClass",
      );
      expect(missingStaticClass).toMatchObject({
        code: "missingStaticClass",
        severity: DiagnosticSeverity.Warning,
        data: {
          suggestion: "unknown",
          querySeverity: "warning",
          provenance: ["omena-query.source-syntax-index", "omena-query.style-selector-definitions"],
          createSelector: {
            uri: "file:///fake/ws/src/Button.module.scss",
            selectorName: "unknonw",
          },
        },
      });
      expect(missingStaticClass?.message).toContain("Class '.unknonw'");
      expect(missingStaticClass?.message).toContain("Did you mean 'unknown'?");
      expect(
        result.find((diagnostic) => diagnostic.code === "missingResolvedClassValues"),
      ).toMatchObject({
        code: "missingResolvedClassValues",
        severity: DiagnosticSeverity.Warning,
        message: "Missing class for possible value: 'indicator'.",
        data: {
          querySeverity: "warning",
          provenance: ["omena-query.source-syntax-index", "omena-query.style-selector-definitions"],
        },
      });
      expect(result.some((diagnostic) => diagnostic.message.includes("directOnly"))).toBe(false);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("snapshots the selected-query source merged-output oracle for every diagnostic code", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    const queryCodes = [
      "missingModule",
      "missingStaticClass",
      "missingTemplatePrefix",
      "missingResolvedClassValues",
      "missingResolvedClassDomain",
    ] as const;
    const runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync = async <
      T,
    >() =>
      ({
        product: "omena-query.diagnostics-for-file",
        fileKind: "source",
        diagnostics: queryCodes.map((code) => ({
          code,
          severity: "warning",
          provenance: SOURCE_QUERY_PROVENANCE,
          precision: sourceQueryDiagnosticPrecision(code),
          range: MISSING_CLASS_RANGE,
          message: sourceQueryDiagnosticMessage(code),
          ...(code === "missingStaticClass"
            ? {
                createSelector: {
                  uri: "/fake/ws/src/Button.module.scss",
                  range: {
                    start: { line: 1, character: 0 },
                    end: { line: 1, character: 0 },
                  },
                  newText: "\n\n.unknonw {\n}\n",
                  selectorName: "unknonw",
                },
              }
            : {}),
        })),
      }) as T;

    try {
      const diagnostics = await computeDiagnostics(baseParams, {
        ...makeDeps(),
        runRustSelectedQueryBackendJsonAsync,
      });

      expect(diagnostics.map((diagnostic) => diagnostic.code).toSorted()).toEqual(
        [...queryCodes].toSorted(),
      );
      for (const diagnostic of diagnostics) {
        expect(diagnostic.severity).toBe(DiagnosticSeverity.Warning);
        expect(diagnostic.data).toMatchObject({
          querySeverity: "warning",
          provenance: SOURCE_QUERY_PROVENANCE,
          precision: {
            product: "omena-query.analysis-precision",
            revisionAxis: "OmenaQuerySourceDiagnosticsForFileV0.input",
          },
        });
      }
      expect(stableDiagnosticSnapshot(diagnostics)).toEqual(
        stableDiagnosticSnapshot(selectedQuerySourceMergedOutputReferenceDiagnostics(queryCodes)),
      );
      expect(stableDiagnosticSnapshot(diagnostics)).toMatchInlineSnapshot(`
        "[
          {
            "code": "missingModule",
            "severity": 2,
            "source": "omena-css",
            "message": "Cannot resolve CSS Module './Missing.module.scss'. The file does not exist.",
            "range": {
              "start": {
                "line": 5,
                "character": 14
              },
              "end": {
                "line": 5,
                "character": 21
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-syntax-index",
                "omena-query.style-selector-definitions",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "precision": {
                "product": "omena-query.analysis-precision",
                "valueDomain": "styleModuleResolution",
                "flowSensitivity": "sourceImportResolution",
                "contextSensitivity": "perImportSpecifier",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
              }
            }
          },
          {
            "code": "missingResolvedClassDomain",
            "severity": 2,
            "source": "omena-css",
            "message": "No class matched resolved prefix 'unknonw'.",
            "range": {
              "start": {
                "line": 5,
                "character": 14
              },
              "end": {
                "line": 5,
                "character": 21
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-syntax-index",
                "omena-query.style-selector-definitions",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "precision": {
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSelectorReference",
                "contextSensitivity": "resolvedClassValueDomain",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
              }
            }
          },
          {
            "code": "missingResolvedClassValues",
            "severity": 2,
            "source": "omena-css",
            "message": "Missing class for possible value: 'unknonw'.",
            "range": {
              "start": {
                "line": 5,
                "character": 14
              },
              "end": {
                "line": 5,
                "character": 21
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-syntax-index",
                "omena-query.style-selector-definitions",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "precision": {
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSelectorReference",
                "contextSensitivity": "resolvedClassValueDomain",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
              }
            }
          },
          {
            "code": "missingStaticClass",
            "severity": 2,
            "source": "omena-css",
            "message": "Class '.unknonw' not found in target CSS Module. Did you mean 'unknown'?",
            "range": {
              "start": {
                "line": 5,
                "character": 14
              },
              "end": {
                "line": 5,
                "character": 21
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-syntax-index",
                "omena-query.style-selector-definitions",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "precision": {
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSelectorReference",
                "contextSensitivity": "perSourceReference",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
              },
              "createSelector": {
                "uri": "file:///fake/ws/src/Button.module.scss",
                "range": {
                  "start": {
                    "line": 1,
                    "character": 0
                  },
                  "end": {
                    "line": 1,
                    "character": 0
                  }
                },
                "newText": "\\n\\n.unknonw {\\n}\\n",
                "selectorName": "unknonw"
              }
            }
          },
          {
            "code": "missingTemplatePrefix",
            "severity": 2,
            "source": "omena-css",
            "message": "No class starting with 'unknonw' found in target CSS Module.",
            "range": {
              "start": {
                "line": 5,
                "character": 14
              },
              "end": {
                "line": 5,
                "character": 21
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-syntax-index",
                "omena-query.style-selector-definitions",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "precision": {
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSelectorReference",
                "contextSensitivity": "perSourceReference",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
              }
            }
          }
        ]"
      `);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("does not fall back to checker diagnostics in the selected-query source path", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    let runnerCalled = false;
    const runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync = async <
      T,
    >() => {
      runnerCalled = true;
      return {
        product: "omena-query.diagnostics-for-file",
        fileKind: "source",
        diagnostics: [],
      } as T;
    };

    try {
      const diagnostics = await computeDiagnostics(baseParams, {
        ...makeDeps(),
        runRustSelectedQueryBackendJsonAsync,
      });

      expect(runnerCalled).toBe(true);
      expect(diagnostics).toEqual([]);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("returns an empty array when the file does not import classnames/bind", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({ stylesBindings: new Map(), bindings: [] }),
    });
    const result = computeDiagnostics(
      { ...baseParams, content: "const x = 1;\n", filePath: "/fake/ws/src/Plain.tsx", version: 2 },
      makeDeps({
        analysisCache: new DocumentAnalysisCache({
          sourceFileCache,
          sourceFrontendAnalysis,
          fileExists: () => true,
          aliasResolver: EMPTY_ALIAS_RESOLVER,
          max: 10,
        }),
      }),
    );
    expect(result).toEqual([]);
  });

  it("isolates per-call exceptions — one throw does not erase other diagnostics", () => {
    const logError = vi.fn();
    const result = computeDiagnostics(
      baseParams,
      makeDeps({
        styleDocumentForPath: () => {
          throw new Error("boom");
        },
        logError,
      }),
    );
    // Both cx() calls throw, so we get no diagnostics but TWO
    // isolated log entries — NOT a single "abort everything"
    // entry. A single bad call must not silently drop every
    // other diagnostic in the same document.
    expect(result).toEqual([]);
    expect(logError).toHaveBeenCalledTimes(2);
    expect(logError).toHaveBeenCalledWith(
      "diagnostics per-call validation failed",
      expect.any(Error),
    );
  });

  it("warns on a template-literal call whose prefix matches nothing", () => {
    const templateWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const x = "value";
const a = cx(/*<template>*/\`prefix-\${x}\`/*</template>*/);
`,
    });
    const expressionRange = templateWorkspace.range("template", SOURCE_PATH).range;
    const deps = makeExpressionDeps(
      (bindings) => [
        {
          kind: "template",
          origin: "cxCall",
          rawTemplate: "prefix-${x}",
          staticPrefix: "prefix-",
          range: expressionRange,
          scssModulePath: bindings[0]!.scssModulePath,
        },
      ],
      new Map([
        ["indicator", info("indicator")],
        ["active", info("active")],
      ]),
    );
    const result = computeDiagnostics(diagnosticParams(templateWorkspace), deps);
    expect(result).toHaveLength(1);
    expect(result[0]!.message).toContain("No class starting with 'prefix-'");
  });

  it("warns on a variable call whose union has a missing member", () => {
    const unionWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = chooseSize();
const a = cx(/*<size>*/size/*</size>*/);
`,
    });
    const expressionRange = unionWorkspace.range("size", SOURCE_PATH).range;
    // Union has three values but classMap only has two of them.
    class UnionResolver implements TypeResolver {
      resolve(_filePath?: string, _variableName?: string, _workspaceRoot?: string, _range?: Range) {
        return { kind: "union" as const, values: ["small", "medium", "large"] as const };
      }
      invalidate() {}
      clear() {}
    }
    const deps = makeExpressionDeps(
      (bindings) => [
        {
          kind: "symbolRef",
          origin: "cxCall",
          rawReference: "size",
          range: expressionRange,
          scssModulePath: bindings[0]!.scssModulePath,
        },
      ],
      new Map([
        ["small", info("small")],
        ["medium", info("medium")],
      ]),
      new UnionResolver(),
    );
    const result = computeDiagnostics(diagnosticParams(unionWorkspace), deps);
    expect(result).toHaveLength(1);
    expect(result[0]!.message).toContain("Missing class for union member");
    expect(result[0]!.message).toContain("'large'");
    expect(result[0]!.message).toContain(
      "Analysis reason: TypeScript exposed multiple string-literal candidates.",
    );
    expect(result[0]!.message).toContain("Analysis shape: bounded finite (3).");
  });

  it("warns on a variable call when local flow resolves a missing value", () => {
    const flowWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = enabled ? 'small' : 'large';
const a = cx(/*<size>*/size/*</size>*/);
`,
    });
    const expressionRange = flowWorkspace.range("size", SOURCE_PATH).range;
    const deps = makeExpressionDeps(
      (bindings) => [
        {
          kind: "symbolRef",
          origin: "cxCall",
          rawReference: "size",
          range: expressionRange,
          scssModulePath: bindings[0]!.scssModulePath,
        },
      ],
      new Map([["small", info("small")]]),
    );
    const result = computeDiagnostics(diagnosticParams(flowWorkspace), deps);
    expect(result).toHaveLength(1);
    expect(result[0]!.message).toContain("Missing class for possible value");
    expect(result[0]!.message).toContain("'large'");
    expect(result[0]!.message).toContain(
      "Analysis reason: analysis preserved multiple finite candidate values.",
    );
    expect(result[0]!.message).toContain("Analysis shape: bounded finite (2).");
  });

  it("skips variable calls with an unresolvable type (ignoreUnresolvableUnions)", () => {
    const unresolvedWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const unknown = getClassName();
const a = cx(/*<unknown>*/unknown/*</unknown>*/);
`,
    });
    const expressionRange = unresolvedWorkspace.range("unknown", SOURCE_PATH).range;
    const deps = makeExpressionDeps(
      (bindings) => [
        {
          kind: "symbolRef",
          origin: "cxCall",
          rawReference: "unknown",
          range: expressionRange,
          scssModulePath: bindings[0]!.scssModulePath,
        },
      ],
      new Map([["indicator", info("indicator")]]),
    );
    const result = computeDiagnostics(diagnosticParams(unresolvedWorkspace), deps);
    expect(result).toEqual([]);
  });

  it("keeps clean diagnostics when one call throws — per-call isolation", () => {
    const logError = vi.fn();
    // Throw only for 'unknonw', succeed for 'indicator'.
    let callCount = 0;
    const result = computeDiagnostics(
      baseParams,
      makeDeps({
        selectorMapForPath: () => {
          callCount += 1;
          if (callCount === 2) throw new Error("only the second one");
          return new Map([["indicator", info("indicator")]]);
        },
        logError,
      }),
    );
    // 'indicator' resolved cleanly → zero diagnostics for it.
    // 'unknonw' threw → isolated, logged, does not erase the
    // rest. Final diagnostics is [] because 'indicator' was
    // clean and 'unknonw' was dropped by the catch. The win is
    // that the throw didn't propagate outward.
    expect(result).toEqual([]);
    expect(logError).toHaveBeenCalledTimes(1);
  });
});

// ── missing-module diagnostics ───────────────────────────────

describe("missing-module diagnostics", () => {
  const MISSING_SOURCE_PATH = "/fake/ws/src/App.tsx";
  const MISSING_SOURCE_URI = "file:///fake/ws/src/App.tsx";
  const MISSING_WORKSPACE = workspace({
    [MISSING_SOURCE_PATH]:
      "import styles from /*<module>*/'./typo.module.scss/*</module>*/';\nconst a = styles.foo;\n",
  });
  const missingParams = documentFixture({
    workspace: MISSING_WORKSPACE,
    filePath: MISSING_SOURCE_PATH,
    documentUri: MISSING_SOURCE_URI,
  });

  function makeMissingDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
    const missingModuleRange = MISSING_WORKSPACE.range("module", MISSING_SOURCE_PATH).range;
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => false,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({
        stylesBindings: new Map([
          [
            "styles",
            {
              kind: "missing" as const,
              absolutePath: "/fake/ws/src/typo.module.scss",
              specifier: "./typo.module.scss",
              range: missingModuleRange,
            },
          ],
        ]),
        bindings: [],
      }),
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      sourceFrontendAnalysis,
      fileExists: () => false,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    return makeBaseDeps({ analysisCache, workspaceRoot: "/fake/ws", ...overrides });
  }

  it("emits one diagnostic per missing import with code 'missing-module'", () => {
    const deps = makeMissingDeps();
    const missingModuleRange = MISSING_WORKSPACE.range("module", MISSING_SOURCE_PATH).range;
    const result = computeDiagnostics(missingParams, deps);
    expect(result).toHaveLength(1);
    expect(result[0]!.code).toBe("missing-module");
    expect(result[0]!.message).toContain("./typo.module.scss");
    expect(result[0]!.range.start).toEqual(missingModuleRange.start);
    expect(result[0]!.range.end).toEqual(missingModuleRange.end);
    expect(result[0]!.data).toEqual({
      createModuleFile: {
        uri: "file:///fake/ws/src/typo.module.scss",
      },
    });
  });

  it("uses query-owned missing-module diagnostics in the selected-query path", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    const missingModuleRange = MISSING_WORKSPACE.range("module", MISSING_SOURCE_PATH).range;
    const commands: string[] = [];
    const runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync = async (
      command,
      input,
    ) => {
      commands.push(command);
      expect(command).toBe(SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile);
      expect(input).toMatchObject({
        sourcePath: MISSING_SOURCE_PATH,
        sourceSource: missingParams.content,
        styles: [],
      });
      return {
        product: "omena-query.diagnostics-for-file",
        fileKind: "source",
        diagnostics: [
          {
            code: "missingModule",
            severity: "warning",
            provenance: [
              "omena-query.source-import-declarations",
              "omena-resolver.style-module-resolution",
              "omena-query-checker-orchestrator.product-diagnostic-gate",
              "omena-checker.rule-registry",
            ],
            range: missingModuleRange,
            message: "Cannot resolve CSS Module './typo.module.scss'. The file does not exist.",
          },
        ],
      };
    };

    try {
      const result = await computeDiagnostics(missingParams, {
        ...makeMissingDeps(),
        runRustSelectedQueryBackendJsonAsync,
      });
      expect(commands).toEqual([SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile]);
      expect(result).toHaveLength(1);
      expect(result[0]).toMatchObject({
        code: "missingModule",
        message: "Cannot resolve CSS Module './typo.module.scss'. The file does not exist.",
        data: {
          querySeverity: "warning",
          provenance: [
            "omena-query.source-import-declarations",
            "omena-resolver.style-module-resolution",
            "omena-query-checker-orchestrator.product-diagnostic-gate",
            "omena-checker.rule-registry",
          ],
          createModuleFile: {
            uri: "file:///fake/ws/src/typo.module.scss",
          },
        },
      });
      expect(stableDiagnosticSnapshot(result)).toMatchInlineSnapshot(`
        "[
          {
            "code": "missingModule",
            "severity": 2,
            "source": "omena-css",
            "message": "Cannot resolve CSS Module './typo.module.scss'. The file does not exist.",
            "range": {
              "start": {
                "line": 0,
                "character": 19
              },
              "end": {
                "line": 0,
                "character": 38
              }
            },
            "data": {
              "querySeverity": "warning",
              "provenance": [
                "omena-query.source-import-declarations",
                "omena-resolver.style-module-resolution",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry"
              ],
              "createModuleFile": {
                "uri": "file:///fake/ws/src/typo.module.scss"
              }
            }
          }
        ]"
      `);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("does not emit when diagnostics.missingModule is false", () => {
    const deps = makeMissingDeps({
      settings: {
        ...makeBaseDeps().settings,
        diagnostics: {
          ...makeBaseDeps().settings.diagnostics,
          missingModule: false,
        },
      },
    });
    const result = computeDiagnostics(missingParams, deps);
    expect(result).toEqual([]);
  });

  it("does not emit missing-module for a resolved import", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({
        stylesBindings: new Map([
          [
            "styles",
            { kind: "resolved" as const, absolutePath: "/fake/ws/src/Button.module.scss" },
          ],
        ]),
        bindings: [],
      }),
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const deps = makeBaseDeps({ analysisCache });
    const result = computeDiagnostics(
      {
        documentUri: "file:///fake/ws/src/App.tsx",
        content: "import styles from './Button.module.scss';\n",
        filePath: "/fake/ws/src/App.tsx",
        version: 1,
      },
      deps,
    );
    const missing = result.filter((d) => d.code === "missing-module");
    expect(missing).toEqual([]);
  });

  it("missing-module check fires on pure styles.x access without a classnames/bind import", () => {
    // The fixture deliberately omits `classnames/bind` so the
    // only hook for the missing-module loop is the `styles.x`
    // property access. Pins that the loop does NOT gate on a
    // `classnames/bind` token being present in the file, so
    // plain CSS Modules consumers still get diagnostics.
    const PURE_STYLES_TSX = `import styles from './typo.module.scss';\nexport const A = () => styles.a;\n`;
    const deps = makeMissingDeps();
    const result = computeDiagnostics(
      {
        documentUri: "file:///fake/ws/src/App.tsx",
        content: PURE_STYLES_TSX,
        filePath: "/fake/ws/src/App.tsx",
        version: 1,
      },
      deps,
    );
    expect(result).toHaveLength(1);
    expect(result[0]!.code).toBe("missing-module");
  });

  it("returns empty for a file with no style imports at all", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({ stylesBindings: new Map(), bindings: [] }),
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const deps = makeBaseDeps({ analysisCache });
    const result = computeDiagnostics(
      {
        documentUri: "file:///fake/ws/src/App.tsx",
        content: "import React from 'react';\nexport const A = () => null;\n",
        filePath: "/fake/ws/src/App.tsx",
        version: 1,
      },
      deps,
    );
    expect(result).toEqual([]);
  });
});
