import type {
  BemSuffixInfo,
  ComposesClassToken,
  ComposesRef,
  Range,
} from "@css-module-explainer/shared";
import {
  parse as postcssParse,
  type AtRule,
  type ChildNode,
  type Declaration,
  type Root,
  type Rule,
} from "postcss";
import {
  makeStyleDocumentHIR,
  type AnimationNameRefHIR,
  type CustomPropertyDeclAtRuleContextHIR,
  type CustomPropertyDeclContextHIR,
  type CustomPropertyDeclHIR,
  type CustomPropertyRefHIR,
  type KeyframesDeclHIR,
  type NestedSelectorSafety,
  type SassModuleForwardHIR,
  type SassModuleForwardMemberHIR,
  type SassModuleMemberRefHIR,
  type SassModuleUseHIR,
  type SassSymbolDeclHIR,
  type SassSymbolKind,
  type SassSymbolOccurrenceHIR,
  type SassSymbolResolution,
  type SassSymbolRole,
  type SelectorDeclHIR,
  type StyleAtRuleContextHIR,
  type StyleDocumentHIR,
  type StylePreprocessorSymbolSyntax,
  type ValueDeclHIR,
  type ValueImportHIR,
  type ValueRefHIR,
} from "../hir/style-types";
import { rangeContains } from "../util/range-utils";
import { classifyBemSuffixSite } from "./bem-suffix";
import { findStyleDocumentLangForPath, getRuntimeSyntax } from "./lang-registry";
import {
  atRootTokenRange,
  enumerateGroups,
  extractClassNames,
  extractIntroducedClassNames,
  findClassTokenRange,
  rangeForSourceNode,
  resolveSelector,
} from "./scss-selector-utils";

/**
 * Context threaded through the recursive SCSS walk. Carries the
 * resolved parent selector plus a narrower view for `&`-nested
 * rename support: `className` is set iff the parent rule is a
 * bare single `.classname`, and `isGrouped` tracks whether the
 * parent rule had `selectors.length > 1`.
 *
 * `className` flows into `bemSuffix.parentResolvedName` for
 * BEM-safe nested entries; the rename provider reads it to
 * drive suffix-math edits.
 */
export interface ParentContext {
  readonly selector: string;
  readonly className?: string;
  readonly isGrouped?: boolean;
}

/**
 * Build the child recursion context from a parent rule's resolved
 * selectors and one specific child branch.
 *
 * Sets `className` only when the **current branch** is a bare
 * single class (`.foo`) AND the parent rule was not grouped —
 * otherwise `className` is undefined so rename-safe-nested
 * entries reject downstream.
 *
 * Exported for unit testing of the context derivation logic.
 */
export function buildChildContext(
  resolvedSelectors: readonly string[],
  nextResolved: string,
): ParentContext {
  const classesInParent = extractClassNames(nextResolved);
  const isBareSingleClass =
    resolvedSelectors.length === 1 &&
    classesInParent.length === 1 &&
    nextResolved === "." + classesInParent[0];
  const ctx: ParentContext = {
    selector: nextResolved,
    ...(isBareSingleClass ? { className: classesInParent[0] } : {}),
    ...(resolvedSelectors.length > 1 ? { isGrouped: true } : {}),
  };
  return ctx;
}

/**
 * Parse a CSS Module file into style-document HIR.
 *
 * Parsing is best-effort: a parse error produces an empty document, never
 * throws. The caller (StyleIndexCache) treats an empty document as a
 * legitimate "no classes found" result, so upstream providers keep
 * running even when one file is broken.
 */
export function parseStyleDocument(content: string, filePath: string): StyleDocumentHIR {
  const selectors: SelectorDeclHIR[] = [];
  const keyframesByName = new Map<string, KeyframesDeclHIR>();
  const animationNameRefs: AnimationNameRefHIR[] = [];

  const lang = findStyleDocumentLangForPath(filePath);
  // shared.StyleLang.syntax is typed as `unknown` so the shared
  // module stays runtime-free. The narrowing cast lives in
  // `getRuntimeSyntax` (the single documented `as` cast).
  const syntax = lang ? getRuntimeSyntax(lang) : null;

  // postcss's top-level `parse` is hardcoded to the CSS grammar
  // and silently ignores `opts.syntax`. Delegate to the Syntax
  // object's own `.parse` when one is provided (SCSS, LESS, …)
  // so non-CSS features — `//` line comments, `#{...}`
  // interpolation, SASS directives — actually reach the right
  // grammar. Plain CSS falls back to the top-level postcss parser
  // because `lang-registry` records `syntax: null` for it.
  const parse = typeof syntax?.parse === "function" ? syntax.parse.bind(syntax) : postcssParse;
  let root: Root;
  try {
    root = parse(content, { from: filePath }) as Root;
  } catch {
    return makeStyleDocumentHIR(filePath, []);
  }

  const valuePathAliases = collectValuePathAliases(root);
  const valueDecls = collectValueDecls(root, valuePathAliases);
  const valueImports = collectValueImports(root, valuePathAliases);
  const valueRefs = collectValueRefs(root, valueDecls, valueImports);
  const customPropertyDecls = collectCustomPropertyDecls(root);
  const customPropertyRefs = collectCustomPropertyRefs(root);
  const sassModuleUses = collectSassModuleUses(root);
  const sassModuleForwards = collectSassModuleForwards(root);
  const symbolSyntax = lang?.id === "less" ? "less" : undefined;
  const sassSymbolDecls = collectSassSymbolDecls(root, symbolSyntax);
  const sassSymbolTargets = collectSassSymbolTargetContext(
    root,
    sassSymbolDecls,
    sassModuleUses,
    symbolSyntax,
  );
  const sassSymbols: SassSymbolOccurrenceHIR[] = [];
  const sassModuleMemberRefs: SassModuleMemberRefHIR[] = [];

  walkStyleNodes(
    root.nodes as ChildNode[],
    { selector: "" },
    selectors,
    keyframesByName,
    animationNameRefs,
    sassSymbols,
    sassModuleMemberRefs,
    sassSymbolTargets,
  );
  return makeStyleDocumentHIR(
    filePath,
    [...selectors].toSorted(compareSelectors),
    Array.from(keyframesByName.values()).toSorted(compareNamedStyleFacts),
    [...animationNameRefs].toSorted(compareNamedStyleFacts),
    [...valueDecls].toSorted(compareNamedStyleFacts),
    [...valueImports].toSorted(compareNamedStyleFacts),
    [...valueRefs].toSorted(compareNamedStyleFacts),
    [...customPropertyDecls].toSorted(compareNamedStyleFacts),
    [...customPropertyRefs].toSorted(compareNamedStyleFacts),
    [...sassSymbols].toSorted(compareSassSymbolOccurrences),
    [...sassSymbolDecls].toSorted(compareSassSymbolDecls),
    [...sassModuleUses].toSorted(compareSassModuleUses),
    [...sassModuleMemberRefs].toSorted(compareSassModuleMemberRefs),
    [...sassModuleForwards].toSorted(compareSassModuleForwards),
  );
}

/**
 * Recurse through postcss nodes, recording classes that CSS Modules
 * exposes on the `styles` object. Transparent at-rules (@media,
 * @supports, @at-root) are unwrapped; @keyframes/@font-face are skipped.
 */
function walkStyleNodes(
  nodes: ChildNode[] | undefined,
  parentCtx: ParentContext,
  selectors: SelectorDeclHIR[],
  keyframesByName: Map<string, KeyframesDeclHIR>,
  animationNameRefs: AnimationNameRefHIR[],
  sassSymbols: SassSymbolOccurrenceHIR[],
  sassModuleMemberRefs: SassModuleMemberRefHIR[],
  sassSymbolTargets: SassSymbolTargetContext,
): void {
  if (!nodes) return;
  for (const node of nodes) {
    if (node.type === "rule") {
      if (isGlobalBlockRule(node.selector)) continue;
      if (isLocalBlockRule(node.selector)) {
        walkStyleNodes(
          node.nodes,
          parentCtx,
          selectors,
          keyframesByName,
          animationNameRefs,
          sassSymbols,
          sassModuleMemberRefs,
          sassSymbolTargets,
        );
        continue;
      }
      recordRule(
        node,
        parentCtx,
        selectors,
        keyframesByName,
        animationNameRefs,
        sassSymbols,
        sassModuleMemberRefs,
        sassSymbolTargets,
      );
    } else if (node.type === "atrule" && isKeyframesAtRule(node.name)) {
      recordKeyframesAtRule(node, keyframesByName);
    } else if (node.type === "atrule" && isTransparentAtRule(node.name)) {
      if (node.name === "at-root" && isInlineAtRoot(node)) {
        recordAtRootInlineRule(
          node,
          selectors,
          sassSymbols,
          sassModuleMemberRefs,
          sassSymbolTargets,
        );
      } else if (node.name === "at-root") {
        walkStyleNodes(
          node.nodes,
          { selector: "" },
          selectors,
          keyframesByName,
          animationNameRefs,
          sassSymbols,
          sassModuleMemberRefs,
          sassSymbolTargets,
        );
      } else {
        walkStyleNodes(
          node.nodes,
          parentCtx,
          selectors,
          keyframesByName,
          animationNameRefs,
          sassSymbols,
          sassModuleMemberRefs,
          sassSymbolTargets,
        );
      }
    }
  }
}

