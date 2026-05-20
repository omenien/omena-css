import assert from "node:assert/strict";
import { CodeActionKind, type CodeAction } from "vscode-languageserver-protocol/node";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import { handleCodeAction } from "../server/lsp-server/src/providers/code-actions";
import type { ProviderDeps } from "../server/lsp-server/src/providers/provider-deps";

const STYLE_URI = "file:///workspace/src/Button.module.scss";
const STYLE_PATH = "/workspace/src/Button.module.scss";
const STYLE_SOURCE = ".button { color: #fff; }\n";
const RANGE = {
  start: { line: 0, character: 17 },
  end: { line: 0, character: 21 },
};
const INLINE_STYLE_SOURCE = [
  ".base { color: red; padding: 4px; }",
  ".button {",
  "  composes: base;",
  "  background: blue;",
  "}",
  "",
].join("\n");
const INLINE_RANGE = {
  start: { line: 2, character: 12 },
  end: { line: 2, character: 16 },
};

const previousBackend = process.env.CME_SELECTED_QUERY_BACKEND;
process.env.CME_SELECTED_QUERY_BACKEND = "rust-selected-query";

try {
  const errors: unknown[] = [];
  const actions = handleCodeAction(
    {
      textDocument: { uri: STYLE_URI },
      range: RANGE,
      context: { diagnostics: [], triggerKind: 1 },
    },
    {
      fileExists: () => true,
      buildStyleDocument: (filePath: string, content: string) =>
        parseStyleDocument(content, filePath),
      readStyleFile: () => null,
      logError: (_message: string, err: unknown) => errors.push(err),
    } as ProviderDeps,
    STYLE_SOURCE,
  ) as CodeAction[] | null;

  assert.deepEqual(errors, []);
  assert(actions, "code-action provider should return refactor actions");
  const extract = actions.find(
    (action) => action.kind === CodeActionKind.RefactorExtract && action.title.includes("--"),
  );
  assert(extract, "expected query-owned CSS custom property extract refactor");
  assert.equal(extract.title, "Extract CSS custom property '--extracted-color'");
  assert.deepEqual(extract.edit?.changes?.[STYLE_URI], [
    {
      range: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 0 },
      },
      newText: ":root {\n  --extracted-color: #fff;\n}\n\n",
    },
    {
      range: RANGE,
      newText: "var(--extracted-color)",
    },
  ]);
  const inlineActions = handleCodeAction(
    {
      textDocument: { uri: STYLE_URI },
      range: INLINE_RANGE,
      context: { diagnostics: [], triggerKind: 1 },
    },
    {
      fileExists: () => true,
      buildStyleDocument: (filePath: string, content: string) =>
        parseStyleDocument(content, filePath),
      readStyleFile: () => null,
      logError: (_message: string, err: unknown) => errors.push(err),
    } as ProviderDeps,
    INLINE_STYLE_SOURCE,
  ) as CodeAction[] | null;

  assert.deepEqual(errors, []);
  assert(inlineActions, "code-action provider should return inline refactor actions");
  const inline = inlineActions.find((action) => action.kind === CodeActionKind.RefactorInline);
  assert(inline, "expected query-owned composed-class inline refactor");
  assert.equal(inline.title, "Inline composed class 'base'");
  assert.deepEqual(inline.edit?.changes?.[STYLE_URI], [
    {
      range: {
        start: { line: 2, character: 2 },
        end: { line: 2, character: 17 },
      },
      newText: "color: red;\n  padding: 4px;",
    },
  ]);
  process.stdout.write(
    `validated code-action query consumer: provider=LSP actions=refactor.extract,refactor.inline source=${STYLE_PATH}\n`,
  );
} finally {
  if (previousBackend === undefined) {
    delete process.env.CME_SELECTED_QUERY_BACKEND;
  } else {
    process.env.CME_SELECTED_QUERY_BACKEND = previousBackend;
  }
}
