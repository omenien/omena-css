import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

const IDENTITY_INDEX_TYPE = "OmenaResolverStyleModuleConfirmationIdentityIndexV0";
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = path.join(repoRoot, "rust/omena-resolver-identity-index-caller-census.json");
const writeMode = process.argv.includes("--write");
const injectDisguisedNone = process.argv.includes("--inject-disguised-none");
const sourceRef = valueAfter("--source-ref");

assert.ok(
  !(writeMode && sourceRef !== undefined),
  "a historical source ref cannot update the current caller census",
);

type ExceptionKind =
  | "legacy-public-compatibility"
  | "tracked-query-boundary"
  | "test-reference-control";

interface FunctionParameter {
  readonly name: string;
  readonly typeSource: string;
  readonly isSelf: boolean;
}

interface RustFunction {
  readonly path: string;
  readonly name: string;
  readonly nameStart: number;
  readonly start: number;
  readonly signatureEnd: number;
  readonly bodyStart?: number;
  readonly end: number;
  readonly parameters: readonly FunctionParameter[];
  readonly identityParameterIndex?: number;
  readonly identityCallArgumentIndex?: number;
  readonly acceptsOptionalIdentityIndex: boolean;
  readonly visibility: "public" | "private";
  readonly attributes: string;
  readonly cfgTest: boolean;
  readonly returnType: string;
}

interface RustSource {
  readonly path: string;
  readonly source: string;
  readonly code: string;
  readonly productionSource: string;
  readonly functions: readonly RustFunction[];
}

interface CallerSite {
  readonly path: string;
  readonly line: number;
  readonly caller: string;
  readonly callee: string;
  readonly callOrdinal: number;
  readonly identityArgument: string;
  readonly identityFlow: "threaded" | "none";
  readonly disposition: "threaded" | "none-with-typed-justification" | "violation";
  readonly exceptionKind?: ExceptionKind;
}

interface TypedException {
  readonly siteKey: string;
  readonly kind: ExceptionKind;
}

interface CallerCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-query.resolver-identity-index-caller-census";
  readonly policy: {
    readonly sourceAuthority: "git-ls-files-rust-crates";
    readonly owningCheck: "rust/omena-resolver/identity-index-callers";
    readonly packageScript: "check:rust-omena-resolver-identity-index-callers";
    readonly allowedExceptionKinds: readonly ExceptionKind[];
  };
  readonly acceptingFunctions: readonly string[];
  readonly typedExceptions: readonly TypedException[];
  readonly sites: readonly CallerSite[];
  readonly summary: {
    readonly acceptingFunctionCount: number;
    readonly callSiteCount: number;
    readonly threadedCount: number;
    readonly justifiedNoneCount: number;
    readonly violationCount: number;
  };
  readonly siteDigest: string;
}

interface ExistingCensus {
  readonly typedExceptions?: readonly TypedException[];
}

const existing = readExistingCensus();
const typedExceptions = existing?.typedExceptions ?? [];
runScannerSelfTests();

const repoSources = loadTrackedRustSources(injectDisguisedNone, sourceRef);
const repoFunctions = repoSources.flatMap((source) => source.functions);
const identityAcceptingFunctions = repoFunctions.filter(
  (entry) => entry.identityParameterIndex !== undefined,
);
assert.ok(
  identityAcceptingFunctions.length > 0,
  "identity-index function census must be non-vacuous",
);

const callerSites = deriveCallerSites(
  repoSources,
  repoFunctions,
  identityAcceptingFunctions,
  typedExceptions,
);
const violations = callerSites.filter((site) => site.disposition === "violation");
assert.deepEqual(
  violations,
  [],
  `resolver identity-index callers contain unthreaded violations:\n${violations
    .map((site) => `  ${stableSiteKey(site)} line=${site.line} argument=${site.identityArgument}`)
    .join("\n")}`,
);

