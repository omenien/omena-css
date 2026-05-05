/** 0-based line and character position in a text document. */
export interface Position {
  readonly line: number;
  readonly character: number;
}

/** 0-based range with inclusive `start` and exclusive `end`. */
export interface Range {
  readonly start: Position;
  readonly end: Position;
}

/**
 * Style language descriptor — one entry per `.module.<ext>` target.
 *
 * `syntax` is typed as `unknown` so this module can remain runtime-free
 * (Layer 3 rule: shared must not import postcss or any runtime dep).
 * The server narrows it back to `postcss.Syntax | null` at the boundary.
 */
export interface StyleLang {
  readonly id: "scss" | "css" | "less";
  readonly extensions: readonly string[];
  readonly syntax: unknown;
  readonly displayName: string;
}

/**
 * A `composes` reference found inside a CSS Module rule.
 * e.g. `composes: base lg from './base.module.css'`
 */
export interface ComposesRef {
  readonly classNames: readonly string[];
  /** Source range for the whole `composes` declaration. */
  readonly range?: Range;
  readonly classTokens?: readonly ComposesClassToken[];
  /** Relative path to the source module. Undefined = same file. */
  readonly from?: string;
  /** `composes: x from global` — global (non-module) scope. */
  readonly fromGlobal?: boolean;
}

export interface ComposesClassToken {
  readonly className: string;
  readonly range: Range;
}

/**
 * Information needed to rewrite a BEM-suffix nested selector
 * (`&--primary`, `&__icon`) as a surgical suffix-only edit.
 *
 * The three fields are always produced together by the parser:
 *   - rawTokenRange: source-accurate span of the `&`-fragment
 *   - rawToken: verbatim slice (e.g. `"&--primary"`)
 *   - parentResolvedName: resolved class of the enclosing bare
 *     `.classname` rule, used for suffix-math via
 *     `name.slice(parent.length)`
 */
export interface BemSuffixInfo {
  readonly rawTokenRange: Range;
  readonly rawToken: string;
  readonly parentResolvedName: string;
}

/**
 * Outcome of resolving a CSS Module `import styles from '...'`
 * specifier against the filesystem. Every entry in an analysis's
 * `stylesBindings` map carries one of these variants.
 *
 * `resolved`: the target file exists on disk and its class map can
 * be loaded. `missing`: the target path was computed but the file
 * does not exist — diagnostics can underline the specifier and
 * inform the user.
 */
export type StyleImport =
  | {
      readonly kind: "resolved";
      readonly absolutePath: string;
    }
  | {
      readonly kind: "missing";
      readonly absolutePath: string;
      readonly specifier: string;
      readonly range: Range;
    };

/**
 * Which source syntax produced a class reference.
 * Deliberately a string literal union (no payload); this can be
 * widened to a discriminated union later without breaking
 * consumers that only read the discriminator.
 */
export type ClassRefOrigin = "cxCall" | "styleAccess";

/**
 * Result of resolving a TypeScript identifier to its string-literal
 * union type.
 *
 * - `kind: "union"` carries every literal member the checker saw.
 *   A single-member union (single string literal) is represented
 *   the same way, so consumers do not branch on arity.
 * - `kind: "unresolvable"` is returned when the identifier cannot
 *   be matched to a string-literal union — either because the
 *   symbol is missing, the type is not a literal union, or the
 *   program could not be built. The empty `values` array keeps
 *   the consumer code branch-free.
 */
export type ResolvedType =
  | { readonly kind: "union"; readonly values: readonly string[] }
  | { readonly kind: "unresolvable"; readonly values: readonly [] };

/**
 * Structured classification of what a CallSite matches, used as
 * the reverse-index key. Discriminated union — the shape is
 * authoritative, no string parsing.
 *
 * `static` sites carry both `className` (exact token the user
 * wrote — `btnPrimary` for `styles.btnPrimary`, `btn-primary` for
 * `cx('btn-primary')`) and `canonicalName` (the original SCSS
 * selector name — always the non-alias form). The reverse index
 * keys by `canonicalName` so a single query finds every site that
 * references a given SCSS class, regardless of which alias form
 * the user wrote. Rename uses `className` to pick the correct
 * rewrite form per site.
 */
export type CallSiteMatch =
  | {
      readonly kind: "static";
      readonly className: string;
      readonly canonicalName: string;
    }
  | { readonly kind: "template"; readonly staticPrefix: string }
  | { readonly kind: "variable"; readonly variableName: string };

/**
 * Whether a CallSite corresponds to a literal token the user wrote
 * ("direct") or a synthesized entry produced by expanding a
 * template/variable ref against a class map ("expanded").
 *
 * Rename filters out expanded sites (rewriting them would destroy
 * the template/variable source). Find References includes them.
 */
export type CallSiteExpansion = "direct" | "expanded";

/**
 * One entry in the `editor.action.showReferences` location list
 * (LSP JSON shape, before the client middleware maps it to
 * `vscode.Location`).
 */
export interface ShowReferencesLocation {
  readonly uri: string;
  readonly range: Range;
}

/**
 * Tuple contract for `editor.action.showReferences` arguments.
 *
 * VS Code's built-in command takes `(uri, position, locations)`
 * positionally, so the wire shape must be a tuple — not a single
 * object. This type documents the tuple order and lets the
 * server (`reference-lens.ts`) and the client middleware
 * (`extension.ts`) agree on a single authoritative contract,
 * replacing ad-hoc `as` casts at both ends.
 */
export type ShowReferencesArgs = readonly [
  uri: string,
  position: Position,
  locations: readonly ShowReferencesLocation[],
];

/**
 * One recorded call site of a specific class name. The
 * WorkspaceReverseIndex maps (scssFilePath, className) →
 * CallSite[] using `match.kind === "static"` to pick the
 * indexable subset.
 */
export interface CallSite {
  /** URI of the TSX/JSX/TS/JS file containing the cx() call. */
  readonly uri: string;
  /** Range covering the class token the user wrote. */
  readonly range: Range;
  /** Absolute path of the `.module.scss|css` file this site references. */
  readonly scssModulePath: string;
  /** Structured discriminator describing the matched pattern. */
  readonly match: CallSiteMatch;
  /** Whether this is a direct token or synthesized from a dynamic ref. */
  readonly expansion: CallSiteExpansion;
}