function isTransparentAtRule(name: string): boolean {
  return name === "media" || name === "at-root" || name === "supports" || name === "layer";
}

function isKeyframesAtRule(name: string): boolean {
  return name === "keyframes" || name === "-webkit-keyframes";
}

function isGlobalBlockRule(selector: string): boolean {
  return /^:global\s*$/.test(selector.trim());
}

function isLocalBlockRule(selector: string): boolean {
  return /^:local\s*$/.test(selector.trim());
}

function isInlineAtRoot(atrule: AtRule): boolean {
  return atrule.params.trim().length > 0;
}

function recordRule(
  rule: Rule,
  parentCtx: ParentContext,
  selectors: SelectorDeclHIR[],
  keyframesByName: Map<string, KeyframesDeclHIR>,
  animationNameRefs: AnimationNameRefHIR[],
  sassSymbols: SassSymbolOccurrenceHIR[],
  sassModuleMemberRefs: SassModuleMemberRefHIR[],
  sassSymbolTargets: SassSymbolTargetContext,
): void {
  const { declarations, composes, animationRefs } = collectDeclarationsAndComposes(rule.nodes);
  const ruleRange = rangeForSourceNode(rule);
  const context = selectorRuleContextForRule(rule);
  animationNameRefs.push(...animationRefs);

  const selectorSource = rule.raws.selector?.raw ?? rule.selector;
  const groups = enumerateGroups(selectorSource);
  const resolvedSelectors: string[] = [];

  for (const { raw, offset } of groups) {
    const resolved = resolveSelector(raw, parentCtx.selector);
    resolvedSelectors.push(resolved);
    const bemSuffix = classifyBemSuffixSite(rule, raw, offset, parentCtx, groups.length);
    const isNested = parentCtx.selector !== "" || raw.includes("&");
    let introducedClassCount = 0;

    for (const className of extractIntroducedClassNames(raw, resolved)) {
      introducedClassCount += 1;
      const existing = selectors.find(
        (selector) => selector.name === className && selector.viewKind === "canonical",
      );
      if (existing?.nestedSafety === "flat" && isNested && raw.includes("&")) continue;
      selectors.push(
        buildSelectorDecl({
          className,
          resolved,
          raw,
          rule,
          declarations,
          composes,
          ruleRange,
          context,
          isNested,
          bemSuffix,
        }),
      );
      sassSymbols.push(
        ...collectDirectSassSymbolOccurrences(rule.nodes, className, ruleRange, sassSymbolTargets),
      );
      sassModuleMemberRefs.push(
        ...collectDirectSassModuleMemberRefs(rule.nodes, className, ruleRange),
      );
    }

    if (introducedClassCount === 0 && parentCtx.className) {
      sassSymbols.push(
        ...collectDirectSassSymbolOccurrences(
          rule.nodes,
          parentCtx.className,
          ruleRange,
          sassSymbolTargets,
        ),
      );
      sassModuleMemberRefs.push(
        ...collectDirectSassModuleMemberRefs(rule.nodes, parentCtx.className, ruleRange),
      );
    }
  }

  const parents = resolvedSelectors.length > 0 ? resolvedSelectors : [parentCtx.selector];
  for (const nextResolved of parents) {
    walkStyleNodes(
      rule.nodes,
      buildChildContext(resolvedSelectors, nextResolved),
      selectors,
      keyframesByName,
      animationNameRefs,
      sassSymbols,
      sassModuleMemberRefs,
      sassSymbolTargets,
    );
  }
}

interface BuildEntryArgs {
  readonly className: string;
  readonly resolved: string;
  readonly raw: string;
  readonly rule: Rule;
  readonly declarations: string;
  readonly composes: readonly ComposesRef[];
  readonly ruleRange: Range;
  readonly context: SelectorDeclHIR["context"];
  readonly isNested: boolean;
  readonly bemSuffix: BemSuffixInfo | null;
}

function buildSelectorDecl(args: BuildEntryArgs): SelectorDeclHIR {
  const tokenRange = findClassTokenRange(args.rule.source?.start, args.className, args.raw);
  return {
    kind: "selector",
    id: `selector:${args.className}:${tokenRange.start.line}:${tokenRange.start.character}`,
    name: args.className,
    canonicalName: args.className,
    viewKind: "canonical",
    range: tokenRange,
    fullSelector: args.resolved,
    declarations: args.declarations,
    ruleRange: args.ruleRange,
    composes: args.composes,
    nestedSafety: classifyNestedSafety(args.isNested, args.bemSuffix),
    ...(args.context ? { context: args.context } : {}),
    ...(args.bemSuffix ? { bemSuffix: args.bemSuffix } : {}),
  };
}

function selectorRuleContextForRule(rule: Rule): SelectorDeclHIR["context"] {
  const wrapperAtRules = wrapperAtRulesForStyleContainer(rule);
  if (wrapperAtRules.length === 0) return undefined;
  return { wrapperAtRules };
}

function collectDeclarationsAndComposes(nodes: ChildNode[] | undefined): {
  declarations: string;
  composes: ComposesRef[];
  animationRefs: AnimationNameRefHIR[];
} {
  if (!nodes) return { declarations: "", composes: [], animationRefs: [] };

  const composes: ComposesRef[] = [];
  const declParts: string[] = [];
  const animationRefs: AnimationNameRefHIR[] = [];

  for (const node of nodes) {
    if (node.type !== "decl") continue;
    if (node.prop === "composes") {
      const ref = parseComposesValue(node);
      if (ref) composes.push(ref);
    } else {
      declParts.push(`${node.prop}: ${node.value}`);
      animationRefs.push(...findAnimationNameTokens(node));
    }
  }

  return { declarations: declParts.join("; "), composes, animationRefs };
}

const COMPOSES_FROM_RE = /^(.+?)\s+from\s+(?:'([^']+)'|"([^"]+)"|(global))\s*$/;

function parseComposesValue(node: Extract<ChildNode, { type: "decl" }>): ComposesRef | null {
  const value = node.value;
  const trimmed = value.trim();
  const range = rangeForDeclNode(node);
  const match = COMPOSES_FROM_RE.exec(trimmed);
  if (match) {
    const classNames = match[1]!.trim().split(/\s+/);
    const from = match[2] ?? match[3];
    const fromGlobal = match[4] === "global" || undefined;
    const classTokens = findComposesClassTokens(node, classNames);
    return {
      classNames,
      range,
      ...(classTokens.length > 0 ? { classTokens } : {}),
      ...(from ? { from } : {}),
      ...(fromGlobal ? { fromGlobal } : {}),
    };
  }
  const classNames = trimmed.split(/\s+/).filter((s) => s.length > 0);
  if (classNames.length === 0) return null;
  const classTokens = findComposesClassTokens(node, classNames);
  return {
    classNames,
    range,
    ...(classTokens.length > 0 ? { classTokens } : {}),
  };
}

function findComposesClassTokens(
  node: Extract<ChildNode, { type: "decl" }>,
  classNames: readonly string[],
): readonly ComposesClassToken[] {
  const source = node.source;
  if (!source?.start) return [];
  const propIndex = node.toString().indexOf(node.prop);
  const valueIndex = node.toString().indexOf(node.value, propIndex + node.prop.length);
  if (valueIndex < 0) return [];
  const baseOffset = source.start.offset + valueIndex;
  const tokens: ComposesClassToken[] = [];
  let searchStart = 0;

  for (const className of classNames) {
    const localOffset = node.value.indexOf(className, searchStart);
    if (localOffset < 0) continue;
    const startOffset = baseOffset + localOffset;
    const endOffset = startOffset + className.length;
    const startPos = source.input.fromOffset(startOffset);
    const endPos = source.input.fromOffset(endOffset);
    if (startPos && endPos) {
      tokens.push({
        className,
        range: {
          start: { line: startPos.line - 1, character: startPos.col - 1 },
          end: { line: endPos.line - 1, character: endPos.col - 1 },
        },
      });
    }
    searchStart = localOffset + className.length;
  }

  return tokens;
}

function recordKeyframesAtRule(
  atrule: AtRule,
  keyframesByName: Map<string, KeyframesDeclHIR>,
): void {
  const keyframesName = parseKeyframesName(atrule);
  if (!keyframesName) return;

  keyframesByName.set(keyframesName.name, {
    kind: "keyframes",
    id: `keyframes:${keyframesName.name}:${keyframesName.range.start.line}:${keyframesName.range.start.character}`,
    name: keyframesName.name,
    range: keyframesName.range,
    ruleRange: rangeForSourceNode(atrule),
  });
}

function collectValuePathAliases(root: Root): ReadonlyMap<string, string> {
  const aliases = new Map<string, string>();
  root.walkAtRules("value", (atrule) => {
    const pathAlias = parseValuePathAlias(atrule);
    if (!pathAlias) return;
    aliases.set(pathAlias.name, pathAlias.target);
  });
  return aliases;
}

