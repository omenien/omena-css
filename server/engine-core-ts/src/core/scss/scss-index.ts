import type { StyleDocumentHIR } from "../hir/style-types";
import { contentHash } from "../util/hash";
import { LruMap } from "../util/lru-map";
import {
  expandStyleDocumentWithTransform,
  type ClassnameTransformMode,
} from "./classname-transform";
import { parseStyleDocument } from "./scss-parser";

export { parseStyleDocument, buildChildContext, type ParentContext } from "./scss-parser";
export { findBemSuffixSpan } from "./bem-suffix";
export { enumerateGroups } from "./scss-selector-utils";

export interface StyleIndexEntry {
  readonly hash: string;
  readonly mode: ClassnameTransformMode;
  readonly styleDocument: StyleDocumentHIR;
}

export type StyleDocumentBuilder = (filePath: string, content: string) => StyleDocumentHIR;

const CLASSNAME_TRANSFORM_MODES: readonly ClassnameTransformMode[] = [
  "asIs",
  "camelCase",
  "camelCaseOnly",
  "dashes",
  "dashesOnly",
];

/**
 * Content-hashed LRU cache for transformed style-document HIR.
 *
 * - Hit path: provider asks for a file + its current content; the
 *   cache returns the previously-built `StyleIndexEntry` by
 *   reference identity when (content-hash, mode) match.
 * - Miss path: parse `StyleDocumentHIR` → apply transform aliases →
 *   store the transformed style document.
 * - Mode change: `setMode` clears the whole LRU. Keys that were
 *   valid under the old mode may not exist under the new one, so
 *   a full rebuild is correct. 500-entry LRU makes this cheap.
 */
export class StyleIndexCache {
  private readonly lru: LruMap<string, StyleIndexEntry>;
  private readonly buildBaseStyleDocument: StyleDocumentBuilder;

  constructor(options: { max: number; buildStyleDocument?: StyleDocumentBuilder }) {
    this.lru = new LruMap(options.max);
    this.buildBaseStyleDocument =
      options.buildStyleDocument ?? ((filePath, content) => parseStyleDocument(content, filePath));
  }

  getStyleDocument(
    filePath: string,
    content: string,
    mode: ClassnameTransformMode = "asIs",
  ): StyleDocumentHIR {
    return this.getEntry(filePath, content, mode).styleDocument;
  }

  getEntry(
    filePath: string,
    content: string,
    mode: ClassnameTransformMode = "asIs",
  ): StyleIndexEntry {
    const cacheKey = this.key(filePath, mode);
    const hash = contentHash(content);
    const cached = this.lru.get(cacheKey);
    if (cached && cached.hash === hash && cached.mode === mode) {
      this.lru.touch(cacheKey, cached);
      return cached;
    }

    const base = this.buildBaseStyleDocument(filePath, content);
    const styleDocument = expandStyleDocumentWithTransform(base, mode);
    const entry: StyleIndexEntry = {
      hash,
      mode,
      styleDocument,
    };
    this.lru.set(cacheKey, entry);
    return entry;
  }

  peekEntry(filePath: string, mode: ClassnameTransformMode = "asIs"): StyleIndexEntry | null {
    return this.lru.get(this.key(filePath, mode)) ?? null;
  }

  invalidate(filePath: string): void {
    for (const mode of CLASSNAME_TRANSFORM_MODES) {
      this.lru.delete(this.key(filePath, mode));
    }
  }

  clear(): void {
    this.lru.clear();
  }

  private key(filePath: string, mode: ClassnameTransformMode): string {
    return `${mode}\u0000${filePath}`;
  }
}

