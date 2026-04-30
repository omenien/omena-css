import { CodeLensRefreshRequest } from "vscode-languageserver-protocol/node";
import type { Connection, TextDocuments } from "vscode-languageserver/node";
import type { TextDocument } from "vscode-languageserver-textdocument";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import type { RuntimeSink } from "../../engine-host-node/src/runtime";

export function readStyleTextFromOpenDocuments(
  path: string,
  documents: TextDocuments<TextDocument>,
): string | null {
  const uri = pathToFileUrl(path);
  const doc = documents.get(uri);
  return doc?.getText() ?? null;
}

export function createRuntimeSink(
  connection: Connection,
  supportsCodeLensRefresh: boolean,
  options: { readonly codeLensRefreshDebounceMs?: number } = {},
): RuntimeSink {
  const codeLensRefreshDebounceMs = options.codeLensRefreshDebounceMs ?? 50;
  let pendingCodeLensRefresh: ReturnType<typeof setTimeout> | null = null;

  const sendCodeLensRefresh = (): void => {
    pendingCodeLensRefresh = null;
    void connection.sendRequest(CodeLensRefreshRequest.type).catch(() => {});
  };

  return {
    info(message: string): void {
      connection.console.info(message);
    },
    error(message: string): void {
      connection.console.error(message);
    },
    clearDiagnostics(uri: string): void {
      connection.sendDiagnostics({ uri, diagnostics: [] });
    },
    requestCodeLensRefresh(): void {
      if (!supportsCodeLensRefresh) return;
      if (pendingCodeLensRefresh) return;
      pendingCodeLensRefresh = setTimeout(sendCodeLensRefresh, codeLensRefreshDebounceMs);
    },
  };
}
