import { afterEach, describe, expect, it, vi } from "vitest";
import { CodeLensRefreshRequest } from "vscode-languageserver-protocol/node";
import { createRuntimeSink } from "../../../server/lsp-server/src/server-runtime-support";

describe("server runtime support", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("debounces repeated CodeLens refresh requests", () => {
    vi.useFakeTimers();
    const sendRequest = vi.fn().mockResolvedValue(null);
    const sink = createRuntimeSink(makeConnection({ sendRequest }), true, {
      codeLensRefreshDebounceMs: 25,
    });

    sink.requestCodeLensRefresh();
    sink.requestCodeLensRefresh();
    sink.requestCodeLensRefresh();

    expect(sendRequest).not.toHaveBeenCalled();
    vi.advanceTimersByTime(24);
    expect(sendRequest).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);

    expect(sendRequest).toHaveBeenCalledTimes(1);
    expect(sendRequest).toHaveBeenCalledWith(CodeLensRefreshRequest.type);
  });

  it("does not schedule CodeLens refresh when the client does not support it", () => {
    vi.useFakeTimers();
    const sendRequest = vi.fn().mockResolvedValue(null);
    const sink = createRuntimeSink(makeConnection({ sendRequest }), false, {
      codeLensRefreshDebounceMs: 25,
    });

    sink.requestCodeLensRefresh();
    vi.advanceTimersByTime(25);

    expect(sendRequest).not.toHaveBeenCalled();
  });
});

function makeConnection(args: { readonly sendRequest: ReturnType<typeof vi.fn> }) {
  return {
    console: {
      info: vi.fn(),
      error: vi.fn(),
    },
    sendDiagnostics: vi.fn(),
    sendRequest: args.sendRequest,
  } as never;
}
