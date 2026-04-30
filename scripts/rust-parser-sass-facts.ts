import type { Range } from "@css-module-explainer/shared";
import type { SassSymbolDeclHIR } from "../server/engine-core-ts/src/core/hir/style-types";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";

export interface ParserSassSeedFactsV0 {
  readonly variableDeclNames: readonly string[];
  readonly symbolDeclFacts?: readonly unknown[];
  readonly variableParameterNames: readonly string[];
  readonly variableRefNames: readonly string[];
  readonly selectorsWithVariableRefsNames: readonly string[];
  readonly selectorsWithResolvedVariableRefsNames: readonly string[];
  readonly selectorsWithUnresolvedVariableRefsNames: readonly string[];
  readonly mixinDeclNames: readonly string[];
  readonly mixinIncludeNames: readonly string[];
  readonly selectorsWithMixinIncludesNames: readonly string[];
  readonly selectorsWithResolvedMixinIncludesNames: readonly string[];
  readonly selectorsWithUnresolvedMixinIncludesNames: readonly string[];
  readonly functionDeclNames: readonly string[];
  readonly functionCallNames: readonly string[];
  readonly selectorsWithFunctionCallsNames: readonly string[];
  readonly selectorSymbolFacts: readonly ParserSassSelectorSymbolFactV0[];
  readonly moduleUseSources: readonly string[];
  readonly moduleUseEdges: readonly ParserSassModuleUseFactV0[];
  readonly moduleForwardSources: readonly string[];
  readonly moduleImportSources: readonly string[];
  readonly sameFileResolution: ParserSassSameFileResolutionFactsV0;
}

export interface ParserSassModuleUseFactV0 {
  readonly source: string;
  readonly namespaceKind: "default" | "alias" | "wildcard";
  readonly namespace: string | null;
}

export interface ParserSassSameFileResolutionFactsV0 {
  readonly resolvedVariableRefNames: readonly string[];
  readonly unresolvedVariableRefNames: readonly string[];
  readonly resolvedMixinIncludeNames: readonly string[];
  readonly unresolvedMixinIncludeNames: readonly string[];
  readonly resolvedFunctionCallNames: readonly string[];
}

export interface ParserByteSpanV0 {
  readonly start: number;
  readonly end: number;
}

export interface ParserPositionV0 {
  readonly line: number;
  readonly character: number;
}

export interface ParserRangeV0 {
  readonly start: ParserPositionV0;
  readonly end: ParserPositionV0;
}

export interface ParserSassSelectorSymbolFactV0 {
  readonly selectorName: string;
  readonly symbolKind: "variable" | "mixin" | "function";
  readonly name: string;
  readonly namespace: string | null;
  readonly role: "reference" | "include" | "call";
  readonly resolution: "resolved" | "unresolved" | "external";
  readonly byteSpan: ParserByteSpanV0;
  readonly range: ParserRangeV0;
}

