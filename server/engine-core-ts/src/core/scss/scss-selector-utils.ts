import type { Range } from "@css-module-explainer/shared";
import type { AtRule, Rule } from "postcss";

/**
 * Split a selector source on top-level commas, tracking the offset
 * of each group within the original string. Unlike `rule.selectors`
 * (which loses per-group offsets), this walker respects paren depth
 * so commas inside `:is(.a, .b)` or `:not(.c, .d)` are preserved.
 */
export function enumerateGroups(selectorSource: string): Array<{ raw: string; offset: number }> {
  const groups: Array<{ raw: string; offset: number }> = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < selectorSource.length; i++) {
    const ch = selectorSource[i];
    if (ch === "(" || ch === "[") depth++;
    else if (ch === ")" || ch === "]") depth--;
    else if (ch === "," && depth === 0) {
      groups.push(trimmedGroup(selectorSource, start, i));
      start = i + 1;
    }
  }
  groups.push(trimmedGroup(selectorSource, start, selectorSource.length));
  return groups;
}

function trimmedGroup(
  selectorSource: string,
  start: number,
  end: number,
): { raw: string; offset: number } {
  const raw = selectorSource.slice(start, end);
  const trimmedStart = raw.search(/\S/);
  if (trimmedStart < 0) return { raw: "", offset: start };
  return { raw: raw.trim(), offset: start + trimmedStart };
}

/**
 * Resolve a raw selector against its parent selector the way SCSS
 * does:
 *   parent ".button", raw "&--primary" → ".button--primary"
 *   parent ".card",   raw ".inner"      → ".card .inner"
 *   parent "",        raw ".top"        → ".top"
 */
export function resolveSelector(raw: string, parent: string): string {
  const trimmed = raw.trim();
  if (parent === "") return trimmed;
  if (trimmed.includes("&")) {
    return trimmed.replace(/&/g, parent);
  }
  return `${parent} ${trimmed}`;
}

/**
 * Extract class names from a resolved selector. Strips :global
 * (dropped), :local (unwrapped), and pseudos, then returns every
 * class in the rightmost compound segment.
 */
export function extractClassNames(resolvedSelector: string): string[] {
  const segments = enumerateCompoundSegments(stripSelectorForClassMatching(resolvedSelector));
  const lastSegment = segments[segments.length - 1] ?? "";
  return Array.from(extractClassTokens(lastSegment));
}

/**
 * Extract classes introduced by one raw selector group.
 *
 * Rule:
 * - classes in the rightmost compound are exported
 * - classes written alongside `&` in an ampersand compound are also exported
 * - suffix continuations such as `&-active` or `&_active` are exported from
 *   the resolved compound, even though the raw form contains no `.` token
 *
 * This avoids re-registering parent classes introduced only by `&`
 * expansion while still preserving nested compounds such as
 * `&.compact .body`.
 */
export function extractIntroducedClassNames(
  rawSelector: string,
  resolvedSelector: string,
): string[] {
  const rawCompounds = enumerateCompoundSegments(stripSelectorForClassMatching(rawSelector));
  const resolvedCompounds = enumerateCompoundSegments(
    stripSelectorForClassMatching(resolvedSelector),
  );
  const rightmostIndex =
    rawCompounds.length > 0 ? rawCompounds.length - 1 : resolvedCompounds.length - 1;
  const classNames: string[] = [];

  for (let index = 0; index <= rightmostIndex; index++) {
    const rawCompound = rawCompounds[index] ?? "";
    const resolvedCompound = resolvedCompounds[index] ?? "";
    const rawClasses = extractClassTokens(rawCompound);

    if (index === rightmostIndex || rawCompound.includes("&")) {
      classNames.push(...rawClasses);
    }

    if (
      rawCompound.includes("&") &&
      rawClasses.length === 0 &&
      isAmpersandClassSuffixContinuation(rawCompound)
    ) {
      classNames.push(...extractClassTokens(resolvedCompound));
    }
  }

  return Array.from(new Set(classNames));
}

/**
 * Word-boundary-aware class token offset finder.
 *
 * `".btn-primary .btn".indexOf(".btn")` returns `0`, pointing at
 * the `btn` *inside* `btn-primary` — wrong. We anchor the match
 * on `(?![\w-])` so `btn` won't match inside `btn-primary`.
 */
function findClassOffset(rawSelector: string, className: string): number {
  const escaped = className.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(`\\.${escaped}(?![\\w-])`);
  const match = re.exec(rawSelector);
  return match?.index ?? -1;
}

