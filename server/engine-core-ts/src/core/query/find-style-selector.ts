import path from "node:path";
import type {
  AnimationNameRefHIR,
  CustomPropertyDeclHIR,
  CustomPropertyRefHIR,
  KeyframesDeclHIR,
  SassModuleForwardHIR,
  SassModuleMemberRefHIR,
  SassModuleUseHIR,
  SassSymbolDeclHIR,
  SassSymbolKind,
  SassSymbolOccurrenceHIR,
  SelectorDeclHIR,
  StyleDocumentHIR,
  ValueDeclHIR,
  ValueImportHIR,
  ValueRefHIR,
} from "../hir/style-types";
import { rangeContains } from "../util/range-utils";
import type { ComposesClassToken, ComposesRef } from "@omena/shared";

export function findSelectorAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): SelectorDeclHIR | null {
  for (const selector of styleDocument.selectors) {
    const hitRange = selector.bemSuffix?.rawTokenRange ?? selector.range;
    if (rangeContains(hitRange, line, character)) return selector;
  }
  return null;
}

export function findCanonicalSelector(
  styleDocument: StyleDocumentHIR,
  selector: SelectorDeclHIR,
): SelectorDeclHIR {
  if (selector.viewKind === "canonical") return selector;
  return (
    styleDocument.selectors.find(
      (candidate) =>
        candidate.canonicalName === selector.canonicalName && candidate.viewKind === "canonical",
    ) ?? selector
  );
}

export function findCanonicalSelectorsByName(
  styleDocument: StyleDocumentHIR,
  viewName: string,
): readonly SelectorDeclHIR[] {
  const matches = styleDocument.selectors.filter((selector) => selector.name === viewName);
  if (matches.length === 0) return [];
  const canonicalNames = new Set(matches.map((selector) => selector.canonicalName));
  const canonicalSelectors = styleDocument.selectors.filter(
    (selector) => selector.viewKind === "canonical" && canonicalNames.has(selector.canonicalName),
  );
  return canonicalSelectors.length > 0 ? canonicalSelectors : matches;
}

export function listCanonicalSelectors(
  styleDocument: StyleDocumentHIR,
): readonly SelectorDeclHIR[] {
  const canonicalSelectors: SelectorDeclHIR[] = [];
  const emitted = new Set<string>();

  for (const selector of styleDocument.selectors) {
    if (emitted.has(selector.canonicalName)) continue;
    emitted.add(selector.canonicalName);
    canonicalSelectors.push(findCanonicalSelector(styleDocument, selector));
  }

  return canonicalSelectors;
}

export interface ComposesTokenHit {
  readonly selector: SelectorDeclHIR;
  readonly ref: ComposesRef;
  readonly token: ComposesClassToken;
}

export interface ResolvedComposesTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly selector: SelectorDeclHIR;
}

export function findComposesTokenAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): ComposesTokenHit | null {
  for (const selector of styleDocument.selectors) {
    for (const ref of selector.composes) {
      for (const token of ref.classTokens ?? []) {
        if (rangeContains(token.range, line, character)) {
          return { selector, ref, token };
        }
      }
    }
  }
  return null;
}

export function resolveComposesTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  hit: ComposesTokenHit | null,
): ResolvedComposesTarget | null {
  if (!hit || hit.ref.fromGlobal) return null;

  const targetFilePath = hit.ref.from
    ? path.resolve(path.dirname(styleFilePath), hit.ref.from)
    : styleFilePath;
  const targetDocument = styleDocumentForPath(targetFilePath);
  if (!targetDocument) return null;

  const selector =
    targetDocument.selectors.find(
      (candidate) =>
        candidate.canonicalName === hit.token.className && candidate.viewKind === "canonical",
    ) ??
    targetDocument.selectors.find((candidate) => candidate.canonicalName === hit.token.className);
  if (!selector) return null;

  return {
    filePath: targetDocument.filePath,
    styleDocument: targetDocument,
    selector: findCanonicalSelector(targetDocument, selector),
  };
}

export function findKeyframesAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): KeyframesDeclHIR | null {
  for (const keyframes of styleDocument.keyframes) {
    if (rangeContains(keyframes.range, line, character)) return keyframes;
  }
  return null;
}

export function findAnimationNameRefAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): AnimationNameRefHIR | null {
  for (const ref of styleDocument.animationNameRefs) {
    if (rangeContains(ref.range, line, character)) return ref;
  }
  return null;
}

export function findKeyframesByName(
  styleDocument: StyleDocumentHIR,
  name: string,
): KeyframesDeclHIR | null {
  return styleDocument.keyframes.find((keyframes) => keyframes.name === name) ?? null;
}

export function listAnimationNameRefs(
  styleDocument: StyleDocumentHIR,
  name: string,
): readonly AnimationNameRefHIR[] {
  return styleDocument.animationNameRefs.filter((ref) => ref.name === name);
}

export function findValueDeclAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): ValueDeclHIR | null {
  for (const valueDecl of styleDocument.valueDecls) {
    if (rangeContains(valueDecl.range, line, character)) return valueDecl;
  }
  return null;
}

export function findValueRefAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): ValueRefHIR | null {
  for (const valueRef of styleDocument.valueRefs) {
    if (rangeContains(valueRef.range, line, character)) return valueRef;
  }
  return null;
}

export function findValueImportAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): ValueImportHIR | null {
  for (const valueImport of styleDocument.valueImports) {
    if (rangeContains(valueImport.range, line, character)) return valueImport;
  }
  return null;
}

export function findSassModuleUseAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): SassModuleUseHIR | null {
  for (const moduleUse of styleDocument.sassModuleUses) {
    if (rangeContains(moduleUse.range, line, character)) return moduleUse;
  }
  return null;
}

