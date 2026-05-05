import { readCompletionContext } from "../../engine-core-ts/src/core/query";
import type { SelectorDeclHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  classValueMatchesCandidate,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
  type AbstractClassValue,
} from "../../engine-core-ts/src/core/abstract-value/class-value-domain";

export function resolveSourceCompletionSelectors(
  params: CursorParams,
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath">,
): readonly SelectorDeclHIR[] {
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  if (
    entry.sourceDocument.utilityBindings.length === 0 &&
    entry.sourceDocument.styleImports.length === 0
  ) {
    return [];
  }

  const textBefore = getTextBefore(params.content, params.line, params.character);
  const ctx = readCompletionContext(entry, textBefore);
  if (!ctx) return [];

  const styleDocument = deps.styleDocumentForPath(ctx.scssModulePath);
  if (!styleDocument || styleDocument.selectors.length === 0) return [];
  const textAfter = getTextAfter(params.content, params.line, params.character);
  const expectedValueDomain = readCompletionExpectedValueDomain(textBefore, textAfter);
  if (!expectedValueDomain) return styleDocument.selectors;
  return styleDocument.selectors.filter((selector) =>
    classValueMatchesCandidate(expectedValueDomain, selector.name),
  );
}

function getTextBefore(content: string, line: number, character: number): string {
  let offset = 0;
  for (let i = 0; i < line; i++) {
    const nl = content.indexOf("\n", offset);
    if (nl === -1) return content;
    offset = nl + 1;
  }
  return content.slice(0, offset + character);
}

function getTextAfter(content: string, line: number, character: number): string {
  let offset = 0;
  for (let i = 0; i < line; i++) {
    const nl = content.indexOf("\n", offset);
    if (nl === -1) return "";
    offset = nl + 1;
  }
  return content.slice(offset + character);
}

function readCompletionExpectedValueDomain(
  textBefore: string,
  textAfter: string,
): AbstractClassValue | null {
  const prefix = readCompletionPrefix(textBefore);
  const suffix = readCompletionSuffix(textAfter);
  if (prefix && suffix) return prefixSuffixClassValue(prefix, suffix);
  if (prefix) return prefixClassValue(prefix);
  if (suffix) return suffixClassValue(suffix);
  return null;
}

function readCompletionPrefix(textBefore: string): string | null {
  return (
    readStylesPropertyAccessPrefix(textBefore) ??
    readBracketStringAccessPrefix(textBefore) ??
    readStringClassTokenPrefix(textBefore) ??
    readObjectKeyPrefix(textBefore)
  );
}

function readStylesPropertyAccessPrefix(textBefore: string): string | null {
  const match = /(?:^|[^\p{L}\p{N}_$])[\p{L}_$][\p{L}\p{N}_$]*\.([\p{L}\p{N}_-]*)$/u.exec(
    textBefore,
  );
  return match?.[1] ?? null;
}

function readBracketStringAccessPrefix(textBefore: string): string | null {
  const match = /\[\s*(['"`])([^'"`\]]*)$/u.exec(textBefore);
  if (!match) return null;
  return readLastClassTokenPrefix(match[2]!);
}

function readStringClassTokenPrefix(textBefore: string): string | null {
  let quoteIndex = -1;
  for (let index = textBefore.length - 1; index >= 0; index -= 1) {
    const ch = textBefore[index];
    if ((ch === "'" || ch === '"' || ch === "`") && !isEscaped(textBefore, index)) {
      quoteIndex = index;
      break;
    }
    if (ch === "\n" || ch === "\r") break;
  }
  if (quoteIndex < 0) return null;
  const prefix = textBefore.slice(quoteIndex + 1);
  if (/[),\]}]/u.test(prefix)) return null;
  return readLastClassTokenPrefix(prefix);
}

function readObjectKeyPrefix(textBefore: string): string | null {
  const match = /(?:[{,]\s*)([\p{L}_-][\p{L}\p{N}_-]*)$/u.exec(textBefore);
  return match?.[1] ?? null;
}

function readLastClassTokenPrefix(value: string): string | null {
  const match = /(?:^|\s)([\p{L}\p{N}_-]*)$/u.exec(value);
  return match ? match[1]! : null;
}

function readCompletionSuffix(textAfter: string): string | null {
  const match = /^([\p{L}\p{N}_-]+)/u.exec(textAfter);
  return match?.[1] ?? null;
}

function isEscaped(text: string, index: number): boolean {
  let slashCount = 0;
  for (let cursor = index - 1; cursor >= 0 && text[cursor] === "\\"; cursor -= 1) {
    slashCount += 1;
  }
  return slashCount % 2 === 1;
}