const census = buildCensus(identityAcceptingFunctions, typedExceptions, callerSites);
if (writeMode) {
  assert.equal(
    injectDisguisedNone,
    false,
    "the disguised-None mutation cannot be combined with --write",
  );
  writeFileSync(censusPath, `${JSON.stringify(census, null, 2)}\n`);
  const format = spawnSync(
    path.join(repoRoot, "node_modules/.bin/oxfmt"),
    ["--write", path.relative(repoRoot, censusPath)],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(format.status, 0, `failed to format caller census: ${format.stderr}`);
} else if (sourceRef === undefined) {
  assert.ok(
    existsSync(censusPath),
    "resolver identity-index caller census is missing; run the update command",
  );
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "resolver identity-index caller census is stale",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      acceptingFunctionCount: census.summary.acceptingFunctionCount,
      callSiteCount: census.summary.callSiteCount,
      threadedCount: census.summary.threadedCount,
      justifiedNoneCount: census.summary.justifiedNoneCount,
      violationCount: census.summary.violationCount,
      siteDigest: census.siteDigest,
    },
    null,
    2,
  )}\n`,
);

function readExistingCensus(): ExistingCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  return JSON.parse(readFileSync(censusPath, "utf8")) as ExistingCensus;
}

function loadTrackedRustSources(includeInjection: boolean, ref: string | undefined): RustSource[] {
  const result = spawnSync(
    "git",
    ref === undefined
      ? ["ls-files", "rust/crates"]
      : ["ls-tree", "-r", "--name-only", ref, "--", "rust/crates"],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );
  assert.equal(
    result.status,
    0,
    `failed to enumerate Rust sources${ref === undefined ? "" : ` at ${ref}`}: ${result.stderr}`,
  );
  const paths = result.stdout
    .split(/\r?\n/u)
    .filter((entry) => entry.endsWith(".rs"))
    .toSorted();
  const loaded = paths.map((relativePath) => loadRustSource(relativePath, ref));
  if (includeInjection) loaded.push(loadSyntheticSource(disguisedNoneFixture(), "<injected>"));
  return loaded;
}

function loadRustSource(relativePath: string, ref?: string): RustSource {
  if (ref === undefined) {
    return loadSyntheticSource(
      readFileSync(path.join(repoRoot, relativePath), "utf8"),
      relativePath,
    );
  }
  const result = spawnSync("git", ["show", `${ref}:${relativePath}`], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `failed to read ${relativePath} at ${ref}: ${result.stderr}`);
  return loadSyntheticSource(result.stdout, relativePath);
}

function loadSyntheticSource(source: string, sourcePath: string): RustSource {
  const code = maskRustNonCode(source);
  const productionSource = maskRustCfgTestItems(source);
  return {
    path: sourcePath,
    source,
    code,
    productionSource,
    functions: extractFunctions(sourcePath, source, code, productionSource),
  };
}

function extractFunctions(
  sourcePath: string,
  source: string,
  code: string,
  productionSource: string,
): RustFunction[] {
  const functions: RustFunction[] = [];
  const definition = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu;
  for (const match of code.matchAll(definition)) {
    const name = match[1];
    const nameStart = match.index + match[0].lastIndexOf(name);
    const openParen = findNextStructuralCharacter(code, match.index + match[0].length, "(");
    if (openParen < 0) continue;
    const closeParen = matchingDelimiter(code, openParen, "(", ")");
    assert.ok(closeParen >= 0, `${sourcePath}:${lineNumberAt(source, match.index)} unbalanced fn`);
    const itemStart = itemPrefixStart(code, match.index);
    const bodyStart = findFunctionBodyStart(code, closeParen + 1);
    const declarationEnd = bodyStart < 0 ? code.indexOf(";", closeParen + 1) + 1 : undefined;
    const signatureEnd = bodyStart < 0 ? declarationEnd! : bodyStart;
    const end = bodyStart < 0 ? declarationEnd! : matchingDelimiter(code, bodyStart, "{", "}") + 1;
    assert.ok(
      end > closeParen,
      `${sourcePath}:${lineNumberAt(source, match.index)} unbalanced body`,
    );
    const parameters = parseFunctionParameters(code.slice(openParen + 1, closeParen));
    const identityParameterIndex = parameters.findIndex((parameter) =>
      parameter.typeSource.includes(IDENTITY_INDEX_TYPE),
    );
    const rawIdentityIndex = identityParameterIndex < 0 ? undefined : identityParameterIndex;
    const identityCallArgumentIndex =
      rawIdentityIndex === undefined
        ? undefined
        : parameters.slice(0, rawIdentityIndex).filter((parameter) => !parameter.isSelf).length;
    const attributes = source.slice(itemStart, match.index);
    const visibilityPrefix = code.slice(itemStart, match.index);
    functions.push({
      path: sourcePath,
      name,
      nameStart,
      start: itemStart,
      signatureEnd,
      bodyStart: bodyStart < 0 ? undefined : bodyStart,
      end,
      parameters,
      identityParameterIndex: rawIdentityIndex,
      identityCallArgumentIndex,
      acceptsOptionalIdentityIndex:
        rawIdentityIndex !== undefined &&
        parameters[rawIdentityIndex]?.typeSource.includes("Option") === true,
      visibility: /\bpub(?:\s*\([^)]*\))?\s*$/u.test(visibilityPrefix) ? "public" : "private",
      attributes,
      cfgTest:
        sourcePath.includes("/tests/") ||
        sourcePath.endsWith("/tests.rs") ||
        productionSource.slice(match.index, match.index + match[0].length).trim().length === 0,
      returnType: code.slice(closeParen + 1, bodyStart < 0 ? closeParen + 1 : bodyStart).trim(),
    });
  }
  return functions;
}

function parseFunctionParameters(parameterSource: string): FunctionParameter[] {
  return topLevelArguments(parameterSource).map((parameter) => {
    const trimmed = parameter.trim();
    const isSelf = /^(?:&\s*(?:'[_A-Za-z][_A-Za-z0-9]*\s*)?)?(?:mut\s+)?self$/u.test(trimmed);
    const colon = topLevelColon(trimmed);
    return {
      name: isSelf ? "self" : colon < 0 ? trimmed : trimmed.slice(0, colon).trim(),
      typeSource: isSelf || colon < 0 ? "" : trimmed.slice(colon + 1).trim(),
      isSelf,
    };
  });
}

function deriveCallerSites(
  sources: readonly RustSource[],
  allFunctions: readonly RustFunction[],
  acceptingFunctions: readonly RustFunction[],
  exceptions: readonly TypedException[],
): CallerSite[] {
  const exceptionsByKey = new Map(exceptions.map((entry) => [entry.siteKey, entry]));
  assert.equal(exceptionsByKey.size, exceptions.length, "typed exception keys must be unique");
  const declarationsByName = Map.groupBy(acceptingFunctions, (entry) => entry.name);
  const sites: CallerSite[] = [];
  const seenExceptionKeys = new Set<string>();

  for (const source of sources) {
    for (const [callee, declarations] of declarationsByName) {
      const callPattern = new RegExp(
        `\\b${escapeRegExp(callee)}\\s*(?:::\\s*<[^>{}()]*>)?\\s*\\(`,
        "gu",
      );
      const ordinalByCaller = new Map<string, number>();
      for (const match of source.code.matchAll(callPattern)) {
        const nameStart = match.index;
        if (source.functions.some((entry) => entry.nameStart === nameStart)) continue;
        const declaration = compatibleDeclarationShape(declarations);
        const openParen = source.code.indexOf("(", nameStart + callee.length);
        const closeParen = matchingDelimiter(source.code, openParen, "(", ")");
        assert.ok(closeParen >= 0, `${source.path}:${lineNumberAt(source.source, nameStart)} call`);
        const caller = enclosingFunction(source.functions, nameStart);
        const methodCall = previousNonWhitespace(source.code, nameStart) === ".";
        const argumentIndex = methodCall
          ? declaration.identityCallArgumentIndex
          : declaration.identityParameterIndex;
        assert.notEqual(argumentIndex, undefined, `${callee} must expose an identity argument`);
        const argumentsList = topLevelArguments(source.code.slice(openParen + 1, closeParen));
        const identityArgument = argumentsList[argumentIndex!]?.trim();
        assert.ok(
          identityArgument !== undefined,
          `${source.path}:${lineNumberAt(source.source, nameStart)} ${callee} identity argument missing`,
        );
        const callerName = caller?.name ?? "<module>";
        const ordinalKey = `${callerName}#${callee}`;
        const callOrdinal = ordinalByCaller.get(ordinalKey) ?? 0;
        ordinalByCaller.set(ordinalKey, callOrdinal + 1);
        const identityFlow =
          declaration.acceptsOptionalIdentityIndex &&
          expressionIsProvablyNone(identityArgument, source, caller, nameStart, allFunctions)
            ? "none"
            : "threaded";
        const provisional: CallerSite = {
          path: source.path,
          line: lineNumberAt(source.source, nameStart),
          caller: callerName,
          callee,
          callOrdinal,
          identityArgument: compactExpression(identityArgument),
          identityFlow,
          disposition: identityFlow === "threaded" ? "threaded" : "violation",
        };
        const exception = exceptionsByKey.get(stableSiteKey(provisional));
        if (exception !== undefined) {
          assert.equal(
            identityFlow,
            "none",
            `threaded site cannot retain exception ${exception.siteKey}`,
          );
          assert.ok(
            caller !== undefined,
            `exception caller must be a Rust function: ${exception.siteKey}`,
          );
          assert.ok(
            exceptionGroundHolds(exception.kind, caller, allFunctions, source),
            `typed exception ground no longer holds: ${exception.siteKey} (${exception.kind})`,
          );
          seenExceptionKeys.add(exception.siteKey);
          sites.push({
            ...provisional,
            disposition: "none-with-typed-justification",
            exceptionKind: exception.kind,
          });
        } else {
          sites.push(provisional);
        }
      }
    }
  }

  const unthreadedPrivateAdapters = new Map<string, RustFunction>();
  for (const site of sites) {
    if (site.disposition !== "violation") continue;
    const caller = allFunctions.find(
      (entry) => entry.path === site.path && entry.name === site.caller,
    );
    if (
      caller?.visibility === "private" &&
      caller.identityParameterIndex === undefined &&
      caller.bodyStart !== undefined
    ) {
      unthreadedPrivateAdapters.set(`${caller.path}#${caller.name}`, caller);
    }
  }

  let discoveredAdapter = true;
  while (discoveredAdapter) {
    discoveredAdapter = false;
    const adapterNames = new Set(
      [...unthreadedPrivateAdapters.values()].map((entry) => entry.name),
    );
    for (const candidate of allFunctions) {
      const key = `${candidate.path}#${candidate.name}`;
      if (
        unthreadedPrivateAdapters.has(key) ||
        candidate.visibility !== "private" ||
        candidate.identityParameterIndex !== undefined ||
        candidate.bodyStart === undefined
      ) {
        continue;
      }
      const candidateSource = sources.find((source) => source.path === candidate.path);
      assert.ok(candidateSource, `source missing for ${key}`);
      const body = candidateSource.code.slice(candidate.bodyStart + 1, candidate.end - 1);
      if (
        [...unthreadedPrivateAdapters.values()].some(
          (adapter) =>
            privateItemVisibleFrom(adapter.path, candidate.path) &&
            adapterNames.has(adapter.name) &&
            new RegExp(`\\b${escapeRegExp(adapter.name)}\\s*\\(`, "u").test(body),
        )
      ) {
        unthreadedPrivateAdapters.set(key, candidate);
        discoveredAdapter = true;
      }
    }
  }

  const adapterDeclarationsByName = Map.groupBy(
    [...unthreadedPrivateAdapters.values()],
    (entry) => entry.name,
  );
  for (const source of sources) {
    for (const [callee, declarations] of adapterDeclarationsByName) {
      const visibleDeclarations = declarations
        .filter((declaration) => privateItemVisibleFrom(declaration.path, source.path))
        .toSorted(
          (left, right) =>
            rustModuleKey(right.path).split("::").length -
            rustModuleKey(left.path).split("::").length,
        );
      if (visibleDeclarations.length === 0) continue;
      const callPattern = new RegExp(`\\b${escapeRegExp(callee)}\\s*\\(`, "gu");
      const ordinalByCaller = new Map<string, number>();
      for (const match of source.code.matchAll(callPattern)) {
        const nameStart = match.index;
        if (source.functions.some((entry) => entry.nameStart === nameStart)) continue;
        const caller = enclosingFunction(source.functions, nameStart);
        const callerName = caller?.name ?? "<module>";
        const ordinalKey = `${callerName}#${callee}`;
        const callOrdinal = ordinalByCaller.get(ordinalKey) ?? 0;
        ordinalByCaller.set(ordinalKey, callOrdinal + 1);
        const declaration = visibleDeclarations[0];
        assert.ok(declaration, `private adapter declaration missing for ${callee}`);
        const provisional: CallerSite = {
          path: source.path,
          line: lineNumberAt(source.source, nameStart),
          caller: callerName,
          callee,
          callOrdinal,
          identityArgument: `<implicit-none:${declaration.path}#${declaration.name}>`,
          identityFlow: "none",
          disposition: "violation",
        };
        const key = stableSiteKey(provisional);
        if (sites.some((site) => stableSiteKey(site) === key)) continue;
        const exception = exceptionsByKey.get(key);
        if (exception === undefined) {
          sites.push(provisional);
          continue;
        }
        assert.ok(caller, `exception caller must be a Rust function: ${key}`);
        assert.ok(
          exceptionGroundHolds(exception.kind, caller, allFunctions, source),
          `typed exception ground no longer holds: ${key} (${exception.kind})`,
        );
        seenExceptionKeys.add(key);
        sites.push({
          ...provisional,
          disposition: "none-with-typed-justification",
          exceptionKind: exception.kind,
        });
      }
    }
  }

  assert.deepEqual(
    [...exceptionsByKey.keys()].filter((key) => !seenExceptionKeys.has(key)),
    [],
    "typed exception registry contains stale sites",
  );
  return sites.toSorted(
    (left, right) =>
      compareCodeUnits(left.path, right.path) ||
      left.line - right.line ||
      compareCodeUnits(left.caller, right.caller) ||
      compareCodeUnits(left.callee, right.callee) ||
      left.callOrdinal - right.callOrdinal,
  );
}