export function findSassModuleMemberRefAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): SassModuleMemberRefHIR | null {
  for (const memberRef of styleDocument.sassModuleMemberRefs) {
    if (rangeContains(memberRef.range, line, character)) return memberRef;
  }
  return null;
}

export function findValueDeclByName(
  styleDocument: StyleDocumentHIR,
  name: string,
): ValueDeclHIR | null {
  return styleDocument.valueDecls.find((valueDecl) => valueDecl.name === name) ?? null;
}

export function findValueImportByName(
  styleDocument: StyleDocumentHIR,
  name: string,
): ValueImportHIR | null {
  return styleDocument.valueImports.find((valueImport) => valueImport.name === name) ?? null;
}

export function listValueRefs(
  styleDocument: StyleDocumentHIR,
  name: string,
): readonly ValueRefHIR[] {
  return styleDocument.valueRefs.filter((valueRef) => valueRef.name === name);
}

export function findCustomPropertyDeclAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): CustomPropertyDeclHIR | null {
  for (const decl of styleDocument.customPropertyDecls) {
    if (rangeContains(decl.range, line, character)) return decl;
  }
  return null;
}

export function findCustomPropertyRefAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): CustomPropertyRefHIR | null {
  for (const ref of styleDocument.customPropertyRefs) {
    if (rangeContains(ref.range, line, character)) return ref;
  }
  return null;
}

export function findCustomPropertyDeclByName(
  styleDocument: StyleDocumentHIR,
  name: string,
): CustomPropertyDeclHIR | null {
  for (let index = styleDocument.customPropertyDecls.length - 1; index >= 0; index -= 1) {
    const decl = styleDocument.customPropertyDecls[index]!;
    if (decl.name === name) return decl;
  }
  return null;
}

export function listCustomPropertyRefs(
  styleDocument: StyleDocumentHIR,
  name: string,
): readonly CustomPropertyRefHIR[] {
  return styleDocument.customPropertyRefs.filter((ref) => ref.name === name);
}

export interface CustomPropertyDeclLookup {
  readonly getCustomPropertyDecls: (name: string) => readonly (Pick<
    CustomPropertyDeclHIR,
    "name" | "value" | "range" | "ruleRange" | "context"
  > & {
    readonly filePath: string;
  })[];
}

export interface ResolvedCustomPropertyDeclTarget {
  readonly filePath: string;
  readonly decl: Pick<CustomPropertyDeclHIR, "name" | "value" | "range" | "ruleRange" | "context">;
}

export function listCustomPropertyModuleUseDeclTargets(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): readonly ResolvedCustomPropertyDeclTarget[] {
  const targets: ResolvedCustomPropertyDeclTarget[] = [];
  const seen = new Set<string>();
  for (const moduleUse of styleDocument.sassModuleUses) {
    const target = resolveSassModuleUseTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleUse,
      aliasResolver,
      options,
    );
    if (!target) continue;
    for (const exportedTarget of listCustomPropertyModuleExportedDeclTargets(
      styleDocumentForPath,
      target.filePath,
      target.styleDocument,
      aliasResolver,
      new Set(),
      options,
    )) {
      const { decl } = exportedTarget;
      const filePath = exportedTarget.filePath;
      const key = `${filePath}\u0000${decl.name}`;
      if (seen.has(key)) continue;
      seen.add(key);
      targets.push({ filePath, decl });
    }
  }
  return targets;
}

function listCustomPropertyModuleExportedDeclTargets(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  aliasResolver?: SassModulePathAliasResolver,
  visited: ReadonlySet<string> = new Set(),
  options: SassModuleResolutionOptions = {},
): readonly ResolvedCustomPropertyDeclTarget[] {
  if (visited.has(styleFilePath)) return [];
  const nextVisited = new Set(visited);
  nextVisited.add(styleFilePath);

  const targets = styleDocument.customPropertyDecls.map<ResolvedCustomPropertyDeclTarget>(
    (decl) => ({
      filePath: styleFilePath,
      decl,
    }),
  );

  for (const moduleForward of styleDocument.sassModuleForwards) {
    const forwardTarget = resolveSassModuleForwardTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleForward,
      aliasResolver,
      options,
    );
    if (!forwardTarget) continue;
    targets.push(
      ...listCustomPropertyModuleExportedDeclTargets(
        styleDocumentForPath,
        forwardTarget.filePath,
        forwardTarget.styleDocument,
        aliasResolver,
        nextVisited,
        options,
      ),
    );
  }

  return targets;
}

export function resolveCustomPropertyDeclTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  refOrName: string | Pick<CustomPropertyRefHIR, "name" | "context">,
  workspaceDeclLookup?: CustomPropertyDeclLookup,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): ResolvedCustomPropertyDeclTarget | null {
  const name = typeof refOrName === "string" ? refOrName : refOrName.name;
  const referenceContext = typeof refOrName === "string" ? undefined : refOrName.context;
  const localDecl = pickBestCustomPropertyDecl(
    styleDocument.customPropertyDecls,
    name,
    referenceContext,
  );
  if (localDecl) return { filePath: styleFilePath, decl: localDecl };

  const moduleDecls = listCustomPropertyModuleUseDeclTargets(
    styleDocumentForPath,
    styleFilePath,
    styleDocument,
    aliasResolver,
    options,
  );
  const moduleDecl = pickBestCustomPropertyTarget(moduleDecls, name, referenceContext);
  if (moduleDecl) return moduleDecl;

  const workspaceDecls = workspaceDeclLookup
    ?.getCustomPropertyDecls(name)
    .toSorted((a, b) => a.filePath.localeCompare(b.filePath));
  const workspaceDecl = workspaceDecls
    ? pickBestCustomPropertyTarget(
        workspaceDecls.map((decl) => ({ filePath: decl.filePath, decl })),
        name,
        referenceContext,
      )
    : null;
  return workspaceDecl;
}

