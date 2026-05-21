import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type JsonRpcMessage = {
  readonly id?: number;
  readonly result?: unknown;
  readonly error?: unknown;
};

const repoRoot = process.cwd();
const platformDir = `${process.platform}-${process.arch}`;
const serverBinaryName = process.platform === "win32" ? "omena-lsp-server.exe" : "omena-lsp-server";
const serverPath = path.join(repoRoot, "dist", "bin", platformDir, serverBinaryName);

if (!existsSync(serverPath)) {
  throw new Error(
    `Missing built omena-lsp-server binary at ${serverPath}. Run pnpm cme-check run core/build/omena-lsp-server first.`,
  );
}

const workspaceRoot = mkdtempSync(path.join(tmpdir(), "cme-rust-lsp-typefacts-"));
const srcDir = path.join(workspaceRoot, "src");
mkdirSync(srcDir, { recursive: true });

const sourceText = `import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
// 기준 size와 font-size가 다르게 선언된 경우에도 tsgo positions must stay aligned.
interface Props { size: "medium" | "small"; fontSize?: 10 | 12; weight?: "bold" | "medium"; }
export function Badge({ size, fontSize }: Props) {
  return <span className={cx(size, \`font-size-\${fontSize}\`)} />;
}
`;
const styleText = `.medium { color: red; }
.small { color: blue; }
.font-size-10 { font-size: 10px; }
.font-size-12 { font-size: 12px; }
`;
const lateSourceText = `import bind from "classnames/bind";
import styles from "./Late.module.scss";
const cx = bind.bind(styles);
interface Props { tone: "info" | "warn"; }
export function LateBadge({ tone }: Props) {
  return <span className={cx(tone)} />;
}
`;
const lateStyleText = `.info { color: blue; }
.warn { color: orange; }
`;

async function main(): Promise<void> {
  try {
    writeFixtureProject();
    await runTypeFactProtocolSmoke();
  } finally {
    rmSync(workspaceRoot, { recursive: true, force: true });
  }
}

function writeFixtureProject(): void {
  writeFileSync(
    path.join(workspaceRoot, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          jsx: "preserve",
          module: "esnext",
          moduleResolution: "bundler",
          target: "es2022",
          allowArbitraryExtensions: true,
        },
        include: ["src"],
      },
      null,
      2,
    ),
  );
  writeFileSync(
    path.join(srcDir, "global.d.ts"),
    [
      'declare module "*.module.scss" {',
      "  const classes: Record<string, string>;",
      "  export default classes;",
      "}",
      "declare namespace JSX { interface IntrinsicElements { span: any } }",
      "",
    ].join("\n"),
  );
  writeFileSync(path.join(srcDir, "App.tsx"), sourceText);
  writeFileSync(path.join(srcDir, "App.module.scss"), styleText);
  writeFileSync(path.join(srcDir, "Late.tsx"), lateSourceText);
  for (let index = 0; index < 140; index += 1) {
    writeFileSync(
      path.join(srcDir, `Synthetic${index}.tsx`),
      `export const synthetic${index} = ${index};\n`,
    );
  }
}

