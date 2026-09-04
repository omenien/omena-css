import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  compareCodePoint,
  maskRustCommentsAndLiterals,
  matchingRustDelimiter,
  rustNamedFunctions,
} from "./lib/rust-write-authority";
import {
  attributeRegions,
  attributes,
  cfgPredicates,
  combinePredicates,
  deriveRustModuleContexts,
  evaluateCfg,
  moduleDeclarations,
  parseRustcCfg,
  renderCfg,
  withPackageCfg,
  withTest,
  type CfgExpr,
  type CfgSet,
} from "./lib/rust-cfg-acquisition";

interface CargoTarget {
  readonly name: string;
  readonly kind: readonly string[];
  readonly src_path: string;
}

interface CargoPackage {
  readonly id: string;
  readonly name: string;
  readonly manifest_path: string;
  readonly targets: readonly CargoTarget[];
  readonly features: Readonly<Record<string, readonly string[]>>;
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
  readonly resolve: {
    readonly nodes: readonly {
      readonly id: string;
      readonly deps: readonly {
        readonly pkg: string;
        readonly dep_kinds: readonly { readonly kind: string | null }[];
      }[];
    }[];
  };
}

interface PrecisionPoint {
  readonly id: string;
  readonly sourcePath: string;
  readonly function: string;
  readonly disposition?: { readonly bindingId?: string; readonly kind?: string };
  readonly reason?: string;
}

interface MutationChange {
  readonly sourcePath: string;
  readonly from: string;
  readonly to: string;
}

interface MutationProbe {
  readonly id: string;
  readonly sourcePath?: string;
  readonly from?: string;
  readonly to?: string;
  readonly changes?: readonly MutationChange[];
  readonly command: readonly string[];
}

interface GatedExclusion {
  readonly id: string;
  readonly pointId: string;
  readonly probe: string;
}

interface PrecisionAuthority {
  readonly precisionEmissionPoints: readonly PrecisionPoint[];
  readonly mutationProbes: readonly MutationProbe[];
  readonly gatedExclusions?: readonly GatedExclusion[];
}

interface RegisteredCallSite {
  readonly crate: string;
  readonly file: string;
  readonly item: string;
  readonly member: string;
  readonly ordinal: number;
  readonly owner: string;
}

interface RegisteredContainer {
  readonly identity: string;
  readonly owner: string;
}

interface CensusInstrumentAuthority {
  readonly precision: {
    readonly familyCallSites: readonly RegisteredCallSite[];
    readonly deserializationContainers: readonly RegisteredContainer[];
    readonly birthFloors: {
      readonly familyCallSites: number;
      readonly deserializationContainers: number;
    };
  };
}

interface RustSource {
  readonly crate: string;
  readonly file: string;
  readonly modulePath: string;
  readonly targetName: string;
  readonly targetRoot: string;
  readonly targetKind: readonly string[];
  readonly package: CargoPackage;
  readonly inheritedPredicates: readonly CfgExpr[];
  readonly runnerCfg: CfgSet;
  readonly source: string;
  readonly code: string;
}

interface TypeDefinition {
  readonly identity: string;
  readonly symbolKey: string;
  readonly crate: string;
  readonly targetRoot: string;
  readonly targetKind: readonly string[];
  readonly modulePath: string;
  readonly name: string;
  readonly file: string;
  readonly body: string;
  readonly fieldTypes: readonly {
    readonly field: string;
    readonly type: string;
    readonly offset: number;
  }[];
  readonly derivesDeserialize: boolean;
  readonly genericParameters: ReadonlySet<string>;
}

interface AliasDefinition {
  readonly identity: string;
  readonly symbolKey: string;
  readonly crate: string;
  readonly targetRoot: string;
  readonly targetKind: readonly string[];
  readonly modulePath: string;
  readonly name: string;
  readonly target: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const precisionAuthorityPath = path.join(repoRoot, "rust/omena-precision-floor-authority.json");
const instrumentAuthorityPath = path.join(repoRoot, "rust/census-instrument-s0.json");

function lineNumber(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function productionRustSource(file: string, source: string): string | undefined {
  if (file.endsWith("/tests.rs") || file.includes("/tests/")) return undefined;
  const output = source.split("");
  const code = maskRustCommentsAndLiterals(source);
  for (const match of code.matchAll(
    /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z0-9_]+\s*\{/gu,
  )) {
    const start = match.index ?? 0;
    const open = code.indexOf("{", start);
    const close = matchingRustDelimiter(code, open, "{", "}");
    for (let index = start; index <= close; index += 1) {
      if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
    }
  }
  return output.join("");
}

function walk(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolute = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(absolute) : [absolute];
  });
}

function readMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--all-features", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as CargoMetadata;
}

function precisionScope(metadata: CargoMetadata): CargoPackage[] {
  const owner = metadata.packages.find(({ name }) => name === "omena-evidence-graph");
  assert.ok(owner, "sealed family not resolvable omena-evidence-graph");
  const reverse = new Map<string, Set<string>>();
  for (const node of metadata.resolve.nodes) {
    for (const dependency of node.deps) {
      if (!dependency.dep_kinds.some(({ kind }) => kind === null || kind === "build")) continue;
      const dependents = reverse.get(dependency.pkg) ?? new Set<string>();
      dependents.add(node.id);
      reverse.set(dependency.pkg, dependents);
    }
  }
  const reached = new Set([owner.id]);
  const queue = [owner.id];
  while (queue.length > 0) {
    for (const dependent of reverse.get(queue.shift()!) ?? []) {
      if (reached.has(dependent)) continue;
      reached.add(dependent);
      queue.push(dependent);
    }
  }
  return metadata.packages
    .filter(({ id }) => reached.has(id))
    .toSorted((left, right) => compareCodePoint(left.name, right.name));
}

function precisionSources(packages: readonly CargoPackage[]): RustSource[] {
  const moduleCensus = deriveRustModuleContexts(repoRoot, packages);
  const runnerBase = parseRustcCfg(execFileSync("rustc", ["--print", "cfg"], { encoding: "utf8" }));
  const packagesByName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const sources: RustSource[] = [];
  const seen = new Set<string>();
  for (const context of moduleCensus.contexts) {
    const pkg = packagesByName.get(context.crate)!;
    const file = path.relative(repoRoot, context.file).split(path.sep).join("/");
    const identity = `${file}|${context.modulePath}|${context.inheritedPredicates.map(renderCfg).join("&&")}`;
    if (seen.has(identity)) continue;
    seen.add(identity);
    const raw = readFileSync(context.file, "utf8");
    const source = productionRustSource(file, raw);
    if (source === undefined) continue;
    sources.push({
      crate: pkg.name,
      file,
      modulePath: context.modulePath,
      targetName: context.targetName,
      targetRoot: context.targetRoot,
      targetKind: context.targetKind,
      package: pkg,
      inheritedPredicates: context.inheritedPredicates,
      runnerCfg: withTest(withPackageCfg(runnerBase, pkg, []), false),
      source,
      code: maskRustCommentsAndLiterals(source),
    });
  }
  return sources.toSorted((left, right) => compareCodePoint(left.file, right.file));
}

