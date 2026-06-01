import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const root = process.cwd();
const sourcePath = path.join(
  root,
  "examples/src/scenarios/12-nested-style-facts/NestedStyleFactsScenario.tsx",
);
const nestedStylePath = path.join(
  root,
  "examples/src/scenarios/12-nested-style-facts/NestedStyleFacts.module.scss",
);
const aliasStylePath = path.join(root, "examples/src/scenarios/06-alias/Alias.module.scss");

const sourceText = readFileSync(sourcePath, "utf8");
const nestedStyleText = readFileSync(nestedStylePath, "utf8");
const aliasStyleText = readFileSync(aliasStylePath, "utf8");

const rootUri = pathToFileURL(root).href.replace(/\/$/, "");
const sourceUri = pathToFileURL(sourcePath).href;
const nestedStyleUri = pathToFileURL(nestedStylePath).href;
const aliasStyleUri = pathToFileURL(aliasStylePath).href;

const initializeRequest = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: null,
    rootUri,
    workspaceFolders: [{ uri: rootUri, name: "omena-css" }],
    capabilities: {},
  },
};

const messages = [
  initializeRequest,
  didOpen(aliasStyleUri, "scss", aliasStyleText),
  didOpen(nestedStyleUri, "scss", nestedStyleText),
  didOpen(sourceUri, "typescriptreact", sourceText),
  request(2, "textDocument/definition", sourceUri, {
    position: sourcePosition('cx("wrapper")', "wrapper"),
  }),
  request(3, "textDocument/hover", sourceUri, {
    position: sourcePosition('cx("item", "type-card"', "type-card"),
  }),
  request(4, "textDocument/definition", sourceUri, {
    position: sourcePosition('cx("item", "item--primary"', "item--primary"),
  }),
  request(5, "textDocument/definition", sourceUri, {
    position: sourcePosition('cx("item__icon")', "item__icon"),
  }),
  request(6, "textDocument/codeLens", nestedStyleUri),
  request(7, "shutdown", nestedStyleUri),
  { jsonrpc: "2.0", method: "exit" },
];

const invocation = resolveOmenaLspServerInvocation();
const result = spawnSync(invocation.command, invocation.args, {
  cwd: root,
  input: messages.map(frame).join(""),
  encoding: "utf8",
  stdio: ["pipe", "pipe", "pipe"],
});

assert.equal(
  result.status,
  0,
  [
    "omena-lsp-server examples regression failed",
    result.error ? `error=${result.error.message}` : null,
    result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
  ]
    .filter(Boolean)
    .join("\n"),
);

const protocolFrames = readFrames(result.stdout);
const responses = protocolFrames.filter((message) => "id" in message);
const diagnostics = protocolFrames.filter(
  (message) => message.method === "textDocument/publishDiagnostics",
);

const wrapperDefinition = responseById(responses, 2).result;
assert.deepEqual(wrapperDefinition, [
  {
    uri: nestedStyleUri,
    range: styleRange(".wrapper", "wrapper"),
  },
]);

const typeCardHover = responseById(responses, 3).result;
assert.equal(typeCardHover.contents.kind, "markdown");
assert.match(typeCardHover.contents.value, /`\.type-card`/);
assert.match(typeCardHover.contents.value, /NestedStyleFacts\.module\.scss/);
assert.match(typeCardHover.contents.value, /background: #eff6ff/);
assert.doesNotMatch(typeCardHover.contents.value, /Rust opened|document index/);

assert.deepEqual(responseById(responses, 4).result, [
  {
    uri: nestedStyleUri,
    range: styleRange("&--primary", "--primary"),
  },
]);
assert.deepEqual(responseById(responses, 5).result, [
  {
    uri: nestedStyleUri,
    range: styleRange("&__icon", "__icon"),
  },
]);

const finalSourceDiagnostics = lastDiagnosticsForUri(diagnostics, sourceUri);
assert.deepEqual(finalSourceDiagnostics, []);

const codeLenses = responseById(responses, 6).result;
assertCodeLensReferenceCount(codeLenses, ".item", "item", 3);
assertCodeLensReferenceCount(codeLenses, "&.type-card", "type-card", 1);
assertCodeLensReferenceCount(codeLenses, "&.compact", "compact", 1);
assertCodeLensReferenceCount(codeLenses, ".body", "body", 3);
assertCodeLensReferenceCount(codeLenses, "&.type-inline", "type-inline", 1);
assertCodeLensReferenceCount(codeLenses, "&.disabled", "disabled", 1);
assertCodeLensReferenceCount(codeLenses, "&--primary", "--primary", 1);
assertCodeLensReferenceCount(codeLenses, "&__icon", "__icon", 1);
assertCodeLensReferenceCount(codeLenses, ".wrapper", "wrapper", 1);
assertCodeLensReferenceCount(codeLenses, ".inner", "inner", 1);

process.stdout.write(
  [
    "validated omena-lsp-server examples regression:",
    `definitions=${wrapperDefinition.length + responseById(responses, 4).result.length + responseById(responses, 5).result.length}`,
    `codeLenses=${codeLenses.length}`,
    `sourceDiagnostics=${finalSourceDiagnostics.length}`,
  ].join(" "),
);
process.stdout.write("\n");

function didOpen(uri: string, languageId: string, text: string): unknown {
  return {
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri,
        languageId,
        version: 1,
        text,
      },
    },
  };
}

