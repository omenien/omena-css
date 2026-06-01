import type { Range } from "@omena/shared";

/**
 * Base shape for document-local HIR nodes.
 *
 * IDs are document-scoped and deterministic for one adapter output.
 * They are stable within a single analysis result, but they are not
 * intended to be cross-version graph identities.
 */
export interface HirNodeBase {
  readonly id: string;
  readonly range?: Range;
}

export type HirDocumentKind = "source" | "style";

export interface HirDocumentBase {
  readonly kind: HirDocumentKind;
  readonly filePath: string;
}

export type SourceLanguage =
  | "javascript"
  | "javascriptreact"
  | "typescript"
  | "typescriptreact"
  | "vue"
  | "unknown";
