import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Surface = "napi" | "wasm" | "cli" | "lsp";
type Workflow = "snapshot" | "query" | "diagnostics" | "build" | "explain";
type ErrorCase = "input" | "workspace" | "resolution" | "unsupported";

interface CoverageEntry {
  readonly workflow: Workflow;
  readonly surface: Surface;
  readonly status: "covered";
  readonly evidence: "native-addon" | "webassembly" | "process" | "lsp-stdio";
}

interface ParityMatrix {
  readonly schemaVersion: "0";
  readonly product: "omena-sdk.workflow-parity-matrix";
  readonly workflows: readonly Workflow[];
  readonly surfaces: readonly Surface[];
  readonly errorCases: readonly ErrorCase[];
  readonly entries: readonly CoverageEntry[];
}

interface SurfaceResult {
  readonly workflows: Readonly<Record<Workflow, unknown>>;
  readonly errors: Readonly<Record<ErrorCase, unknown>>;
  readonly publicationSnapshotId?: unknown;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const matrixPath = path.join(repoRoot, "rust/omena-sdk-workflow-parity-matrix.json");
const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-sdk-workflow-parity-"));
const targetDir = path.join(repoRoot, "rust/target/sdk-workflow-parity");
const workspaceRoot = "file:///virtual/omena-sdk-workspace";
const stylePath = `${workspaceRoot}/src/card.module.scss`;
const styleSource = ":root { --known: red; } .card { color: var(--missing); }";
const styleSources = [{ stylePath, styleSource }];
const snapshotId = { value: 1 };
const workflows = ["snapshot", "query", "diagnostics", "build", "explain"] as const;
const surfaces = ["napi", "wasm", "cli", "lsp"] as const;
const errorCases = ["input", "workspace", "resolution", "unsupported"] as const;
const writeMode = process.argv.includes("--write");

assertGeneratedResponseAuthority();
const cliBinary = buildRustBinary("omena-cli", "omena");
const lspBinary = buildRustBinary("omena-lsp-server", "omena-lsp-server");
const napiModule = buildNapiModule();
const wasmModule = buildWasmModule();

const results: Record<Surface, SurfaceResult> = {
  napi: runNodeSurface("napi", napiModule),
  wasm: runNodeSurface("wasm", wasmModule),
  cli: runCliSurface(cliBinary),
  lsp: runLspSurface(lspBinary),
};

if (process.env.OMENA_SDK_PARITY_TEST_DROP_FIELD === "1") {
  const diagnostics = results.wasm.workflows.diagnostics as Record<string, unknown>;
  const summary = diagnostics.summary as Record<string, unknown>;
  delete summary.classSelectorCount;
}
if (process.env.OMENA_SDK_PARITY_TEST_CHANGE_ERROR === "1") {
  const error = results.wasm.errors.workspace as Record<string, unknown>;
  error.code = "workspace.unregistered-error";
}

for (const workflow of workflows) {
  const expected = canonicalize(results.cli.workflows[workflow]);
  for (const surface of surfaces) {
    assert.deepEqual(
      canonicalize(results[surface].workflows[workflow]),
      expected,
      `${workflow} response diverged on ${surface}`,
    );
  }
}

for (const errorCase of errorCases) {
  const expected = canonicalize(results.cli.errors[errorCase]);
  for (const surface of surfaces) {
    assert.deepEqual(
      canonicalize(results[surface].errors[errorCase]),
      expected,
      `${errorCase} error diverged on ${surface}`,
    );
  }
}

assert.deepEqual(
  canonicalize(results.lsp.publicationSnapshotId),
  canonicalize((results.lsp.workflows.diagnostics as Record<string, unknown>).snapshotId),
  "LSP diagnostics publication and typed workflow must read the same snapshot",
);

const expectedMatrix = buildMatrix();
if (writeMode) {
  fs.writeFileSync(matrixPath, `${JSON.stringify(expectedMatrix, null, 2)}\n`);
} else {
  const matrix = JSON.parse(fs.readFileSync(matrixPath, "utf8")) as ParityMatrix;
  if (process.env.OMENA_SDK_PARITY_TEST_CLAIM_EXTRA_COVERAGE === "1") {
    (matrix.entries as CoverageEntry[]).push({
      workflow: "query",
      surface: "cli",
      status: "covered",
      evidence: "process",
    });
  }
  assert.deepEqual(matrix, expectedMatrix, "committed SDK workflow coverage matrix drifted");
}

process.stdout.write(
  `Omena SDK workflow parity OK: workflows=${workflows.length} surfaces=${surfaces.length} errors=${errorCases.length}\n`,
);

function requests(): Readonly<Record<Workflow, unknown>> {
  return {
    snapshot: { workspaceRoot },
    query: {
      snapshotId,
      queryKind: "styleSummary",
      input: { stylePath },
    },
    diagnostics: { snapshotId, stylePath, styleSource },
    build: { snapshotId, stylePath, styleSource, passIds: [] },
    explain: {
      snapshotId,
      stylePath,
      position: { line: 0, character: 1 },
    },
  };
}

function errorRequests(): Readonly<Record<ErrorCase, { operation: Workflow; request: unknown }>> {
  return {
    input: {
      operation: "build",
      request: { snapshotId, stylePath, styleSource, passIds: [], context: "invalid" },
    },
    workspace: {
      operation: "query",
      request: { snapshotId: { value: 2 }, queryKind: "styleSummary", input: { stylePath } },
    },
    resolution: {
      operation: "query",
      request: {
        snapshotId,
        queryKind: "styleSummary",
        input: { stylePath: `${workspaceRoot}/src/missing.css` },
      },
    },
    unsupported: {
      operation: "query",
      request: { snapshotId, queryKind: "unknownQuery", input: { stylePath } },
    },
  };
}

function buildRustBinary(packageName: string, binaryName: string): string {
  run("cargo", [
    "build",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    packageName,
    "--bin",
    binaryName,
  ]);
  return path.join(
    targetDir,
    "debug",
    process.platform === "win32" ? `${binaryName}.exe` : binaryName,
  );
}

function buildNapiModule(): string {
  run("cargo", ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-napi", "--release"]);
  const extension =
    process.platform === "darwin" ? "dylib" : process.platform === "win32" ? "dll" : "so";
  const libraryName =
    process.platform === "win32" ? "omena_napi.dll" : `libomena_napi.${extension}`;
  const target = path.join(workDir, "napi", "omena.node");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(path.join(targetDir, "release", libraryName), target);
  return target;
}

function buildWasmModule(): string {
  const outputDir = path.join(workDir, "wasm");
  run("wasm-pack", [
    "build",
    "rust/crates/omena-wasm",
    "--target",
    "nodejs",
    "--release",
    "--out-dir",
    outputDir,
  ]);
  return path.join(outputDir, "omena_wasm.js");
}

function runNodeSurface(surface: "napi" | "wasm", modulePath: string): SurfaceResult {
  const inputPath = path.join(workDir, `${surface}-workspace-input.json`);
  fs.writeFileSync(
    inputPath,
    JSON.stringify({ workspaceRoot, styleSources, requests: requests(), errors: errorRequests() }),
  );
  const script = `
const fs=require("fs");
const m=require(process.argv[1]);
const input=JSON.parse(fs.readFileSync(process.argv[2],"utf8"));
const isNapi=process.argv[3]==="napi";
const workspace=new m.Workspace(input.workspaceRoot,isNapi?JSON.stringify(input.styleSources):input.styleSources);
const call=(operation,request)=>{
  if(operation==="snapshot") return isNapi?JSON.parse(workspace.snapshotJson()):workspace.snapshot();
  const method=operation;
  const value=isNapi?JSON.stringify(request):request;
  const result=workspace[isNapi?method+"Json":method](value);
  return isNapi?JSON.parse(result):result;
};
const normalizeError=(thrown)=>{
  let value=thrown;
  if(isNapi) value=JSON.parse(thrown.message);
  if(typeof value==="string") value=JSON.parse(value);
  const error=value.error;
  return {class:error.class,code:error.context.code,severity:error.context.severity,recoverability:error.context.recoverability};
};
const workflows={};
for(const [operation,request] of Object.entries(input.requests)) workflows[operation]=call(operation,request);
const errors={};
for(const [name,spec] of Object.entries(input.errors)) {
  try { call(spec.operation,spec.request); throw new Error("expected typed error"); }
  catch(error) { errors[name]=normalizeError(error); }
}
process.stdout.write(JSON.stringify({workflows,errors}));`;
  return JSON.parse(
    execFileSync(process.execPath, ["-e", script, modulePath, inputPath, surface], {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    }),
  ) as SurfaceResult;
}

function runCliSurface(binary: string): SurfaceResult {
  const workflowOutputs = {} as Record<Workflow, unknown>;
  for (const workflow of workflows) {
    workflowOutputs[workflow] = runCliRequest(binary, workflow, requests()[workflow], false);
  }
  const errors = {} as Record<ErrorCase, unknown>;
  for (const [name, spec] of Object.entries(errorRequests()) as [
    ErrorCase,
    { operation: Workflow; request: unknown },
  ][]) {
    errors[name] = runCliRequest(binary, spec.operation, spec.request, true);
  }
  return { workflows: workflowOutputs, errors };
}

function runCliRequest(
  binary: string,
  operation: Workflow,
  request: unknown,
  expectError: boolean,
): unknown {
  const requestPath = path.join(
    workDir,
    `cli-${operation}-${expectError ? "error" : "ok"}-${Math.random()}.json`,
  );
  fs.writeFileSync(
    requestPath,
    JSON.stringify({ workspaceRoot, styleSources, operation, request }),
  );
  const result = spawnSync(binary, ["sdk", requestPath], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (expectError) {
    assert.notEqual(result.status, 0, `${operation} CLI error fixture unexpectedly succeeded`);
    return normalizedError(JSON.parse(result.stderr.trim()));
  }
  assert.equal(result.status, 0, result.stderr);
  const envelope = JSON.parse(result.stdout) as { product: string; payload: unknown };
  assert.equal(envelope.product, "omena-cli.sdk-workflow");
  return envelope.payload;
}

function runLspSurface(binary: string): SurfaceResult {
  const messages: unknown[] = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        rootUri: workspaceRoot,
        workspaceFolders: [{ uri: workspaceRoot, name: "sdk-workspace" }],
        capabilities: { textDocument: { publishDiagnostics: {} } },
      },
    },
    {
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: stylePath,
          languageId: "scss",
          version: 1,
          text: styleSource,
        },
      },
    },
  ];
  let id = 10;
  for (const workflow of workflows) {
    messages.push(lspWorkflowRequest(id, workflow, requests()[workflow]));
    id += 1;
  }
  for (const spec of Object.values(errorRequests())) {
    messages.push(lspWorkflowRequest(id, spec.operation, spec.request));
    id += 1;
  }
  messages.push({ jsonrpc: "2.0", id, method: "shutdown" });
  messages.push({ jsonrpc: "2.0", method: "exit" });

  const result = spawnSync(binary, [], {
    cwd: repoRoot,
    input: messages.map(frame).join(""),
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  const output = readFrames(result.stdout);
  const workflowOutputs = Object.fromEntries(
    workflows.map((workflow, index) => [workflow, responseById(output, 10 + index).result]),
  ) as Record<Workflow, unknown>;
  const errors = Object.fromEntries(
    errorCases.map((errorCase, index) => {
      const response = responseById(output, 10 + workflows.length + index);
      assert.ok(response.error, `LSP ${errorCase} fixture unexpectedly succeeded`);
      return [errorCase, normalizedError(response.error.data)];
    }),
  ) as Record<ErrorCase, unknown>;
  const publications = output.filter(
    (message) =>
      message.method === "textDocument/publishDiagnostics" && message.params?.uri === stylePath,
  );
  assert.ok(publications.length > 0, "LSP didOpen did not publish diagnostics");
  const publishedDiagnostic = publications
    .flatMap((publication) => publication.params.diagnostics)
    .find((diagnostic: any) => diagnostic.data?.snapshotId);
  assert.ok(
    publishedDiagnostic,
    `LSP publication did not carry workspace snapshot identity: ${JSON.stringify(publications.map((publication) => publication.params.diagnostics))}`,
  );
  return {
    workflows: workflowOutputs,
    errors,
    publicationSnapshotId: publishedDiagnostic.data.snapshotId,
  };
}

function lspWorkflowRequest(id: number, operation: Workflow, request: unknown): unknown {
  return {
    jsonrpc: "2.0",
    id,
    method: "omena/sdkWorkflow",
    params: { workspaceRoot, operation, request },
  };
}

function normalizedError(value: any): unknown {
  const error = value.error;
  return {
    class: error.class,
    code: error.context.code,
    severity: error.context.severity,
    recoverability: error.context.recoverability,
  };
}

function buildMatrix(): ParityMatrix {
  const evidence: Record<Surface, CoverageEntry["evidence"]> = {
    napi: "native-addon",
    wasm: "webassembly",
    cli: "process",
    lsp: "lsp-stdio",
  };
  return {
    schemaVersion: "0",
    product: "omena-sdk.workflow-parity-matrix",
    workflows: [...workflows],
    surfaces: [...surfaces],
    errorCases: [...errorCases],
    entries: workflows.flatMap((workflow) =>
      surfaces.map((surface) => ({
        workflow,
        surface,
        status: "covered" as const,
        evidence: evidence[surface],
      })),
    ),
  };
}

function assertGeneratedResponseAuthority(): void {
  const sourceFiles = listRustSources(path.join(repoRoot, "rust/crates"));
  const responseDefinitions = sourceFiles.flatMap((sourcePath) => {
    const source = fs.readFileSync(sourcePath, "utf8");
    return [...source.matchAll(/pub struct OmenaSdk[A-Za-z0-9]*ResponseV0\s*\{/gu)].map(
      (match) => ({ sourcePath: path.relative(repoRoot, sourcePath), name: match[0] }),
    );
  });
  if (process.env.OMENA_SDK_PARITY_TEST_INJECT_RESPONSE_PLANE === "1") {
    responseDefinitions.push({
      sourcePath: "rust/crates/omena-wasm/src/injected_response.rs",
      name: "pub struct OmenaSdkInjectedResponseV0 {",
    });
  }
  assert.ok(responseDefinitions.length > 0, "generated SDK response authority is empty");
  assert.deepEqual(
    [...new Set(responseDefinitions.map((definition) => definition.sourcePath))],
    ["rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs"],
    "SDK workflow responses must have one generated contract authority",
  );
}

function listRustSources(root: string): string[] {
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) return listRustSources(entryPath);
    return entry.isFile() && entry.name.endsWith(".rs") ? [entryPath] : [];
  });
}

function canonicalize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .toSorted(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, canonicalize(entry)]),
  );
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
    const bodyStart = headerEnd + 4;
    const bodyEnd = bodyStart + Number(match[1]);
    frames.push(JSON.parse(stdout.slice(bodyStart, bodyEnd)));
    offset = bodyEnd;
  }
  return frames;
}

function responseById(messages: readonly any[], id: number): any {
  const response = messages.find((message) => message.id === id);
  assert.ok(response, `missing LSP response ${id}`);
  return response;
}

function run(command: string, args: readonly string[]): void {
  execFileSync(command, args, {
    cwd: repoRoot,
    env: rustBuildEnv(),
    stdio: "inherit",
  });
}

function rustBuildEnv(): NodeJS.ProcessEnv {
  const env = { ...process.env, CARGO_TARGET_DIR: targetDir };
  const stableDeveloperDir = "/Applications/Xcode.app/Contents/Developer";
  if (process.platform === "darwin" && fs.existsSync(stableDeveloperDir)) {
    env.DEVELOPER_DIR = stableDeveloperDir;
  }
  return env;
}
