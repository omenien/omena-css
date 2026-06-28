import ts from "typescript";
import { describe, expect, it } from "vitest";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/binder/binder-builder";
import { buildSourceBindingGraph } from "../../../server/engine-core-ts/src/core/binder/source-binding-graph";
import type { AnalysisEntry } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  makeSourceDocumentHIR,
  makeStyleImportBinding,
  makeSymbolRefClassExpression,
} from "../../../server/engine-core-ts/src/core/hir/source-types";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { collectSemanticReferenceContribution } from "../../../server/engine-core-ts/src/core/semantic/reference-collector";
import type { StyleImport } from "@omena/shared";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import { info } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";
import type { SemanticModuleUsageSite } from "../../../server/engine-core-ts/src/core/semantic/reference-collector";
import type { SemanticReferenceSite } from "../../../server/engine-core-ts/src/core/semantic/reference-types";

const FILE_PATH = "/fake/ws/src/App.tsx";
const SCSS_PATH = "/fake/ws/src/Button.module.scss";
const URI = "file:///fake/ws/src/App.tsx";

describe("WorkspaceSemanticWorkspaceReferenceIndex", () => {
  it("tracks referencing uris for unresolved dynamic module usage", () => {
    const entry = makeEntry({
      sourceText: "cx(size);",
      rawReference: "size",
      rootName: "size",
      token: "size",
    });
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([["indicator", info("indicator")]]),
    );

    const contribution = collectSemanticReferenceContribution(URI, entry, {
      styleDocumentForPath: (path) => (path === SCSS_PATH ? styleDocument : null),
      typeResolver: new FakeTypeResolver(),
      filePath: FILE_PATH,
      workspaceRoot: "/fake/ws",
      settingsKey: "transform:asIs;alias:",
    });

    expect(contribution.referenceSites).toEqual([]);
    expect(contribution.moduleUsages).toMatchObject([
      {
        scssModulePath: SCSS_PATH,
        isDynamic: true,
        hasResolvedTargets: false,
      },
    ]);

    const index = new WorkspaceSemanticWorkspaceReferenceIndex();
    index.record(URI, contribution.referenceSites, contribution.moduleUsages);
    expect(index.findReferencingUris(SCSS_PATH)).toEqual([URI]);
  });

  it("marks type-resolved dynamic usage as resolved", () => {
    const entry = makeEntry({
      sourceText: ["const size = 'indicator';", "cx(size);"].join("\n"),
      rawReference: "size",
      rootName: "size",
      token: "size",
      tokenOccurrence: "last",
    });
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([["indicator", info("indicator")]]),
    );

    const contribution = collectSemanticReferenceContribution(URI, entry, {
      styleDocumentForPath: (path) => (path === SCSS_PATH ? styleDocument : null),
      typeResolver: new FakeTypeResolver(["indicator"]),
      filePath: FILE_PATH,
      workspaceRoot: "/fake/ws",
      settingsKey: "transform:asIs;alias:",
    });

    expect(contribution.referenceSites).toHaveLength(1);
    expect(contribution.moduleUsages).toMatchObject([
      {
        scssModulePath: SCSS_PATH,
        isDynamic: true,
        hasResolvedTargets: true,
      },
    ]);
  });

  it("uses the analysis entry binding graph for style dependency discovery", () => {
    const entry = makeEntry({
      sourceText: ["const size = 'indicator';", "cx(size);"].join("\n"),
      rawReference: "size",
      rootName: "size",
      token: "size",
      tokenOccurrence: "last",
    });
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([["indicator", info("indicator")]]),
    );
    const lookedUpStylePaths: string[] = [];
    const contribution = collectSemanticReferenceContribution(
      URI,
      {
        ...entry,
        sourceBindingGraph: {
          filePath: FILE_PATH,
          nodes: [],
          edges: [],
        },
      },
      {
        styleDocumentForPath: (path) => {
          lookedUpStylePaths.push(path);
          return path === SCSS_PATH ? styleDocument : null;
        },
        typeResolver: new FakeTypeResolver(),
        filePath: FILE_PATH,
        workspaceRoot: "/fake/ws",
        settingsKey: "transform:asIs;alias:",
      },
    );

    expect(lookedUpStylePaths).toEqual([]);
    expect(contribution.referenceSites).toEqual([]);
    expect(contribution.moduleUsages).toEqual([]);
    expect(contribution.deps.stylePaths).toEqual([]);
  });

  it("indexes contribution dependencies by workspaceRoot and settingsKey", () => {
    const index = new WorkspaceSemanticWorkspaceReferenceIndex();
    index.record(
      "file:///fake/ws/src/App.tsx",
      [],
      [
        {
          refId: "ref:1",
          uri: "file:///fake/ws/src/App.tsx",
          filePath: "/fake/ws/src/App.tsx",
          range: {
            start: { line: 0, character: 0 },
            end: { line: 0, character: 1 },
          },
          origin: "cxCall",
          scssModulePath: SCSS_PATH,
          expressionKind: "symbolRef",
          hasResolvedTargets: false,
          isDynamic: true,
        },
      ],
      {
        workspaceRoot: "/fake/ws",
        settingsKey: "transform:asIs;alias:",
        stylePaths: [SCSS_PATH],
        sourcePaths: [FILE_PATH, "/fake/ws/src/theme.ts"],
      },
    );

    expect(index.findUrisBySettingsDependency("/fake/ws", "transform:asIs;alias:")).toEqual([
      "file:///fake/ws/src/App.tsx",
    ]);
    expect(index.findUrisBySourceDependency("/fake/ws", "/fake/ws/src/theme.ts")).toEqual([
      "file:///fake/ws/src/App.tsx",
    ]);
  });

  it("removes stale selector entries on update and forget", () => {
    const index = new WorkspaceSemanticWorkspaceReferenceIndex();
    index.record(
      URI,
      [makeReferenceSite(SCSS_PATH, "button", "ref:1")],
      [makeModuleUsage(URI, SCSS_PATH, "ref:1")],
      makeDeps(),
    );
    index.record(
      URI,
      [makeReferenceSite(SCSS_PATH, "card", "ref:2")],
      [makeModuleUsage(URI, SCSS_PATH, "ref:2")],
      makeDeps(),
    );

    expect(index.findSelectorReferences(SCSS_PATH, "button")).toEqual([]);
    expect(index.findSelectorReferences(SCSS_PATH, "card")).toHaveLength(1);

    index.forget(URI);
    expect(index.findSelectorReferences(SCSS_PATH, "card")).toEqual([]);
    expect(index.findReferencingUris(SCSS_PATH)).toEqual([]);
  });

  it("matches a rebuild oracle for update sequences", () => {
    const operations: ReadonlyArray<
      | {
          readonly kind: "record";
          readonly uri: string;
          readonly sites: readonly SemanticReferenceSite[];
          readonly usages: readonly SemanticModuleUsageSite[];
        }
      | { readonly kind: "forget"; readonly uri: string }
    > = [
      {
        kind: "record",
        uri: "file:///fake/ws/src/App.tsx",
        sites: [makeReferenceSite(SCSS_PATH, "button", "ref:1")],
        usages: [makeModuleUsage("file:///fake/ws/src/App.tsx", SCSS_PATH, "ref:1")],
      },
      {
        kind: "record",
        uri: "file:///fake/ws/src/Card.tsx",
        sites: [makeReferenceSite(SCSS_PATH, "card", "ref:2")],
        usages: [makeModuleUsage("file:///fake/ws/src/Card.tsx", SCSS_PATH, "ref:2")],
      },
      {
        kind: "record",
        uri: "file:///fake/ws/src/App.tsx",
        sites: [makeReferenceSite(SCSS_PATH, "chip", "ref:3")],
        usages: [makeModuleUsage("file:///fake/ws/src/App.tsx", SCSS_PATH, "ref:3")],
      },
      { kind: "forget", uri: "file:///fake/ws/src/Card.tsx" },
    ];

    const incremental = new WorkspaceSemanticWorkspaceReferenceIndex();
    const oracle = new Map<
      string,
      {
        readonly sites: readonly SemanticReferenceSite[];
        readonly usages: readonly SemanticModuleUsageSite[];
      }
    >();

    for (const op of operations) {
      if (op.kind === "record") {
        incremental.record(op.uri, op.sites, op.usages, makeDeps());
        oracle.set(op.uri, { sites: op.sites, usages: op.usages });
      } else {
        incremental.forget(op.uri);
        oracle.delete(op.uri);
      }

      const rebuilt = buildOracleState(oracle);
      expect(incremental.findAllForScssPath(SCSS_PATH)).toEqual(
        rebuilt.findAllForScssPath(SCSS_PATH),
      );
      expect(incremental.findReferencingUris(SCSS_PATH)).toEqual(
        rebuilt.findReferencingUris(SCSS_PATH),
      );
      expect(incremental.findUrisBySettingsDependency("/fake/ws", "transform:asIs;alias:")).toEqual(
        rebuilt.findUrisBySettingsDependency("/fake/ws", "transform:asIs;alias:"),
      );
      expect(incremental.findUrisBySourceDependency("/fake/ws", "/fake/ws/src/theme.ts")).toEqual(
        rebuilt.findUrisBySourceDependency("/fake/ws", "/fake/ws/src/theme.ts"),
      );
    }
  });
});

