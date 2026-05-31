import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const REQUEST_CANCELLED_ERROR_CODE = -32800;

const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "cme-rust-lsp-cancellation-"));
const srcDir = path.join(workspaceRoot, "src");
const sourcePath = path.join(srcDir, "App.tsx");
const stylePath = path.join(srcDir, "App.module.scss");
const workspaceUri = pathToFileURL(workspaceRoot).toString();
const sourceUri = pathToFileURL(sourcePath).toString();
const styleUri = pathToFileURL(stylePath).toString();

try {
  mkdirSync(srcDir, { recursive: true });
  writeFileSync(
    sourcePath,
    [
      'import styles from "./App.module.scss";',
      "export const App = () => <div className={styles.root} />;",
    ].join("\n"),
  );
  writeFileSync(stylePath, ".root { color: red; }\n");

  const invocation = resolveOmenaLspServerInvocation();
  const result = spawnSync(invocation.command, [...invocation.args], {
    cwd: process.cwd(),
    input: [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          processId: null,
          rootUri: workspaceUri,
          workspaceFolders: [{ uri: workspaceUri, name: "cme-rust-lsp-cancellation" }],
          capabilities: {
            workspace: {
              workspaceFolders: true,
            },
          },
        },
      },
      {
        jsonrpc: "2.0",
        method: "initialized",
        params: {},
      },
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: styleUri,
            languageId: "scss",
            version: 1,
            text: ".root { color: red; }\n",
          },
        },
      },
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: sourceUri,
            languageId: "typescriptreact",
            version: 1,
            text: [
              'import styles from "./App.module.scss";',
              "export const App = () => <div className={styles.root} />;",
            ].join("\n"),
          },
        },
      },
      {
        jsonrpc: "2.0",
        method: "$/cancelRequest",
        params: {
          id: "cancelled-hover",
        },
      },
      {
        jsonrpc: "2.0",
        id: "cancelled-hover",
        method: "textDocument/hover",
        params: {
          textDocument: {
            uri: styleUri,
          },
          position: {
            line: 0,
            character: 2,
          },
        },
      },
      {
        jsonrpc: "2.0",
        id: "active-hover",
        method: "textDocument/hover",
        params: {
          textDocument: {
            uri: styleUri,
          },
          position: {
            line: 0,
            character: 2,
          },
        },
      },
      {
        jsonrpc: "2.0",
        id: "debug-state",
        method: "omena/rustLspState",
      },
      {
        jsonrpc: "2.0",
        id: "shutdown",
        method: "shutdown",
      },
      {
        jsonrpc: "2.0",
        method: "exit",
      },
    ]
      .map(frame)
      .join(""),
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });

  assert.equal(
    result.status,
    0,
    [
      "omena-lsp-server cancellation gate failed",
      result.error ? `error=${result.error.message}` : null,
      result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );

  const responses = readFrames(result.stdout).filter((message) => "id" in message);
  const initializeResponse = responseById(responses, 1);
  const cancelledHover = responseById(responses, "cancelled-hover");
  const activeHover = responseById(responses, "active-hover");
  const debugState = responseById(responses, "debug-state");
  const shutdown = responseById(responses, "shutdown");

  assert.equal(initializeResponse.error, undefined);
  assert.equal(cancelledHover.error?.code, REQUEST_CANCELLED_ERROR_CODE);
  assert.match(cancelledHover.error?.message ?? "", /cancelled/iu);
  assert.equal(activeHover.error, undefined);
  assert.ok(activeHover.result, "non-cancelled hover must still run after a cancelled request");
  assert.equal(debugState.result.cancelledRequestCount, 0);
  assert.equal(debugState.result.documentCount, 2);
  assert.equal(shutdown.result, null);

  process.stdout.write(
    [
      "validated omena-lsp-server cancellation:",
      `command=${invocation.command}`,
      `responses=${responses.length}`,
      `cancelledCode=${cancelledHover.error.code}`,
      `documents=${debugState.result.documentCount}`,
      `cancelledRequestCount=${debugState.result.cancelledRequestCount}`,
      `postCancelHover=${activeHover.result ? "ok" : "missing"}`,
    ].join(" "),
  );
  process.stdout.write("\n");
} finally {
  rmSync(workspaceRoot, { recursive: true, force: true });
}

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
}

function readFrames(stdout: string): any[] {
  const frames: any[] = [];
  let offset = 0;

  while (offset < stdout.length) {
    const headerEnd = stdout.indexOf("\r\n\r\n", offset);
    if (headerEnd < 0) break;
    const header = stdout.slice(offset, headerEnd);
    const match = /^Content-Length:\s*(\d+)$/imu.exec(header);
    assert.ok(match, `missing Content-Length in response header: ${header}`);
    const length = Number(match[1]);
    const bodyStart = headerEnd + 4;
    const bodyEnd = bodyStart + length;
    assert.ok(bodyEnd <= stdout.length, "incomplete response body");
    frames.push(JSON.parse(stdout.slice(bodyStart, bodyEnd)));
    offset = bodyEnd;
  }

  return frames;
}

function responseById(responses: any[], id: string | number): any {
  const response = responses.find((candidate) => candidate.id === id);
  assert.ok(response, `missing response ${id}`);
  return response;
}