function stripSelectorForClassMatching(selector: string): string {
  const withoutGlobal = selector.replace(/:global\s*\(\s*[^)]*\)/g, "");
  const withoutLocal = withoutGlobal.replace(/:local\s*\(\s*([^)]*)\s*\)/g, "$1");
  return withoutLocal.replace(/::?[a-zA-Z-]+(?:\([^)]*\))?/g, "");
}

function enumerateCompoundSegments(selector: string): readonly string[] {
  const segments: string[] = [];
  let depth = 0;
  let start = -1;

  for (let index = 0; index < selector.length; index++) {
    const ch = selector.charAt(index);
    if (ch === "(" || ch === "[") {
      depth++;
      if (start === -1) start = index;
      continue;
    }
    if (ch === ")" || ch === "]") {
      depth--;
      continue;
    }

    if (depth === 0 && (ch === ">" || ch === "+" || ch === "~" || /\s/.test(ch))) {
      pushCompoundSegment(segments, selector, start, index);
      start = -1;
      continue;
    }

    if (start === -1) start = index;
  }

  pushCompoundSegment(segments, selector, start, selector.length);
  return segments;
}

function pushCompoundSegment(
  segments: string[],
  selector: string,
  start: number,
  end: number,
): void {
  if (start === -1) return;
  const segment = selector.slice(start, end).trim();
  if (segment.length > 0) segments.push(segment);
}

function extractClassTokens(selectorSegment: string): readonly string[] {
  // Unicode property classes so identifiers outside the ASCII
  // subset (e.g. `.한글`, `.日本語`, `.español-btn`, or the NFD
  // form `.café` where `é` is `e` + U+0301) survive token
  // extraction. `\p{M}` keeps combining marks attached.
  const matches = selectorSegment.match(/\.[\p{L}_][\p{L}\p{N}\p{M}_-]*/gu) ?? [];
  return matches.map((m) => m.slice(1));
}

function isAmpersandClassSuffixContinuation(selectorSegment: string): boolean {
  const ampIndex = selectorSegment.indexOf("&");
  if (ampIndex < 0) return false;

  const afterAmp = selectorSegment.slice(ampIndex + 1);
  if (afterAmp.length === 0) return false;

  // `&` followed by selector syntax starts a new selector component,
  // not a continuation of the current class name.
  return !/^[:.[#>+~\s,]/.test(afterAmp);
}

export function findClassTokenRange(
  sourceStart: { line: number; column: number } | undefined,
  className: string,
  rawSelector: string,
  groupOffset = 0,
): Range {
  if (!sourceStart) return zeroRange();
  const line = sourceStart.line - 1;
  const offset = findClassOffset(rawSelector, className);
  const baseCol = sourceStart.column - 1;
  if (offset >= 0) {
    const character = baseCol + groupOffset + offset + 1;
    return {
      start: { line, character },
      end: { line, character: character + className.length },
    };
  }

  const suffixRange = findAmpersandSuffixRange(rawSelector, className);
  const character = baseCol + groupOffset + (suffixRange?.start ?? 0);
  return {
    start: { line, character },
    end: { line, character: character + (suffixRange?.length ?? className.length) },
  };
}

function findAmpersandSuffixRange(
  rawSelector: string,
  className: string,
): { readonly start: number; readonly length: number } | null {
  const match = /&((?:--|__)[a-zA-Z_][\w-]*)/.exec(rawSelector);
  if (!match?.[1] || !className.endsWith(match[1])) return null;
  return {
    start: match.index + 1,
    length: match[1].length,
  };
}

export function atRootTokenRange(
  startLine: number,
  baseColumn: number,
  className: string,
  rawSelector: string,
): Range {
  const line = startLine - 1;
  const offset = findClassOffset(rawSelector, className);
  const character = offset >= 0 ? baseColumn + offset + 1 : baseColumn;
  return {
    start: { line, character },
    end: { line, character: character + className.length },
  };
}

export function rangeForSourceNode(node: Rule | AtRule): Range {
  const start = node.source?.start;
  const end = node.source?.end;
  return {
    start: start
      ? { line: start.line - 1, character: start.column - 1 }
      : { line: 0, character: 0 },
    end: end ? { line: end.line - 1, character: end.column - 1 } : { line: 0, character: 0 },
  };
}

function zeroRange(): Range {
  return { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } };
}