function parseValuePathAlias(atrule: AtRule): { name: string; target: string } | null {
  if (/\bfrom\b/u.test(atrule.params)) return null;
  const match = /^\s*([\p{L}_-][\p{L}\p{N}\p{M}_-]*)\s*:\s*(?:"([^"]+)"|'([^']+)')\s*$/u.exec(
    atrule.params,
  );
  const name = match?.[1];
  const target = match?.[2] ?? match?.[3];
  if (!name || !target) return null;
  if (!looksLikeStyleRequest(target)) return null;
  return { name, target };
}

function collectValueDecls(
  root: Root,
  valuePathAliases: ReadonlyMap<string, string>,
): readonly ValueDeclHIR[] {
  const valueDecls: ValueDeclHIR[] = [];
  root.walkAtRules("value", (atrule) => {
    const localValueDecl = parseLocalValueDecl(atrule, valuePathAliases);
    if (!localValueDecl) return;
    valueDecls.push(localValueDecl);
  });
  return valueDecls;
}

function parseLocalValueDecl(
  atrule: AtRule,
  valuePathAliases: ReadonlyMap<string, string>,
): ValueDeclHIR | null {
  if (/\bfrom\b/u.test(atrule.params)) return null;
  const match = /^\s*([\p{L}_-][\p{L}\p{N}\p{M}_-]*)\s*:\s*(.+?)\s*$/u.exec(atrule.params);
  const name = match?.[1];
  const value = match?.[2];
  if (!name || !value) return null;
  if (valuePathAliases.has(name) || isQuotedStyleRequest(value.trim())) return null;
  const range = findAtRuleParamTokenRange(atrule, name, name.length);
  if (!range) return null;
  return {
    kind: "valueDecl",
    id: `value:${name}:${range.start.line}:${range.start.character}`,
    name,
    value,
    range,
    ruleRange: rangeForSourceNode(atrule),
  };
}

function collectValueImports(
  root: Root,
  valuePathAliases: ReadonlyMap<string, string>,
): readonly ValueImportHIR[] {
  const valueImports: ValueImportHIR[] = [];
  root.walkAtRules("value", (atrule) => {
    valueImports.push(...parseValueImports(atrule, valuePathAliases));
  });
  return valueImports;
}

function parseValueImports(
  atrule: AtRule,
  valuePathAliases: ReadonlyMap<string, string>,
): readonly ValueImportHIR[] {
  const parts = /^\s*(.+?)\s+from\s+(.+?)\s*$/u.exec(atrule.params);
  if (!parts) return [];
  const source = resolveValueImportSource(parts[2]!, valuePathAliases);
  if (!source) return [];
  const specs = splitValueImportSpecs(parts[1]!);
  return specs.flatMap((spec, index) => {
    const parsed = parseValueImportSpec(spec.raw);
    if (!parsed) return [];
    const range = findAtRuleParamValueTokenRange(
      atrule,
      spec.offset + parsed.localOffset,
      parsed.localName.length,
    );
    if (!range) return [];
    const importedNameRange = parsed.hasAlias
      ? findAtRuleParamValueTokenRange(
          atrule,
          spec.offset + parsed.importedOffset,
          parsed.importedName.length,
        )
      : null;
    return [
      {
        kind: "valueImport",
        id: `value-import:${atrule.source?.start?.line ?? 0}:${index}:${parsed.localName}`,
        name: parsed.localName,
        importedName: parsed.importedName,
        ...(importedNameRange ? { importedNameRange } : {}),
        from: source,
        range,
        ruleRange: rangeForSourceNode(atrule),
      },
    ];
  });
}

function resolveValueImportSource(
  rawSource: string,
  valuePathAliases: ReadonlyMap<string, string>,
): string | null {
  const trimmed = rawSource.trim();
  const quoted = unquoteCssString(trimmed);
  if (quoted) return quoted;
  return valuePathAliases.get(trimmed) ?? null;
}

function splitValueImportSpecs(raw: string): Array<{ raw: string; offset: number }> {
  return raw
    .split(",")
    .map((part, index, all) => {
      const rawPart = part.trim();
      const prefixLength = all
        .slice(0, index)
        .reduce((sum, current) => sum + current.length + 1, 0);
      return { raw: rawPart, offset: raw.indexOf(rawPart, prefixLength) };
    })
    .filter((part) => part.raw.length > 0);
}

function parseValueImportSpec(raw: string): {
  importedName: string;
  importedOffset: number;
  localName: string;
  localOffset: number;
  hasAlias: boolean;
} | null {
  const match =
    /^\s*([\p{L}_-][\p{L}\p{N}\p{M}_-]*)(?:\s+as\s+([\p{L}_-][\p{L}\p{N}\p{M}_-]*))?\s*$/u.exec(
      raw,
    );
  const importedName = match?.[1];
  const aliasName = match?.[2];
  const localName = aliasName ?? importedName;
  if (!importedName || !localName) return null;
  return {
    importedName,
    importedOffset: raw.indexOf(importedName),
    localName,
    localOffset: raw.lastIndexOf(localName),
    hasAlias: aliasName !== undefined,
  };
}

function collectValueRefs(
  root: Root,
  valueDecls: readonly ValueDeclHIR[],
  valueImports: readonly ValueImportHIR[],
): readonly ValueRefHIR[] {
  if (valueDecls.length === 0 && valueImports.length === 0) return [];
  const valueNames = new Set([
    ...valueDecls.map((decl) => decl.name),
    ...valueImports.map((valueImport) => valueImport.name),
  ]);
  const refs: ValueRefHIR[] = [];

  root.walkDecls((node) => {
    if (node.prop === "composes") return;
    const matches = findValueIdentifierMatches(node.value, valueNames);
    for (const match of matches) {
      const range = findDeclValueTokenRange(node, match.offset, match.name.length);
      if (!range) continue;
      refs.push({
        kind: "valueRef",
        id: `value-ref:decl:${node.source?.start?.line ?? 0}:${match.name}:${match.offset}`,
        name: match.name,
        range,
        source: "declaration",
      });
    }
  });

  root.walkAtRules("value", (atrule) => {
    const localValueDecl = parseLocalValueDecl(atrule, new Map());
    if (!localValueDecl) return;
    const colonIndex = atrule.params.indexOf(":");
    if (colonIndex < 0) return;
    const rawValue = atrule.params.slice(colonIndex + 1);
    const matches = findValueIdentifierMatches(rawValue, valueNames);
    for (const match of matches) {
      if (match.name === localValueDecl.name) continue;
      const range = findAtRuleParamValueTokenRange(
        atrule,
        colonIndex + 1 + match.offset,
        match.name.length,
      );
      if (!range) continue;
      refs.push({
        kind: "valueRef",
        id: `value-ref:value:${atrule.source?.start?.line ?? 0}:${match.name}:${match.offset}`,
        name: match.name,
        range,
        source: "valueDecl",
      });
    }
  });

  return refs;
}

function collectCustomPropertyDecls(root: Root): readonly CustomPropertyDeclHIR[] {
  const decls: CustomPropertyDeclHIR[] = [];
  root.walkDecls((node) => {
    if (!isCssCustomPropertyName(node.prop)) return;
    const range = findDeclPropTokenRange(node, node.prop.length);
    if (!range) return;
    decls.push({
      kind: "customPropertyDecl",
      id: `custom-property:${node.prop}:${range.start.line}:${range.start.character}`,
      name: node.prop,
      value: node.value,
      range,
      ruleRange: rangeForDeclarationContainer(node),
      context: customPropertyContextForDeclaration(node),
    });
  });
  return decls;
}

function customPropertyContextForDeclaration(node: Declaration): CustomPropertyDeclContextHIR {
  const parent = node.parent;
  if (parent?.type === "rule") {
    return {
      containerKind: "rule",
      selectorText: parent.selector,
      atRuleName: null,
      atRuleParams: null,
      wrapperAtRules: wrapperAtRulesForDeclaration(node),
    };
  }
  if (parent?.type === "atrule") {
    return {
      containerKind: "atrule",
      selectorText: null,
      atRuleName: parent.name,
      atRuleParams: parent.params,
      wrapperAtRules: wrapperAtRulesForDeclaration(node),
    };
  }
  return {
    containerKind: "root",
    selectorText: null,
    atRuleName: null,
    atRuleParams: null,
    wrapperAtRules: wrapperAtRulesForDeclaration(node),
  };
}

function wrapperAtRulesForDeclaration(
  node: Declaration,
): readonly CustomPropertyDeclAtRuleContextHIR[] {
  return wrapperAtRulesForStyleContainer(node.parent);
}

function wrapperAtRulesForStyleContainer(
  node: ChildNode | Root | undefined,
): readonly StyleAtRuleContextHIR[] {
  const wrappers: StyleAtRuleContextHIR[] = [];
  let current = parentContainerForStyleContext(node);
  while (current) {
    if (current.type === "atrule") {
      wrappers.push({
        name: current.name,
        params: current.params,
        range: rangeForSourceNode(current),
      });
    }
    current = parentContainerForStyleContext(current);
  }
  return wrappers.toReversed();
}

