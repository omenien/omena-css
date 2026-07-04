import assert from "node:assert/strict";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type IdentityTier = "session-stable" | "snapshot-local" | "session-runtime";

interface ScannedCollection {
  readonly id: string;
  readonly sourcePath: string;
  readonly sourceAnchor: string;
  readonly owner: string;
  readonly name: string;
  readonly collectionType: string;
  readonly keyType: string;
}

interface InventoryEntry extends ScannedCollection {
  readonly identityTier: IdentityTier;
  readonly persistentIdentityKey: boolean;
  readonly justification: string;
}

interface ScanTarget {
  readonly sourcePath: string;
  readonly structOwners?: readonly string[];
  readonly functionParameters?: readonly {
    readonly functionName: string;
    readonly parameterName: string;
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contractPath = "rust/omena-two-tier-identity-contract.md";
const inventoryPath = "rust/omena-two-tier-identity-inventory.json";

const scanTargets: readonly ScanTarget[] = [
  {
    sourcePath: "rust/crates/omena-query/src/style/salsa_memo.rs",
    structOwners: [
      "OmenaQueryStyleRevisionSelectorV0",
      "OmenaQueryStyleWorkspaceTransactionCommitV0",
      "OmenaQueryStyleWorkspaceTransactionCoreCommitV0",
      "OmenaQueryStyleWorkspaceTransactionV0",
      "OmenaQueryStyleMemoHostV0",
    ],
  },
  {
    sourcePath: "rust/crates/omena-lsp-server/src/state.rs",
    structOwners: ["LspFileIdentityInterner", "LspResolutionSettings", "LspShellState"],
  },
  {
    sourcePath: "rust/crates/omena-parser/src/closed_world.rs",
    functionParameters: [
      {
        functionName: "compute_reachability",
        parameterName: "by_instance",
      },
      {
        functionName: "stable_closure_hash",
        parameterName: "by_instance",
      },
    ],
  },
];

function main(): void {
  const expected = buildExpectedInventory();
  const inventoryFullPath = path.join(repoRoot, inventoryPath);
  if (process.argv.includes("--write")) {
    writeFileSync(inventoryFullPath, `${stableJson(expected)}\n`);
    console.log(`wrote ${inventoryPath}: entries=${expected.length}`);
    return;
  }

  assertContractText();
  assert.ok(existsSync(inventoryFullPath), `${inventoryPath} must exist`);
  const actual = JSON.parse(readFileSync(inventoryFullPath, "utf8")) as readonly InventoryEntry[];
  assert.deepEqual(
    actual,
    expected,
    `${inventoryPath} is out of date; run this check with --write`,
  );

  const persistentSnapshotLocal = actual.filter(
    (entry) => entry.identityTier === "snapshot-local" && entry.persistentIdentityKey,
  );
  assert.deepEqual(
    persistentSnapshotLocal,
    [],
    "snapshot-local keys must not key persistent identity stores",
  );

  const tierBuckets = new Set(actual.map((entry) => entry.identityTier));
  assert.ok(tierBuckets.has("session-stable"), "inventory must cover session-stable entries");
  assert.ok(tierBuckets.has("session-runtime"), "inventory must cover session-runtime entries");
  assert.ok(
    actual.length >= 14,
    `inventory must cover a non-vacuous structure count; got ${actual.length}`,
  );

  console.log(
    `two-tier identity contract: entries=${actual.length} sessionStable=${
      actual.filter((entry) => entry.identityTier === "session-stable").length
    } sessionRuntime=${actual.filter((entry) => entry.identityTier === "session-runtime").length}`,
  );
}

function assertContractText(): void {
  const text = readFileSync(path.join(repoRoot, contractPath), "utf8");
  for (const required of [
    "Clause T1",
    "Clause T2",
    "Clause T3",
    "Clause T4",
    "ModuleIdV0",
    "ModuleInstanceKeyV0",
    "LspFileId",
    "StableNodeKeyV0",
    "STYLE_IDENTITY_CACHE_VERSION",
    "CANONICALIZE_PATH_CACHE_VERSION",
    "Fact-Key Interning",
    "Cache Generation Clocks",
  ]) {
    assert.ok(text.includes(required), `${contractPath} must include ${required}`);
  }
}

function buildExpectedInventory(): readonly InventoryEntry[] {
  return scanTargets
    .flatMap((target) => scanTarget(target))
    .map(classifyCollection)
    .sort((left, right) => left.id.localeCompare(right.id));
}

function scanTarget(target: ScanTarget): readonly ScannedCollection[] {
  const fullPath = path.join(repoRoot, target.sourcePath);
  const source = readFileSync(fullPath, "utf8");
  const lines = source.split(/\r?\n/);
  const entries: ScannedCollection[] = [];
  let activeStruct: string | null = null;
  let structDepth = 0;
  let activeFunction: string | null = null;
  let functionParenDepth = 0;

  lines.forEach((line, index) => {
    const lineNumber = index + 1;
    const structMatch = /^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z0-9_]+)/.exec(line);
    if (structMatch) {
      activeStruct = structMatch[1];
      structDepth = braceDelta(line);
    } else if (activeStruct) {
      structDepth += braceDelta(line);
      if (structDepth <= 0) activeStruct = null;
    }

    const functionMatch = /^\s*fn\s+([A-Za-z0-9_]+)\s*\(/.exec(line);
    if (functionMatch) {
      activeFunction = functionMatch[1];
      functionParenDepth = parenDelta(line);
    }

    if (activeStruct && target.structOwners?.includes(activeStruct)) {
      const collection = parseCollectionDeclaration(line);
      if (collection) {
        entries.push(toScannedCollection(target.sourcePath, lineNumber, activeStruct, collection));
      }
    }

    if (target.functionParameters && activeFunction) {
      const shouldScan = target.functionParameters.some(
        (parameter) =>
          parameter.functionName === activeFunction && line.includes(`${parameter.parameterName}:`),
      );
      if (shouldScan) {
        const collection = parseCollectionDeclaration(line);
        if (collection) {
          entries.push(
            toScannedCollection(target.sourcePath, lineNumber, activeFunction, collection),
          );
        }
      }
    }

    if (activeFunction) {
      if (!functionMatch) {
        functionParenDepth += parenDelta(line);
      }
      if (functionParenDepth <= 0) activeFunction = null;
    }
  });

  return entries;
}

function toScannedCollection(
  sourcePath: string,
  lineNumber: number,
  owner: string,
  collection: Omit<ScannedCollection, "id" | "sourcePath" | "sourceAnchor" | "owner">,
): ScannedCollection {
  return {
    id: `${sourcePath}#${owner}.${collection.name}`,
    sourcePath,
    sourceAnchor: `${sourcePath}:${lineNumber}`,
    owner,
    ...collection,
  };
}

function parseCollectionDeclaration(
  line: string,
): Omit<ScannedCollection, "id" | "sourcePath" | "sourceAnchor" | "owner"> | null {
  const match =
    /(?:pub(?:\([^)]*\))?\s+)?(?<name>[A-Za-z0-9_]+)\s*:\s*&?(?<type>(?:BTreeMap|HashMap|IndexMap|BTreeSet|HashSet)<.+>)/.exec(
      line,
    );
  if (!match?.groups) return null;
  const collectionType = normalizeType(match.groups.type.replace(/[,;]\s*$/, ""));
  const container = /^([A-Za-z0-9_]+)</.exec(collectionType)?.[1];
  assert.ok(container, `missing container in ${line}`);
  const inner = collectionType.slice(container.length + 1, -1);
  const args = splitTopLevel(inner);
  const keyType = container.endsWith("Set") ? args[0] : args[0];
  return {
    name: match.groups.name,
    collectionType,
    keyType: normalizeType(keyType),
  };
}

