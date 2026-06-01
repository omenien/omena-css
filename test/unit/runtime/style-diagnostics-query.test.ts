import { describe, expect, it } from "vitest";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { resolveStyleDiagnosticFindings } from "../../../server/engine-host-node/src/style-diagnostics-query";
import type {
  StyleSemanticGraphCssModulesCrossFileResolutionV0,
  StyleSemanticGraphSummaryV0,
} from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import {
  buildStyleDocumentFromSelectorMap,
  makeStyleDocumentFixture,
  makeTestSelector,
} from "../../_fixtures/style-documents";

describe("resolveStyleDiagnosticFindings", () => {
  it("returns style checker findings through the host boundary", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 5, scssPath),
    ]);

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      { semanticReferenceIndex },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "unused-selector",
    });
  });

  it("can source unused-selector findings from rust selector-usage payloads", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake",
    });

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: (_args, _deps, _filePath, canonicalName) =>
          canonicalName === "indicator"
            ? {
                canonicalName,
                totalReferences: 2,
                directReferenceCount: 1,
                editableDirectReferenceCount: 1,
                exactReferenceCount: 1,
                inferredOrBetterReferenceCount: 2,
                hasExpandedReferences: true,
                hasStyleDependencyReferences: false,
                hasAnyReferences: true,
              }
            : {
                canonicalName,
                totalReferences: 0,
                directReferenceCount: 0,
                editableDirectReferenceCount: 0,
                exactReferenceCount: 0,
                inferredOrBetterReferenceCount: 0,
                hasExpandedReferences: false,
                hasStyleDependencyReferences: false,
                hasAnyReferences: false,
              },
      },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "unused-selector",
      canonicalName: "active",
    });
  });

  it("can source unused-selector findings from rust style semantic graph references", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake",
    });

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeReferenceGraph(scssPath),
        readRustSelectorUsagePayloadForWorkspaceTarget: () => {
          throw new Error("unexpected selector-usage fallback");
        },
      },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "unused-selector",
      canonicalName: "active",
    });
  });

  it("forwards precomputed workspace inputs to rust style semantic graph diagnostics", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake",
    });
    const sourceDocuments = [
      {
        uri: "file:///fake/App.tsx",
        filePath: "/fake/App.tsx",
        content: "const app = true;",
        version: 1,
      },
    ];
    const styleFiles = [scssPath];
    const styleSemanticGraphCache = new Map();
    let forwardedOptions:
      | {
          readonly sourceDocuments?: readonly unknown[];
          readonly styleFiles?: readonly string[];
          readonly styleSemanticGraphCache?: unknown;
        }
      | undefined;

    resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        sourceDocuments,
        styleFiles,
        styleSemanticGraphCache,
        readRustStyleSemanticGraphForWorkspaceTarget: (_args, _deps, _filePath, options) => {
          forwardedOptions = options;
          return makeReferenceGraph(scssPath);
        },
        readRustSelectorUsagePayloadForWorkspaceTarget: () => {
          throw new Error("unexpected selector-usage fallback");
        },
      },
    );

    expect(forwardedOptions?.sourceDocuments).toBe(sourceDocuments);
    expect(forwardedOptions?.styleFiles).toBe(styleFiles);
    expect(forwardedOptions?.styleSemanticGraphCache).toBe(styleSemanticGraphCache);
  });

  it("uses the runtime style semantic graph cache when no query cache is provided", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake",
    });
    const styleSemanticGraphCache = new Map();
    let forwardedOptions:
      | {
          readonly styleSemanticGraphCache?: unknown;
        }
      | undefined;

    resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
        styleSemanticGraphCache,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: (_args, _deps, _filePath, options) => {
          forwardedOptions = options;
          return makeReferenceGraph(scssPath);
        },
        readRustSelectorUsagePayloadForWorkspaceTarget: () => {
          throw new Error("unexpected selector-usage fallback");
        },
      },
    );

    expect(forwardedOptions?.styleSemanticGraphCache).toBe(styleSemanticGraphCache);
  });

  it("forwards the runtime selector-usage payload cache to rust unused-selector diagnostics", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = buildStyleDocumentFromSelectorMap(
      scssPath,
      new Map([["indicator", infoAtLine("indicator", 1)]]),
    );
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const selectorUsagePayloadCache = new Map();
    let forwardedCache: unknown = null;

    resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
        selectorUsagePayloadCache,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadsForWorkspaceTarget: (_args, _deps, _filePath, cache) => {
          forwardedCache = cache;
          return [
            {
              kind: "selector-usage",
              filePath: scssPath,
              queryId: "indicator",
              payload: {
                canonicalName: "indicator",
                totalReferences: 0,
                directReferenceCount: 0,
                editableDirectReferenceCount: 0,
                exactReferenceCount: 0,
                inferredOrBetterReferenceCount: 0,
                hasExpandedReferences: false,
                hasStyleDependencyReferences: false,
                hasAnyReferences: false,
                allSites: [],
                editableDirectSites: [],
              },
            },
          ];
        },
      },
    );

    expect(forwardedCache).toBe(selectorUsagePayloadCache);
  });

  it("does not fall back to current unused-selector diagnostics when rust deps are incomplete", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = makeStyleDocumentFixture(scssPath, [
      makeTestSelector("indicator", 1),
      makeTestSelector("active", 3),
      makeTestSelector("composed", 5, {
        composes: [{ classNames: ["missing"], from: "./Other.module.scss" }],
      }),
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 5, scssPath),
    ]);

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        semanticReferenceIndex,
        styleDocumentForPath: () => null,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "missing-composed-module",
    });
  });

  it("suppresses missing custom-property findings when rust design-token ranking has a winner", () => {
    const scssPath = "/fake/Button.module.scss";
    const styleDocument = parseStyleDocument(
      `:root {
  --other: #fff;
  color: var(--brand);
}`,
      scssPath,
    );
    const deps = makeBaseDeps({ workspaceRoot: "/fake" });

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex: deps.semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () =>
          makeDesignTokenDefinitionGraph({
            referenceName: "--brand",
            winnerDeclarationFilePath: "/fake/theme.module.scss",
            winnerDeclarationRange: {
              start: { line: 0, character: 8 },
              end: { line: 0, character: 15 },
            },
            stylePath: scssPath,
          }),
      },
    );

    expect(findings.filter((finding) => finding.code === "missing-custom-property")).toEqual([]);
  });

  it("suppresses CSS Modules missing findings when rust relation resolution matched the import", () => {
    const scssPath = "/fake/Button.module.scss";
    const basePath = "/fake/Base.module.scss";
    const styleDocument = parseStyleDocument(
      `.button {
  composes: base from "./Base.module.scss";
}`,
      scssPath,
    );
    const staleBaseDocument = parseStyleDocument(".other { color: blue; }", basePath);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/App.tsx", [
      semanticSiteAt("file:///fake/App.tsx", "button", 10, scssPath),
    ]);
    const deps = makeBaseDeps({
      readStyleFile: (filePath) =>
        filePath === scssPath
          ? `.button { composes: base from "./Base.module.scss"; }`
          : filePath === basePath
            ? ".base { color: blue; }"
            : null,
      styleDocumentForPath: (filePath) =>
        filePath === scssPath ? styleDocument : filePath === basePath ? staleBaseDocument : null,
      workspaceRoot: "/fake",
    });

    const findings = resolveStyleDiagnosticFindings(
      { scssPath, styleDocument },
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        semanticReferenceIndex,
        styleDependencyGraph: deps.styleDependencyGraph,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
      },
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        sourceDocuments: [],
        styleFiles: [scssPath, basePath],
        runRustSelectedQueryBackendJson: <T>(command: string): T => {
          if (command !== "style-semantic-graph-batch") {
            throw new Error(`unexpected runner command: ${command}`);
          }
          return {
            schemaVersion: "0",
            product: "omena-semantic.style-semantic-graph-batch",
            cssModulesResolution: makeCssModulesResolution({
              fromStylePath: scssPath,
              source: "./Base.module.scss",
              resolvedStylePath: basePath,
              matchedNames: ["base"],
            }),
            graphs: [],
          } as T;
        },
      },
    );

    expect(findings.filter((finding) => finding.code === "missing-composed-selector")).toEqual([]);
  });
});

