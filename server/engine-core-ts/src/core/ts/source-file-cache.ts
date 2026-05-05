import ts from "typescript";
import { contentHash } from "../util/hash";
import { LruMap } from "../util/lru-map";
import { projectVueSfcScriptToTypeScriptSource } from "./vue-sfc-source";

interface SourceFileCacheEntry {
  hash: string;
  sourceFile: ts.SourceFile;
}

/**
 * In-flight tier of the 2-tier TypeScript strategy.
 *
 * The provider hot path needs to re-read a live editor buffer on
 * every keystroke without waiting for disk I/O. SourceFileCache
 * parses `ts.createSourceFile` once per (path, contentHash) and
 * returns the cached node on subsequent calls with the same text.
 *
 * parentNodes is always set, so consumers can walk up the tree to
 * find an enclosing function or statement (cx/binding-detector
 * relies on this).
 */
export class SourceFileCache {
  private readonly lru: LruMap<string, SourceFileCacheEntry>;

  constructor(options: { max: number }) {
    this.lru = new LruMap(options.max);
  }

  get(filePath: string, content: string): ts.SourceFile {
    const hash = contentHash(content);
    const cached = this.lru.get(filePath);
    if (cached && cached.hash === hash) {
      // Touch: move to the end so frequently-used files stay warm.
      this.lru.touch(filePath, cached);
      return cached.sourceFile;
    }
    const source = filePath.endsWith(".vue")
      ? projectVueSfcScriptToTypeScriptSource(content)
      : content;
    const sourceFile = ts.createSourceFile(
      filePath,
      source,
      ts.ScriptTarget.Latest,
      /*setParentNodes*/ true,
      scriptKindFor(filePath),
    );
    this.lru.set(filePath, { hash, sourceFile });
    return sourceFile;
  }

  invalidate(filePath: string): void {
    this.lru.delete(filePath);
  }

  clear(): void {
    this.lru.clear();
  }
}

/**
 * Pick the ts.ScriptKind from a file extension. Unknown extensions
 * fall back to TSX (the most permissive parser).
 */
function scriptKindFor(filePath: string): ts.ScriptKind {
  if (filePath.endsWith(".tsx")) return ts.ScriptKind.TSX;
  if (filePath.endsWith(".mts") || filePath.endsWith(".cts") || filePath.endsWith(".ts"))
    return ts.ScriptKind.TS;
  if (filePath.endsWith(".jsx")) return ts.ScriptKind.JSX;
  if (filePath.endsWith(".mjs") || filePath.endsWith(".cjs") || filePath.endsWith(".js"))
    return ts.ScriptKind.JS;
  return ts.ScriptKind.TSX;
}
