import { describe, expect, it } from "vitest";
import type { StyleDocumentHIR } from "../../../server/engine-core-ts/src/core/hir/style-types";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import {
  resolveStyleHoverResult,
  resolveStyleHoverResultAsync,
  resolveStyleSelectorHoverResult,
  resolveStyleSelectorHoverResultAsync,
} from "../../../server/engine-host-node/src/style-hover-query";
import type {
  StyleSemanticGraphCache,
  StyleSemanticGraphSummaryV0,
} from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";
const TOKENS_PATH = "/fake/ws/src/tokens.module.scss";
const TOKENS_CSS_PATH = "/fake/ws/src/tokens.module.css";
const UTILS_PATH = "/fake/ws/src/_utils.scss";
const PACKAGE_TOKENS_ROOT = "/fake/ws/node_modules/@design/tokens";
const PACKAGE_TOKENS_JSON_PATH = `${PACKAGE_TOKENS_ROOT}/package.json`;
const PACKAGE_TOKENS_INDEX_PATH = `${PACKAGE_TOKENS_ROOT}/src/index.scss`;
const PACKAGE_TOKENS_COLORS_ENTRY_PATH = `${PACKAGE_TOKENS_ROOT}/src/colors.scss`;
const PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH = `${PACKAGE_TOKENS_ROOT}/src/typography.scss`;
const PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH = `${PACKAGE_TOKENS_ROOT}/src/_colors.scss`;
const PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH = `${PACKAGE_TOKENS_ROOT}/src/_typography.scss`;
const PACKAGE_COLORS_PATH = "/fake/ws/node_modules/@design/tokens/_colors.scss";
const PACKAGE_VARIABLES_CSS_PATH = "/fake/ws/node_modules/@design/tokens/variables.css";
const PACKAGE_TYPOGRAPHY_PATH = "/fake/ws/node_modules/@design/tokens/_typography.scss";

