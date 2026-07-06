import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

type BoundaryClass = "json-string" | "jsvalue-any" | "typed";
type ExportKind = "function" | "method" | "constructor";
type CrateName = "omena-napi" | "omena-wasm";

interface CensusRow {
  readonly crate: CrateName;
  readonly ordinal: number;
  readonly jsName: string;
  readonly rustName: string;
  readonly exportKind: ExportKind;
  readonly boundaryClass: BoundaryClass;
  readonly sourcePath: string;
  readonly line: number;
  readonly signature: string;
}

interface BoundaryCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-ffi.boundary-typing-census";
  readonly sources: readonly string[];
  readonly summary: {
    readonly totalCallableCount: number;
    readonly jsonStringCount: number;
    readonly jsValueAnyCount: number;
    readonly typedCount: number;
    readonly untypedBoundaryCount: number;
  };
  readonly rows: readonly CensusRow[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-ffi-boundary-typing-census.json");
const writeMode = process.argv.includes("--write");
const sources = ["rust/crates/omena-napi/src/lib.rs", "rust/crates/omena-wasm/src/lib.rs"] as const;

const census = buildCensus();
const expected = formatCensusJson(census);

if (writeMode) {
  writeFileSync(censusPath, expected);
} else {
  let actual = "";
  try {
    actual = readFileSync(censusPath, "utf8");
  } catch {
    throw new Error(
      `missing FFI boundary typing census at ${path.relative(repoRoot, censusPath)}; run this check with --write to create the scan-derived baseline`,
    );
  }
  assert.equal(
    actual,
    expected,
    "FFI boundary typing census is stale; regenerate the scan-derived baseline",
  );
}

process.stdout.write(
  `FFI boundary typing census OK: ${census.summary.totalCallableCount} callables, ${census.summary.untypedBoundaryCount} untyped\n`,
);

function buildCensus(): BoundaryCensus {
  const rows = [
    ...scanNapiSource("rust/crates/omena-napi/src/lib.rs"),
    ...scanWasmSource("rust/crates/omena-wasm/src/lib.rs"),
  ];
  const jsonStringCount = rows.filter((row) => row.boundaryClass === "json-string").length;
  const jsValueAnyCount = rows.filter((row) => row.boundaryClass === "jsvalue-any").length;
  const typedCount = rows.filter((row) => row.boundaryClass === "typed").length;
  assert.ok(
    rows.some((row) => row.crate === "omena-napi"),
    "napi FFI surface is empty",
  );
  assert.ok(
    rows.some((row) => row.crate === "omena-wasm"),
    "wasm FFI surface is empty",
  );
  assert.ok(jsonStringCount > 0, "napi JSON-string boundary count is zero");
  assert.ok(jsValueAnyCount > 0, "wasm JsValue-any boundary count is zero");
  return {
    schemaVersion: "0",
    product: "omena-ffi.boundary-typing-census",
    sources,
    summary: {
      totalCallableCount: rows.length,
      jsonStringCount,
      jsValueAnyCount,
      typedCount,
      untypedBoundaryCount: jsonStringCount + jsValueAnyCount,
    },
    rows,
  };
}

function formatCensusJson(census: BoundaryCensus): string {
  const sourceLine = `  "sources": [${census.sources.map((source) => JSON.stringify(source)).join(", ")}],`;
  return `${JSON.stringify(census, null, 2).replace(
    /  "sources": \[\n(?:    "[^"]+",?\n)+  \],/,
    sourceLine,
  )}\n`;
}

function scanNapiSource(sourcePath: string): CensusRow[] {
  const lines = readSourceLines(sourcePath);
  const rows: CensusRow[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].trim();
    const jsName = parseQuotedJsName(line, "napi");
    if (jsName) {
      const signature = readNextCallableSignature(lines, index + 1);
      if (!signature) continue;
      rows.push(
        row({
          crateName: "omena-napi",
          rows,
          jsName,
          exportKind: isInsideExpressionRuntime(lines, index) ? "method" : "function",
          sourcePath,
          attrLine: index + 1,
          signature,
        }),
      );
      continue;
    }
    if (line === "#[napi(constructor)]") {
      const signature = readNextCallableSignature(lines, index + 1);
      assert.ok(signature, `missing napi constructor signature after ${sourcePath}:${index + 1}`);
      rows.push(
        row({
          crateName: "omena-napi",
          rows,
          jsName: "constructor",
          exportKind: "constructor",
          sourcePath,
          attrLine: index + 1,
          signature,
        }),
      );
    }
  }
  return rows;
}