function makeCssModulesResolution(args: {
  readonly fromStylePath: string;
  readonly source: string;
  readonly resolvedStylePath: string;
  readonly matchedNames: readonly string[];
}): StyleSemanticGraphCssModulesCrossFileResolutionV0 {
  return {
    schemaVersion: "0",
    product: "omena-query.css-modules-cross-file-resolution",
    status: "icssExportImportClosureSeed",
    resolutionScope: "batchImportGraph",
    styleCount: 2,
    importEdgeCount: 1,
    resolvedImportEdgeCount: 1,
    unresolvedImportEdgeCount: 0,
    matchedNameCount: args.matchedNames.length,
    edges: [
      {
        fromStylePath: args.fromStylePath,
        importKind: "composes",
        source: args.source,
        resolvedStylePath: args.resolvedStylePath,
        status: "resolved",
        importGraphDistance: 1,
        importGraphOrder: 0,
        importedNames: ["base"],
        exportedNames: ["base"],
        matchedNames: args.matchedNames,
      },
    ],
    composesClosureEdgeCount: 1,
    valueClosureEdgeCount: 0,
    icssClosureEdgeCount: 0,
    composesCycleCount: 0,
    valueCycleCount: 0,
    icssCycleCount: 0,
    composesClosureEdges: [
      {
        fromStylePath: args.fromStylePath,
        ownerSelectorName: "button",
        targetStylePath: args.resolvedStylePath,
        targetSelectorName: "base",
        depth: 1,
        path: [`${args.fromStylePath}#button`, `${args.resolvedStylePath}#base`],
      },
    ],
    valueClosureEdges: [],
    icssClosureEdges: [],
    cycles: [],
    capabilities: {
      importSourceResolutionReady: true,
      composesNameMatchReady: true,
      valueNameMatchReady: true,
      icssNameMatchReady: true,
      transitiveClosureReady: true,
      valueGraphClosureReady: true,
      icssExportImportClosureReady: true,
      cycleDetectionReady: true,
    },
    nextPriorities: [],
  };
}

