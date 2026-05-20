import type { DocumentAnalysisCache } from "./core/indexing/document-analysis-cache";
import type { AliasResolver } from "./core/cx/alias-resolver";
import type { StyleDocumentHIR } from "./core/hir/style-types";
import type { SemanticWorkspaceReferenceIndex, StyleDependencyGraph } from "./core/semantic";
import type { TypeResolver } from "./core/ts/type-resolver";
import type { Settings } from "./settings";

/**
 * Identity + content of a single open document. Used by
 * document-wide computations (diagnostics) that do not care
 * about a cursor position.
 */
export interface DocumentParams {
  readonly documentUri: string;
  readonly content: string;
  readonly filePath: string;
  readonly version: number;
}

/**
 * One request's cursor location, plus the document context the
 * provider needs to resolve it. Extends `DocumentParams` so
 * cursor providers can be passed to document-wide helpers
 * without an explicit narrowing step.
 */
export interface CursorParams extends DocumentParams {
  readonly line: number;
  readonly character: number;
}

/**
 * The dependency bag every provider handler accepts.
 *
 * Server wiring builds this once at startup; provider unit tests
 * build a stub via `makeBaseDeps()` in test helpers. Keeping this
 * a plain interface with no methods keeps provider tests trivial.
 */
export interface ProviderDeps {
  readonly analysisCache: DocumentAnalysisCache;
  /**
   * Current workspace-scoped import-path resolver. Source analysis
   * reads the same resolver through `analysisCache`; path-only style
   * features use this direct surface to avoid re-parsing settings.
   */
  readonly aliasResolver: AliasResolver;
  /**
   * Look up the style-document HIR for a style module file path.
   */
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly semanticReferenceIndex: SemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: StyleDependencyGraph;
  readonly workspaceRoot: string;
  readonly workspaceFolderUri: string;
  /**
   * Log a provider-level exception. Wired to
   * `connection.console.error` in production; tests pass
   * `NOOP_LOG_ERROR`. Required so the "log + return empty
   * result" contract is explicit at every call site.
   */
  readonly logError: (message: string, err: unknown) => void;
  /**
   * Invalidate the cached style-index entry for a single file path.
   * Used by the file-watcher when a `.module.*` file changes.
   */
  readonly invalidateStyle: (path: string) => void;
  readonly peekStyleDocument: (path: string) => StyleDocumentHIR | null;
  readonly buildStyleDocument: (path: string, content: string) => StyleDocumentHIR;
  /**
   * Return unsaved open-document text for a style file when the client has it.
   * Query-backed providers use this before disk reads so diagnostics and quick
   * fixes reflect the user's current buffer, not only saved files.
   */
  readonly readOpenDocumentText?: (path: string) => string | null;
  readonly readStyleFile: (path: string) => string | null;
  /**
   * Filesystem existence check. Used by recovery/setup workflows
   * that need to plan file creation without reaching into runtime
   * internals directly.
   */
  readonly fileExists: (path: string) => boolean;
  /**
   * Queue a single style file for incremental re-indexing by the
   * background indexer worker. Used by the file-watcher alongside
   * `invalidateStyle`.
   */
  readonly pushStyleFile: (path: string) => void;
  /**
   * Resolves when the initial indexer walk completes. Diagnostics
   * subscribers await this before running SCSS diagnostics that
   * depend on workspace-level reference data.
   */
  readonly indexerReady: Promise<void>;
  /**
   * Stop the background indexer worker. Called from `onShutdown`.
   */
  readonly stopIndexer: () => void;
  /**
   * Current extension settings. Plain mutable field — `reloadSettings`
   * replaces this on every `didChangeConfiguration`. Providers read
   * the field at call time so a config change between analyze calls
   * is observed on the next request.
   */
  settings: Settings;
  /**
   * Rebuild the workspace-scoped import-path resolver against the
   * latest extension `pathAlias` map plus the current tsconfig/jsconfig
   * `compilerOptions.paths` state. Callers (handler-registration's
   * reloadSettings and config-file watcher) MUST also clear cached
   * analysis so stale import resolutions are discarded. The resolver
   * itself lives inside `DocumentAnalysisCache` — no provider reads it
   * directly.
   */
  rebuildAliasResolver(pathAlias: Readonly<Record<string, string>>): void;
  /**
   * Ask the client to refresh visible CodeLens entries after the
   * semantic reference graph changes.
   */
  refreshCodeLens(): void;
}

/** No-op logError stub for tests — keeps `ProviderDeps.logError` required. */
export const NOOP_LOG_ERROR: (message: string, err: unknown) => void = () => {};
