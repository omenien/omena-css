import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { detectClassUtilImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import { resolveSourceCompletionSelectors } from "../../../server/engine-host-node/src/source-completion-query";
import {
  EMPTY_ALIAS_RESOLVER,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('
`;

const PREFIX_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('btn-
`;

const PREFIX_SUFFIX_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('btn--active
`;

const DIRECT_STYLES_TSX = `
import styles from './Button.module.scss';
const el = styles.btn-
`;

const DIRECT_BRACKET_TSX = `
import styles from './Button.module.scss';
const el = styles["btn-
`;

const SHADOWED_STYLES_TSX = `
import styles from './Button.module.scss';
function render(styles: Record<string, string>) {
  return styles.btn-
}
`;

const detectCxBindings = (sourceFile: ts.SourceFile): CxBinding[] =>
  sourceFile.text.includes("classnames/bind") && sourceFile.text.includes(".module.")
    ? [
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
      ]
    : [];

function makeDeps(
  selectorNames = ["indicator", "active", "btn-primary", "btn-secondary"],
): ProviderDeps {
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({
      stylesBindings: sf.text.includes("import styles")
        ? new Map([
            [
              "styles",
              { kind: "resolved" as const, absolutePath: "/fake/ws/src/Button.module.scss" },
            ],
          ])
        : new Map(),
      bindings: detectCxBindings(sf),
    }),
    detectClassUtilImports,
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () =>
      new Map(selectorNames.map((selectorName) => [selectorName, info(selectorName)])),
  });
}

describe("resolveSourceCompletionSelectors", () => {
  it("returns selector candidates inside a class utility call", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 4,
        character: 16,
        version: 1,
      },
      makeDeps(),
    );

    expect(result.map((selector) => selector.name).toSorted()).toEqual([
      "active",
      "btn-primary",
      "btn-secondary",
      "indicator",
    ]);
  });

  it("narrows selector candidates by the in-progress class value prefix", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: PREFIX_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 4,
        character: 19,
        version: 1,
      },
      makeDeps(),
    );

    expect(result.map((selector) => selector.name).toSorted()).toEqual([
      "btn-primary",
      "btn-secondary",
    ]);
  });

  it("narrows selector candidates by in-progress prefix and suffix tokens", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: PREFIX_SUFFIX_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 4,
        character: 19,
        version: 1,
      },
      makeDeps([
        "btn-primary",
        "btn-disabled",
        "btn-primary-active",
        "btn-secondary-active",
        "card-active",
      ]),
    );

    expect(result.map((selector) => selector.name).toSorted()).toEqual([
      "btn-primary-active",
      "btn-secondary-active",
    ]);
  });

  it("returns narrowed selector candidates for direct styles property access", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: DIRECT_STYLES_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 2,
        character: 22,
        version: 1,
      },
      makeDeps(["btn-primary", "btn-secondary", "card"]),
    );

    expect(result.map((selector) => selector.name).toSorted()).toEqual([
      "btn-primary",
      "btn-secondary",
    ]);
  });

  it("returns narrowed selector candidates for direct styles bracket access", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: DIRECT_BRACKET_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 2,
        character: 23,
        version: 1,
      },
      makeDeps(["btn-primary", "btn-secondary", "card"]),
    );

    expect(result.map((selector) => selector.name).toSorted()).toEqual([
      "btn-primary",
      "btn-secondary",
    ]);
  });

  it("does not return direct styles completions when a local binding shadows the import", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: SHADOWED_STYLES_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 3,
        character: 20,
        version: 1,
      },
      makeDeps(["btn-primary", "btn-secondary", "card"]),
    );

    expect(result).toEqual([]);
  });

  it("returns an empty list outside a class utility call", () => {
    const result = resolveSourceCompletionSelectors(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: TSX,
        filePath: "/fake/ws/src/Button.tsx",
        line: 1,
        character: 0,
        version: 1,
      },
      makeDeps(),
    );

    expect(result).toEqual([]);
  });
});