describe("style hover query", () => {
  it("uses rust selector-usage payloads for style hover summaries", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
      workspaceRoot: "/fake/ws",
    });

    const result = resolveStyleSelectorHoverResult(
      {
        filePath: SCSS_PATH,
        line: 5,
        character: 3,
      },
      deps,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
          canonicalName: "indicator",
          totalReferences: 4,
          directReferenceCount: 2,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 3,
          hasExpandedReferences: true,
          hasStyleDependencyReferences: true,
          hasAnyReferences: true,
        }),
      },
    );

    expect(result).not.toBeNull();
    expect(result?.usageSummary).toMatchObject({
      totalReferences: 4,
      directReferenceCount: 2,
      hasExpandedReferences: true,
      hasStyleDependencyReferences: true,
      hasAnyReferences: true,
    });
  });

  it("falls back to semantic selector usage when rust payload is unavailable", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/ws/src/App.tsx", [
      semanticSiteAt("file:///fake/ws/src/App.tsx", "indicator", 10, SCSS_PATH),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
      workspaceRoot: "/fake/ws",
      semanticReferenceIndex,
    });

    const result = resolveStyleSelectorHoverResult(
      {
        filePath: SCSS_PATH,
        line: 5,
        character: 3,
      },
      deps,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
      },
    );

    expect(result).not.toBeNull();
    expect(result?.usageSummary).toMatchObject({
      totalReferences: 1,
      directReferenceCount: 1,
      hasExpandedReferences: false,
      hasStyleDependencyReferences: false,
      hasAnyReferences: true,
    });
  });

  it("attaches rust semantic graph selector identity metadata for selector hovers", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
      workspaceRoot: "/fake/ws",
    });

    const result = resolveStyleSelectorHoverResult(
      {
        filePath: SCSS_PATH,
        line: 5,
        character: 3,
      },
      deps,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeGraph("blocked"),
      },
    );

    expect(result?.selectorIdentity).toMatchObject({
      canonicalId: "selector:indicator",
      canonicalName: "indicator",
      identityKind: "localClass",
      rewriteSafety: "blocked",
      blockers: ["nested-expansion"],
      range: { start: { line: 5, character: 1 }, end: { line: 5, character: 10 } },
    });
    expect(result?.usageSummary).toMatchObject({
      totalReferences: 3,
      directReferenceCount: 1,
      hasExpandedReferences: true,
      hasAnyReferences: true,
    });
  });

  it("attaches async rust semantic graph selector identity metadata for selector hovers", async () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
      workspaceRoot: "/fake/ws",
    });

    const result = await resolveStyleSelectorHoverResultAsync(
      {
        filePath: SCSS_PATH,
        line: 5,
        character: 3,
      },
      deps,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async () => makeGraph("blocked"),
      },
    );

    expect(result?.selectorIdentity).toMatchObject({
      canonicalId: "selector:indicator",
      canonicalName: "indicator",
      identityKind: "localClass",
      rewriteSafety: "blocked",
      blockers: ["nested-expansion"],
      range: { start: { line: 5, character: 1 }, end: { line: 5, character: 10 } },
    });
    expect(result?.usageSummary).toMatchObject({
      totalReferences: 3,
      directReferenceCount: 1,
      hasExpandedReferences: true,
      hasAnyReferences: true,
    });
  });

  it("shares the runtime style semantic graph cache across async selector hover reads", async () => {
    const styleSemanticGraphCache: StyleSemanticGraphCache = new Map();
    const deps = {
      ...makeBaseDeps({
        selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
        workspaceRoot: "/fake/ws",
      }),
      styleSemanticGraphCache,
    };
    let graphBuildCount = 0;

    const result = await resolveStyleSelectorHoverResultAsync(
      {
        filePath: SCSS_PATH,
        line: 5,
        character: 3,
      },
      deps,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async (
          _args,
          _deps,
          stylePath,
          queryOptions,
        ) => {
          expect(queryOptions.styleSemanticGraphCache).toBe(styleSemanticGraphCache);
          if (queryOptions.styleSemanticGraphCache?.has(stylePath)) {
            return queryOptions.styleSemanticGraphCache.get(stylePath) ?? null;
          }
          graphBuildCount += 1;
          const graph = makeGraph("blocked");
          queryOptions.styleSemanticGraphCache?.set(stylePath, graph);
          return graph;
        },
      },
    );

    expect(result?.selectorIdentity?.canonicalName).toBe("indicator");
    expect(result?.usageSummary.totalReferences).toBe(3);
    expect(graphBuildCount).toBe(1);
  });

  it("resolves animation references to keyframes hover data", () => {
    const scss = `@keyframes fade {
  from { opacity: 0; }
  to { opacity: 1; }
}

.box {
  animation: fade 1s linear;
}

.pulse {
  animation-name: fade;
}
`;
    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 6,
        character: 15,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([parseStyleDocument(scss, SCSS_PATH)]),
      }),
    );

    expect(result).toMatchObject({
      kind: "keyframes",
      scssModulePath: SCSS_PATH,
      headingName: "fade",
      note: "Referenced via `animation`",
      referenceCount: 2,
    });
  });

  it("resolves imported value references to value hover data", () => {
    const buttonScss = `@value primary from "./tokens.module.scss";

.button {
  color: primary;
}
`;
    const tokensScss = `@value primary: #ff3355;`;
    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 10,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([
          parseStyleDocument(buttonScss, SCSS_PATH),
          parseStyleDocument(tokensScss, TOKENS_PATH),
        ]),
      }),
    );

    expect(result).toMatchObject({
      kind: "value",
      scssModulePath: TOKENS_PATH,
      headingName: "primary",
      note: "Referenced via `declaration value`; imported from `./tokens.module.scss` as `primary`",
      referenceCount: 1,
    });
  });

  it("resolves same-file CSS custom property references to declaration hover data", () => {
    const css = `:root { --color-gray-700: #767678; }
.title {
  color: var(--color-gray-700);
}
.footer {
  color: var(--color-gray-700);
}
`;
    const result = resolveStyleHoverResult(
      {
        filePath: TOKENS_CSS_PATH,
        line: 2,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([parseStyleDocument(css, TOKENS_CSS_PATH)]),
      }),
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: TOKENS_CSS_PATH,
      headingName: "--color-gray-700",
      note: "Referenced via `var()`",
      referenceCount: 2,
    });
  });

  it("attaches rust design token ranking to custom property reference hovers", () => {
    const css = [
      ":root { --brand: red; }",
      ".theme { --brand: blue; }",
      ".button { --brand: green; color: var(--brand); }",
    ].join("\n");
    const styleDocument = parseStyleDocument(css, TOKENS_CSS_PATH);

    const result = resolveStyleHoverResult(
      {
        filePath: TOKENS_CSS_PATH,
        line: 2,
        character: 39,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument]),
      }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" },
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeDesignTokenRankingGraph(),
      },
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      headingName: "--brand",
      designTokenRanking: {
        winnerDeclaration: { value: "green" },
        shadowedDeclarations: [{ value: "red" }, { value: "blue" }],
      },
    });
  });

  it("attaches async rust design token ranking to custom property reference hovers", async () => {
    const css = [
      ":root { --brand: red; }",
      ".theme { --brand: blue; }",
      ".button { --brand: green; color: var(--brand); }",
    ].join("\n");
    const styleDocument = parseStyleDocument(css, TOKENS_CSS_PATH);

    const result = await resolveStyleHoverResultAsync(
      {
        filePath: TOKENS_CSS_PATH,
        line: 2,
        character: 39,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument]),
      }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" },
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async () =>
          makeDesignTokenRankingGraph(),
      },
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      headingName: "--brand",
      designTokenRanking: {
        winnerDeclaration: { value: "green" },
        shadowedDeclarations: [{ value: "red" }, { value: "blue" }],
      },
    });
  });

  it("uses async rust cross-file design token winners as custom property hover targets", async () => {
    const buttonCss = `.button {
  color: var(--brand);
}
`;
    const tokensCss = `:root { --brand: red; }`;
    const buttonDocument = parseStyleDocument(buttonCss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, TOKENS_CSS_PATH);
    const winnerDeclaration = tokensDocument.customPropertyDecls[0]!;

    const result = await resolveStyleHoverResultAsync(
      {
        filePath: SCSS_PATH,
        line: 1,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? buttonDocument : null),
        readStyleFile: (filePath) => (filePath === TOKENS_CSS_PATH ? tokensCss : null),
        buildStyleDocument: (filePath, content) => parseStyleDocument(content, filePath),
      }),
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" },
        readRustStyleSemanticGraphForWorkspaceTargetAsync: async () =>
          makeDesignTokenDefinitionGraph({
            referenceName: "--brand",
            winnerDeclarationFilePath: TOKENS_CSS_PATH,
            winnerDeclarationRange: winnerDeclaration.range,
          }),
      },
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: TOKENS_CSS_PATH,
      headingName: "--brand",
      customPropertyDecl: {
        name: "--brand",
        value: "red",
      },
      designTokenRanking: {
        winnerDeclarationFilePath: TOKENS_CSS_PATH,
        crossFileCandidateScope: "cross-file-import-candidate",
      },
    });
  });

  it("resolves workspace-indexed CSS custom property references to source files", () => {
    const buttonScss = `.button {
  color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, TOKENS_CSS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(SCSS_PATH, buttonDocument);
    styleDependencyGraph.record(TOKENS_CSS_PATH, tokensDocument);

    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 1,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([buttonDocument, tokensDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: TOKENS_CSS_PATH,
      headingName: "--color-gray-700",
      note: "Referenced via `var()`",
      referenceCount: 1,
      customPropertyDecl: {
        name: "--color-gray-700",
        value: "#767678",
      },
    });
  });

  it("resolves imported package CSS custom property references to source files", () => {
    const buttonScss = `@use "@design/tokens/variables.css";

.button {
  color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([buttonDocument, tokensDocument]),
      }),
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: PACKAGE_VARIABLES_CSS_PATH,
      headingName: "--color-gray-700",
      customPropertyDecl: {
        name: "--color-gray-700",
        value: "#767678",
      },
    });
  });

  it("resolves package-root CSS custom property references through package.json style entries", () => {
    const buttonScss = `@use "@design/tokens";

.button {
  color: var(--color-gray-700);
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([buttonDocument, tokensDocument]),
        readStyleFile: (filePath) =>
          filePath === PACKAGE_TOKENS_JSON_PATH ? `{"style":"variables.css"}` : null,
      }),
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: PACKAGE_VARIABLES_CSS_PATH,
      headingName: "--color-gray-700",
      customPropertyDecl: {
        name: "--color-gray-700",
        value: "#767678",
      },
    });
  });

  it("resolves CSS custom property references forwarded from a package root through a local utility module", () => {
    const buttonScss = `@use "utils" as *;

.button {
  color: var(--color-gray-700);
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const utilsDocument = parseStyleDocument(utilsScss, UTILS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 16,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([buttonDocument, utilsDocument, tokensDocument]),
        readStyleFile: (filePath) =>
          filePath === PACKAGE_TOKENS_JSON_PATH ? `{"style":"variables.css"}` : null,
      }),
    );

    expect(result).toMatchObject({
      kind: "customProperty",
      scssModulePath: PACKAGE_VARIABLES_CSS_PATH,
      headingName: "--color-gray-700",
      customPropertyDecl: {
        name: "--color-gray-700",
        value: "#767678",
      },
    });
  });

  it("resolves same-file Sass symbol references to declaration hover data", () => {
    const scss = `$gap: 1rem;
.button {
  color: $gap;
}
`;
    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 2,
        character: 10,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([parseStyleDocument(scss, SCSS_PATH)]),
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: SCSS_PATH,
      headingName: "gap",
      note: "Referenced via Sass reference",
      referenceCount: 1,
    });
  });

  it("resolves Less variable references to declaration hover data", () => {
    const less = `@gap: 1rem;
.button {
  color: @gap;
}
`;
    const filePath = "/fake/ws/src/Button.module.less";
    const result = resolveStyleHoverResult(
      {
        filePath,
        line: 2,
        character: 10,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([parseStyleDocument(less, filePath)]),
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: filePath,
      headingName: "gap",
      note: "Referenced via Less reference",
      referenceCount: 1,
    });
  });

  it("resolves namespace-qualified Sass member references to target module hover data", () => {
    const buttonScss = `@use "./tokens.module" as tokens;

.button {
  color: tokens.$gap;
  margin: tokens.$gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 18,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([
          parseStyleDocument(buttonScss, SCSS_PATH),
          parseStyleDocument(tokensScss, TOKENS_PATH),
        ]),
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      headingName: "tokens.gap",
      note: "Referenced via Sass module reference",
      referenceCount: 2,
    });
  });

  it("resolves wildcard Sass module member references to target module hover data", () => {
    const buttonScss = `@use "./tokens.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 10,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([
          parseStyleDocument(buttonScss, SCSS_PATH),
          parseStyleDocument(tokensScss, TOKENS_PATH),
        ]),
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      headingName: "gap",
      note: "Referenced via Sass wildcard reference",
      referenceCount: 2,
    });
  });

  it("resolves package-root Sass symbol hovers through package.json sass entries", () => {
    const buttonScss = `@use "@design/tokens" as *;

.button {
  color: $gray700;
  @include typography16;
}
`;
    const tokensScss = `$gray700: #767678;
@mixin typography16 {}
`;
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensScss, PACKAGE_TOKENS_INDEX_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: styleDocumentMap([buttonDocument, tokensDocument]),
      readStyleFile: (filePath) =>
        filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null,
    });

    const variableResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 10,
      },
      deps,
    );
    expect(variableResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_INDEX_PATH,
      headingName: "gray700",
      note: "Referenced via Sass wildcard reference",
    });

    const mixinResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 4,
        character: 12,
      },
      deps,
    );
    expect(mixinResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_INDEX_PATH,
      headingName: "typography16",
      note: "Referenced via Sass wildcard include",
    });
  });

  it("counts namespace-qualified Sass member references from declaration hover data", () => {
    const buttonScss = `@use "./tokens.module" as tokens;

.button {
  color: tokens.$gap;
  margin: tokens.$gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(SCSS_PATH, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => TOKENS_PATH,
    });

    const result = resolveStyleHoverResult(
      {
        filePath: TOKENS_PATH,
        line: 0,
        character: 1,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      referenceCount: 2,
    });
  });

  it("counts wildcard Sass module member references from declaration hover data", () => {
    const buttonScss = `@use "./tokens.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const tokensScss = `$gap: 1rem;`;
    const styleDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(SCSS_PATH, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => TOKENS_PATH,
    });

    const result = resolveStyleHoverResult(
      {
        filePath: TOKENS_PATH,
        line: 0,
        character: 1,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      referenceCount: 2,
    });
  });

  it("counts forwarded Sass module member references from declaration hover data", () => {
    const buttonScss = `@use "./theme.module" as *;

.button {
  color: $gap;
  margin: $gap;
}
`;
    const themeScss = `@forward "./tokens.module";`;
    const tokensScss = `$gap: 1rem;`;
    const themePath = "/fake/ws/src/theme.module.scss";
    const styleDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const themeDocument = parseStyleDocument(themeScss, themePath);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(SCSS_PATH, styleDocument, {
      resolveSassModuleUseTargetFilePath: () => themePath,
      resolveSassModuleExportedSymbolTargetFilePaths: () => [TOKENS_PATH],
    });

    const result = resolveStyleHoverResult(
      {
        filePath: TOKENS_PATH,
        line: 0,
        character: 1,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, themeDocument, targetDocument]),
        styleDependencyGraph,
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      referenceCount: 2,
    });
  });

  it("counts prefixed forwarded wildcard references from reference hover data", () => {
    const buttonScss = `@use "./theme.module" as *;

.button {
  color: $theme-gap;
  margin: $theme-gap;
}
`;
    const themeScss = `@forward "./tokens.module" as theme-* show $gap;`;
    const tokensScss = `$gap: 1rem;`;
    const themePath = "/fake/ws/src/theme.module.scss";
    const styleDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const themeDocument = parseStyleDocument(themeScss, themePath);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);

    const result = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 12,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap([styleDocument, themeDocument, targetDocument]),
      }),
    );

    expect(result).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: TOKENS_PATH,
      headingName: "theme-gap",
      referenceCount: 2,
    });
  });

  it("resolves hover for prefixed Sass members forwarded from package targets through a local utility module", () => {
    const buttonScss = `@use "utils" as *;

.title {
  color: $ds_gray700;
  @include ds_typography16;
}
`;
    const utilsScss = `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;
`;
    const colorsScss = `$gray700: #767678;`;
    const typographyScss = `@mixin typography16 {}`;
    const documents = [
      parseStyleDocument(buttonScss, SCSS_PATH),
      parseStyleDocument(utilsScss, UTILS_PATH),
      parseStyleDocument(colorsScss, PACKAGE_COLORS_PATH),
      parseStyleDocument(typographyScss, PACKAGE_TYPOGRAPHY_PATH),
    ];

    const variableResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 14,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap(documents),
      }),
    );
    expect(variableResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_COLORS_PATH,
      headingName: "ds_gray700",
      note: "Referenced via Sass wildcard reference",
    });

    const mixinResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 4,
        character: 15,
      },
      makeBaseDeps({
        styleDocumentForPath: styleDocumentMap(documents),
      }),
    );
    expect(mixinResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TYPOGRAPHY_PATH,
      headingName: "ds_typography16",
      note: "Referenced via Sass wildcard include",
    });
  });

  it("resolves hover for prefixed Sass members forwarded from package export patterns", () => {
    const buttonScss = `@use "utils" as *;

.title {
  color: $ds_gray700;
  @include ds_typography16;
}
`;
    const utilsScss = `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;
`;
    const colorsScss = `$gray700: #767678;`;
    const typographyScss = `@mixin typography16 {}`;
    const documents = [
      parseStyleDocument(buttonScss, SCSS_PATH),
      parseStyleDocument(utilsScss, UTILS_PATH),
      parseStyleDocument(colorsScss, PACKAGE_TOKENS_COLORS_ENTRY_PATH),
      parseStyleDocument(typographyScss, PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH),
    ];
    const deps = makeBaseDeps({
      styleDocumentForPath: styleDocumentMap(documents),
      readStyleFile: (filePath) =>
        filePath === PACKAGE_TOKENS_JSON_PATH
          ? `{"exports":{"./*":{"sass":"./src/*.scss"}}}`
          : null,
    });

    const variableResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 14,
      },
      deps,
    );
    expect(variableResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      headingName: "ds_gray700",
      note: "Referenced via Sass wildcard reference",
    });

    const mixinResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 4,
        character: 15,
      },
      deps,
    );
    expect(mixinResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH,
      headingName: "ds_typography16",
      note: "Referenced via Sass wildcard include",
    });
  });

  it("resolves hover for prefixed Sass members forwarded from a package root through a local utility module", () => {
    const buttonScss = `@use "utils" as *;

.title {
  color: $ds_gray700;
  @include ds_typography16;
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const tokensScss = `$gray700: #767678;
@mixin typography16 {}
`;
    const documents = [
      parseStyleDocument(buttonScss, SCSS_PATH),
      parseStyleDocument(utilsScss, UTILS_PATH),
      parseStyleDocument(tokensScss, PACKAGE_TOKENS_INDEX_PATH),
    ];
    const deps = makeBaseDeps({
      styleDocumentForPath: styleDocumentMap(documents),
      readStyleFile: (filePath) =>
        filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null,
    });

    const variableResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 14,
      },
      deps,
    );
    expect(variableResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_INDEX_PATH,
      headingName: "ds_gray700",
      note: "Referenced via Sass wildcard reference",
    });

    const mixinResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 4,
        character: 15,
      },
      deps,
    );
    expect(mixinResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_INDEX_PATH,
      headingName: "ds_typography16",
      note: "Referenced via Sass wildcard include",
    });
  });

  it("resolves hover through package-root internal forward chains", () => {
    const buttonScss = `@use "utils" as *;

.title {
  color: $ds_gray700;
  @include ds_typography16;
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const indexScss = `@forward "./colors";
@forward "./typography";
`;
    const colorsScss = `$gray700: #767678;`;
    const typographyScss = `@mixin typography16 {}`;
    const documents = [
      parseStyleDocument(buttonScss, SCSS_PATH),
      parseStyleDocument(utilsScss, UTILS_PATH),
      parseStyleDocument(indexScss, PACKAGE_TOKENS_INDEX_PATH),
      parseStyleDocument(colorsScss, PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH),
      parseStyleDocument(typographyScss, PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH),
    ];
    const deps = makeBaseDeps({
      styleDocumentForPath: styleDocumentMap(documents),
      readStyleFile: (filePath) =>
        filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null,
    });

    const variableResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 3,
        character: 14,
      },
      deps,
    );
    expect(variableResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH,
      headingName: "ds_gray700",
      note: "Referenced via Sass wildcard reference",
    });

    const mixinResult = resolveStyleHoverResult(
      {
        filePath: SCSS_PATH,
        line: 4,
        character: 15,
      },
      deps,
    );
    expect(mixinResult).toMatchObject({
      kind: "sassSymbol",
      scssModulePath: PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH,
      headingName: "ds_typography16",
      note: "Referenced via Sass wildcard include",
    });
  });
});