function compatibleDeclarationShape(declarations: readonly RustFunction[]): RustFunction {
  const [first] = declarations;
  assert.ok(first, "callee declaration must exist");
  for (const declaration of declarations.slice(1)) {
    assert.equal(
      declaration.identityParameterIndex,
      first.identityParameterIndex,
      `ambiguous identity argument position for ${first.name}`,
    );
    assert.equal(
      declaration.acceptsOptionalIdentityIndex,
      first.acceptsOptionalIdentityIndex,
      `ambiguous identity option shape for ${first.name}`,
    );
  }
  return first;
}

function expressionIsProvablyNone(
  expression: string,
  source: RustSource,
  caller: RustFunction | undefined,
  callStart: number,
  allFunctions: readonly RustFunction[],
  visited = new Set<string>(),
): boolean {
  const compact = stripOuterParentheses(expression.replace(/\s+/gu, " ").trim());
  if (/^(?:(?:std|core)\s*::\s*option\s*::\s*Option\s*::\s*)?None$/u.test(compact)) {
    return true;
  }
  if (/^(?:<[^;]+>\s*::\s*)?(?:Default\s*::\s*)?default\s*\(\s*\)$/u.test(compact)) {
    return true;
  }
  if (/^[A-Za-z_][A-Za-z0-9_]*$/u.test(compact) && caller?.bodyStart !== undefined) {
    const bindingKey = `${source.path}#${caller.name}#${compact}`;
    if (visited.has(bindingKey)) return false;
    visited.add(bindingKey);
    const prefix = source.code.slice(caller.bodyStart + 1, callStart);
    const binding = new RegExp(
      `\\blet\\s+(?:mut\\s+)?${escapeRegExp(compact)}(?:\\s*:[^=;]+)?\\s*=\\s*([^;]+);`,
      "gu",
    );
    const matches = [...prefix.matchAll(binding)];
    const latest = matches.at(-1)?.[1];
    return latest === undefined
      ? false
      : expressionIsProvablyNone(latest, source, caller, callStart, allFunctions, visited);
  }
  const helper = compact.match(
    /^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)$/u,
  )?.[1];
  if (helper !== undefined) {
    return allFunctions
      .filter((entry) => entry.name === helper && entry.returnType.includes("Option"))
      .some((entry) => functionBodyReturnsNone(entry, source, allFunctions, visited));
  }
  return false;
}