function rustSymbolKey(
  crateName: string,
  targetRoot: string,
  modulePath: string,
  name: string,
): string {
  return `${crateName}|${targetRoot}|${modulePath}|${name}`;
}

function rustPublicIdentity(entry: RustSource, modulePath: string, name: string): string {
  const target = entry.targetKind.includes("bin") ? `bin-${entry.targetName}::` : "";
  return `${entry.crate}::${target}${modulePath}::${name}`;
}

interface PrecisionSourceAnalysis {
  readonly innerPredicates: readonly CfgExpr[];
  readonly attributeRegions: ReturnType<typeof attributeRegions>;
  readonly inlineModules: readonly {
    readonly start: number;
    readonly end: number;
    readonly modulePath: string;
  }[];
}

const sourceAnalysisCache = new WeakMap<RustSource, PrecisionSourceAnalysis>();

function sourceAnalysis(entry: RustSource): PrecisionSourceAnalysis {
  const cached = sourceAnalysisCache.get(entry);
  if (cached) return cached;
  const inlineModules: Array<{ start: number; end: number; modulePath: string }> = [];
  const visit = (start: number, end: number, modulePath: string): void => {
    for (const declaration of moduleDeclarations(entry.source, start, end)) {
      if (!declaration.body) continue;
      const childModulePath = `${modulePath}::${declaration.name}`;
      inlineModules.push({
        start: declaration.body.start,
        end: declaration.body.end,
        modulePath: childModulePath,
      });
      visit(declaration.body.start, declaration.body.end, childModulePath);
    }
  };
  visit(0, entry.source.length, entry.modulePath);
  const analysis = {
    innerPredicates: attributes(entry.source)
      .filter(({ inner }) => inner)
      .flatMap(cfgPredicates),
    attributeRegions: attributeRegions(entry.source),
    inlineModules,
  };
  sourceAnalysisCache.set(entry, analysis);
  return analysis;
}

function modulePathAt(entry: RustSource, offset: number): string {
  return (
    sourceAnalysis(entry).inlineModules.findLast(
      ({ start, end }) => start <= offset && offset < end,
    )?.modulePath ?? entry.modulePath
  );
}

function cfgAtoms(expression: CfgExpr): Array<{ name: string; value?: string }> {
  switch (expression.kind) {
    case "atom":
      return [{ name: expression.name }];
    case "value":
      return [{ name: expression.name, value: expression.value }];
    case "not":
      return cfgAtoms(expression.operand);
    case "all":
    case "any":
      return expression.operands.flatMap(cfgAtoms);
  }
}

function assertCfgDecidable(entry: RustSource, expression: CfgExpr, offset: number): void {
  const standardAtoms = new Set([
    "debug_assertions",
    "doc",
    "doctest",
    "miri",
    "panic",
    "proc_macro",
    "test",
    "unix",
    "windows",
  ]);
  for (const atom of cfgAtoms(expression)) {
    const known =
      atom.name === "feature" ||
      atom.name.startsWith("target_") ||
      standardAtoms.has(atom.name) ||
      entry.runnerCfg.atoms.has(atom.name) ||
      entry.runnerCfg.values.has(atom.name);
    assert.ok(known, `cfg predicate undecidable ${entry.file}:${lineNumber(entry.source, offset)}`);
  }
}

function nodeIsActive(entry: RustSource, offset: number): boolean {
  const predicates = [
    ...entry.inheritedPredicates,
    ...sourceAnalysis(entry).innerPredicates,
    ...sourceAnalysis(entry)
      .attributeRegions.filter(({ start, end }) => start <= offset && offset < end)
      .flatMap(({ predicates: rows }) => rows),
  ];
  const effective = combinePredicates(predicates);
  assertCfgDecidable(entry, effective, offset);
  const active = evaluateCfg(effective, entry.runnerCfg);
  if (
    process.argv.includes("--debug-fields") &&
    entry.source.slice(offset, offset + 80).includes("failpoint")
  ) {
    process.stderr.write(
      `field-cfg ${entry.file}:${lineNumber(entry.source, offset)} ${JSON.stringify(entry.source.slice(offset, offset + 40))} ${renderCfg(effective)} ${active}\n`,
    );
  }
  return active;
}

function precedingAttributes(source: string, declarationStart: number): string {
  let cursor = declarationStart;
  while (cursor > 0) {
    const prefix = source.slice(0, cursor);
    const match = prefix.match(/#\s*\[[^\]]*\]\s*$/u);
    if (!match?.index) break;
    cursor = match.index;
  }
  return source.slice(cursor, declarationStart);
}

function genericNames(header: string): ReadonlySet<string> {
  const open = header.indexOf("<");
  if (open < 0) return new Set();
  const close = matchingRustDelimiter(header, open, "<", ">");
  return new Set(
    header
      .slice(open + 1, close)
      .split(",")
      .map((part) => part.trim().match(/^(?:const\s+)?([A-Z][A-Za-z0-9_]*)\b/u)?.[1])
      .filter((name): name is string => name !== undefined),
  );
}

function stripLeadingRustAttributes(source: string): string {
  let cursor = 0;
  const code = maskRustCommentsAndLiterals(source);
  while (true) {
    while (/\s/u.test(code[cursor] ?? "")) cursor += 1;
    if (code[cursor] !== "#") return source.slice(cursor).trim();
    let bracket = cursor + 1;
    if (code[bracket] === "!") bracket += 1;
    if (code[bracket] !== "[") return source.slice(cursor).trim();
    cursor = matchingRustDelimiter(code, bracket, "[", "]") + 1;
  }
}

function topLevelColon(source: string): number {
  let round = 0;
  let square = 0;
  let curly = 0;
  let angle = 0;
  for (let index = 0; index < source.length; index += 1) {
    const token = source[index]!;
    if (token === "(") round += 1;
    else if (token === ")") round -= 1;
    else if (token === "[") square += 1;
    else if (token === "]") square -= 1;
    else if (token === "{") curly += 1;
    else if (token === "}") curly -= 1;
    else if (token === "<") angle += 1;
    else if (token === ">") angle = Math.max(0, angle - 1);
    else if (
      token === ":" &&
      source[index + 1] !== ":" &&
      source[index - 1] !== ":" &&
      round === 0 &&
      square === 0 &&
      curly === 0 &&
      angle === 0
    )
      return index;
  }
  return -1;
}

function namedFieldTypes(
  body: string,
  baseOffset = 0,
): Array<{ field: string; type: string; offset: number }> {
  return splitTopLevelRows(body).flatMap(({ text: raw, start }) => {
    const field = stripLeadingRustAttributes(raw).replace(/^pub(?:\([^)]*\))?\s+/u, "");
    const colon = topLevelColon(field);
    if (colon < 0) return [];
    const fieldName = field.slice(0, colon).trim();
    return [
      {
        field: fieldName,
        type: field.slice(colon + 1).trim(),
        offset: baseOffset + start + Math.max(0, raw.indexOf(fieldName)),
      },
    ];
  });
}

