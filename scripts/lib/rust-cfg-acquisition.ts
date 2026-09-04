import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import {
  compareCodePoint,
  deriveWriteScope,
  isCargoLibraryTarget,
  maskRustCommentsAndLiterals,
  matchingRustDelimiter,
  readCargoMetadata,
  readClippyDisallowedMethods,
  rustNamedFunctions,
  type CargoPackage,
} from "./rust-write-authority";

export type CfgExpr =
  | { readonly kind: "atom"; readonly name: string }
  | { readonly kind: "value"; readonly name: string; readonly value: string }
  | { readonly kind: "all" | "any"; readonly operands: readonly CfgExpr[] }
  | { readonly kind: "not"; readonly operand: CfgExpr };

export interface CfgSet {
  readonly atoms: ReadonlySet<string>;
  readonly values: ReadonlyMap<string, ReadonlySet<string>>;
}

export interface Attribute {
  readonly start: number;
  readonly end: number;
  readonly text: string;
  readonly inner: boolean;
}

export interface AttributeRegion {
  readonly attribute: Attribute;
  readonly start: number;
  readonly end: number;
  readonly predicates: readonly CfgExpr[];
}

export interface AttributeNode {
  readonly start: number;
  readonly end: number;
  readonly attributes: readonly Attribute[];
  readonly predicates: readonly CfgExpr[];
}

export interface ModuleContext {
  readonly crate: string;
  readonly packageRoot: string;
  readonly targetName: string;
  readonly targetRoot: string;
  readonly targetKind: readonly string[];
  readonly file: string;
  readonly modulePath: string;
  readonly inheritedPredicates: readonly CfgExpr[];
}

interface LexicalAcquisition {
  readonly file: string;
  readonly offset: number;
  readonly line: number;
  readonly api: string;
  readonly function: string;
}

export interface CfgCountedAcquisition {
  readonly crate: string;
  readonly file: string;
  readonly line: number;
  readonly function: string;
  readonly api: string;
  readonly effectivePredicate: string;
  readonly verificationLane: string;
}

export interface CfgAcquisitionCensus {
  readonly runnerCfg: readonly string[];
  readonly supportedTargets: readonly string[];
  readonly reachedProductionFiles: number;
  readonly testNamedOrphans: readonly string[];
  readonly attributeNodeCount: number;
  readonly preOrderEvaluatedAttributeNodeCount: number;
  readonly compiledSpellingCount: number;
  readonly testGatedSpellingCount: number;
  readonly cfgCounted: readonly CfgCountedAcquisition[];
}

class CfgParser {
  private cursor = 0;

  constructor(private readonly input: string) {}

  parse(): CfgExpr {
    const expression = this.expression();
    this.whitespace();
    assert.equal(
      this.cursor,
      this.input.length,
      `cfg predicate undecidable trailing token ${this.input.slice(this.cursor)}`,
    );
    return expression;
  }

  private expression(): CfgExpr {
    this.whitespace();
    const name = this.identifier();
    this.whitespace();
    if (this.input[this.cursor] === "=") {
      this.cursor += 1;
      this.whitespace();
      return { kind: "value", name, value: this.string() };
    }
    if (this.input[this.cursor] !== "(") return { kind: "atom", name };
    this.cursor += 1;
    if (name === "not") {
      const operand = this.expression();
      this.whitespace();
      assert.equal(this.input[this.cursor], ")", "cfg predicate undecidable not arity");
      this.cursor += 1;
      return { kind: "not", operand };
    }
    assert.ok(name === "all" || name === "any", `cfg predicate undecidable operator ${name}`);
    const operands: CfgExpr[] = [];
    while (true) {
      this.whitespace();
      if (this.input[this.cursor] === ")") {
        this.cursor += 1;
        break;
      }
      operands.push(this.expression());
      this.whitespace();
      if (this.input[this.cursor] === ",") {
        this.cursor += 1;
        continue;
      }
      assert.equal(this.input[this.cursor], ")", "cfg predicate undecidable operand separator");
      this.cursor += 1;
      break;
    }
    return { kind: name, operands };
  }

