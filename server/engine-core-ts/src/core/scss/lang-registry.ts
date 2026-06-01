import type { StyleLang } from "@omena/shared";
import type { Syntax } from "postcss";
import postcssLess from "postcss-less";
import postcssScss from "postcss-scss";

/**
 * Immutable list of every style language this extension understands.
 *
 * Adding a new language (e.g. LESS in 1.1+) is one new entry plus
 * importing its postcss syntax. No other file in the project
 * hard-codes an extension or a syntax — they all read from this
 * list via the helpers below.
 */
export const STYLE_LANGS: readonly StyleLang[] = [
  {
    id: "scss",
    extensions: [".module.scss"],
    syntax: postcssScss,
    displayName: "SCSS",
  },
  {
    id: "css",
    extensions: [".module.css"],
    syntax: null, // vanilla postcss handles plain CSS
    displayName: "CSS",
  },
  {
    id: "less",
    extensions: [".module.less"],
    syntax: postcssLess,
    displayName: "LESS",
  },
] as const;

const STYLE_DOCUMENT_LANGS: readonly StyleLang[] = STYLE_LANGS.map((lang) => ({
  ...lang,
  extensions: [...lang.extensions, ...lang.extensions.map((ext) => ext.replace(".module.", "."))],
}));

/** Flat list of every `.module.<ext>` this project indexes. */
export function getAllStyleExtensions(): readonly string[] {
  return STYLE_LANGS.flatMap((lang) => lang.extensions);
}

/**
 * Narrow `StyleLang.syntax` (typed as `unknown` in the shared
 * package so the shared layer stays runtime-free and does not
 * import postcss) to postcss `Syntax | null` at the
 * server/runtime boundary. This is the single documented
 * `as` cast in the codebase.
 */
export function getRuntimeSyntax(lang: StyleLang): Syntax | null {
  return lang.syntax as Syntax | null;
}

/** Pick the lang entry for a file path, or null if unrelated. */
export function findLangForPath(filePath: string): StyleLang | null {
  for (const lang of STYLE_LANGS) {
    for (const ext of lang.extensions) {
      if (filePath.endsWith(ext)) {
        return lang;
      }
    }
  }
  return null;
}

/**
 * Pick the parser/runtime language for any style document we may
 * need to read. Top-level providers still use `findLangForPath`
 * so extension features only activate on CSS Modules, but imported
 * package token files commonly use plain `.css` / `.scss` names.
 */
export function findStyleDocumentLangForPath(filePath: string): StyleLang | null {
  for (const lang of STYLE_DOCUMENT_LANGS) {
    for (const ext of lang.extensions) {
      if (filePath.endsWith(ext)) {
        return lang;
      }
    }
  }
  return null;
}

/**
 * Build the `workspace/didChangeWatchedFiles` glob pattern.
 * Example output: `**\/\*.module.{scss,css}`
 */
export function buildStyleFileWatcherGlob(): string {
  const stems = STYLE_LANGS.flatMap((lang) =>
    lang.extensions.map((ext) => ext.replace(/^\.module\./, "")),
  );
  return `**/*.module.{${stems.join(",")}}`;
}