function buildOracleState(
  contributions: ReadonlyMap<
    string,
    {
      readonly sites: readonly SemanticReferenceSite[];
      readonly usages: readonly SemanticModuleUsageSite[];
    }
  >,
): {
  readonly findAllForScssPath: (scssPath: string) => readonly SemanticReferenceSite[];
  readonly findReferencingUris: (scssPath: string) => readonly string[];
  readonly findUrisBySettingsDependency: (
    workspaceRoot: string,
    settingsKey: string,
  ) => readonly string[];
  readonly findUrisBySourceDependency: (
    workspaceRoot: string,
    sourcePath: string,
  ) => readonly string[];
} {
  const ordered = [...contributions.entries()];
  return {
    findAllForScssPath(scssPath) {
      return ordered.flatMap(([, contribution]) =>
        contribution.sites.filter((site) => site.selectorFilePath === scssPath),
      );
    },
    findReferencingUris(scssPath) {
      const uris = ordered.flatMap(([, contribution]) =>
        contribution.usages
          .filter((usage) => usage.scssModulePath === scssPath)
          .map((usage) => usage.uri),
      );
      return [...new Set(uris)].toSorted();
    },
    findUrisBySettingsDependency(workspaceRoot, settingsKey) {
      if (workspaceRoot !== "/fake/ws" || settingsKey !== "transform:asIs;alias:") {
        return [];
      }
      return ordered.map(([uri]) => uri);
    },
    findUrisBySourceDependency(workspaceRoot, sourcePath) {
      if (workspaceRoot !== "/fake/ws" || sourcePath !== "/fake/ws/src/theme.ts") {
        return [];
      }
      return ordered.map(([uri]) => uri);
    },
  };
}

