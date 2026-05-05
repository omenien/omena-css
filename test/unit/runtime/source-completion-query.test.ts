import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { detectClassUtilImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import { resolveSourceCompletionSelectors } from "../../../server/engine-host-node/src/source-completion-query";
import { EMPTY_ALIAS_RESOLVER, info, makeBaseDeps } from "../../_fixtures/test-helpers";

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
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
    detectClassUtilImports,
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
