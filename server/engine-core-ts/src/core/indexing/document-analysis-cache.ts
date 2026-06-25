import * as nodeUrl from "node:url";
import type ts from "../../ts-facade";
import type { StyleImport } from "@omena/shared";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import { buildSourceBindingGraph } from "../binder/source-binding-graph";
import { composeBinderPluginsV0, type BinderPluginV0 } from "../binder/binder-plugin";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type { SourceBinderResult } from "../binder/scope-types";
import { buildSourceBinder } from "../binder/binder-builder";
import type { CxBinding } from "../cx/cx-types";
import { resolveCxBindings, type ResolvedCxBinding } from "../cx/resolved-bindings";
import { buildSourceDocument } from "../hir/builders/ts-source-adapter";
import type {
  ClassExpressionHIR,
  DomainClassReferenceHIR,
  SourceDocumentHIR,
} from "../hir/source-types";
import { contentHash } from "../util/hash";
import { LruMap } from "../util/lru-map";
import type { SourceFileCache } from "../ts/source-file-cache";
import type { AliasResolver } from "../cx/alias-resolver";
import { collectSourceDependencyPaths } from "../ts/source-dependencies";

/**
 * Single-parse analysis result for one TS/JS source file.
 *
 * Providers receive this object from `DocumentAnalysisCache.get`
 * and treat it as read-only. The `version` field mirrors VS Code's
 * `TextDocument.version` — cache hits on matching version are
 * O(1), with a content-hash fallback for the "same content, new
 * version" case that happens during incremental sync edge cases.
 */
export interface AnalysisEntry {
  readonly version: number;
  readonly contentHash: string;
  readonly sourceFile: ts.SourceFile;
  readonly sourceBinder: SourceBinderResult;
  readonly sourceBindingGraph: SourceBindingGraph;
  /**
   * Document-level source HIR derived from the current scan/parser
   * outputs.
   */
  readonly sourceDocument: SourceDocumentHIR;
  /**
   * Map of style-import local name → resolution outcome. The
   * `resolved` variant carries the absolute SCSS path; the
   * `missing` variant adds the raw specifier + LSP range so the
   * diagnostics provider can underline the broken import.
   */
  readonly stylesBindings: ReadonlyMap<string, StyleImport>;
  /**
   * Local identifiers bound to `clsx`, `clsx/lite`, or `classnames`
   * imports (NOT `classnames/bind`). Used by the completion provider
   * to detect whether the cursor sits inside a class-util call. Empty
   * when the file has no such imports.
   */
  readonly classUtilNames: readonly string[];
  readonly classValueUniverses: readonly ClassValueUniverseEntryV0[];
  readonly sourceDependencyPaths: readonly string[];
}

