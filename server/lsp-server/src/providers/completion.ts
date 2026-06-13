import { CompletionItemKind, type CompletionItem } from "vscode-languageserver/node";
import type {
  SassSymbolDeclHIR,
  SelectorDeclHIR,
} from "../../../engine-core-ts/src/core/hir/style-types";
import { findLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import {
  resolveSourceCompletionSelectorContext,
  resolveSourceCompletionSelectors,
  type SourceCompletionSelectorContext,
} from "../../../engine-host-node/src/source-completion-query";
import {
  resolveStyleCompletionItems,
  type StyleCompletionItem,
} from "../../../engine-host-node/src/style-completion-query";
import {
  resolveStyleDesignTokenDeclarationCandidatesForDocumentAsync,
  type StyleDesignTokenDeclarationCandidateReadModel,
} from "../../../engine-host-node/src/style-design-token-ranking-query";
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  resolveSelectedQueryBackendKind,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "../../../engine-host-node/src/selected-query-backend";
import type { CursorParams, ProviderDeps } from "./provider-deps";
import { toLspRange } from "./lsp-adapters";
import { wrapHandler } from "./_wrap-handler";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";

interface QueryCompletionAtPositionV0 {
  readonly product: "omena-query.completion-at";
  readonly fileKind: "source" | "style";
  readonly prefix?: string;
  readonly items: readonly QueryCompletionItemV0[];
}

interface QueryCompletionItemV0 {
  readonly label: string;
  readonly insertText: string;
  readonly sortText: string;
  readonly detail: string;
  readonly itemKind: string;
  readonly rankingSource: string;
  readonly source: string;
}

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

  if (rustRunner && resolveSelectedQueryBackendKind() === "rust-selected-query") {
    return computeSourceCompletionAsync(params, deps, rustRunner);
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
  const queryCompletion = await resolveQueryOwnedStyleCompletion(params, rustRunner).catch(
    (err: unknown) => {
      deps.logError("query-owned style completion failed", err);
      return [];
    },
  );
  const designTokenDeclarationCandidates =
    await resolveStyleDesignTokenDeclarationCandidatesForDocumentAsync(
      { filePath: params.filePath, styleDocument },
      deps,
      { runRustSelectedQueryBackendJsonAsync: rustRunner },
    );
  const localCompletion =
    computeStyleCompletion(params, deps, designTokenDeclarationCandidates ?? undefined) ?? [];
  if (queryCompletion.length === 0) return localCompletion.length > 0 ? localCompletion : null;
  return mergeCompletionItems(queryCompletion, localCompletion);
}

async function computeSourceCompletionAsync(
  params: CursorParams,
  deps: ProviderDeps,
  rustRunner: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<CompletionItem[] | null> {
  const context = resolveSourceCompletionSelectorContext(params, deps);
  if (!context || context.selectors.length === 0) return null;
  const styleSource =
    deps.readOpenDocumentText?.(context.scssModulePath) ??
    deps.readStyleFile(context.scssModulePath);
  if (styleSource === null) return context.selectors.map(toCompletionItem);

  const summary = await rustRunner<QueryCompletionAtPositionV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.completionAt,
    {
      fileUri: params.filePath,
      fileKind: "source",
      position: { line: params.line, character: params.character },
      styles: [{ stylePath: context.scssModulePath, styleSource }],
      targetStyleUri: context.scssModulePath,
      ...(context.valuePrefix ? { valuePrefix: context.valuePrefix } : {}),
      preferredSelectorNames: context.preferredSelectorNames,
    },
  ).catch((err: unknown) => {
    deps.logError("query-owned source completion failed", err);
    return null;
  });
  if (!summary) return context.selectors.map(toCompletionItem);
  if (summary.product !== "omena-query.completion-at" || summary.fileKind !== "source") {
    return context.selectors.map(toCompletionItem);
  }
  if (summary.items.length === 0) return null;
  return summary.items.map((item) => toQuerySourceCompletionItem(item, context));
}

async function resolveQueryOwnedStyleCompletion(
  params: CursorParams,
  rustRunner: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<CompletionItem[]> {
  const summary = await rustRunner<QueryCompletionAtPositionV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.completionAt,
    {
      fileUri: params.filePath,
      fileKind: "style",
      position: { line: params.line, character: params.character },
      styleSource: params.content,
      styles: [{ stylePath: params.filePath, styleSource: params.content }],
    },
  );
  if (summary.product !== "omena-query.completion-at" || summary.fileKind !== "style") return [];
  return summary.items.map((item) =>
    toQueryStyleCompletionItem(item, params, summary.prefix ?? ""),
  );
}

function mergeCompletionItems(
  primary: readonly CompletionItem[],
  fallback: readonly CompletionItem[],
): CompletionItem[] {
  const seen = new Set(primary.map((item) => item.label));
  return [...primary, ...fallback.filter((item) => !seen.has(item.label))];
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

function toQueryStyleCompletionItem(
  item: QueryCompletionItemV0,
  params: CursorParams,
  prefix: string,
): CompletionItem {
  return {
    label: item.label,
    kind: toQueryCompletionKind(item.itemKind),
    detail: item.detail,
    sortText: item.sortText,
    filterText: item.label,
    insertText: item.insertText,
    textEdit: {
      range: {
        start: { line: params.line, character: params.character - prefix.length },
        end: { line: params.line, character: params.character },
      },
      newText: item.insertText,
    },
    data: {
      product: "omena-query.completion-at",
      rankingSource: item.rankingSource,
      source: item.source,
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

function toQuerySourceCompletionItem(
  item: QueryCompletionItemV0,
  context: SourceCompletionSelectorContext,
): CompletionItem {
  const selector = context.selectors.find((candidate) => candidate.name === item.label);
  return {
    ...(selector ? toCompletionItem(selector) : {}),
    label: item.label,
    kind: toQueryCompletionKind(item.itemKind),
    detail: selector?.declarations.trim() || item.detail,
    sortText: item.sortText,
    filterText: item.label,
    insertText: item.insertText,
    data: {
      product: "omena-query.completion-at",
      rankingSource: item.rankingSource,
      source: item.source,
    },
  };
}

function toQueryCompletionKind(itemKind: string): CompletionItemKind {
  switch (itemKind) {
    case "cssCustomProperty":
    case "sassVariable":
      return CompletionItemKind.Variable;
    case "sassMixin":
    case "sassFunction":
    case "sassSymbol":
      return CompletionItemKind.Function;
    case "cssModuleSelector":
    default:
      return CompletionItemKind.Value;
  }
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