  private identifier(): string {
    const match = this.input.slice(this.cursor).match(/^[A-Za-z_][A-Za-z0-9_]*/u);
    assert.ok(match, `cfg predicate undecidable at ${this.input.slice(this.cursor)}`);
    this.cursor += match[0].length;
    return match[0];
  }

  private string(): string {
    assert.equal(this.input[this.cursor], '"', "cfg predicate undecidable non-string value");
    this.cursor += 1;
    let value = "";
    while (this.cursor < this.input.length) {
      const current = this.input[this.cursor++]!;
      if (current === '"') return value;
      if (current === "\\") {
        const escaped = this.input[this.cursor++];
        assert.ok(escaped, "cfg predicate undecidable unterminated escape");
        value += escaped;
      } else {
        value += current;
      }
    }
    assert.fail("cfg predicate undecidable unterminated string");
  }

  private whitespace(): void {
    while (/\s/u.test(this.input[this.cursor] ?? "")) this.cursor += 1;
  }
}

export function parseCfg(input: string): CfgExpr {
  return new CfgParser(input.trim()).parse();
}

export function renderCfg(expression: CfgExpr): string {
  switch (expression.kind) {
    case "atom":
      return expression.name;
    case "value":
      return `${expression.name} = "${expression.value}"`;
    case "not":
      return `not(${renderCfg(expression.operand)})`;
    case "all":
    case "any":
      return `${expression.kind}(${expression.operands.map(renderCfg).join(", ")})`;
  }
}

export function evaluateCfg(expression: CfgExpr, cfg: CfgSet): boolean {
  switch (expression.kind) {
    case "atom":
      return cfg.atoms.has(expression.name);
    case "value":
      return cfg.values.get(expression.name)?.has(expression.value) ?? false;
    case "not":
      return !evaluateCfg(expression.operand, cfg);
    case "all":
      return expression.operands.every((operand) => evaluateCfg(operand, cfg));
    case "any":
      return expression.operands.some((operand) => evaluateCfg(operand, cfg));
  }
}

export function parseRustcCfg(output: string): CfgSet {
  const atoms = new Set<string>();
  const values = new Map<string, Set<string>>();
  for (const line of output.trim().split("\n").filter(Boolean)) {
    const value = line.match(/^([A-Za-z_][A-Za-z0-9_]*)="([^"]*)"$/u);
    if (value) {
      const entries = values.get(value[1]!) ?? new Set<string>();
      entries.add(value[2]!);
      values.set(value[1]!, entries);
    } else {
      assert.match(
        line,
        /^[A-Za-z_][A-Za-z0-9_]*$/u,
        `cfg predicate undecidable rustc output ${line}`,
      );
      atoms.add(line);
    }
  }
  return { atoms, values };
}

export function withPackageCfg(
  base: CfgSet,
  pkg: CargoPackage,
  buildCfgs: readonly string[],
): CfgSet {
  const atoms = new Set(base.atoms);
  const values = new Map([...base.values].map(([name, rows]) => [name, new Set(rows)]));
  const featureValues = values.get("feature") ?? new Set<string>();
  for (const feature of Object.keys(pkg.features)) featureValues.add(feature);
  values.set("feature", featureValues);
  for (const row of buildCfgs) {
    const parsed = parseCfg(row);
    if (parsed.kind === "atom") atoms.add(parsed.name);
    else if (parsed.kind === "value") {
      const entries = values.get(parsed.name) ?? new Set<string>();
      entries.add(parsed.value);
      values.set(parsed.name, entries);
    } else {
      assert.fail(`cfg predicate undecidable build-script cfg ${row}`);
    }
  }
  return { atoms, values };
}

export function withTest(cfg: CfgSet, enabled: boolean): CfgSet {
  const atoms = new Set(cfg.atoms);
  if (enabled) atoms.add("test");
  else atoms.delete("test");
  return { atoms, values: cfg.values };
}

