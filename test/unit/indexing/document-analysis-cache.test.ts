import { describe, expect, it, vi } from "vitest";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  makeClassUtilBinding,
  makeDomainLiteralClassReference,
  makeLiteralClassExpression,
  makeSourceDocumentHIR,
  makeStyleAccessClassExpression,
  makeStyleImportBinding,
  type ClassExpressionHIR,
  type DomainClassReferenceHIR,
  type UtilityBindingHIR,
} from "../../../server/engine-core-ts/src/core/hir/source-types";
import type { SourceBinderResult } from "../../../server/engine-core-ts/src/core/binder/scope-types";
import type { SourceBindingGraph } from "../../../server/engine-core-ts/src/core/binder/source-binding-graph";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

const SOURCE = `
  import classNames from 'classnames/bind';
  import styles from './Button.module.scss';
  const cx = classNames.bind(styles);
  const el = cx('indicator');
`;

function makeCache() {
  const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
    projectedSourceFrontendAnalysis({ filePath, content }),
  );
  const cache = new DocumentAnalysisCache({
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return { cache, sourceFrontendAnalysis };
}

describe("DocumentAnalysisCache", () => {
  it("uses source frontend projection instead of rebuilding TypeScript binding facts", () => {
    const sourceFrontendAnalysis = vi.fn(() => projectedSourceFrontendAnalysis());
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get("file:///fake/Button.tsx", SOURCE, "/fake/Button.tsx", 1);

    expect(sourceFrontendAnalysis).toHaveBeenCalledWith({
      filePath: "/fake/Button.tsx",
      content: SOURCE,
    });
    expect(entry.sourceBinder.decls).toMatchObject([{ name: "styles" }]);
    expect(entry.filePath).toBe("/fake/Button.tsx");
    expect(entry.sourceText).toBe(SOURCE);
    expect(entry.sourceDocument.styleImports).toMatchObject([
      { localName: "styles", resolved: { absolutePath: "/fake/Button.module.scss" } },
    ]);
    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
    expect(entry.stylesBindings.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/Button.module.scss",
    });
    expect(entry.classUtilNames).toEqual(["clsx"]);
  });

  it("fails when the required source frontend projection is missing", () => {
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis: () => null,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    expect(() => cache.get("file:///fake/Button.tsx", SOURCE, "/fake/Button.tsx", 1)).toThrow(
      "Rust source frontend analysis is required for /fake/Button.tsx",
    );
  });

  it("preserves CSS Module and domain facts supplied by source frontend projection", () => {
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({
        filePath,
        content,
        domainClassReferences: [
          makeDomainLiteralClassReference(
            "domain:flex",
            "utility-css",
            "utility-css",
            "jsxClassAttribute",
            "flex",
            { start: { line: 5, character: 45 }, end: { line: 5, character: 49 } },
          ),
          makeDomainLiteralClassReference(
            "domain:gap-2",
            "utility-css",
            "utility-css",
            "jsxClassAttribute",
            "gap-2",
            { start: { line: 5, character: 50 }, end: { line: 5, character: 55 } },
          ),
        ],
      }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
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

  it("uses class expressions supplied by source frontend projection", () => {
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({
        filePath,
        content,
        classExpressions: [
          makeLiteralClassExpression(
            "class-expr:0",
            "cxCall",
            "/fake/src/Button.module.scss",
            "indicator",
            { start: { line: 4, character: 16 }, end: { line: 4, character: 25 } },
          ),
        ],
      }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);

    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
  });

  it("derives source dependency paths from Rust source frontend module specifiers", () => {
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({
        filePath,
        content,
        classExpressions: [],
        sourceModuleSpecifiers: ["./theme", "./Button.module.scss"],
      }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get("file:///fake/src/Button.tsx", SOURCE, "/fake/src/Button.tsx", 1);

    expect(entry.sourceDependencyPaths).toEqual(
      expect.arrayContaining([
        "/fake/src/Button.tsx",
        "/fake/src/theme.ts",
        "/fake/src/theme.tsx",
        "/fake/src/theme/index.ts",
      ]),
    );
    expect(entry.sourceDependencyPaths).not.toContain("/fake/src/Button.module.scss");
  });

  it("analyzes a document on the first get and caches the entry", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    const entry = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(
      entry.sourceDocument.utilityBindings.filter((binding) => binding.kind === "classnamesBind"),
    ).toHaveLength(1);
    expect(entry.sourceDocument.filePath).toBe("/fake/a.tsx");
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
  });

  it("returns the same entry when (uri, version) matches", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    const first = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    const second = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(second).toBe(first);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
  });

  it("returns an entry via content-hash fallback when version bumps but content is identical", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    const first = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    const second = cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 2);
    expect(second.sourceDocument.utilityBindings).toBe(first.sourceDocument.utilityBindings);
    expect(second.version).toBe(2);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
  });

  it("re-analyzes when content changes", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    cache.get("file:///fake/a.tsx", `${SOURCE}\nconst y = cx('extra');`, "/fake/a.tsx", 2);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(2);
  });

  it("invalidate(uri) drops the cached entry", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    cache.invalidate("file:///fake/a.tsx");
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(2);
  });

  it("clear() drops every entry", () => {
    const { cache, sourceFrontendAnalysis } = makeCache();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    cache.get("file:///fake/b.tsx", SOURCE, "/fake/b.tsx", 1);
    cache.clear();
    cache.get("file:///fake/a.tsx", SOURCE, "/fake/a.tsx", 1);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(3);
  });

  it("evicts the LRU entry beyond the max", () => {
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({ filePath, content, classExpressions: [] }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 2,
    });
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1);
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1);
    cache.get("file:///c.tsx", "const c = 3;", "/c.tsx", 1);
    sourceFrontendAnalysis.mockClear();
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
  });

  it("re-puts the same uri under LRU pressure without evicting a touched sibling", () => {
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({ filePath, content, classExpressions: [] }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 2,
    });
    cache.get("file:///a.tsx", "const a = 1;", "/a.tsx", 1);
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    cache.get("file:///a.tsx", "const a = 2;", "/a.tsx", 2);
    sourceFrontendAnalysis.mockClear();
    cache.get("file:///b.tsx", "const b = 2;", "/b.tsx", 1);
    expect(sourceFrontendAnalysis).not.toHaveBeenCalled();
  });

  it("invalidates an uncached uri via the fileURLToPath fallback", () => {
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis: ({ filePath, content }) =>
        projectedSourceFrontendAnalysis({ filePath, content, classExpressions: [] }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    cache.invalidate("file:///never/seen.tsx");
    expect(() => cache.invalidate("file:///never/seen.tsx")).not.toThrow();
  });

  it("swallows a malformed uri in invalidate without throwing", () => {
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis: ({ filePath, content }) =>
        projectedSourceFrontendAnalysis({ filePath, content, classExpressions: [] }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
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
    const sourceFrontendAnalysis = vi.fn(({ filePath, content }) =>
      projectedSourceFrontendAnalysis({
        filePath,
        content,
        utilityBindings: [],
        classExpressions: [
          makeStyleAccessClassExpression(
            "class-expr:0",
            "/fake/src/Button.module.scss",
            "synthetic-style-import-decl:test",
            "indicator",
            ["indicator"],
            { start: { line: 3, character: 42 }, end: { line: 3, character: 51 } },
          ),
        ],
      }),
    );
    const cache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });

    const entry = cache.get("file:///fake/a.tsx", clsxSource, "/fake/a.tsx", 1);

    expect(entry.sourceDocument.utilityBindings).toHaveLength(0);
    expect(entry.sourceDocument.classExpressions).toHaveLength(1);
    expect(entry.sourceDocument.classExpressions[0]).toMatchObject({
      kind: "styleAccess",
      className: "indicator",
    });
    expect(sourceFrontendAnalysis).toHaveBeenCalledTimes(1);
  });
});

