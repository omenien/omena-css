import type { CallSite, StyleImport } from "@omena/shared";
import type { ResolvedCxBinding } from "../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { SourceFileCache } from "../../server/engine-core-ts/src/core/ts/source-file-cache";
import type { SelectorDeclHIR } from "../../server/engine-core-ts/src/core/hir/style-types";
import { DocumentAnalysisCache } from "../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { NullSemanticWorkspaceReferenceIndex } from "../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import {
  NOOP_LOG_ERROR,
  type ProviderDeps,
} from "../../server/lsp-server/src/providers/cursor-dispatch";
import { DEFAULT_SETTINGS } from "../../server/engine-core-ts/src/settings";
import { AliasResolver } from "../../server/engine-core-ts/src/core/cx/alias-resolver";
import { FakeTypeResolver } from "./fake-type-resolver";
import { buildClassExpressions } from "./source-documents";
import { buildStyleDocumentFromSelectorMap, makeTestSelector } from "./style-documents";

export const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake/ws", {});

/** Create a minimal selector for testing (fixed line 11 position). */
export function info(name: string): SelectorDeclHIR {
  return makeTestSelector(name, 11, {
    ruleRange: { start: { line: 10, character: 0 }, end: { line: 13, character: 1 } },
  });
}

/** Create a minimal selector at a specific line. */
export function infoAtLine(name: string, line: number): SelectorDeclHIR {
  return makeTestSelector(name, line, {
    range: { start: { line, character: 1 }, end: { line, character: 1 + name.length } },
    ruleRange: { start: { line, character: 0 }, end: { line: line + 2, character: 1 } },
  });
}

/** Create a selector at a specific line with custom declarations. */
export function infoWithDeclarations(
  name: string,
  line: number,
  declarations: string,
): SelectorDeclHIR {
  return makeTestSelector(name, line, { declarations });
}

/**
 * Create a minimal static CallSite for testing. `canonicalName`
 * defaults to `className` (the non-alias case); tests exercising
 * alias-form access pass an explicit `canonicalName` to distinguish
 * the source token from the original SCSS key.
 */
export function siteAt(
  uri: string,
  className: string,
  line: number,
  scssPath: string = "/fake/a.module.scss",
  canonicalName: string = className,
): CallSite {
  return {
    uri,
    range: { start: { line, character: 10 }, end: { line, character: 10 + className.length } },
    scssModulePath: scssPath,
    match: { kind: "static" as const, className, canonicalName },
    expansion: "direct",
  };
}

export function semanticSiteAt(
  uri: string,
  className: string,
  line: number,
  scssPath: string = "/fake/a.module.scss",
  canonicalName: string = className,
  options: {
    start?: number;
    end?: number;
    certainty?: "exact" | "inferred" | "possible";
    reason?:
      | "literal"
      | "styleAccess"
      | "templatePrefix"
      | "typeUnion"
      | "flowLiteral"
      | "flowBranch";
    origin?: "cxCall" | "styleAccess";
  } = {},
) {
  const certainty = options.certainty ?? "exact";
  const start = options.start ?? 10;
  const end = options.end ?? start + className.length;
  return {
    refId: `ref:${uri}:${line}:${start}`,
    selectorId: `selector:${scssPath}:${canonicalName}`,
    filePath: uri.replace("file://", ""),
    uri,
    range: { start: { line, character: start }, end: { line, character: end } },
    origin: options.origin ?? "cxCall",
    scssModulePath: scssPath,
    selectorFilePath: scssPath,
    canonicalName,
    className,
    selectorCertainty: certainty,
    reason: options.reason ?? "literal",
    expansion: certainty === "exact" ? "direct" : "expanded",
  } as const;
}

export function buildTestClassExpressions(args: {
  readonly filePath: string;
  readonly bindings: readonly ResolvedCxBinding[];
  readonly stylesBindings?: ReadonlyMap<string, StyleImport>;
  readonly classUtilNames?: readonly string[];
  readonly expressions: Parameters<typeof buildClassExpressions>[0]["expressions"];
}) {
  return buildClassExpressions({
    filePath: args.filePath,
    bindings: args.bindings,
    stylesBindings: args.stylesBindings ?? new Map(),
    classUtilNames: args.classUtilNames ?? [],
    expressions: args.expressions,
  });
}

type BaseDepsOverrides = Partial<ProviderDeps> & {
  readonly selectorMapForPath?: (path: string) => ReadonlyMap<string, SelectorDeclHIR> | null;
};

/**
 * Build a default ProviderDeps with sensible empty defaults.
 *
 * Callers override individual fields via the `overrides` argument.
 * Keeps test setup DRY across hover, completion, and diagnostics tests.
 */
export function makeBaseDeps(overrides: BaseDepsOverrides = {}): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    scanCxImports: () => ({ stylesBindings: new Map(), bindings: [] }),
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  const { selectorMapForPath = () => null, styleDocumentForPath, ...providerOverrides } = overrides;
  return {
    analysisCache,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    styleDocumentForPath:
      styleDocumentForPath ??
      ((path: string) => {
        const selectors = selectorMapForPath(path);
        return selectors ? buildStyleDocumentFromSelectorMap(path, selectors) : null;
      }),
    typeResolver: new FakeTypeResolver(),
    semanticReferenceIndex: new NullSemanticWorkspaceReferenceIndex(),
    styleDependencyGraph: new WorkspaceStyleDependencyGraph(),
    workspaceRoot: "/fake/ws",
    workspaceFolderUri: "file:///fake/ws",
    logError: NOOP_LOG_ERROR,
    invalidateStyle: () => {},
    peekStyleDocument: () => null,
    buildStyleDocument: (path: string) => {
      const selectors = selectorMapForPath(path);
      return selectors
        ? buildStyleDocumentFromSelectorMap(path, selectors)
        : buildStyleDocumentFromSelectorMap(path, new Map());
    },
    readStyleFile: () => null,
    fileExists: () => true,
    pushStyleFile: () => {},
    indexerReady: Promise.resolve(),
    stopIndexer: () => {},
    settings: DEFAULT_SETTINGS,
    rebuildAliasResolver: () => {},
    ...providerOverrides,
  };
}