export function attributes(source: string): Attribute[] {
  const code = maskRustCommentsAndLiterals(source);
  const rows: Attribute[] = [];
  for (let cursor = 0; cursor < code.length; cursor += 1) {
    if (code[cursor] !== "#") continue;
    let bracket = cursor + 1;
    const inner = code[bracket] === "!";
    if (inner) bracket += 1;
    if (code[bracket] !== "[") continue;
    const end = matchingRustDelimiter(code, bracket, "[", "]") + 1;
    rows.push({ start: cursor, end, text: source.slice(cursor, end), inner });
    cursor = end - 1;
  }
  return rows;
}

export function cfgPredicates(attribute: Attribute): CfgExpr[] {
  const direct = attribute.text.match(/^#!?\s*\[\s*cfg\s*\(([\s\S]*)\)\s*\]$/u);
  if (direct) return [parseCfg(direct[1]!)];
  const cfgAttr = attribute.text.match(/^#!?\s*\[\s*cfg_attr\s*\(([\s\S]*)\)\s*\]$/u);
  if (!cfgAttr) return [];
  const operands = splitTopLevel(cfgAttr[1]!);
  assert.ok(operands.length >= 2, `cfg predicate undecidable cfg_attr ${attribute.text}`);
  const condition = parseCfg(operands[0]!);
  return operands.slice(1).flatMap((operand) => {
    const nested = operand.trim().match(/^cfg\s*\(([\s\S]*)\)$/u);
    return nested
      ? [
          {
            kind: "any" as const,
            operands: [{ kind: "not" as const, operand: condition }, parseCfg(nested[1]!)],
          },
        ]
      : [];
  });
}

function splitTopLevel(input: string): string[] {
  const rows: string[] = [];
  let start = 0;
  let round = 0;
  let square = 0;
  let quote = false;
  let escaped = false;
  for (let cursor = 0; cursor < input.length; cursor += 1) {
    const current = input[cursor]!;
    if (quote) {
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === '"') quote = false;
      continue;
    }
    if (current === '"') quote = true;
    else if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "," && round === 0 && square === 0) {
      rows.push(input.slice(start, cursor));
      start = cursor + 1;
    }
  }
  rows.push(input.slice(start));
  return rows;
}

function skipAttributesAndWhitespace(code: string, start: number): number {
  let cursor = start;
  while (true) {
    while (/\s/u.test(code[cursor] ?? "")) cursor += 1;
    if (code[cursor] !== "#") return cursor;
    let bracket = cursor + 1;
    if (code[bracket] === "!") bracket += 1;
    if (code[bracket] !== "[") return cursor;
    cursor = matchingRustDelimiter(code, bracket, "[", "]") + 1;
  }
}

function nodeEnd(code: string, start: number): number {
  let round = 0;
  let square = 0;
  let angle = 0;
  for (let cursor = start; cursor < code.length; cursor += 1) {
    const current = code[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "<") angle += 1;
    else if (current === ">") angle = Math.max(0, angle - 1);
    else if (current === "{" && round === 0 && square === 0 && angle === 0) {
      return matchingRustDelimiter(code, cursor, "{", "}") + 1;
    } else if ((current === ";" || current === ",") && round === 0 && square === 0 && angle === 0) {
      return cursor + 1;
    }
  }
  assert.fail(`cfg predicate undecidable attribute node at byte ${start}`);
}

export function attributeRegions(source: string): AttributeRegion[] {
  const code = maskRustCommentsAndLiterals(source);
  return attributes(source)
    .filter(({ inner }) => !inner)
    .map((attribute) => {
      const start = skipAttributesAndWhitespace(code, attribute.end);
      return { attribute, start, end: nodeEnd(code, start), predicates: cfgPredicates(attribute) };
    });
}

/**
 * Group adjacent attributes by the Rust node they govern. Source order is a
 * pre-order walk for nested Rust nodes: an enclosing node's attributes precede
 * every attributed node in its body. The flat collection is intentional so a
 * cfg-false parent never prunes discovery of attributed descendants.
 */
