import { describe, it, expect, vi } from "vitest";
import type ts from "typescript";
import type { StyleImport } from "@omena/shared";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import type { ResolvedCxBinding } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { cssModulesClassnamesBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  makeLiteralClassExpression,
  makeStyleAccessClassExpression,
} from "../../../server/engine-core-ts/src/core/hir/source-types";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

const SOURCE = `
  import classNames from 'classnames/bind';
  import styles from './Button.module.scss';
  const cx = classNames.bind(styles);
  const el = cx('indicator');
`;

function makeCache() {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const detectSpy = vi.fn((_sourceFile: ts.SourceFile, _filePath: string): CxBinding[] => {
    return [
      {
        cxVarName: "cx",
        stylesVarName: "styles",
        scssModulePath: "/fake/src/Button.module.scss",
        classNamesImportName: "classNames",
        bindingRange: {
          start: { line: 3, character: 8 },
          end: { line: 3, character: 10 },
        },
      },
    ];
  });
  const parseSpy = vi.fn(
    (_sourceFile: ts.SourceFile, _bindings: readonly ResolvedCxBinding[]) => [],
  );
  const cache = new DocumentAnalysisCache({
    sourceFileCache,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectSpy(sf, fp) }),
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    parseClassExpressions: parseSpy,
    max: 10,
  });
  return { cache, detectSpy, parseSpy, sourceFileCache };
}

describe("DocumentAnalysisCache", () => {
  it("can route production analysis through BinderPluginV0 without legacy scan deps", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      binderPlugin: cssModulesClassnamesBinderPluginV0,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get("file:///fake/Button.tsx", SOURCE, "/fake/Button.tsx", 1);

    expect(entry.sourceDocument.utilityBindings).toMatchObject([
      { kind: "classnamesBind", localName: "cx", stylesLocalName: "styles" },
    ]);
    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
  });

  it("can compose multiple BinderPluginV0 implementations without losing CSS Module facts", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      binderPlugins: [cssModulesClassnamesBinderPluginV0, tailwindUnoUtilityBinderPluginV0],
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get(
      "file:///fake/Button.tsx",
      `
        import classNames from 'classnames/bind';
        import clsx from 'clsx';
        import styles from './Button.module.scss';
        const cx = classNames.bind(styles);
        const el = <div className={clsx(cx('indicator'), "flex gap-2")} />;
      `,
      "/fake/Button.tsx",
      1,
    );

    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
    expect(entry.sourceDocument.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "flex", domain: "utility-css" },
      { matchKind: "literal", className: "gap-2", domain: "utility-css" },
    ]);
  });

  it("prefers direct class-expression parsing when available", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const parseClassExpressions = vi.fn(() => [
      makeLiteralClassExpression(
        "class-expr:0",
        "cxCall",
        "/fake/src/Button.module.scss",
        "indicator",
        { start: { line: 4, character: 16 }, end: { line: 4, character: 25 } },
      ),
    ]);
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: (_sf, _fp) => ({
        stylesBindings: new Map(),
        bindings: [
          {
            cxVarName: "cx",
            stylesVarName: "styles",
            scssModulePath: "/fake/src/Button.module.scss",
            classNamesImportName: "classNames",
            bindingRange: {
              start: { line: 3, character: 8 },
              end: { line: 3, character: 10 },
            },
          },
        ],
      }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions,
      max: 10,
    });

    const entry = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);

    expect(parseClassExpressions).toHaveBeenCalledTimes(1);
    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
  });

  it("analyzes a document on the first get and caches the entry", () => {
    const { cache, detectSpy, parseSpy } = makeCache();
    const entry = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(
      entry.sourceDocument.utilityBindings.filter((binding) => binding.kind === "classnamesBind"),
    ).toHaveLength(1);
    expect(entry.sourceDocument.filePath).toBe("/fake/a.tsx");
    expect(detectSpy).toHaveBeenCalledTimes(1);
    expect(parseSpy).toHaveBeenCalledTimes(1);
  });

  it("returns the same entry when (uri, version) matches", () => {
    const { cache, detectSpy } = makeCache();
    const first = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    const second = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(second).toBe(first);
    expect(detectSpy).toHaveBeenCalledTimes(1);
  });

  it("returns an entry via content-hash fallback when version bumps but content is identical", () => {
    const { cache, detectSpy } = makeCache();
    const first = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    const second = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 2);
    // Not reference-equal because the entry is upgraded with a
    // new version field, but the underlying parse result is
    // preserved — detectSpy stays at one call.
    expect(second.sourceDocument.utilityBindings).toBe(first.sourceDocument.utilityBindings);
    expect(second.sourceFile).toBe(first.sourceFile);
    expect(second.version).toBe(2);
    expect(detectSpy).toHaveBeenCalledTimes(1);
  });

  it("re-analyzes when content changes", () => {
    const { cache, detectSpy } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    cache.get("file:///fake/a.tsx", `${SOURCE}\nconst y = cx('extra');`, "/fake/a.tsx", 2);
    expect(detectSpy).toHaveBeenCalledTimes(2);
  });

  it("invalidate(uri) drops the cached entry and the underlying source file", () => {
    const { cache, detectSpy, sourceFileCache } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    const invalidate = vi.spyOn(sourceFileCache, "invalidate");
    cache.invalidate("file:///fake/a.tsx");
    expect(invalidate).toHaveBeenCalledWith("/fake/a.tsx");
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(detectSpy).toHaveBeenCalledTimes(2);
  });

  it("clear() drops every entry", () => {
    const { cache, detectSpy } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    cache.get("file:///fake/b.tsx", SOURCE, "/fake/b.tsx", 1);
    cache.clear();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(detectSpy).toHaveBeenCalledTimes(3);
  });

  it("evicts the LRU entry beyond the max", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const detectSpy = vi.fn((): CxBinding[] => []);
    const parseSpy = vi.fn(() => []);
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectSpy(sf, fp) }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions: parseSpy,
      max: 2,
    });
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1);
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1); // touch a
    cache.get("file:///c.tsx", "const c = 3;", "/c.tsx", 1); // evict b
    detectSpy.mockClear();
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    expect(detectSpy).toHaveBeenCalledTimes(1);
  });

  it("re-puts the same uri under LRU pressure without evicting a touched sibling", () => {
    // Exercises the `entries.has(uri)` branch inside put(): when a
    // cached uri is re-analyzed with changed content, we delete+
    // re-insert rather than evict a different key.
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const detectSpy = vi.fn((): CxBinding[] => []);
    const parseSpy = vi.fn(() => []);
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectSpy(sf, fp) }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions: parseSpy,
      max: 2,
    });
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1);
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    // Same uri (a.tsx) with changed content — hits put() with
    // entries.has(uri) === true, should NOT evict b.
    cache.get("file:///a.tsx", "const a = 2;", "/a.tsx", 2);
    detectSpy.mockClear();
    // If b was evicted we would re-analyze here. Expectation: no
    // re-analysis because b is still in the cache.
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    expect(detectSpy).not.toHaveBeenCalled();
  });

  it("invalidates an uncached uri via the fileURLToPath fallback", () => {
    // Exercises the fallback branch in invalidate() when no
    // AnalysisEntry exists for the uri. The SourceFileCache might
    // still hold the entry under the derived path.
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const detectSpy = vi.fn((): CxBinding[] => []);
    const parseSpy = vi.fn(() => []);
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectSpy(sf, fp) }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions: parseSpy,
      max: 10,
    });
    const sfcInvalidate = vi.spyOn(sourceFileCache, "invalidate");
    cache.invalidate("file:///never/seen.tsx");
    expect(sfcInvalidate).toHaveBeenCalledWith("/never/seen.tsx");
  });

  it("swallows a malformed uri in invalidate without throwing", () => {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: () => ({ stylesBindings: new Map(), bindings: [] }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions: () => [],
      max: 10,
    });
    expect(() => cache.invalidate("not::a::uri")).not.toThrow();
  });
});