function styleDocumentMap(documents: readonly StyleDocumentHIR[]) {
  const byPath = new Map(documents.map((document) => [document.filePath, document]));
  return (filePath: string) => byPath.get(filePath) ?? null;
}

function makeGraph(rewriteSafety: "safe" | "blocked"): StyleSemanticGraphSummaryV0 {
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
          rewriteSafety,
          blockers: rewriteSafety === "blocked" ? ["nested-expansion"] : [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: rewriteSafety === "safe",
        safeCanonicalIds: rewriteSafety === "safe" ? ["selector:indicator"] : [],
        blockedCanonicalIds: rewriteSafety === "blocked" ? ["selector:indicator"] : [],
        blockers: rewriteSafety === "blocked" ? ["nested-expansion"] : [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath: SCSS_PATH,
      selectorCount: 1,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 0,
      totalReferenceSites: 1,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: SCSS_PATH,
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

function makeDesignTokenRankingGraph(): StyleSemanticGraphSummaryV0 {
  return {
    ...makeGraph("safe"),
    designTokenSemantics: {
      schemaVersion: "0",
      product: "omena-semantic.design-token-semantics",
      status: "same-file-cascade-ranking-seed",
      resolutionScope: "same-file",
      declarationCount: 3,
      referenceCount: 1,
      resolvedReferenceCount: 1,
      unresolvedReferenceCount: 0,
      selectorsWithReferencesCount: 1,
      contextSignal: {
        declarationContextSelectorCount: 1,
        declarationWrapperContextCount: 0,
        mediaContextSelectorCount: 0,
        supportsContextSelectorCount: 0,
        layerContextSelectorCount: 0,
        wrapperContextCount: 0,
      },
      resolutionSignal: {
        declarationFactCount: 3,
        referenceFactCount: 1,
        sourceOrderedDeclarationCount: 3,
        sourceOrderedReferenceCount: 1,
        occurrenceResolvedReferenceCount: 1,
        occurrenceUnresolvedReferenceCount: 0,
        contextMatchedReferenceCount: 1,
        contextUnmatchedReferenceCount: 0,
        rootDeclarationCount: 1,
        selectorScopedDeclarationCount: 2,
        wrapperScopedDeclarationCount: 0,
      },
      cascadeRankingSignal: {
        rankedReferenceCount: 1,
        unrankedReferenceCount: 0,
        sourceOrderWinnerDeclarationCount: 1,
        sourceOrderShadowedDeclarationCount: 2,
        repeatedNameDeclarationCount: 3,
        rankedReferences: [
          {
            referenceName: "--brand",
            referenceSourceOrder: 0,
            winnerDeclarationSourceOrder: 2,
            shadowedDeclarationSourceOrders: [0, 1],
            candidateDeclarationCount: 3,
          },
        ],
      },
      capabilities: {
        sameFileResolutionReady: true,
        wrapperContextSignalReady: false,
        sourceOrderSignalReady: true,
        sourceOrderCascadeRankingReady: true,
        occurrenceResolutionSignalReady: true,
        selectorContextResolutionReady: true,
        themeOverrideContextSignalReady: true,
        crossFileImportGraphReady: false,
        crossPackageCascadeRankingReady: false,
        themeOverrideContextReady: false,
      },
      blockingGaps: ["crossFileImportGraph", "crossPackageCascadeRanking", "themeOverrideContext"],
      nextPriorities: [
        "crossFileImportGraph",
        "crossPackageCascadeRanking",
        "themeOverrideContext",
      ],
    },
  };
}
