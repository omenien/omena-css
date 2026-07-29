import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { API } from "@typescript/native-preview/unstable/async";
import { SyntaxKind } from "@typescript/native-preview/unstable/ast";
import { buildTsgoTypeFactApiOptions } from "../server/engine-host-node/src/tsgo-type-fact-collector";

const repoRoot = process.cwd();
const rustSourcePath = path.join(repoRoot, "rust/crates/omena-tsgo-client/src/lib.rs");
const packagePath = path.join(repoRoot, "node_modules/@typescript/native-preview/package.json");
const enumPath = path.join(
  repoRoot,
  "node_modules/@typescript/native-preview/dist/enums/typeFlags.enum.d.ts",
);
const protocolDeclarationPath = path.join(
  repoRoot,
  "node_modules/@typescript/native-preview/dist/api/node/protocol.d.ts",
);
const protocolRuntimePath = path.join(
  repoRoot,
  "node_modules/@typescript/native-preview/dist/api/node/protocol.js",
);

const rustSource = readFileSync(rustSourcePath, "utf8");
const packageJson = JSON.parse(readFileSync(packagePath, "utf8")) as {
  readonly name?: unknown;
  readonly version?: unknown;
};
const enumSource = readFileSync(enumPath, "utf8");
const protocolDeclarationSource = readFileSync(protocolDeclarationPath, "utf8");
const protocolRuntimeSource = readFileSync(protocolRuntimePath, "utf8");

assert.equal(packageJson.name, "@typescript/native-preview", "native TypeScript package name");
assert.equal(typeof packageJson.version, "string", "native TypeScript package version");

const derivedTypeFlagConstants = new Set(["TSGO_TYPE_FLAGS_NULLISH"]);
const rustTypeFlagConstants = [
  ...rustSource.matchAll(/^pub const (TSGO_TYPE_FLAGS_[A-Z0-9_]+): u64\s*=/gmu),
].map((match) => match[1]);
assert(rustTypeFlagConstants.length > 0, "Rust TypeFlags domain must not be empty");
const boundTypeFlagConstants = rustTypeFlagConstants.filter(
  (name) => !derivedTypeFlagConstants.has(name),
);
assert.deepEqual(
  [...new Set([...boundTypeFlagConstants, ...derivedTypeFlagConstants])].sort(),
  [...rustTypeFlagConstants].sort(),
  "every Rust TypeFlags constant must be bound or declared derived",
);

const observedTypeFlags = boundTypeFlagConstants.map((rustName) => {
  const enumName = screamingSnakeToPascal(rustName.slice("TSGO_TYPE_FLAGS_".length));
  const rustValue = parseRustIntegerConstant(rustSource, rustName, "u64");
  const enumValue = parseEnumMember(enumSource, enumName);
  assert.equal(
    rustValue,
    enumValue,
    `${rustName} must match @typescript/native-preview TypeFlags.${enumName}`,
  );
  return { rustName, enumName, value: rustValue };
});

assert.equal(
  new Set(observedTypeFlags.map((entry) => entry.value)).size,
  observedTypeFlags.length,
  "bound TypeFlags values must remain distinct",
);

const binaryAstLayoutNames = [
  "HEADER_OFFSET_STRING_TABLE_OFFSETS",
  "HEADER_OFFSET_STRING_TABLE",
  "HEADER_OFFSET_EXTENDED_DATA",
  "HEADER_OFFSET_NODES",
  "HEADER_SIZE",
  "NODE_LEN",
  "NODE_OFFSET_KIND",
  "NODE_OFFSET_POS",
  "NODE_OFFSET_END",
  "NODE_OFFSET_DATA",
  "KIND_NODE_LIST",
  "NODE_EXTENDED_DATA_MASK",
] as const;
const rustBinaryAstLayoutNames = [
  ...rustSource.matchAll(
    /^(?:pub(?:\([^)\n]+\))?\s+)?const ((?:HEADER_|NODE_|KIND_)[A-Z0-9_]+):/gmu,
  ),
].map((match) => match[1]);
assert.deepEqual(
  [...binaryAstLayoutNames].sort(),
  rustBinaryAstLayoutNames.sort(),
  "every top-level Rust binary AST layout constant must be bound to the installed protocol",
);

