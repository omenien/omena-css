import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";

interface PublishTrainClosure {
  readonly canonicalPublishOrder: readonly string[];
}

interface CargoPackage {
  readonly name: string;
  readonly manifest_path: string;
}

interface SerdeMember {
  readonly memberPath: string;
  readonly memberKind: "field" | "variant";
  readonly rustName: string;
  readonly serializeKey: string | null;
  readonly deserializeKey: string | null;
  readonly typeExpression: string;
  readonly optional: boolean;
  readonly flatten: boolean;
  readonly skipSerializing: boolean;
  readonly skipDeserializing: boolean;
}

interface SerdeType {
  readonly crate: string;
  readonly typeName: string;
  readonly kind: "struct" | "enum";
  readonly sourcePath: string;
  readonly nonExhaustive: boolean;
  readonly containerSerde: string;
  readonly members: readonly SerdeMember[];
}

interface SerdeCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-rust-serde-field-contracts";
  readonly domainAuthority: "rust.publish-train-closure.canonicalPublishOrder";
  readonly crateSeeds: readonly ["omena-bridge", "omena-tsgo-client"];
  readonly seedSelection: "source type-fact producer and external provider wire crates";
  readonly domainMethod: "publish-train-bound crate seeds plus recursive src/**/*.rs walk";
  readonly responseSurfaceSplitComparison: {
    readonly records: "{crate,typeName,sourcePath,line}";
    readonly doesNotRecord: "fields, optionality, or serde keys";
    readonly limitation: string;
  };
  readonly crateCount: number;
  readonly sourceFileCount: number;
  readonly types: readonly SerdeType[];
}

interface AdapterMember {
  readonly interfaceName: string;
  readonly member: string;
  readonly optional: boolean;
  readonly typeExpression: string;
}

interface AdapterSnapshot {
  readonly schemaVersion: "0";
  readonly product: "omena-css-build-adapter-interface-members";
  readonly source: "packages/css-build-adapter/index.d.ts";
  readonly changeClasses: readonly AdapterChangeClass[];
  readonly executionSummaryWireResidual: readonly string[];
  readonly members: readonly AdapterMember[];
}

type AdapterChangeClass = "additive-optional" | "additive-required" | "removal" | "narrowing";

interface AdapterChange {
  readonly key: string;
  readonly class: AdapterChangeClass;
  readonly detail: string;
}

const repoRoot = process.cwd();
const serdeCensusPath = path.join(repoRoot, "rust/omena-rust-serde-field-contracts.json");
const adapterSnapshotPath = path.join(
  repoRoot,
  "packages/css-build-adapter/interface-member-snapshot.json",
);
const writeSerde = process.argv.includes("--write-serde");
const writeAdapter = process.argv.includes("--write-adapter");
const initializeAdapter = process.argv.includes("--initialize-adapter");
const acceptedMembers = readArgs("--accept-member");
const acceptedClasses = readArgs("--accept-class") as AdapterChangeClass[];
const adapterChangeClasses: readonly AdapterChangeClass[] = [
  "additive-optional",
  "additive-required",
  "removal",
  "narrowing",
];
const serdeCrateSeeds = ["omena-bridge", "omena-tsgo-client"] as const;

const closure = readPublishTrainClosure();
const packages = readCargoPackages();
const packageByName = new Map(packages.map((pkg) => [pkg.name, pkg]));
for (const crate of serdeCrateSeeds) {
  assert(
    closure.canonicalPublishOrder.includes(crate),
    `serde field contract crate seed is outside the computed publish train: ${crate}`,
  );
}
const sourceFiles = serdeCrateSeeds.flatMap((crate) => {
  const pkg = packageByName.get(crate);
  assert(pkg, `missing Cargo package for publish-train crate ${crate}`);
  return listRustSources(path.join(path.dirname(pkg.manifest_path), "src")).map((sourcePath) => ({
    crate,
    sourcePath,
  }));
});
const currentSerdeTypes = sourceFiles
  .flatMap(({ crate, sourcePath }) => scanSerdeTypes(crate, sourcePath))
  .toSorted((left, right) =>
    `${left.crate}:${left.typeName}:${left.sourcePath}`.localeCompare(
      `${right.crate}:${right.typeName}:${right.sourcePath}`,
    ),
  );
