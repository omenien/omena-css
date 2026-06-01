import { describe, expect, it } from "vitest";
import type { StyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { resolveUnusedStyleSelectors } from "../../../server/engine-host-node/src/style-module-usage-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";

describe("style module usage query", () => {
  it("uses rust selector-usage payloads to decide unused selectors", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
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
      workspaceRoot: "/fake/ws",
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
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
    });

    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });

  it("does not precompute the semantic fallback when rust payloads cover all selectors", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
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
      styleDependencyGraph: throwingStyleDependencyGraph(),
      workspaceRoot: "/fake/ws",
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayloadForWorkspaceTarget: (_args, _deps, _filePath, canonicalName) => ({
        canonicalName,
        totalReferences: canonicalName === "indicator" ? 1 : 0,
        directReferenceCount: canonicalName === "indicator" ? 1 : 0,
        editableDirectReferenceCount: canonicalName === "indicator" ? 1 : 0,
        exactReferenceCount: canonicalName === "indicator" ? 1 : 0,
        inferredOrBetterReferenceCount: canonicalName === "indicator" ? 1 : 0,
        hasExpandedReferences: false,
        hasStyleDependencyReferences: false,
        hasAnyReferences: canonicalName === "indicator",
      }),
    });

    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });

  it("reads rust selector-usage payloads once per style file", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
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
      workspaceRoot: "/fake/ws",
    });
    let payloadReads = 0;

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayloadsForWorkspaceTarget: () => {
        payloadReads += 1;
        return [
          makeSelectorUsageCandidate("indicator", true),
          makeSelectorUsageCandidate("active", false),
        ];
      },
    });

    expect(payloadReads).toBe(1);
    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });

  it("uses rust style semantic graph references before selector-usage payloads", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
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
      workspaceRoot: "/fake/ws",
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeReferenceGraph(),
      readRustSelectorUsagePayloadForWorkspaceTarget: () => {
        throw new Error("unexpected selector-usage fallback");
      },
    });

    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });

  it("falls back to current usage when rust graph drops known source references", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/ws/src/App.tsx", [
      semanticSiteAt("file:///fake/ws/src/App.tsx", "indicator", 8, SCSS_PATH),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake/ws",
      semanticReferenceIndex,
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeReferenceGraph({ indicator: false }),
      readRustSelectorUsagePayloadForWorkspaceTarget: () => {
        throw new Error("unexpected selector-usage fallback");
      },
    });

    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });

  it("does not read rust graph diagnostics when current usage has no unused selectors", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/ws/src/App.tsx", [
      semanticSiteAt("file:///fake/ws/src/App.tsx", "indicator", 8, SCSS_PATH),
      semanticSiteAt("file:///fake/ws/src/App.tsx", "active", 9, SCSS_PATH),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake/ws",
      semanticReferenceIndex,
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () => {
        throw new Error("unexpected rust graph read");
      },
      readRustSelectorUsagePayloadForWorkspaceTarget: () => {
        throw new Error("unexpected selector-usage fallback");
      },
    });

    expect(unused).toEqual([]);
  });

  it("falls back to semantic usage summary when rust payloads are unavailable", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([
        ["indicator", infoAtLine("indicator", 1)],
        ["active", infoAtLine("active", 3)],
      ]),
    );
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/ws/src/App.tsx", [
      semanticSiteAt("file:///fake/ws/src/App.tsx", "indicator", 8, SCSS_PATH),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["indicator", infoAtLine("indicator", 1)],
          ["active", infoAtLine("active", 3)],
        ]),
      workspaceRoot: "/fake/ws",
      semanticReferenceIndex,
    });

    const unused = resolveUnusedStyleSelectors({ scssPath: SCSS_PATH, styleDocument }, deps, {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
    });

    expect(unused).toEqual([
      expect.objectContaining({
        canonicalName: "active",
      }),
    ]);
  });
});

function throwingStyleDependencyGraph(): StyleDependencyGraph {
  return {
    record: () => {
      throw new Error("unexpected semantic fallback");
    },
    forget: () => {},
    forgetWithinRoot: () => {},
    getIncoming: () => [],
    getOutgoing: () => [],
    getIncomingSassModuleMemberRefs: () => [],
    getAllCustomPropertyDecls: () => [],
    getCustomPropertyDecls: () => [],
    getCustomPropertyRefs: () => [],
  };
}

function makeReferenceGraph(
  references: { readonly indicator: boolean; readonly active: boolean } = {
    indicator: true,
    active: false,
  },
): StyleSemanticGraphSummaryV0 {
  const referencedSelectorCount = Number(references.indicator) + Number(references.active);
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
      stylePath: SCSS_PATH,
      selectorCount: 2,
      referencedSelectorCount,
      unreferencedSelectorCount: 2 - referencedSelectorCount,
      totalReferenceSites: referencedSelectorCount,
      selectors: [
        makeSelectorReferenceSummary("indicator", references.indicator),
        makeSelectorReferenceSummary("active", references.active),
      ],
    },
    sourceInputEvidence: {
      referenceSiteIdentity: {
        status: "ready",
        referenceSiteCount: referencedSelectorCount,
      },
    },
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

function makeSelectorReferenceSummary(localName: string, hasAnyReferences: boolean) {
  const referenceCount = hasAnyReferences ? 1 : 0;
  return {
    canonicalId: `selector:${localName}`,
    filePath: SCSS_PATH,
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
            filePath: "/fake/ws/src/App.tsx",
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
            filePath: "/fake/ws/src/App.tsx",
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

function makeSelectorUsageCandidate(canonicalName: string, hasAnyReferences: boolean) {
  const referenceCount = hasAnyReferences ? 1 : 0;
  return {
    kind: "selector-usage" as const,
    filePath: SCSS_PATH,
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
      ...(hasAnyReferences
        ? {
            allSites: [
              {
                filePath: "/fake/ws/src/App.tsx",
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
                filePath: "/fake/ws/src/App.tsx",
                range: {
                  start: { line: 12, character: 8 },
                  end: { line: 12, character: 17 },
                },
                className: canonicalName,
              },
            ],
          }
        : {
            allSites: [],
            editableDirectSites: [],
          }),
    },
  };
}