const observedBinaryAstLayout = binaryAstLayoutNames.map((name) => {
  const rustValue = parseRustIntegerConstant(
    rustSource,
    name,
    name.startsWith("KIND_") || name.endsWith("_MASK") ? "u32" : "usize",
  );
  const declarationValue = parseExportedConstant(protocolDeclarationSource, name);
  const runtimeValue = parseExportedConstant(protocolRuntimeSource, name);
  assert.equal(
    rustValue,
    declarationValue,
    `${name} must match the installed binary AST declaration`,
  );
  assert.equal(declarationValue, runtimeValue, `${name} declaration and runtime values must agree`);
  return { name, value: rustValue };
});

const protocolVersion = parseExportedConstant(protocolDeclarationSource, "PROTOCOL_VERSION");
assert.equal(
  protocolVersion,
  parseExportedConstant(protocolRuntimeSource, "PROTOCOL_VERSION"),
  "binary AST protocol declaration and runtime version must agree",
);

void main();

async function main(): Promise<void> {
  const installedPayloadWitness = await decodeInstalledPayloadWitness(
    Object.fromEntries(observedBinaryAstLayout.map(({ name, value }) => [name, value])),
  );

  process.stdout.write(
    `${JSON.stringify(
      {
        product: "omena.tsgo.binary-ast-abi",
        package: packageJson.name,
        packageVersion: packageJson.version,
        protocolVersion,
        typeFlags: {
          bindings: observedTypeFlags,
          derived: [...derivedTypeFlagConstants].sort(),
        },
        binaryAstLayout: observedBinaryAstLayout,
        installedPayloadWitness,
      },
      null,
      2,
    )}\n`,
  );
}

async function decodeInstalledPayloadWitness(
  layout: Readonly<Record<string, number>>,
): Promise<Readonly<Record<string, unknown>>> {
  const workspaceRoot = path.join(
    repoRoot,
    "test/_fixtures/type-fact-backend-parity/literal-union",
  );
  const filePath = path.join(workspaceRoot, "src/App.ts");
  const source = readFileSync(filePath, "utf8");
  const api = new API(buildTsgoTypeFactApiOptions(workspaceRoot));
  let snapshot: Awaited<ReturnType<API["updateSnapshot"]>> | undefined;
  try {
    snapshot = await api.updateSnapshot({
      openProject: path.join(workspaceRoot, "tsconfig.json"),
    });
    const project = await snapshot.getDefaultProjectForFile(filePath);
    assert(project, `binary AST witness has no tsgo project: ${filePath}`);
    const sourceFile = await project.program.getSourceFile(filePath);
    assert(sourceFile, `binary AST witness has no source file: ${filePath}`);
    const view = (sourceFile as unknown as { readonly view?: DataView }).view;
    assert(view instanceof DataView, "installed source file must retain its binary AST DataView");

    const offsetNodes = view.getUint32(layout.HEADER_OFFSET_NODES, true);
    const offsetExtendedData = view.getUint32(layout.HEADER_OFFSET_EXTENDED_DATA, true);
    const offsetStringTable = view.getUint32(layout.HEADER_OFFSET_STRING_TABLE, true);
    const offsetStringTableOffsets = view.getUint32(
      layout.HEADER_OFFSET_STRING_TABLE_OFFSETS,
      true,
    );
    assert.equal(
      (view.byteLength - offsetNodes) % layout.NODE_LEN,
      0,
      "installed binary AST node records must align to the Rust node length",
    );

    const sourceFileRecord = offsetNodes + layout.NODE_LEN;
    const sourceFileData = view.getUint32(sourceFileRecord + layout.NODE_OFFSET_DATA, true);
    const sourceFileExtendedData =
      offsetExtendedData + (sourceFileData & layout.NODE_EXTENDED_DATA_MASK);
    const textIndex = view.getUint32(
      sourceFileExtendedData +
        parseRustIntegerConstant(rustSource, "SOURCE_FILE_EXTENDED_DATA_OFFSET_TEXT", "usize"),
      true,
    );
    const pathIndex = view.getUint32(
      sourceFileExtendedData +
        parseRustIntegerConstant(rustSource, "SOURCE_FILE_EXTENDED_DATA_OFFSET_PATH", "usize"),
      true,
    );
    const decodedSource = readBinaryAstString(
      view,
      offsetStringTableOffsets,
      offsetStringTable,
      textIndex,
    );
    const decodedPath = readBinaryAstString(
      view,
      offsetStringTableOffsets,
      offsetStringTable,
      pathIndex,
    );
    assert.equal(decodedSource, source, "installed payload source-text offset must decode exactly");
    assert.equal(
      comparablePath(decodedPath),
      comparablePath(filePath),
      "installed payload source-path offset must decode exactly",
    );

    const needle = "pickTone()";
    const start = source.lastIndexOf(needle);
    assert(start >= 0, `binary AST witness source must contain ${needle}`);
    const end = start + needle.length;
    const matchingNodes: { readonly kind: number; readonly pos: number; readonly end: number }[] =
      [];
    const nodeCount = (view.byteLength - offsetNodes) / layout.NODE_LEN;
    for (let index = 2; index < nodeCount; index += 1) {
      const record = offsetNodes + index * layout.NODE_LEN;
      const kind = view.getUint32(record + layout.NODE_OFFSET_KIND, true);
      if (kind === layout.KIND_NODE_LIST) {
        continue;
      }
      const pos = view.getInt32(record + layout.NODE_OFFSET_POS, true);
      const nodeEnd = view.getInt32(record + layout.NODE_OFFSET_END, true);
      if (nodeEnd === end && pos >= 0 && source.slice(pos, nodeEnd).trimStart() === needle) {
        matchingNodes.push({ kind, pos, end: nodeEnd });
      }
    }
    assert.deepEqual(matchingNodes, [{ kind: SyntaxKind.CallExpression, pos: start - 1, end }]);

    return {
      sourceFile: path.relative(repoRoot, filePath),
      textIndex,
      pathIndex,
      exactNode: matchingNodes[0],
    };
  } finally {
    await snapshot?.dispose();
    await api.close();
  }
}

