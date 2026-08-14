import { readFileSync } from "node:fs";
import path from "node:path";
import type { Connection } from "vscode-languageserver/node";
import type { TextDocument } from "vscode-languageserver-textdocument";
import type { TextDocuments } from "vscode-languageserver/node";
import { DiagnosticSeverity, type Diagnostic } from "vscode-languageserver/node";
import { computeDiagnostics } from "./providers/diagnostics";
import { computeScssUnusedDiagnostics } from "./providers/scss-diagnostics";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { fileUrlToPath } from "../../engine-core-ts/src/core/util/text-utils";
import { findLangForPath } from "../../engine-core-ts/src/core/scss/lang-registry";
import { isSourceFilePath } from "../../engine-core-ts/src/core/indexing/file-supplier";
import type { WindowSettings } from "../../engine-core-ts/src/settings";
import type { SelectorUsagePayloadCache } from "../../engine-host-node/src/selector-usage-query-backend";
import type {
  StyleSemanticGraphBatchOutputCache,
  StyleSemanticGraphCache,
} from "../../engine-host-node/src/style-semantic-graph-query-backend";
import {
  resolveSelectedQueryBackendKind,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "../../engine-host-node/src/selected-query-backend";
import { loadExternalSifsForWorkspace } from "../../engine-host-node/src/external-sif-loader";
import type { SourceCorpusFileRead } from "../../engine-host-node/src/runtime/workspace-source-path-inventory";

type RuntimeProviderDeps = ProviderDeps & {
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache?: StyleSemanticGraphBatchOutputCache;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
  readonly readSourceFileForCorpus?: (filePath: string) => SourceCorpusFileRead | null;
};

const DIAGNOSTICS_DEBOUNCE_MS = 200;

export interface DiagnosticsSchedulerDeps {
  readonly connection: Connection;
  readonly documents: TextDocuments<TextDocument>;
  getDeps(uri: string): ProviderDeps | null;
  getAllDeps(): readonly ProviderDeps[];
}

export interface DiagnosticsScheduler {
  scheduleTsx(uri: string): void;
  scheduleScss(uri: string): void;
  shutdown(): void;
  refreshSettings(s: WindowSettings): void;
  /** Subscribe to indexer readiness so SCSS diagnostics fire after the initial walk. */
  ensureReadySubscribed(): void;
  /** Cancel pending timers for a closed document and clear its diagnostics. */
  handleDocumentClose(uri: string): void;
  sourceCorpusSupplyCounters(): SourceCorpusSupplyCounters;
}

export interface SourceCorpusSupplyCounters {
  readonly collections: number;
  readonly skippedForBackend: number;
  readonly incompleteCollections: number;
  readonly suppliedFiles: number;
  readonly suppliedBytes: number;
  readonly diskReadFiles: number;
  readonly diskReadBytes: number;
  readonly cacheHitFiles: number;
}

interface MutableSourceCorpusSupplyCounters {
  collections: number;
  skippedForBackend: number;
  incompleteCollections: number;
  suppliedFiles: number;
  suppliedBytes: number;
  diskReadFiles: number;
  diskReadBytes: number;
  cacheHitFiles: number;
}

const SEVERITY_MAP: Record<string, DiagnosticSeverity> = {
  error: DiagnosticSeverity.Error,
  warning: DiagnosticSeverity.Warning,
  information: DiagnosticSeverity.Information,
  hint: DiagnosticSeverity.Hint,
};

function parseSeverity(value: string): DiagnosticSeverity {
  return SEVERITY_MAP[value] ?? DiagnosticSeverity.Warning;
}

export function createDiagnosticsScheduler(
  deps: DiagnosticsSchedulerDeps,
  settings: WindowSettings,
): DiagnosticsScheduler {
  return new DiagnosticsSchedulerImpl(deps, settings);
}

/**
 * Encapsulates TSX and SCSS diagnostic debounce/timer logic so
 * that handler-registration stays a thin routing table.
 *
 * The debounce skeleton (cancel existing timer → arm new one) is
 * centralized in `debounce()`; `runTsxDiagnostics` and
 * `runScssDiagnostics` hold only the compute-and-publish bodies.
 */
class DiagnosticsSchedulerImpl implements DiagnosticsScheduler {
  private readonly tsxTimers = new Map<string, NodeJS.Timeout>();
  private readonly scssTimers = new Map<string, NodeJS.Timeout>();
  private currentSettings: WindowSettings;
  private indexReady = false;
  private stopped = false;
  private readonly readySubscribed = new Set<string>();
  private readonly sourceCorpusCounters: MutableSourceCorpusSupplyCounters = {
    collections: 0,
    skippedForBackend: 0,
    incompleteCollections: 0,
    suppliedFiles: 0,
    suppliedBytes: 0,
    diskReadFiles: 0,
    diskReadBytes: 0,
    cacheHitFiles: 0,
  };

  constructor(
    private readonly deps: DiagnosticsSchedulerDeps,
    settings: WindowSettings,
  ) {
    this.currentSettings = settings;
  }

  scheduleTsx(uri: string): void {
    this.debounce(this.tsxTimers, uri, () => this.runTsxDiagnostics(uri));
  }

  scheduleScss(uri: string): void {
    this.ensureReadySubscribed();
    if (!this.indexReady) return;
    if (!this.currentSettings.diagnostics.unusedSelector) return;
    this.debounce(this.scssTimers, uri, () => this.runScssDiagnostics(uri));
  }

  shutdown(): void {
    this.stopped = true;
    clearAll(this.tsxTimers);
    clearAll(this.scssTimers);
  }

  refreshSettings(s: WindowSettings): void {
    this.currentSettings = s;
  }

  sourceCorpusSupplyCounters(): SourceCorpusSupplyCounters {
    return { ...this.sourceCorpusCounters };
  }

  ensureReadySubscribed(): void {
    const providerDeps = this.deps.getAllDeps();
    if (providerDeps.length === 0) return;
    for (const deps of providerDeps) {
      if (this.readySubscribed.has(deps.workspaceFolderUri)) continue;
      this.readySubscribed.add(deps.workspaceFolderUri);
      deps.indexerReady
        .then(() => {
          if (this.stopped) return;
          this.indexReady = true;
          for (const doc of this.deps.documents.all()) {
            if (findLangForPath(fileUrlToPath(doc.uri))) {
              this.scheduleScss(doc.uri);
            }
          }
        })
        .catch((err: unknown) => {
          const message = err instanceof Error ? err.message : String(err);
          this.deps.connection.console.error(`[omena-css] indexer readiness failed: ${message}`);
        });
    }
  }

  handleDocumentClose(uri: string): void {
    cancelTimer(this.tsxTimers, uri);
    cancelTimer(this.scssTimers, uri);
    this.safeSendDiagnostics(uri, []);
  }

  private debounce(timers: Map<string, NodeJS.Timeout>, uri: string, run: () => void): void {
    const existing = timers.get(uri);
    if (existing) clearTimeout(existing);
    const timer = setTimeout(() => {
      timers.delete(uri);
      if (this.stopped) return;
      run();
    }, DIAGNOSTICS_DEBOUNCE_MS);
    timers.set(uri, timer);
  }

  private runTsxDiagnostics(uri: string): void {
    const providerDeps = this.deps.getDeps(uri);
    const doc = this.deps.documents.get(uri);
    if (!providerDeps || !doc) return;
    const severity = parseSeverity(this.currentSettings.diagnostics.severity);
    const diagnostics = computeDiagnostics(
      {
        documentUri: uri,
        content: doc.getText(),
        filePath: fileUrlToPath(uri),
        version: doc.version,
      },
      providerDeps,
      severity,
    );
    this.safeSendDiagnostics(uri, diagnostics);
  }

  private runScssDiagnostics(uri: string): void {
    const providerDeps = this.deps.getDeps(uri);
    const doc = this.deps.documents.get(uri);
    if (!providerDeps || !doc) return;
    const runtimeProviderDeps = providerDeps as RuntimeProviderDeps;
    const filePath = fileUrlToPath(uri);
    const styleDocument = providerDeps.styleDocumentForPath(filePath);
    if (!styleDocument) return;
    const sourceCorpus = this.shouldCollectSourceCorpus(runtimeProviderDeps)
      ? this.collectSourceCorpus(runtimeProviderDeps, filePath)
      : null;
    if (sourceCorpus === null) {
      this.sourceCorpusCounters.skippedForBackend += 1;
    } else {
      this.recordSourceCorpusSupply(sourceCorpus);
    }
    const diagnostics = computeScssUnusedDiagnostics(
      filePath,
      styleDocument,
      providerDeps.semanticReferenceIndex,
      providerDeps.styleDependencyGraph,
      providerDeps.styleDocumentForPath,
      {
        analysisCache: providerDeps.analysisCache,
        buildStyleDocument: providerDeps.buildStyleDocument,
        ...(providerDeps.readOpenDocumentText
          ? { readOpenDocumentText: providerDeps.readOpenDocumentText }
          : {}),
        readStyleFile: providerDeps.readStyleFile,
        styleDocumentForPath: providerDeps.styleDocumentForPath,
        typeResolver: providerDeps.typeResolver,
        workspaceRoot: providerDeps.workspaceRoot,
        settings: providerDeps.settings,
        aliasResolver: providerDeps.aliasResolver,
        styleSource: doc.getText(),
        ...(runtimeProviderDeps.styleSemanticGraphCache
          ? { styleSemanticGraphCache: runtimeProviderDeps.styleSemanticGraphCache }
          : {}),
        ...(runtimeProviderDeps.styleSemanticGraphBatchOutputCache
          ? {
              styleSemanticGraphBatchOutputCache:
                runtimeProviderDeps.styleSemanticGraphBatchOutputCache,
            }
          : {}),
        ...(runtimeProviderDeps.selectorUsagePayloadCache
          ? { selectorUsagePayloadCache: runtimeProviderDeps.selectorUsagePayloadCache }
          : {}),
        ...(runtimeProviderDeps.runRustSelectedQueryBackendJsonAsync
          ? {
              runRustSelectedQueryBackendJsonAsync:
                runtimeProviderDeps.runRustSelectedQueryBackendJsonAsync,
            }
          : {}),
        ...(sourceCorpus ? { sourceDocuments: sourceCorpus.documents } : {}),
        ...(sourceCorpus !== null && sourceCorpus.completeSourcePathEnumeration !== null
          ? {
              completeSourcePathEnumeration: sourceCorpus.completeSourcePathEnumeration,
            }
          : {}),
        // Source external SIFs from the workspace `omena.lock` so the engine
        // wire leaves `Ignored` mode (#32). Empty set => identical behaviour
        // to before (no externalMode/externalSifs forwarded).
        ...this.resolveExternalSifDeps(providerDeps.workspaceRoot),
        env: process.env,
      },
    );
    this.safeSendDiagnostics(uri, diagnostics);
  }

  private shouldCollectSourceCorpus(providerDeps: RuntimeProviderDeps): boolean {
    return (
      providerDeps.runRustSelectedQueryBackendJsonAsync !== undefined &&
      resolveSelectedQueryBackendKind(process.env) === "rust-selected-query"
    );
  }

  private recordSourceCorpusSupply(sourceCorpus: SourceCorpusCollection): void {
    this.sourceCorpusCounters.collections += 1;
    if (sourceCorpus.completeSourcePathEnumeration === null) {
      this.sourceCorpusCounters.incompleteCollections += 1;
    }
    this.sourceCorpusCounters.suppliedFiles += sourceCorpus.documents.length;
    this.sourceCorpusCounters.suppliedBytes += sourceCorpus.suppliedBytes;
    this.sourceCorpusCounters.diskReadFiles += sourceCorpus.diskReadFiles;
    this.sourceCorpusCounters.diskReadBytes += sourceCorpus.diskReadBytes;
    this.sourceCorpusCounters.cacheHitFiles += sourceCorpus.cacheHitFiles;
  }

  private resolveExternalSifDeps(workspaceRoot: string | undefined):
    | {
        readonly externalMode: "sif";
        readonly externalSifs: ReturnType<typeof loadExternalSifsForWorkspace>;
      }
    | Record<string, never> {
    const externalSifs = loadExternalSifsForWorkspace(workspaceRoot);
    if (externalSifs.length === 0) return {};
    return { externalMode: "sif", externalSifs };
  }

  private collectReferencingSourceDocuments(
    providerDeps: RuntimeProviderDeps,
    stylePath: string,
  ): SourceCorpusCollection {
    const referencingUris = providerDeps.semanticReferenceIndex.findReferencingUris(stylePath);
    const documents: SourceCorpusDocument[] = [];
    let suppliedBytes = 0;
    let diskReadFiles = 0;
    let diskReadBytes = 0;
    let cacheHitFiles = 0;
    for (const uri of referencingUris) {
      const filePath = fileUrlToPath(uri);
      const openDocument = this.deps.documents.get(uri);
      if (openDocument) {
        const sourceSource = openDocument.getText();
        suppliedBytes += Buffer.byteLength(sourceSource, "utf8");
        documents.push({ sourcePath: filePath, sourceSource });
        continue;
      }
      const read = this.readSourceFileForCorpus(providerDeps, filePath);
      if (!read) continue;
      suppliedBytes += read.utf8Bytes;
      if (read.cacheHit) {
        cacheHitFiles += 1;
      } else {
        diskReadFiles += 1;
        diskReadBytes += read.utf8Bytes;
      }
      documents.push({ sourcePath: filePath, sourceSource: read.source });
    }
    return {
      documents,
      completeSourcePathEnumeration: null,
      suppliedBytes,
      diskReadFiles,
      diskReadBytes,
      cacheHitFiles,
    };
  }

  private collectSourceCorpus(
    providerDeps: RuntimeProviderDeps,
    stylePath: string,
  ): SourceCorpusCollection {
    const diskEnumeration = providerDeps.completeSourcePathEnumeration?.() ?? null;
    if (diskEnumeration === null) {
      return this.collectReferencingSourceDocuments(providerDeps, stylePath);
    }

    const openSources = new Map<string, string>();
    const sourcePaths = new Set(diskEnumeration.map((filePath) => path.resolve(filePath)));
    for (const document of this.deps.documents.all()) {
      let filePath: string;
      try {
        filePath = path.resolve(fileUrlToPath(document.uri));
      } catch {
        continue;
      }
      if (!isWithinWorkspace(filePath, providerDeps.workspaceRoot)) continue;
      if (!isSourceFilePath(filePath) || findLangForPath(filePath)) continue;
      sourcePaths.add(filePath);
      openSources.set(filePath, document.getText());
    }

    const completeSourcePathEnumeration = [...sourcePaths].toSorted(compareCodeUnits);
    const documents: SourceCorpusDocument[] = [];
    let suppliedBytes = 0;
    let diskReadFiles = 0;
    let diskReadBytes = 0;
    let cacheHitFiles = 0;
    for (const sourcePath of completeSourcePathEnumeration) {
      const openSource = openSources.get(sourcePath);
      if (openSource !== undefined) {
        suppliedBytes += Buffer.byteLength(openSource, "utf8");
        documents.push({ sourcePath, sourceSource: openSource });
        continue;
      }
      const read = this.readSourceFileForCorpus(providerDeps, sourcePath);
      if (!read) continue;
      suppliedBytes += read.utf8Bytes;
      if (read.cacheHit) {
        cacheHitFiles += 1;
      } else {
        diskReadFiles += 1;
        diskReadBytes += read.utf8Bytes;
      }
      documents.push({ sourcePath, sourceSource: read.source });
    }
    return {
      documents,
      completeSourcePathEnumeration,
      suppliedBytes,
      diskReadFiles,
      diskReadBytes,
      cacheHitFiles,
    };
  }

  private readSourceFileForCorpus(
    providerDeps: RuntimeProviderDeps,
    sourcePath: string,
  ): SourceCorpusFileRead | null {
    if (providerDeps.readSourceFileForCorpus) {
      return providerDeps.readSourceFileForCorpus(sourcePath);
    }
    try {
      const source = readFileSync(sourcePath, "utf8");
      return {
        source,
        utf8Bytes: Buffer.byteLength(source, "utf8"),
        cacheHit: false,
      };
    } catch {
      return null;
    }
  }

  private safeSendDiagnostics(uri: string, diagnostics: MaybePromise<readonly Diagnostic[]>): void {
    if (isPromiseLike(diagnostics)) {
      Promise.resolve(diagnostics)
        .then((resolved) => this.safeSendDiagnostics(uri, resolved))
        .catch((err: unknown) => {
          const message = err instanceof Error ? err.message : String(err);
          this.deps.connection.console.error(`[omena-css] diagnostics failed: ${message}`);
          this.safeSendDiagnostics(uri, []);
        });
      return;
    }
    if (this.stopped) return;
    try {
      this.deps.connection.sendDiagnostics({ uri, diagnostics: [...diagnostics] });
    } catch {
      this.stopped = true;
    }
  }
}

interface SourceCorpusDocument {
  readonly sourcePath: string;
  readonly sourceSource: string;
}

interface SourceCorpusCollection {
  readonly documents: readonly SourceCorpusDocument[];
  readonly completeSourcePathEnumeration: readonly string[] | null;
  readonly suppliedBytes: number;
  readonly diskReadFiles: number;
  readonly diskReadBytes: number;
  readonly cacheHitFiles: number;
}

function isWithinWorkspace(filePath: string, workspaceRoot: string): boolean {
  const relative = path.relative(workspaceRoot, filePath);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function compareCodeUnits(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

type MaybePromise<T> = T | PromiseLike<T>;

function isPromiseLike<T>(value: MaybePromise<T>): value is PromiseLike<T> {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as { then?: unknown }).then === "function"
  );
}

function cancelTimer(timers: Map<string, NodeJS.Timeout>, uri: string): void {
  const existing = timers.get(uri);
  if (existing) {
    clearTimeout(existing);
    timers.delete(uri);
  }
}

function clearAll(timers: Map<string, NodeJS.Timeout>): void {
  for (const timer of timers.values()) clearTimeout(timer);
  timers.clear();
}
