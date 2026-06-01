import type { BemSuffixInfo, Range } from "@omena/shared";
import type { Rule } from "postcss";

/**
 * Classify whether a raw selector group is a BEM-safe nested
 * rename target and, if so, return the `BemSuffixInfo` the parser
 * should attach to the resulting map entry.
 *
 * All six conditions must hold:
 *   1. the group is nested (contains `&`)
 *   2. it contains exactly one `&`
 *   3. the parent rule is a bare single class
 *   4. the parent rule is not grouped
 *   5. the CURRENT rule is not grouped (groupsLength === 1)
 *   6. `findBemSuffixSpan` returns a non-null span (pure BEM
 *      suffix form — `&--x` or `&__x`, last token in group,
 *      nothing before the `&`)
 *
 * Returns `null` for any non-BEM-safe shape. The caller still
 * marks the entry with `isNested: true` when applicable; this
 * function only decides whether the surgical-rename trio can
 * be produced.
 */
/**
 * Minimal view of the parent-rule context the BEM classifier
 * needs. Structurally satisfied by the parser's `ParentContext`
 * so the call site can pass it directly without unpacking.
 */
export interface BemParentContext {
  readonly className?: string;
  readonly isGrouped?: boolean;
}

export function classifyBemSuffixSite(
  rule: Rule,
  raw: string,
  groupOffset: number,
  parent: BemParentContext,
  groupsLength: number,
): BemSuffixInfo | null {
  if (!raw.includes("&")) return null;
  const ampCount = raw.match(/&/g)?.length ?? 0;
  if (ampCount !== 1) return null;
  if (parent.className === undefined) return null;
  if (parent.isGrouped === true) return null;
  if (groupsLength !== 1) return null;

  const span = findBemSuffixSpan(rule, groupOffset, raw);
  if (!span) return null;

  return {
    rawToken: span.rawToken,
    rawTokenRange: span.range,
    parentResolvedName: parent.className,
  };
}

/**
 * Locate the `&--suffix` or `&__suffix` raw token inside a
 * postcss Rule's source and return both the token string and its
 * LSP Range. Returns `null` for every non-BEM nested shape:
 * compound (`&.active`), pseudo (`&:hover`), attribute (`&[x]`),
 * id (`&#foo`), combinator-before (`.a &--x`, `.a&--x`, etc.),
 * descendant-after (`&--x .y`), trailing compound (`&--x.other`).
 *
 * Exported for unit testing.
 */
export function findBemSuffixSpan(
  rule: Rule,
  groupOffset: number,
  rawGroup: string,
): { rawToken: string; range: Range } | null {
  const src = rule.source;
  if (!src?.start) return null;
  const startOffset = src.start.offset;

  const ampIndex = rawGroup.indexOf("&");
  if (ampIndex < 0) return null;
  const groupAbsOffset = startOffset + groupOffset;

  if (rawGroup.slice(0, ampIndex).trim() !== "") return null;

  const tail = rawGroup.slice(ampIndex + 1);
  const match = /^(--|__)[a-zA-Z_][\w-]*/.exec(tail);
  if (!match) return null;

  const fragment = "&" + match[0];

  if (rawGroup.slice(ampIndex + fragment.length).trim() !== "") return null;

  const tokenStartOffset = groupAbsOffset + ampIndex;
  const tokenEndOffset = tokenStartOffset + fragment.length;

  const startPos = src.input.fromOffset(tokenStartOffset);
  const endPos = src.input.fromOffset(tokenEndOffset);
  if (!startPos || !endPos) return null;

  return {
    rawToken: fragment,
    range: {
      start: { line: startPos.line - 1, character: startPos.col - 1 },
      end: { line: endPos.line - 1, character: endPos.col - 1 },
    },
  };
}