function pickBestCustomPropertyTarget(
  targets: readonly ResolvedCustomPropertyDeclTarget[],
  name: string,
  referenceContext?: CustomPropertyRefHIR["context"],
): ResolvedCustomPropertyDeclTarget | null {
  const decl = pickBestCustomPropertyDecl(
    targets.map((target) => target.decl),
    name,
    referenceContext,
  );
  if (!decl) return null;
  return targets.find((target) => target.decl === decl) ?? null;
}

function pickBestCustomPropertyDecl<T extends Pick<CustomPropertyDeclHIR, "name" | "context">>(
  decls: readonly T[],
  name: string,
  referenceContext?: CustomPropertyRefHIR["context"],
): T | null {
  let bestDecl: T | null = null;
  let bestScore = Number.NEGATIVE_INFINITY;
  for (const decl of decls) {
    if (decl.name !== name) continue;
    const score = scoreCustomPropertyContextMatch(decl.context, referenceContext);
    if (score === Number.NEGATIVE_INFINITY) continue;
    if (score < bestScore) continue;
    bestDecl = decl;
    bestScore = score;
  }
  return bestDecl;
}

function scoreCustomPropertyContextMatch(
  declContext: CustomPropertyDeclHIR["context"],
  referenceContext?: CustomPropertyRefHIR["context"],
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
  referenceContext: CustomPropertyRefHIR["context"],
): boolean {
  return declContext.wrapperAtRules.every((declWrapper) =>
    referenceContext.wrapperAtRules.some(
      (refWrapper) =>
        refWrapper.name === declWrapper.name && refWrapper.params === declWrapper.params,
    ),
  );
}

export function findSassSymbolDeclAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): SassSymbolDeclHIR | null {
  for (const decl of styleDocument.sassSymbolDecls) {
    if (rangeContains(decl.range, line, character)) return decl;
  }
  return null;
}

export function findSassSymbolAtCursor(
  styleDocument: StyleDocumentHIR,
  line: number,
  character: number,
): SassSymbolOccurrenceHIR | null {
  for (const symbol of styleDocument.sassSymbols) {
    if (rangeContains(symbol.range, line, character)) return symbol;
  }
  return null;
}

export function findSassSymbolDeclByName(
  styleDocument: StyleDocumentHIR,
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
): SassSymbolDeclHIR | null {
  return (
    styleDocument.sassSymbolDecls.find(
      (decl) => decl.symbolKind === symbolKind && decl.name === name,
    ) ?? null
  );
}

export function findSassSymbolDeclForSymbol(
  styleDocument: StyleDocumentHIR,
  symbol: SassSymbolOccurrenceHIR,
): SassSymbolDeclHIR | null {
  const candidates = styleDocument.sassSymbolDecls.filter(
    (decl) =>
      decl.symbolKind === symbol.symbolKind &&
      decl.name === symbol.name &&
      sassSymbolSyntax(decl) === sassSymbolSyntax(symbol),
  );
  if (candidates.length === 0) return null;
  if (symbol.symbolKind !== "variable") return candidates[0] ?? null;

  const localCandidates = candidates
    .filter((decl) => !isFileScopeSassVariableDecl(decl))
    .filter((decl) =>
      rangeContains(decl.ruleRange, symbol.range.start.line, symbol.range.start.character),
    )
    .toSorted(compareSassDeclScopeSpecificity);
  return localCandidates[0] ?? candidates.find(isFileScopeSassVariableDecl) ?? null;
}

export function listSassSymbols(
  styleDocument: StyleDocumentHIR,
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
): readonly SassSymbolOccurrenceHIR[] {
  return styleDocument.sassSymbols.filter(
    (symbol) => symbol.symbolKind === symbolKind && symbol.name === name,
  );
}

export function listSassSymbolsForDecl(
  styleDocument: StyleDocumentHIR,
  decl: SassSymbolDeclHIR,
): readonly SassSymbolOccurrenceHIR[] {
  return styleDocument.sassSymbols.filter((symbol) => {
    if (
      symbol.symbolKind !== decl.symbolKind ||
      symbol.name !== decl.name ||
      sassSymbolSyntax(symbol) !== sassSymbolSyntax(decl)
    ) {
      return false;
    }
    return findSassSymbolDeclForSymbol(styleDocument, symbol) === decl;
  });
}

export function listSassModuleMemberRefsForMember(
  styleDocument: StyleDocumentHIR,
  memberRef: SassModuleMemberRefHIR,
): readonly SassModuleMemberRefHIR[] {
  return styleDocument.sassModuleMemberRefs.filter(
    (candidate) =>
      candidate.namespace === memberRef.namespace &&
      candidate.symbolKind === memberRef.symbolKind &&
      candidate.name === memberRef.name,
  );
}

function isFileScopeSassVariableDecl(decl: SassSymbolDeclHIR): boolean {
  return (
    decl.symbolKind === "variable" &&
    decl.range.start.line === decl.ruleRange.start.line &&
    decl.range.start.character === decl.ruleRange.start.character
  );
}

function isExportedSassSymbolDecl(decl: SassSymbolDeclHIR): boolean {
  if (sassSymbolSyntax(decl) !== "sass") return false;
  return decl.symbolKind !== "variable" || isFileScopeSassVariableDecl(decl);
}

function sassSymbolSyntax(
  symbol: Pick<SassSymbolDeclHIR | SassSymbolOccurrenceHIR, "syntax">,
): "sass" | "less" {
  return symbol.syntax ?? "sass";
}

