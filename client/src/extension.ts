import * as vscode from "vscode";
import {
  LanguageClient,
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

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const typeFactBackend = readClientTypeFactBackendSetting(
    vscode.workspace.getConfiguration("cssModuleExplainer").get("typeFactBackend"),
  );
  const serverEnv = buildTypeFactBackendEnv(typeFactBackend, process.env);
  const lspServerRuntime = readClientLspServerRuntimeSetting(
    vscode.workspace.getConfiguration("cssModuleExplainer").get("lspServerRuntime"),
  );
  let runtimeSelection;
  try {
    runtimeSelection = resolveLspServerRuntimeSelection(
      lspServerRuntime,
      context.extensionPath,
      process.env,
    );
  } catch (err) {
    void vscode.window.showErrorMessage(
      `CSS Module Explainer failed to resolve language-server runtime: ${
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
      configurationSection: ["cssModuleExplainer", "cssModules"],
      fileEvents: rustLspFileEvents,
    },
    outputChannelName: "CSS Module Explainer",
    progressOnInitialization: true,
  };

  client = new LanguageClient("cssModuleExplainer", "CSS Module Explainer", serverOptions, {
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

  void client.start().catch((err) => {
    void vscode.window.showErrorMessage(
      `CSS Module Explainer failed to start: ${err instanceof Error ? err.message : String(err)}`,
    );
  });

  context.subscriptions.push({
    dispose: () => {
      void client?.stop();
    },
  });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
