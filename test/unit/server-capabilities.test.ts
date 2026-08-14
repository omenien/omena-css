import type { Connection } from "vscode-languageserver/node";
import { describe, expect, it, vi } from "vitest";
import {
  buildServerCapabilities,
  registerDynamicFileWatchers,
  resolveClientRuntimeCapabilities,
} from "../../server/lsp-server/src/server-capabilities";
import {
  isSourceFilePath,
  SOURCE_FILE_EXTENSIONS,
  SOURCE_FILE_WATCHER_GLOB,
} from "../../server/engine-core-ts/src/core/indexing/file-supplier";

describe("server capabilities", () => {
  it("builds the stable LSP capability surface", () => {
    const capabilities = buildServerCapabilities();

    expect(capabilities.textDocumentSync).toBe(2);
    expect(capabilities.definitionProvider).toBe(true);
    expect(capabilities.hoverProvider).toBe(true);
    expect(capabilities.referencesProvider).toBe(true);
    expect(capabilities.workspace?.workspaceFolders?.supported).toBe(true);
  });

  it("derives client runtime capability flags from initialize params", () => {
    const resolved = resolveClientRuntimeCapabilities({
      processId: null,
      capabilities: {
        workspace: {
          didChangeWatchedFiles: { dynamicRegistration: true },
          codeLens: { refreshSupport: true },
          workspaceFolders: true,
        },
      },
    });

    expect(resolved).toEqual({
      dynamicWatchers: true,
      codeLensRefresh: true,
      workspaceFolders: true,
    });
  });

  it("reports watcher registration success instead of assuming capability support", async () => {
    const dispose = vi.fn();
    const register = vi.fn().mockResolvedValue({ dispose });
    const connection = { client: { register } } as unknown as Connection;

    const result = await registerDynamicFileWatchers(connection, true);
    expect(result?.registered).toBe(true);
    expect(register).toHaveBeenCalledWith(expect.anything(), {
      watchers: expect.arrayContaining([{ globPattern: SOURCE_FILE_WATCHER_GLOB }]),
    });
    result?.dispose();
    expect(dispose).toHaveBeenCalledOnce();
  });

  it("derives source inventory recognition and watcher coverage from one extension set", () => {
    expect(SOURCE_FILE_WATCHER_GLOB).toBe(`**/*.{${SOURCE_FILE_EXTENSIONS.join(",")}}`);
    for (const extension of SOURCE_FILE_EXTENSIONS) {
      expect(isSourceFilePath(`/workspace/src/example.${extension}`)).toBe(true);
    }
    expect(isSourceFilePath("/workspace/src/example.json")).toBe(false);
  });

  it("reports registration failure and unsupported clients as incomplete coverage", async () => {
    const connection = {
      client: { register: vi.fn().mockRejectedValue(new Error("client rejected registration")) },
    } as unknown as Connection;

    await expect(registerDynamicFileWatchers(connection, true)).resolves.toMatchObject({
      registered: false,
    });
    expect(registerDynamicFileWatchers(connection, false)).toBeNull();
  });
});
