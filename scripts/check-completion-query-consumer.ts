import assert from "node:assert/strict";
import { CompletionItemKind } from "vscode-languageserver-protocol/node";
import { AliasResolver } from "../server/engine-core-ts/src/core/cx/alias-resolver";
import { DocumentAnalysisCache } from "../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { NullSemanticWorkspaceReferenceIndex } from "../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import {
  UNRESOLVABLE_TYPE,
  type TypeResolver,
} from "../server/engine-core-ts/src/core/ts/type-resolver";
import { DEFAULT_SETTINGS } from "../server/engine-core-ts/src/settings";
import { runRustSelectedQueryBackendJsonAsync } from "../server/engine-host-node/src/selected-query-backend";
import { createRequiredRustSourceFrontendAnalysisProvider } from "../server/engine-host-node/src/source-frontend-analysis-provider";
import { handleCompletion } from "../server/lsp-server/src/providers/completion";
import type { CursorParams, ProviderDeps } from "../server/lsp-server/src/providers/provider-deps";

const SOURCE_PATH = "/workspace/src/App.tsx";
const SOURCE_URI = "file:///workspace/src/App.tsx";
const STYLE_PATH = "/workspace/src/Button.module.scss";
const STYLE_URI = "file:///workspace/src/Button.module.scss";
const SOURCE_WITH_MARKER = [
  'import classNames from "classnames/bind";',
  'import styles from "./Button.module.scss";',
  "const cx = classNames.bind(styles);",
  'export const view = cx("item--/*|*/ive");',
  "",
].join("\n");
const STYLE_WITH_SELECTORS = [
  ".item--large { color: red; }",
  ".item--active { color: green; }",
  ".item--passive { color: blue; }",
  "",
].join("\n");
const STYLE_WITH_CUSTOM_PROPERTIES = [
  ":root { --alpha: red; }",
  ".card { --zeta: blue; color: var(--/*|*/); }",
  ".next { --omega: red; }",
  "",
].join("\n");

const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
const previousDaemon = process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = "0";

main().catch((err: unknown) => {
  console.error(err);
  process.exitCode = 1;
});

async function main(): Promise<void> {
  try {
    const sourceFixture = stripMarker(SOURCE_WITH_MARKER);
    const sourceDeps = makeDeps(STYLE_WITH_SELECTORS);
    const sourceItems = await handleCompletion(
      {
        documentUri: SOURCE_URI,
        content: sourceFixture.content,
        filePath: SOURCE_PATH,
        line: sourceFixture.line,
        character: sourceFixture.character,
        version: 1,
      },
      sourceDeps,
    );
    assert(sourceItems, "source completion should return query-ranked selector items");
    assert.deepEqual(
      sourceItems.slice(0, 3).map((item) => item.label),
      ["item--active", "item--passive", "item--large"],
    );
    assert.equal(sourceItems[0]?.data?.product, "omena-query.completion-at");
    assert.equal(sourceItems[0]?.data?.rankingSource, "valueDomainSelectorProjection");
    assert.equal(sourceItems[0]?.kind, CompletionItemKind.Value);

    const styleFixture = stripMarker(STYLE_WITH_CUSTOM_PROPERTIES);
    const styleDeps = makeDeps(styleFixture.content);
    const styleItems = await handleCompletion(
      {
        documentUri: STYLE_URI,
        content: styleFixture.content,
        filePath: STYLE_PATH,
        line: styleFixture.line,
        character: styleFixture.character,
        version: 1,
      },
      styleDeps,
    );
    assert(styleItems, "style completion should return query-ranked custom properties");
    assert.deepEqual(
      styleItems.slice(0, 3).map((item) => item.label),
      ["--zeta", "--alpha", "--omega"],
    );
    assert.equal(styleItems[0]?.data?.product, "omena-query.completion-at");
    assert.equal(styleItems[0]?.data?.rankingSource, "sameFileSourceOrderCascade");
    assert.equal(styleItems[0]?.kind, CompletionItemKind.Variable);

    process.stdout.write(
      [
        "validated completion query consumer:",
        "provider=LSP",
        "sourceRanking=valueDomainSelectorProjection",
        "styleRanking=sameFileSourceOrderCascade",
      ].join(" ") + "\n",
    );
  } finally {
    if (previousBackend === undefined) {
      delete process.env.OMENA_SELECTED_QUERY_BACKEND;
    } else {
      process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
    }
    if (previousDaemon === undefined) {
      delete process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
    } else {
      process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = previousDaemon;
    }
  }
}

function makeDeps(styleSource: string): ProviderDeps & {
  readonly runRustSelectedQueryBackendJsonAsync: typeof runRustSelectedQueryBackendJsonAsync;
} {
  const aliasResolver = new AliasResolver("/workspace", {});
  const fileExists = (filePath: string) => filePath === STYLE_PATH;
  const sourceFrontendAnalysis = createRequiredRustSourceFrontendAnalysisProvider({
    aliasResolver: () => aliasResolver,
    fileExists,
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFrontendAnalysis,
    fileExists,
    aliasResolver,
    max: 10,
  });
  const typeResolver: TypeResolver = {
    resolve: () => UNRESOLVABLE_TYPE,
    invalidate: () => {},
    clear: () => {},
  };
  return {
    analysisCache,
    aliasResolver,
    styleDocumentForPath: (filePath: string) =>
      filePath === STYLE_PATH ? parseStyleDocument(styleSource, STYLE_PATH) : null,
    typeResolver,
    semanticReferenceIndex: new NullSemanticWorkspaceReferenceIndex(),
    styleDependencyGraph: new WorkspaceStyleDependencyGraph(),
    workspaceRoot: "/workspace",
    workspaceFolderUri: "file:///workspace",
    logError: (_message: string, err: unknown) => {
      throw err;
    },
    invalidateStyle: () => {},
    peekStyleDocument: () => null,
    buildStyleDocument: (filePath: string, content: string) =>
      parseStyleDocument(content, filePath),
    readOpenDocumentText: (filePath: string) => (filePath === STYLE_PATH ? styleSource : null),
    readStyleFile: (filePath: string) => (filePath === STYLE_PATH ? styleSource : null),
    fileExists,
    pushStyleFile: () => {},
    indexerReady: Promise.resolve(),
    stopIndexer: () => {},
    settings: DEFAULT_SETTINGS,
    rebuildAliasResolver: () => {},
    refreshCodeLens: () => {},
    runRustSelectedQueryBackendJsonAsync,
  };
}

function stripMarker(source: string): Pick<CursorParams, "content" | "line" | "character"> {
  const marker = "/*|*/";
  const offset = source.indexOf(marker);
  assert(offset >= 0, "fixture must contain cursor marker");
  const content = source.slice(0, offset) + source.slice(offset + marker.length);
  const prefix = source.slice(0, offset);
  const lines = prefix.split("\n");
  return {
    content,
    line: lines.length - 1,
    character: lines[lines.length - 1]!.length,
  };
}
