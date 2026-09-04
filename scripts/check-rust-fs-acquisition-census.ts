import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  compareCodePoint,
  deriveWriteScope,
  forceWarnCensusArgv,
  maskRustCommentsAndLiterals,
  readCargoMetadata,
  matchingRustDelimiter,
  rustNamedFunctions,
  type RustNamedFunction,
} from "./lib/rust-write-authority";
import {
  attributeNodes,
  deriveCfgAcquisitionCensus,
  type CfgCountedAcquisition,
} from "./lib/rust-cfg-acquisition";

interface CompilerSpan {
  readonly file_name: string;
  readonly line_start: number;
  readonly column_start: number;
  readonly is_primary: boolean;
}

interface CargoCompilerMessage {
  readonly reason: string;
  readonly package_id: string;
  readonly message: {
    readonly code: { readonly code: string } | null;
    readonly message: string;
    readonly spans: readonly CompilerSpan[];
  };
}

export interface AcquisitionSite {
  readonly crate: string;
  readonly file: string;
  readonly function: string;
  readonly api: string;
  readonly expressionCount: number;
  readonly owner: string;
  readonly disposition: string;
}

interface CensusAuthority {
  readonly acquisitionSites: readonly AcquisitionSite[];
  readonly cfgCountedAcquisitions: readonly (CfgCountedAcquisition & {
    readonly owner: string;
    readonly disposition: string;
  })[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authorityPath = path.join(repoRoot, "rust/omena-fs-acquisition-census.json");

const writeScope = deriveWriteScope(repoRoot);
assert.ok(writeScope.scope.length >= 1, "write scope package set is empty");
const scopePackages = readCargoMetadata(repoRoot).packages.filter(({ name }) =>
  writeScope.scope.includes(name),
);
assert.deepEqual(
  scopePackages.map(({ name }) => name).toSorted(),
  writeScope.scope,
  "write scope package set diverged from cargo metadata",
);
for (const pkg of scopePackages) {
  const manifest = readFileSync(pkg.manifest_path, "utf8");
  assert.match(
    manifest,
    /\[lints\]\s*workspace\s*=\s*true\b/u,
    `crate ${pkg.name} does not inherit the workspace lint table`,
  );
}

function packageName(packageId: string): string {
  const source = packageId.slice(0, packageId.lastIndexOf("#"));
  return path.basename(source);
}

function modulePath(file: string): string {
  const sourcePath = file.replace(/^rust\/crates\/[^/]+\/src\//u, "").replace(/\.rs$/u, "");
  const parts = sourcePath.split("/");
  if (["lib", "main", "mod"].includes(parts.at(-1) ?? "")) parts.pop();
  return parts.length === 0 ? "crate" : `crate::${parts.join("::")}`;
}

function offsetAt(source: string, line: number, column: number): number {
  const lines = source.split(/(?<=\n)/u);
  assert.ok(line >= 1 && line <= lines.length, `source line is out of range: ${line}`);
  return (
    lines.slice(0, line - 1).reduce((total, current) => total + current.length, 0) + column - 1
  );
}

function enclosingFunction(
  source: string,
  file: string,
  line: number,
  column: number,
): RustNamedFunction {
  const offset = offsetAt(source, line, column);
  const fn = rustNamedFunctions(source, modulePath(file)).findLast(
    ({ start, end }) => start < offset && offset < end,
  );
  assert.ok(fn, `enclosing item not resolvable ${file}:${line}`);
  return fn;
}

function defaultOwner(crateName: string, fn: RustNamedFunction): string {
  return `${crateName} ${fn.shortName.replaceAll("_", " ")}`;
}

function deriveAcquisitionSites(stdout: string): AcquisitionSite[] {
  const messages = stdout
    .split("\n")
    .filter(Boolean)
    .flatMap((line) => {
      try {
        return [JSON.parse(line) as CargoCompilerMessage];
      } catch {
        return [];
      }
    })
    .filter(
      (message) =>
        message.reason === "compiler-message" &&
        message.message.code?.code === "clippy::disallowed_methods",
    );
  assert.ok(messages.length >= 1, "force-warn census emitted no expected lint compiler-message");
  const scope = new Set(deriveWriteScope(repoRoot).scope);
  const grouped = new Map<string, AcquisitionSite>();
  for (const message of messages) {
    const crateName = packageName(message.package_id);
    assert.ok(scope.has(crateName), `census invocation diverged: message from ${crateName}`);
    const span =
      message.message.spans.find(({ is_primary }) => is_primary) ?? message.message.spans[0];
    assert.ok(span, `lint message has no source span: ${message.message.message}`);
    const file = path.posix.join("rust", span.file_name);
    const source = readFileSync(path.join(repoRoot, file), "utf8");
    const fn = enclosingFunction(source, file, span.line_start, span.column_start);
    const api = message.message.message.match(/`([^`]+)`/u)?.[1];
    assert.ok(api, `lint API not resolvable from message: ${message.message.message}`);
    const key = [crateName, file, fn.name, api].join("|");
    const current = grouped.get(key);
    grouped.set(key, {
      crate: crateName,
      file,
      function: fn.name,
      api,
      expressionCount: (current?.expressionCount ?? 0) + 1,
      owner: current?.owner ?? defaultOwner(crateName, fn),
      disposition: current?.disposition ?? "retained filesystem lifecycle boundary",
    });
  }
  return [...grouped.values()].toSorted((left, right) =>
    compareCodePoint(siteIdentity(left), siteIdentity(right)),
  );
}

function siteIdentity(site: Pick<AcquisitionSite, "crate" | "file" | "function" | "api">): string {
  return [site.crate, site.file, site.function, site.api].join("|");
}

interface RustAttribute {
  readonly start: number;
  readonly end: number;
  readonly text: string;
}

function rustAttributes(source: string): RustAttribute[] {
  const code = maskRustCommentsAndLiterals(source);
  const attributes: RustAttribute[] = [];
  for (let cursor = 0; cursor < code.length; cursor += 1) {
    if (code[cursor] !== "#") continue;
    let bracket = cursor + 1;
    if (code[bracket] === "!") bracket += 1;
    if (code[bracket] !== "[") continue;
    const end = matchingRustDelimiter(code, bracket, "[", "]") + 1;
    attributes.push({ start: cursor, end, text: source.slice(cursor, end) });
    cursor = end - 1;
  }
  return attributes;
}

function onlyAttributesAndWhitespace(source: string): boolean {
  const code = maskRustCommentsAndLiterals(source).split("");
  for (const attribute of rustAttributes(source)) {
    for (let index = attribute.start; index < attribute.end; index += 1) {
      if (code[index] !== "\n" && code[index] !== "\r") code[index] = " ";
    }
  }
  return code.join("").trim().length === 0;
}

function attributedNonFunctionItem(source: string, suppression: RustAttribute): string {
  const node = attributeNodes(source).find(({ attributes }) =>
    attributes.some(({ start }) => start === suppression.start),
  );
  if (!node) {
    const line = source.slice(0, suppression.start).split("\n").length;
    return `line-${line}`;
  }
  const header = maskRustCommentsAndLiterals(source.slice(node.start, node.end));
  const named = header.match(
    /^\s*(?:pub(?:\([^)]*\))?\s+)?(mod|trait|struct|enum|union)\s+([A-Za-z_][A-Za-z0-9_]*)/u,
  );
  if (named) return `${named[1]} ${named[2]}`;
  const implementation = header.match(/^\s*(impl)\b([^<{]*)/u);
  if (implementation) return `${implementation[1]} ${implementation[2]!.trim()}`;
  const line = source.slice(0, suppression.start).split("\n").length;
  return `line-${line}`;
}

function assertSuppressionGranularity(sites: readonly AcquisitionSite[]): void {
  const files = [...new Set(sites.map(({ file }) => file))];
  for (const file of files) {
    const source = readFileSync(path.join(repoRoot, file), "utf8");
    const functions = rustNamedFunctions(source, modulePath(file));
    const suppressions = rustAttributes(source).filter(({ text }) =>
      /clippy\s*::\s*disallowed_methods/u.test(text),
    );
    const functionSuppressions = new Map<string, RustAttribute[]>();
    for (const suppression of suppressions) {
      const insideBody = functions.findLast(
        ({ bodyStart, end }) => bodyStart < suppression.start && suppression.end < end,
      );
      if (insideBody) {
        const line = source.slice(0, suppression.start).split("\n").length;
        assert.fail(`suppression below fn granularity ${file}:${line}`);
      }
      const owner = functions.find(
        ({ start }) =>
          suppression.end <= start &&
          onlyAttributesAndWhitespace(source.slice(suppression.end, start)),
      );
      if (!owner) {
        assert.fail(
          `suppression above fn granularity ${file}:${attributedNonFunctionItem(source, suppression)}`,
        );
      }
      assert.match(
        suppression.text,
        /\bexpect\s*\(/u,
        `suppression above fn granularity ${file}:${owner.name}`,
      );
      assert.match(
        suppression.text,
        /reason\s*=\s*"[^"]+:[^"]+"/u,
        `suppression missing owner disposition ${file}:${owner.name}`,
      );
      const rows = functionSuppressions.get(owner.name) ?? [];
      rows.push(suppression);
      functionSuppressions.set(owner.name, rows);
    }
    for (const site of sites.filter(({ file: siteFile }) => siteFile === file)) {
      assert.ok(
        (functionSuppressions.get(site.function)?.length ?? 0) >= 1,
        `registered acquisition missing fn expectation ${file}:${site.function}`,
      );
    }
  }
}

const { cargoArgs, scope } = forceWarnCensusArgv(repoRoot);
const result = spawnSync("cargo", cargoArgs, {
  cwd: repoRoot,
  encoding: "utf8",
  maxBuffer: 128 * 1024 * 1024,
});
const exit = result.status ?? 127;
assert.equal(exit, 0, `force-warn census exited ${exit}:\n${result.stderr}`);
const observed = deriveAcquisitionSites(result.stdout);
const cfgCensus = deriveCfgAcquisitionCensus(repoRoot, result.stdout);
assert.equal(
  cfgCensus.preOrderEvaluatedAttributeNodeCount,
  cfgCensus.attributeNodeCount,
  "cfg attribute node pre-order evaluation is incomplete",
);

if (process.argv.includes("--print-baseline")) {
  process.stdout.write(`${JSON.stringify(observed, null, 2)}\n`);
  process.exit(0);
}
if (process.argv.includes("--print-cfg")) {
  process.stdout.write(`${JSON.stringify(cfgCensus, null, 2)}\n`);
  process.exit(0);
}

assertSuppressionGranularity(observed);
const authority = JSON.parse(readFileSync(authorityPath, "utf8")) as CensusAuthority;
assert.ok(Array.isArray(authority.acquisitionSites), "census authority lacks acquisitionSites");
assert.ok(
  Array.isArray(authority.cfgCountedAcquisitions),
  "census authority lacks cfgCountedAcquisitions",
);
assert.deepEqual(
  authority.cfgCountedAcquisitions.map(
    ({ owner: _owner, disposition: _disposition, ...site }) => site,
  ),
  cfgCensus.cfgCounted,
  "cfg-gated acquisition uncounted",
);
const registeredByIdentity = new Map(
  authority.acquisitionSites.map((site) => [siteIdentity(site), site]),
);
const observedByIdentity = new Map(observed.map((site) => [siteIdentity(site), site]));
for (const site of observed) {
  const registered = registeredByIdentity.get(siteIdentity(site));
  if (!registered) {
    assert.fail(`unregistered acquisition site ${site.crate}::${site.function}::${site.api}`);
  }
  assert.ok(
    site.expressionCount <= registered.expressionCount,
    `registered acquisition expression count grew ${siteIdentity(site)}`,
  );
}
const removals = authority.acquisitionSites.filter(
  (site) => !observedByIdentity.has(siteIdentity(site)),
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.fs-acquisition-census",
      cargoArgv: ["cargo", ...cargoArgs],
      scopePackageCount: scope.scope.length,
      refusalPackages: scope.refusals,
      messagePackageSet: [...new Set(observed.map(({ crate: crateName }) => crateName))].toSorted(),
      registeredKeyCount: authority.acquisitionSites.length,
      observedKeyCount: observed.length,
      observedExpressionCount: observed.reduce((count, site) => count + site.expressionCount, 0),
      removals: removals.map(siteIdentity),
      cfgCountedAcquisitionCount: cfgCensus.cfgCounted.length,
      reachedProductionFiles: cfgCensus.reachedProductionFiles,
      testNamedOrphans: cfgCensus.testNamedOrphans,
      supportedTargets: cfgCensus.supportedTargets,
      attributeNodeCount: cfgCensus.attributeNodeCount,
      preOrderEvaluatedAttributeNodeCount: cfgCensus.preOrderEvaluatedAttributeNodeCount,
    },
    null,
    2,
  )}\n`,
);