async function runTypeFactProtocolSmoke(): Promise<void> {
  const sourceUri = fileUri(path.join(srcDir, "App.tsx"));
  const styleUri = fileUri(path.join(srcDir, "App.module.scss"));
  const sizePosition = positionForOffset(sourceText, sourceText.indexOf("cx(size") + "cx(".length);
  const fontSizePosition = positionForOffset(sourceText, sourceText.lastIndexOf("fontSize"));
  const client = new JsonRpcClient(serverPath, repoRoot);

  try {
    await client.request("initialize", {
      processId: process.pid,
      rootUri: fileUri(workspaceRoot),
      workspaceFolders: [{ uri: fileUri(workspaceRoot), name: "typefacts" }],
      capabilities: {},
    });
    client.notify("initialized", {});
    client.notify("textDocument/didOpen", {
      textDocument: {
        uri: sourceUri,
        languageId: "typescriptreact",
        version: 1,
        text: sourceText,
      },
    });

    const hover = await client.request("textDocument/hover", {
      textDocument: { uri: sourceUri },
      position: sizePosition,
    });
    const fontSizeHover = await client.request("textDocument/hover", {
      textDocument: { uri: sourceUri },
      position: fontSizePosition,
    });
    const definition = await client.request("textDocument/definition", {
      textDocument: { uri: sourceUri },
      position: sizePosition,
    });
    const references = await client.request("textDocument/references", {
      textDocument: { uri: sourceUri },
      position: sizePosition,
      context: { includeDeclaration: true },
    });
    const prepareRename = await client.request("textDocument/prepareRename", {
      textDocument: { uri: sourceUri },
      position: sizePosition,
    });
    const rename = await client.request("textDocument/rename", {
      textDocument: { uri: sourceUri },
      position: sizePosition,
      newName: "large",
    });

    const hoverText = readString(hover, ["contents", "value"]);
    const fontSizeHoverText = readString(fontSizeHover, ["contents", "value"]);
    const definitionUris = readArray(definition).map((location) => readString(location, ["uri"]));
    const referenceUris = readArray(references).map((location) => readString(location, ["uri"]));
    const prepareRenamePlaceholder = readString(prepareRename, ["placeholder"]);
    const renameStyleEdits = readWorkspaceEdits(rename, styleUri);

    assert(
      hoverText.includes("`.medium`") && hoverText.includes("`.small`"),
      `hover did not include projected selector facts: ${hoverText}`,
    );
    assert(
      fontSizeHoverText.includes("`.font-size-10`") &&
        fontSizeHoverText.includes("`.font-size-12`"),
      `fontSize hover did not include optional union selector facts: ${fontSizeHoverText}`,
    );
    assert(
      definitionUris.filter((uri) => fileUriEquivalent(uri, styleUri)).length === 2,
      `definition did not resolve both projected style selectors: ${JSON.stringify(definition)}`,
    );
    assert(
      referenceUris.some((uri) => fileUriEquivalent(uri, styleUri)) &&
        referenceUris.some((uri) => fileUriEquivalent(uri, sourceUri)),
      `references did not include style and source locations: ${JSON.stringify(references)}`,
    );
    assert(
      prepareRenamePlaceholder === "medium" || prepareRenamePlaceholder === "small",
      `prepareRename did not route through projected selector candidates: ${JSON.stringify(prepareRename)}`,
    );
    assert(
      renameStyleEdits.length > 0,
      `rename did not produce projected style selector edits: ${JSON.stringify(rename)}`,
    );

    process.stdout.write(
      [
        "validated rust omena-lsp-server type-fact protocol:",
        `definitions=${definitionUris.length}`,
        `references=${referenceUris.length}`,
        `prepareRename=${prepareRenamePlaceholder}`,
        `renameStyleEdits=${renameStyleEdits.length}`,
        "diskFallback=unopened-style",
        "unicodePosition=utf16",
        "union=nullish-soft-skip",
        "projection=omena-query",
      ].join(" "),
    );
    process.stdout.write("\n");
    await runStyleOpenRetriggerSmoke(client);
  } finally {
    await client.shutdown();
  }
}

async function runStyleOpenRetriggerSmoke(client: JsonRpcClient): Promise<void> {
  const lateSourceUri = fileUri(path.join(srcDir, "Late.tsx"));
  const lateStylePath = path.join(srcDir, "Late.module.scss");
  const lateStyleUri = fileUri(lateStylePath);
  const tonePosition = positionForOffset(
    lateSourceText,
    lateSourceText.indexOf("cx(tone") + "cx(".length,
  );

  client.notify("textDocument/didOpen", {
    textDocument: {
      uri: lateSourceUri,
      languageId: "typescriptreact",
      version: 1,
      text: lateSourceText,
    },
  });
  writeFileSync(lateStylePath, lateStyleText);
  client.notify("textDocument/didOpen", {
    textDocument: {
      uri: lateStyleUri,
      languageId: "scss",
      version: 1,
      text: lateStyleText,
    },
  });

  const hover = await client.request("textDocument/hover", {
    textDocument: { uri: lateSourceUri },
    position: tonePosition,
  });
  const hoverText = readString(hover, ["contents", "value"]);
  assert(
    hoverText.includes("`.info`") && hoverText.includes("`.warn`"),
    `late style didOpen did not retrigger source projection: ${hoverText}`,
  );
}

class JsonRpcClient {
  readonly #child;
  readonly #pending = new Map<number, (message: JsonRpcMessage) => void>();
  #buffer = Buffer.alloc(0);
  #stderr = "";
  #nextId = 1;