function parentContainerForStyleContext(
  node: ChildNode | Root | undefined,
): Rule | AtRule | Root | undefined {
  const parent = node?.parent;
  if (!parent) return undefined;
  if (parent.type === "rule" || parent.type === "atrule" || parent.type === "root") {
    return parent as Rule | AtRule | Root;
  }
  return undefined;
}

function rangeForDeclarationContainer(node: Declaration): Range {
  const parent = node.parent;
  if (parent?.type === "rule" || parent?.type === "atrule") return rangeForSourceNode(parent);
  const start = node.source?.start;
  const end = node.source?.end;
  return {
    start: start
      ? { line: start.line - 1, character: start.column - 1 }
      : { line: 0, character: 0 },
    end: end ? { line: end.line - 1, character: end.column - 1 } : { line: 0, character: 0 },
  };
}

function collectCustomPropertyRefs(root: Root): readonly CustomPropertyRefHIR[] {
  const refs: CustomPropertyRefHIR[] = [];
  root.walkDecls((node) => {
    for (const match of findCssVarFunctionMatches(node.value)) {
      const range = findDeclValueTokenRange(node, match.offset, match.name.length);
      if (!range) continue;
      refs.push({
        kind: "customPropertyRef",
        id: `custom-property-ref:${node.source?.start?.line ?? 0}:${match.name}:${match.offset}`,
        name: match.name,
        range,
        context: customPropertyContextForDeclaration(node),
      });
    }
  });
  return refs;
}

function findCssVarFunctionMatches(value: string): Array<{ name: string; offset: number }> {
  const matches: Array<{ name: string; offset: number }> = [];
  for (const match of value.matchAll(/\bvar\(\s*(--[\p{L}_-][\p{L}\p{N}\p{M}_-]*)/gu)) {
    const raw = match[0]!;
    const name = match[1]!;
    matches.push({
      name,
      offset: (match.index ?? 0) + raw.lastIndexOf(name),
    });
  }
  return matches;
}

function isCssCustomPropertyName(name: string): boolean {
  return /^--[\p{L}_-][\p{L}\p{N}\p{M}_-]*$/u.test(name);
}

function collectSassModuleUses(root: Root): readonly SassModuleUseHIR[] {
  const moduleUses: SassModuleUseHIR[] = [];
  root.walkAtRules("use", (atrule) => {
    moduleUses.push(...parseSassModuleUses(atrule));
  });
  root.walkAtRules("import", (atrule) => {
    moduleUses.push(...parseSassImportModuleUses(atrule));
  });
  return moduleUses;
}

function collectSassModuleForwards(root: Root): readonly SassModuleForwardHIR[] {
  const moduleForwards: SassModuleForwardHIR[] = [];
  root.walkAtRules("forward", (atrule) => {
    moduleForwards.push(...parseSassModuleForwards(atrule));
  });
  return moduleForwards;
}

function parseSassModuleUses(atrule: AtRule): readonly SassModuleUseHIR[] {
  const alias = parseSassModuleUseAlias(atrule.params);
  const ruleRange = rangeForSourceNode(atrule);
  return [...atrule.params.matchAll(/["']([^"']+)["']/g)].flatMap((match, index) => {
    const source = match[1];
    if (!source) return [];
    const sourceOffset = (match.index ?? 0) + 1;
    const range = findAtRuleParamValueTokenRange(atrule, sourceOffset, source.length);
    if (!range) return [];
    const namespace = resolveSassModuleUseNamespace(source, alias);
    return [
      {
        kind: "sassModuleUse",
        id: `sass-use:${atrule.source?.start?.line ?? 0}:${index}:${source}`,
        source,
        namespaceKind: namespace.namespaceKind,
        namespace: namespace.namespace,
        range,
        ruleRange,
      },
    ];
  });
}

function parseSassImportModuleUses(atrule: AtRule): readonly SassModuleUseHIR[] {
  const ruleRange = rangeForSourceNode(atrule);
  return [...atrule.params.matchAll(/["']([^"']+)["']/g)].flatMap((match, index) => {
    const source = match[1];
    if (!source || isPlainCssImportSource(source)) return [];
    const sourceOffset = (match.index ?? 0) + 1;
    const range = findAtRuleParamValueTokenRange(atrule, sourceOffset, source.length);
    if (!range) return [];
    return [
      {
        kind: "sassModuleUse",
        id: `sass-import:${atrule.source?.start?.line ?? 0}:${index}:${source}`,
        source,
        namespaceKind: "wildcard",
        namespace: null,
        range,
        ruleRange,
      },
    ];
  });
}

function isPlainCssImportSource(source: string): boolean {
  return /^(?:https?:)?\/\//u.test(source) || /\.css(?:[?#].*)?$/iu.test(source);
}

function parseSassModuleForwards(atrule: AtRule): readonly SassModuleForwardHIR[] {
  const ruleRange = rangeForSourceNode(atrule);
  const prefix = parseSassModuleForwardPrefix(atrule.params);
  const visibility = parseSassModuleForwardVisibility(atrule.params);
  return [...atrule.params.matchAll(/["']([^"']+)["']/g)].flatMap((match, index) => {
    const source = match[1];
    if (!source) return [];
    const sourceOffset = (match.index ?? 0) + 1;
    const range = findAtRuleParamValueTokenRange(atrule, sourceOffset, source.length);
    if (!range) return [];
    return [
      {
        kind: "sassModuleForward",
        id: `sass-forward:${atrule.source?.start?.line ?? 0}:${index}:${source}`,
        source,
        prefix,
        visibilityKind: visibility.visibilityKind,
        visibilityMembers: visibility.visibilityMembers,
        range,
        ruleRange,
      },
    ];
  });
}

function parseSassModuleForwardPrefix(params: string): string {
  const withoutQuotedSource = params.replaceAll(/["'](?:\\.|[^"'])*["']/g, " ");
  return /\bas\s+([A-Za-z_-][A-Za-z0-9_-]*)\*/.exec(withoutQuotedSource)?.[1] ?? "";
}

function parseSassModuleForwardVisibility(
  params: string,
): Pick<SassModuleForwardHIR, "visibilityKind" | "visibilityMembers"> {
  const withoutQuotedSource = params.replaceAll(/["'](?:\\.|[^"'])*["']/g, " ");
  const policyText = withoutQuotedSource.split(/\bwith\s*\(/u, 1)[0]!.trim();
  const match = /\b(show|hide)\s+(.+)$/u.exec(policyText);
  if (!match) return { visibilityKind: "all", visibilityMembers: [] };
  const visibilityKind = match[1] === "hide" ? "hide" : "show";
  const visibilityMembers = match[2]!
    .split(",")
    .map((raw) => parseSassModuleForwardMember(raw.trim()))
    .filter((member): member is SassModuleForwardMemberHIR => member !== null);
  return { visibilityKind, visibilityMembers };
}

function parseSassModuleForwardMember(raw: string): SassModuleForwardMemberHIR | null {
  if (raw.startsWith("$")) {
    const name = raw.slice(1);
    return isValidSassNamespace(name) ? { name, symbolKind: "variable" } : null;
  }
  return isValidSassNamespace(raw) ? { name: raw, symbolKind: null } : null;
}

function parseSassModuleUseAlias(params: string): string | undefined {
  const withoutQuotedSource = params.replaceAll(/["'](?:\\.|[^"'])*["']/g, " ");
  return /\bas\s+(\*|[A-Za-z_-][A-Za-z0-9_-]*)/.exec(withoutQuotedSource)?.[1];
}

function resolveSassModuleUseNamespace(
  source: string,
  alias: string | undefined,
): Pick<SassModuleUseHIR, "namespaceKind" | "namespace"> {
  if (alias === "*") {
    return { namespaceKind: "wildcard", namespace: null };
  }
  if (alias !== undefined && isValidSassNamespace(alias)) {
    return { namespaceKind: "alias", namespace: alias };
  }
  return {
    namespaceKind: "default",
    namespace: defaultSassNamespaceForSource(source),
  };
}

function defaultSassNamespaceForSource(source: string): string | null {
  const clean = source.split(/[?#]/, 1)[0]!.replace(/\/+$/g, "");
  const segment = clean.split("/").at(-1) ?? clean;
  const packageSegment = segment.split(":").at(-1) ?? segment;
  const stem = packageSegment.includes(".")
    ? packageSegment.slice(0, packageSegment.lastIndexOf("."))
    : packageSegment;
  const namespace = stem.startsWith("_") ? stem.slice(1) : stem;
  return isValidSassNamespace(namespace) ? namespace : null;
}

function isValidSassNamespace(value: string): boolean {
  return /^[A-Za-z_-][A-Za-z0-9_-]*$/.test(value);
}

interface SassSymbolTargetContext {
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly variableDecls: readonly SassSymbolDeclHIR[];
  readonly mixinTargets: ReadonlySet<string>;
  readonly functionTargets: ReadonlySet<string>;
  readonly allowUnresolvedFunctionCalls: boolean;
}

function collectSassSymbolDecls(
  root: Root,
  syntax?: StylePreprocessorSymbolSyntax,
): readonly SassSymbolDeclHIR[] {
  if (syntax === "less") return collectLessVariableDecls(root);

  const decls: SassSymbolDeclHIR[] = [];

  root.walkDecls((decl) => {
    const name = parseSassVariableDeclName(decl.prop);
    if (!name) return;
    const range = findDeclPropTokenRange(decl, decl.prop.length);
    if (!range) return;
    decls.push(
      makeSassSymbolDecl({
        symbolKind: "variable",
        name,
        range,
        ruleRange: rangeForSassVariableDeclScope(decl),
      }),
    );
  });

  root.walkAtRules((atrule) => {
    if (atrule.name !== "mixin" && atrule.name !== "function") return;
    const callable = parseSassCallableName(atrule.params);
    const ruleRange = rangeForSourceNode(atrule);
    if (callable) {
      const range = findAtRuleParamValueTokenRange(atrule, callable.offset, callable.raw.length);
      if (range) {
        decls.push(
          makeSassSymbolDecl({
            symbolKind: atrule.name === "mixin" ? "mixin" : "function",
            name: callable.name,
            range,
            ruleRange,
          }),
        );
      }
    }
    for (const variable of findSassVariableMatches(atrule.params)) {
      const range = findAtRuleParamValueTokenRange(atrule, variable.offset, variable.raw.length);
      if (!range) continue;
      decls.push(
        makeSassSymbolDecl({
          symbolKind: "variable",
          name: variable.name,
          range,
          ruleRange,
        }),
      );
    }
  });

  return decls;
}

function collectLessVariableDecls(root: Root): readonly SassSymbolDeclHIR[] {
  const decls: SassSymbolDeclHIR[] = [];
  root.walkAtRules((atrule) => {
    if (!isLessVariableDeclAtRule(atrule)) return;
    const range = findLessVariableDeclNameRange(atrule);
    if (!range) return;
    decls.push(
      makeSassSymbolDecl({
        syntax: "less",
        symbolKind: "variable",
        name: atrule.name,
        range,
        ruleRange: rangeForLessVariableDeclScope(atrule),
      }),
    );
  });
  return decls;
}

function collectSassSymbolTargetContext(
  root: Root,
  sassSymbolDecls: readonly SassSymbolDeclHIR[],
  sassModuleUses: readonly SassModuleUseHIR[],
  syntax?: StylePreprocessorSymbolSyntax,
): SassSymbolTargetContext {
  const mixinTargets = new Set<string>();
  const functionTargets = new Set<string>();

  if (syntax !== "less") {
    root.walkAtRules((atrule) => {
      if (atrule.name !== "mixin" && atrule.name !== "function") return;
      const callable = parseSassCallableName(atrule.params);
      if (callable) {
        if (atrule.name === "mixin") mixinTargets.add(callable.name);
        else functionTargets.add(callable.name);
      }
    });
  }

  return {
    ...(syntax ? { syntax } : {}),
    variableDecls: sassSymbolDecls.filter(
      (decl) => decl.symbolKind === "variable" && (decl.syntax ?? "sass") === (syntax ?? "sass"),
    ),
    mixinTargets,
    functionTargets,
    allowUnresolvedFunctionCalls: sassModuleUses.some(
      (moduleUse) => moduleUse.namespaceKind === "wildcard",
    ),
  };
}

function makeSassSymbolDecl(args: {
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly range: Range;
  readonly ruleRange: Range;
}): SassSymbolDeclHIR {
  return {
    kind: "sassSymbolDecl",
    id: `sass-decl:${args.symbolKind}:${args.name}:${args.range.start.line}:${args.range.start.character}`,
    ...(args.syntax ? { syntax: args.syntax } : {}),
    symbolKind: args.symbolKind,
    name: args.name,
    range: args.range,
    ruleRange: args.ruleRange,
  };
}

function collectDirectSassSymbolOccurrences(
  nodes: ChildNode[] | undefined,
  selectorName: string,
  ruleRange: Range,
  targets: SassSymbolTargetContext,
): readonly SassSymbolOccurrenceHIR[] {
  if (!nodes) return [];

  const occurrences: SassSymbolOccurrenceHIR[] = [];
  for (const node of nodes) {
    if (node.type === "decl") {
      if (targets.syntax === "less") {
        pushLessDeclarationValueOccurrences(occurrences, node, selectorName, ruleRange, targets);
        continue;
      }
      pushSassDeclarationValueOccurrences(occurrences, node, selectorName, ruleRange, targets);
      continue;
    }
    if (node.type !== "atrule") continue;
    if (targets.syntax === "less") {
      pushLessAtRuleParamOccurrences(occurrences, node, selectorName, ruleRange, targets);
      continue;
    }
    if (node.name === "mixin" || node.name === "function") continue;
    pushSassAtRuleParamOccurrences(occurrences, node, selectorName, ruleRange, targets);
  }
  return occurrences;
}

function collectDirectSassModuleMemberRefs(
  nodes: ChildNode[] | undefined,
  selectorName: string,
  ruleRange: Range,
): readonly SassModuleMemberRefHIR[] {
  if (!nodes) return [];

  const refs: SassModuleMemberRefHIR[] = [];
  for (const node of nodes) {
    if (node.type === "decl") {
      pushSassDeclarationValueModuleMemberRefs(refs, node, selectorName, ruleRange);
      continue;
    }
    if (node.type !== "atrule") continue;
    if (node.name === "mixin" || node.name === "function") continue;
    pushSassAtRuleParamModuleMemberRefs(refs, node, selectorName, ruleRange);
  }
  return refs;
}

function pushSassDeclarationValueOccurrences(
  occurrences: SassSymbolOccurrenceHIR[],
  node: Extract<ChildNode, { type: "decl" }>,
  selectorName: string,
  ruleRange: Range,
  targets: SassSymbolTargetContext,
): void {
  for (const match of findSassVariableMatches(node.value)) {
    const range = findDeclValueTokenRange(node, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        selectorName,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        resolution: resolveSassVariableReference(targets, match.name, range),
        range,
        ruleRange,
      }),
    );
  }

  for (const match of findSassFunctionCallMatches(
    node.value,
    targets.functionTargets,
    targets.allowUnresolvedFunctionCalls,
  )) {
    const range = findDeclValueTokenRange(node, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        selectorName,
        symbolKind: "function",
        name: match.name,
        role: "call",
        resolution: targets.functionTargets.has(match.name) ? "resolved" : "unresolved",
        range,
        ruleRange,
      }),
    );
  }
}