function enumFieldTypes(
  body: string,
  baseOffset = 0,
): Array<{ field: string; type: string; offset: number }> {
  const fields: Array<{ field: string; type: string; offset: number }> = [];
  for (const { text: rawVariant, start: variantStart } of splitTopLevelRows(body)) {
    const variant = stripLeadingRustAttributes(rawVariant);
    const name = variant.match(/^([A-Z][A-Za-z0-9_]*)/u)?.[1];
    if (!name) continue;
    const nameOffset = rawVariant.indexOf(name);
    const rest = variant.slice(name.length).trim();
    if (rest.startsWith("(")) {
      const close = matchingRustDelimiter(maskRustCommentsAndLiterals(rest), 0, "(", ")");
      for (const [index, row] of splitTopLevelRows(rest.slice(1, close)).entries()) {
        fields.push({
          field: `${name}.${index}`,
          type: stripLeadingRustAttributes(row.text),
          offset:
            baseOffset +
            variantStart +
            rawVariant.indexOf("(", nameOffset + name.length) +
            1 +
            row.start,
        });
      }
    } else if (rest.startsWith("{")) {
      const close = matchingRustDelimiter(maskRustCommentsAndLiterals(rest), 0, "{", "}");
      for (const field of namedFieldTypes(
        rest.slice(1, close),
        baseOffset + variantStart + rawVariant.indexOf("{", nameOffset + name.length) + 1,
      )) {
        fields.push({ field: `${name}.${field.field}`, type: field.type, offset: field.offset });
      }
    }
  }
  return fields;
}

function parseTypes(sources: readonly RustSource[]): {
  definitions: TypeDefinition[];
  aliases: AliasDefinition[];
} {
  const definitions: TypeDefinition[] = [];
  const aliases: AliasDefinition[] = [];
  const declaration =
    /^[\t ]*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(struct|enum|type)\s+([A-Z][A-Za-z0-9_]*)/gmu;
  for (const entry of sources) {
    for (const match of entry.code.matchAll(declaration)) {
      const start = match.index ?? 0;
      if (!nodeIsActive(entry, start)) continue;
      const kind = match[1]!;
      const name = match[2]!;
      const modulePath = modulePathAt(entry, start);
      const statementStart = start + match[0].length;
      let cursor = statementStart;
      let round = 0;
      let square = 0;
      let angle = 0;
      while (cursor < entry.code.length) {
        const token = entry.code[cursor]!;
        if (token === "(") round += 1;
        else if (token === ")") round -= 1;
        else if (token === "[") square += 1;
        else if (token === "]") square -= 1;
        else if (token === "<") angle += 1;
        else if (token === ">") angle = Math.max(0, angle - 1);
        else if ((token === "{" || token === ";") && round === 0 && square === 0 && angle === 0)
          break;
        cursor += 1;
      }
      const identity = rustPublicIdentity(entry, modulePath, name);
      const symbolKey = rustSymbolKey(entry.crate, entry.targetRoot, modulePath, name);
      const header = entry.source.slice(start, cursor);
      if (kind === "type") {
        const equals = header.indexOf("=");
        if (equals < 0) continue;
        aliases.push({
          identity,
          symbolKey,
          crate: entry.crate,
          targetRoot: entry.targetRoot,
          targetKind: entry.targetKind,
          modulePath,
          name,
          target: header.slice(equals + 1).trim(),
        });
        continue;
      }
      let body = "";
      let bodyOffset = cursor;
      let tupleBody = false;
      if (entry.code[cursor] === "{") {
        const close = matchingRustDelimiter(entry.code, cursor, "{", "}");
        body = entry.source.slice(cursor + 1, close);
        bodyOffset = cursor + 1;
      } else {
        const tupleOpen = entry.code.indexOf("(", statementStart);
        if (tupleOpen >= 0 && tupleOpen < cursor) {
          const tupleClose = matchingRustDelimiter(entry.code, tupleOpen, "(", ")");
          body = entry.source.slice(tupleOpen + 1, tupleClose);
          bodyOffset = tupleOpen + 1;
          tupleBody = true;
        }
      }
      const attributes = precedingAttributes(entry.source, start);
      definitions.push({
        identity,
        symbolKey,
        crate: entry.crate,
        targetRoot: entry.targetRoot,
        targetKind: entry.targetKind,
        modulePath,
        name,
        file: entry.file,
        body,
        fieldTypes: (kind === "enum"
          ? enumFieldTypes(body, bodyOffset)
          : tupleBody
            ? splitTopLevelRows(body).map(({ text: type, start }, index) => ({
                field: String(index),
                type: stripLeadingRustAttributes(type),
                offset:
                  bodyOffset + start + Math.max(0, type.indexOf(stripLeadingRustAttributes(type))),
              }))
            : namedFieldTypes(body, bodyOffset)
        ).filter(({ offset }) => nodeIsActive(entry, offset)),
        derivesDeserialize: /\bderive\s*\([^)]*\bDeserialize\b[^)]*\)/su.test(attributes),
        genericParameters: genericNames(header),
      });
    }
  }
  return { definitions, aliases };
}

function typeReferences(source: string): string[] {
  return [
    ...maskRustCommentsAndLiterals(source).matchAll(
      /\b(?:(?:crate|self|super|[a-z_][a-z0-9_]*)::)*[A-Z][A-Za-z0-9_]*\b/gu,
    ),
  ]
    .map(([reference]) => reference)
    .filter((reference) => !/^[A-Z0-9_]+$/u.test(reference.split("::").at(-1)!));
}

