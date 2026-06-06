import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const documentUri = "file:///tmp/cme-rust-lsp-diagnostics-coalescing/src/App.module.scss";
const initialStyleText =
  ":root { --brand: red; } .btn { width: var(--missing); color: red; color: blue; }";
const changedStyleText = ":root { --brand: red; } .btn { width: var(--missing); }";

const initializeRequest = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: null,
    rootUri: "file:///tmp/cme-rust-lsp-diagnostics-coalescing",
    workspaceFolders: [
      {
        uri: "file:///tmp/cme-rust-lsp-diagnostics-coalescing",
        name: "cme-rust-lsp-diagnostics-coalescing",
      },
    ],
    capabilities: {
      textDocument: {
        publishDiagnostics: {},
      },
    },
  },
};
const didOpenStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: documentUri,
      languageId: "scss",
      version: 1,
      text: initialStyleText,
    },
  },
};
const didChangeStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didChange",
  params: {
    textDocument: {
      uri: documentUri,
      version: 2,
    },
    contentChanges: [
      {
        text: changedStyleText,
      },
    ],
  },
};
const shutdownRequest = {
  jsonrpc: "2.0",
  id: 2,
  method: "shutdown",
};
const exitNotification = {
  jsonrpc: "2.0",
  method: "exit",
};

const invocation = resolveOmenaLspServerInvocation();
const result = spawnSync(invocation.command, [...invocation.args], {
  cwd: process.cwd(),
  input: [
    initializeRequest,
    didOpenStyleNotification,
    didChangeStyleNotification,
    shutdownRequest,
    exitNotification,
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
    "omena-lsp-server diagnostics coalescing gate failed",
    result.error ? `error=${result.error.message}` : null,
    result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
  ]
    .filter(Boolean)
    .join("\n"),
);

const messages = readFrames(result.stdout);
const diagnosticNotifications = messages.filter(
  (message) => message.method === "textDocument/publishDiagnostics",
);

assert.equal(diagnosticNotifications.length, 2);
for (const notification of diagnosticNotifications) {
  assert.equal(notification.params.uri, documentUri);
  assert.deepEqual(
    diagnosticCodes(notification),
    ["missingCustomProperty"],
    "stale optimizing diagnostics must not publish after a newer document change",
  );
  assert.equal(notification.params.diagnostics[0]?.data?.pipelineTier, "baseline");
}

process.stdout.write(
  [
    "validated omena-lsp-server diagnostics coalescing:",
    `command=${invocation.command}`,
    `diagnosticNotifications=${diagnosticNotifications.length}`,
    `codes=${diagnosticNotifications.map(diagnosticCodes).flat().join(",")}`,
  ].join(" ") + "\n",
);

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

function diagnosticCodes(notification: any): string[] {
  assert.ok(Array.isArray(notification.params?.diagnostics));
  return notification.params.diagnostics.map((diagnostic: any) => diagnostic.code).sort();
}