function makeDeps() {
  return {
    workspaceRoot: "/fake/ws",
    settingsKey: "transform:asIs;alias:",
    stylePaths: [SCSS_PATH],
    sourcePaths: [FILE_PATH, "/fake/ws/src/theme.ts"],
  };
}

function makeModuleUsage(
  uri: string,
  scssModulePath: string,
  refId: string,
): SemanticModuleUsageSite {
  return {
    refId,
    uri,
    filePath: uri.replace("file://", ""),
    range: {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 1 },
    },
    origin: "cxCall",
    scssModulePath,
    expressionKind: "symbolRef",
    hasResolvedTargets: true,
    isDynamic: true,
  };
}

function makeReferenceSite(
  selectorFilePath: string,
  canonicalName: string,
  refId: string,
): SemanticReferenceSite {
  return {
    refId,
    selectorId: `selector:${selectorFilePath}:${canonicalName}`,
    filePath: FILE_PATH,
    uri: URI,
    range: {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 1 },
    },
    origin: "cxCall",
    scssModulePath: selectorFilePath,
    selectorFilePath,
    canonicalName,
    className: canonicalName,
    selectorCertainty: "exact",
    reason: "literal",
    expansion: "direct",
  };
}

function makeEntry(args: {
  sourceText: string;
  rawReference: string;
  rootName: string;
  token: string;
  tokenOccurrence?: "first" | "last";
}): AnalysisEntry {
  const sourceFile = ts.createSourceFile(
    FILE_PATH,
    args.sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const sourceDocument = makeSourceDocumentHIR({
    filePath: FILE_PATH,
    language: "tsx",
    styleImports: [
      makeStyleImportBinding("style:styles", "styles", "decl:0", {
        kind: "resolved",
        absolutePath: SCSS_PATH,
      } satisfies StyleImport),
    ],
    utilityBindings: [],
    classExpressions: [
      makeSymbolRefClassExpression(
        "ref:size",
        "cxCall",
        SCSS_PATH,
        args.rawReference,
        args.rootName,
        [],
        rangeForToken(sourceFile, args.token, args.tokenOccurrence ?? "first"),
      ),
    ],
  });

  return {
    version: 1,
    contentHash: "fixture",
    sourceFile,
    sourceBinder: buildSourceBinder(sourceFile),
    sourceBindingGraph: buildSourceBindingGraph(sourceDocument, buildSourceBinder(sourceFile)),
    sourceDocument,
    stylesBindings: new Map([
      [
        "styles",
        {
          kind: "resolved",
          absolutePath: SCSS_PATH,
        } satisfies StyleImport,
      ],
    ]),
    classUtilNames: [],
    classValueUniverses: [],
    sourceDependencyPaths: [FILE_PATH, "/fake/ws/src/theme.ts"],
  };
}

function rangeForToken(sourceFile: ts.SourceFile, token: string, occurrence: "first" | "last") {
  const start =
    occurrence === "last" ? sourceFile.text.lastIndexOf(token) : sourceFile.text.indexOf(token);
  if (start === -1) throw new Error(`Token not found: ${token}`);
  const end = start + token.length;
  const startLc = sourceFile.getLineAndCharacterOfPosition(start);
  const endLc = sourceFile.getLineAndCharacterOfPosition(end);
  return {
    start: { line: startLc.line, character: startLc.character },
    end: { line: endLc.line, character: endLc.character },
  };
}
