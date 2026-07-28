import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const rustSourcePath = path.join(repoRoot, "rust/crates/omena-tsgo-client/src/lib.rs");
const packagePath = path.join(repoRoot, "node_modules/@typescript/native-preview/package.json");
const enumPath = path.join(
  repoRoot,
  "node_modules/@typescript/native-preview/dist/enums/typeFlags.enum.d.ts",
);

const rustSource = readFileSync(rustSourcePath, "utf8");
const packageJson = JSON.parse(readFileSync(packagePath, "utf8")) as {
  readonly name?: unknown;
  readonly version?: unknown;
};
const enumSource = readFileSync(enumPath, "utf8");

assert.equal(packageJson.name, "@typescript/native-preview", "native TypeScript package name");
assert.equal(typeof packageJson.version, "string", "native TypeScript package version");

const bindings = [
  ["TSGO_TYPE_FLAGS_ANY", "Any"],
  ["TSGO_TYPE_FLAGS_UNDEFINED", "Undefined"],
  ["TSGO_TYPE_FLAGS_NULL", "Null"],
  ["TSGO_TYPE_FLAGS_UNION", "Union"],
] as const;

const observed = bindings.map(([rustName, enumName]) => {
  const rustValue = parseRustConstant(rustSource, rustName);
  const enumValue = parseEnumMember(enumSource, enumName);
  assert.equal(
    rustValue,
    enumValue,
    `${rustName} must match @typescript/native-preview TypeFlags.${enumName}`,
  );
  return { rustName, enumName, value: rustValue };
});

assert.equal(
  new Set(observed.map((entry) => entry.value)).size,
  observed.length,
  "bound TypeFlags values must remain distinct",
);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena.tsgo.type-flags-abi",
      package: packageJson.name,
      version: packageJson.version,
      bindings: observed,
    },
    null,
    2,
  )}\n`,
);

function parseRustConstant(source: string, name: string): number {
  const match = new RegExp(`pub const ${escapeRegExp(name)}: u64 = ([0-9_]+);`, "u").exec(source);
  assert.ok(match, `missing Rust TypeFlags constant ${name}`);
  const value = Number(match[1].replaceAll("_", ""));
  assert.ok(Number.isSafeInteger(value), `${name} must be a safe integer`);
  return value;
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