export function styleDocumentSemanticFingerprint(styleDocument: StyleDocumentHIR): string {
  const selectorFingerprint = styleDocument.selectors
    .map((selector) => {
      const composes = selector.composes
        .map((ref) => {
          const classNames = [...ref.classNames].toSorted().join(",");
          return `${ref.from ?? ""}:${ref.fromGlobal ? "global" : "local"}:${classNames}`;
        })
        .toSorted()
        .join("|");
      const bemSuffix = selector.bemSuffix
        ? `${selector.bemSuffix.rawToken}:${selector.bemSuffix.parentResolvedName}`
        : "";
      const wrapperContext =
        selector.context?.wrapperAtRules
          .map((wrapper) =>
            [
              wrapper.name,
              wrapper.params,
              wrapper.range.start.line,
              wrapper.range.start.character,
              wrapper.range.end.line,
              wrapper.range.end.character,
            ].join(":"),
          )
          .join("|") ?? "";
      return [
        selector.name,
        selector.canonicalName,
        selector.viewKind,
        selector.fullSelector,
        selector.range.start.line,
        selector.range.start.character,
        selector.range.end.line,
        selector.range.end.character,
        selector.nestedSafety,
        selector.originalName ?? "",
        bemSuffix,
        wrapperContext,
        composes,
      ].join("::");
    })
    .join("\n");
  const keyframesFingerprint = styleDocument.keyframes
    .map((keyframes) =>
      [
        keyframes.name,
        keyframes.range.start.line,
        keyframes.range.start.character,
        keyframes.range.end.line,
        keyframes.range.end.character,
      ].join("::"),
    )
    .join("\n");
  const animationRefFingerprint = styleDocument.animationNameRefs
    .map((ref) =>
      [
        ref.name,
        ref.property,
        ref.range.start.line,
        ref.range.start.character,
        ref.range.end.line,
        ref.range.end.character,
      ].join("::"),
    )
    .join("\n");
  const valueDeclFingerprint = styleDocument.valueDecls
    .map((valueDecl) =>
      [
        valueDecl.name,
        valueDecl.value,
        valueDecl.range.start.line,
        valueDecl.range.start.character,
        valueDecl.range.end.line,
        valueDecl.range.end.character,
      ].join("::"),
    )
    .join("\n");
  const valueImportFingerprint = styleDocument.valueImports
    .map((valueImport) =>
      [
        valueImport.name,
        valueImport.importedName,
        valueImport.from,
        valueImport.range.start.line,
        valueImport.range.start.character,
        valueImport.range.end.line,
        valueImport.range.end.character,
      ].join("::"),
    )
    .join("\n");
  const valueRefFingerprint = styleDocument.valueRefs
    .map((valueRef) =>
      [
        valueRef.name,
        valueRef.source,
        valueRef.range.start.line,
        valueRef.range.start.character,
        valueRef.range.end.line,
        valueRef.range.end.character,
      ].join("::"),
    )
    .join("\n");
  const customPropertyDeclFingerprint = styleDocument.customPropertyDecls
    .map((decl) =>
      [
        decl.name,
        decl.value,
        decl.range.start.line,
        decl.range.start.character,
        decl.range.end.line,
        decl.range.end.character,
        decl.ruleRange.start.line,
        decl.ruleRange.start.character,
        decl.ruleRange.end.line,
        decl.ruleRange.end.character,
        decl.context.containerKind,
        decl.context.selectorText ?? "",
        decl.context.atRuleName ?? "",
        decl.context.atRuleParams ?? "",
        decl.context.wrapperAtRules
          .map(
            (wrapper) =>
              `${wrapper.name}(${wrapper.params})@${wrapper.range.start.line}:${wrapper.range.start.character}-${wrapper.range.end.line}:${wrapper.range.end.character}`,
          )
          .join(","),
      ].join("::"),
    )
    .join("\n");
  const customPropertyRefFingerprint = styleDocument.customPropertyRefs
    .map((ref) =>
      [
        ref.name,
        ref.range.start.line,
        ref.range.start.character,
        ref.range.end.line,
        ref.range.end.character,
        ref.context.containerKind,
        ref.context.selectorText ?? "",
        ref.context.atRuleName ?? "",
        ref.context.atRuleParams ?? "",
        ref.context.wrapperAtRules
          .map(
            (wrapper) =>
              `${wrapper.name}(${wrapper.params})@${wrapper.range.start.line}:${wrapper.range.start.character}-${wrapper.range.end.line}:${wrapper.range.end.character}`,
          )
          .join(","),
      ].join("::"),
    )
    .join("\n");
  const sassSymbolFingerprint = styleDocument.sassSymbols
    .map((symbol) =>
      [
        symbol.selectorName,
        symbol.syntax ?? "sass",
        symbol.symbolKind,
        symbol.name,
        symbol.role,
        symbol.resolution,
        symbol.range.start.line,
        symbol.range.start.character,
        symbol.range.end.line,
        symbol.range.end.character,
        symbol.ruleRange.start.line,
        symbol.ruleRange.start.character,
        symbol.ruleRange.end.line,
        symbol.ruleRange.end.character,
      ].join("::"),
    )
    .join("\n");
  const sassSymbolDeclFingerprint = styleDocument.sassSymbolDecls
    .map((decl) =>
      [
        decl.symbolKind,
        decl.syntax ?? "sass",
        decl.name,
        decl.range.start.line,
        decl.range.start.character,
        decl.range.end.line,
        decl.range.end.character,
        decl.ruleRange.start.line,
        decl.ruleRange.start.character,
        decl.ruleRange.end.line,
        decl.ruleRange.end.character,
      ].join("::"),
    )
    .join("\n");
  const sassModuleUseFingerprint = styleDocument.sassModuleUses
    .map((moduleUse) =>
      [
        moduleUse.source,
        moduleUse.namespaceKind,
        moduleUse.namespace ?? "",
        moduleUse.range.start.line,
        moduleUse.range.start.character,
        moduleUse.range.end.line,
        moduleUse.range.end.character,
        moduleUse.ruleRange.start.line,
        moduleUse.ruleRange.start.character,
        moduleUse.ruleRange.end.line,
        moduleUse.ruleRange.end.character,
      ].join("::"),
    )
    .join("\n");
  const sassModuleForwardFingerprint = styleDocument.sassModuleForwards
    .map((moduleForward) =>
      [
        moduleForward.source,
        moduleForward.prefix,
        moduleForward.visibilityKind,
        moduleForward.visibilityMembers
          .map((member) => `${member.symbolKind ?? "member"}:${member.name}`)
          .join(","),
        moduleForward.range.start.line,
        moduleForward.range.start.character,
        moduleForward.range.end.line,
        moduleForward.range.end.character,
        moduleForward.ruleRange.start.line,
        moduleForward.ruleRange.start.character,
        moduleForward.ruleRange.end.line,
        moduleForward.ruleRange.end.character,
      ].join("::"),
    )
    .join("\n");
  const sassModuleMemberRefFingerprint = styleDocument.sassModuleMemberRefs
    .map((memberRef) =>
      [
        memberRef.selectorName,
        memberRef.namespace,
        memberRef.symbolKind,
        memberRef.name,
        memberRef.role,
        memberRef.range.start.line,
        memberRef.range.start.character,
        memberRef.range.end.line,
        memberRef.range.end.character,
        memberRef.ruleRange.start.line,
        memberRef.ruleRange.start.character,
        memberRef.ruleRange.end.line,
        memberRef.ruleRange.end.character,
      ].join("::"),
    )
    .join("\n");
  return [
    selectorFingerprint,
    keyframesFingerprint,
    animationRefFingerprint,
    valueDeclFingerprint,
    valueImportFingerprint,
    valueRefFingerprint,
    customPropertyDeclFingerprint,
    customPropertyRefFingerprint,
    sassSymbolFingerprint,
    sassSymbolDeclFingerprint,
    sassModuleUseFingerprint,
    sassModuleForwardFingerprint,
    sassModuleMemberRefFingerprint,
  ].join("\n---\n");
}