function makeReferenceGraph(stylePath: string): StyleSemanticGraphSummaryV0 {
  return {
    schemaVersion: "0",
    product: "omena-semantic.style-semantic-graph",
    language: "scss",
    parserFacts: {},
    semanticFacts: {},
    selectorIdentityEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-identity",
      canonicalIdCount: 2,
      canonicalIds: [
        {
          canonicalId: "selector:indicator",
          localName: "indicator",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
        {
          canonicalId: "selector:active",
          localName: "active",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: true,
        safeCanonicalIds: ["selector:indicator", "selector:active"],
        blockedCanonicalIds: [],
        blockers: [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath,
      selectorCount: 2,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 1,
      totalReferenceSites: 1,
      selectors: [
        makeSelectorReferenceSummary(stylePath, "indicator", true),
        makeSelectorReferenceSummary(stylePath, "active", false),
      ],
    },
    sourceInputEvidence: {},
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

function makeSelectorReferenceSummary(
  stylePath: string,
  localName: string,
  hasAnyReferences: boolean,
) {
  const referenceCount = hasAnyReferences ? 1 : 0;
  return {
    canonicalId: `selector:${localName}`,
    filePath: stylePath,
    localName,
    totalReferences: referenceCount,
    directReferenceCount: referenceCount,
    editableDirectReferenceCount: referenceCount,
    exactReferenceCount: referenceCount,
    inferredOrBetterReferenceCount: referenceCount,
    hasExpandedReferences: false,
    hasStyleDependencyReferences: false,
    hasAnyReferences,
    sites: hasAnyReferences
      ? [
          {
            filePath: "/fake/App.tsx",
            range: {
              start: { line: 8, character: 10 },
              end: { line: 8, character: 19 },
            },
            expansion: "direct",
            referenceKind: "source",
          },
        ]
      : [],
    editableDirectSites: hasAnyReferences
      ? [
          {
            filePath: "/fake/App.tsx",
            range: {
              start: { line: 8, character: 10 },
              end: { line: 8, character: 19 },
            },
            className: localName,
          },
        ]
      : [],
  };
}