function readBinaryAstString(
  view: DataView,
  offsetStringTableOffsets: number,
  offsetStringTable: number,
  index: number,
): string {
  const start = view.getUint32(offsetStringTableOffsets + index * 4, true);
  const end = view.getUint32(offsetStringTableOffsets + (index + 1) * 4, true);
  const bytes = new Uint8Array(
    view.buffer,
    view.byteOffset + offsetStringTable + start,
    end - start,
  );
  return new TextDecoder().decode(bytes);
}

function comparablePath(value: string): string {
  const normalized = path.normalize(value);
  return process.platform === "darwin" || process.platform === "win32"
    ? normalized.toLowerCase()
    : normalized;
}

function parseRustIntegerConstant(
  source: string,
  name: string,
  type: "u32" | "u64" | "usize",
): number {
  const match = new RegExp(
    `^(?:pub )?const ${escapeRegExp(name)}: ${type} = ([0-9_]+);$`,
    "mu",
  ).exec(source);
  assert(match, `missing Rust integer constant ${name}`);
  const value = Number(match[1].replaceAll("_", ""));
  assert.ok(Number.isSafeInteger(value), `${name} must be a safe integer`);
  return value;
}

function parseExportedConstant(source: string, name: string): number {
  const match = new RegExp(
    `^export (?:declare )?const ${escapeRegExp(name)} = (0x[0-9A-Fa-f_]+|[0-9_]+);$`,
    "mu",
  ).exec(source);
  assert(match, `missing installed binary AST constant ${name}`);
  const value = Number(match[1].replaceAll("_", ""));
  assert.ok(Number.isSafeInteger(value), `${name} must be a safe integer`);
  return value;
}

function screamingSnakeToPascal(value: string): string {
  return value
    .toLowerCase()
    .split("_")
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1)}`)
    .join("");
}

function parseEnumMember(source: string, name: string): number {
  const match = new RegExp(`^\\s*${escapeRegExp(name)}\\s*=\\s*([0-9]+),?\\s*$`, "mu").exec(source);
  assert.ok(match, `missing installed TypeFlags.${name}`);
  const value = Number(match[1]);
  assert.ok(Number.isSafeInteger(value), `TypeFlags.${name} must be a safe integer`);
  return value;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