function projectedSourceFrontendAnalysis(
  args: {
    readonly filePath?: string;
    readonly content?: string;
    readonly classExpressions?: readonly ClassExpressionHIR[];
    readonly utilityBindings?: readonly UtilityBindingHIR[];
    readonly domainClassReferences?: readonly DomainClassReferenceHIR[];
    readonly sourceModuleSpecifiers?: readonly string[];
  } = {},
) {
  const filePath = args.filePath ?? "/fake/Button.tsx";
  const content = args.content ?? SOURCE;
  const styleImportOffset = Math.max(0, content.indexOf("styles"));
  const sourceBinder: SourceBinderResult = {
    filePath,
    scopes: [
      {
        id: "scope:source",
        kind: "sourceFile",
        filePath,
        span: { start: 0, end: content.length },
      },
    ],
    decls: [
      {
        id: "decl:styles",
        kind: "import",
        scopeId: "scope:source",
        name: "styles",
        filePath,
        span: { start: styleImportOffset, end: styleImportOffset + 6 },
        importPath: "./Button.module.scss",
      },
    ],
  };
  const sourceDocument = makeSourceDocumentHIR({
    filePath,
    language: "typescriptreact",
    styleImports: [
      makeStyleImportBinding("style-import:styles", "styles", "decl:styles", {
        kind: "resolved",
        absolutePath: "/fake/Button.module.scss",
      }),
    ],
    utilityBindings: args.utilityBindings ?? [
      makeClassUtilBinding("utility-binding:clsx", "clsx", "decl:clsx"),
      {
        kind: "classnamesBind",
        id: "utility-binding:cx",
        localName: "cx",
        stylesLocalName: "styles",
        scssModulePath: "/fake/Button.module.scss",
        classNamesImportName: "classNames",
        bindingDeclId: "decl:cx",
      },
    ],
    classExpressions: args.classExpressions ?? [
      makeLiteralClassExpression(
        "class-expression:indicator",
        "cxCall",
        "/fake/Button.module.scss",
        "indicator",
        { start: { line: 4, character: 16 }, end: { line: 4, character: 25 } },
      ),
    ],
    domainClassReferences: args.domainClassReferences,
  });
  const sourceBindingGraph: SourceBindingGraph = {
    filePath,
    nodes: [],
    edges: [],
  };
  return {
    sourceBinder,
    sourceDocument,
    sourceBindingGraph,
    sourceModuleSpecifiers: args.sourceModuleSpecifiers ?? ["./Button.module.scss"],
  };
}