function pushLessDeclarationValueOccurrences(
  occurrences: SassSymbolOccurrenceHIR[],
  node: Extract<ChildNode, { type: "decl" }>,
  selectorName: string,
  ruleRange: Range,
  targets: SassSymbolTargetContext,
): void {
  for (const match of findLessVariableMatches(node.value)) {
    const range = findDeclValueTokenRange(node, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        syntax: "less",
        selectorName,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        resolution: resolveSassVariableReference(targets, match.name, range),
        range,
        ruleRange,
      }),
    );
  }
}

function pushSassDeclarationValueModuleMemberRefs(
  refs: SassModuleMemberRefHIR[],
  node: Extract<ChildNode, { type: "decl" }>,
  selectorName: string,
  ruleRange: Range,
): void {
  for (const match of findSassModuleVariableMatches(node.value)) {
    const range = findDeclValueTokenRange(node, match.offset, match.raw.length);
    if (!range) continue;
    refs.push(
      makeSassModuleMemberRef({
        selectorName,
        namespace: match.namespace,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        range,
        ruleRange,
      }),
    );
  }

  for (const match of findSassModuleFunctionCallMatches(node.value)) {
    const range = findDeclValueTokenRange(node, match.offset, match.raw.length);
    if (!range) continue;
    refs.push(
      makeSassModuleMemberRef({
        selectorName,
        namespace: match.namespace,
        symbolKind: "function",
        name: match.name,
        role: "call",
        range,
        ruleRange,
      }),
    );
  }
}

function pushSassAtRuleParamOccurrences(
  occurrences: SassSymbolOccurrenceHIR[],
  atrule: AtRule,
  selectorName: string,
  ruleRange: Range,
  targets: SassSymbolTargetContext,
): void {
  const ignoredFunctionCallOffsets = new Set<number>();

  if (atrule.name === "include") {
    const callable = parseSassCallableName(atrule.params);
    if (callable) {
      ignoredFunctionCallOffsets.add(callable.offset);
      const range = findAtRuleParamValueTokenRange(atrule, callable.offset, callable.raw.length);
      if (range) {
        occurrences.push(
          makeSassSymbolOccurrence({
            selectorName,
            symbolKind: "mixin",
            name: callable.name,
            role: "include",
            resolution: targets.mixinTargets.has(callable.name) ? "resolved" : "unresolved",
            range,
            ruleRange,
          }),
        );
      }
    }
  }

  for (const match of findSassVariableMatches(atrule.params)) {
    const range = findAtRuleParamValueTokenRange(atrule, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        selectorName,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        resolution: resolveSassVariableReference(targets, match.name, range),
        range,
        ruleRange,
      }),
    );
  }

  for (const match of findSassFunctionCallMatches(
    atrule.params,
    targets.functionTargets,
    targets.allowUnresolvedFunctionCalls,
  )) {
    if (ignoredFunctionCallOffsets.has(match.offset)) continue;
    const range = findAtRuleParamValueTokenRange(atrule, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        selectorName,
        symbolKind: "function",
        name: match.name,
        role: "call",
        resolution: targets.functionTargets.has(match.name) ? "resolved" : "unresolved",
        range,
        ruleRange,
      }),
    );
  }
}