export interface DocumentAnalysisCacheDeps {
  readonly sourceFileCache: SourceFileCache;
  /**
   * Built-in class-name binding plugin entrypoint. Production
   * runtimes should prefer this over wiring cx/style scans
   * separately so future domains (Tailwind, vanilla-extract, Vue
   * modules) can enter through the same boundary.
   */
  readonly binderPlugin?: BinderPluginV0;
  readonly binderPlugins?: readonly BinderPluginV0[];
  /**
   * Single-pass scan of the file's top-level import declarations
   * and cx binding initializers. Returns both the style-import
   * map (with `resolved`/`missing` variants derived from
   * `fileExists`) and the active `cx = classnames.bind(styles)`
   * bindings in one traversal, eliminating the previous
   * double-walk pattern.
   */
  readonly scanCxImports?: (
    sourceFile: ts.SourceFile,
    filePath: string,
    fileExists: (p: string) => boolean,
    aliasResolver: AliasResolver,
  ) => {
    readonly stylesBindings: ReadonlyMap<string, StyleImport>;
    readonly bindings: readonly CxBinding[];
  };
  /**
   * Returns true iff `path` exists on disk. Injected so tests can
   * stub the check and the analysis cache stays free of `node:fs`.
   * Composition root wires `fs.existsSync`.
   */
  readonly fileExists: (path: string) => boolean;
  /**
   * Read-only accessor for the current workspace-scoped path-alias
   * resolver. Returns the latest resolver — `rebuildAliasResolver`
   * in composition root replaces the shared closure variable, so
   * `analyze()` always observes fresh alias config.
   */
  readonly aliasResolver: AliasResolver;
  /**
   * Direct source-expression parser. Optional for tests that do not
   * care about class-expression analysis; falls back to `[]`.
   */
  readonly parseClassExpressions?: (
    sourceFile: ts.SourceFile,
    bindings: readonly ResolvedCxBinding[],
    stylesBindings: ReadonlyMap<string, StyleImport>,
    sourceBinder: SourceBinderResult,
  ) => readonly ClassExpressionHIR[];
  /**
   * Detect `clsx` / `clsx/lite` / `classnames` (not `/bind`) imports
   * and return their local identifier names. Optional for test
   * helpers that don't care about completion; falls back to `[]`.
   */
  readonly detectClassUtilImports?: (sourceFile: ts.SourceFile) => readonly string[];
  readonly max: number;
  /**
   * Callback fired exactly once per (uri, version) when the cache
   * produces a fresh AnalysisEntry. Composition root wires this to
   * workspace-level reference stores so each document contributes
   * its resolved class-reference data once per document update —
   * not once per hover/def/completion keystroke.
   */
  readonly onAnalyze?: (uri: string, entry: AnalysisEntry) => void;
}

/**
 * The single-parse hub for every provider hot path.
 *
 * `get(uri, content, filePath, version)` returns an AnalysisEntry
 * containing the AST, bindings, and source expressions. The cache
 * guarantees that `ts.createSourceFile + detectCxBindings +
 * parseClassExpressions` run at most once per (uri, version) — same-version
 * repeat calls are O(1), and a content-hash fallback catches the
 * case where the version bumped but the actual text is identical.
 *
 * This class is the "one parse per file" enforcement point.
 * Providers never call `ts.createSourceFile` directly — every
 * analysis goes through this cache.
 */
export class DocumentAnalysisCache {
  private readonly lru: LruMap<string, AnalysisEntry>;
  private readonly deps: DocumentAnalysisCacheDeps;

  constructor(deps: DocumentAnalysisCacheDeps) {
    this.deps = deps;
    this.lru = new LruMap(deps.max);
  }

  get(uri: string, content: string, filePath: string, version: number): AnalysisEntry {
    const cached = this.lru.get(uri);
    if (cached && cached.version === version) {
      // Exact version match — cheapest hit.
      this.lru.touch(uri, cached);
      return cached;
    }
    const hash = contentHash(content);
    if (cached && cached.contentHash === hash) {
      // Content unchanged even though version bumped. Upgrade the
      // entry's version in place so subsequent exact-version hits
      // stay cheap, and keep the reference identity.
      const upgraded: AnalysisEntry = { ...cached, version };
      this.lru.touch(uri, upgraded);
      return upgraded;
    }
    const entry = this.analyze(content, filePath, version, hash);
    this.lru.set(uri, entry);
    // Single write point into workspace-level analysis side effects.
    this.deps.onAnalyze?.(uri, entry);
    return entry;
  }

  invalidate(uri: string): void {
    // Grab the path BEFORE deleting the entry so we can propagate
    // the invalidation to the SourceFileCache (which keys by
    // filePath, not uri).
    const cached = this.lru.get(uri);
    const filePath = cached?.sourceFile.fileName;
    this.lru.delete(uri);
    if (filePath !== undefined) {
      this.deps.sourceFileCache.invalidate(filePath);
      return;
    }
    // Fallback: no entry existed, derive the path from the uri.
    try {
      const derived = nodeUrl.fileURLToPath(uri);
      this.deps.sourceFileCache.invalidate(derived);
    } catch {
      // Malformed URI — nothing to invalidate anyway.
    }
  }

