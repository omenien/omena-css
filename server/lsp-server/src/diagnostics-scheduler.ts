import type { Connection } from "vscode-languageserver/node";
import type { TextDocument } from "vscode-languageserver-textdocument";
import type { TextDocuments } from "vscode-languageserver/node";
import { DiagnosticSeverity, type Diagnostic } from "vscode-languageserver/node";
import { computeDiagnostics } from "./providers/diagnostics";
import { computeScssUnusedDiagnostics } from "./providers/scss-diagnostics";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { fileUrlToPath } from "../../engine-core-ts/src/core/util/text-utils";
import { findLangForPath } from "../../engine-core-ts/src/core/scss/lang-registry";
import type { WindowSettings } from "../../engine-core-ts/src/settings";
import type { SelectorUsagePayloadCache } from "../../engine-host-node/src/selector-usage-query-backend";
import type {
  StyleSemanticGraphBatchOutputCache,
  StyleSemanticGraphCache,
} from "../../engine-host-node/src/style-semantic-graph-query-backend";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../engine-host-node/src/selected-query-backend";

type RuntimeProviderDeps = ProviderDeps & {
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache?: StyleSemanticGraphBatchOutputCache;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
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
          this.deps.connection.console.error(
            `[css-module-explainer] indexer readiness failed: ${message}`,
          );
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
    const diagnostics = computeScssUnusedDiagnostics(
      filePath,
      styleDocument,
      providerDeps.semanticReferenceIndex,
      providerDeps.styleDependencyGraph,
      providerDeps.styleDocumentForPath,
      {
        analysisCache: providerDeps.analysisCache,
        readStyleFile: providerDeps.readStyleFile,
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
        env: process.env,
      },
    );
    this.safeSendDiagnostics(uri, diagnostics);
  }

  private safeSendDiagnostics(uri: string, diagnostics: MaybePromise<readonly Diagnostic[]>): void {
    if (isPromiseLike(diagnostics)) {
      Promise.resolve(diagnostics)
        .then((resolved) => this.safeSendDiagnostics(uri, resolved))
        .catch((err: unknown) => {
          const message = err instanceof Error ? err.message : String(err);
          this.deps.connection.console.error(
            `[css-module-explainer] diagnostics failed: ${message}`,
          );
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