export function attributeNodes(source: string): AttributeNode[] {
  const grouped = new Map<string, AttributeNode>();
  for (const region of attributeRegions(source)) {
    const key = `${region.start}:${region.end}`;
    const current = grouped.get(key);
    grouped.set(key, {
      start: region.start,
      end: region.end,
      attributes: [...(current?.attributes ?? []), region.attribute],
      predicates: [...(current?.predicates ?? []), ...region.predicates],
    });
  }
  return [...grouped.values()].toSorted((left, right) => {
    const leftAttribute = left.attributes[0]!.start;
    const rightAttribute = right.attributes[0]!.start;
    return leftAttribute - rightAttribute || right.end - left.end;
  });
}

export function effectiveCfgPredicatesAt(
  inheritedPredicates: readonly CfgExpr[],
  innerPredicates: readonly CfgExpr[],
  regions: readonly AttributeRegion[],
  offset: number,
): CfgExpr[] {
  return [
    ...inheritedPredicates,
    ...innerPredicates,
    ...regions
      .filter(({ start, end }) => start <= offset && offset < end)
      .flatMap(({ predicates }) => predicates),
  ];
}

export function moduleDeclarations(
  source: string,
  start: number,
  end: number,
): Array<{
  name: string;
  attributes: readonly Attribute[];
  body?: { start: number; end: number };
  pathOverride?: string;
}> {
  const code = maskRustCommentsAndLiterals(source);
  const allAttributes = attributes(source);
  const rows: Array<{
    name: string;
    attributes: readonly Attribute[];
    body?: { start: number; end: number };
    pathOverride?: string;
  }> = [];
  let cursor = start;
  let pending: Attribute[] = [];
  while (cursor < end) {
    while (/\s/u.test(code[cursor] ?? "")) cursor += 1;
    if (cursor >= end) break;
    const attribute = allAttributes.find(({ start: attributeStart }) => attributeStart === cursor);
    if (attribute) {
      if (!attribute.inner) pending.push(attribute);
      cursor = attribute.end;
      continue;
    }
    const remaining = code.slice(cursor, end);
    const declaration = remaining.match(
      /^(?:pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*([;{])/u,
    );
    if (declaration) {
      const name = declaration[1]!;
      const delimiter = cursor + declaration[0].lastIndexOf(declaration[2]!);
      const pathOverride = pending
        .map(({ text }) => text.match(/\bpath\s*=\s*"([^"]+)"/u)?.[1])
        .find(Boolean);
      if (declaration[2] === "{") {
        const close = matchingRustDelimiter(code, delimiter, "{", "}");
        rows.push({
          name,
          attributes: pending,
          body: { start: delimiter + 1, end: close },
          pathOverride,
        });
        cursor = close + 1;
      } else {
        rows.push({ name, attributes: pending, pathOverride });
        cursor = delimiter + 1;
      }
      pending = [];
      continue;
    }
    pending = [];
    const finish = nodeEnd(code, cursor);
    cursor = finish;
  }
  return rows;
}

function childModuleDirectory(file: string): string {
  const stem = path.basename(file, ".rs");
  return ["lib", "main", "mod"].includes(stem)
    ? path.dirname(file)
    : path.join(path.dirname(file), stem);
}

function resolveExternalModule(
  declaration: { readonly name: string; readonly pathOverride?: string },
  containingFile: string,
  inlineDirectory: string,
): string {
  if (declaration.pathOverride) {
    return path.resolve(path.dirname(containingFile), declaration.pathOverride);
  }
  const root = path.join(inlineDirectory, declaration.name);
  const candidates = [`${root}.rs`, path.join(root, "mod.rs")].filter((candidate) => {
    try {
      return readFileSync(candidate, "utf8").length >= 0;
    } catch {
      return false;
    }
  });
  assert.equal(
    candidates.length,
    1,
    `module source not resolvable ${containingFile}::${declaration.name}`,
  );
  return candidates[0]!;
}

function walk(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const item = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(item) : [item];
  });
}

function isTestNamedSource(file: string, sourceRoot: string): boolean {
  const relative = path.relative(sourceRoot, file).split(path.sep);
  return relative.includes("tests") || path.basename(file) === "tests.rs";
}