export function deriveSassSummary(
  source: string,
  filePath = "/f.module.scss",
): ParserSassSeedFactsV0 {
  const variableDeclNames = [...source.matchAll(/(^|[{\s;])\$([A-Za-z_-][A-Za-z0-9_-]*)\s*:/g)].map(
    (match) => match[2]!,
  );
  const variableRefMatches = findSassVariableReferenceMatches(source);
  const variableRefNames = variableRefMatches.map((match) => match.name);
  const mixinDeclNames = [...source.matchAll(/@mixin\s+([A-Za-z_-][A-Za-z0-9_-]*)/g)].map(
    (match) => match[1]!,
  );
  const mixinIncludeNames = [
    ...source.matchAll(/@include\s+(?:([A-Za-z_-][A-Za-z0-9_-]*)\.)?([A-Za-z_-][A-Za-z0-9_-]*)/g),
  ]
    .filter((match) => !match[1])
    .map((match) => match[2]!);
  const functionDeclNames = [...source.matchAll(/@function\s+([A-Za-z_-][A-Za-z0-9_-]*)/g)].map(
    (match) => match[1]!,
  );
  const functionCallNames = functionDeclNames.flatMap((name) => {
    const callPattern = new RegExp(`\\b${escapeRegExp(name)}\\s*\\(`, "g");
    const sameFileCalls = [...source.matchAll(callPattern)].filter(
      (match) => !isSassModuleQualifiedReference(source, match.index ?? 0),
    );
    return sameFileCalls.length > 1 ? [name] : [];
  });
  const variableParameterNames = [
    ...source.matchAll(/@(mixin|function)\s+[A-Za-z_-][A-Za-z0-9_-]*\(([^)]*)\)/g),
  ].flatMap((match) =>
    [...match[2]!.matchAll(/\$([A-Za-z_-][A-Za-z0-9_-]*)/g)].map((paramMatch) => paramMatch[1]!),
  );
  const sourceForAtRule = (name: "use" | "forward" | "import") =>
    [...source.matchAll(new RegExp(`@${name}\\s+([^;{]+)`, "g"))].flatMap((match) =>
      [...match[1]!.matchAll(/["']([^"']+)["']/g)].map((sourceMatch) => sourceMatch[1]!),
    );
  const sortedVariableDeclNames = uniqueSorted(variableDeclNames);
  const sortedVariableParameterNames = uniqueSorted(variableParameterNames);
  const sortedVariableRefNames = uniqueSorted(variableRefNames);
  const sortedMixinDeclNames = uniqueSorted(mixinDeclNames);
  const sortedMixinIncludeNames = uniqueSorted(mixinIncludeNames);
  const sortedFunctionDeclNames = uniqueSorted(functionDeclNames);
  const sortedFunctionCallNames = uniqueSorted(functionCallNames);
  const selectorAttachments = deriveSassSelectorAttachments(source, filePath, {
    variableDeclNames: sortedVariableDeclNames,
    variableParameterNames: sortedVariableParameterNames,
    mixinDeclNames: sortedMixinDeclNames,
    functionDeclNames: sortedFunctionDeclNames,
  });

  return {
    variableDeclNames: sortedVariableDeclNames,
    variableParameterNames: sortedVariableParameterNames,
    variableRefNames: sortedVariableRefNames,
    selectorsWithVariableRefsNames: selectorAttachments.selectorsWithVariableRefsNames,
    selectorsWithResolvedVariableRefsNames:
      selectorAttachments.selectorsWithResolvedVariableRefsNames,
    selectorsWithUnresolvedVariableRefsNames:
      selectorAttachments.selectorsWithUnresolvedVariableRefsNames,
    mixinDeclNames: sortedMixinDeclNames,
    mixinIncludeNames: sortedMixinIncludeNames,
    selectorsWithMixinIncludesNames: selectorAttachments.selectorsWithMixinIncludesNames,
    selectorsWithResolvedMixinIncludesNames:
      selectorAttachments.selectorsWithResolvedMixinIncludesNames,
    selectorsWithUnresolvedMixinIncludesNames:
      selectorAttachments.selectorsWithUnresolvedMixinIncludesNames,
    functionDeclNames: sortedFunctionDeclNames,
    functionCallNames: sortedFunctionCallNames,
    selectorsWithFunctionCallsNames: selectorAttachments.selectorsWithFunctionCallsNames,
    selectorSymbolFacts: selectorAttachments.selectorSymbolFacts,
    moduleUseSources: uniqueSorted([...sourceForAtRule("use"), ...sourceForAtRule("import")]),
    moduleUseEdges: uniqueSortedUseEdges([
      ...deriveSassUseEdges(source),
      ...deriveSassImportUseEdges(source),
    ]),
    moduleForwardSources: uniqueSorted(sourceForAtRule("forward")),
    moduleImportSources: uniqueSorted(sourceForAtRule("import")),
    sameFileResolution: deriveSameFileResolution(source, filePath, {
      variableRefNames: uniqueSorted(
        variableRefMatches.filter((match) => match.namespace === null).map((match) => match.name),
      ),
      mixinDeclNames: sortedMixinDeclNames,
      mixinIncludeNames: sortedMixinIncludeNames,
      functionDeclNames: sortedFunctionDeclNames,
      functionCallNames: sortedFunctionCallNames,
    }),
  };
}

function deriveSassSelectorAttachments(
  source: string,
  filePath: string,
  input: {
    readonly variableDeclNames: readonly string[];
    readonly variableParameterNames: readonly string[];
    readonly mixinDeclNames: readonly string[];
    readonly functionDeclNames: readonly string[];
  },
): Pick<
  ParserSassSeedFactsV0,
  | "selectorsWithVariableRefsNames"
  | "selectorsWithResolvedVariableRefsNames"
  | "selectorsWithUnresolvedVariableRefsNames"
  | "selectorsWithMixinIncludesNames"
  | "selectorsWithResolvedMixinIncludesNames"
  | "selectorsWithUnresolvedMixinIncludesNames"
  | "selectorsWithFunctionCallsNames"
  | "selectorSymbolFacts"