function splitTopLevelRows(input: string): Array<{ text: string; start: number }> {
  const parts: Array<{ text: string; start: number }> = [];
  const push = (from: number, to: number): void => {
    const raw = input.slice(from, to);
    const leading = raw.search(/\S/u);
    if (leading < 0) return;
    parts.push({ text: raw.trim(), start: from + leading });
  };
  let start = 0;
  let round = 0;
  let square = 0;
  let curly = 0;
  let angle = 0;
  for (let index = 0; index < input.length; index += 1) {
    const token = input[index]!;
    if (token === "(") round += 1;
    else if (token === ")") round -= 1;
    else if (token === "[") square += 1;
    else if (token === "]") square -= 1;
    else if (token === "{") curly += 1;
    else if (token === "}") curly -= 1;
    else if (token === "<") angle += 1;
    else if (token === ">") angle = Math.max(0, angle - 1);
    else if (token === "," && round === 0 && square === 0 && curly === 0 && angle === 0) {
      push(start, index);
      start = index + 1;
    }
  }
  push(start, input.length);
  return parts;
}

function splitTopLevel(input: string): string[] {
  return splitTopLevelRows(input).map(({ text }) => text);
}

interface ExpandedUse {
  readonly name: string;
  readonly target: string;
}

function expandUse(specification: string, prefix = ""): ExpandedUse[] {
  const spec = specification.trim();
  let brace = -1;
  let angle = 0;
  for (let index = 0; index < spec.length; index += 1) {
    if (spec[index] === "<") angle += 1;
    else if (spec[index] === ">") angle = Math.max(0, angle - 1);
    else if (spec[index] === "{" && angle === 0) {
      brace = index;
      break;
    }
  }
  if (brace >= 0) {
    const close = matchingRustDelimiter(spec, brace, "{", "}");
    assert.equal(spec.slice(close + 1).trim(), "", `use path unresolved ${spec}`);
    const base = spec
      .slice(0, brace)
      .replace(/::\s*$/u, "")
      .trim();
    const joined = [prefix, base].filter(Boolean).join("::");
    return splitTopLevel(spec.slice(brace + 1, close)).flatMap((part) => expandUse(part, joined));
  }
  if (spec === "*") return prefix ? [{ name: "*", target: prefix }] : [];
  const renamed = spec.match(/^(.*?)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/su);
  const rawTarget = (renamed?.[1] ?? spec).trim();
  const target = rawTarget === "self" ? prefix : [prefix, rawTarget].filter(Boolean).join("::");
  if (target.endsWith("::*")) return [{ name: "*", target: target.slice(0, -3) }];
  const name = renamed?.[2] ?? target.split("::").at(-1)!;
  return [{ name, target }];
}

function moduleKey(crateName: string, targetRoot: string, modulePath: string): string {
  return `${crateName}|${targetRoot}|${modulePath}`;
}