function pushLessAtRuleParamOccurrences(
  occurrences: SassSymbolOccurrenceHIR[],
  atrule: AtRule,
  selectorName: string,
  ruleRange: Range,
  targets: SassSymbolTargetContext,
): void {
  for (const match of findLessVariableMatches(atrule.params)) {
    const range = findAtRuleParamValueTokenRange(atrule, match.offset, match.raw.length);
    if (!range) continue;
    occurrences.push(
      makeSassSymbolOccurrence({
        syntax: "less",
        selectorName,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        resolution: resolveSassVariableReference(targets, match.name, range),
        range,
        ruleRange,
      }),
    );
  }
}

function pushSassAtRuleParamModuleMemberRefs(
  refs: SassModuleMemberRefHIR[],
  atrule: AtRule,
  selectorName: string,
  ruleRange: Range,
): void {
  const includeMemberOffsets = new Set<number>();
  if (atrule.name === "include") {
    const callable = parseSassModuleCallableName(atrule.params);
    if (callable) {
      includeMemberOffsets.add(callable.offset);
      const range = findAtRuleParamValueTokenRange(atrule, callable.offset, callable.raw.length);
      if (range) {
        refs.push(
          makeSassModuleMemberRef({
            selectorName,
            namespace: callable.namespace,
            symbolKind: "mixin",
            name: callable.name,
            role: "include",
            range,
            ruleRange,
          }),
        );
      }
    }
  }

  for (const match of findSassModuleVariableMatches(atrule.params)) {
    const range = findAtRuleParamValueTokenRange(atrule, match.offset, match.raw.length);
    if (!range) continue;
    refs.push(
      makeSassModuleMemberRef({
        selectorName,
        namespace: match.namespace,
        symbolKind: "variable",
        name: match.name,
        role: "reference",
        range,
        ruleRange,
      }),
    );
  }

  for (const match of findSassModuleFunctionCallMatches(atrule.params)) {
    if (includeMemberOffsets.has(match.offset)) continue;
    const range = findAtRuleParamValueTokenRange(atrule, match.offset, match.raw.length);
    if (!range) continue;
    refs.push(
      makeSassModuleMemberRef({
        selectorName,
        namespace: match.namespace,
        symbolKind: "function",
        name: match.name,
        role: "call",
        range,
        ruleRange,
      }),
    );
  }
}

function makeSassSymbolOccurrence(args: {
  readonly syntax?: StylePreprocessorSymbolSyntax;
  readonly selectorName: string;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly role: SassSymbolRole;
  readonly resolution: SassSymbolResolution;
  readonly range: Range;
  readonly ruleRange: Range;
}): SassSymbolOccurrenceHIR {
  return {
    kind: "sassSymbol",
    id: `sass:${args.selectorName}:${args.symbolKind}:${args.name}:${args.range.start.line}:${args.range.start.character}`,
    selectorName: args.selectorName,
    ...(args.syntax ? { syntax: args.syntax } : {}),
    symbolKind: args.symbolKind,
    name: args.name,
    role: args.role,
    resolution: args.resolution,
    range: args.range,
    ruleRange: args.ruleRange,
  };
}

function makeSassModuleMemberRef(args: {
  readonly selectorName: string;
  readonly namespace: string;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly role: SassSymbolRole;
  readonly range: Range;
  readonly ruleRange: Range;
}): SassModuleMemberRefHIR {
  return {
    kind: "sassModuleMemberRef",
    id: `sass-module:${args.selectorName}:${args.namespace}:${args.symbolKind}:${args.name}:${args.range.start.line}:${args.range.start.character}`,
    selectorName: args.selectorName,
    namespace: args.namespace,
    symbolKind: args.symbolKind,
    name: args.name,
    role: args.role,
    range: args.range,
    ruleRange: args.ruleRange,
  };
}

function resolveSassVariableReference(
  targets: SassSymbolTargetContext,
  name: string,
  range: Range,
): SassSymbolResolution {
  const matchingDecls = targets.variableDecls.filter((decl) => decl.name === name);
  if (matchingDecls.length === 0) return "unresolved";

  const localDecl = matchingDecls
    .filter((decl) => !isFileScopeSassVariableDecl(decl))
    .filter((decl) => rangeContains(decl.ruleRange, range.start.line, range.start.character))
    .toSorted(compareSassVariableDeclScopeSpecificity)[0];
  if (localDecl) return "resolved";
  return matchingDecls.some(isFileScopeSassVariableDecl) ? "resolved" : "unresolved";
}

function isFileScopeSassVariableDecl(decl: SassSymbolDeclHIR): boolean {
  return (
    decl.range.start.line === decl.ruleRange.start.line &&
    decl.range.start.character === decl.ruleRange.start.character
  );
}

function compareSassVariableDeclScopeSpecificity(
  a: SassSymbolDeclHIR,
  b: SassSymbolDeclHIR,
): number {
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

function parseSassVariableDeclName(property: string): string | null {
  return /^\$([A-Za-z_-][A-Za-z0-9_-]*)$/.exec(property.trim())?.[1] ?? null;
}

function isLessVariableDeclAtRule(atrule: AtRule): boolean {
  return (
    /^[A-Za-z_-][A-Za-z0-9_-]*$/.test(atrule.name) && (atrule.raws.afterName ?? "").includes(":")
  );
}

function parseSassCallableName(raw: string): { name: string; raw: string; offset: number } | null {
  const match = /^\s*([A-Za-z_-][A-Za-z0-9_-]*)/.exec(raw);
  const name = match?.[1];
  if (!name) return null;
  const offset = raw.indexOf(name);
  const next = raw.slice(offset + name.length).match(/\S/)?.[0];
  if (next === ".") return null;
  return {
    name,
    raw: name,
    offset,
  };
}

function parseSassModuleCallableName(
  raw: string,
): { namespace: string; name: string; raw: string; offset: number } | null {
  const match = /^\s*([A-Za-z_-][A-Za-z0-9_-]*)\.([A-Za-z_-][A-Za-z0-9_-]*)/.exec(raw);
  const namespace = match?.[1];
  const name = match?.[2];
  if (!namespace || !name) return null;
  const offset = raw.indexOf(`${namespace}.${name}`) + namespace.length + 1;
  return {
    namespace,
    name,
    raw: name,
    offset,
  };
}

function findSassVariableMatches(
  raw: string,
): Array<{ name: string; raw: string; offset: number }> {
  const matches: Array<{ name: string; raw: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      continue;
    }
    if (ch !== "$") continue;
    const match = /^\$([A-Za-z_-][A-Za-z0-9_-]*)/.exec(raw.slice(index));
    const token = match?.[0];
    const name = match?.[1];
    if (!token || !name) continue;
    if (isSassModuleQualifiedReference(raw, index)) continue;
    matches.push({ name, raw: token, offset: index });
    index += token.length - 1;
  }

  return matches;
}

function findLessVariableMatches(
  raw: string,
): Array<{ name: string; raw: string; offset: number }> {
  const matches: Array<{ name: string; raw: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      continue;
    }
    if (ch !== "@") continue;
    if (raw[index - 1] === "@") continue;
    const match = /^@([A-Za-z_-][A-Za-z0-9_-]*)/.exec(raw.slice(index));
    const token = match?.[0];
    const name = match?.[1];
    if (!token || !name) continue;
    matches.push({ name, raw: token, offset: index });
    index += token.length - 1;
  }

  return matches;
}

function findSassModuleVariableMatches(
  raw: string,
): Array<{ namespace: string; name: string; raw: string; offset: number }> {
  const matches: Array<{ namespace: string; name: string; raw: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      continue;
    }
    if (ch !== "$") continue;
    const namespace = readSassModuleQualifier(raw, index);
    if (!namespace) continue;
    const match = /^\$([A-Za-z_-][A-Za-z0-9_-]*)/.exec(raw.slice(index));
    const token = match?.[0];
    const name = match?.[1];
    if (!token || !name) continue;
    matches.push({ namespace, name, raw: token, offset: index });
    index += token.length - 1;
  }

  return matches;
}

