import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const EXPLAIN_HOVER_TRACE_REQUEST = "omena/explainHoverTrace";
const workspaceUri = "file:///tmp/cme-rust-lsp-explain-hover-trace";
const sourceUri = `${workspaceUri}/src/App.tsx`;
const styleUri = `${workspaceUri}/src/App.module.scss`;
const sourceText =
  'import bind from "classnames/bind";\nimport styles from "./App.module.scss";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx("foo")} />;\n';
const styleText = ".foo { color: red; }\n";

const requests = [
  {
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      processId: process.pid,
      rootUri: workspaceUri,
      workspaceFolders: [{ uri: workspaceUri, name: "cme-rust-lsp-explain-hover-trace" }],
      capabilities: {},
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri: styleUri,
        languageId: "scss",
        version: 1,
        text: styleText,
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
        text: sourceText,
      },
    },
  },
  {
    jsonrpc: "2.0",
    id: 2,
    method: EXPLAIN_HOVER_TRACE_REQUEST,
    params: {
      textDocument: {
        uri: sourceUri,
      },
      position: positionForNeedle(sourceText, "foo"),
    },
  },
  {
    jsonrpc: "2.0",
    id: 3,
    method: EXPLAIN_HOVER_TRACE_REQUEST,
    params: {
      textDocument: {
        uri: styleUri,
      },
      position: positionForNeedle(styleText, "foo"),
    },
  },
  {
    jsonrpc: "2.0",
    id: 4,
    method: "shutdown",
  },
  {
    jsonrpc: "2.0",
    method: "exit",
  },
];

const invocation = resolveOmenaLspServerInvocation();
const result = spawnSync(invocation.command, [...invocation.args], {
  cwd: process.cwd(),
  input: requests.map(frame).join(""),
  encoding: "utf8",
  stdio: ["pipe", "pipe", "pipe"],
});

assert.equal(
  result.status,
  0,
  [
    "omena-lsp-server explain hover trace failed",
    result.error ? `error=${result.error.message}` : null,
    result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
  ]
    .filter(Boolean)
    .join("\n"),
);

const responses = readFrames(result.stdout).filter((message) => "id" in message);
const sourceTrace = responseById(responses, 2).result;
const styleTrace = responseById(responses, 3).result;

assert.equal(sourceTrace.product, "omena-lsp-server.explain-hover-trace");
assert.equal(sourceTrace.fileKind, "source");
assert.equal(sourceTrace.matched, true);
assert.equal(sourceTrace.matchedCandidateCount, 1);
assert.equal(sourceTrace.definitionCount, 1);
assert.equal(sourceTrace.definitions[0].uri, styleUri);
assert.match(sourceTrace.renderedMarkdown, /color:\s*red/u);
assert.deepEqual(sourceTrace.resolutionPath, [
  "sourceSyntaxIndex",
  "sourceProviderCandidateResolution",
  "styleSelectorDefinitionResolver",
  "hoverMarkdownRenderer",
]);
assert.ok(sourceTrace.readySurfaces.includes("explainHoverTraceRpc"));

assert.equal(styleTrace.product, "omena-lsp-server.explain-hover-trace");
assert.equal(styleTrace.fileKind, "style");
assert.equal(styleTrace.matched, true);
assert.equal(styleTrace.candidateCount, 1);
assert.equal(styleTrace.definitionCount, 1);
assert.equal(styleTrace.definitions[0].name, "foo");
assert.match(styleTrace.renderedMarkdown, /color:\s*red/u);
assert.ok(styleTrace.readySurfaces.includes("explainHoverTraceRpc"));

process.stdout.write(
  [
    "omena-lsp-server explain hover trace ok:",
    `command=${invocation.command}`,
    `sourceDefinitions=${sourceTrace.definitionCount}`,
    `styleDefinitions=${styleTrace.definitionCount}`,
    `sourceReady=${sourceTrace.readySurfaces.join(",")}`,
  ].join(" ") + "\n",
);

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
}

function responseById(messages: readonly any[], id: number): any {
  const response = messages.find((message) => message.id === id);
  assert.ok(response, `missing response id ${id}`);
  assert.equal(response.error, undefined, `response id ${id} errored`);
  return response;
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

function positionForNeedle(
  text: string,
  needle: string,
): {
  readonly line: number;
  readonly character: number;
} {
  const offset = text.indexOf(needle);
  assert.notEqual(offset, -1, `fixture is missing ${needle}`);
  const prefix = text.slice(0, offset + 1);
  const lines = prefix.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)!.length - 1,
  };
}