> {
  const sassVariableDecls = parseStyleDocument(source, filePath).sassSymbolDecls.filter(
    (decl) => decl.symbolKind === "variable",
  );
  const mixinTargets = new Set(input.mixinDeclNames);
  const selectorsWithVariableRefsNames: string[] = [];
  const selectorsWithResolvedVariableRefsNames: string[] = [];
  const selectorsWithUnresolvedVariableRefsNames: string[] = [];
  const selectorsWithMixinIncludesNames: string[] = [];
  const selectorsWithResolvedMixinIncludesNames: string[] = [];
  const selectorsWithUnresolvedMixinIncludesNames: string[] = [];
  const selectorsWithFunctionCallsNames: string[] = [];
  const selectorSymbolFacts: ParserSassSelectorSymbolFactV0[] = [];

  for (const match of source.matchAll(/\.([A-Za-z_-][A-Za-z0-9_-]*)[^{]*\{([^{}]*)\}/g)) {
    const selectorName = match[1]!;
    const body = match[2]!;
    const ruleStart = match.index ?? 0;
    const bodyStart = ruleStart + match[0].indexOf("{") + 1;
    const variableRefMatches = [...body.matchAll(/\$([A-Za-z_-][A-Za-z0-9_-]*)/g)]
      .map((variableMatch) => {
        const start = bodyStart + (variableMatch.index ?? 0);
        const namespace = sassModuleQualifiedNamespaceBefore(source, start);
        return {
          name: variableMatch[1]!,
          namespace,
          codeUnitStart: start,
          codeUnitEnd: start + variableMatch[0].length,
          location: symbolLocation(source, start, start + variableMatch[0].length),
        };
      })
      .filter((variableMatch) => !isSassVariableDeclarationLike(source, variableMatch.codeUnitEnd));
    const variableRefs = variableRefMatches.map((variableMatch) => variableMatch.name);
    if (variableRefs.length > 0) {
      selectorsWithVariableRefsNames.push(selectorName);
      if (
        variableRefMatches.some(
          ({ name, namespace, location }) =>
            namespace === null &&
            resolveSassVariableReference(sassVariableDecls, name, location.range) === "resolved",
        )
      ) {
        selectorsWithResolvedVariableRefsNames.push(selectorName);
      }
      if (
        variableRefMatches.some(
          ({ name, namespace, location }) =>
            namespace === null &&
            resolveSassVariableReference(sassVariableDecls, name, location.range) === "unresolved",
        )
      ) {
        selectorsWithUnresolvedVariableRefsNames.push(selectorName);
      }
      for (const { name, namespace, location } of variableRefMatches) {
        const resolution =
          namespace === null
            ? resolveSassVariableReference(sassVariableDecls, name, location.range)
            : "external";
        selectorSymbolFacts.push({
          selectorName,
          symbolKind: "variable",
          name,
          namespace,
          role: "reference",
          resolution,
          ...location,
        });
      }
    }

    const mixinIncludeMatches = [
      ...body.matchAll(/@include\s+(?:([A-Za-z_-][A-Za-z0-9_-]*)\.)?([A-Za-z_-][A-Za-z0-9_-]*)/g),
    ].map((includeMatch) => {
      const namespace = includeMatch[1] ?? null;
      const name = includeMatch[2]!;
      const start = bodyStart + (includeMatch.index ?? 0) + includeMatch[0].indexOf(name);
      return {
        name,
        namespace,
        codeUnitEnd: start + name.length,
        location: symbolLocation(source, start, start + name.length),
      };
    });
    const mixinIncludes = mixinIncludeMatches
      .filter((includeMatch) => includeMatch.namespace === null)
      .map((includeMatch) => includeMatch.name);
    if (mixinIncludes.length > 0) {
      selectorsWithMixinIncludesNames.push(selectorName);
      if (mixinIncludes.some((name) => mixinTargets.has(name))) {
        selectorsWithResolvedMixinIncludesNames.push(selectorName);
      }
      if (mixinIncludes.some((name) => !mixinTargets.has(name))) {
        selectorsWithUnresolvedMixinIncludesNames.push(selectorName);
      }
    }
    if (mixinIncludeMatches.length > 0) {
      if (mixinIncludes.length === 0) {
        selectorsWithMixinIncludesNames.push(selectorName);
      }
      for (const { name, namespace, location } of mixinIncludeMatches) {
        selectorSymbolFacts.push({
          selectorName,
          symbolKind: "mixin",
          name,
          namespace,
          role: "include",
          resolution:
            namespace === null ? (mixinTargets.has(name) ? "resolved" : "unresolved") : "external",
          ...location,
        });
      }
    }

    const functionCalls = input.functionDeclNames.flatMap((name) =>
      [...body.matchAll(new RegExp(`\\b(${escapeRegExp(name)})\\s*\\(`, "g"))]
        .map((callMatch) => {
          const callName = callMatch[1]!;
          const start = bodyStart + (callMatch.index ?? 0) + callMatch[0].indexOf(callName);
          return {
            name: callName,
            codeUnitStart: start,
            location: symbolLocation(source, start, start + callName.length),
          };
        })
        .filter((callMatch) => !isSassModuleQualifiedReference(source, callMatch.codeUnitStart)),
    );
    const moduleFunctionCalls = [
      ...body.matchAll(/\b([A-Za-z_-][A-Za-z0-9_-]*)\.([A-Za-z_-][A-Za-z0-9_-]*)\s*\(/g),
    ].map((callMatch) => {
      const namespace = callMatch[1]!;
      const name = callMatch[2]!;
      const start = bodyStart + (callMatch.index ?? 0) + namespace.length + 1;
      return {
        name,
        namespace,
        location: symbolLocation(source, start, start + name.length),
      };
    });
    if (functionCalls.length > 0 || moduleFunctionCalls.length > 0) {
      selectorsWithFunctionCallsNames.push(selectorName);
      selectorSymbolFacts.push(
        ...functionCalls.map(({ name, location }) => ({
          selectorName,
          symbolKind: "function" as const,
          name,
          namespace: null,
          role: "call" as const,
          resolution: "resolved" as const,
          byteSpan: location.byteSpan,
          range: location.range,
        })),
        ...moduleFunctionCalls.map(({ name, namespace, location }) => ({
          selectorName,
          symbolKind: "function" as const,
          name,
          namespace,
          role: "call" as const,
          resolution: "external" as const,
          byteSpan: location.byteSpan,
          range: location.range,
        })),
      );
    }
  }

  return {
    selectorsWithVariableRefsNames: uniqueSorted(selectorsWithVariableRefsNames),
    selectorsWithResolvedVariableRefsNames: uniqueSorted(selectorsWithResolvedVariableRefsNames),
    selectorsWithUnresolvedVariableRefsNames: uniqueSorted(
      selectorsWithUnresolvedVariableRefsNames,
    ),
    selectorsWithMixinIncludesNames: uniqueSorted(selectorsWithMixinIncludesNames),
    selectorsWithResolvedMixinIncludesNames: uniqueSorted(selectorsWithResolvedMixinIncludesNames),
    selectorsWithUnresolvedMixinIncludesNames: uniqueSorted(
      selectorsWithUnresolvedMixinIncludesNames,
    ),
    selectorsWithFunctionCallsNames: uniqueSorted(selectorsWithFunctionCallsNames),
    selectorSymbolFacts: uniqueSortedSelectorSymbolFacts(selectorSymbolFacts),
  };
}

function resolveSassVariableReference(
  decls: readonly SassSymbolDeclHIR[],
  name: string,
  range: ParserRangeV0,
): "resolved" | "unresolved" {
  const matchingDecls = decls.filter((decl) => decl.name === name);
  if (matchingDecls.length === 0) return "unresolved";

  const localDecl = matchingDecls
    .filter((decl) => !isFileScopeSassVariableDecl(decl))
    .filter((decl) => rangeContains(decl.ruleRange, range))
    .toSorted(compareSassDeclScopeSpecificity)[0];
  if (localDecl) return "resolved";
  return matchingDecls.some(isFileScopeSassVariableDecl) ? "resolved" : "unresolved";
}

function isSassVariableDeclarationLike(source: string, end: number): boolean {
  return /\S/.exec(source.slice(end))?.[0] === ":";
}

function isFileScopeSassVariableDecl(decl: SassSymbolDeclHIR): boolean {
  return (
    decl.range.start.line === decl.ruleRange.start.line &&
    decl.range.start.character === decl.ruleRange.start.character
  );
}

function compareSassDeclScopeSpecificity(a: SassSymbolDeclHIR, b: SassSymbolDeclHIR): number {
  const sizeCompare = rangeSize(a.ruleRange) - rangeSize(b.ruleRange);
  if (sizeCompare !== 0) return sizeCompare;
  const lineCompare = b.range.start.line - a.range.start.line;
  if (lineCompare !== 0) return lineCompare;
  return b.range.start.character - a.range.start.character;
}

function rangeSize(range: Range): number {
  return (
    (range.end.line - range.start.line) * 1_000_000 + (range.end.character - range.start.character)
  );
}

function comparePosition(
  left: { readonly line: number; readonly character: number },
  right: { readonly line: number; readonly character: number },
): number {
  if (left.line !== right.line) return left.line - right.line;
  return left.character - right.character;
}

function rangeContains(outer: Range, inner: ParserRangeV0): boolean {
  return (
    comparePosition(outer.start, inner.start) <= 0 && comparePosition(outer.end, inner.end) >= 0
  );
}

function deriveSameFileResolution(
  source: string,
  filePath: string,
  input: {
    readonly variableRefNames: readonly string[];
    readonly mixinDeclNames: readonly string[];
    readonly mixinIncludeNames: readonly string[];
    readonly functionDeclNames: readonly string[];
    readonly functionCallNames: readonly string[];
  },
): ParserSassSameFileResolutionFactsV0 {
  const sassVariableDecls = parseStyleDocument(source, filePath).sassSymbolDecls.filter(
    (decl) => decl.symbolKind === "variable",
  );
  const variableRefMatches = findSassVariableReferenceMatches(source);
  const mixinTargets = new Set(input.mixinDeclNames);
  const functionTargets = new Set(input.functionDeclNames);

  return {
    resolvedVariableRefNames: uniqueSorted(
      variableRefMatches
        .filter(
          ({ name, namespace, location }) =>
            namespace === null &&
            resolveSassVariableReference(sassVariableDecls, name, location.range) === "resolved",
        )
        .map((match) => match.name),
    ),
    unresolvedVariableRefNames: uniqueSorted(
      variableRefMatches
        .filter(
          ({ name, namespace, location }) =>
            namespace === null &&
            resolveSassVariableReference(sassVariableDecls, name, location.range) === "unresolved",
        )
        .map((match) => match.name),
    ),
    resolvedMixinIncludeNames: input.mixinIncludeNames.filter((name) => mixinTargets.has(name)),
    unresolvedMixinIncludeNames: input.mixinIncludeNames.filter((name) => !mixinTargets.has(name)),
    resolvedFunctionCallNames: input.functionCallNames.filter((name) => functionTargets.has(name)),
  };
}

function findSassVariableReferenceMatches(source: string): Array<{
  readonly name: string;
  readonly namespace: string | null;
  readonly codeUnitStart: number;
  readonly location: Pick<ParserSassSelectorSymbolFactV0, "byteSpan" | "range">;
}> {
  return [...source.matchAll(/\$([A-Za-z_-][A-Za-z0-9_-]*)/g)]
    .map((match) => {
      const start = match.index ?? 0;
      return {
        name: match[1]!,
        namespace: sassModuleQualifiedNamespaceBefore(source, start),
        codeUnitStart: start,
        codeUnitEnd: start + match[0].length,
        location: symbolLocation(source, start, start + match[0].length),
      };
    })
    .filter((match) => !isSassVariableDeclarationLike(source, match.codeUnitEnd));
}

function isSassModuleQualifiedReference(source: string, start: number): boolean {
  return sassModuleQualifiedNamespaceBefore(source, start) !== null;
}

function sassModuleQualifiedNamespaceBefore(source: string, start: number): string | null {
  if (start <= 1 || source[start - 1] !== ".") return null;
  const match = /([A-Za-z_-][A-Za-z0-9_-]*)$/.exec(source.slice(0, start - 1));
  return match?.[1] ?? null;
}

function deriveSassUseEdges(source: string): ParserSassModuleUseFactV0[] {
  return [...source.matchAll(/@use\s+([^;{]+)/g)].flatMap((match) => {
    const params = match[1]!;
    const alias = parseSassUseAlias(params);
    return [...params.matchAll(/["']([^"']+)["']/g)].map((sourceMatch) => {
      const sourceValue = sourceMatch[1]!;
      if (alias === "*") {
        return {
          source: sourceValue,
          namespaceKind: "wildcard",
          namespace: null,
        };
      }
      if (alias !== undefined) {
        return {
          source: sourceValue,
          namespaceKind: "alias",
          namespace: alias,
        };
      }
      return {
        source: sourceValue,
        namespaceKind: "default",
        namespace: defaultSassNamespaceForSource(sourceValue),
      };
    });
  });
}

function deriveSassImportUseEdges(source: string): ParserSassModuleUseFactV0[] {
  return [...source.matchAll(/@import\s+([^;{]+)/g)].flatMap((match) =>
    [...match[1]!.matchAll(/["']([^"']+)["']/g)].map((sourceMatch) => ({
      source: sourceMatch[1]!,
      namespaceKind: "wildcard",
      namespace: null,
    })),
  );
}

function parseSassUseAlias(params: string): string | undefined {
  const withoutQuotedSource = params.replaceAll(/["'](?:\\.|[^"'])*["']/g, " ");
  return /\bas\s+(\*|[A-Za-z_-][A-Za-z0-9_-]*)/.exec(withoutQuotedSource)?.[1];
}

function defaultSassNamespaceForSource(source: string): string | null {
  const clean = source.split(/[?#]/, 1)[0]!.replace(/\/+$/g, "");
  const segment = clean.split("/").at(-1) ?? clean;
  const packageSegment = segment.split(":").at(-1) ?? segment;
  const stem = packageSegment.includes(".")
    ? packageSegment.slice(0, packageSegment.lastIndexOf("."))
    : packageSegment;
  const namespace = stem.startsWith("_") ? stem.slice(1) : stem;
  return /^[A-Za-z_-][A-Za-z0-9_-]*$/.test(namespace) ? namespace : null;
}

function uniqueSorted(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted((left, right) => left.localeCompare(right));
}

function uniqueSortedUseEdges(
  values: readonly ParserSassModuleUseFactV0[],
): ParserSassModuleUseFactV0[] {
  const byKey = new Map(values.map((value) => [JSON.stringify(value), value]));
  return [...byKey.values()].toSorted((left, right) => {
    const sourceCompare = left.source.localeCompare(right.source);
    if (sourceCompare !== 0) return sourceCompare;
    const kindCompare = left.namespaceKind.localeCompare(right.namespaceKind);
    if (kindCompare !== 0) return kindCompare;
    return (left.namespace ?? "").localeCompare(right.namespace ?? "");
  });
}

function uniqueSortedSelectorSymbolFacts(
  values: readonly ParserSassSelectorSymbolFactV0[],
): ParserSassSelectorSymbolFactV0[] {
  const byKey = new Map(values.map((value) => [JSON.stringify(value), value]));
  return [...byKey.values()].toSorted((left, right) => {
    const selectorCompare = left.selectorName.localeCompare(right.selectorName);
    if (selectorCompare !== 0) return selectorCompare;
    const kindCompare = left.symbolKind.localeCompare(right.symbolKind);
    if (kindCompare !== 0) return kindCompare;
    const nameCompare = left.name.localeCompare(right.name);
    if (nameCompare !== 0) return nameCompare;
    const roleCompare = left.role.localeCompare(right.role);
    if (roleCompare !== 0) return roleCompare;
    const resolutionCompare = left.resolution.localeCompare(right.resolution);
    if (resolutionCompare !== 0) return resolutionCompare;
    const spanStartCompare = left.byteSpan.start - right.byteSpan.start;
    if (spanStartCompare !== 0) return spanStartCompare;
    return left.byteSpan.end - right.byteSpan.end;
  });
}

function symbolLocation(
  source: string,
  startCodeUnitOffset: number,
  endCodeUnitOffset: number,
): Pick<ParserSassSelectorSymbolFactV0, "byteSpan" | "range"> {
  return {
    byteSpan: {
      start: utf8ByteOffsetForCodeUnitOffset(source, startCodeUnitOffset),
      end: utf8ByteOffsetForCodeUnitOffset(source, endCodeUnitOffset),
    },
    range: {
      start: positionAtCodeUnitOffset(source, startCodeUnitOffset),
      end: positionAtCodeUnitOffset(source, endCodeUnitOffset),
    },
  };
}

function utf8ByteOffsetForCodeUnitOffset(source: string, codeUnitOffset: number): number {
  return new TextEncoder().encode(source.slice(0, codeUnitOffset)).length;
}

function positionAtCodeUnitOffset(source: string, codeUnitOffset: number): ParserPositionV0 {
  let line = 0;
  let character = 0;
  let index = 0;

  while (index < codeUnitOffset) {
    const codePoint = source.codePointAt(index);
    if (codePoint === undefined) break;
    const codeUnitLength = codePoint > 0xffff ? 2 : 1;
    const value = source.slice(index, index + codeUnitLength);
    if (value === "\n") {
      line += 1;
      character = 0;
    } else {
      character += codeUnitLength;
    }
    index += codeUnitLength;
  }

  return { line, character };
}

function escapeRegExp(value: string): string {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
