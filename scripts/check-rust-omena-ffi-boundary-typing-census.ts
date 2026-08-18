import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

type BoundaryClass = "json-string" | "jsvalue-any" | "typed" | "unclassified";
type BoundaryClassifierRule =
  | "napi-json-string"
  | "wasm-jsvalue"
  | "wasm-typed-value"
  | "typed-self"
  | "typed-versioned-struct";
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
  readonly ratchet: {
    readonly direction: "decrease-only";
    readonly maximumJsonStringCount: number;
    readonly maximumJsValueAnyCount: number;
    readonly maximumUnclassifiedByRuleCount: number;
  };
  readonly summary: {
    readonly totalCallableCount: number;
    readonly jsonStringCount: number;
    readonly jsValueAnyCount: number;
    readonly typedCount: number;
    readonly untypedBoundaryCount: number;
    readonly unclassifiedByRuleCount: number;
  };
  readonly classificationCoverage: {
    readonly ruleIds: readonly BoundaryClassifierRule[];
    readonly pinnedRows: readonly BoundaryClassPin[];
    readonly limitation: string;
  };
  readonly rows: readonly CensusRow[];
}

interface BoundaryClassPin {
  readonly crate: CrateName;
  readonly jsName: string;
  readonly signatureIncludes: string;
  readonly expectedClass: Exclude<BoundaryClass, "unclassified">;
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-ffi-boundary-typing-census.json");
const writeMode = process.argv.includes("--write");
const sources = [
  "rust/crates/omena-napi/src/lib.rs",
  "rust/crates/omena-napi/src/sdk_workspace.rs",
  "rust/crates/omena-wasm/src/lib.rs",
  "rust/crates/omena-wasm/src/sdk_workspace.rs",
] as const;
const classifierRuleIds: readonly BoundaryClassifierRule[] = [
  "napi-json-string",
  "wasm-jsvalue",
  "wasm-typed-value",
  "typed-self",
  "typed-versioned-struct",
];
const pinnedRows: readonly BoundaryClassPin[] = [
  {
    crate: "omena-napi",
    jsName: "buildStyleSourceJson",
    signatureIncludes: "napi::Result<String>",
    expectedClass: "json-string",
  },
  {
    crate: "omena-wasm",
    jsName: "buildStyleSource",
    signatureIncludes: "Result<JsValue, JsValue>",
    expectedClass: "jsvalue-any",
  },
  {
    crate: "omena-napi",
    jsName: "buildStyleSourceWithContext",
    signatureIncludes: "EngineNapiConsumerBuildSummaryV0Json",
    expectedClass: "typed",
  },
  {
    crate: "omena-wasm",
    jsName: "constructor",
    signatureIncludes: "pub fn new() -> Self",
    expectedClass: "typed",
  },
  {
    crate: "omena-wasm",
    jsName: "readSourceImportDeclarations",
    signatureIncludes: "OmenaWasmSourceImportDeclarationsV0",
    expectedClass: "typed",
  },
];

const existing = readExistingCensus();
const census = buildCensus(existing);
const expected = formatCensusJson(census);

if (writeMode) {
  writeFileSync(censusPath, expected);
  execFileSync(path.join(repoRoot, "node_modules/.bin/oxfmt"), ["--write", censusPath], {
    cwd: repoRoot,
    stdio: "inherit",
  });
} else {
  let actual: BoundaryCensus;
  try {
    actual = JSON.parse(readFileSync(censusPath, "utf8")) as BoundaryCensus;
  } catch {
    throw new Error(
      `missing FFI boundary typing census at ${path.relative(repoRoot, censusPath)}; run this check with --write to create the scan-derived baseline`,
    );
  }
  assert.deepEqual(
    actual,
    census,
    "FFI boundary typing census is stale; regenerate the scan-derived baseline",
  );
}

process.stdout.write(
  `FFI boundary typing census OK: ${census.summary.totalCallableCount} callables, ${census.summary.untypedBoundaryCount} untyped\n`,
);

function buildCensus(existing: BoundaryCensus | undefined): BoundaryCensus {
  const napiRows = [
    ...scanNapiSource("rust/crates/omena-napi/src/lib.rs"),
    ...scanNapiSource("rust/crates/omena-napi/src/sdk_workspace.rs"),
  ].map((row, index) => ({ ...row, ordinal: index + 1 }));
  const wasmRows = [
    ...scanWasmSource("rust/crates/omena-wasm/src/lib.rs"),
    ...scanWasmSource("rust/crates/omena-wasm/src/sdk_workspace.rs"),
  ].map((row, index) => ({ ...row, ordinal: index + 1 }));
  const rows = [...napiRows, ...wasmRows];
  const jsonStringCount = rows.filter((row) => row.boundaryClass === "json-string").length;
  const jsValueAnyCount = rows.filter((row) => row.boundaryClass === "jsvalue-any").length;
  const typedCount = rows.filter((row) => row.boundaryClass === "typed").length;
  const unclassifiedByRuleCount = rows.filter((row) => row.boundaryClass === "unclassified").length;
  assert.ok(
    rows.some((row) => row.crate === "omena-napi"),
    "napi FFI surface is empty",
  );
  assert.ok(
    rows.some((row) => row.crate === "omena-wasm"),
    "wasm FFI surface is empty",
  );
  for (const pin of pinnedRows) {
    const matches = rows.filter(
      (row) =>
        row.crate === pin.crate &&
        row.jsName === pin.jsName &&
        row.signature.includes(pin.signatureIncludes),
    );
    assert.equal(
      matches.length,
      1,
      `FFI boundary class pin must identify one row: ${pin.crate}:${pin.jsName}:${pin.signatureIncludes}`,
    );
    assert.equal(
      matches[0]?.boundaryClass,
      pin.expectedClass,
      `FFI boundary class pin changed: ${pin.crate}:${pin.jsName}`,
    );
  }
  const previousMaximumJsonStringCount =
    existing?.ratchet?.maximumJsonStringCount ?? existing?.summary.jsonStringCount;
  const previousMaximumJsValueAnyCount =
    existing?.ratchet?.maximumJsValueAnyCount ?? existing?.summary.jsValueAnyCount;
  const previousMaximumUnclassifiedByRuleCount =
    existing?.ratchet?.maximumUnclassifiedByRuleCount ??
    existing?.summary.unclassifiedByRuleCount ??
    0;
  if (previousMaximumJsonStringCount !== undefined) {
    assert.ok(
      jsonStringCount <= previousMaximumJsonStringCount,
      `napi JSON-string boundary count increased: maximum=${previousMaximumJsonStringCount} current=${jsonStringCount}`,
    );
  }
  if (previousMaximumJsValueAnyCount !== undefined) {
    assert.ok(
      jsValueAnyCount <= previousMaximumJsValueAnyCount,
      `wasm JsValue-any boundary count increased: maximum=${previousMaximumJsValueAnyCount} current=${jsValueAnyCount}`,
    );
  }
  assert.ok(
    unclassifiedByRuleCount <= previousMaximumUnclassifiedByRuleCount,
    `FFI boundaries not covered by a classifier rule increased: maximum=${previousMaximumUnclassifiedByRuleCount} current=${unclassifiedByRuleCount}`,
  );
  return {
    schemaVersion: "0",
    product: "omena-ffi.boundary-typing-census",
    sources,
    ratchet: {
      direction: "decrease-only",
      maximumJsonStringCount: writeMode
        ? jsonStringCount
        : (previousMaximumJsonStringCount ?? jsonStringCount),
      maximumJsValueAnyCount: writeMode
        ? jsValueAnyCount
        : (previousMaximumJsValueAnyCount ?? jsValueAnyCount),
      maximumUnclassifiedByRuleCount: writeMode
        ? unclassifiedByRuleCount
        : previousMaximumUnclassifiedByRuleCount,
    },
    summary: {
      totalCallableCount: rows.length,
      jsonStringCount,
      jsValueAnyCount,
      typedCount,
      untypedBoundaryCount: jsonStringCount + jsValueAnyCount,
      unclassifiedByRuleCount,
    },
    classificationCoverage: {
      ruleIds: classifierRuleIds,
      pinnedRows,
      limitation:
        "Pinned rows detect classifier-wide drift but cannot detect a rule error confined to an unpinned signature.",
    },
    rows,
  };
}

function formatCensusJson(census: BoundaryCensus): string {
  return `${JSON.stringify(census, null, 2)}\n`;
}

function readExistingCensus(): BoundaryCensus | undefined {
  try {
    const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as BoundaryCensus;
    assert.equal(parsed.schemaVersion, "0", "FFI boundary census schemaVersion");
    assert.equal(parsed.product, "omena-ffi.boundary-typing-census", "FFI boundary census product");
    if (parsed.ratchet !== undefined) {
      assert.equal(parsed.ratchet.direction, "decrease-only", "FFI boundary ratchet direction");
      assert.equal(
        parsed.ratchet.maximumJsonStringCount,
        parsed.summary.jsonStringCount,
        "committed JSON-string floor must equal its measured count",
      );
      assert.equal(
        parsed.ratchet.maximumJsValueAnyCount,
        parsed.summary.jsValueAnyCount,
        "committed JsValue-any floor must equal its measured count",
      );
      assert.equal(
        parsed.ratchet.maximumUnclassifiedByRuleCount,
        parsed.summary.unclassifiedByRuleCount,
        "committed unclassified-by-rule floor must equal its measured count",
      );
    }
    return parsed;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
    throw error;
  }
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
          exportKind:
            isIndentedAttribute(lines[index]) || isInsideExpressionRuntime(lines, index)
              ? "method"
              : "function",
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
          exportKind:
            isIndentedAttribute(lines[index]) || isInsideExpressionRuntime(lines, index)
              ? "method"
              : "function",
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
  const rule = classifierRuleForBoundary(crateName, signature);
  switch (rule) {
    case "wasm-jsvalue":
      return "jsvalue-any";
    case "napi-json-string":
      return "json-string";
    case "typed-self":
    case "typed-versioned-struct":
    case "wasm-typed-value":
      return "typed";
    default:
      return "unclassified";
  }
}

function classifierRuleForBoundary(
  crateName: CrateName,
  signature: string,
): BoundaryClassifierRule | undefined {
  if (crateName === "omena-wasm" && /\bJsValue\b/.test(signature)) {
    return "wasm-jsvalue";
  }
  if (
    crateName === "omena-wasm" &&
    /->\s*(?:Option\s*<\s*)?(?:String|u(?:8|16|32|64|128|size)|i(?:8|16|32|64|128|size)|bool|[A-Z][A-Za-z0-9_]*V\d+)\s*>?/.test(
      signature,
    )
  ) {
    return "wasm-typed-value";
  }
  if (
    crateName === "omena-napi" &&
    (/\b[a-zA-Z0-9_]*_?json\s*:\s*String\b/i.test(signature) ||
      /->\s*napi::Result\s*<\s*String\s*>/.test(signature))
  ) {
    return "napi-json-string";
  }
  if (/->\s*Self\b/.test(signature)) {
    return "typed-self";
  }
  if (
    crateName === "omena-napi" &&
    /->\s*napi::Result\s*<\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Z][A-Za-z0-9_]*V\d+(?:Json)?\s*>/.test(
      signature,
    )
  ) {
    return "typed-versioned-struct";
  }
  return undefined;
}

function readSourceLines(sourcePath: string): string[] {
  return readFileSync(path.join(repoRoot, sourcePath), "utf8").split(/\r?\n/);
}

function parseQuotedJsName(line: string, attrName: string): string | undefined {
  const match = line.match(new RegExp(`^#\\[${attrName}\\(js_name\\s*=\\s*"([^"]+)"\\)\\]$`));
  return match?.[1];
}

function isIndentedAttribute(line: string): boolean {
  return /^\s+#\[/u.test(line);
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
