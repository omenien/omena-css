import { describe, expect, it } from "vitest";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import {
  resolveStyleReferenceLenses,
  resolveStyleReferenceLensesAsync,
} from "../../../server/engine-host-node/src/style-reference-lens-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  return makeBaseDeps({
    selectorMapForPath: () =>
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

describe("resolveStyleReferenceLenses", () => {
  it("returns an empty list when no selector has references", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    expect(
      resolveStyleReferenceLenses("/fake/src/Button.module.scss", styleDocument, makeDeps()),
    ).toEqual([]);
  });

  it("returns code-lens summaries for referenced selectors", () => {
    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    );

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps({ semanticReferenceIndex: idx }),
    );

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
      position: { line: 5, character: 1 },
      title: "1 reference",
    });
    expect(result[0]!.locations).toHaveLength(1);
    expect(result[0]!.locations[0]!.uri).toBe("file:///a.tsx");
  });

  it("uses rust selector-usage payloads for the lens title summary and locations", () => {
    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    );

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps({ semanticReferenceIndex: idx }),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: (_args, _deps, _filePath, canonicalName) =>
          canonicalName === "indicator"
            ? {
                canonicalName: "indicator",
                totalReferences: 4,
                directReferenceCount: 2,
                editableDirectReferenceCount: 1,
                exactReferenceCount: 1,
                inferredOrBetterReferenceCount: 3,
                hasExpandedReferences: true,
                hasStyleDependencyReferences: true,
                hasAnyReferences: true,
                allSites: [
                  {
                    filePath: "/fake/src/App.tsx",
                    range: {
                      start: { line: 12, character: 8 },
                      end: { line: 12, character: 17 },
                    },
                    expansion: "direct",
                    referenceKind: "source",
                  },
                  {
                    filePath: "/fake/src/Other.module.scss",
                    range: {
                      start: { line: 3, character: 1 },
                      end: { line: 3, character: 10 },
                    },
                    expansion: "direct",
                    referenceKind: "styleDependency",
                  },
                ],
              }
            : null,
      },
    );

    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("4 references (2 direct, composed, dynamic)");
    expect(result[0]?.locations).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 12, character: 8 },
          end: { line: 12, character: 17 },
        },
      },
      {
        uri: "file:///fake/src/Other.module.scss",
        range: {
          start: { line: 3, character: 1 },
          end: { line: 3, character: 10 },
        },
      },
    ]);
  });

  it("reads rust selector-usage payloads once for all selectors in a style file", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    );
    const deps = makeDeps();
    let payloadReads = 0;

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      deps,
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadsForWorkspaceTarget: () => {
          payloadReads += 1;
          return [
            makeSelectorUsageCandidate("indicator", true),
            makeSelectorUsageCandidate("active", false),
          ];
        },
      },
    );

    expect(payloadReads).toBe(1);
    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("1 reference");
  });

  it("forwards the shared selector-usage payload cache to rust payload readers", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );
    const selectorUsagePayloadCache = new Map();
    let forwardedCache: unknown = null;

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps(),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        selectorUsagePayloadCache,
        readRustSelectorUsagePayloadsForWorkspaceTarget: (_args, _deps, _filePath, cache) => {
          forwardedCache = cache;
          return [makeSelectorUsageCandidate("indicator", true)];
        },
      },
    );

    expect(forwardedCache).toBe(selectorUsagePayloadCache);
    expect(result).toHaveLength(1);
  });

  it("uses rust style semantic graph references for lens title summary and locations", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps(),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeReferenceGraph(),
      },
    );

    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("3 references (1 direct, dynamic)");
    expect(result[0]?.locations).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 12, character: 8 },
          end: { line: 12, character: 17 },
        },
      },
    ]);
  });

  it("reads rust style semantic graph references once for all selectors in a style file", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    );
    let graphReads = 0;

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps(),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => {
          graphReads += 1;
          return makeReferenceGraphWithSelectors(["indicator", "active"]);
        },
        readRustSelectorUsagePayloadsForWorkspaceTarget: () => [],
      },
    );

    expect(graphReads).toBe(1);
    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("3 references (1 direct, dynamic)");
  });

  it("reads async rust style semantic graph references once for all selectors in a style file", async () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([
        ["indicator", infoAtLine("indicator", 5)],
        ["active", infoAtLine("active", 10)],
      ]),
    );
    let graphReads = 0;

    const result = await resolveStyleReferenceLensesAsync(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps(),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async () => {
          graphReads += 1;
          return makeReferenceGraphWithSelectors(["indicator", "active"]);
        },
        readRustSelectorUsagePayloadsForWorkspaceTargetAsync: async () => [],
      },
    );

    expect(graphReads).toBe(1);
    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("3 references (1 direct, dynamic)");
  });

  it("rechecks rust selector-usage when rust graph has an empty selector reference summary", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferenceLenses(
      "/fake/src/Button.module.scss",
      styleDocument,
      makeDeps(),
      {
        env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeEmptyReferenceGraph(),
        readRustSelectorUsagePayloadsForWorkspaceTarget: () => [
          makeSelectorUsageCandidate("indicator", true),
        ],
      },
    );

    expect(result).toHaveLength(1);
    expect(result[0]?.title).toBe("1 reference");
    expect(result[0]?.locations).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 12, character: 8 },
          end: { line: 12, character: 17 },
        },
      },
    ]);
  });
});