function discoverSupportedTargets(repoRoot: string): string[] {
  const workflowRoot = path.join(repoRoot, ".github/workflows");
  const targets = new Set<string>();
  const pattern =
    /\b(?:aarch64|x86_64|wasm32)-(?:apple-darwin|unknown-linux-gnu|pc-windows-msvc|unknown-unknown)\b/gu;
  for (const file of walk(workflowRoot).filter((candidate) => /\.ya?ml$/u.test(candidate))) {
    for (const match of readFileSync(file, "utf8").matchAll(pattern)) targets.add(match[0]);
  }
  assert.ok(targets.has("wasm32-unknown-unknown"), "supported target derivation lacks wasm32");
  return [...targets].toSorted();
}

function buildScriptCfgs(cargoStdout: string): Map<string, string[]> {
  const rows = new Map<string, string[]>();
  for (const line of cargoStdout.split("\n").filter(Boolean)) {
    try {
      const message = JSON.parse(line) as {
        reason?: string;
        package_id?: string;
        cfgs?: string[];
      };
      if (message.reason !== "build-script-executed" || !message.package_id) continue;
      const source = message.package_id.slice(0, message.package_id.lastIndexOf("#"));
      rows.set(path.basename(source), message.cfgs ?? []);
    } catch {
      // Cargo may interleave a non-JSON diagnostic; the force-warn parser owns
      // acceptance of compiler messages, while this arm uses only typed rows.
    }
  }
  return rows;
}