describe("DocumentAnalysisCache / styleAccess without classnames/bind", () => {
  it("populates sourceDocument class expressions for a file with style imports but no classnames/bind", () => {
    const clsxSource = `
      import clsx from 'clsx';
      import styles from './Button.module.scss';
      const el = <div className={clsx(styles.indicator)} />;
    `;
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const parseClassExpressionsSpy = vi.fn(
      (
        _sf: ts.SourceFile,
        _bindings: readonly ResolvedCxBinding[],
        stylesBindings: ReadonlyMap<string, StyleImport>,
      ) => {
        if (stylesBindings.size > 0 && stylesBindings.has("styles")) {
          return [
            makeStyleAccessClassExpression(
              "class-expr:0",
              "/fake/src/Button.module.scss",
              "synthetic-style-import-decl:test",
              "indicator",
              ["indicator"],
              { start: { line: 3, character: 42 }, end: { line: 3, character: 51 } },
            ),
          ];
        }
        return [];
      },
    );
    const scanSpy = vi.fn(
      (): { stylesBindings: ReadonlyMap<string, StyleImport>; bindings: CxBinding[] } => ({
        stylesBindings: new Map([
          ["styles", { kind: "resolved", absolutePath: "/fake/src/Button.module.scss" }],
        ]),
        bindings: [],
      }),
    );

    const cache = new DocumentAnalysisCache({
      sourceFileCache,
      scanCxImports: scanSpy,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      parseClassExpressions: parseClassExpressionsSpy,
      max: 10,
    });

    const entry = cache.get("file:///fake/a.tsx", clsxSource, "/fake/a.tsx", 1);

    // styleAccess refs must be populated even though the scan
    // returned an empty bindings list.
    expect(entry.sourceDocument.utilityBindings).toHaveLength(0);
    expect(entry.sourceDocument.classExpressions).toHaveLength(1);
    expect(entry.sourceDocument.classExpressions[0]).toMatchObject({
      kind: "styleAccess",
      className: "indicator",
    });
    expect(scanSpy).toHaveBeenCalledTimes(1);
    expect(parseClassExpressionsSpy).toHaveBeenCalledTimes(1);
    expect(parseClassExpressionsSpy.mock.calls[0]![2].get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Button.module.scss",
    });
  });
});