function functionBodyReturnsNone(
  helper: RustFunction,
  callSource: RustSource,
  allFunctions: readonly RustFunction[],
  visited: Set<string>,
): boolean {
  if (helper.bodyStart === undefined) return false;
  const helperSource =
    helper.path === callSource.path ? callSource : loadRustSource(helper.path, sourceRef);
  const body = helperSource.code.slice(helper.bodyStart + 1, helper.end - 1).trim();
  const returned = body.match(/^(?:return\s+)?(.+?);?$/su)?.[1] ?? body;
  return expressionIsProvablyNone(
    returned,
    helperSource,
    helper,
    helper.end - 1,
    allFunctions,
    visited,
  );
}

function exceptionGroundHolds(
  kind: ExceptionKind,
  caller: RustFunction,
  allFunctions: readonly RustFunction[],
  source: RustSource,
): boolean {
  if (kind === "tracked-query-boundary") {
    return (
      caller.attributes.includes("salsa::tracked") && caller.identityParameterIndex === undefined
    );
  }
  if (kind === "test-reference-control") {
    const body =
      caller.bodyStart === undefined ? "" : source.code.slice(caller.bodyStart + 1, caller.end - 1);
    return (
      caller.cfgTest &&
      (/\bassert(?:_eq|_ne)?!\s*\(/u.test(body) ||
        /(?:precomputed|reference|unthreaded|parity)/u.test(caller.name))
    );
  }
  if (kind === "legacy-public-compatibility") {
    if (caller.visibility !== "public" || caller.identityParameterIndex !== undefined) return false;
    return allFunctions.some(
      (candidate) =>
        candidate.path === caller.path &&
        candidate.visibility === "public" &&
        candidate.identityParameterIndex !== undefined &&
        candidate.name !== caller.name &&
        (candidate.name.startsWith(`${caller.name}_`) ||
          caller.name.startsWith(
            `${candidate.name.replace(/_(?:with|and)_identity_index.*$/u, "")}`,
          )),
    );
  }
  return false;
}

function buildCensus(
  acceptingFunctions: readonly RustFunction[],
  exceptionRegistry: readonly TypedException[],
  sites: readonly CallerSite[],
): CallerCensus {
  const acceptingFunctionKeys = acceptingFunctions
    .map((entry) => `${entry.path}#${entry.name}`)
    .toSorted();
  const normalizedExceptions = [...exceptionRegistry].toSorted((left, right) =>
    compareCodeUnits(left.siteKey, right.siteKey),
  );
  const summary = {
    acceptingFunctionCount: acceptingFunctionKeys.length,
    callSiteCount: sites.length,
    threadedCount: sites.filter((site) => site.disposition === "threaded").length,
    justifiedNoneCount: sites.filter((site) => site.disposition === "none-with-typed-justification")
      .length,
    violationCount: sites.filter((site) => site.disposition === "violation").length,
  };
  return {
    schemaVersion: "0",
    product: "omena-query.resolver-identity-index-caller-census",
    policy: {
      sourceAuthority: "git-ls-files-rust-crates",
      owningCheck: "rust/omena-resolver/identity-index-callers",
      packageScript: "check:rust-omena-resolver-identity-index-callers",
      allowedExceptionKinds: [
        "legacy-public-compatibility",
        "tracked-query-boundary",
        "test-reference-control",
      ],
    },
    acceptingFunctions: acceptingFunctionKeys,
    typedExceptions: normalizedExceptions,
    sites,
    summary,
    siteDigest: `sha256:${createHash("sha256").update(JSON.stringify(sites)).digest("hex")}`,
  };
}

function runScannerSelfTests(): void {
  const fixture = loadSyntheticSource(disguisedNoneFixture(), "<selftest>");
  const sites = deriveCallerSites(
    [fixture],
    fixture.functions,
    fixture.functions.filter((entry) => entry.identityParameterIndex !== undefined),
    [],
  );
  assert.deepEqual(
    sites
      .filter((site) => site.identityFlow === "none")
      .map((site) => `${site.caller}->${site.callee}`)
      .toSorted(),
    [
      "helper_default_caller->consume_identity_index",
      "local_alias_caller->consume_identity_index",
      "private_adapter->consume_identity_index",
      "private_adapter_caller->private_adapter",
    ],
    "scanner must resolve disguised None flows and callers of private unthreaded adapters",
  );
  assert.equal(sites.filter((site) => site.disposition === "violation").length, 4);
  assert.equal(
    sites.some((site) => site.caller === "literal_decoy"),
    false,
    "comments and literals must not create call sites",
  );
  assert.equal(
    privateItemVisibleFrom(
      "rust/crates/example-crate/src/style.rs",
      "rust/crates/example-crate/examples/consumer.rs",
    ),
    false,
    "a private library item must not be projected into a separate example crate",
  );
  assert.equal(
    privateItemVisibleFrom(
      "rust/crates/example-crate/examples/consumer.rs",
      "rust/crates/example-crate/examples/consumer/helper.rs",
    ),
    true,
    "an example crate's private item must remain visible to its child module",
  );
  assert.equal(
    privateItemVisibleFrom(
      "rust/crates/example-crate/examples/consumer.rs",
      "rust/crates/example-crate/examples/sibling.rs",
    ),
    false,
    "private items must not leak between separate example crates",
  );
  assert.equal(
    privateItemVisibleFrom(
      "rust/crates/example-crate/src/lib.rs",
      "rust/crates/example-crate/build.rs",
    ),
    false,
    "a private library item must not be projected into the build-script crate",
  );
}

function disguisedNoneFixture(): string {
  return `
struct OmenaResolverStyleModuleConfirmationIdentityIndexV0;
fn consume_identity_index(
  identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) { let _ = identity_index; }
fn default_identity_index() -> Option<&'static OmenaResolverStyleModuleConfirmationIdentityIndexV0> {
  Default::default()
}
fn local_alias_caller() {
  let hidden_identity = None;
  consume_identity_index(hidden_identity);
}
fn helper_default_caller() {
  consume_identity_index(default_identity_index());
}
fn private_adapter() {
  consume_identity_index(None);
}
fn private_adapter_caller() {
  private_adapter();
}
fn literal_decoy() {
  let _ = "consume_identity_index(None)";
  // consume_identity_index(None);
}
`;
}

function stableSiteKey(
  site: Pick<CallerSite, "path" | "caller" | "callee" | "callOrdinal">,
): string {
  return `${site.path}#${site.caller}#${site.callee}#${site.callOrdinal}`;
}

function enclosingFunction(
  functions: readonly RustFunction[],
  position: number,
): RustFunction | undefined {
  return functions
    .filter(
      (entry) =>
        entry.bodyStart !== undefined && entry.bodyStart < position && position < entry.end,
    )
    .toSorted((left, right) => left.end - left.start - (right.end - right.start))[0];
}

function itemPrefixStart(code: string, functionStart: number): number {
  const prefix = code.slice(0, functionStart);
  return Math.max(prefix.lastIndexOf("}"), prefix.lastIndexOf(";"), prefix.lastIndexOf("{")) + 1;
}

function findFunctionBodyStart(code: string, start: number): number {
  let parenDepth = 0;
  let bracketDepth = 0;
  let angleDepth = 0;
  for (let index = start; index < code.length; index += 1) {
    const character = code[index];
    if (character === "(") parenDepth += 1;
    else if (character === ")") parenDepth -= 1;
    else if (character === "[") bracketDepth += 1;
    else if (character === "]") bracketDepth -= 1;
    else if (character === "<") angleDepth += 1;
    else if (character === ">" && angleDepth > 0) angleDepth -= 1;
    else if (character === "{" && parenDepth === 0 && bracketDepth === 0 && angleDepth === 0)
      return index;
    else if (character === ";" && parenDepth === 0 && bracketDepth === 0 && angleDepth === 0)
      return -1;
  }
  return -1;
}

function findNextStructuralCharacter(code: string, start: number, expected: string): number {
  const found = code.indexOf(expected, start);
  const itemEnd = Math.min(
    ...[code.indexOf(";", start), code.indexOf("{", start)].filter((index) => index >= 0),
  );
  return found >= 0 && (!Number.isFinite(itemEnd) || found < itemEnd) ? found : -1;
}

function matchingDelimiter(source: string, opening: number, open: string, close: string): number {
  if (opening < 0 || source[opening] !== open) return -1;
  let depth = 0;
  for (let index = opening; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    else if (source[index] === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function topLevelArguments(body: string): string[] {
  if (body.trim().length === 0) return [];
  const argumentsList: string[] = [];
  let start = 0;
  let parenDepth = 0;
  let bracketDepth = 0;
  let braceDepth = 0;
  let angleDepth = 0;
  for (let index = 0; index < body.length; index += 1) {
    const character = body[index];
    if (character === "(") parenDepth += 1;
    else if (character === ")") parenDepth -= 1;
    else if (character === "[") bracketDepth += 1;
    else if (character === "]") bracketDepth -= 1;
    else if (character === "{") braceDepth += 1;
    else if (character === "}") braceDepth -= 1;
    else if (character === "<") angleDepth += 1;
    else if (character === ">" && angleDepth > 0) angleDepth -= 1;
    else if (
      character === "," &&
      parenDepth === 0 &&
      bracketDepth === 0 &&
      braceDepth === 0 &&
      angleDepth === 0
    ) {
      argumentsList.push(body.slice(start, index));
      start = index + 1;
    }
  }
  argumentsList.push(body.slice(start));
  return argumentsList;
}

function topLevelColon(source: string): number {
  let angleDepth = 0;
  let parenDepth = 0;
  for (let index = 0; index < source.length; index += 1) {
    if (source[index] === "<") angleDepth += 1;
    else if (source[index] === ">" && angleDepth > 0) angleDepth -= 1;
    else if (source[index] === "(") parenDepth += 1;
    else if (source[index] === ")") parenDepth -= 1;
    else if (
      source[index] === ":" &&
      source[index + 1] !== ":" &&
      angleDepth === 0 &&
      parenDepth === 0
    )
      return index;
  }
  return -1;
}

function previousNonWhitespace(source: string, position: number): string | undefined {
  return source.slice(0, position).match(/\S(?=\s*$)/u)?.[0];
}

function valueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  return index < 0 ? undefined : process.argv[index + 1];
}

function compactExpression(expression: string): string {
  return expression.replace(/\s+/gu, " ").trim();
}

function compareCodeUnits(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function privateItemVisibleFrom(declarationPath: string, callerPath: string): boolean {
  const declarationModule = rustModuleKey(declarationPath);
  const callerModule = rustModuleKey(callerPath);
  return callerModule === declarationModule || callerModule.startsWith(`${declarationModule}::`);
}

function rustModuleKey(sourcePath: string): string {
  if (sourcePath.startsWith("<")) return sourcePath;
  const buildScript = sourcePath.match(/^rust\/crates\/([^/]+)\/build\.rs$/u);
  if (buildScript) return `${buildScript[1]}::$build`;
  const match = sourcePath.match(
    /^rust\/crates\/([^/]+)\/(src|examples|tests|benches)\/(.+)\.rs$/u,
  );
  assert.ok(match, `Rust source must live below a supported crate target: ${sourcePath}`);
  const [, crateName, targetKind, relative] = match;
  const segments = relative.split("/");
  const file = segments.pop();
  assert.ok(file, `Rust source file missing: ${sourcePath}`);
  if (file !== "lib" && file !== "main" && file !== "mod") segments.push(file);
  return [crateName, `$${targetKind}`, ...segments].join("::");
}

function stripOuterParentheses(expression: string): string {
  let current = expression;
  while (current.startsWith("(") && current.endsWith(")")) {
    const close = matchingDelimiter(current, 0, "(", ")");
    if (close !== current.length - 1) break;
    current = current.slice(1, -1).trim();
  }
  return current;
}

function lineNumberAt(source: string, position: number): number {
  return source.slice(0, position).split("\n").length;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function maskRustNonCode(source: string): string {
  const chars = source.split("");
  const mask = (start: number, end: number) => {
    for (let index = start; index < end; index += 1) {
      if (chars[index] !== "\n" && chars[index] !== "\r") chars[index] = " ";
    }
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      const end = newline < 0 ? source.length : newline;
      mask(index, end);
      index = end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let cursor = index + 2;
      let depth = 1;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith("*/", cursor)) {
          depth -= 1;
          cursor += 2;
        } else cursor += 1;
      }
      mask(index, cursor);
      index = cursor;
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|r)(?<hashes>#+)?"/u);
    if (raw?.groups !== undefined) {
      const hashes = raw.groups.hashes ?? "";
      const closing = `"${hashes}`;
      const closeAt = source.indexOf(closing, index + raw[0].length);
      const end = closeAt < 0 ? source.length : closeAt + closing.length;
      mask(index, end);
      index = end;
      continue;
    }
    if (source[index] === '"') {
      let cursor = index + 1;
      while (cursor < source.length) {
        if (source[cursor] === "\\") cursor += 2;
        else if (source[cursor] === '"') {
          cursor += 1;
          break;
        } else cursor += 1;
      }
      mask(index, cursor);
      index = cursor;
      continue;
    }
    if (source[index] === "'") {
      let cursor = index + 1;
      if (source[cursor] === "\\") cursor += 2;
      else cursor += source.codePointAt(cursor)! > 0xffff ? 2 : 1;
      if (source[cursor] === "'") {
        cursor += 1;
        mask(index, cursor);
        index = cursor;
        continue;
      }
    }
    index += 1;
  }
  return chars.join("");
}