function classifyCollection(entry: ScannedCollection): InventoryEntry {
  const lowerName = entry.name.toLowerCase();
  const lowerKey = entry.keyType.toLowerCase();

  if (lowerName.includes("progress_request_token")) {
    return {
      ...entry,
      identityTier: "session-runtime",
      persistentIdentityKey: false,
      justification:
        "LSP server request tokens are protocol runtime state, not cross-snapshot identity keys.",
    };
  }

  if (lowerKey.includes("usize") || lowerKey.includes("span") || lowerKey.includes("range")) {
    return {
      ...entry,
      identityTier: "snapshot-local",
      persistentIdentityKey: true,
      justification:
        "Byte positions, spans, and arena indices are snapshot-local and cannot key persistent stores.",
    };
  }

  if (entry.keyType === "String") {
    return {
      ...entry,
      identityTier: "session-stable",
      persistentIdentityKey: true,
      justification:
        "String key is a canonical style path, URI, URL, or content hash in the scanned persistent store.",
    };
  }

  if (entry.keyType === "LspFileId") {
    return {
      ...entry,
      identityTier: "session-stable",
      persistentIdentityKey: true,
      justification:
        "LspFileId is assigned by the session file-identity interner and is stable across revisions.",
    };
  }

  if (entry.keyType === "ModuleInstanceKeyV0") {
    return {
      ...entry,
      identityTier: "session-stable",
      persistentIdentityKey: true,
      justification:
        "ModuleInstanceKeyV0 keys a module path plus configuration in the closed-world linker.",
    };
  }

  throw new Error(`unclassified scanned identity key ${entry.id}: ${entry.keyType}`);
}

function splitTopLevel(value: string): readonly string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];
    if (char === "<" || char === "(" || char === "[" || char === "{") depth += 1;
    if (char === ">" || char === ")" || char === "]" || char === "}") depth -= 1;
    if (char === "," && depth === 0) {
      parts.push(normalizeType(value.slice(start, index)));
      start = index + 1;
    }
  }
  parts.push(normalizeType(value.slice(start)));
  return parts;
}

function normalizeType(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function braceDelta(line: string): number {
  let delta = 0;
  for (const char of line) {
    if (char === "{") delta += 1;
    if (char === "}") delta -= 1;
  }
  return delta;
}

function parenDelta(line: string): number {
  let delta = 0;
  for (const char of line) {
    if (char === "(") delta += 1;
    if (char === ")") delta -= 1;
  }
  return delta;
}

function stableJson(value: unknown): string {
  return JSON.stringify(value, null, 2);
}

main();