function findSassFunctionCallMatches(
  raw: string,
  functionTargets: ReadonlySet<string>,
  allowUnresolvedFunctionCalls = false,
): Array<{ name: string; raw: string; offset: number }> {
  const matches: Array<{ name: string; raw: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      continue;
    }
    if (!/[A-Za-z_-]/.test(ch)) continue;
    const match = /^[A-Za-z_-][A-Za-z0-9_-]*/.exec(raw.slice(index));
    const name = match?.[0];
    if (!name) continue;
    if (isSassModuleQualifiedReference(raw, index)) {
      index += name.length - 1;
      continue;
    }
    const nextNonWhitespace = raw.slice(index + name.length).match(/\S/)?.[0];
    if ((functionTargets.has(name) || allowUnresolvedFunctionCalls) && nextNonWhitespace === "(") {
      matches.push({ name, raw: name, offset: index });
    }
    index += name.length - 1;
  }

  return matches;
}

function findSassModuleFunctionCallMatches(
  raw: string,
): Array<{ namespace: string; name: string; raw: string; offset: number }> {
  const matches: Array<{ namespace: string; name: string; raw: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      quote = ch;
      continue;
    }
    if (!/[A-Za-z_-]/.test(ch)) continue;
    const match = /^[A-Za-z_-][A-Za-z0-9_-]*/.exec(raw.slice(index));
    const name = match?.[0];
    if (!name) continue;
    const namespace = readSassModuleQualifier(raw, index);
    const nextNonWhitespace = raw.slice(index + name.length).match(/\S/)?.[0];
    if (namespace && nextNonWhitespace === "(") {
      matches.push({ namespace, name, raw: name, offset: index });
    }
    index += name.length - 1;
  }

  return matches;
}

function isSassModuleQualifiedReference(raw: string, start: number): boolean {
  return readSassModuleQualifier(raw, start) !== null;
}

function readSassModuleQualifier(raw: string, start: number): string | null {
  if (start <= 1 || raw[start - 1] !== ".") return null;
  const namespaceMatch = /[A-Za-z_-][A-Za-z0-9_-]*$/.exec(raw.slice(0, start - 1));
  return namespaceMatch?.[0] ?? null;
}

function looksLikeStyleRequest(value: string): boolean {
  return /^\.{0,2}\/.+\.(?:css|scss|sass|less)$/iu.test(value);
}

function isQuotedStyleRequest(value: string): boolean {
  const unquoted = unquoteCssString(value);
  return Boolean(unquoted && looksLikeStyleRequest(unquoted));
}

function unquoteCssString(value: string): string | null {
  const match = /^(?:"([^"]+)"|'([^']+)')$/u.exec(value);
  return match?.[1] ?? match?.[2] ?? null;
}

function findValueIdentifierMatches(
  raw: string,
  valueNames: ReadonlySet<string>,
): Array<{ name: string; offset: number }> {
  const matches: Array<{ name: string; offset: number }> = [];
  let quote: "'" | '"' | null = null;
  let identifierStart = -1;

  const flushIdentifier = (endIndex: number) => {
    if (identifierStart === -1) return;
    const name = raw.slice(identifierStart, endIndex);
    if (valueNames.has(name)) {
      matches.push({ name, offset: identifierStart });
    }
    identifierStart = -1;
  };

  for (let index = 0; index < raw.length; index += 1) {
    const ch = raw[index]!;
    if (quote) {
      if (ch === "\\" && index + 1 < raw.length) {
        index += 1;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === "'" || ch === '"') {
      flushIdentifier(index);
      quote = ch;
      continue;
    }
    if (identifierStart === -1) {
      if (/[\p{L}_-]/u.test(ch)) {
        identifierStart = index;
      }
      continue;
    }
    if (!/[\p{L}\p{N}\p{M}_-]/u.test(ch)) {
      flushIdentifier(index);
    }
  }

  flushIdentifier(raw.length);
  return matches;
}

function parseKeyframesName(atrule: AtRule): { name: string; range: Range } | null {
  const rawName = atrule.params.trim();
  if (!rawName) return null;
  const match = /^(?:"([^"]+)"|'([^']+)'|([\p{L}_][\p{L}\p{N}\p{M}_-]*))$/u.exec(rawName);
  const name = match?.[1] ?? match?.[2] ?? match?.[3];
  if (!name) return null;
  const range = findAtRuleParamTokenRange(atrule, rawName, rawName.length);
  return range ? { name, range } : null;
}

function findAnimationNameTokens(
  node: Extract<ChildNode, { type: "decl" }>,
): readonly AnimationNameRefHIR[] {
  if (node.prop === "animation-name") {
    return enumerateAnimationSegments(node).flatMap((segment, index) => {
      const token = parseStandaloneAnimationNameToken(segment.raw);
      if (!token) return [];
      const range = findDeclValueTokenRange(node, segment.offset + token.offset, token.raw.length);
      if (!range) return [];
      return [
        {
          kind: "animationNameRef",
          id: `animation-ref:${node.source?.start?.line ?? 0}:${index}:${token.name}`,
          name: token.name,
          range,
          property: "animation-name",
        },
      ];
    });
  }

  if (node.prop === "animation") {
    return enumerateAnimationSegments(node).flatMap((segment, index) => {
      const token = findAnimationShorthandNameToken(segment.raw);
      if (!token) return [];
      const range = findDeclValueTokenRange(node, segment.offset + token.offset, token.raw.length);
      if (!range) return [];
      return [
        {
          kind: "animationNameRef",
          id: `animation-ref:${node.source?.start?.line ?? 0}:${index}:${token.name}`,
          name: token.name,
          range,
          property: "animation",
        },
      ];
    });
  }

  return [];
}

function enumerateAnimationSegments(
  node: Extract<ChildNode, { type: "decl" }>,
): Array<{ raw: string; offset: number }> {
  const value = node.value;
  const segments: Array<{ raw: string; offset: number }> = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < value.length; index++) {
    const ch = value[index];
    if (ch === "(" || ch === "[") depth += 1;
    else if (ch === ")" || ch === "]") depth -= 1;
    else if (ch === "," && depth === 0) {
      segments.push({ raw: value.slice(start, index), offset: start });
      start = index + 1;
    }
  }
  segments.push({ raw: value.slice(start), offset: start });
  return segments;
}

function parseStandaloneAnimationNameToken(
  rawSegment: string,
): { name: string; raw: string; offset: number } | null {
  const trimmed = rawSegment.trim();
  if (!trimmed) return null;
  const match = /^(?:"([^"]+)"|'([^']+)'|([\p{L}_][\p{L}\p{N}\p{M}_-]*))$/u.exec(trimmed);
  const name = match?.[1] ?? match?.[2] ?? match?.[3];
  if (!name || isReservedAnimationKeyword(name)) return null;
  return {
    name,
    raw: trimmed,
    offset: rawSegment.indexOf(trimmed),
  };
}

function findAnimationShorthandNameToken(
  rawSegment: string,
): { name: string; raw: string; offset: number } | null {
  const tokens = enumerateWhitespaceTokens(rawSegment);
  for (const token of tokens) {
    const match = /^(?:"([^"]+)"|'([^']+)'|([\p{L}_][\p{L}\p{N}\p{M}_-]*))$/u.exec(token.raw);
    const name = match?.[1] ?? match?.[2] ?? match?.[3];
    if (!name) continue;
    if (isReservedAnimationKeyword(name)) continue;
    if (isAnimationFunctionToken(token.raw)) continue;
    return {
      name,
      raw: token.raw,
      offset: token.offset,
    };
  }
  return null;
}

function enumerateWhitespaceTokens(segment: string): Array<{ raw: string; offset: number }> {
  const tokens: Array<{ raw: string; offset: number }> = [];
  let depth = 0;
  let start = -1;
  for (let index = 0; index < segment.length; index++) {
    const ch = segment.charAt(index);
    if (ch === "(" || ch === "[") {
      depth += 1;
      if (start === -1) start = index;
      continue;
    }
    if (ch === ")" || ch === "]") {
      depth -= 1;
      continue;
    }
    if (depth === 0 && /\s/.test(ch)) {
      pushWhitespaceToken(tokens, segment, start, index);
      start = -1;
      continue;
    }
    if (start === -1) start = index;
  }
  pushWhitespaceToken(tokens, segment, start, segment.length);
  return tokens;
}

function pushWhitespaceToken(
  tokens: Array<{ raw: string; offset: number }>,
  segment: string,
  start: number,
  end: number,
): void {
  if (start === -1) return;
  const raw = segment.slice(start, end).trim();
  if (!raw) return;
  tokens.push({ raw, offset: segment.indexOf(raw, start) });
}

function isReservedAnimationKeyword(token: string): boolean {
  const lower = token.toLowerCase();
  return (
    lower === "none" ||
    lower === "infinite" ||
    lower === "normal" ||
    lower === "reverse" ||
    lower === "alternate" ||
    lower === "alternate-reverse" ||
    lower === "forwards" ||
    lower === "backwards" ||
    lower === "both" ||
    lower === "running" ||
    lower === "paused" ||
    lower === "linear" ||
    lower === "ease" ||
    lower === "ease-in" ||
    lower === "ease-out" ||
    lower === "ease-in-out" ||
    lower === "step-start" ||
    lower === "step-end" ||
    lower === "initial" ||
    lower === "inherit" ||
    lower === "unset" ||
    lower === "revert" ||
    lower === "revert-layer"
  );
}