function lineNumber(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function lexicalAcquisitions(context: ModuleContext, source: string): LexicalAcquisition[] {
  const code = maskRustCommentsAndLiterals(source);
  const configured = readClippyDisallowedMethods(path.resolve(context.packageRoot, "../../.."));
  const rows: LexicalAcquisition[] = [];
  const seen = new Set<string>();
  const add = (offset: number, api: string): void => {
    const identity = `${offset}:${api}`;
    if (seen.has(identity)) return;
    seen.add(identity);
    const fn = rustNamedFunctions(source, context.modulePath).findLast(
      ({ start, end }) => start < offset && offset < end,
    );
    assert.ok(fn, `enclosing item not resolvable ${context.file}:${lineNumber(source, offset)}`);
    rows.push({
      file: context.file,
      offset,
      line: lineNumber(source, offset),
      api,
      function: fn.name,
    });
  };
  for (const { path: api } of configured) {
    const escaped = api.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    for (const match of code.matchAll(new RegExp(`(?<![A-Za-z0-9_:])(?:::)?${escaped}\\b`, "gu"))) {
      add(match.index ?? 0, api);
    }
  }
  const freeFsNames = configured
    .map(({ path: api }) => api.match(/^std::fs::([a-z_]+)$/u)?.[1])
    .filter((name): name is string => Boolean(name));
  for (const name of freeFsNames) {
    for (const match of code.matchAll(new RegExp(`\\bfs::${name}\\b`, "gu"))) {
      add(match.index ?? 0, `std::fs::${name}`);
    }
  }
  return rows.toSorted((left, right) => left.offset - right.offset);
}

export function combinePredicates(predicates: readonly CfgExpr[]): CfgExpr {
  return predicates.length === 1 ? predicates[0]! : { kind: "all", operands: predicates };
}

export interface RustModuleContextCensus {
  readonly contexts: readonly ModuleContext[];
  readonly reachedProductionFiles: number;
  readonly testNamedOrphans: readonly string[];
}

/**
 * Walk every library and binary module root compiled by the production lint
 * invocation. Each external file carries the effective cfg predicates inherited
 * from its declaring module; inline-node predicates remain available through
 * attributeRegions() at the consumer's exact offset.
 */
export function deriveRustModuleContexts(
  repoRoot: string,
  packages: readonly CargoPackage[],
): RustModuleContextCensus {
  const roots: ModuleContext[] = [];
  for (const pkg of packages) {
    const packageRoot = path.dirname(pkg.manifest_path);
    for (const target of pkg.targets.filter(
      (target) => isCargoLibraryTarget(target) || target.kind.includes("bin"),
    )) {
      roots.push({
        crate: pkg.name,
        packageRoot,
        targetName: target.name,
        targetRoot: path.resolve(target.src_path),
        targetKind: target.kind,
        file: path.resolve(target.src_path),
        modulePath: "crate",
        inheritedPredicates: [],
      });
    }
  }

  const reached = new Set<string>();
  const contexts: ModuleContext[] = [];
  const queue = [...roots];
  const visited = new Set<string>();
  while (queue.length > 0) {
    const context = queue.shift()!;
    const visitKey = `${context.file}|${context.inheritedPredicates.map(renderCfg).join("&&")}`;
    if (visited.has(visitKey)) continue;
    visited.add(visitKey);
    reached.add(context.file);
    contexts.push(context);
    const source = readFileSync(context.file, "utf8");
    const scanModules = (
      start: number,
      end: number,
      inlineDirectory: string,
      modulePath: string,
      inherited: readonly CfgExpr[],
    ): void => {
      for (const declaration of moduleDeclarations(source, start, end)) {
        const predicates = [...inherited, ...declaration.attributes.flatMap(cfgPredicates)];
        const childModulePath = `${modulePath}::${declaration.name}`;
        if (declaration.body) {
          scanModules(
            declaration.body.start,
            declaration.body.end,
            path.join(inlineDirectory, declaration.name),
            childModulePath,
            predicates,
          );
        } else {
          queue.push({
            ...context,
            file: resolveExternalModule(declaration, context.file, inlineDirectory),
            modulePath: childModulePath,
            inheritedPredicates: predicates,
          });
        }
      }
    };
    scanModules(
      0,
      source.length,
      childModuleDirectory(context.file),
      context.modulePath,
      context.inheritedPredicates,
    );
  }

  const testNamedOrphans: string[] = [];
  for (const pkg of packages) {
    const sourceRoot = path.join(path.dirname(pkg.manifest_path), "src");
    for (const file of walk(sourceRoot).filter((candidate) => candidate.endsWith(".rs"))) {
      if (reached.has(file)) continue;
      const relative = path.relative(repoRoot, file).split(path.sep).join("/");
      if (isTestNamedSource(file, sourceRoot)) testNamedOrphans.push(relative);
      else assert.fail(`production file not reached by module walk ${relative}`);
    }
  }
  return {
    contexts,
    reachedProductionFiles: reached.size,
    testNamedOrphans: testNamedOrphans.toSorted(),
  };
}

export function deriveCfgAcquisitionCensus(
  repoRoot: string,
  cargoStdout: string,
): CfgAcquisitionCensus {
  const metadata = readCargoMetadata(repoRoot);
  const writeScope = new Set(deriveWriteScope(repoRoot).scope);
  const packages = metadata.packages.filter(({ name }) => writeScope.has(name));
  const runnerOutput = execFileSync("rustc", ["--print", "cfg"], { encoding: "utf8" });
  const runnerBase = parseRustcCfg(runnerOutput);
  const supportedTargets = discoverSupportedTargets(repoRoot);
  const targetBases = new Map(
    supportedTargets.map((target) => [
      target,
      parseRustcCfg(
        execFileSync("rustc", ["--print", "cfg", "--target", target], { encoding: "utf8" }),
      ),
    ]),
  );
  const buildCfgByCrate = buildScriptCfgs(cargoStdout);
  const moduleCensus = deriveRustModuleContexts(repoRoot, packages);
  const contextRows = moduleCensus.contexts;

  let attributeNodeCount = 0;
  let preOrderEvaluatedAttributeNodeCount = 0;
  let compiledSpellingCount = 0;
  let testGatedSpellingCount = 0;
  const candidateStates = new Map<
    string,
    Array<{
      context: ModuleContext;
      acquisition: LexicalAcquisition;
      predicate: CfgExpr;
      runner: boolean;
      testGated: boolean;
      lanes: string[];
    }>
  >();
  for (const context of contextRows) {
    const pkg = packages.find(({ name }) => name === context.crate)!;
    const sourceRoot = path.join(path.dirname(pkg.manifest_path), "src");
    if (isTestNamedSource(context.file, sourceRoot)) continue;
    const source = readFileSync(context.file, "utf8");
    const regions = attributeRegions(source);
    const nodes = attributeNodes(source);
    attributeNodeCount += nodes.length;
    const innerPredicates = attributes(source)
      .filter(({ inner }) => inner)
      .flatMap(cfgPredicates);
    const runnerCfg = withPackageCfg(runnerBase, pkg, buildCfgByCrate.get(pkg.name) ?? []);
    for (const node of nodes) {
      const predicate = combinePredicates(
        effectiveCfgPredicatesAt(context.inheritedPredicates, innerPredicates, regions, node.start),
      );
      evaluateCfg(predicate, withTest(runnerCfg, false));
      evaluateCfg(predicate, withTest(runnerCfg, true));
      for (const targetCfg of targetBases.values()) {
        evaluateCfg(
          predicate,
          withTest(withPackageCfg(targetCfg, pkg, buildCfgByCrate.get(pkg.name) ?? []), false),
        );
      }
      preOrderEvaluatedAttributeNodeCount += 1;
    }
    for (const acquisition of lexicalAcquisitions(context, source)) {
      const predicates = effectiveCfgPredicatesAt(
        context.inheritedPredicates,
        innerPredicates,
        regions,
        acquisition.offset,
      );
      const predicate = combinePredicates(predicates);
      const runner = evaluateCfg(predicate, withTest(runnerCfg, false));
      const testGated =
        !runner &&
        evaluateCfg(predicate, withTest(runnerCfg, true)) &&
        [...targetBases].every(
          ([, targetCfg]) =>
            !evaluateCfg(
              predicate,
              withTest(withPackageCfg(targetCfg, pkg, buildCfgByCrate.get(pkg.name) ?? []), false),
            ),
        );
      const lanes = [...targetBases]
        .filter(([, targetCfg]) =>
          evaluateCfg(
            predicate,
            withTest(withPackageCfg(targetCfg, pkg, buildCfgByCrate.get(pkg.name) ?? []), false),
          ),
        )
        .map(([target]) => target);
      const key = `${context.file}|${acquisition.offset}|${acquisition.api}`;
      const rows = candidateStates.get(key) ?? [];
      rows.push({ context, acquisition, predicate, runner, testGated, lanes });
      candidateStates.set(key, rows);
    }
  }

  const cfgCounted: CfgCountedAcquisition[] = [];
  for (const states of candidateStates.values()) {
    if (states.some(({ runner }) => runner)) {
      compiledSpellingCount += 1;
      continue;
    }
    if (states.every(({ testGated }) => testGated)) {
      testGatedSpellingCount += 1;
      continue;
    }
    const state = states.find(({ testGated }) => !testGated)!;
    const lanes = [...new Set(states.flatMap(({ lanes: rows }) => rows))].toSorted();
    cfgCounted.push({
      crate: state.context.crate,
      file: path.relative(repoRoot, state.acquisition.file).split(path.sep).join("/"),
      line: state.acquisition.line,
      function: state.acquisition.function,
      api: state.acquisition.api,
      effectivePredicate: renderCfg(state.predicate),
      verificationLane: lanes[0] ?? "no lane compiles it",
    });
  }

  return {
    runnerCfg: runnerOutput.trim().split("\n"),
    supportedTargets,
    reachedProductionFiles: moduleCensus.reachedProductionFiles,
    testNamedOrphans: moduleCensus.testNamedOrphans,
    attributeNodeCount,
    preOrderEvaluatedAttributeNodeCount,
    compiledSpellingCount,
    testGatedSpellingCount,
    cfgCounted: cfgCounted.toSorted((left, right) =>
      compareCodePoint(
        `${left.file}:${left.line}:${left.api}`,
        `${right.file}:${right.line}:${right.api}`,
      ),
    ),
  };
}
