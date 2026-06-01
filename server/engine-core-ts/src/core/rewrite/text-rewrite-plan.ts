import type { Range } from "@omena/shared";

export interface PlannedTextEdit {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
}

export interface TextRewritePlan<TTarget> {
  readonly target: TTarget;
  readonly edits: readonly PlannedTextEdit[];
}

export function groupTextEditsByUri(
  edits: readonly PlannedTextEdit[],
): ReadonlyMap<string, readonly PlannedTextEdit[]> {
  const grouped = new Map<string, PlannedTextEdit[]>();
  for (const edit of edits) {
    const bucket = grouped.get(edit.uri);
    if (bucket) {
      bucket.push(edit);
      continue;
    }
    grouped.set(edit.uri, [edit]);
  }
  return grouped;
}