function deriveDeserializationContainers(sources: readonly RustSource[]): string[] {
  const { definitions, aliases } = parseTypes(sources);
  const symbols = [...definitions, ...aliases];
  const symbolsByKey = new Map(symbols.map((symbol) => [symbol.symbolKey, symbol]));
  const packageByRustName = new Map(
    [...new Set(symbols.map(({ crate: crateName }) => crateName))].map((crateName) => [
      crateName.replaceAll("-", "_"),
      crateName,
    ]),
  );
  const knownModules = new Set<string>();
  for (const entry of sources) {
    knownModules.add(moduleKey(entry.crate, entry.targetRoot, entry.modulePath));
    for (const inline of sourceAnalysis(entry).inlineModules) {
      knownModules.add(moduleKey(entry.crate, entry.targetRoot, inline.modulePath));
    }
  }
  const imports = new Map<string, Map<string, string>>();
  const globImports = new Map<string, string[]>();
  for (const entry of sources) {
    for (const match of entry.code.matchAll(/^[\t ]*(?:pub(?:\([^)]*\))?\s+)?use\s+([^;]+);/gmu)) {
      const offset = match.index ?? 0;
      if (!nodeIsActive(entry, offset)) continue;
      const modulePath = modulePathAt(entry, offset);
      const key = moduleKey(entry.crate, entry.targetRoot, modulePath);
      const table = imports.get(key) ?? new Map<string, string>();
      const globs = globImports.get(key) ?? [];
      for (const expanded of expandUse(match[1]!)) {
        if (expanded.name === "*") globs.push(expanded.target);
        else {
          const previous = table.get(expanded.name);
          assert.ok(
            previous === undefined || previous === expanded.target,
            `deserialization field type unresolved ${entry.crate}::${modulePath}::${expanded.name}`,
          );
          table.set(expanded.name, expanded.target);
        }
      }
      imports.set(key, table);
      globImports.set(key, globs);
    }
  }

  const moduleParts = (modulePath: string): string[] =>
    modulePath === "crate" ? [] : modulePath.replace(/^crate::/u, "").split("::");
  const modulePathFor = (modules: readonly string[]): string =>
    modules.length === 0 ? "crate" : `crate::${modules.join("::")}`;
  const libraryRootByCrate = new Map(
    sources
      .filter(({ targetKind }) => !targetKind.includes("bin"))
      .map(({ crate: crateName, targetRoot }) => [crateName, targetRoot]),
  );

  const resolve = (
    reference: string,
    context: { readonly crate: string; readonly targetRoot: string; readonly modulePath: string },
    seen = new Set<string>(),
  ): string | undefined => {
    const visit = `${context.crate}|${context.modulePath}|${reference}`;
    if (seen.has(visit)) return undefined;
    seen.add(visit);
    const segments = reference.split("::");
    const name = segments.at(-1)!;
    if (segments.length === 1) {
      const imported = imports
        .get(moduleKey(context.crate, context.targetRoot, context.modulePath))
        ?.get(name);
      if (imported) return resolve(imported, context, seen);
      const local = rustSymbolKey(context.crate, context.targetRoot, context.modulePath, name);
      if (symbolsByKey.has(local)) return local;
      const candidates = new Set<string>();
      for (const glob of globImports.get(
        moduleKey(context.crate, context.targetRoot, context.modulePath),
      ) ?? []) {
        const viaGlob = resolve(`${glob}::${name}`, context, new Set(seen));
        if (viaGlob) candidates.add(viaGlob);
      }
      assert.ok(
        candidates.size <= 1,
        `deserialization field type unresolved ${visit} [${[...candidates].join(", ")}]`,
      );
      return [...candidates][0];
    }

    let targetCrate = context.crate;
    let targetRoot = context.targetRoot;
    let targetModules: string[];
    const first = segments[0]!;
    const middle = segments.slice(1, -1);
    if (first === "crate") targetModules = middle;
    else if (first === "self") targetModules = [...moduleParts(context.modulePath), ...middle];
    else if (first === "super") {
      targetModules = moduleParts(context.modulePath);
      let cursor = 0;
      while (segments[cursor] === "super") {
        targetModules.pop();
        cursor += 1;
      }
      targetModules.push(...segments.slice(cursor, -1));
    } else if (packageByRustName.has(first)) {
      targetCrate = packageByRustName.get(first)!;
      targetRoot = libraryRootByCrate.get(targetCrate) ?? "";
      assert.ok(targetRoot, `deserialization field type unresolved ${visit}`);
      targetModules = middle;
    } else {
      targetModules = [...moduleParts(context.modulePath), ...segments.slice(0, -1)];
    }
    const targetModulePath = modulePathFor(targetModules);
    const direct = rustSymbolKey(targetCrate, targetRoot, targetModulePath, name);
    if (symbolsByKey.has(direct)) return direct;
    const reexport = imports.get(moduleKey(targetCrate, targetRoot, targetModulePath))?.get(name);
    if (reexport) {
      return resolve(
        reexport,
        { crate: targetCrate, targetRoot, modulePath: targetModulePath },
        seen,
      );
    }
    const candidates = new Set<string>();
    for (const glob of globImports.get(moduleKey(targetCrate, targetRoot, targetModulePath)) ??
      []) {
      const viaGlob = resolve(
        `${glob}::${name}`,
        { crate: targetCrate, targetRoot, modulePath: targetModulePath },
        new Set(seen),
      );
      if (viaGlob) candidates.add(viaGlob);
    }
    assert.ok(
      candidates.size <= 1,
      `deserialization field type unresolved ${visit} [${[...candidates].join(", ")}]`,
    );
    const candidate = [...candidates][0];
    if (candidate) return candidate;
    const externalPath =
      !packageByRustName.has(first) &&
      !["crate", "self", "super"].includes(first) &&
      !knownModules.has(moduleKey(targetCrate, targetRoot, targetModulePath));
    return externalPath ? `external|${reference}` : undefined;
  };

  const sealed = symbols.find(
    ({ identity }) =>
      identity === "omena-evidence-graph::crate::analysis_precision::AnalysisPrecisionV1",
  );
  assert.ok(sealed, "sealed family not resolvable omena-evidence-graph");
  const reached = new Set([sealed.symbolKey]);
  const standardContainers = new Set([
    "Arc",
    "ArrayVec",
    "BTreeMap",
    "BTreeSet",
    "Box",
    "Cell",
    "Cow",
    "Duration",
    "Fn",
    "FnMut",
    "FnOnce",
    "HashMap",
    "HashSet",
    "IndexMap",
    "Instant",
    "Map",
    "Mutex",
    "NonZeroU16",
    "NonZeroU32",
    "NonZeroU64",
    "NonZeroU8",
    "Option",
    "OsString",
    "Path",
    "PathBuf",
    "PhantomData",
    "Range",
    "RangeInclusive",
    "Rc",
    "RefCell",
    "Result",
    "RwLock",
    "Send",
    "SmallVec",
    "SocketAddr",
    "String",
    "SystemTime",
    "Sync",
    "Url",
    "Value",
    "Vec",
    "VecDeque",
  ]);
  const isOpaqueExternal = (
    reference: string,
    context: { readonly crate: string; readonly targetRoot: string; readonly modulePath: string },
  ): boolean => {
    const segments = reference.split("::");
    const name = segments.at(-1)!;
    if (name === "Self" || standardContainers.has(name)) return true;
    if (segments.length > 1) {
      const first = segments[0]!;
      if (["std", "core", "alloc"].includes(first)) return true;
      if (!packageByRustName.has(first) && !["crate", "self", "super"].includes(first)) {
        return true;
      }
    }
    const imported = imports
      .get(moduleKey(context.crate, context.targetRoot, context.modulePath))
      ?.get(name);
    if (imported) {
      const first = imported.split("::")[0]!;
      return !packageByRustName.has(first) && !["crate", "self", "super"].includes(first);
    }
    return false;
  };
  let changed = true;
  while (changed) {
    changed = false;
    for (const symbol of symbols) {
      if (reached.has(symbol.symbolKey)) continue;
      let containsReached = false;
      const fields =
        "fieldTypes" in symbol
          ? symbol.fieldTypes
          : [{ field: "alias", type: symbol.target, offset: 0 }];
      for (const field of fields) {
        for (const reference of typeReferences(field.type)) {
          const first = reference.split("::")[0]!;
          if ("genericParameters" in symbol && symbol.genericParameters.has(first)) continue;
          const resolved = resolve(reference, symbol);
          if (resolved && reached.has(resolved)) {
            containsReached = true;
            break;
          }
          if (!resolved && process.argv.includes("--debug-resolution")) {
            process.stderr.write(
              `unresolved ${symbol.identity}::${field.field} ${reference} import=${
                imports
                  .get(moduleKey(symbol.crate, symbol.targetRoot, symbol.modulePath))
                  ?.get(reference.split("::").at(-1)!) ?? "<none>"
              }\n`,
            );
          }
          assert.ok(
            resolved || isOpaqueExternal(reference, symbol),
            `deserialization field type unresolved ${symbol.identity}::${field.field}`,
          );
        }
        if (containsReached) break;
      }
      if (!containsReached) continue;
      reached.add(symbol.symbolKey);
      changed = true;
      if (process.argv.includes("--debug-containers")) {
        process.stderr.write(`reach ${symbol.identity}\n`);
      }
    }
  }

  return definitions
    .filter(({ symbolKey, derivesDeserialize }) => reached.has(symbolKey) && derivesDeserialize)
    .map(({ identity }) => identity)
    .toSorted(compareCodePoint);
}

function callIdentity(site: Omit<RegisteredCallSite, "owner">): string {
  return [site.crate, site.file, site.item, site.member, String(site.ordinal)].join("|");
}