function scanWasmSource(sourcePath: string): CensusRow[] {
  const lines = readSourceLines(sourcePath);
  const rows: CensusRow[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].trim();
    const jsName = parseBareJsName(line, "wasm_bindgen");
    if (jsName) {
      const signature = readNextCallableSignature(lines, index + 1);
      if (!signature) continue;
      rows.push(
        row({
          crateName: "omena-wasm",
          rows,
          jsName,
          exportKind: isInsideExpressionRuntime(lines, index) ? "method" : "function",
          sourcePath,
          attrLine: index + 1,
          signature,
        }),
      );
      continue;
    }
    if (line === "#[wasm_bindgen(constructor)]") {
      const signature = readNextCallableSignature(lines, index + 1);
      assert.ok(signature, `missing wasm constructor signature after ${sourcePath}:${index + 1}`);
      rows.push(
        row({
          crateName: "omena-wasm",
          rows,
          jsName: "constructor",
          exportKind: "constructor",
          sourcePath,
          attrLine: index + 1,
          signature,
        }),
      );
    }
  }
  return rows;
}

function row(input: {
  readonly crateName: CrateName;
  readonly rows: readonly CensusRow[];
  readonly jsName: string;
  readonly exportKind: ExportKind;
  readonly sourcePath: string;
  readonly attrLine: number;
  readonly signature: string;
}): CensusRow {
  return {
    crate: input.crateName,
    ordinal: input.rows.length + 1,
    jsName: input.jsName,
    rustName: parseRustName(input.signature, input.sourcePath, input.attrLine),
    exportKind: input.exportKind,
    boundaryClass: classifyBoundary(input.crateName, input.signature),
    sourcePath: input.sourcePath,
    line: input.attrLine,
    signature: normalizeSignature(input.signature),
  };
}

function classifyBoundary(crateName: CrateName, signature: string): BoundaryClass {
  if (crateName === "omena-wasm" && /\bJsValue\b/.test(signature)) {
    return "jsvalue-any";
  }
  if (
    crateName === "omena-napi" &&
    (/\b[a-zA-Z0-9_]*_?json\s*:\s*String\b/i.test(signature) ||
      /->\s*napi::Result\s*<\s*String\s*>/.test(signature))
  ) {
    return "json-string";
  }
  return "typed";
}

function readSourceLines(sourcePath: string): string[] {
  return readFileSync(path.join(repoRoot, sourcePath), "utf8").split(/\r?\n/);
}

function parseQuotedJsName(line: string, attrName: string): string | undefined {
  const match = line.match(new RegExp(`^#\\[${attrName}\\(js_name\\s*=\\s*"([^"]+)"\\)\\]$`));
  return match?.[1];
}

function parseBareJsName(line: string, attrName: string): string | undefined {
  const match = line.match(new RegExp(`^#\\[${attrName}\\(js_name\\s*=\\s*([A-Za-z0-9_]+)\\)\\]$`));
  return match?.[1];
}

function readNextCallableSignature(
  lines: readonly string[],
  startIndex: number,
): string | undefined {
  for (let index = startIndex; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();
    if (!trimmed || trimmed.startsWith("#[")) continue;
    if (trimmed.startsWith("pub struct ")) return undefined;
    if (!trimmed.startsWith("pub fn ")) continue;
    const signatureLines = [lines[index]];
    for (let cursor = index + 1; cursor < lines.length; cursor += 1) {
      if (signatureLines.join("\n").includes("{")) break;
      signatureLines.push(lines[cursor]);
    }
    return signatureLines.join("\n");
  }
  return undefined;
}

function parseRustName(signature: string, sourcePath: string, line: number): string {
  const match = signature.match(/\bpub fn\s+([A-Za-z0-9_]+)/);
  assert.ok(match, `unable to parse Rust function name for ${sourcePath}:${line}`);
  return match[1];
}

function normalizeSignature(signature: string): string {
  return signature
    .replace(/\s+/g, " ")
    .replace(/\s*\{\s*$/, "")
    .trim();
}

function isInsideExpressionRuntime(lines: readonly string[], attrIndex: number): boolean {
  const windowStart = Math.max(0, attrIndex - 30);
  const window = lines.slice(windowStart, attrIndex).join("\n");
  return /impl\s+(?:OmenaNapiExpressionDomainFlowRuntimeV0|OmenaWasmExpressionDomainFlowRuntimeV0)\s*\{/.test(
    window,
  );
}
