import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { buildServerCapabilities } from "../server/lsp-server/src/server-capabilities";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const initializeRequest = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: null,
    rootUri: "file:///tmp/cme-rust-lsp-shell",
    workspaceFolders: [
      {
        uri: "file:///tmp/cme-rust-lsp-shell",
        name: "cme-rust-lsp-shell",
      },
    ],
    capabilities: {},
  },
};
const initializedNotification = {
  jsonrpc: "2.0",
  method: "initialized",
  params: {},
};
const didOpenSourceNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.tsx",
      languageId: "typescriptreact",
      version: 1,
      text: "import styles from \"./App.module.scss\"; const view = <div className={styles.root} />; const tone = 'blue';",
    },
  },
};
const didChangeSourceNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didChange",
  params: {
    textDocument: {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.tsx",
      version: 2,
    },
    contentChanges: [
      {
        range: {
          start: { line: 0, character: 100 },
          end: { line: 0, character: 104 },
        },
        text: "red",
      },
    ],
  },
};
const didOpenStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
      languageId: "scss",
      version: 1,
      text: ".root { color: var(--brand); } :root { --brand: red; }",
    },
  },
};
const didCloseStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didClose",
  params: {
    textDocument: {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
    },
  },
};
const didChangeWorkspaceFoldersNotification = {
  jsonrpc: "2.0",
  method: "workspace/didChangeWorkspaceFolders",
  params: {
    event: {
      removed: [
        {
          uri: "file:///tmp/cme-rust-lsp-shell",
          name: "cme-rust-lsp-shell",
        },
      ],
      added: [
        {
          uri: "file:///tmp/cme-rust-lsp-shell-next",
          name: "cme-rust-lsp-shell-next",
        },
      ],
    },
  },
};
const didChangeConfigurationNotification = {
  jsonrpc: "2.0",
  method: "workspace/didChangeConfiguration",
  params: {
    settings: {
      omena: {
        lspServerRuntime: "omena-lsp-server",
      },
    },
  },
};
const didChangeWatchedFilesNotification = {
  jsonrpc: "2.0",
  method: "workspace/didChangeWatchedFiles",
  params: {
    changes: [
      {
        uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
        type: 2,
      },
    ],
  },
};
const debugStateRequest = {
  jsonrpc: "2.0",
  id: 3,
  method: "omena/rustLspState",
};
const debugPostRuntimeChangeRequest = {
  jsonrpc: "2.0",
  id: 4,
  method: "omena/rustLspState",
};
const shutdownRequest = {
  jsonrpc: "2.0",
  id: 5,
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
    initializedNotification,
    didOpenSourceNotification,
    didChangeSourceNotification,
    didOpenStyleNotification,
    debugStateRequest,
    didCloseStyleNotification,
    didChangeWorkspaceFoldersNotification,
    didChangeConfigurationNotification,
    didChangeWatchedFilesNotification,
    debugPostRuntimeChangeRequest,
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
    "omena-lsp-server shell failed",
    result.error ? `error=${result.error.message}` : null,
    result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
  ]
    .filter(Boolean)
    .join("\n"),
);

const messages = readFrames(result.stdout);
const responses = messages.filter((message) => "id" in message);
const diagnosticNotifications = messages.filter(
  (message) => message.method === "textDocument/publishDiagnostics",
);
assert.equal(responses.length, 4);
assert.deepEqual(
  diagnosticNotifications.map((notification) => notification.params),
  [
    {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
      diagnostics: [],
    },
    {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
      diagnostics: [],
    },
    {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
      diagnostics: [],
    },
    {
      uri: "file:///tmp/cme-rust-lsp-shell/src/App.tsx",
      diagnostics: [
        {
          range: {
            start: { line: 0, character: 69 },
            end: { line: 0, character: 80 },
          },
          severity: 2,
          code: "unknownClassValueDomain",
          source: "omena-css",
          message:
            "CSS Module class value domain is unknown because tsgo could not find a project for this source.",
          data: {
            querySeverity: "warning",
            provenance: [
              "omena-query.source-syntax-index",
              "omena-tsgo-client.provider-capabilities",
              "tsgo-provider.unavailable->unknown-precision",
            ],
            precision: {
              product: "omena-query.analysis-precision",
              valueDomain: "unknown",
              flowSensitivity: "typeOracleProviderUnavailable",
              contextSensitivity: "perTypeFactTarget",
              revisionAxis: "OmenaQuerySourceDiagnosticsForFileV0.input",
            },
            pipelineTier: "optimizing",
            pipelineTierEvidence: "workspaceSourceDiagnosticsV0",
          },
        },
      ],
    },
  ],
);

