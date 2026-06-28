import type { AnalysisEntry } from "../indexing/document-analysis-cache";
import { getBindingDeclById, resolveBindingAtOffset } from "../binder/source-binding-graph";

export interface CompletionContext {
  readonly scssModulePath: string;
}

export function readCompletionContext(
  entry: AnalysisEntry,
  textBefore: string,
): CompletionContext | null {
  for (const binding of entry.sourceDocument.utilityBindings) {
    if (binding.kind !== "classnamesBind") continue;
    const callOffset = findOpenCallOffset(textBefore, binding.localName);
    if (callOffset === null) continue;
    const resolution = resolveBindingAtOffset(
      entry.sourceBindingGraph,
      binding.localName,
      callOffset,
    );
    const decl = resolution
      ? getBindingDeclById(entry.sourceBindingGraph, resolution.declId)
      : null;
    if (!decl) continue;
    if (!bindingDeclIdsMatch(binding.bindingDeclId, decl.id, "synthetic-binding-decl:")) continue;
    if (isInsideCall(textBefore, binding.localName)) {
      return { scssModulePath: binding.scssModulePath };
    }
  }

  for (const styleImport of entry.sourceDocument.styleImports) {
    if (styleImport.resolved.kind !== "resolved") continue;
    const accessOffset = findStyleAccessOffset(textBefore, styleImport.localName);
    if (accessOffset === null) continue;
    const resolution = resolveBindingAtOffset(
      entry.sourceBindingGraph,
      styleImport.localName,
      accessOffset,
    );
    const decl = resolution
      ? getBindingDeclById(entry.sourceBindingGraph, resolution.declId)
      : null;
    if (!decl || !bindingDeclIdsMatch(styleImport.bindingDeclId, decl.id)) continue;
    return { scssModulePath: styleImport.resolved.absolutePath };
  }

  const classUtilBindings = entry.sourceDocument.utilityBindings.filter(
    (binding) => binding.kind === "classUtil",
  );
  if (classUtilBindings.length === 0 || entry.sourceDocument.styleImports.length === 0) return null;

  for (const binding of classUtilBindings) {
    const callOffset = findOpenCallOffset(textBefore, binding.localName);
    if (callOffset === null) continue;
    const resolution = resolveBindingAtOffset(
      entry.sourceBindingGraph,
      binding.localName,
      callOffset,
    );
    const decl = resolution
      ? getBindingDeclById(entry.sourceBindingGraph, resolution.declId)
      : null;
    if (!decl || !bindingDeclIdsMatch(binding.bindingDeclId, decl.id, "synthetic-binding-decl:")) {
      continue;
    }
    if (!isInsideCall(textBefore, binding.localName)) continue;

    for (const styleImport of entry.sourceDocument.styleImports) {
      if (styleImport.resolved.kind !== "resolved") continue;
      const varName = styleImport.localName;
      const dotPrefix = `${varName}.`;
      const idx = textBefore.lastIndexOf(dotPrefix);
      if (idx < 0) continue;
      const afterDot = textBefore.slice(idx + dotPrefix.length);
      if (afterDot.length > 0 && !/^\w+$/.test(afterDot)) continue;
      return { scssModulePath: styleImport.resolved.absolutePath };
    }
  }

  return null;
}

function bindingDeclIdsMatch(expected: string, actual: string, syntheticPrefix?: string): boolean {
  return (
    expected === actual || (syntheticPrefix !== undefined && expected.startsWith(syntheticPrefix))
  );
}

export function isInsideCall(textBefore: string, callName: string): boolean {
  const needle = `${callName}(`;
  const callIdx = textBefore.lastIndexOf(needle);
  if (callIdx === -1) return false;

  let depth = 1;
  let quote: string | null = null;
  for (let i = callIdx + needle.length; i < textBefore.length; i += 1) {
    const ch = textBefore[i]!;
    if (quote) {
      if (ch === quote && textBefore[i - 1] !== "\\") quote = null;
      continue;
    }
    if (ch === "'" || ch === '"' || ch === "`") {
      quote = ch;
      continue;
    }
    if (ch === "(") depth += 1;
    else if (ch === ")") {
      depth -= 1;
      if (depth === 0) return false;
    }
  }
  return depth > 0;
}

function findOpenCallOffset(textBefore: string, callName: string): number | null {
  const needle = `${callName}(`;
  const callIdx = textBefore.lastIndexOf(needle);
  if (callIdx === -1) return null;
  if (!isInsideCall(textBefore, callName)) return null;
  return callIdx;
}

function findStyleAccessOffset(textBefore: string, localName: string): number | null {
  return (
    findStylePropertyAccessOffset(textBefore, localName) ??
    findStyleBracketAccessOffset(textBefore, localName)
  );
}

function findStylePropertyAccessOffset(textBefore: string, localName: string): number | null {
  const needle = `${localName}.`;
  const accessOffset = textBefore.lastIndexOf(needle);
  if (accessOffset === -1) return null;
  if (accessOffset > 0 && isIdentifierPart(textBefore[accessOffset - 1]!)) return null;
  const afterDot = textBefore.slice(accessOffset + needle.length);
  if (!/^[\p{L}\p{N}_-]*$/u.test(afterDot)) return null;
  return accessOffset;
}

function findStyleBracketAccessOffset(textBefore: string, localName: string): number | null {
  const needle = `${localName}[`;
  const accessOffset = textBefore.lastIndexOf(needle);
  if (accessOffset === -1) return null;
  if (accessOffset > 0 && isIdentifierPart(textBefore[accessOffset - 1]!)) return null;
  const afterBracket = textBefore.slice(accessOffset + needle.length);
  if (!/^\s*(['"`])[^'"`\]]*$/u.test(afterBracket)) return null;
  return accessOffset;
}

function isIdentifierPart(value: string): boolean {
  return /^[\p{L}\p{N}_$]$/u.test(value);
}