function isAnimationFunctionToken(token: string): boolean {
  if (/^-?\d*\.?\d+(ms|s)$/i.test(token)) return true;
  if (/^-?\d*\.?\d+$/.test(token)) return true;
  return /^[a-z-]+\(.+\)$/i.test(token);
}

function findDeclValueTokenRange(
  node: Extract<ChildNode, { type: "decl" }>,
  valueOffset: number,
  tokenLength: number,
): Range | null {
  const source = node.source;
  if (!source?.start) return null;
  const propIndex = node.toString().indexOf(node.prop);
  const valueIndex = node.toString().indexOf(node.value, propIndex + node.prop.length);
  if (valueIndex < 0) return null;
  const baseOffset = source.start.offset + valueIndex;
  const startOffset = baseOffset + valueOffset;
  const endOffset = startOffset + tokenLength;
  const startPos = source.input.fromOffset(startOffset);
  const endPos = source.input.fromOffset(endOffset);
  if (!startPos || !endPos) return null;
  return {
    start: { line: startPos.line - 1, character: startPos.col - 1 },
    end: { line: endPos.line - 1, character: endPos.col - 1 },
  };
}

function findDeclPropTokenRange(node: Declaration, tokenLength: number): Range | null {
  const source = node.source;
  if (!source?.start) return null;
  const propIndex = node.toString().indexOf(node.prop);
  if (propIndex < 0) return null;
  const startOffset = source.start.offset + propIndex;
  const endOffset = startOffset + tokenLength;
  const startPos = source.input.fromOffset(startOffset);
  const endPos = source.input.fromOffset(endOffset);
  if (!startPos || !endPos) return null;
  return {
    start: { line: startPos.line - 1, character: startPos.col - 1 },
    end: { line: endPos.line - 1, character: endPos.col - 1 },
  };
}

function rangeForDeclNode(node: Declaration): Range {
  const start = node.source?.start;
  const end = node.source?.end;
  return {
    start: start
      ? { line: start.line - 1, character: start.column - 1 }
      : { line: 0, character: 0 },
    end: end ? { line: end.line - 1, character: end.column - 1 } : { line: 0, character: 0 },
  };
}

function rangeForSassVariableDeclScope(node: Declaration): Range {
  const parent = node.parent;
  if (parent?.type === "rule" || parent?.type === "atrule") {
    return rangeForSourceNode(parent);
  }
  return rangeForDeclNode(node);
}

function rangeForLessVariableDeclScope(atrule: AtRule): Range {
  const parent = atrule.parent;
  if (parent?.type === "rule" || parent?.type === "atrule") {
    return rangeForSourceNode(parent);
  }
  return rangeForSourceNode(atrule);
}

function findLessVariableDeclNameRange(atrule: AtRule): Range | null {
  const source = atrule.source;
  if (!source?.start) return null;
  const startOffset = source.start.offset;
  const endOffset = startOffset + atrule.name.length + 1;
  const startPos = source.input.fromOffset(startOffset);
  const endPos = source.input.fromOffset(endOffset);
  if (!startPos || !endPos) return null;
  return {
    start: { line: startPos.line - 1, character: startPos.col - 1 },
    end: { line: endPos.line - 1, character: endPos.col - 1 },
  };
}

function findAtRuleParamTokenRange(
  atrule: AtRule,
  token: string,
  tokenLength: number,
): Range | null {
  const source = atrule.source;
  if (!source?.start) return null;
  const paramsIndex = atrule.toString().indexOf(atrule.params);
  const tokenIndex = atrule.params.indexOf(token);
  if (paramsIndex < 0 || tokenIndex < 0) return null;
  const startOffset = source.start.offset + paramsIndex + tokenIndex;
  const endOffset = startOffset + tokenLength;
  const startPos = source.input.fromOffset(startOffset);
  const endPos = source.input.fromOffset(endOffset);
  if (!startPos || !endPos) return null;
  return {
    start: { line: startPos.line - 1, character: startPos.col - 1 },
    end: { line: endPos.line - 1, character: endPos.col - 1 },
  };
}

function findAtRuleParamValueTokenRange(
  atrule: AtRule,
  valueOffset: number,
  tokenLength: number,
): Range | null {
  const source = atrule.source;
  if (!source?.start) return null;
  const paramsIndex = atrule.toString().indexOf(atrule.params);
  if (paramsIndex < 0) return null;
  const startOffset = source.start.offset + paramsIndex + valueOffset;
  const endOffset = startOffset + tokenLength;
  const startPos = source.input.fromOffset(startOffset);
  const endPos = source.input.fromOffset(endOffset);
  if (!startPos || !endPos) return null;
  return {
    start: { line: startPos.line - 1, character: startPos.col - 1 },
    end: { line: endPos.line - 1, character: endPos.col - 1 },
  };
}

function recordAtRootInlineRule(
  atrule: AtRule,
  selectorDecls: SelectorDeclHIR[],
  sassSymbols: SassSymbolOccurrenceHIR[],
  sassModuleMemberRefs: SassModuleMemberRefHIR[],
  sassSymbolTargets: SassSymbolTargetContext,
): void {
  const selector = atrule.params.trim();
  const { declarations } = collectDeclarationsAndComposes(atrule.nodes);
  const ruleRange = rangeForSourceNode(atrule);
  const groups = selector.split(",").map((s) => s.trim());

  for (const raw of groups) {
    for (const className of extractClassNames(raw)) {
      const start = atrule.source?.start;
      const baseColumn = (start?.column ?? 1) - 1 + "@at-root ".length;
      selectorDecls.push({
        kind: "selector",
        id: `selector:${className}:${start?.line ?? 1}:${baseColumn}`,
        name: className,
        canonicalName: className,
        viewKind: "canonical",
        range: atRootTokenRange(start?.line ?? 1, baseColumn, className, raw),
        fullSelector: raw,
        declarations,
        ruleRange,
        composes: [],
        nestedSafety: "flat",
      });
      sassSymbols.push(
        ...collectDirectSassSymbolOccurrences(
          atrule.nodes,
          className,
          ruleRange,
          sassSymbolTargets,
        ),
      );
      sassModuleMemberRefs.push(
        ...collectDirectSassModuleMemberRefs(atrule.nodes, className, ruleRange),
      );
    }
  }
}

function classifyNestedSafety(
  isNested: boolean,
  bemSuffix: BemSuffixInfo | null,
): NestedSelectorSafety {
  if (bemSuffix) return "bemSuffixSafe";
  if (isNested) return "nestedUnsafe";
  return "flat";
}

function compareSelectors(a: SelectorDeclHIR, b: SelectorDeclHIR): number {
  return compareNamedStyleFacts(a, b);
}

function compareNamedStyleFacts(
  a: { range: { start: { line: number; character: number } }; name: string },
  b: { range: { start: { line: number; character: number } }; name: string },
): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  return a.name.localeCompare(b.name);
}

function compareSassSymbolOccurrences(
  a: SassSymbolOccurrenceHIR,
  b: SassSymbolOccurrenceHIR,
): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  const selectorCompare = a.selectorName.localeCompare(b.selectorName);
  if (selectorCompare !== 0) return selectorCompare;
  const kindCompare = a.symbolKind.localeCompare(b.symbolKind);
  if (kindCompare !== 0) return kindCompare;
  const nameCompare = a.name.localeCompare(b.name);
  if (nameCompare !== 0) return nameCompare;
  return a.role.localeCompare(b.role);
}

function compareSassSymbolDecls(a: SassSymbolDeclHIR, b: SassSymbolDeclHIR): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  const kindCompare = a.symbolKind.localeCompare(b.symbolKind);
  if (kindCompare !== 0) return kindCompare;
  return a.name.localeCompare(b.name);
}

function compareSassModuleUses(a: SassModuleUseHIR, b: SassModuleUseHIR): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  const sourceCompare = a.source.localeCompare(b.source);
  if (sourceCompare !== 0) return sourceCompare;
  const kindCompare = a.namespaceKind.localeCompare(b.namespaceKind);
  if (kindCompare !== 0) return kindCompare;
  return (a.namespace ?? "").localeCompare(b.namespace ?? "");
}

function compareSassModuleForwards(a: SassModuleForwardHIR, b: SassModuleForwardHIR): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  return a.source.localeCompare(b.source);
}

function compareSassModuleMemberRefs(a: SassModuleMemberRefHIR, b: SassModuleMemberRefHIR): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  const selectorCompare = a.selectorName.localeCompare(b.selectorName);
  if (selectorCompare !== 0) return selectorCompare;
  const namespaceCompare = a.namespace.localeCompare(b.namespace);
  if (namespaceCompare !== 0) return namespaceCompare;
  const kindCompare = a.symbolKind.localeCompare(b.symbolKind);
  if (kindCompare !== 0) return kindCompare;
  const nameCompare = a.name.localeCompare(b.name);
  if (nameCompare !== 0) return nameCompare;
  return a.role.localeCompare(b.role);
}
