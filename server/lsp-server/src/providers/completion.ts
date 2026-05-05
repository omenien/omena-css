import { CompletionItemKind, type CompletionItem } from "vscode-languageserver/node";
import type {
  SassSymbolDeclHIR,
  SelectorDeclHIR,
} from "../../../engine-core-ts/src/core/hir/style-types";
import { findLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import { resolveSourceCompletionSelectors } from "../../../engine-host-node/src/source-completion-query";
import {
  resolveStyleCompletionItems,
  type StyleCompletionItem,
} from "../../../engine-host-node/src/style-completion-query";
import {
  resolveStyleDesignTokenDeclarationCandidatesForDocumentAsync,
  type StyleDesignTokenDeclarationCandidateReadModel,
} from "../../../engine-host-node/src/style-design-token-ranking-query";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../engine-host-node/src/selected-query-backend";
import type { CursorParams, ProviderDeps } from "./provider-deps";
import { toLspRange } from "./lsp-adapters";
import { wrapHandler } from "./_wrap-handler";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";

/**
 * Handle `textDocument/completion` inside a class-util call.
 *
 * Pipeline:
 * 1. Fetch the single AnalysisEntry. Bail if it has neither
 *    `bindings` (cx pipeline) nor `stylesBindings` (clsx path).
 * 2. Ask `findCompletionContext` for the SCSS module whose
 *    style document should feed completions at the cursor. It walks
 *    cx bindings first, then class-util imports.
 * 3. Convert every selector in that style document to a CompletionItem.
 */
export const handleCompletion = wrapHandler<CursorParams, [], CompletionItem[] | null>(
  "completion",
  computeCompletion,
  null,
);

function computeCompletion(
  params: CursorParams,
  deps: ProviderDeps,
): CompletionItem[] | null | Promise<CompletionItem[] | null> {
  const rustRunner = getRustSelectedQueryBackendJsonRunnerAsync(deps);
  if (findLangForPath(params.filePath)) {
    if (rustRunner) return computeStyleCompletionAsync(params, deps, rustRunner);
    return computeStyleCompletion(params, deps);
  }

  const selectors = resolveSourceCompletionSelectors(params, deps);
  if (selectors.length === 0) return null;
  return selectors.map(toCompletionItem);
}

function computeStyleCompletion(
  params: CursorParams,
  deps: ProviderDeps,
  designTokenDeclarationCandidates?: readonly StyleDesignTokenDeclarationCandidateReadModel[],
): CompletionItem[] | null {
  const styleDocument = deps.styleDocumentForPath(params.filePath);
  if (!styleDocument) return null;
  const items = resolveStyleCompletionItems({
    content: params.content,
    line: params.line,
    character: params.character,
    styleDocument,
    styleDocumentForPath: deps.styleDocumentForPath,
    aliasResolver: deps.aliasResolver,
    styleDependencyGraph: deps.styleDependencyGraph,
    readFile: deps.readStyleFile,
    ...(designTokenDeclarationCandidates ? { designTokenDeclarationCandidates } : {}),
  });
  return items.length > 0 ? items.map(toStyleCompletionItem) : null;
}

async function computeStyleCompletionAsync(
  params: CursorParams,
  deps: ProviderDeps,
  rustRunner: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<CompletionItem[] | null> {
  const styleDocument = deps.styleDocumentForPath(params.filePath);
  if (!styleDocument) return null;
  const designTokenDeclarationCandidates =
    await resolveStyleDesignTokenDeclarationCandidatesForDocumentAsync(
      { filePath: params.filePath, styleDocument },
      deps,
      { runRustSelectedQueryBackendJsonAsync: rustRunner },
    );
  return computeStyleCompletion(params, deps, designTokenDeclarationCandidates ?? undefined);
}

function toStyleCompletionItem(item: StyleCompletionItem): CompletionItem {
  return {
    label: item.label,
    kind: toSassSymbolCompletionKind(item.symbolKind),
    detail: item.detail,
    sortText: item.filterText,
    filterText: item.filterText,
    insertText: item.insertText,
    textEdit: {
      range: toLspRange(item.replacementRange),
      newText: item.insertText,
    },
  };
}

function toSassSymbolCompletionKind(
  symbolKind: SassSymbolDeclHIR["symbolKind"] | "customProperty" | "value",
): CompletionItemKind {
  switch (symbolKind) {
    case "customProperty":
    case "value":
    case "variable":
      return CompletionItemKind.Variable;
    case "mixin":
    case "function":
      return CompletionItemKind.Function;
  }
}

function toCompletionItem(selector: SelectorDeclHIR): CompletionItem {
  const detail = selector.declarations.trim() || selector.fullSelector;
  return {
    label: selector.name,
    kind: CompletionItemKind.Value,
    detail,
    documentation: {
      kind: "markdown",
      value: `\`\`\`scss\n.${selector.name} { ${selector.declarations.trim()} }\n\`\`\``,
    },
    sortText: selector.name,
    filterText: selector.name,
    insertText: selector.name,
  };
}

/**
 * Trigger characters for the completion provider: `'`, `"`,
 * `` ` ``, `,`, `.`, `$`, `@`, and `-`.
 *
 * The `.` trigger is needed for `styles.` inside clsx/classnames
 * calls where completion must fire on the dot.
 * `$`, `@`, and `-` trigger Sass/Less/custom-property completions
 * in style files.
 */
export const COMPLETION_TRIGGER_CHARACTERS = ["'", '"', "`", ",", ".", "$", "@", "-"] as const;

/**
 * Return true when the last `<name>(` on `textBefore` is still
 * open — i.e. the cursor sits inside the argument list of that
 * call. Used for both `cx(` (classnames/bind) and `clsx(` /
 * `classnames(` (clsx-style) detection.
 *
 * String-aware: parentheses inside `'…'`, `"…"`, or `` `…` ``
 * are ignored. Escaped quotes (backslash) are handled. This
 * means `cx(')')` correctly remains "inside" the call.
 */
