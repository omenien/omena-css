import { describe, expect, it } from "vitest";
import type ts from "typescript";
import { reducedProductClassValueUniverseV0 } from "../../../server/engine-core-ts/src/core/abstract-value/class-value-universe";
import type { BinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import type { ResolvedCxBinding } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { checkSourceDocument } from "../../../server/engine-core-ts/src/core/checker";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  info,
} from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const a = cx('indicator');
const b = cx('unknonw');
`;

const PROVIDER_UNIVERSE_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const suffix = pick();
const button = "global_" + suffix;
const value = cx(button);
`;

const detectCxBindings = (_sourceFile: ts.SourceFile): CxBinding[] => [
  {
    cxVarName: "cx",
    stylesVarName: "styles",
    scssModulePath: "/fake/ws/src/Button.module.scss",
    classNamesImportName: "classNames",
    bindingRange: {
      start: { line: 3, character: 6 },
      end: { line: 3, character: 8 },
    },
  },
];

const parseClassExpressions = (_sf: ts.SourceFile, bindings: readonly ResolvedCxBinding[]) =>
  buildTestClassExpressions({
    filePath: "/fake/ws/src/Button.tsx",
    bindings,
    expressions:
      bindings.length === 0
        ? []
        : [
            {
              kind: "literal",
              origin: "cxCall",
              className: "indicator",
              range: { start: { line: 4, character: 14 }, end: { line: 4, character: 23 } },
              scssModulePath: bindings[0]!.scssModulePath,
            },
            {
              kind: "literal",
              origin: "cxCall",
              className: "unknonw",
              range: { start: { line: 5, character: 14 }, end: { line: 5, character: 21 } },
              scssModulePath: bindings[0]!.scssModulePath,
            },
          ],
  });

function makeAnalysisCache() {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  return new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
    parseClassExpressions,
    max: 10,
  });
}

function makeProviderUniverseAnalysisCache() {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  return new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    binderPlugin: providerUniverseBinderPlugin,
    max: 10,
  });
}

const providerUniverseBinderPlugin: BinderPluginV0 = {
  id: "test-cva-provider-universe",
  version: "0",
  stability: "builtIn",
  domains: ["cva-recipes"],
  importTargets: ["class-variance-authority"],
  utilityTargets: ["cva"],
  ownsSurfaces: ["classReferenceExtraction", "classValueUniverseProvider"],
  analyzeSource() {
    return {
      pluginId: "test-cva-provider-universe",
      stylesBindings: new Map(),
      rawCxBindings: [],
      cxBindings: [],
      classUtilNames: [],
      classExpressions: [
        {
          kind: "symbolRef",
          id: "expr:button",
          origin: "cxCall",
          rawReference: "button",
          rootName: "button",
          pathSegments: [],
          range: { start: { line: 6, character: 17 }, end: { line: 6, character: 23 } },
          scssModulePath: "/fake/ws/src/Button.module.scss",
        },
      ],
      domainClassReferences: [],
      classValueUniverses: [
        {
          id: "universe:button",
          pluginId: "cva-recipe-domain",
          domain: "cva-recipe",
          ownerName: "button",
          universe: reducedProductClassValueUniverseV0({
            baseClassNames: ["button_base"],
            axes: [],
          }),
        },
      ],
    };
  },
};

describe("checkSourceDocument", () => {
  const params = {
    documentUri: "file:///fake/ws/src/Button.tsx",
    content: TSX,
    filePath: "/fake/ws/src/Button.tsx",
    version: 1,
  };

  it("returns stable source findings for missing classes", () => {
    const findings = checkSourceDocument(
      params,
      {
        analysisCache: makeAnalysisCache(),
        styleDocumentForPath: () =>
          buildStyleDocumentFromSelectorMap(
            "/fake/ws/src/Button.module.scss",
            new Map([
              ["indicator", info("indicator")],
              ["unknown", info("unknown")],
            ]),
          ),
        typeResolver: new FakeTypeResolver(),
        workspaceRoot: "/fake/ws",
      },
      { includeMissingModule: false },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      category: "source",
      code: "missing-static-class",
      className: "unknonw",
      suggestion: "unknown",
    });
  });

  it("threads provider class-value universes through source diagnostics", () => {
    const findings = checkSourceDocument(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: PROVIDER_UNIVERSE_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      {
        analysisCache: makeProviderUniverseAnalysisCache(),
        styleDocumentForPath: () =>
          buildStyleDocumentFromSelectorMap(
            "/fake/ws/src/Button.module.scss",
            new Map([
              ["button_base", info("button_base")],
              ["global_unknown", info("global_unknown")],
            ]),
          ),
        typeResolver: new FakeTypeResolver(),
        workspaceRoot: "/fake/ws",
      },
      { includeMissingModule: false },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      category: "source",
      code: "missing-resolved-class-domain",
      scssModulePath: "/fake/ws/src/Button.module.scss",
      abstractValue: { kind: "prefix", prefix: "global_" },
    });
  });
});