// A missing recursive source root can make the domain empty; Cargo metadata and
// the required wire contract keep that producer failure observable here.
assert(currentSerdeTypes.length > 0, "serde field census must not be empty");
assert(
  currentSerdeTypes.some((row) => row.typeName === "SourceTypeFactTargetSkippedFactV0"),
  "serde field census must include the source type-fact skip contract",
);
const currentCensus: SerdeCensus = {
  schemaVersion: "0",
  product: "omena-rust-serde-field-contracts",
  domainAuthority: "rust.publish-train-closure.canonicalPublishOrder",
  crateSeeds: serdeCrateSeeds,
  seedSelection: "source type-fact producer and external provider wire crates",
  domainMethod: "publish-train-bound crate seeds plus recursive src/**/*.rs walk",
  responseSurfaceSplitComparison: {
    records: "{crate,typeName,sourcePath,line}",
    doesNotRecord: "fields, optionality, or serde keys",
    limitation:
      "A field insertion can move later declaration lines and make that census drift, but it cannot identify the changed field or its semver class; regeneration can therefore launder the contract change.",
  },
  crateCount: serdeCrateSeeds.length,
  sourceFileCount: sourceFiles.length,
  types: currentSerdeTypes,
};

if (writeSerde) {
  writeFileSync(serdeCensusPath, `${JSON.stringify(currentCensus, null, 2)}\n`);
} else {
  const expected = JSON.parse(readFileSync(serdeCensusPath, "utf8")) as SerdeCensus;
  assert.equal(expected.schemaVersion, "0", "serde field census schemaVersion");
  assert.equal(expected.product, "omena-rust-serde-field-contracts", "serde field census product");
  const changes = compareSerdeCensus(expected, currentCensus);
  if (changes.length > 0) {
    throw new Error(
      `Rust serde field contract changed:\n${changes.map((row) => `- ${row}`).join("\n")}`,
    );
  }
}

