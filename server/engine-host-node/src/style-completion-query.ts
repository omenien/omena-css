import type { Range } from "@css-module-explainer/shared";
import type {
  CustomPropertyDeclHIR,
  SassSymbolDeclHIR,
  StylePreprocessorSymbolSyntax,
  StyleDocumentHIR,
  ValueDeclHIR,
  ValueImportHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { StyleDependencyGraph } from "../../engine-core-ts/src/core/semantic";
import {
  listCustomPropertyModuleUseDeclTargets,
  listSassModuleExportedSymbols,
  resolveValueTarget,
  resolveSassModuleUseTarget,
  type SassModulePathAliasResolver,
  type SassModuleResolutionOptions,
} from "../../engine-core-ts/src/core/query";
import { rangeContains } from "../../engine-core-ts/src/core/util/range-utils";
import type { StyleDesignTokenDeclarationCandidateReadModel } from "./style-design-token-ranking-query";

export interface StyleCompletionItem {
  readonly label: string;
  readonly detail: string;
  readonly insertText: string;
  readonly filterText: string;
  readonly replacementRange: Range;
  readonly sourceRange?: Range;
  readonly sourceFilePath?: string;
  readonly symbolSyntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolDeclHIR["symbolKind"] | "customProperty" | "value";
}

type StyleCompletionContext =
  | {
      readonly symbolKind: "customProperty";
      readonly prefix: string;
      readonly replacementStartCharacter: number;
    }
  | {
      readonly symbolKind: "variable";
      readonly symbolSyntax?: StylePreprocessorSymbolSyntax;
      readonly prefix: string;
      readonly replacementStartCharacter: number;
    }
  | {
      readonly symbolKind: "mixin" | "function";
      readonly symbolSyntax?: undefined;
      readonly prefix: string;
      readonly replacementStartCharacter: number;
    }
  | {
      readonly symbolKind: "valueOrFunction";
      readonly symbolSyntax?: undefined;
      readonly prefix: string;
      readonly replacementStartCharacter: number;
    };

interface SassSymbolCompletionDecl {
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolDeclHIR["symbolKind"];
  readonly name: string;
  readonly range: Range;
  readonly ruleRange: Range;
}

interface CustomPropertyCompletionDecl extends Pick<
  CustomPropertyDeclHIR,
  "name" | "range" | "ruleRange" | "context"
> {
  readonly symbolKind: "customProperty";
  readonly sourceFilePath?: string;
}

interface ValueCompletionDecl extends Pick<ValueDeclHIR, "name" | "range" | "ruleRange"> {
  readonly symbolKind: "value";
  readonly sourceKind: "decl" | "import";
  readonly sourceFilePath?: string;
  readonly from?: string;
  readonly importedName?: string;
}

type CustomPropertyCompletionSourceRank = 1 | 2 | 3;

export function resolveStyleCompletionItems(args: {
  readonly content: string;
  readonly line: number;
  readonly character: number;
  readonly styleDocument: StyleDocumentHIR;
  readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
  readonly aliasResolver?: SassModulePathAliasResolver;
  readonly styleDependencyGraph?: StyleDependencyGraph;
  readonly readFile?: (filePath: string) => string | null;
  readonly designTokenDeclarationCandidates?: readonly StyleDesignTokenDeclarationCandidateReadModel[];
}): readonly StyleCompletionItem[] {
  const lineText = readLine(args.content, args.line);
  const linePrefix = lineText.slice(0, args.character);
  const context = readStyleCompletionContext(linePrefix);
  if (!context) return [];

  const replacementRange: Range = {
    start: { line: args.line, character: context.replacementStartCharacter },
    end: { line: args.line, character: args.character },
  };
  if (context.symbolKind === "customProperty") {
    return collectCustomPropertyCompletionDecls(args)
      .filter((decl) => decl.name.startsWith(context.prefix))
      .map((decl) => toCustomPropertyCompletionItem(decl, replacementRange))
      .toSorted((a, b) => a.filterText.localeCompare(b.filterText));
  }

  if (context.symbolKind === "valueOrFunction") {
    const valueItems = collectValueCompletionDecls(args)
      .filter((decl) => decl.name.startsWith(context.prefix))
      .map((decl) => toValueCompletionItem(decl, replacementRange));
    const functionItems = collectSassSymbolCompletionDecls(
      readSassSymbolCompletionDecls(args),
      "function",
      undefined,
      args.line,
      args.character,
    )
      .filter((decl) => completionFilterText(decl).startsWith(context.prefix))
      .map((decl) => toStyleCompletionItem(decl, replacementRange));
    return [...valueItems, ...functionItems].toSorted(compareCompletionItems);
  }

  const candidates = collectSassSymbolCompletionDecls(
    readSassSymbolCompletionDecls(args),
    context.symbolKind,
    context.symbolSyntax,
    args.line,
    args.character,
  );
  return candidates
    .filter((decl) => completionFilterText(decl).startsWith(context.prefix))
    .map((decl) => toStyleCompletionItem(decl, replacementRange))
    .toSorted((a, b) => a.filterText.localeCompare(b.filterText));
}

function readStyleCompletionContext(linePrefix: string): StyleCompletionContext | null {
  const customProperty = /var\(\s*(--[\p{L}\p{N}\p{M}_-]*)?$/u.exec(linePrefix);
  if (customProperty) {
    const prefix = customProperty[1] ?? "";
    return {
      symbolKind: "customProperty",
      prefix,
      replacementStartCharacter: linePrefix.length - prefix.length,
    };
  }

  const variable = /(\$[A-Za-z0-9_-]*)$/u.exec(linePrefix);
  if (variable) {
    return {
      symbolKind: "variable",
      symbolSyntax: "sass",
      prefix: variable[1]!.slice(1),
      replacementStartCharacter: linePrefix.length - variable[1]!.length,
    };
  }

  const lessVariable = /(@[A-Za-z0-9_-]*)$/u.exec(linePrefix);
  if (lessVariable && isSassFunctionValueContext(linePrefix)) {
    return {
      symbolKind: "variable",
      symbolSyntax: "less",
      prefix: lessVariable[1]!.slice(1),
      replacementStartCharacter: linePrefix.length - lessVariable[1]!.length,
    };
  }

  const include = /@include\s+([A-Za-z_-][A-Za-z0-9_-]*)?$/u.exec(linePrefix);
  if (include) {
    const prefix = include[1] ?? "";
    return {
      symbolKind: "mixin",
      prefix,
      replacementStartCharacter: linePrefix.length - prefix.length,
    };
  }

  const functionName = /([A-Za-z_-][A-Za-z0-9_-]*)$/u.exec(linePrefix);
  if (!functionName) return null;
  if (isStyleValueCompletionContext(linePrefix)) {
    return {
      symbolKind: "valueOrFunction",
      prefix: functionName[1]!,
      replacementStartCharacter: linePrefix.length - functionName[1]!.length,
    };
  }
  if (!isSassFunctionValueContext(linePrefix)) return null;
  return {
    symbolKind: "function",
    prefix: functionName[1]!,
    replacementStartCharacter: linePrefix.length - functionName[1]!.length,
  };
}

function isSassFunctionValueContext(linePrefix: string): boolean {
  const lastColon = linePrefix.lastIndexOf(":");
  const lastSemicolon = linePrefix.lastIndexOf(";");
  if (lastColon > lastSemicolon) return true;
  return /@return\s+[\w-]*$/u.test(linePrefix);
}

function isStyleValueCompletionContext(linePrefix: string): boolean {
  if (isValueDeclarationValueContext(linePrefix)) return true;
  const lastColon = linePrefix.lastIndexOf(":");
  const lastSemicolon = linePrefix.lastIndexOf(";");
  return lastColon > lastSemicolon;
}

function isValueDeclarationValueContext(linePrefix: string): boolean {
  return /@value\s+[\p{L}_-][\p{L}\p{N}\p{M}_-]*\s*:\s*[\p{L}\p{N}\p{M}_-]*$/u.test(linePrefix);
}

function collectSassSymbolCompletionDecls(
  decls: readonly SassSymbolCompletionDecl[],
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  symbolSyntax: StylePreprocessorSymbolSyntax | undefined,
  line: number,
  character: number,
): readonly SassSymbolCompletionDecl[] {
  const seen = new Set<string>();
  const results: SassSymbolCompletionDecl[] = [];
  for (const decl of decls) {
    if (decl.symbolKind !== symbolKind) continue;
    if ((decl.syntax ?? "sass") !== (symbolSyntax ?? "sass")) continue;
    if (decl.symbolKind === "variable" && !isVariableDeclVisible(decl, line, character)) continue;
    if (seen.has(decl.name)) continue;
    seen.add(decl.name);
    results.push(decl);
  }
  return results;
}

function collectValueCompletionDecls(args: {
  readonly styleDocument: StyleDocumentHIR;
  readonly line: number;
  readonly character: number;
  readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
}): readonly ValueCompletionDecl[] {
  const results: ValueCompletionDecl[] = [];
  const seen = new Set<string>();
  const containingDeclName = readContainingValueDeclName(
    args.styleDocument,
    args.line,
    args.character,
  );

  const push = (decl: ValueCompletionDecl) => {
    if (seen.has(decl.name)) return;
    seen.add(decl.name);
    results.push(decl);
  };

  for (const valueDecl of args.styleDocument.valueDecls) {
    if (valueDecl.name === containingDeclName) continue;
    push({
      symbolKind: "value",
      sourceKind: "decl",
      name: valueDecl.name,
      range: valueDecl.range,
      ruleRange: valueDecl.ruleRange,
      sourceFilePath: args.styleDocument.filePath,
    });
  }
  for (const valueImport of args.styleDocument.valueImports) {
    push(valueCompletionDeclFromImport(args, valueImport));
  }
  return results;
}

function readContainingValueDeclName(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): string | null {
  for (const valueDecl of styleDocument.valueDecls) {
    if (rangeContains(valueDecl.ruleRange, line, character)) return valueDecl.name;
  }
  return null;
}

function valueCompletionDeclFromImport(
  args: {
    readonly styleDocument: StyleDocumentHIR;
    readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
  },
  valueImport: ValueImportHIR,
): ValueCompletionDecl {
  const resolved = args.styleDocumentForPath
    ? resolveValueTarget(
        args.styleDocumentForPath,
        args.styleDocument.filePath,
        args.styleDocument,
        valueImport.name,
      )
    : null;
  return {
    symbolKind: "value",
    sourceKind: "import",
    name: valueImport.name,
    range: resolved?.valueDecl.range ?? valueImport.range,
    ruleRange: resolved?.valueDecl.ruleRange ?? valueImport.ruleRange,
    sourceFilePath: resolved?.filePath ?? args.styleDocument.filePath,
    from: valueImport.from,
    importedName: valueImport.importedName,
  };
}

function collectCustomPropertyCompletionDecls(args: {
  readonly content: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly line: number;
  readonly character: number;
  readonly styleDependencyGraph?: StyleDependencyGraph;
  readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
  readonly aliasResolver?: SassModulePathAliasResolver;
  readonly readFile?: (filePath: string) => string | null;
  readonly designTokenDeclarationCandidates?: readonly StyleDesignTokenDeclarationCandidateReadModel[];
}): readonly CustomPropertyCompletionDecl[] {
  const referenceContext = readCustomPropertyCompletionReferenceContext(args);
  let sourceOrder = 0;
  const byName = new Map<
    string,
    {
      readonly decl: CustomPropertyCompletionDecl;
      readonly score: number;
      readonly sourceRank: CustomPropertyCompletionSourceRank;
      readonly sourceOrder: number;
    }
  >();
  const pushDecl = (
    decl: Pick<CustomPropertyDeclHIR, "name" | "range" | "ruleRange" | "context">,
    sourceFilePath?: string,
    sourceRank: CustomPropertyCompletionSourceRank = 1,
  ) => {
    const candidate = {
      ...(sourceFilePath ? { sourceFilePath } : {}),
      symbolKind: "customProperty" as const,
      ...decl,
    };
    const score = scoreCustomPropertyCompletionContext(candidate.context, referenceContext);
    if (score === Number.NEGATIVE_INFINITY) return;
    const previous = byName.get(candidate.name);
    const next = { decl: candidate, score, sourceRank, sourceOrder: sourceOrder++ };
    if (previous && compareCustomPropertyCompletionCandidate(previous, next) >= 0) return;
    byName.set(candidate.name, next);
  };

  const localDecls =
    args.styleDocument.customPropertyDecls.length > 0
      ? args.styleDocument.customPropertyDecls
      : collectFallbackCustomPropertyCompletionDecls(args.content);
  for (const decl of localDecls) pushDecl(decl, args.styleDocument.filePath, 3);
  if (args.styleDocumentForPath) {
    for (const target of listCustomPropertyModuleUseDeclTargets(
      args.styleDocumentForPath,
      args.styleDocument.filePath,
      args.styleDocument,
      args.aliasResolver,
      sassModuleResolutionOptions(args.readFile),
    )) {
      pushDecl(target.decl, target.filePath, 2);
    }
  }
  for (const candidate of args.designTokenDeclarationCandidates ?? []) {
    pushDecl(
      customPropertyCompletionDeclFromDesignTokenCandidate(candidate),
      candidate.filePath,
      candidate.filePath === args.styleDocument.filePath ? 3 : 2,
    );
  }
  for (const decl of args.styleDependencyGraph?.getAllCustomPropertyDecls() ?? []) {
    pushDecl(decl, decl.filePath, 1);
  }
  return [...byName.values()].map((entry) => entry.decl);
}

function customPropertyCompletionDeclFromDesignTokenCandidate(
  candidate: StyleDesignTokenDeclarationCandidateReadModel,
): Pick<CustomPropertyDeclHIR, "name" | "range" | "ruleRange" | "context"> {
  return {
    name: candidate.name,
    range: candidate.range,
    ruleRange: candidate.range,
    context: {
      containerKind: "rule",
      selectorText: candidate.selectorContexts.at(-1) ?? ":root",
      atRuleName: null,
      atRuleParams: null,
      wrapperAtRules: [],
    },
  };
}

function compareCustomPropertyCompletionCandidate(
  left: {
    readonly score: number;
    readonly sourceRank: CustomPropertyCompletionSourceRank;
    readonly sourceOrder: number;
  },
  right: {
    readonly score: number;
    readonly sourceRank: CustomPropertyCompletionSourceRank;
    readonly sourceOrder: number;
  },
): number {
  return (
    left.score - right.score ||
    left.sourceRank - right.sourceRank ||
    left.sourceOrder - right.sourceOrder
  );
}

function collectFallbackCustomPropertyCompletionDecls(
  content: string,
): readonly Pick<CustomPropertyDeclHIR, "name" | "range" | "ruleRange" | "context">[] {
  const decls: Pick<CustomPropertyDeclHIR, "name" | "range" | "ruleRange" | "context">[] = [];
  const lines = content.split(/\r?\n/u);
  const customPropertyDecl = /(?:^|[;{])\s*(--[\p{L}\p{N}\p{M}_-]+)\s*:/gu;
  for (const [line, text] of lines.entries()) {
    customPropertyDecl.lastIndex = 0;
    for (const match of text.matchAll(customPropertyDecl)) {
      const name = match[1]!;
      const character = match.index + match[0]!.lastIndexOf(name);
      const range = {
        start: { line, character },
        end: { line, character: character + name.length },
      };
      decls.push({
        name,
        range,
        ruleRange: range,
        context: {
          containerKind: "rule",
          selectorText: ":root",
          atRuleName: null,
          atRuleParams: null,
          wrapperAtRules: [],
        },
      });
    }
  }
  return decls;
}

function readCustomPropertyCompletionReferenceContext(args: {
  readonly styleDocument: StyleDocumentHIR;
  readonly line: number;
  readonly character: number;
}): CustomPropertyDeclHIR["context"] | undefined {
  const refAtCursor = args.styleDocument.customPropertyRefs.find((ref) =>
    rangeContains(ref.range, args.line, args.character),
  );
  if (refAtCursor) return refAtCursor.context;

  const containingSelector = args.styleDocument.selectors
    .filter((selector) => rangeContains(selector.ruleRange, args.line, args.character))
    .toSorted((left, right) => rangeSize(left.ruleRange) - rangeSize(right.ruleRange))[0];
  if (!containingSelector) return undefined;
  return {
    containerKind: "rule",
    selectorText: containingSelector.fullSelector,
    atRuleName: null,
    atRuleParams: null,
    wrapperAtRules: containingSelector.context?.wrapperAtRules ?? [],
  };
}

function scoreCustomPropertyCompletionContext(
  declContext: CustomPropertyDeclHIR["context"],
  referenceContext: CustomPropertyDeclHIR["context"] | undefined,
): number {
  if (!referenceContext) return 0;
  if (!declWrapperContextMatches(declContext, referenceContext)) {
    return Number.NEGATIVE_INFINITY;
  }
  let score = 0;
  if (declContext.selectorText) {
    if (referenceContext.selectorText === declContext.selectorText) {
      score += 100;
    } else if (
      referenceContext.selectorText &&
      referenceContext.selectorText.includes(declContext.selectorText)
    ) {
      score += 80;
    } else if (
      selectorContextTokensMatch(referenceContext.selectorText, declContext.selectorText)
    ) {
      score += 70;
    } else if (declContext.selectorText === ":root") {
      score += 10;
    } else {
      return Number.NEGATIVE_INFINITY;
    }
  }
  score += declContext.wrapperAtRules.length * 20;
  return score;
}

function selectorContextTokensMatch(
  referenceSelectorText: string | null | undefined,
  declSelectorText: string,
): boolean {
  if (!referenceSelectorText) return false;
  const tokens = [...declSelectorText.matchAll(/(?:\[[^\]]+\]|[.#][A-Za-z0-9_-]+)/gu)].map(
    (match) => match[0],
  );
  return tokens.length > 0 && tokens.every((token) => referenceSelectorText.includes(token));
}

function declWrapperContextMatches(
  declContext: CustomPropertyDeclHIR["context"],
  referenceContext: CustomPropertyDeclHIR["context"],
): boolean {
  return declContext.wrapperAtRules.every((declWrapper) =>
    referenceContext.wrapperAtRules.some(
      (refWrapper) =>
        refWrapper.name === declWrapper.name && refWrapper.params === declWrapper.params,
    ),
  );
}

function rangeSize(range: Range): number {
  return (range.end.line - range.start.line) * 10_000 + range.end.character - range.start.character;
}

function readSassSymbolCompletionDecls(args: {
  readonly content: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
  readonly aliasResolver?: SassModulePathAliasResolver;
  readonly readFile?: (filePath: string) => string | null;
}): readonly SassSymbolCompletionDecl[] {
  const { styleDocument, content } = args;
  const localDecls =
    styleDocument.sassSymbolDecls.length > 0
      ? styleDocument.sassSymbolDecls
      : collectFallbackSassSymbolCompletionDecls(content);
  if (!args.styleDocumentForPath) return localDecls;
  return [
    ...localDecls,
    ...collectWildcardSassSymbolCompletionDecls(
      styleDocument,
      args.styleDocumentForPath,
      args.aliasResolver,
      sassModuleResolutionOptions(args.readFile),
    ),
  ];
}

function collectWildcardSassSymbolCompletionDecls(
  styleDocument: StyleDocumentHIR,
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  aliasResolver?: SassModulePathAliasResolver,
  options: { readonly readFile?: (filePath: string) => string | null } = {},
): readonly SassSymbolCompletionDecl[] {
  const decls: SassSymbolCompletionDecl[] = [];
  const seen = new Set<string>();
  for (const moduleUse of styleDocument.sassModuleUses) {
    if (moduleUse.namespaceKind !== "wildcard") continue;
    const target = resolveSassModuleUseTarget(
      styleDocumentForPath,
      styleDocument.filePath,
      moduleUse,
      aliasResolver,
      sassModuleResolutionOptions(options.readFile),
    );
    if (!target) continue;
    for (const exportedTarget of listSassModuleExportedSymbols(
      styleDocumentForPath,
      target.filePath,
      target.styleDocument,
      aliasResolver,
      new Set(),
      sassModuleResolutionOptions(options.readFile),
    )) {
      const key = `${exportedTarget.decl.syntax ?? "sass"}:${exportedTarget.decl.symbolKind}:${
        exportedTarget.exportedName
      }`;
      if (seen.has(key)) continue;
      seen.add(key);
      decls.push({ ...exportedTarget.decl, name: exportedTarget.exportedName });
    }
  }
  return decls;
}

function sassModuleResolutionOptions(
  readFile: ((filePath: string) => string | null) | undefined,
): SassModuleResolutionOptions {
  return readFile ? { readFile } : {};
}

function collectFallbackSassSymbolCompletionDecls(
  content: string,
): readonly SassSymbolCompletionDecl[] {
  const decls: SassSymbolCompletionDecl[] = [];
  const lines = content.split(/\r?\n/u);
  for (const [line, text] of lines.entries()) {
    const variable = /^(\s*)\$([A-Za-z_-][A-Za-z0-9_-]*)\s*:/u.exec(text);
    if (variable) {
      decls.push(makeFallbackDecl("variable", variable[2]!, line, variable[1]!.length));
      continue;
    }

    const lessVariable = /^(\s*)@([A-Za-z_-][A-Za-z0-9_-]*)\s*:/u.exec(text);
    if (lessVariable) {
      decls.push(
        makeFallbackDecl("variable", lessVariable[2]!, line, lessVariable[1]!.length, "less"),
      );
      continue;
    }

    const callable = /^(\s*)@(mixin|function)\s+([A-Za-z_-][A-Za-z0-9_-]*)/u.exec(text);
    if (!callable) continue;
    const symbolKind = callable[2] === "mixin" ? "mixin" : "function";
    const character = callable[1]!.length + `@${callable[2]} `.length;
    decls.push(makeFallbackDecl(symbolKind, callable[3]!, line, character));
  }
  return decls;
}

function makeFallbackDecl(
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
  line: number,
  character: number,
  syntax?: StylePreprocessorSymbolSyntax,
): SassSymbolCompletionDecl {
  const range = {
    start: { line, character },
    end: {
      line,
      character: character + (symbolKind === "variable" ? name.length + 1 : name.length),
    },
  };
  return { ...(syntax ? { syntax } : {}), symbolKind, name, range, ruleRange: range };
}

function toCustomPropertyCompletionItem(
  decl: CustomPropertyCompletionDecl,
  replacementRange: Range,
): StyleCompletionItem {
  return {
    label: decl.name,
    detail: "CSS custom property",
    insertText: decl.name,
    filterText: decl.name,
    replacementRange,
    sourceRange: decl.range,
    ...(decl.sourceFilePath ? { sourceFilePath: decl.sourceFilePath } : {}),
    symbolKind: "customProperty",
  };
}

function toValueCompletionItem(
  decl: ValueCompletionDecl,
  replacementRange: Range,
): StyleCompletionItem {
  return {
    label: decl.name,
    detail: valueCompletionDetail(decl),
    insertText: decl.name,
    filterText: decl.name,
    replacementRange,
    sourceRange: decl.range,
    ...(decl.sourceFilePath ? { sourceFilePath: decl.sourceFilePath } : {}),
    symbolKind: "value",
  };
}

function valueCompletionDetail(decl: ValueCompletionDecl): string {
  if (decl.sourceKind === "decl") return "CSS Modules @value";
  if (decl.importedName && decl.importedName !== decl.name) {
    return `CSS Modules @value import (${decl.importedName} from ${decl.from ?? "module"})`;
  }
  return `CSS Modules @value import from ${decl.from ?? "module"}`;
}

function isVariableDeclVisible(
  decl: SassSymbolCompletionDecl,
  line: number,
  character: number,
): boolean {
  if (startsAtSamePosition(decl.range, decl.ruleRange)) return true;
  return rangeContains(decl.ruleRange, line, character);
}

function startsAtSamePosition(a: Range, b: Range): boolean {
  return a.start.line === b.start.line && a.start.character === b.start.character;
}

function toStyleCompletionItem(
  decl: SassSymbolCompletionDecl,
  replacementRange: Range,
): StyleCompletionItem {
  return {
    label: completionLabel(decl),
    detail: completionDetail(decl),
    insertText: completionInsertText(decl),
    filterText: completionFilterText(decl),
    replacementRange,
    ...(decl.syntax ? { symbolSyntax: decl.syntax } : {}),
    symbolKind: decl.symbolKind,
  };
}

function completionLabel(decl: SassSymbolCompletionDecl): string {
  if (decl.syntax === "less") return `@${decl.name}`;
  return decl.symbolKind === "variable" ? `$${decl.name}` : decl.name;
}

function completionInsertText(decl: SassSymbolCompletionDecl): string {
  return completionLabel(decl);
}

function completionFilterText(decl: SassSymbolCompletionDecl): string {
  return decl.name;
}

function completionDetail(decl: SassSymbolCompletionDecl): string {
  switch (decl.symbolKind) {
    case "variable":
      return decl.syntax === "less" ? "Less variable" : "Sass variable";
    case "mixin":
      return "Sass mixin";
    case "function":
      return "Sass function";
  }
}

function compareCompletionItems(a: StyleCompletionItem, b: StyleCompletionItem): number {
  return (
    a.filterText.localeCompare(b.filterText) ||
    completionItemSymbolRank(a.symbolKind) - completionItemSymbolRank(b.symbolKind)
  );
}

function completionItemSymbolRank(symbolKind: StyleCompletionItem["symbolKind"]): number {
  switch (symbolKind) {
    case "value":
    case "customProperty":
      return 0;
    case "variable":
      return 1;
    case "function":
    case "mixin":
      return 2;
  }
}

function readLine(content: string, line: number): string {
  return content.split(/\r?\n/u)[line] ?? "";
}