function dedupeSassModuleExportedSymbolTargets(
  targets: readonly ResolvedSassModuleExportedSymbolTarget[],
): readonly ResolvedSassModuleExportedSymbolTarget[] {
  const byKey = new Map<string, ResolvedSassModuleExportedSymbolTarget>();
  for (const target of targets) {
    const key = sassExportedSymbolTargetKey(target.filePath, target.decl, target.exportedName);
    if (!byKey.has(key)) byKey.set(key, target);
  }
  return [...byKey.values()];
}

function sassExportedSymbolTargetKey(
  filePath: string,
  decl: SassSymbolDeclHIR,
  exportedName: string,
): string {
  return [
    filePath,
    decl.symbolKind,
    decl.name,
    exportedName,
    decl.range.start.line,
    decl.range.start.character,
    decl.range.end.line,
    decl.range.end.character,
  ].join("\u0000");
}

function compareSassDeclScopeSpecificity(a: SassSymbolDeclHIR, b: SassSymbolDeclHIR): number {
  const sizeCompare = rangeSize(a.ruleRange) - rangeSize(b.ruleRange);
  if (sizeCompare !== 0) return sizeCompare;
  const lineCompare = b.range.start.line - a.range.start.line;
  if (lineCompare !== 0) return lineCompare;
  return b.range.start.character - a.range.start.character;
}

function rangeSize(range: SassSymbolDeclHIR["range"]): number {
  return (
    (range.end.line - range.start.line) * 1_000_000 + (range.end.character - range.start.character)
  );
}

export interface ResolvedValueTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly valueDecl: ValueDeclHIR;
  readonly bindingKind: "local" | "imported";
  readonly valueImport?: ValueImportHIR;
}

export interface SassModulePathAliasResolver {
  resolve(
    specifier: string,
    fileExists?: (candidate: string) => boolean,
    containingFilePath?: string,
  ): string | null;
}

export interface SassModuleResolutionOptions {
  readonly readFile?: (filePath: string) => string | null;
}

export interface ResolvedSassModuleUseTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly moduleUse: SassModuleUseHIR;
}

export interface ResolvedSassModuleForwardTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly moduleForward: SassModuleForwardHIR;
}

export interface ResolvedSassModuleExportedSymbolTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly decl: SassSymbolDeclHIR;
  readonly exportedName: string;
}

export interface ResolvedSassModuleMemberTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly moduleUse: SassModuleUseHIR;
  readonly memberRef: SassModuleMemberRefHIR;
  readonly decl: SassSymbolDeclHIR;
  readonly exportedName: string;
}

export interface ResolvedSassWildcardSymbolTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly moduleUse: SassModuleUseHIR;
  readonly symbol: SassSymbolOccurrenceHIR;
  readonly decl: SassSymbolDeclHIR;
  readonly exportedName: string;
}

const SASS_MODULE_EXTENSIONS = [".scss", ".sass", ".css"] as const;

export function listSassModuleUseCandidatePaths(
  styleFilePath: string,
  moduleUse: Pick<SassModuleUseHIR, "source">,
  aliasResolver?: SassModulePathAliasResolver,
  fileExists?: (candidate: string) => boolean,
  options: SassModuleResolutionOptions = {},
): readonly string[] {
  const basePath = resolveSassModuleBasePath(
    styleFilePath,
    moduleUse.source,
    aliasResolver,
    fileExists,
    options,
  );
  if (!basePath) return [];
  return expandSassModuleCandidatePaths(basePath);
}

export function resolveSassModuleUseTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  moduleUse: SassModuleUseHIR | null,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): ResolvedSassModuleUseTarget | null {
  if (!moduleUse) return null;
  const fileExists = (candidatePath: string): boolean =>
    expandSassModuleCandidatePaths(candidatePath).some(
      (expandedPath) => styleDocumentForPath(expandedPath) !== null,
    );
  for (const candidatePath of listSassModuleUseCandidatePaths(
    styleFilePath,
    moduleUse,
    aliasResolver,
    fileExists,
    options,
  )) {
    const styleDocument = styleDocumentForPath(candidatePath);
    if (!styleDocument) continue;
    return {
      filePath: styleDocument.filePath,
      styleDocument,
      moduleUse,
    };
  }
  return null;
}

export function resolveSassModuleForwardTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  moduleForward: SassModuleForwardHIR | null,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): ResolvedSassModuleForwardTarget | null {
  if (!moduleForward) return null;
  const fileExists = (candidatePath: string): boolean =>
    expandSassModuleCandidatePaths(candidatePath).some(
      (expandedPath) => styleDocumentForPath(expandedPath) !== null,
    );
  for (const candidatePath of listSassModuleUseCandidatePaths(
    styleFilePath,
    moduleForward,
    aliasResolver,
    fileExists,
    options,
  )) {
    const styleDocument = styleDocumentForPath(candidatePath);
    if (!styleDocument) continue;
    return {
      filePath: styleDocument.filePath,
      styleDocument,
      moduleForward,
    };
  }
  return null;
}

export function resolveSassModuleUseTargetFilePath(
  styleFilePath: string,
  moduleUse: SassModuleUseHIR | null,
  aliasResolver: SassModulePathAliasResolver | undefined,
  fileExists: (candidate: string) => boolean,
  options: SassModuleResolutionOptions = {},
): string | null {
  if (!moduleUse) return null;
  const targetExists = (candidatePath: string): boolean =>
    expandSassModuleCandidatePaths(candidatePath).some(fileExists);
  for (const candidatePath of listSassModuleUseCandidatePaths(
    styleFilePath,
    moduleUse,
    aliasResolver,
    targetExists,
    options,
  )) {
    if (fileExists(candidatePath)) return candidatePath;
  }
  return null;
}