  constructor(command: string, cwd: string) {
    this.#child = spawn(command, [], {
      cwd,
      env: { ...process.env },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.#child.stdout.on("data", (chunk: Buffer) => {
      this.#buffer = Buffer.concat([this.#buffer, chunk]);
      this.#pump();
    });
    this.#child.stderr.on("data", (chunk: Buffer) => {
      this.#stderr += chunk.toString("utf8");
    });
  }

  request(method: string, params: unknown): Promise<unknown> {
    const id = this.#nextId++;
    this.#send({ jsonrpc: "2.0", id, method, params });
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(
          new Error(
            `Timed out waiting for ${method}${
              this.#stderr.trim() ? `\nstderr:\n${this.#stderr.trim()}` : ""
            }`,
          ),
        );
      }, 8_000);
      this.#pending.set(id, (message) => {
        clearTimeout(timer);
        if (message.error) {
          reject(new Error(`${method} returned error: ${JSON.stringify(message.error)}`));
          return;
        }
        resolve(message.result);
      });
    });
  }

  notify(method: string, params: unknown): void {
    this.#send({ jsonrpc: "2.0", method, params });
  }

  async shutdown(): Promise<void> {
    await this.request("shutdown", null).catch(() => undefined);
    this.notify("exit", {});
    this.#child.kill();
  }

  #send(message: unknown): void {
    const body = Buffer.from(JSON.stringify(message));
    this.#child.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    this.#child.stdin.write(body);
  }

  #pump(): void {
    while (true) {
      const headerEnd = this.#buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) return;
      const header = this.#buffer.subarray(0, headerEnd).toString("utf8");
      const match = /Content-Length: (\d+)/iu.exec(header);
      if (!match) {
        throw new Error(`Invalid JSON-RPC header: ${header}`);
      }
      const contentLength = Number(match[1]);
      const bodyStart = headerEnd + 4;
      const bodyEnd = bodyStart + contentLength;
      if (this.#buffer.length < bodyEnd) return;
      const body = this.#buffer.subarray(bodyStart, bodyEnd).toString("utf8");
      this.#buffer = this.#buffer.subarray(bodyEnd);
      const message = JSON.parse(body) as JsonRpcMessage;
      if (message.id !== undefined) {
        this.#pending.get(message.id)?.(message);
        this.#pending.delete(message.id);
      }
    }
  }
}

function fileUri(filePath: string): string {
  return `file://${filePath}`;
}

function fileUriEquivalent(left: string, right: string): boolean {
  return realpathSync(filePathFromUri(left)) === realpathSync(filePathFromUri(right));
}

function filePathFromUri(uri: string): string {
  return decodeURIComponent(uri.replace(/^file:\/\//u, ""));
}

function positionForOffset(source: string, offset: number): { line: number; character: number } {
  const prefix = source.slice(0, offset);
  const lines = prefix.split("\n");
  return { line: lines.length - 1, character: lines[lines.length - 1]?.length ?? 0 };
}

function readArray(value: unknown): readonly unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Expected array result, got ${JSON.stringify(value)}`);
  }
  return value;
}

function readString(value: unknown, pathParts: readonly string[]): string {
  let current = value;
  for (const pathPart of pathParts) {
    current =
      current && typeof current === "object"
        ? (current as Record<string, unknown>)[pathPart]
        : undefined;
  }
  if (typeof current !== "string") {
    throw new Error(`Expected string at ${pathParts.join(".")}, got ${JSON.stringify(value)}`);
  }
  return current;
}

function readWorkspaceEdits(value: unknown, targetUri: string): readonly unknown[] {
  const changes = readRecord(value, ["changes"]);
  for (const [uri, edits] of Object.entries(changes)) {
    if (fileUriEquivalent(uri, targetUri)) {
      return readArray(edits);
    }
  }
  throw new Error(`Expected workspace edits for ${targetUri}, got ${JSON.stringify(value)}`);
}

function readRecord(value: unknown, pathParts: readonly string[]): Record<string, unknown> {
  let current = value;
  for (const pathPart of pathParts) {
    current =
      current && typeof current === "object"
        ? (current as Record<string, unknown>)[pathPart]
        : undefined;
  }
  if (!current || typeof current !== "object" || Array.isArray(current)) {
    throw new Error(`Expected object at ${pathParts.join(".")}, got ${JSON.stringify(value)}`);
  }
  return current as Record<string, unknown>;
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

void main();
