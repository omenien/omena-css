import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";
import {
  createProtocolConnection,
  DefinitionRequest,
  DidOpenTextDocumentNotification,
  HoverRequest,
  InitializedNotification,
  InitializeRequest,
  ShutdownRequest,
  type DefinitionParams,
  type HoverParams,
  type InitializeParams,
  type InitializeResult,
  type ProtocolConnection,
} from "vscode-languageserver-protocol/node";
import { StreamMessageReader, StreamMessageWriter } from "vscode-jsonrpc/node";

const REPO_ROOT = process.cwd();
const SERVER_ENTRY = path.join(REPO_ROOT, "dist/server/server.js");
const WORKSPACE_ROOT = path.join(REPO_ROOT, "test/_fixtures/lsp-stdio-smoke");
const SOURCE_PATH = path.join(WORKSPACE_ROOT, "src/App.tsx");
const SOURCE_TEXT = readFileSync(SOURCE_PATH, "utf8");
const SOURCE_URI = pathToFileURL(SOURCE_PATH).toString();

async function main(): Promise<void> {
  if (!existsSync(SERVER_ENTRY)) {
    throw new Error(`Built server not found at ${SERVER_ENTRY}. Run 'pnpm build' first.`);
  }

  const child = spawn(process.execPath, [SERVER_ENTRY, "--stdio"], {
    cwd: REPO_ROOT,
    stdio: ["pipe", "pipe", "pipe"],
  });

  const stderr: string[] = [];
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  const connection: ProtocolConnection = createProtocolConnection(
    new StreamMessageReader(child.stdout),
    new StreamMessageWriter(child.stdin),
  );

  connection.onRequest(
    "workspace/configuration",
    (params: { items: Array<{ section?: string; scopeUri?: string }> }) =>
      params.items.map(() => ({})),
  );

  connection.listen();

  let exitCode: number | null = null;
  try {
    const initResult = await requestWithTimeout(
      connection.sendRequest<InitializeResult>(InitializeRequest.type, initializeParams()),
      "initialize",
    );
    if (initResult.serverInfo?.name !== "omena-css") {
      throw new Error(`Unexpected server name: ${initResult.serverInfo?.name ?? "<missing>"}`);
    }

    connection.sendNotification(InitializedNotification.type, {});
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: SOURCE_URI,
        languageId: "typescriptreact",
        version: 1,
        text: SOURCE_TEXT,
      },
    });

    const hover = await requestWithTimeout(
      connection.sendRequest(HoverRequest.type, hoverParams("chip")),
      "hover",
    );
    const hoverValue =
      hover && typeof hover.contents === "object" && "value" in hover.contents
        ? hover.contents.value
        : "";
    if (!hoverValue.includes("`.chip`") || !hoverValue.includes("color: red;")) {
      throw new Error(`Unexpected hover payload:\n${hoverValue}`);
    }

    const definition = await requestWithTimeout(
      connection.sendRequest(DefinitionRequest.type, definitionParams("chip")),
      "definition",
    );
    if (!Array.isArray(definition) || definition.length !== 1) {
      throw new Error(`Unexpected definition payload: ${JSON.stringify(definition)}`);
    }
    const link = definition[0] as { targetUri?: string; uri?: string };
    const targetUri = link.targetUri ?? link.uri ?? "";
    if (!targetUri.endsWith("App.module.scss")) {
      throw new Error(`Definition target mismatch: ${targetUri}`);
    }

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");
  } finally {
    connection.dispose();
    child.kill();
    exitCode = await waitForExit(child);
  }

  if (exitCode !== null && exitCode !== 0) {
    throw new Error(`lsp-server exited with code ${exitCode}\n${stderr.join("")}`);
  }
}

function initializeParams(): InitializeParams {
  const workspaceUri = pathToFileURL(WORKSPACE_ROOT).toString();
  return {
    processId: process.pid,
    rootUri: workspaceUri,
    workspaceFolders: [{ uri: workspaceUri, name: "lsp-stdio-smoke" }],
    capabilities: {
      workspace: {
        configuration: true,
        workspaceFolders: true,
      },
    },
  };
}

function hoverParams(token: string): HoverParams {
  return {
    textDocument: { uri: SOURCE_URI },
    position: positionOfToken(SOURCE_TEXT, token),
  };
}

function definitionParams(token: string): DefinitionParams {
  return {
    textDocument: { uri: SOURCE_URI },
    position: positionOfToken(SOURCE_TEXT, token),
  };
}

function positionOfToken(text: string, token: string): { line: number; character: number } {
  const index = text.indexOf(`"${token}"`);
  if (index < 0) {
    throw new Error(`Token '${token}' not found.`);
  }
  const before = text.slice(0, index + 1);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)!.length,
  };
}

async function requestWithTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
  let timer: NodeJS.Timeout | null = null;
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 5000);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

function waitForExit(child: ReturnType<typeof spawn>): Promise<number | null> {
  return new Promise((resolve) => {
    child.once("exit", (code) => resolve(code));
    child.once("close", (code) => resolve(code));
  });
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});