  clear(): void {
    this.lru.clear();
    this.deps.sourceFileCache.clear();
  }

  private analyze(content: string, filePath: string, version: number, hash: string): AnalysisEntry {
    const sourceFile = this.deps.sourceFileCache.get(filePath, content);
    const sourceBinder = buildSourceBinder(sourceFile);
    const plugin = this.resolveBinderPlugin();
    const pluginAnalysis = plugin?.analyzeSource({
      sourceFile,
      filePath,
      sourceBinder,
      fileExists: this.deps.fileExists,
      aliasResolver: this.deps.aliasResolver,
    });
    const legacyAnalysis = pluginAnalysis
      ? null
      : this.analyzeLegacyBindings(sourceFile, filePath, sourceBinder);
    const stylesBindings = pluginAnalysis?.stylesBindings ?? legacyAnalysis!.stylesBindings;
    const cxBindings = pluginAnalysis?.cxBindings ?? legacyAnalysis!.cxBindings;
    const classUtilNames = pluginAnalysis?.classUtilNames ?? legacyAnalysis!.classUtilNames;
    const classExpressions = pluginAnalysis?.classExpressions ?? legacyAnalysis!.classExpressions;
    const domainClassReferences =
      pluginAnalysis?.domainClassReferences ?? legacyAnalysis!.domainClassReferences;
    const classValueUniverses =
      pluginAnalysis?.classValueUniverses ?? legacyAnalysis!.classValueUniverses;
    const sourceDependencyPaths = collectSourceDependencyPaths(
      sourceFile,
      filePath,
      this.deps.aliasResolver,
    );
    const sourceDocument = buildSourceDocument({
      filePath,
      cxBindings,
      stylesBindings,
      classUtilNames,
      sourceBinder,
      classExpressions,
      domainClassReferences,
    });
    const sourceBindingGraph = buildSourceBindingGraph(sourceDocument, sourceBinder);

    return {
      version,
      contentHash: hash,
      sourceFile,
      sourceBinder,
      sourceBindingGraph,
      sourceDocument,
      stylesBindings,
      classUtilNames,
      classValueUniverses,
      sourceDependencyPaths,
    };
  }

  private analyzeLegacyBindings(
    sourceFile: ts.SourceFile,
    filePath: string,
    sourceBinder: SourceBinderResult,
  ): {
    readonly stylesBindings: ReadonlyMap<string, StyleImport>;
    readonly cxBindings: readonly ResolvedCxBinding[];
    readonly classUtilNames: readonly string[];
    readonly classExpressions: readonly ClassExpressionHIR[];
    readonly domainClassReferences: readonly DomainClassReferenceHIR[];
    readonly classValueUniverses: readonly ClassValueUniverseEntryV0[];
  } {
    if (!this.deps.scanCxImports) {
      throw new Error("DocumentAnalysisCache requires binderPlugin or scanCxImports");
    }
    // Legacy injection path kept for focused unit tests. Production
    // runtimes now route through BinderPluginV0.
    const { stylesBindings, bindings } = this.deps.scanCxImports(
      sourceFile,
      filePath,
      this.deps.fileExists,
      this.deps.aliasResolver,
    );
    const cxBindings = resolveCxBindings(bindings, sourceBinder, sourceFile);
    const classUtilNames = this.deps.detectClassUtilImports?.(sourceFile) ?? [];
    const classExpressions =
      this.deps.parseClassExpressions?.(sourceFile, cxBindings, stylesBindings, sourceBinder) ?? [];

    return {
      stylesBindings,
      cxBindings,
      classUtilNames,
      classExpressions,
      domainClassReferences: [],
      classValueUniverses: [],
    };
  }

  private resolveBinderPlugin(): BinderPluginV0 | null {
    if (this.deps.binderPlugins && this.deps.binderPlugins.length > 0) {
      return composeBinderPluginsV0(this.deps.binderPlugins);
    }
    return this.deps.binderPlugin ?? null;
  }
}
