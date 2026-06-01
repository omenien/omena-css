import { describe, expect, it } from "vitest";
import type { StyleDocumentHIR } from "../../../server/engine-core-ts/src/core/hir/style-types";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import {
  resolveStyleReferencesAtCursor,
  resolveStyleReferencesAtCursorAsync,
} from "../../../server/engine-host-node/src/style-references-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  return makeBaseDeps({
    selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

describe("resolveStyleReferencesAtCursor", () => {
  it("returns selector reference locations from the semantic index by default", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSiteAt("file:///fake/src/App.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: "/fake/src/Button.module.scss",
        line: 5,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ semanticReferenceIndex }),
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 10, character: 10 },
          end: { line: 10, character: 19 },
        },
      },
    ]);
  });

  it("uses rust selector-usage payloads for selector reference locations", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSiteAt("file:///fake/src/App.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: "/fake/src/Button.module.scss",
        line: 5,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ semanticReferenceIndex }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
          canonicalName: "indicator",
          totalReferences: 2,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 1,
          hasExpandedReferences: true,
          hasStyleDependencyReferences: true,
          hasAnyReferences: true,
          allSites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 10, character: 10 },
                end: { line: 10, character: 19 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
            {
              filePath: "/fake/src/Other.module.scss",
              range: {
                start: { line: 2, character: 1 },
                end: { line: 2, character: 10 },
              },
              expansion: "direct",
              referenceKind: "styleDependency",
            },
          ],
        }),
      },
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 10, character: 10 },
          end: { line: 10, character: 19 },
        },
      },
      {
        uri: "file:///fake/src/Other.module.scss",
        range: {
          start: { line: 2, character: 1 },
          end: { line: 2, character: 10 },
        },
      },
    ]);
  });

  it("uses rust style semantic graph reference sites for selector references", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: "/fake/src/Button.module.scss",
        line: 5,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps(),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeReferenceGraph(),
      },
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 12, character: 8 },
          end: { line: 12, character: 17 },
        },
      },
    ]);
  });

  it("rechecks rust selector-usage when rust graph has an empty selector reference summary", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: "/fake/src/Button.module.scss",
        line: 5,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps(),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeEmptyReferenceGraph(),
        readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
          canonicalName: "indicator",
          totalReferences: 1,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 1,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: false,
          hasAnyReferences: true,
          allSites: [
            {
              filePath: "/fake/src/FromRustUsage.tsx",
              range: {
                start: { line: 8, character: 4 },
                end: { line: 8, character: 13 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
        }),
      },
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/FromRustUsage.tsx",
        range: {
          start: { line: 8, character: 4 },
          end: { line: 8, character: 13 },
        },
      },
    ]);
  });

  it("does not fall through to the current reference index when rust graph and selector-usage agree on no references", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSiteAt("file:///fake/src/App.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      new Map([["indicator", infoAtLine("indicator", 5)]]),
    );

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: "/fake/src/Button.module.scss",
        line: 5,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ semanticReferenceIndex }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeEmptyReferenceGraph(),
        readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
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
        }),
      },
    );

    expect(result).toEqual([]);
  });

  it("returns same-file Sass symbol references from a declaration cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const scss = `$gap: 1rem;
.button {
  color: $gap;
  margin: $gap;
}
`;
    const styleDocument = parseStyleDocument(scss, filePath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );

    expect(result).toHaveLength(3);
    expect(result.every((location) => location.uri === "file:///fake/src/Button.module.scss")).toBe(
      true,
    );
    expect(result.map((location) => location.range.start.line)).toEqual([0, 2, 3]);
  });

  it("keeps local Sass variable references separate from same-name file-scope variables", () => {
    const filePath = "/fake/src/Button.module.scss";
    const scss = `$gap: 1rem;
.one {
  $gap: 2rem;
  color: $gap;
}
.two {
  color: $gap;
}
`;
    const styleDocument = parseStyleDocument(scss, filePath);

    const localResult = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 2,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );
    expect(localResult.map((location) => location.range.start.line)).toEqual([2, 3]);

    const fileScopeResult = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );
    expect(fileScopeResult.map((location) => location.range.start.line)).toEqual([0, 6]);
  });

  it("keeps local Less variable references separate from same-name file-scope variables", () => {
    const filePath = "/fake/src/Button.module.less";
    const less = `@gap: 1rem;
.one {
  @gap: 2rem;
  color: @gap;
}
.two {
  color: @gap;
}
`;
    const styleDocument = parseStyleDocument(less, filePath);

    const localResult = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 2,
        character: 3,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );
    expect(localResult.map((location) => location.range.start.line)).toEqual([2, 3]);

    const fileScopeResult = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );
    expect(fileScopeResult.map((location) => location.range.start.line)).toEqual([0, 6]);
  });

  it("returns same-file CSS custom property references from a declaration cursor", () => {
    const filePath = "/fake/src/tokens.module.css";
    const css = `:root { --color-gray-700: #767678; }
.title {
  color: var(--color-gray-700);
}
.footer {
  color: var(--color-gray-700);
}
`;
    const styleDocument = parseStyleDocument(css, filePath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 0,
        character: 10,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument]) }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.css",
      "file:///fake/src/tokens.module.css",
      "file:///fake/src/tokens.module.css",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 2, 5]);
  });

  it("returns workspace-indexed CSS custom property references from a reference cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/src/tokens.module.css";
    const buttonScss = `.button {
  color: var(--color-gray-700);
}
.footer {
  border-color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensCss, tokensPath);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(filePath, styleDocument);
    styleDependencyGraph.record(tokensPath, targetDocument);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 1,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 1, 4]);
  });

  it("returns imported package CSS custom property references from a reference cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/node_modules/@design/tokens/variables.css";
    const buttonScss = `@use "@design/tokens/variables.css";

.button {
  color: var(--color-gray-700);
}
.footer {
  border-color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensCss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/node_modules/@design/tokens/variables.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 6]);
  });

  it("uses Rust design-token winner ranges for external CSS custom property references", () => {
    const filePath = "/fake/src/Button.module.scss";
    const generatedTokensPath = "/fake/workspace/src/generated.tokens.css";
    const winnerRange = {
      start: { line: 8, character: 2 },
      end: { line: 8, character: 9 },
    };
    const buttonScss = `.button {
  color: var(--brand);
}
.footer {
  border-color: var(--brand);
}
`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 1,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument]),
      }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTarget: () =>
          makeDesignTokenDefinitionGraph({
            referenceName: "--brand",
            winnerDeclarationFilePath: generatedTokensPath,
            winnerDeclarationRange: winnerRange,
          }),
      },
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/workspace/src/generated.tokens.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([8, 1, 4]);
  });

  it("uses async Rust design-token winner ranges for external CSS custom property references", async () => {
    const filePath = "/fake/src/Button.module.scss";
    const generatedTokensPath = "/fake/workspace/src/generated.tokens.css";
    const winnerRange = {
      start: { line: 8, character: 2 },
      end: { line: 8, character: 9 },
    };
    const buttonScss = `.button {
  color: var(--brand);
}
.footer {
  border-color: var(--brand);
}
`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);

    const result = await resolveStyleReferencesAtCursorAsync(
      {
        filePath,
        line: 1,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument]),
      }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async () =>
          makeDesignTokenDefinitionGraph({
            referenceName: "--brand",
            winnerDeclarationFilePath: generatedTokensPath,
            winnerDeclarationRange: winnerRange,
          }),
      },
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/workspace/src/generated.tokens.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([8, 1, 4]);
  });

  it("returns package-root CSS custom property references through package.json style entries", () => {
    const filePath = "/fake/src/Button.module.scss";
    const packageRoot = "/fake/node_modules/@design/tokens";
    const packageJsonPath = `${packageRoot}/package.json`;
    const tokensPath = `${packageRoot}/variables.css`;
    const buttonScss = `@use "@design/tokens";

.button {
  color: var(--color-gray-700);
}
.footer {
  border-color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensCss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        readStyleFile: (candidate) =>
          candidate === packageJsonPath ? `{"style":"variables.css"}` : null,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/node_modules/@design/tokens/variables.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 6]);
  });

  it("returns CSS custom property references forwarded from a package root through local utilities", () => {
    const filePath = "/fake/src/Button.module.scss";
    const utilsPath = "/fake/src/_utils.scss";
    const packageRoot = "/fake/node_modules/@design/tokens";
    const packageJsonPath = `${packageRoot}/package.json`;
    const tokensPath = `${packageRoot}/variables.css`;
    const buttonScss = `@use "utils" as *;

.button {
  color: var(--color-gray-700);
}
.footer {
  border-color: var(--color-gray-700);
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const utilsDocument = parseStyleDocument(utilsScss, utilsPath);
    const targetDocument = parseStyleDocument(tokensCss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 16,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, utilsDocument, targetDocument]),
        readStyleFile: (candidate) =>
          candidate === packageJsonPath ? `{"style":"variables.css"}` : null,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/node_modules/@design/tokens/variables.css",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 6]);
  });

  it("returns namespace-qualified Sass member references with the target declaration", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./tokens.module" as tokens;

.button {
  color: tokens.$gap;
  margin: tokens.$gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 18,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]) }),
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/tokens.module.scss",
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 4 },
        },
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: {
          start: { line: 3, character: 16 },
          end: { line: 3, character: 20 },
        },
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: {
          start: { line: 4, character: 17 },
          end: { line: 4, character: 21 },
        },
      },
    ]);
  });

  it("returns wildcard Sass module member references with the target declaration", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./tokens.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 10,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({ styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]) }),
    );

    expect(result).toEqual([
      {
        uri: "file:///fake/src/tokens.module.scss",
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 4 },
        },
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: {
          start: { line: 3, character: 9 },
          end: { line: 3, character: 13 },
        },
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: {
          start: { line: 4, character: 10 },
          end: { line: 4, character: 14 },
        },
      },
    ]);
  });

  it("returns namespace-qualified Sass member references from the target declaration cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./tokens.module" as tokens;

.button {
  color: tokens.$gap;
  margin: tokens.$gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(filePath, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => tokensPath,
    });

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: tokensPath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument: targetDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.scss",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 4]);
  });

  it("returns wildcard Sass module member references from the target declaration cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./tokens.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(filePath, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => tokensPath,
    });

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: tokensPath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument: targetDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.scss",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 4]);
  });

  it("returns forwarded Sass module member references from the target declaration cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const themePath = "/fake/src/theme.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./theme.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const themeScss = `@forward "./tokens.module";`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const themeDocument = parseStyleDocument(themeScss, themePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(filePath, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => themePath,
      resolveSassModuleExportedSymbolTargetFilePaths: () => [tokensPath],
    });

    const result = resolveStyleReferencesAtCursor(
      {
        filePath: tokensPath,
        line: 0,
        character: 1,
        includeDeclaration: true,
        styleDocument: targetDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, themeDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.scss",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 4]);
  });

  it("returns prefixed forwarded wildcard references from the reference cursor", () => {
    const filePath = "/fake/src/Button.module.scss";
    const themePath = "/fake/src/theme.module.scss";
    const tokensPath = "/fake/src/tokens.module.scss";
    const buttonScss = `@use "./theme.module" as *;

.button {
  color: $theme-gap;
  margin: $theme-gap;
}
`;
    const themeScss = `@forward "./tokens.module" as theme-* show $gap;`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, filePath);
    const themeDocument = parseStyleDocument(themeScss, themePath);
    const targetDocument = parseStyleDocument(tokensScss, tokensPath);

    const result = resolveStyleReferencesAtCursor(
      {
        filePath,
        line: 3,
        character: 12,
        includeDeclaration: true,
        styleDocument,
      },
      makeDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, themeDocument, targetDocument]),
      }),
    );

    expect(result.map((location) => location.uri)).toEqual([
      "file:///fake/src/tokens.module.scss",
      "file:///fake/src/Button.module.scss",
      "file:///fake/src/Button.module.scss",
    ]);
    expect(result.map((location) => location.range.start.line)).toEqual([0, 3, 4]);
  });
});

function styleDocumentMap(documents: readonly StyleDocumentHIR[]) {
  const byPath = new Map(documents.map((document) => [document.filePath, document]));
  return (filePath: string) => byPath.get(filePath) ?? null;
}

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
          totalReferences: 1,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 1,
          hasExpandedReferences: false,
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
