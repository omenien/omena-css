import * as vscode from "vscode";
import {
  LanguageClient,
  State,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";
import {
  buildTypeFactBackendEnv,
  readClientTypeFactBackendSetting,
} from "./type-fact-backend-config";
import {
  buildThinClientDocumentSelector,
  buildThinClientRuntimeEndpoint,
  buildThinClientServerOptions,
  readClientLspServerRuntimeSetting,
  resolveLspServerRuntimeSelection,
} from "./lsp-server-runtime-config";
import { isShowReferencesArgs } from "./util/show-references-guards";

const EXPLAIN_HOVER_TRACE_REQUEST = "omena/explainHoverTrace";
const EXPLAIN_HOVER_TRACE_COMMAND = "omena.explainHoverTrace";
const SHOW_SERVER_OUTPUT_COMMAND = "omena.showLanguageServerOutput";

let client: LanguageClient | undefined;
let clientReady: Promise<void> | undefined;
let serverStatusItem: vscode.StatusBarItem | undefined;

/**
 * Render the language-server lifecycle on the status bar item (omena-css#62
 * Phase 1). `State.StartFailed` (v10) and the pre-client resolution failure
 * both fold into the error rendering.
 */
interface OmenaWorkspaceStatus {
  readonly pendingFiles?: number;
  readonly indexedDocuments?: number;
  readonly settled?: boolean;
  readonly externalTokenSources?: number;
}

let lastWorkspaceStatus: OmenaWorkspaceStatus | undefined;

function renderServerStatus(state: State | "failed"): void {
  if (!serverStatusItem) return;
  switch (state) {
    case State.Starting:
      // A restart must not resurrect the previous session's status.
      lastWorkspaceStatus = undefined;
      serverStatusItem.text = "$(sync~spin) Omena";
      serverStatusItem.tooltip = "Omena CSS Modules language server is starting…";
      break;
    case State.Running:
      // The server's own workspace status (indexing progress, settle) is
      // the richer signal once it starts flowing; keep the plain Running
      // rendering only until the first omena/status arrives.
      if (lastWorkspaceStatus) {
        renderWorkspaceStatus(lastWorkspaceStatus);
        return;
      }
      serverStatusItem.text = "$(check) Omena";
      serverStatusItem.tooltip = "Omena CSS Modules language server is running.";
      break;
    default:
      // State.Stopped, State.StartFailed, or a pre-start failure.
      serverStatusItem.text = "$(error) Omena";
      serverStatusItem.tooltip =
        "Omena CSS Modules language server is not running. Click to open its output.";
      break;
  }
  serverStatusItem.show();
}

function renderWorkspaceStatus(status: OmenaWorkspaceStatus): void {
  if (!serverStatusItem) return;
  lastWorkspaceStatus = status;
  const pending = status.pendingFiles ?? 0;
  const indexed = status.indexedDocuments ?? 0;
  const tokens = status.externalTokenSources ?? 0;
  if (pending === 0 && indexed === 0) {
    // Pre-index window: the first status can arrive before the background
    // index deposits work — keep the starting spinner instead of
    // flickering ready → indexing → ready.
    serverStatusItem.text = "$(sync~spin) Omena";
    serverStatusItem.tooltip = "Omena CSS Modules language server is starting…";
    serverStatusItem.show();
    return;
  }
  if (pending > 0) {
    serverStatusItem.text = `$(sync~spin) Omena ${indexed}/${indexed + pending}`;
    serverStatusItem.tooltip = `Omena CSS Modules is indexing the workspace: ${indexed} documents admitted, ${pending} files pending. Click to open the server output.`;
  } else if (status.settled === false) {
    serverStatusItem.text = `$(sync~spin) Omena · settling`;
    serverStatusItem.tooltip = `Workspace index complete; external token refresh in flight. Click to open the server output.`;
  } else {
    serverStatusItem.text = "$(check) Omena";
    serverStatusItem.tooltip = `Omena CSS Modules is ready: ${indexed} documents indexed, ${tokens} external token sources. Click to open the server output.`;
  }
  serverStatusItem.show();
}

interface ExplainHoverTraceResponse {
  readonly product?: string;
  readonly fileKind?: string;
  readonly matched?: boolean;
  readonly reason?: string;
  readonly definitionCount?: number;
  readonly renderedMarkdown?: string;
  readonly resolutionPath?: readonly unknown[];
  readonly readySurfaces?: readonly unknown[];
  readonly [key: string]: unknown;
}

export function activate(context: vscode.ExtensionContext): void {
  serverStatusItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  serverStatusItem.name = "Omena CSS Modules";
  serverStatusItem.command = SHOW_SERVER_OUTPUT_COMMAND;
  context.subscriptions.push(serverStatusItem);
  context.subscriptions.push(
    vscode.commands.registerCommand(SHOW_SERVER_OUTPUT_COMMAND, () => {
      client?.outputChannel.show(true);
    }),
  );
  renderServerStatus(State.Starting);

  const typeFactBackend = readClientTypeFactBackendSetting(
    vscode.workspace.getConfiguration("omena").get("typeFactBackend"),
  );
  const serverEnv = buildTypeFactBackendEnv(typeFactBackend, process.env);
  const lspServerRuntime = readClientLspServerRuntimeSetting(
    vscode.workspace.getConfiguration("omena").get("lspServerRuntime"),
  );
  let runtimeSelection;
  try {
    runtimeSelection = resolveLspServerRuntimeSelection(
      lspServerRuntime,
      context.extensionPath,
      process.env,
    );
  } catch (err) {
    renderServerStatus("failed");
    void vscode.window.showErrorMessage(
      `Omena CSS Modules failed to resolve language-server runtime: ${
        err instanceof Error ? err.message : String(err)
      }`,
    );
    return;
  }
  const thinClientEndpoint = buildThinClientRuntimeEndpoint(
    runtimeSelection,
    context.extensionPath,
  );

  const serverOptions: ServerOptions = buildThinClientServerOptions(thinClientEndpoint, serverEnv);
  const rustLspFileEvents = thinClientEndpoint.fileWatcherGlobs.map((glob) =>
    vscode.workspace.createFileSystemWatcher(glob),
  );
  context.subscriptions.push(...rustLspFileEvents);

  const clientOptions: LanguageClientOptions = {
    documentSelector: buildThinClientDocumentSelector(),
    synchronize: {
      configurationSection: ["omena", "cssModules"],
      fileEvents: rustLspFileEvents,
    },
    outputChannelName: "Omena CSS Modules",
    progressOnInitialization: true,
  };

  client = new LanguageClient("omena", "Omena CSS Modules", serverOptions, {
    ...clientOptions,
    middleware: {
      provideCodeLenses: async (document, token, next) => {
        const lenses = await next(document, token);
        if (!lenses) return lenses;
        for (const lens of lenses) {
          if (lens.command?.command !== "editor.action.showReferences") continue;
          const args = lens.command.arguments;
          if (!args || !isShowReferencesArgs(args)) continue;
          try {
            const [uri, pos, locations] = args;
            lens.command.arguments = [
              vscode.Uri.parse(uri),
              new vscode.Position(pos.line, pos.character),
              locations.map(
                (loc) =>
                  new vscode.Location(
                    vscode.Uri.parse(loc.uri),
                    new vscode.Range(
                      loc.range.start.line,
                      loc.range.start.character,
                      loc.range.end.line,
                      loc.range.end.character,
                    ),
                  ),
              ),
            ];
          } catch {
            // Conversion failed — leave args as-is.
          }
        }
        return lenses;
      },
    },
  });

  context.subscriptions.push(
    client.onDidChangeState((event) => {
      renderServerStatus(event.newState);
    }),
  );
  context.subscriptions.push(
    client.onNotification("omena/status", (status: OmenaWorkspaceStatus) => {
      renderWorkspaceStatus(status);
    }),
  );

  clientReady = Promise.resolve(client.start());
  void clientReady.catch((err) => {
    renderServerStatus("failed");
    void vscode.window.showErrorMessage(
      `Omena CSS Modules failed to start: ${err instanceof Error ? err.message : String(err)}`,
    );
  });

  context.subscriptions.push(
    vscode.commands.registerCommand(EXPLAIN_HOVER_TRACE_COMMAND, async () => {
      await showHoverTracePanel(context);
    }),
  );

  context.subscriptions.push({
    dispose: () => {
      void client?.stop();
    },
  });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

async function showHoverTracePanel(context: vscode.ExtensionContext): Promise<void> {
  const activeClient = client;
  const ready = clientReady;
  if (!activeClient || !ready) {
    void vscode.window.showErrorMessage("Omena CSS Modules language server is not initialized.");
    return;
  }
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    void vscode.window.showInformationMessage(
      "Open a source or style document to explain hover trace.",
    );
    return;
  }

  try {
    await ready;
    const trace = await activeClient.sendRequest<ExplainHoverTraceResponse>(
      EXPLAIN_HOVER_TRACE_REQUEST,
      {
        textDocument: {
          uri: editor.document.uri.toString(),
        },
        position: {
          line: editor.selection.active.line,
          character: editor.selection.active.character,
        },
      },
    );
    showTracePanel(context, editor, trace);
  } catch (err) {
    void vscode.window.showErrorMessage(
      `Omena hover trace failed: ${err instanceof Error ? err.message : String(err)}`,
    );
  }
}

function showTracePanel(
  context: vscode.ExtensionContext,
  editor: vscode.TextEditor,
  trace: ExplainHoverTraceResponse,
): void {
  const panel = vscode.window.createWebviewPanel(
    "omenaHoverTrace",
    "Omena Hover Trace",
    vscode.ViewColumn.Beside,
    {
      enableScripts: false,
      localResourceRoots: [context.extensionUri],
      retainContextWhenHidden: true,
    },
  );
  panel.webview.html = renderTracePanelHtml(editor, trace);
}

function renderTracePanelHtml(editor: vscode.TextEditor, trace: ExplainHoverTraceResponse): string {
  const matched = trace.matched === true;
  const reason = typeof trace.reason === "string" ? trace.reason : "unknown";
  const fileKind = typeof trace.fileKind === "string" ? trace.fileKind : "unknown";
  const definitionCount =
    typeof trace.definitionCount === "number" ? String(trace.definitionCount) : "0";
  const renderedMarkdown =
    typeof trace.renderedMarkdown === "string" && trace.renderedMarkdown.trim()
      ? trace.renderedMarkdown
      : "(no rendered hover markdown)";
  const resolutionPath = trace.resolutionPath
    ?.filter((entry): entry is string => typeof entry === "string")
    .map((entry) => `<li>${escapeHtml(entry)}</li>`)
    .join("");
  const readySurfaces = trace.readySurfaces
    ?.filter((entry): entry is string => typeof entry === "string")
    .map((entry) => `<span class="chip">${escapeHtml(entry)}</span>`)
    .join("");

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Omena Hover Trace</title>
  <style>
    :root {
      color-scheme: light dark;
      --bg: var(--vscode-editor-background);
      --fg: var(--vscode-editor-foreground);
      --muted: var(--vscode-descriptionForeground);
      --border: var(--vscode-panel-border);
      --accent: var(--vscode-textLink-foreground);
      --code-bg: var(--vscode-textCodeBlock-background);
    }
    body {
      background: var(--bg);
      color: var(--fg);
      font: 13px/1.55 var(--vscode-font-family);
      margin: 0;
      padding: 24px;
    }
    h1 {
      font-size: 20px;
      margin: 0 0 8px;
    }
    h2 {
      font-size: 14px;
      margin: 24px 0 8px;
    }
    .muted {
      color: var(--muted);
    }
    .status {
      border: 1px solid var(--border);
      border-left: 4px solid var(--accent);
      border-radius: 8px;
      margin-top: 18px;
      padding: 14px;
    }
    .grid {
      display: grid;
      gap: 8px 18px;
      grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      margin-top: 12px;
    }
    .label {
      color: var(--muted);
      display: block;
      font-size: 11px;
      text-transform: uppercase;
    }
    .chip {
      border: 1px solid var(--border);
      border-radius: 999px;
      display: inline-block;
      margin: 0 6px 6px 0;
      padding: 2px 8px;
    }
    pre {
      background: var(--code-bg);
      border-radius: 8px;
      overflow: auto;
      padding: 14px;
      white-space: pre-wrap;
    }
  </style>
</head>
<body>
  <h1>Omena Hover Trace</h1>
  <div class="muted">${escapeHtml(editor.document.uri.toString())}:${editor.selection.active.line + 1}:${editor.selection.active.character + 1}</div>

  <section class="status">
    <strong>${matched ? "Matched" : "Not matched"}</strong>
    <div class="grid">
      <div><span class="label">File kind</span>${escapeHtml(fileKind)}</div>
      <div><span class="label">Reason</span>${escapeHtml(reason)}</div>
      <div><span class="label">Definitions</span>${escapeHtml(definitionCount)}</div>
      <div><span class="label">Product</span>${escapeHtml(trace.product ?? "unknown")}</div>
    </div>
  </section>

  <h2>Resolution Path</h2>
  <ol>${resolutionPath || "<li>(empty)</li>"}</ol>

  <h2>Ready Surfaces</h2>
  <div>${readySurfaces || '<span class="muted">(empty)</span>'}</div>

  <h2>Rendered Hover Markdown</h2>
  <pre>${escapeHtml(renderedMarkdown)}</pre>

  <h2>Raw Trace JSON</h2>
  <pre>${escapeHtml(JSON.stringify(trace, null, 2))}</pre>
</body>
</html>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