function familyMembers(leaf: RustSource): Set<string> {
  const implementation = leaf.code.match(/\bimpl\s+AnalysisPrecisionV1\s*\{/u);
  assert.ok(
    implementation?.index !== undefined,
    "sealed family not resolvable omena-evidence-graph",
  );
  const open = leaf.code.indexOf("{", implementation.index);
  const close = matchingRustDelimiter(leaf.code, open, "{", "}");
  const body = leaf.code.slice(open + 1, close);
  const publicFunctions = new Set(
    [...body.matchAll(/\bpub\s+(?:const\s+)?fn\s+([a-z][a-z0-9_]*)\s*\(/gu)].map(
      (match) => match[1]!,
    ),
  );
  const expected = [
    "unknown",
    "from_axes",
    "from_axes_for_tests",
    "with_value_domain",
    "with_flow",
    "with_context",
    "with_provider_completeness",
    "with_world_assumption",
    "with_revision",
  ];
  for (const member of expected) {
    assert.ok(publicFunctions.has(member), `sealed family member missing ${member}`);
  }
  const constructionMembers = new Set([
    "from_axes",
    "with_value_domain",
    "with_flow",
    "with_context",
    "with_provider_completeness",
    "with_world_assumption",
    "with_revision",
  ]);
  const functions = rustNamedFunctions(leaf.source, leaf.modulePath);
  for (const match of leaf.code.matchAll(/\b(?:Self|AnalysisPrecisionV1)\s*\{/gu)) {
    const offset = match.index ?? 0;
    if (/(?:\b(?:struct|impl)|->)\s*$/u.test(leaf.code.slice(Math.max(0, offset - 32), offset)))
      continue;
    const owner = functions.findLast(({ bodyStart, end }) => bodyStart < offset && offset < end);
    assert.ok(
      owner && constructionMembers.has(owner.shortName),
      `unregistered sealed-family member ${leaf.crate}::${owner?.name ?? `line-${lineNumber(leaf.source, offset)}`}`,
    );
  }
  return new Set(expected);
}

function outerTypeName(typeSource: string): string | undefined {
  const normalized = typeSource
    .trim()
    .replace(/^&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?/u, "")
    .replace(/^\(\s*/u, "");
  return normalized.match(
    /^(?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*([A-Z][A-Za-z0-9_]*)\b/u,
  )?.[1];
}

function directPrecisionType(typeSource: string, aliases: ReadonlySet<string>): boolean {
  const name = outerTypeName(typeSource);
  return name !== undefined && aliases.has(name);
}

type PrecisionAttribution = "precision" | "non-precision" | "unresolved";

function functionVariableTypes(
  header: string,
  body: string,
  functionName: string,
  aliases: ReadonlySet<string>,
  members: ReadonlySet<string>,
  returnTypes: ReadonlyMap<string, string>,
): Map<string, string> {
  const types = new Map<string, string>();
  const parameterBinding =
    /\b([a-z][a-z0-9_]*)\s*:\s*((?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?)?(?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*[A-Z][A-Za-z0-9_]*)/gu;
  for (const match of header.matchAll(parameterBinding)) types.set(match[1]!, match[2]!);
  const localBinding =
    /\blet\s+(?:mut\s+)?([a-z][a-z0-9_]*)\s*:\s*((?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?)?(?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*[A-Z][A-Za-z0-9_]*)/gu;
  for (const match of body.matchAll(localBinding)) types.set(match[1]!, match[2]!);

  if (/\bself\b/u.test(header)) {
    const owner = functionName.split("::").at(-2);
    if (owner) types.set("self", owner);
  }
  const assignedConstructor =
    /\blet\s+(?:mut\s+)?([a-z][a-z0-9_]*)\s*=\s*((?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*[A-Z][A-Za-z0-9_]*)\s*\{/gu;
  for (const match of body.matchAll(assignedConstructor)) types.set(match[1]!, match[2]!);

  const aliasAlternation = [...aliases].join("|");
  const memberAlternation = [...members].join("|");
  const assignedPrecision = new RegExp(
    `\\blet\\s+(?:mut\\s+)?([a-z][a-z0-9_]*)\\s*=\\s*(?:${aliasAlternation})\\s*::\\s*(?:${memberAlternation})\\s*\\(`,
    "gu",
  );
  for (const match of body.matchAll(assignedPrecision)) {
    types.set(match[1]!, "AnalysisPrecisionV1");
  }
  const assignedCall =
    /\blet\s+(?:mut\s+)?([a-z][a-z0-9_]*)\s*=\s*(?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*([a-z][a-z0-9_]*)\s*\(/gu;
  for (const match of body.matchAll(assignedCall)) {
    const returnType = returnTypes.get(match[2]!);
    if (returnType) types.set(match[1]!, returnType);
  }
  return types;
}

function deriveFunctionReturnTypes(sources: readonly RustSource[]): ReadonlyMap<string, string> {
  const candidates = new Map<string, Set<string>>();
  for (const entry of sources) {
    const functions = rustNamedFunctions(entry.source, entry.modulePath).filter(({ start }) =>
      nodeIsActive(entry, start),
    );
    for (const fn of functions) {
      const header = entry.code.slice(fn.start, fn.bodyStart);
      const returnType = header.match(
        /->\s*((?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?)?(?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*[A-Z][A-Za-z0-9_]*)/u,
      )?.[1];
      if (!returnType) continue;
      const rows = candidates.get(fn.shortName) ?? new Set<string>();
      rows.add(returnType);
      candidates.set(fn.shortName, rows);
    }
  }
  return new Map(
    [...candidates].flatMap(([name, rows]) =>
      rows.size === 1 ? ([[name, [...rows][0]!]] as const) : [],
    ),
  );
}

function definitionForType(
  typeSource: string,
  entry: RustSource,
  modulePath: string,
  definitions: readonly TypeDefinition[],
): TypeDefinition | undefined {
  const name = outerTypeName(typeSource);
  if (!name) return undefined;
  const candidates = definitions.filter(
    (definition) => definition.crate === entry.crate && definition.name === name,
  );
  const local =
    candidates.find((definition) => definition.modulePath === modulePath) ??
    (candidates.length === 1 ? candidates[0] : undefined);
  if (local) return local;
  const global = definitions.filter((definition) => definition.name === name);
  return global.length === 1 ? global[0] : undefined;
}

function pathReceiverIsPrecision(
  receiverPath: string,
  entry: RustSource,
  modulePath: string,
  variables: ReadonlyMap<string, string>,
  aliases: ReadonlySet<string>,
  definitions: readonly TypeDefinition[],
  genericParameters: ReadonlySet<string>,
): PrecisionAttribution {
  const segments = receiverPath.split(/\s*\.\s*/u);
  let typeSource = variables.get(segments.shift()!);
  if (!typeSource) return "unresolved";
  for (const fieldName of segments) {
    const definition = definitionForType(typeSource, entry, modulePath, definitions);
    const field = definition?.fieldTypes.find(({ field }) => field === fieldName);
    if (!field) return "unresolved";
    typeSource = field.type;
  }
  if (directPrecisionType(typeSource, aliases)) return "precision";
  const name = outerTypeName(typeSource);
  if (!name || genericParameters.has(name)) return "unresolved";
  return definitionForType(typeSource, entry, modulePath, definitions)
    ? "non-precision"
    : "unresolved";
}

function matchingOpenParen(source: string, close: number): number {
  let depth = 0;
  for (let index = close; index >= 0; index -= 1) {
    if (source[index] === ")") depth += 1;
    else if (source[index] === "(") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function previousCodeOffset(source: string, from: number): number {
  let cursor = from;
  while (cursor >= 0 && /\s/u.test(source[cursor]!)) cursor -= 1;
  return cursor;
}

function receiverPrecisionAttribution(
  body: string,
  dotOffset: number,
  entry: RustSource,
  modulePath: string,
  variables: ReadonlyMap<string, string>,
  aliases: ReadonlySet<string>,
  definitions: readonly TypeDefinition[],
  members: ReadonlySet<string>,
  returnTypes: ReadonlyMap<string, string>,
  genericParameters: ReadonlySet<string>,
  seen = new Set<number>(),
): PrecisionAttribution {
  if (seen.has(dotOffset)) return "unresolved";
  seen.add(dotOffset);
  const receiverEnd = previousCodeOffset(body, dotOffset - 1);
  if (receiverEnd < 0) return "unresolved";
  if (body[receiverEnd] !== ")") {
    const suffix = body.slice(Math.max(0, receiverEnd - 512), receiverEnd + 1);
    const pathMatch = suffix.match(
      /([A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)*)\s*$/u,
    );
    return pathMatch
      ? pathReceiverIsPrecision(
          pathMatch[1]!,
          entry,
          modulePath,
          variables,
          aliases,
          definitions,
          genericParameters,
        )
      : "unresolved";
  }

  const open = matchingOpenParen(body, receiverEnd);
  if (open < 0) return "unresolved";
  const nameEnd = previousCodeOffset(body, open - 1);
  let nameStart = nameEnd;
  while (nameStart >= 0 && /[A-Za-z0-9_]/u.test(body[nameStart]!)) nameStart -= 1;
  nameStart += 1;
  const calledName = body.slice(nameStart, nameEnd + 1);
  if (!calledName) {
    const inner = body.slice(open + 1, receiverEnd);
    const pathMatch = inner.match(
      /([A-Za-z_][A-Za-z0-9_]*(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)*)\s*$/u,
    );
    return pathMatch
      ? pathReceiverIsPrecision(
          pathMatch[1]!,
          entry,
          modulePath,
          variables,
          aliases,
          definitions,
          genericParameters,
        )
      : "unresolved";
  }

  const separator = previousCodeOffset(body, nameStart - 1);
  if (separator >= 0 && body[separator] === ".") {
    return members.has(calledName)
      ? receiverPrecisionAttribution(
          body,
          separator,
          entry,
          modulePath,
          variables,
          aliases,
          definitions,
          members,
          returnTypes,
          genericParameters,
          seen,
        )
      : "unresolved";
  }
  const returnType = returnTypes.get(calledName);
  if (returnType) {
    if (directPrecisionType(returnType, aliases)) return "precision";
    const name = outerTypeName(returnType);
    if (name && !genericParameters.has(name)) {
      const definition = definitionForType(returnType, entry, modulePath, definitions);
      if (definition) return "non-precision";
    }
  }
  if (separator >= 1 && body.slice(separator - 1, separator + 1) === "::") {
    const typeEnd = previousCodeOffset(body, separator - 2);
    const prefix = body.slice(Math.max(0, typeEnd - 256), typeEnd + 1);
    const typePath = prefix.match(
      /((?:(?:crate|self|super|[a-z_][a-z0-9_]*)\s*::\s*)*[A-Z][A-Za-z0-9_]*)\s*$/u,
    )?.[1];
    if (typePath && directPrecisionType(typePath, aliases) && members.has(calledName)) {
      return "precision";
    }
  }
  return "unresolved";
}

function deriveFamilyCalls(sources: readonly RustSource[]): RegisteredCallSite[] {
  const leaf = sources.find(({ file }) =>
    file.endsWith("/omena-evidence-graph/src/analysis_precision.rs"),
  );
  assert.ok(leaf, "sealed family not resolvable omena-evidence-graph");
  assert.ok(
    !/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+[a-z][a-z0-9_]*\s*[;{]/mu.test(leaf.code),
    "leaf module has child module",
  );
  const members = familyMembers(leaf);
  const { definitions } = parseTypes(sources);
  const returnTypes = deriveFunctionReturnTypes(sources);
  const rows: RegisteredCallSite[] = [];
  for (const entry of sources) {
    if (entry.file === leaf.file) continue;
    const typeAliases = new Set(["AnalysisPrecisionV1"]);
    for (const match of entry.code.matchAll(
      /\bAnalysisPrecisionV1\s+as\s+([A-Z][A-Za-z0-9_]*)/gu,
    )) {
      typeAliases.add(match[1]!);
    }
    for (const match of entry.code.matchAll(
      /\btype\s+([A-Z][A-Za-z0-9_]*)\s*=\s*(?:[A-Za-z0-9_]+::)*AnalysisPrecisionV1\s*;/gu,
    )) {
      typeAliases.add(match[1]!);
    }
    const staticFamilyReference = new RegExp(
      `\\b(?:${[...typeAliases].join("|")})\\s*::\\s*(${[...members].join("|")})\\b`,
      "gu",
    );
    const functions = rustNamedFunctions(entry.source, entry.modulePath).filter(({ start }) =>
      nodeIsActive(entry, start),
    );
    for (const match of entry.code.matchAll(staticFamilyReference)) {
      const offset = match.index ?? 0;
      if (!nodeIsActive(entry, offset)) continue;
      const owner = functions.findLast(({ bodyStart, end }) => bodyStart < offset && offset < end);
      assert.ok(
        owner,
        `unregistered sealed-family call site ${entry.crate}::${modulePathAt(entry, offset)}::line-${lineNumber(entry.source, offset)}`,
      );
    }
    for (const fn of functions) {
      const body = entry.code.slice(fn.bodyStart + 1, fn.end - 1);
      const header = entry.code.slice(fn.start, fn.bodyStart);
      const actualModulePath = modulePathAt(entry, fn.start);
      const variables = functionVariableTypes(
        header,
        body,
        fn.name,
        typeAliases,
        members,
        returnTypes,
      );
      const genericParameters = genericNames(header);
      const item = fn.name.replace(entry.modulePath, actualModulePath);
      const found: Array<{ member: string; offset: number }> = [];
      for (const match of body.matchAll(staticFamilyReference)) {
        found.push({ member: match[1]!, offset: match.index ?? 0 });
      }
      for (const match of body.matchAll(
        /\.\s*(with_(?:value_domain|flow|context|provider_completeness|world_assumption|revision))\s*\(/gu,
      )) {
        const attribution = receiverPrecisionAttribution(
          body,
          match.index ?? 0,
          entry,
          actualModulePath,
          variables,
          typeAliases,
          definitions,
          members,
          returnTypes,
          genericParameters,
        );
        assert.notEqual(
          attribution,
          "unresolved",
          `sealed-family call receiver unresolved ${entry.crate}::${item}`,
        );
        if (attribution === "precision") {
          found.push({ member: match[1]!, offset: match.index ?? 0 });
        }
      }
      const uniqueFound = [
        ...new Map(found.map((call) => [`${call.offset}|${call.member}`, call])).values(),
      ];
      const ordinals = new Map<string, number>();
      for (const call of uniqueFound.toSorted((left, right) => left.offset - right.offset)) {
        const ordinal = (ordinals.get(call.member) ?? 0) + 1;
        ordinals.set(call.member, ordinal);
        rows.push({
          crate: entry.crate,
          file: entry.file,
          item,
          member: call.member,
          ordinal,
          owner: `${entry.crate} ${fn.shortName.replaceAll("_", " ")}`,
        });
      }
    }
  }
  return rows.toSorted((left, right) => compareCodePoint(callIdentity(left), callIdentity(right)));
}

function probeChanges(probe: MutationProbe): readonly MutationChange[] {
  if (probe.changes) return probe.changes;
  assert.ok(
    probe.sourcePath && probe.from !== undefined && probe.to !== undefined,
    `probe span not resolvable ${probe.id}`,
  );
  return [{ sourcePath: probe.sourcePath, from: probe.from, to: probe.to }];
}

function probeSpanByPoint(
  authority: PrecisionAuthority,
  sources: readonly RustSource[],
): Map<string, Set<string>> {
  const sourcesByFile = new Map(sources.map((entry) => [entry.file, entry]));
  const pointRegions = new Map<string, { file: string; start: number; end: number }>();
  for (const point of authority.precisionEmissionPoints) {
    const entry = sourcesByFile.get(point.sourcePath);
    assert.ok(entry, `point item not resolvable ${point.id}`);
    const functions = rustNamedFunctions(entry.source, entry.modulePath).filter(
      ({ shortName }) => shortName === point.function,
    );
    assert.equal(functions.length, 1, `point item not resolvable ${point.id}`);
    pointRegions.set(point.id, {
      file: point.sourcePath,
      start: functions[0]!.start,
      end: functions[0]!.end,
    });
  }
  const exercised = new Map<string, Set<string>>();
  for (const probe of authority.mutationProbes) {
    const points = new Set<string>();
    for (const change of probeChanges(probe)) {
      const entry = sourcesByFile.get(change.sourcePath);
      if (!entry) {
        const raw = readFileSync(path.join(repoRoot, change.sourcePath), "utf8");
        const occurrences = raw.split(change.from).length - 1;
        assert.equal(occurrences, 1, `probe span not resolvable ${probe.id}`);
        continue;
      }
      const occurrences = entry.source.split(change.from).length - 1;
      assert.equal(occurrences, 1, `probe span not resolvable ${probe.id}`);
      const offset = entry.source.indexOf(change.from);
      for (const [pointId, region] of pointRegions) {
        if (region.file === change.sourcePath && region.start <= offset && offset < region.end)
          points.add(pointId);
      }
    }
    exercised.set(probe.id, points);
  }
  return exercised;
}

function assertBindings(authority: PrecisionAuthority, sources: readonly RustSource[]): void {
  const exercised = probeSpanByPoint(authority, sources);
  const probeIds = new Set(authority.mutationProbes.map(({ id }) => id));
  for (const point of authority.precisionEmissionPoints) {
    const binding =
      point.disposition?.bindingId ??
      point.reason?.match(/^(?:exercisedBy|coveredBySweep|guardedBy):(.+)$/u)?.[1];
    if (!binding || !probeIds.has(binding)) continue;
    assert.ok(exercised.get(binding)?.has(point.id), `binding does not exercise ${point.id}`);
  }
  for (const exclusion of authority.gatedExclusions ?? []) {
    assert.ok(
      exercised.get(exclusion.probe)?.has(exclusion.pointId),
      `gated exclusion ${exclusion.id} not exercised by ${exclusion.probe}`,
    );
  }
}

const metadata = readMetadata();
const scope = precisionScope(metadata);
const sources = precisionSources(scope);
const derivedCalls = deriveFamilyCalls(sources);
const derivedContainers = deriveDeserializationContainers(sources);
const precisionAuthority = JSON.parse(
  readFileSync(precisionAuthorityPath, "utf8"),
) as PrecisionAuthority;
assertBindings(precisionAuthority, sources);

if (process.argv.includes("--print-baseline")) {
  process.stdout.write(
    `${JSON.stringify(
      {
        scopeCrates: scope.map(({ name }) => name),
        familyCallSites: derivedCalls,
        deserializationContainers: derivedContainers.map((identity) => ({
          identity,
          owner: `${identity.split("::")[0]} wire-format maintainers`,
        })),
      },
      null,
      2,
    )}\n`,
  );
  process.exit(0);
}

const instrumentAuthority = JSON.parse(
  readFileSync(instrumentAuthorityPath, "utf8"),
) as CensusInstrumentAuthority;
const registeredCalls = new Map(
  instrumentAuthority.precision.familyCallSites.map((site) => [callIdentity(site), site]),
);
for (const call of derivedCalls) {
  if (call.member === "from_axes_for_tests") {
    assert.fail(`production reaches test constructor ${call.crate}::${call.item}`);
  }
  assert.ok(
    registeredCalls.has(callIdentity(call)),
    `unregistered sealed-family call site ${call.crate}::${call.item}`,
  );
}
assert.deepEqual(
  [...registeredCalls.keys()].toSorted(compareCodePoint),
  derivedCalls.map(callIdentity).toSorted(compareCodePoint),
  "sealed-family call-site census drift",
);

const registeredContainers = new Set(
  instrumentAuthority.precision.deserializationContainers.map(({ identity }) => identity),
);
for (const identity of derivedContainers) {
  assert.ok(
    registeredContainers.has(identity),
    `unregistered deserialization container ${identity}`,
  );
}
assert.deepEqual(
  [...registeredContainers].toSorted(compareCodePoint),
  derivedContainers,
  "deserialization container census drift",
);
assert.ok(
  derivedCalls.length >= instrumentAuthority.precision.birthFloors.familyCallSites,
  "census floor unmet familyCallSites",
);
assert.ok(
  derivedContainers.length >= instrumentAuthority.precision.birthFloors.deserializationContainers,
  "census floor unmet deserializationContainers",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.precision-authority",
      precisionScopeCrateCount: scope.length,
      familyCallSiteCount: derivedCalls.length,
      deserializationContainerCount: derivedContainers.length,
      emissionPointCount: precisionAuthority.precisionEmissionPoints.length,
      mutationProbeCount: precisionAuthority.mutationProbes.length,
    },
    null,
    2,
  )}\n`,
);