function request(id: number, method: string, uri: string, extraParams: object = {}): unknown {
  return {
    jsonrpc: "2.0",
    id,
    method,
    params: {
      textDocument: { uri },
      ...extraParams,
    },
  };
}

function sourcePosition(anchor: string, token: string): Position {
  return positionInText(sourceText, anchor, token);
}

function styleRange(anchor: string, token: string): Range {
  const start = rangeStartInText(nestedStyleText, anchor, token, 0);
  return {
    start,
    end: {
      line: start.line,
      character: start.character + token.length,
    },
  };
}

function rangeStartInText(text: string, anchor: string, token: string, fromIndex = 0): Position {
  const anchorStart = text.indexOf(anchor, fromIndex);
  assert.notEqual(anchorStart, -1, `missing anchor ${anchor}`);
  const tokenStart = text.indexOf(token, anchorStart);
  assert.notEqual(tokenStart, -1, `missing token ${token} after ${anchor}`);
  return positionForOffset(text, tokenStart);
}

function positionInText(text: string, anchor: string, token: string, fromIndex = 0): Position {
  const anchorStart = text.indexOf(anchor, fromIndex);
  assert.notEqual(anchorStart, -1, `missing anchor ${anchor}`);
  const tokenStart = text.indexOf(token, anchorStart);
  assert.notEqual(tokenStart, -1, `missing token ${token} after ${anchor}`);
  const offset = tokenStart + Math.floor(token.length / 2);
  return positionForOffset(text, offset);
}

function positionForOffset(text: string, offset: number): Position {
  const before = text.slice(0, offset).split("\n");
  return {
    line: before.length - 1,
    character: before.at(-1)?.length ?? 0,
  };
}

function assertCodeLensReferenceCount(
  actualCodeLenses: readonly any[],
  anchor: string,
  token: string,
  expectedCount: number,
): void {
  const position = styleRange(anchor, token).start;
  const lens = actualCodeLenses.find(
    (candidate) =>
      candidate.range.start.line === position.line &&
      candidate.range.start.character === position.character,
  );
  assert.ok(lens, `missing code lens for ${token}`);
  assert.equal(lens.command.title, referenceTitle(expectedCount));
  const locations = lens.command.arguments[2];
  assert.equal(locations.length, expectedCount, `unexpected location count for ${token}`);
  const locationKeys = new Set(
    locations.map((location: any) =>
      [
        location.uri,
        location.range.start.line,
        location.range.start.character,
        location.range.end.line,
        location.range.end.character,
      ].join(":"),
    ),
  );
  assert.equal(locationKeys.size, locations.length, `duplicate reference locations for ${token}`);
}

function referenceTitle(count: number): string {
  return count === 1 ? "1 reference" : `${count} references`;
}

function lastDiagnosticsForUri(diagnosticMessages: readonly any[], uri: string): any[] | undefined {
  for (let index = diagnosticMessages.length - 1; index >= 0; index -= 1) {
    const message = diagnosticMessages[index];
    if (message.params.uri === uri) {
      return message.params.diagnostics;
    }
  }
  return undefined;
}

function responseById(actualResponses: readonly any[], id: number): any {
  const response = actualResponses.find((candidate) => candidate.id === id);
  assert.ok(response, `missing response ${id}`);
  return response;
}

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
}

function readFrames(stdout: string): any[] {
  const parsedFrames: any[] = [];
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
    parsedFrames.push(JSON.parse(stdout.slice(bodyStart, bodyEnd)));
    offset = bodyEnd;
  }

  return parsedFrames;
}

interface Position {
  readonly line: number;
  readonly character: number;
}

interface Range {
  readonly start: Position;
  readonly end: Position;
}