export function listSassModuleExportedSymbolTargets(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  symbolKind: SassSymbolDeclHIR["symbolKind"],
  name: string,
  aliasResolver?: SassModulePathAliasResolver,
  visited: ReadonlySet<string> = new Set(),
  options: SassModuleResolutionOptions = {},
): readonly ResolvedSassModuleExportedSymbolTarget[] {
  const visitKey = `${styleFilePath}\u0000${symbolKind}\u0000${name}`;
  if (visited.has(visitKey)) return [];
  const nextVisited = new Set(visited);
  nextVisited.add(visitKey);

  const directTargets = styleDocument.sassSymbolDecls
    .filter(
      (decl) =>
        decl.symbolKind === symbolKind && decl.name === name && isExportedSassSymbolDecl(decl),
    )
    .map<ResolvedSassModuleExportedSymbolTarget>((decl) => ({
      filePath: styleFilePath,
      styleDocument,
      decl,
      exportedName: decl.name,
    }));
  if (directTargets.length > 0) return dedupeSassModuleExportedSymbolTargets(directTargets);

  const forwardedTargets: ResolvedSassModuleExportedSymbolTarget[] = [];
  for (const moduleForward of styleDocument.sassModuleForwards) {
    const forwardedName = unprefixSassModuleForwardExportName(moduleForward, name);
    if (forwardedName === null) continue;
    const forwardTarget = resolveSassModuleForwardTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleForward,
      aliasResolver,
      options,
    );
    if (!forwardTarget) continue;
    const childTargets = listSassModuleExportedSymbolTargets(
      styleDocumentForPath,
      forwardTarget.filePath,
      forwardTarget.styleDocument,
      symbolKind,
      forwardedName,
      aliasResolver,
      nextVisited,
      options,
    );
    forwardedTargets.push(
      ...childTargets.flatMap((target) => {
        const exportedTarget = applySassModuleForwardExportPolicy(moduleForward, target);
        return exportedTarget ? [exportedTarget] : [];
      }),
    );
  }

  return dedupeSassModuleExportedSymbolTargets(forwardedTargets);
}

export function listSassModuleExportedSymbols(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  aliasResolver?: SassModulePathAliasResolver,
  visited: ReadonlySet<string> = new Set(),
  options: SassModuleResolutionOptions = {},
): readonly ResolvedSassModuleExportedSymbolTarget[] {
  if (visited.has(styleFilePath)) return [];
  const nextVisited = new Set(visited);
  nextVisited.add(styleFilePath);

  const targets: ResolvedSassModuleExportedSymbolTarget[] = styleDocument.sassSymbolDecls
    .filter(isExportedSassSymbolDecl)
    .map((decl) => ({
      filePath: styleFilePath,
      styleDocument,
      decl,
      exportedName: decl.name,
    }));

  for (const moduleForward of styleDocument.sassModuleForwards) {
    const forwardTarget = resolveSassModuleForwardTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleForward,
      aliasResolver,
      options,
    );
    if (!forwardTarget) continue;
    const childTargets = listSassModuleExportedSymbols(
      styleDocumentForPath,
      forwardTarget.filePath,
      forwardTarget.styleDocument,
      aliasResolver,
      nextVisited,
      options,
    );
    targets.push(
      ...childTargets.flatMap((target) => {
        const exportedTarget = applySassModuleForwardExportPolicy(moduleForward, target);
        return exportedTarget ? [exportedTarget] : [];
      }),
    );
  }

  return dedupeSassModuleExportedSymbolTargets(targets);
}

function unprefixSassModuleForwardExportName(
  moduleForward: SassModuleForwardHIR,
  name: string,
): string | null {
  if (!moduleForward.prefix) return name;
  return name.startsWith(moduleForward.prefix) ? name.slice(moduleForward.prefix.length) : null;
}

function applySassModuleForwardExportPolicy(
  moduleForward: SassModuleForwardHIR,
  target: ResolvedSassModuleExportedSymbolTarget,
): ResolvedSassModuleExportedSymbolTarget | null {
  if (
    !isSassModuleForwardExportVisible(moduleForward, target.decl.symbolKind, target.exportedName)
  ) {
    return null;
  }
  return {
    ...target,
    exportedName: `${moduleForward.prefix}${target.exportedName}`,
  };
}

function isSassModuleForwardExportVisible(
  moduleForward: SassModuleForwardHIR,
  symbolKind: SassSymbolKind,
  name: string,
): boolean {
  if (moduleForward.visibilityKind === "all") return true;
  const matched = moduleForward.visibilityMembers.some(
    (member) =>
      member.name === name &&
      (member.symbolKind === "variable"
        ? symbolKind === "variable"
        : symbolKind === "mixin" || symbolKind === "function"),
  );
  return moduleForward.visibilityKind === "show" ? matched : !matched;
}

export function resolveSassModuleMemberRefTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  memberRef: SassModuleMemberRefHIR | null,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): ResolvedSassModuleMemberTarget | null {
  if (!memberRef) return null;
  for (const moduleUse of findSassModuleUsesForNamespace(styleDocument, memberRef.namespace)) {
    const moduleTarget = resolveSassModuleUseTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleUse,
      aliasResolver,
      options,
    );
    if (!moduleTarget) continue;
    const targets = listSassModuleExportedSymbolTargets(
      styleDocumentForPath,
      moduleTarget.filePath,
      moduleTarget.styleDocument,
      memberRef.symbolKind,
      memberRef.name,
      aliasResolver,
      new Set(),
      options,
    );
    if (targets.length !== 1) continue;
    const target = targets[0]!;
    return {
      filePath: target.filePath,
      styleDocument: target.styleDocument,
      moduleUse,
      memberRef,
      decl: target.decl,
      exportedName: target.exportedName,
    };
  }
  return null;
}

export function resolveSassWildcardSymbolTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  symbol: SassSymbolOccurrenceHIR | null,
  aliasResolver?: SassModulePathAliasResolver,
  options: SassModuleResolutionOptions = {},
): ResolvedSassWildcardSymbolTarget | null {
  if (!symbol) return null;
  if (findSassSymbolDeclForSymbol(styleDocument, symbol)) return null;

  const matches = new Map<string, ResolvedSassWildcardSymbolTarget>();
  for (const moduleUse of findSassWildcardModuleUses(styleDocument)) {
    const moduleTarget = resolveSassModuleUseTarget(
      styleDocumentForPath,
      styleFilePath,
      moduleUse,
      aliasResolver,
      options,
    );
    if (!moduleTarget) continue;
    const targets = listSassModuleExportedSymbolTargets(
      styleDocumentForPath,
      moduleTarget.filePath,
      moduleTarget.styleDocument,
      symbol.symbolKind,
      symbol.name,
      aliasResolver,
      new Set(),
      options,
    );
    if (targets.length !== 1) continue;
    const target = targets[0]!;
    matches.set(sassWildcardTargetKey(target.filePath, target.decl, target.exportedName), {
      filePath: target.filePath,
      styleDocument: target.styleDocument,
      moduleUse,
      symbol,
      decl: target.decl,
      exportedName: target.exportedName,
    });
  }

  return matches.size === 1 ? [...matches.values()][0]! : null;
}

export function listSassWildcardSymbolsForTarget(
  styleDocument: StyleDocumentHIR,
  target: Pick<ResolvedSassWildcardSymbolTarget, "decl"> & { readonly exportedName?: string },
): readonly SassSymbolOccurrenceHIR[] {
  const exportedName = target.exportedName ?? target.decl.name;
  return styleDocument.sassSymbols.filter(
    (symbol) =>
      symbol.symbolKind === target.decl.symbolKind &&
      symbol.name === exportedName &&
      findSassSymbolDeclForSymbol(styleDocument, symbol) === null,
  );
}

function findSassModuleUsesForNamespace(
  styleDocument: StyleDocumentHIR,
  namespace: string,
): readonly SassModuleUseHIR[] {
  return styleDocument.sassModuleUses.filter(
    (moduleUse) => moduleUse.namespaceKind !== "wildcard" && moduleUse.namespace === namespace,
  );
}

function findSassWildcardModuleUses(styleDocument: StyleDocumentHIR): readonly SassModuleUseHIR[] {
  return styleDocument.sassModuleUses.filter((moduleUse) => moduleUse.namespaceKind === "wildcard");
}

function sassWildcardTargetKey(
  filePath: string,
  decl: SassSymbolDeclHIR,
  exportedName: string,
): string {
  return [
    filePath,
    decl.symbolKind,
    decl.name,
    exportedName,
    decl.range.start.line,
    decl.range.start.character,
    decl.range.end.line,
    decl.range.end.character,
  ].join("\u0000");
}

function resolveSassModuleBasePath(
  styleFilePath: string,
  source: string,
  aliasResolver?: SassModulePathAliasResolver,
  fileExists?: (candidate: string) => boolean,
  options: SassModuleResolutionOptions = {},
): string | null {
  const cleanSource = stripSassModuleSpecifierSuffix(source);
  if (cleanSource.startsWith("sass:")) return null;
  if (isRelativeSpecifier(cleanSource)) {
    return path.resolve(path.dirname(styleFilePath), cleanSource);
  }
  if (path.isAbsolute(cleanSource)) return cleanSource;
  const aliasTarget = aliasResolver?.resolve(cleanSource, fileExists, styleFilePath);
  if (aliasTarget) return aliasTarget;
  const packageTarget = resolveSassPackageBasePath(styleFilePath, cleanSource, fileExists, options);
  if (packageTarget) return packageTarget;
  return resolveSassLocalBareBasePath(styleFilePath, cleanSource, fileExists);
}