const adapterMembers = scanAdapterMembers();
const currentAdapter: AdapterSnapshot = {
  schemaVersion: "0",
  product: "omena-css-build-adapter-interface-members",
  source: "packages/css-build-adapter/index.d.ts",
  changeClasses: adapterChangeClasses,
  executionSummaryWireResidual: readExecutionSummaryWireResidual(),
  members: adapterMembers,
};
if (initializeAdapter) {
  writeFileSync(adapterSnapshotPath, `${JSON.stringify(currentAdapter, null, 2)}\n`);
} else {
  const expected = JSON.parse(readFileSync(adapterSnapshotPath, "utf8")) as AdapterSnapshot;
  assert.deepEqual(
    expected.changeClasses,
    adapterChangeClasses,
    "adapter interface change-class vocabulary",
  );
  assert.deepEqual(
    expected.executionSummaryWireResidual,
    currentAdapter.executionSummaryWireResidual,
    "execution-summary declaration residual changed",
  );
  const changes = compareAdapterSnapshot(expected, currentAdapter);
  // The declaration parser can emit every change class below. Only a new
  // optional member is backward-compatible without an explicit acceptance.
  const blocking = changes.filter((change) => change.class !== "additive-optional");
  if (writeAdapter) {
    assert.equal(
      changes.length,
      acceptedMembers.length,
      "adapter snapshot update must explicitly accept every member change",
    );
    assert.equal(
      acceptedClasses.length,
      acceptedMembers.length,
      "every accepted adapter member must carry a change class",
    );
    for (const [index, change] of changes.entries()) {
      assert.equal(
        change.key,
        acceptedMembers[index],
        "accepted adapter member does not match the diff",
      );
      assert.equal(
        change.class,
        acceptedClasses[index],
        "accepted adapter change class is incorrect",
      );
    }
    writeFileSync(adapterSnapshotPath, `${JSON.stringify(currentAdapter, null, 2)}\n`);
  } else if (blocking.length > 0) {
    throw new Error(
      `CSS build adapter declaration changed:\n${blocking
        .map((change) => `- ${change.key}: ${change.class}: ${change.detail}`)
        .join("\n")}`,
    );
  }
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.contract-shape-census",
      serdeCrateCount: currentCensus.crateCount,
      serdeSourceFileCount: currentCensus.sourceFileCount,
      serdeTypeCount: currentSerdeTypes.length,
      frozenSerdeTypeCount: currentSerdeTypes.filter((row) => !row.nonExhaustive).length,
      nonExhaustiveSerdeTypeCount: currentSerdeTypes.filter((row) => row.nonExhaustive).length,
      adapterInterfaceCount: new Set(adapterMembers.map((row) => row.interfaceName)).size,
      adapterMemberCount: adapterMembers.length,
      adapterExecutionSummaryResidualCount: currentAdapter.executionSummaryWireResidual.length,
    },
    null,
    2,
  )}\n`,
);

function compareSerdeCensus(expected: SerdeCensus, current: SerdeCensus): string[] {
  const changes: string[] = [];
  // Source edits and baseline edits are independent producers of these values,
  // so metadata drift and member drift can each make this comparison false.
  if (current.schemaVersion !== expected.schemaVersion) {
    changes.push(
      `census.schemaVersion: metadata-change ${expected.schemaVersion}->${current.schemaVersion}`,
    );
  }
  if (current.product !== expected.product) {
    changes.push(`census.product: metadata-change ${expected.product}->${current.product}`);
  }
  if (current.domainAuthority !== expected.domainAuthority) {
    changes.push(
      `census.domainAuthority: metadata-change ${expected.domainAuthority}->${current.domainAuthority}`,
    );
  }
  if (current.domainMethod !== expected.domainMethod) {
    changes.push(
      `census.domainMethod: metadata-change ${expected.domainMethod}->${current.domainMethod}`,
    );
  }
  if (
    JSON.stringify(current.responseSurfaceSplitComparison) !==
    JSON.stringify(expected.responseSurfaceSplitComparison)
  ) {
    changes.push("census.responseSurfaceComparison: metadata-change");
  }
  if (current.crateCount !== expected.crateCount) {
    changes.push(`census.crateCount: domain-change ${expected.crateCount}->${current.crateCount}`);
  }
  if (current.sourceFileCount !== expected.sourceFileCount) {
    changes.push(
      `census.sourceFileCount: domain-change ${expected.sourceFileCount}->${current.sourceFileCount}`,
    );
  }
  if (JSON.stringify(current.crateSeeds) !== JSON.stringify(expected.crateSeeds)) {
    changes.push("census.crateSeeds: domain-change");
  }
  if (current.seedSelection !== expected.seedSelection) {
    changes.push("census.seedSelection: metadata-change");
  }
  const expectedTypes = new Map(expected.types.map((row) => [serdeTypeKey(row), row]));
  const currentTypes = new Map(current.types.map((row) => [serdeTypeKey(row), row]));
  for (const [key, row] of currentTypes) {
    const before = expectedTypes.get(key);
    if (!before) {
      changes.push(`${key}: additive-public-type`);
      continue;
    }
    if (
      before.kind !== row.kind ||
      before.nonExhaustive !== row.nonExhaustive ||
      before.containerSerde !== row.containerSerde
    ) {
      changes.push(`${key}: declaration-contract-change`);
      continue;
    }
    const beforeMembers = new Map(before.members.map((member) => [member.memberPath, member]));
    const currentMembers = new Map(row.members.map((member) => [member.memberPath, member]));
    for (const [memberPath, member] of currentMembers) {
      const previous = beforeMembers.get(memberPath);
      if (!previous) {
        // Non-exhaustive declarations admit new members; an exhaustive
        // declaration can emit this branch whenever a public field is added.
        if (!row.nonExhaustive) {
          changes.push(
            `${key}.${memberPath}: ${member.optional ? "additive-optional" : "additive-required"}`,
          );
        }
        continue;
      }
      if (
        previous.rustName !== member.rustName ||
        previous.serializeKey !== member.serializeKey ||
        previous.deserializeKey !== member.deserializeKey
      ) {
        changes.push(`${key}.${memberPath}: rename`);
      } else if (
        previous.optional !== member.optional ||
        previous.skipSerializing !== member.skipSerializing ||
        previous.skipDeserializing !== member.skipDeserializing ||
        previous.flatten !== member.flatten
      ) {
        changes.push(`${key}.${memberPath}: optionality-change`);
      } else if (previous.typeExpression !== member.typeExpression) {
        changes.push(`${key}.${memberPath}: narrowing-or-type-change`);
      }
    }
    for (const memberPath of beforeMembers.keys()) {
      if (!currentMembers.has(memberPath)) {
        changes.push(`${key}.${memberPath}: removal`);
      }
    }
  }
  for (const key of expectedTypes.keys()) {
    if (!currentTypes.has(key)) changes.push(`${key}: removal`);
  }
  return changes.toSorted();
}

function scanSerdeTypes(crate: string, sourcePath: string): SerdeType[] {
  const source = readFileSync(sourcePath, "utf8");
  const lines = source.split(/\r?\n/u);
  const offsets: number[] = [];
  let offset = 0;
  for (const line of lines) {
    offsets.push(offset);
    offset += line.length + 1;
  }
  const rows: SerdeType[] = [];
  let pendingAttributes: string[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();
    if (trimmed.startsWith("#[")) {
      let attribute = lines[index];
      while (countCharacter(attribute, "[") > countCharacter(attribute, "]")) {
        index += 1;
        attribute += `\n${lines[index] ?? ""}`;
      }
      pendingAttributes.push(attribute.trim());
      continue;
    }
    if (!trimmed || trimmed.startsWith("///") || trimmed.startsWith("//!")) continue;
    const declaration = /^\s*pub\s+(struct|enum)\s+([A-Z][A-Za-z0-9_]*)/u.exec(lines[index]);
    if (!declaration) {
      pendingAttributes = [];
      continue;
    }
    const attributes = pendingAttributes.join("\n");
    pendingAttributes = [];
    if (!/derive\s*\([^)]*(?:Serialize|Deserialize)/su.test(attributes)) continue;
    const kind = declaration[1] as "struct" | "enum";
    const typeName = declaration[2];
    const nameOffset = lines[index].indexOf(typeName);
    const declarationOffset = offsets[index] + nameOffset + typeName.length;
    let body: ReturnType<typeof readDeclarationBody>;
    try {
      body = readDeclarationBody(source, declarationOffset);
    } catch (error) {
      throw new Error(
        `failed to read ${crate}:${typeName}:${path.relative(repoRoot, sourcePath)}:${index + 1}`,
        { cause: error },
      );
    }
    const serde = parseSerdeAttributes(attributes);
    const nonExhaustive = /#\s*\[\s*non_exhaustive\s*\]/u.test(attributes);
    rows.push({
      crate,
      typeName,
      kind,
      sourcePath: path.relative(repoRoot, sourcePath),
      nonExhaustive,
      containerSerde: normalizeSerdeAttributeText(attributes),
      members: kind === "struct" ? parseStructMembers(body, serde) : parseEnumMembers(body, serde),
    });
  }
  return rows;
}

function readDeclarationBody(
  source: string,
  start: number,
): { readonly delimiter: "{" | "(" | ";"; readonly content: string } {
  let angleDepth = 0;
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (character === "<") angleDepth += 1;
    if (character === ">" && angleDepth > 0) angleDepth -= 1;
    if (angleDepth > 0) continue;
    if (character === "{") {
      const end = findMatchingDelimiter(source, index, "{", "}");
      return { delimiter: "{", content: source.slice(index + 1, end) };
    }
    if (character === "(") {
      const end = findMatchingDelimiter(source, index, "(", ")");
      return { delimiter: "(", content: source.slice(index + 1, end) };
    }
    if (character === ";") return { delimiter: ";", content: "" };
  }
  throw new Error("unterminated Rust declaration");
}

function parseStructMembers(
  body: { readonly delimiter: "{" | "(" | ";"; readonly content: string },
  containerSerde: ParsedSerdeAttributes,
): SerdeMember[] {
  if (body.delimiter === ";") return [];
  const fields = splitTopLevel(stripRustComments(body.content), ",");
  return fields.flatMap((field, index) => {
    const parsed =
      body.delimiter === "{"
        ? parseNamedRustField(field, containerSerde.fieldRenameAll, `${index}`)
        : parseTupleRustField(field, containerSerde.fieldRenameAll, `${index}`);
    return parsed ? [parsed] : [];
  });
}

function parseEnumMembers(
  body: { readonly delimiter: "{" | "(" | ";"; readonly content: string },
  containerSerde: ParsedSerdeAttributes,
): SerdeMember[] {
  assert.equal(body.delimiter, "{", "serde enum must have a braced body");
  return splitTopLevel(stripRustComments(body.content), ",").flatMap((variantSource) => {
    const { attributes, rest } = takeLeadingAttributes(variantSource);
    const variantMatch = /^([A-Z][A-Za-z0-9_]*)/u.exec(stripRustComments(rest).trim());
    if (!variantMatch) return [];
    const variantName = variantMatch[1];
    const variantSerde = parseSerdeAttributes(attributes);
    const serializeKey =
      variantSerde.serializeRename ??
      applyRenameRule(variantName, containerSerde.serializeRenameAll);
    const deserializeKey =
      variantSerde.deserializeRename ??
      applyRenameRule(variantName, containerSerde.deserializeRenameAll);
    const members: SerdeMember[] = [
      {
        memberPath: variantName,
        memberKind: "variant",
        rustName: variantName,
        serializeKey: variantSerde.skipSerializing ? null : serializeKey,
        deserializeKey: variantSerde.skipDeserializing ? null : deserializeKey,
        typeExpression: "variant",
        optional: false,
        flatten: false,
        skipSerializing: variantSerde.skipSerializing,
        skipDeserializing: variantSerde.skipDeserializing,
      },
    ];
    const suffix = rest.slice(rest.indexOf(variantName) + variantName.length).trim();
    if (suffix.startsWith("{")) {
      const end = findMatchingDelimiter(suffix, 0, "{", "}");
      for (const [index, field] of splitTopLevel(suffix.slice(1, end), ",").entries()) {
        const parsed = parseNamedRustField(
          field,
          containerSerde.variantFieldRenameAll,
          `${variantName}.${index}`,
          variantName,
        );
        if (parsed) members.push(parsed);
      }
    } else if (suffix.startsWith("(")) {
      const end = findMatchingDelimiter(suffix, 0, "(", ")");
      for (const [index, field] of splitTopLevel(suffix.slice(1, end), ",").entries()) {
        const parsed = parseTupleRustField(
          field,
          containerSerde.variantFieldRenameAll,
          `${variantName}.${index}`,
          variantName,
        );
        if (parsed) members.push(parsed);
      }
    }
    return members;
  });
}

function parseNamedRustField(
  source: string,
  renameAll: RenameRulePair,
  fallbackPath: string,
  prefix?: string,
): SerdeMember | undefined {
  const { attributes, rest } = takeLeadingAttributes(source);
  const cleaned = stripRustComments(rest).trim();
  if (!cleaned) return undefined;
  const match = /^(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([\s\S]+)$/u.exec(
    cleaned,
  );
  if (!match) {
    throw new Error(`unsupported named serde field ${fallbackPath}: ${JSON.stringify(cleaned)}`);
  }
  return serdeField(match[1], match[2], attributes, renameAll, prefix);
}

function parseTupleRustField(
  source: string,
  renameAll: RenameRulePair,
  fallbackPath: string,
  prefix?: string,
): SerdeMember | undefined {
  const { attributes, rest } = takeLeadingAttributes(source);
  const typeExpression = stripRustComments(rest)
    .replace(/^pub(?:\([^)]*\))?\s+/u, "")
    .trim();
  if (!typeExpression) return undefined;
  const rustName = fallbackPath.split(".").at(-1) ?? fallbackPath;
  return serdeField(rustName, typeExpression, attributes, renameAll, prefix, fallbackPath);
}

function serdeField(
  rustName: string,
  rawTypeExpression: string,
  attributes: string,
  renameAll: RenameRulePair,
  prefix?: string,
  explicitPath?: string,
): SerdeMember {
  const serde = parseSerdeAttributes(attributes);
  const typeExpression = normalizeTypeExpression(rawTypeExpression);
  const serializeKey = serde.flatten
    ? null
    : (serde.serializeRename ?? applyRenameRule(rustName, renameAll.serialize));
  const deserializeKey = serde.flatten
    ? null
    : (serde.deserializeRename ?? applyRenameRule(rustName, renameAll.deserialize));
  return {
    memberPath: explicitPath ?? (prefix ? `${prefix}.${rustName}` : rustName),
    memberKind: "field",
    rustName,
    serializeKey: serde.skipSerializing ? null : serializeKey,
    deserializeKey: serde.skipDeserializing ? null : deserializeKey,
    typeExpression,
    optional: /\bOption\s*</u.test(typeExpression) || serde.skipSerializingIf,
    flatten: serde.flatten,
    skipSerializing: serde.skipSerializing,
    skipDeserializing: serde.skipDeserializing,
  };
}

interface RenameRulePair {
  readonly serialize?: string;
  readonly deserialize?: string;
}

interface ParsedSerdeAttributes {
  readonly serializeRename?: string;
  readonly deserializeRename?: string;
  readonly serializeRenameAll?: string;
  readonly deserializeRenameAll?: string;
  readonly fieldRenameAll: RenameRulePair;
  readonly variantFieldRenameAll: RenameRulePair;
  readonly skipSerializing: boolean;
  readonly skipDeserializing: boolean;
  readonly skipSerializingIf: boolean;
  readonly flatten: boolean;
}

function parseSerdeAttributes(attributes: string): ParsedSerdeAttributes {
  const serdeBodies = [...attributes.matchAll(/#\s*\[\s*serde\s*\(([\s\S]*?)\)\s*\]/gu)].map(
    (match) => match[1],
  );
  const source = serdeBodies.join(",");
  const directRename = /\brename\s*=\s*"([^"]+)"/u.exec(source)?.[1];
  const renameGroup = /\brename\s*\(([\s\S]*?)\)/u.exec(source)?.[1] ?? "";
  const directRenameAll = /\brename_all\s*=\s*"([^"]+)"/u.exec(source)?.[1];
  const renameAllGroup = /\brename_all\s*\(([\s\S]*?)\)/u.exec(source)?.[1] ?? "";
  const directRenameAllFields = /\brename_all_fields\s*=\s*"([^"]+)"/u.exec(source)?.[1];
  const renameAllFieldsGroup = /\brename_all_fields\s*\(([\s\S]*?)\)/u.exec(source)?.[1] ?? "";
  const hasToken = (token: string) =>
    new RegExp(`(?:^|,)\\s*${token}\\s*(?:,|$)`, "u").test(source);
  return {
    serializeRename: /\bserialize\s*=\s*"([^"]+)"/u.exec(renameGroup)?.[1] ?? directRename,
    deserializeRename: /\bdeserialize\s*=\s*"([^"]+)"/u.exec(renameGroup)?.[1] ?? directRename,
    serializeRenameAll: /\bserialize\s*=\s*"([^"]+)"/u.exec(renameAllGroup)?.[1] ?? directRenameAll,
    deserializeRenameAll:
      /\bdeserialize\s*=\s*"([^"]+)"/u.exec(renameAllGroup)?.[1] ?? directRenameAll,
    fieldRenameAll: {
      serialize: directRenameAll,
      deserialize: directRenameAll,
    },
    variantFieldRenameAll: {
      serialize:
        /\bserialize\s*=\s*"([^"]+)"/u.exec(renameAllFieldsGroup)?.[1] ?? directRenameAllFields,
      deserialize:
        /\bdeserialize\s*=\s*"([^"]+)"/u.exec(renameAllFieldsGroup)?.[1] ?? directRenameAllFields,
    },
    skipSerializing: hasToken("skip") || hasToken("skip_serializing"),
    skipDeserializing: hasToken("skip") || hasToken("skip_deserializing"),
    skipSerializingIf: /\bskip_serializing_if\s*=/u.test(source),
    flatten: hasToken("flatten"),
  };
}

function normalizeSerdeAttributeText(attributes: string): string {
  return [...attributes.matchAll(/#\s*\[\s*serde\s*\(([\s\S]*?)\)\s*\]/gu)]
    .map((match) => match[1].replace(/\s+/gu, " ").trim())
    .join(" | ");
}

function applyRenameRule(name: string, rule: string | undefined): string {
  if (!rule) return name;
  const words = splitIdentifierWords(name);
  switch (rule) {
    case "lowercase":
      return words.join("").toLowerCase();
    case "UPPERCASE":
      return words.join("").toUpperCase();
    case "PascalCase":
      return words.map(capitalize).join("");
    case "camelCase":
      return words[0].toLowerCase() + words.slice(1).map(capitalize).join("");
    case "snake_case":
      return words.join("_").toLowerCase();
    case "SCREAMING_SNAKE_CASE":
      return words.join("_").toUpperCase();
    case "kebab-case":
      return words.join("-").toLowerCase();
    case "SCREAMING-KEBAB-CASE":
      return words.join("-").toUpperCase();
    default:
      throw new Error(`unsupported serde rename rule ${rule}`);
  }
}

function splitIdentifierWords(name: string): string[] {
  const normalized = name
    .replace(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .replace(/([A-Z]+)([A-Z][a-z])/gu, "$1_$2");
  return normalized.split(/_+/u).filter(Boolean);
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
}

function scanAdapterMembers(): AdapterMember[] {
  const source = readFileSync(path.join(repoRoot, "packages/css-build-adapter/index.d.ts"), "utf8");
  const rows: AdapterMember[] = [];
  const pattern = /export\s+interface\s+([A-Za-z_][A-Za-z0-9_]*)[^{]*\{/gu;
  for (const match of source.matchAll(pattern)) {
    const interfaceName = match[1];
    const open = (match.index ?? 0) + match[0].lastIndexOf("{");
    const close = findMatchingDelimiter(source, open, "{", "}");
    for (const memberSource of splitTopLevel(source.slice(open + 1, close), ";")) {
      const cleaned = memberSource.replace(/\s+/gu, " ").trim();
      if (!cleaned) continue;
      const member =
        /^(?:readonly\s+)?(\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)(\?)?\s*:\s*([\s\S]+)$/u.exec(cleaned);
      if (!member) {
        throw new Error(`unsupported adapter interface member ${interfaceName}: ${cleaned}`);
      }
      rows.push({
        interfaceName,
        member: member[1],
        optional: member[2] === "?",
        typeExpression: normalizeTypeExpression(member[3]),
      });
    }
  }
  assert.equal(new Set(rows.map((row) => row.interfaceName)).size, 27);
  return rows.toSorted((left, right) =>
    `${left.interfaceName}.${left.member}`.localeCompare(`${right.interfaceName}.${right.member}`),
  );
}

function compareAdapterSnapshot(
  expected: AdapterSnapshot,
  current: AdapterSnapshot,
): AdapterChange[] {
  const before = new Map(expected.members.map((row) => [adapterMemberKey(row), row]));
  const after = new Map(current.members.map((row) => [adapterMemberKey(row), row]));
  const changes: AdapterChange[] = [];
  for (const [key, row] of after) {
    const previous = before.get(key);
    if (!previous) {
      changes.push({
        key,
        class: row.optional ? "additive-optional" : "additive-required",
        detail: `new ${row.optional ? "optional" : "required"} member`,
      });
      continue;
    }
    if (previous.optional && !row.optional) {
      changes.push({ key, class: "narrowing", detail: "optional member became required" });
    } else if (!previous.optional && row.optional) {
      changes.push({
        key,
        class: "additive-optional",
        detail: "required member became optional",
      });
    } else if (previous.typeExpression !== row.typeExpression) {
      changes.push({
        key,
        class: "narrowing",
        detail: `type changed from ${previous.typeExpression} to ${row.typeExpression}`,
      });
    }
  }
  for (const key of before.keys()) {
    if (!after.has(key)) changes.push({ key, class: "removal", detail: "member removed" });
  }
  return changes.toSorted((left, right) => left.key.localeCompare(right.key));
}

function readExecutionSummaryWireResidual(): string[] {
  const source = readFileSync(
    path.join(repoRoot, "scripts/check-rust-omena-query-bundle-execution-scope.ts"),
    "utf8",
  );
  const block =
    /const TRANSFORM_EXECUTION_SUMMARY_UNDECLARED_WIRE_KEYS = \[([\s\S]*?)\]\.toSorted\(\);/u.exec(
      source,
    );
  assert(block, "missing execution-summary wire residual authority");
  return [...block[1].matchAll(/"([^"]+)"/gu)].map((match) => match[1]).toSorted();
}

function serdeTypeKey(row: SerdeType): string {
  return `${row.crate}:${row.typeName}:${row.sourcePath}`;
}

function adapterMemberKey(row: AdapterMember): string {
  return `${row.interfaceName}.${row.member}`;
}

function normalizeTypeExpression(source: string): string {
  return source
    .replace(/\s+/gu, " ")
    .replace(/\s*([<>,:&|()[\]])\s*/gu, "$1")
    .trim();
}

function takeLeadingAttributes(source: string): {
  readonly attributes: string;
  readonly rest: string;
} {
  let rest = source.trim();
  const attributes: string[] = [];
  while (rest.startsWith("#[")) {
    const end = findMatchingDelimiter(rest, 1, "[", "]");
    attributes.push(rest.slice(0, end + 1));
    rest = rest.slice(end + 1).trim();
  }
  return { attributes: attributes.join("\n"), rest };
}

function splitTopLevel(source: string, delimiter: string): string[] {
  const parts: string[] = [];
  let start = 0;
  const stack: string[] = [];
  let quote: string | undefined;
  let escaped = false;
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === quote) quote = undefined;
      continue;
    }
    if (character === '"') {
      quote = character;
      continue;
    }
    if ("({[<".includes(character)) stack.push(character);
    else if (")}]>".includes(character)) stack.pop();
    else if (character === delimiter && stack.length === 0) {
      parts.push(source.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(source.slice(start));
  return parts;
}

function findMatchingDelimiter(source: string, start: number, open: string, close: string): number {
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === quote) quote = undefined;
      continue;
    }
    if (character === '"') {
      quote = character;
      continue;
    }
    if (character === open) depth += 1;
    if (character === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error(`unterminated ${open}${close} block`);
}

function stripRustComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//gu, "").replace(/\/\/.*$/gmu, "");
}

function countCharacter(source: string, character: string): number {
  return [...source].filter((candidate) => candidate === character).length;
}

function listRustSources(directory: string): string[] {
  if (!existsSync(directory)) return [];
  return readdirSync(directory).flatMap((entry) => {
    const absolute = path.join(directory, entry);
    const stats = statSync(absolute);
    if (stats.isDirectory()) return listRustSources(absolute);
    return stats.isFile() && absolute.endsWith(".rs") ? [absolute] : [];
  });
}

function readPublishTrainClosure(): PublishTrainClosure {
  return JSON.parse(
    execFileSync(
      process.execPath,
      ["--import", "tsx", "./scripts/check-rust-publish-train-closure.ts"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as PublishTrainClosure;
}

function readCargoPackages(): readonly CargoPackage[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as { readonly packages: readonly CargoPackage[] };
  return metadata.packages;
}

function readArgs(name: string): string[] {
  return process.argv.flatMap((argument, index) =>
    argument === name && process.argv[index + 1] ? [process.argv[index + 1]] : [],
  );
}
