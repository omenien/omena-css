import * as nodeUrl from "node:url";
import type { StyleImport } from "@omena/shared";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import type { SourceBinderResult } from "../binder/scope-types";
import type { SourceDocumentHIR } from "../hir/source-types";
import { contentHash } from "../util/hash";
import { LruMap } from "../util/lru-map";
import type { SourceFileCache } from "../ts/source-file-cache";
import type { AliasResolver } from "../cx/alias-resolver";
import { collectSourceDependencyPaths } from "../ts/source-dependencies";
import type { ProjectedRustSourceBindingIndexV0 } from "../source-frontend/rust-binding-index-projection";

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
  readonly filePath: string;
  readonly sourceText: string;
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
  readonly sourceFrontendAnalysis: (
    input: SourceFrontendAnalysisProviderInputV0,
  ) => SourceFrontendAnalysisProviderResultV0 | null;
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

export interface SourceFrontendAnalysisProviderInputV0 {
  readonly filePath: string;
  readonly content: string;
}

export interface SourceFrontendAnalysisProviderResultV0 extends ProjectedRustSourceBindingIndexV0 {
  readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
}

/**
 * The single-parse hub for every provider hot path.
 *
 * `get(uri, content, filePath, version)` returns an AnalysisEntry
 * containing the TypeScript syntax tree needed by legacy consumers plus
 * the Rust source-frontend projection consumed by providers. Same-version
 * repeat calls are O(1), and a content-hash fallback catches the case where
 * the version bumped but the actual text is identical.
 *
 * This class is the single analysis-cache enforcement point. Providers never
 * call `ts.createSourceFile` or the source-frontend projection directly —
 * every analysis goes through this cache.
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
    const filePath = cached?.filePath;
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
    const sourceFrontendAnalysis = this.deps.sourceFrontendAnalysis({ filePath, content });
    if (!sourceFrontendAnalysis) {
      throw new Error(
        `Rust source frontend analysis is required for ${filePath}. Provide sourceFrontendAnalysis instead of relying on the retired TypeScript source frontend.`,
      );
    }

    return {
      version,
      contentHash: hash,
      filePath,
      sourceText: content,
      sourceBinder: sourceFrontendAnalysis.sourceBinder,
      sourceBindingGraph: sourceFrontendAnalysis.sourceBindingGraph,
      sourceDocument: sourceFrontendAnalysis.sourceDocument,
      stylesBindings: stylesBindingsFromSourceDocument(sourceFrontendAnalysis.sourceDocument),
      classUtilNames: classUtilNamesFromSourceDocument(sourceFrontendAnalysis.sourceDocument),
      classValueUniverses: sourceFrontendAnalysis.classValueUniverses ?? [],
      sourceDependencyPaths: collectSourceDependencyPaths(
        sourceFile,
        filePath,
        this.deps.aliasResolver,
      ),
    };
  }
}

function stylesBindingsFromSourceDocument(
  sourceDocument: SourceDocumentHIR,
): ReadonlyMap<string, StyleImport> {
  return new Map(
    sourceDocument.styleImports.map((styleImport) => [styleImport.localName, styleImport.resolved]),
  );
}

function classUtilNamesFromSourceDocument(sourceDocument: SourceDocumentHIR): readonly string[] {
  return sourceDocument.utilityBindings
    .filter((binding) => binding.kind === "classUtil")
    .map((binding) => binding.localName)
    .toSorted();
}
