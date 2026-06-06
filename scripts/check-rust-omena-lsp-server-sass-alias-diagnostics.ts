import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const STYLE_DIAGNOSTICS_REQUEST = "omena/rustStyleDiagnostics";
const workspacePath = mkdtempSync(path.join(os.tmpdir(), "omena-lsp-sass-alias-"));
const workspaceUri = pathToFileURL(workspacePath).href.replace(/\/$/u, "");
const sourceDir = path.join(workspacePath, "src");
const stylesDir = path.join(sourceDir, "styles");
const appStylePath = path.join(sourceDir, "App.module.scss");
const relativeStylePath = path.join(sourceDir, "Relative.module.scss");
const tokensStylePath = path.join(stylesDir, "_tokens.scss");
const relativeTokensStylePath = path.join(sourceDir, "_relative-tokens.scss");
const appStyleUri = pathToFileURL(appStylePath).href;
const relativeStyleUri = pathToFileURL(relativeStylePath).href;
const tokensStyleUri = pathToFileURL(tokensStylePath).href;
const relativeTokensStyleUri = pathToFileURL(relativeTokensStylePath).href;
const appStyleText =
  '@import "$styles/_tokens.scss";\n.button { color: $brand; padding: $missing; }';
const relativeStyleText =
  '@import "./relative-tokens";\n.button { color: $relative-brand; padding: $relative-missing; }';
const tokensStyleText = "$brand: red;";
const relativeTokensStyleText = "$relative-brand: blue;";

mkdirSync(stylesDir, { recursive: true });
writeFileSync(
  path.join(workspacePath, "tsconfig.json"),
  JSON.stringify({
    compilerOptions: {
      baseUrl: ".",
      paths: {
        "$styles/*": ["src/styles/*"],
      },
    },
  }),
);
writeFileSync(appStylePath, appStyleText);
writeFileSync(relativeStylePath, relativeStyleText);
writeFileSync(tokensStylePath, tokensStyleText);
writeFileSync(relativeTokensStylePath, relativeTokensStyleText);

try {
  const invocation = resolveOmenaLspServerInvocation();
  const result = spawnSync(invocation.command, [...invocation.args], {
    cwd: process.cwd(),
    input: [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          processId: process.pid,
          rootUri: workspaceUri,
          workspaceFolders: [{ uri: workspaceUri, name: "omena-lsp-sass-alias" }],
          capabilities: {},
        },
      },
      {
        jsonrpc: "2.0",
        method: "initialized",
        params: {},
      },
      didOpen(appStyleUri, appStyleText),
      didOpen(relativeStyleUri, relativeStyleText),
      didOpen(tokensStyleUri, tokensStyleText),
      didOpen(relativeTokensStyleUri, relativeTokensStyleText),
      styleDiagnosticsRequest(2, appStyleUri),
      styleDiagnosticsRequest(3, relativeStyleUri),
      {
        jsonrpc: "2.0",
        id: 4,
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
      "omena-lsp-server Sass alias diagnostics gate failed",
      result.error ? `error=${result.error.message}` : null,
      result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );

  const responses = readFrames(result.stdout).filter((message) => "id" in message);
  const aliasDiagnostics = responseById(responses, 2).result;
  const relativeDiagnostics = responseById(responses, 3).result;
  const aliasMissingSymbols = missingSassSymbolMessages(aliasDiagnostics);
  const relativeMissingSymbols = missingSassSymbolMessages(relativeDiagnostics);

  assert.deepEqual(
    aliasMissingSymbols,
    ["Sass variable '$missing' not found in the visible Sass module graph."],
    "tsconfig path aliases should expose imported Sass symbols without hiding unresolved controls",
  );
  assert.deepEqual(
    relativeMissingSymbols,
    ["Sass variable '$relative-missing' not found in the visible Sass module graph."],
    "relative Sass imports should keep resolving while preserving unresolved controls",
  );

  process.stdout.write(
    [
      "omena-lsp-server Sass alias diagnostics ok:",
      `command=${invocation.command}`,
      `aliasMissing=${aliasMissingSymbols.length}`,
      `relativeMissing=${relativeMissingSymbols.length}`,
    ].join(" ") + "\n",
  );
} finally {
  rmSync(workspacePath, { force: true, recursive: true });
}

function didOpen(uri: string, text: string): unknown {
  return {
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri,
        languageId: "scss",
        version: 1,
        text,
      },
    },
  };
}

function styleDiagnosticsRequest(id: number, uri: string): unknown {
  return {
    jsonrpc: "2.0",
    id,
    method: STYLE_DIAGNOSTICS_REQUEST,
    params: {
      textDocument: {
        uri,
      },
    },
  };
}

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

function missingSassSymbolMessages(diagnostics: unknown): string[] {
  assert.ok(Array.isArray(diagnostics), "style diagnostics response must be an array");
  return diagnostics
    .filter((diagnostic): diagnostic is { readonly code: string; readonly message: string } => {
      return (
        isRecord(diagnostic) &&
        diagnostic.code === "missingSassSymbol" &&
        typeof diagnostic.message === "string"
      );
    })
    .map((diagnostic) => diagnostic.message);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