function makeReferenceGraph(): StyleSemanticGraphSummaryV0 {
  return {
    schemaVersion: "0",
    product: "omena-semantic.style-semantic-graph",
    language: "scss",
    parserFacts: {},
    semanticFacts: {},
    selectorIdentityEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-identity",
      canonicalIdCount: 1,
      canonicalIds: [
        {
          canonicalId: "selector:indicator",
          localName: "indicator",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: true,
        safeCanonicalIds: ["selector:indicator"],
        blockedCanonicalIds: [],
        blockers: [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath: "/fake/src/Button.module.scss",
      selectorCount: 1,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 0,
      totalReferenceSites: 1,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: "/fake/src/Button.module.scss",
          localName: "indicator",
          totalReferences: 3,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 2,
          hasExpandedReferences: true,
          hasStyleDependencyReferences: false,
          hasAnyReferences: true,
          sites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 12, character: 8 },
                end: { line: 12, character: 17 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
          editableDirectSites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 12, character: 8 },
                end: { line: 12, character: 17 },
              },
              className: "indicator",
            },
          ],
        },
      ],
    },
    sourceInputEvidence: {},
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

function makeReferenceGraphWithSelectors(
  selectorNames: readonly string[],
): StyleSemanticGraphSummaryV0 {
  const base = makeReferenceGraph();
  const referencedSelector = base.selectorReferenceEngine.selectors[0]!;
  const selectors = selectorNames.map((localName) =>
    localName === referencedSelector.localName
      ? referencedSelector
      : {
          ...referencedSelector,
          canonicalId: `selector:${localName}`,
          localName,
          totalReferences: 0,
          directReferenceCount: 0,
          editableDirectReferenceCount: 0,
          exactReferenceCount: 0,
          inferredOrBetterReferenceCount: 0,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: false,
          hasAnyReferences: false,
          sites: [],
          editableDirectSites: [],
        },
  );
  return {
    ...base,
    selectorIdentityEngine: {
      ...base.selectorIdentityEngine,
      canonicalIdCount: selectorNames.length,
      canonicalIds: selectorNames.map((localName) => ({
        canonicalId: `selector:${localName}`,
        localName,
        identityKind: "localClass",
        rewriteSafety: "safe",
        blockers: [],
      })),
    },
    selectorReferenceEngine: {
      ...base.selectorReferenceEngine,
      selectorCount: selectorNames.length,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: Math.max(0, selectorNames.length - 1),
      selectors,
    },
  };
}

function makeEmptyReferenceGraph(): StyleSemanticGraphSummaryV0 {
  return {
    ...makeReferenceGraph(),
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath: "/fake/src/Button.module.scss",
      selectorCount: 1,
      referencedSelectorCount: 0,
      unreferencedSelectorCount: 1,
      totalReferenceSites: 0,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: "/fake/src/Button.module.scss",
          localName: "indicator",
          totalReferences: 0,
          directReferenceCount: 0,
          editableDirectReferenceCount: 0,
          exactReferenceCount: 0,
          inferredOrBetterReferenceCount: 0,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: false,
          hasAnyReferences: false,
          sites: [],
          editableDirectSites: [],
        },
      ],
    },
    sourceInputEvidence: {
      referenceSiteIdentity: {
        status: "ready",
        referenceSiteCount: 1,
      },
    },
  };
}

function makeSelectorUsageCandidate(canonicalName: string, hasAnyReferences: boolean) {
  const referenceCount = hasAnyReferences ? 1 : 0;
  return {
    kind: "selector-usage" as const,
    filePath: "/fake/src/Button.module.scss",
    queryId: canonicalName,
    payload: {
      canonicalName,
      totalReferences: referenceCount,
      directReferenceCount: referenceCount,
      editableDirectReferenceCount: referenceCount,
      exactReferenceCount: referenceCount,
      inferredOrBetterReferenceCount: referenceCount,
      hasExpandedReferences: false,
      hasStyleDependencyReferences: false,
      hasAnyReferences,
      allSites: hasAnyReferences
        ? [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 12, character: 8 },
                end: { line: 12, character: 17 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ]
        : [],
      editableDirectSites: hasAnyReferences
        ? [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 12, character: 8 },
                end: { line: 12, character: 17 },
              },
              className: canonicalName,
            },
          ]
        : [],
    },
  };
}