function stripSassModuleSpecifierSuffix(source: string): string {
  return source.startsWith("#") ? source.split("?", 1)[0]! : source.split(/[?#]/, 1)[0]!;
}

function expandSassModuleCandidatePaths(basePath: string): readonly string[] {
  const parsed = path.parse(basePath);
  const candidates: string[] = [];

  if (isStyleModuleExtension(parsed.ext)) {
    candidates.push(basePath, path.join(parsed.dir, `_${parsed.base}`));
    return uniquePaths(candidates);
  }

  for (const extension of SASS_MODULE_EXTENSIONS) {
    candidates.push(
      `${basePath}${extension}`,
      path.join(parsed.dir, `_${parsed.base}${extension}`),
      path.join(basePath, `index${extension}`),
      path.join(basePath, `_index${extension}`),
    );
  }

  return uniquePaths(candidates);
}

function resolveSassPackageBasePath(
  styleFilePath: string,
  source: string,
  fileExists?: (candidate: string) => boolean,
  options: SassModuleResolutionOptions = {},
): string | null {
  const packageSpecifier = normalizeSassPackageSpecifier(source);
  if (
    !packageSpecifier ||
    isRelativeSpecifier(packageSpecifier) ||
    path.isAbsolute(packageSpecifier)
  ) {
    return null;
  }

  if (isPackageImportSpecifier(packageSpecifier)) {
    return resolveSassPackageImportBasePath(styleFilePath, packageSpecifier, fileExists, options);
  }

  let current = path.dirname(styleFilePath);
  while (true) {
    const packageEntry = resolveSassPackageJsonEntryBasePath(
      current,
      packageSpecifier,
      fileExists,
      options,
    );
    if (packageEntry) return packageEntry;

    const parsed = parsePackageSpecifier(packageSpecifier);
    const candidate = parsed
      ? resolveSassPackageFallbackBasePath(current, parsed, fileExists)
      : null;
    if (candidate) return candidate;

    const parent = path.dirname(current);
    if (parent === current) return null;
    current = parent;
  }
}

function resolveSassPackageFallbackBasePath(
  currentPath: string,
  packageSource: { readonly packageName: string; readonly subpath: string },
  fileExists: ((candidate: string) => boolean) | undefined,
): string | null {
  const packageRoot = path.join(currentPath, "node_modules", packageSource.packageName);
  const candidates =
    packageSource.subpath.length > 0
      ? [
          path.join(packageRoot, packageSource.subpath),
          path.join(packageRoot, "src", packageSource.subpath),
        ]
      : [packageRoot, path.join(packageRoot, "src", "index")];

  if (!fileExists) return candidates[0] ?? null;
  return candidates.find((candidate) => sassPackageJsonEntryExists(candidate, fileExists)) ?? null;
}

function resolveSassPackageImportBasePath(
  styleFilePath: string,
  packageSpecifier: string,
  fileExists: ((candidate: string) => boolean) | undefined,
  options: SassModuleResolutionOptions,
): string | null {
  if (!options.readFile) return null;
  for (const packageJsonPath of currentPackageJsonCandidatePaths(styleFilePath)) {
    const packageJsonText = options.readFile(packageJsonPath);
    if (!packageJsonText) continue;
    const entry = readSassPackageImportEntry(packageJsonText, packageSpecifier);
    if (!entry) continue;
    if (entry.startsWith("./")) {
      const candidate = path.resolve(
        path.dirname(packageJsonPath),
        normalizePackageJsonEntry(entry),
      );
      return !fileExists || sassPackageJsonEntryExists(candidate, fileExists) ? candidate : null;
    }
    if (entry.startsWith("#")) return null;
    return resolveSassPackageBasePath(styleFilePath, entry, fileExists, options);
  }
  return null;
}

function resolveSassPackageJsonEntryBasePath(
  currentPath: string,
  packageSpecifier: string,
  fileExists: ((candidate: string) => boolean) | undefined,
  options: SassModuleResolutionOptions,
): string | null {
  if (!options.readFile) return null;
  const parsed = parsePackageSpecifier(packageSpecifier);
  if (!parsed) return null;

  const packageDir = path.join(currentPath, "node_modules", parsed.packageName);
  const packageJsonText = options.readFile(path.join(packageDir, "package.json"));
  if (!packageJsonText) return null;

  const entry = readSassPackageJsonEntry(packageJsonText, parsed.subpath);
  if (!entry) return null;

  const candidate = path.resolve(packageDir, normalizePackageJsonEntry(entry));
  return !fileExists || sassPackageJsonEntryExists(candidate, fileExists) ? candidate : null;
}

function sassPackageJsonEntryExists(
  candidate: string,
  fileExists: (candidate: string) => boolean,
): boolean {
  return fileExists(candidate) || expandSassModuleCandidatePaths(candidate).some(fileExists);
}

function parsePackageSpecifier(
  packageSpecifier: string,
): { readonly packageName: string; readonly subpath: string } | null {
  const parts = packageSpecifier.split("/").filter(Boolean);
  if (parts.length === 0) return null;
  if (parts[0]!.startsWith("@")) {
    if (parts.length < 2) return null;
    return {
      packageName: `${parts[0]!}/${parts[1]!}`,
      subpath: parts.slice(2).join("/"),
    };
  }
  return {
    packageName: parts[0]!,
    subpath: parts.slice(1).join("/"),
  };
}

function currentPackageJsonCandidatePaths(styleFilePath: string): readonly string[] {
  const candidates: string[] = [];
  let current = path.dirname(styleFilePath);
  while (true) {
    candidates.push(path.join(current, "package.json"));
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return candidates;
}

function readSassPackageJsonEntry(packageJsonText: string, subpath = ""): string | null {
  let packageJson: unknown;
  try {
    packageJson = JSON.parse(packageJsonText);
  } catch {
    return null;
  }
  if (!isRecord(packageJson)) return null;
  if (subpath.length > 0) {
    return readSassPackageExportSubpathEntry(packageJson.exports, subpath);
  }
  return (
    readSassPackageExportEntry(packageJson.exports) ??
    readStringField(packageJson, "sass") ??
    readStringField(packageJson, "scss") ??
    readStringField(packageJson, "style")
  );
}

function readSassPackageImportEntry(packageJsonText: string, specifier: string): string | null {
  let packageJson: unknown;
  try {
    packageJson = JSON.parse(packageJsonText);
  } catch {
    return null;
  }
  if (!isRecord(packageJson) || !isRecord(packageJson.imports)) return null;
  const exactEntry = readSassPackageExportEntry(packageJson.imports[specifier]);
  if (exactEntry) return exactEntry;
  for (const [key, value] of Object.entries(packageJson.imports)) {
    const patternMatch = matchPackageImportPattern(key, specifier);
    if (patternMatch === null) continue;
    const entry = readSassPackageExportEntry(value);
    if (entry) return substitutePackageExportPattern(entry, patternMatch);
  }
  return null;
}

function matchPackageImportPattern(patternKey: string, specifier: string): string | null {
  const [prefix, suffix, ...rest] = patternKey.split("*");
  if (prefix === undefined || suffix === undefined || rest.length > 0) return null;
  if (!specifier.startsWith(prefix) || !specifier.endsWith(suffix)) return null;
  return specifier.slice(prefix.length, specifier.length - suffix.length);
}

function readSassPackageExportSubpathEntry(exportsValue: unknown, subpath: string): string | null {
  if (!isRecord(exportsValue)) return null;
  for (const key of packageExportSubpathKeys(subpath)) {
    const exportValue = exportsValue[key];
    if (exportValue === undefined) continue;
    const entry = readSassPackageExportEntry(exportValue);
    if (entry) return entry;
  }

  for (const [key, exportValue] of Object.entries(exportsValue)) {
    const patternMatch = matchPackageExportSubpathPattern(key, subpath);
    if (patternMatch === null) continue;
    const entry = readSassPackageExportEntry(exportValue);
    if (entry) return substitutePackageExportPattern(entry, patternMatch);
  }

  return null;
}

function packageExportSubpathKeys(subpath: string): readonly string[] {
  const normalized = subpath.replace(/^\.?\//u, "");
  return [`./${normalized}`, `./${normalized}.scss`, `./${normalized}.sass`, `./${normalized}.css`];
}

function matchPackageExportSubpathPattern(patternKey: string, subpath: string): string | null {
  const normalizedPattern = patternKey.replace(/^\.?\//u, "");
  if (!normalizedPattern.includes("*")) return null;

  const [prefix, suffix, ...rest] = normalizedPattern.split("*");
  if (rest.length > 0) return null;

  for (const candidateKey of packageExportSubpathKeys(subpath)) {
    const normalizedCandidate = candidateKey.replace(/^\.?\//u, "");
    if (!normalizedCandidate.startsWith(prefix ?? "")) continue;
    if (!normalizedCandidate.endsWith(suffix ?? "")) continue;

    return normalizedCandidate.slice(
      prefix?.length ?? 0,
      normalizedCandidate.length - (suffix?.length ?? 0),
    );
  }

  return null;
}

function substitutePackageExportPattern(entry: string, patternMatch: string): string {
  return entry.includes("*") ? entry.split("*").join(patternMatch) : entry;
}

function readSassPackageExportEntry(exportsValue: unknown): string | null {
  return readSassPackageExportEntryWithPolicy(exportsValue, true);
}

function readSassPackageExportEntryWithPolicy(
  exportsValue: unknown,
  requireStyleEntry: boolean,
): string | null {
  if (typeof exportsValue === "string") {
    return !requireStyleEntry || isSassPackageStyleExportEntry(exportsValue) ? exportsValue : null;
  }
  if (Array.isArray(exportsValue)) {
    for (const value of exportsValue) {
      const entry = readSassPackageExportEntryWithPolicy(value, requireStyleEntry);
      if (entry) return entry;
    }
    return null;
  }
  if (!isRecord(exportsValue)) return null;

  const rootExport = exportsValue["."];
  if (rootExport !== undefined) {
    const rootEntry = readSassPackageExportEntryWithPolicy(rootExport, requireStyleEntry);
    if (rootEntry) return rootEntry;
  }

  for (const [key, value] of Object.entries(exportsValue)) {
    const entry = isSassPackageExportCondition(key)
      ? readSassPackageExportEntryWithPolicy(value, false)
      : key === "default"
        ? readSassPackageExportEntryWithPolicy(value, true)
        : null;
    if (entry) return entry;
  }
  return null;
}

function isSassPackageExportCondition(key: string): boolean {
  return ["sass", "scss", "style"].includes(key);
}

function isSassPackageStyleExportEntry(entry: string): boolean {
  const extension = path.extname(normalizePackageJsonEntry(entry));
  return extension === "" || [".css", ".scss", ".sass", ".less"].includes(extension);
}

function readStringField(record: Record<string, unknown>, key: string): string | null {
  const value = record[key];
  return typeof value === "string" ? value : null;
}

function normalizePackageJsonEntry(entry: string): string {
  return entry.replace(/^\.?\//u, "");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function resolveSassLocalBareBasePath(
  styleFilePath: string,
  source: string,
  fileExists?: (candidate: string) => boolean,
): string | null {
  const candidate = path.resolve(path.dirname(styleFilePath), source);
  return !fileExists || fileExists(candidate) ? candidate : null;
}

function normalizeSassPackageSpecifier(source: string): string {
  if (source.startsWith("pkg:")) return source.slice("pkg:".length);
  if (source.startsWith("~") && !source.startsWith("~/")) return source.slice(1);
  return source;
}

function isPackageImportSpecifier(specifier: string): boolean {
  return specifier.startsWith("#");
}

function isStyleModuleExtension(extension: string): boolean {
  return SASS_MODULE_EXTENSIONS.includes(extension as (typeof SASS_MODULE_EXTENSIONS)[number]);
}

function isRelativeSpecifier(specifier: string): boolean {
  return specifier.startsWith("./") || specifier.startsWith("../");
}

function uniquePaths(paths: readonly string[]): readonly string[] {
  return [...new Set(paths)];
}

export function resolveValueImportTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  valueImport: ValueImportHIR | null,
): ResolvedValueTarget | null {
  if (!valueImport) return null;
  const targetFilePath = path.resolve(path.dirname(styleFilePath), valueImport.from);
  const targetDocument = styleDocumentForPath(targetFilePath);
  if (!targetDocument) return null;
  const valueDecl = findValueDeclByName(targetDocument, valueImport.importedName);
  if (!valueDecl) return null;
  return {
    filePath: targetDocument.filePath,
    styleDocument: targetDocument,
    valueDecl,
    bindingKind: "imported",
    valueImport,
  };
}

export function resolveValueTarget(
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  styleFilePath: string,
  styleDocument: StyleDocumentHIR,
  name: string,
): ResolvedValueTarget | null {
  const localDecl = findValueDeclByName(styleDocument, name);
  if (localDecl) {
    return {
      filePath: styleDocument.filePath,
      styleDocument,
      valueDecl: localDecl,
      bindingKind: "local",
    };
  }
  return resolveValueImportTarget(
    styleDocumentForPath,
    styleFilePath,
    findValueImportByName(styleDocument, name),
  );
}