const initializeResponse = responses[0]!;
assert.equal(initializeResponse.id, 1);
assert.deepEqual(initializeResponse.result.capabilities, buildServerCapabilities());
assert.deepEqual(initializeResponse.result.serverInfo, {
  name: "omena-css-rust",
});

const debugStateResponse = responses[1]!;
assert.equal(debugStateResponse.id, 3);
assert.equal(debugStateResponse.result.documentCount, 2);
assert.equal(debugStateResponse.result.workspaceFolderCount, 1);
assert.equal(debugStateResponse.result.configurationChangeCount, 0);
assert.equal(debugStateResponse.result.watchedFileEventCount, 0);
assert.deepEqual(debugStateResponse.result.documents, [
  {
    uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
    workspaceFolderUri: "file:///tmp/cme-rust-lsp-shell",
    languageId: "scss",
    version: 1,
    text: ".root { color: var(--brand); } :root { --brand: red; }",
    diagnosticsScheduleCount: 1,
    optimizingTierFeedback: null,
    styleSummary: {
      language: "scss",
      selectorNames: ["root"],
      customPropertyDeclNames: ["--brand"],
      customPropertyRefNames: ["--brand"],
      sassModuleUseSources: [],
      sassModuleForwardSources: [],
      diagnosticCount: 0,
    },
  },
  {
    uri: "file:///tmp/cme-rust-lsp-shell/src/App.tsx",
    workspaceFolderUri: "file:///tmp/cme-rust-lsp-shell",
    languageId: "typescriptreact",
    version: 2,
    text: "import styles from \"./App.module.scss\"; const view = <div className={styles.root} />; const tone = 'red';",
    diagnosticsScheduleCount: 3,
    optimizingTierFeedback: null,
    styleSummary: null,
  },
]);
assert.deepEqual(debugStateResponse.result.workspaceFolders, [
  {
    uri: "file:///tmp/cme-rust-lsp-shell",
    name: "cme-rust-lsp-shell",
  },
]);
assert.deepEqual(debugStateResponse.result.watchedFileChanges, []);

const runtimeChangeResponse = responses[2]!;
assert.equal(runtimeChangeResponse.id, 4);
assert.equal(runtimeChangeResponse.result.documentCount, 1);
assert.equal(runtimeChangeResponse.result.workspaceFolderCount, 1);
assert.equal(runtimeChangeResponse.result.configurationChangeCount, 1);
assert.equal(runtimeChangeResponse.result.watchedFileEventCount, 1);
assert.deepEqual(runtimeChangeResponse.result.documents, [
  {
    uri: "file:///tmp/cme-rust-lsp-shell/src/App.tsx",
    workspaceFolderUri: null,
    languageId: "typescriptreact",
    version: 2,
    text: "import styles from \"./App.module.scss\"; const view = <div className={styles.root} />; const tone = 'red';",
    diagnosticsScheduleCount: 6,
    optimizingTierFeedback: null,
    styleSummary: null,
  },
]);
assert.deepEqual(runtimeChangeResponse.result.workspaceFolders, [
  {
    uri: "file:///tmp/cme-rust-lsp-shell-next",
    name: "cme-rust-lsp-shell-next",
  },
]);
assert.deepEqual(runtimeChangeResponse.result.watchedFileChanges, [
  {
    uri: "file:///tmp/cme-rust-lsp-shell/src/App.module.scss",
    changeType: 2,
  },
]);

const shutdownResponse = responses[3]!;
assert.equal(shutdownResponse.id, 5);
assert.equal(shutdownResponse.result, null);

process.stdout.write(
  [
    "validated omena-lsp-server shell:",
    `command=${invocation.command}`,
    `responses=${responses.length}`,
    `diagnosticNotifications=${diagnosticNotifications.length}`,
    `documents=${runtimeChangeResponse.result.documentCount}`,
    `workspaceFolders=${runtimeChangeResponse.result.workspaceFolderCount}`,
    `configurationChanges=${runtimeChangeResponse.result.configurationChangeCount}`,
    `watchedFileEvents=${runtimeChangeResponse.result.watchedFileEventCount}`,
    `styleSelectors=${debugStateResponse.result.documents[0].styleSummary.selectorNames.length}`,
    `textDocumentSync=${initializeResponse.result.capabilities.textDocumentSync}`,
  ].join(" "),
);
process.stdout.write("\n");

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
