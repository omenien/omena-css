import { describe, expect, it, vi } from "vitest";
import ts from "typescript";
import { CompletionItemKind } from "vscode-languageserver-protocol/node";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handleCompletion } from "../../../server/lsp-server/src/providers/completion";
import { detectClassUtilImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import { EMPTY_ALIAS_RESOLVER, info, makeBaseDeps } from "../../_fixtures/test-helpers";
import {
  cursorFixture,
  scenario,
  workspace,
  type CmeWorkspace,
} from "../../../packages/vitest-cme/src";

const SOURCE_PATH = "/fake/ws/src/Button.tsx";
const SOURCE_URI = "file:///fake/ws/src/Button.tsx";
const STYLE_PATH = "/fake/ws/src/Button.module.scss";
const STYLE_URI = "file:///fake/ws/src/Button.module.scss";
const CX_COMPLETION_WORKSPACE = workspace({
  [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const el = cx('/*|*/
`,
});
const CX_BINDING_RANGE = CX_COMPLETION_WORKSPACE.range("binding", SOURCE_PATH).range;

const detectCxBindings = (sourceFile: ts.SourceFile): CxBinding[] =>
  sourceFile.text.includes("classnames/bind") && sourceFile.text.includes(".module.")
    ? [
        {
          cxVarName: "cx",
          stylesVarName: "styles",
          scssModulePath: "/fake/ws/src/Button.module.scss",
          classNamesImportName: "classNames",
          bindingRange: CX_BINDING_RANGE,
        },
      ]
    : [];

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
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
      new Map([
        ["indicator", info("indicator")],
        ["active", info("active")],
      ]),
    ...overrides,
  });
}

function completionCursor(
  fixture: CmeWorkspace,
  markerName = "cursor",
  filePath = SOURCE_PATH,
  documentUri = SOURCE_URI,
) {
  return cursorFixture({
    workspace: fixture,
    filePath,
    documentUri,
    markerName,
    version: 1,
  });
}

describe("handleCompletion", () => {
  it("returns all classes when inside a cx() call", async () => {
    const spec = scenario({
      name: "cx completion",
      workspace: CX_COMPLETION_WORKSPACE,
      actions: {
        completion: ({ target, workspace: fixture }) =>
          handleCompletion(completionCursor(fixture, target.name, target.filePath), makeDeps()),
      },
    });

    const result = await spec.completion();
    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result!.map((r) => r.label).toSorted()).toEqual(["active", "indicator"]);
    expect(result![0]!.kind).toBe(CompletionItemKind.Value);
  });

  it("returns null when not inside a cx call", () => {
    const notInCallWorkspace = workspace({
      [SOURCE_PATH]: `
/*|*/import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('
`,
    });
    const result = handleCompletion(completionCursor(notInCallWorkspace), makeDeps());
    expect(result).toBeNull();
  });

  it("returns null when file does not import classnames/bind", () => {
    const plainPath = "/fake/ws/src/Plain.tsx";
    const plainUri = "file:///fake/ws/src/Plain.tsx";
    const plainWorkspace = workspace({
      [plainPath]: "const /*|*/x = 1;\n",
    });
    const result = handleCompletion(
      completionCursor(plainWorkspace, "cursor", plainPath, plainUri),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("returns null when classMap is empty", () => {
    const result = handleCompletion(
      completionCursor(CX_COMPLETION_WORKSPACE),
      makeDeps({ selectorMapForPath: () => new Map() }),
    );
    expect(result).toBeNull();
  });

  it("logs and returns null on exception", () => {
    const logError = vi.fn();
    const result = handleCompletion(
      completionCursor(CX_COMPLETION_WORKSPACE),
      makeDeps({
        styleDocumentForPath: () => {
          throw new Error("boom");
        },
        logError,
      }),
    );
    expect(result).toBeNull();
    expect(logError).toHaveBeenCalledTimes(1);
  });

  it("returns CSS custom property completions inside style files", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: `:root { --brand: #0af; }
.button {
  color: var(--br/*|*/)
}
`,
    });
    const params = completionCursor(styleWorkspace, "cursor", STYLE_PATH, STYLE_URI);
    const styleDocument = parseStyleDocument(params.content, STYLE_PATH);
    const result = handleCompletion(
      params,
      makeDeps({
        styleDocumentForPath: () => styleDocument,
      }),
    );

    expect(result).not.toBeNull();
    expect(result![0]).toMatchObject({
      label: "--brand",
      kind: CompletionItemKind.Variable,
      detail: "CSS custom property",
      textEdit: {
        newText: "--brand",
      },
    });
  });

  it("keeps CSS custom property completions while a style file is syntactically incomplete", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: `:root { --brand: #0af; }
.button {
  color: var(--/*|*/`,
    });
    const params = completionCursor(styleWorkspace, "cursor", STYLE_PATH, STYLE_URI);
    const styleDocument = parseStyleDocument(params.content, STYLE_PATH);
    const result = handleCompletion(
      params,
      makeDeps({
        styleDocumentForPath: () => styleDocument,
      }),
    );

    expect(result).not.toBeNull();
    expect(result![0]).toMatchObject({
      label: "--brand",
      kind: CompletionItemKind.Variable,
      detail: "CSS custom property",
      textEdit: {
        newText: "--brand",
      },
    });
  });
});

describe("detectClassUtilImports", () => {
  function parse(source: string): ts.SourceFile {
    return ts.createSourceFile(
      "/fake/src/App.tsx",
      source,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
  }

  it("detects default import from 'clsx'", () => {
    const sf = parse(`import clsx from 'clsx';`);
    expect(detectClassUtilImports(sf)).toEqual(["clsx"]);
  });

  it("detects default import from 'clsx/lite'", () => {
    const sf = parse(`import clsx from 'clsx/lite';`);
    expect(detectClassUtilImports(sf)).toEqual(["clsx"]);
  });

  it("detects default import from 'classnames'", () => {
    const sf = parse(`import classNames from 'classnames';`);
    expect(detectClassUtilImports(sf)).toEqual(["classNames"]);
  });

  it("detects aliased default import", () => {
    const sf = parse(`import cn from 'clsx';`);
    expect(detectClassUtilImports(sf)).toEqual(["cn"]);
  });

  it("detects named import { clsx } from 'clsx'", () => {
    const sf = parse(`import { clsx } from 'clsx';`);
    expect(detectClassUtilImports(sf)).toEqual(["clsx"]);
  });

  it("detects renamed named import { clsx as cx } from 'clsx'", () => {
    const sf = parse(`import { clsx as cx } from 'clsx';`);
    expect(detectClassUtilImports(sf)).toEqual(["cx"]);
  });

  it("detects named import from 'clsx/lite'", () => {
    const sf = parse(`import { clsx } from 'clsx/lite';`);
    expect(detectClassUtilImports(sf)).toEqual(["clsx"]);
  });

  it("does NOT detect 'classnames/bind' (existing pipeline)", () => {
    const sf = parse(`import classNames from 'classnames/bind';`);
    expect(detectClassUtilImports(sf)).toEqual([]);
  });

  it("returns [] for unrelated imports", () => {
    const sf = parse(`import React from 'react';`);
    expect(detectClassUtilImports(sf)).toEqual([]);
  });

  it("collects multiple util imports", () => {
    const sf = parse(`
      import clsx from 'clsx';
      import cn from 'classnames';
    `);
    expect(detectClassUtilImports(sf)).toEqual(["clsx", "cn"]);
  });

  it("collects both default and named from the same specifier", () => {
    // Unusual but valid TS: import clsx, { clsx as cx2 } from 'clsx';
    // In practice this would be a TS error, but the detector should not crash.
    const sf = parse(`import clsx from 'clsx';`);
    expect(detectClassUtilImports(sf)).toEqual(["clsx"]);
  });
});

describe("handleCompletion / clsx path", () => {
  const CLSX_COMPLETION_WORKSPACE = workspace({
    [SOURCE_PATH]: `
import clsx from 'clsx';
import styles from './Button.module.scss';
const el = clsx(styles./*|*/
`,
  });

  function clsxMakeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      // scanCxImports populates stylesBindings so parseClassRefs
      // can see the styles.x access patterns. No cx bindings in
      // the clsx path.
      scanCxImports: () => ({
        stylesBindings: new Map([
          [
            "styles",
            {
              kind: "resolved" as const,
              absolutePath: "/fake/ws/src/Button.module.scss",
            },
          ],
        ]),
        bindings: [],
      }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      detectClassUtilImports,
      max: 10,
    });
    return makeBaseDeps({
      analysisCache,
      selectorMapForPath: (path: string) =>
        path === "/fake/ws/src/Button.module.scss"
          ? new Map([
              ["btn", info("btn")],
              ["active", info("active")],
            ])
          : null,
      ...overrides,
    });
  }

  it("returns class completions inside clsx(styles.|)", async () => {
    const spec = scenario({
      name: "clsx completion",
      workspace: CLSX_COMPLETION_WORKSPACE,
      actions: {
        completion: ({ target, workspace: fixture }) =>
          handleCompletion(completionCursor(fixture, target.name, target.filePath), clsxMakeDeps()),
      },
    });

    const result = await spec.completion();
    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result!.map((r) => r.label).toSorted()).toEqual(["active", "btn"]);
  });

  it("returns null when a local binding shadows the imported clsx identifier", () => {
    const shadowedWorkspace = workspace({
      [SOURCE_PATH]: `
import clsx from 'clsx';
import styles from './Button.module.scss';
function render(clsx: (value: unknown) => string) {
  return clsx(styles./*|*/
}
`,
    });
    const result = handleCompletion(completionCursor(shadowedWorkspace), clsxMakeDeps());
    expect(result).toBeNull();
  });

  it("returns class completions with aliased import (cn from 'clsx')", () => {
    const cnWorkspace = workspace({
      [SOURCE_PATH]: `
import cn from 'clsx';
import styles from './Button.module.scss';
const el = cn(styles./*|*/
`,
    });
    const result = handleCompletion(completionCursor(cnWorkspace), clsxMakeDeps());
    expect(result).not.toBeNull();
    expect(result!.map((r) => r.label).toSorted()).toEqual(["active", "btn"]);
  });

  it("returns class completions with classnames (not /bind)", () => {
    const classNamesWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames';
import styles from './Button.module.scss';
const el = classNames(styles./*|*/
`,
    });
    const result = handleCompletion(completionCursor(classNamesWorkspace), clsxMakeDeps());
    expect(result).not.toBeNull();
    expect(result!.map((r) => r.label).toSorted()).toEqual(["active", "btn"]);
  });

  it("returns class completions with partial prefix (styles.ac)", () => {
    const partialWorkspace = workspace({
      [SOURCE_PATH]: `
import clsx from 'clsx';
import styles from './Button.module.scss';
const el = clsx(styles.ac/*|*/
`,
    });
    const result = handleCompletion(completionCursor(partialWorkspace), clsxMakeDeps());
    expect(result).not.toBeNull();
    expect(result!.map((r) => r.label)).toEqual(["active"]);
  });

  it("returns null when cursor is outside clsx() call", () => {
    const outsideWorkspace = workspace({
      [SOURCE_PATH]: `
import clsx from 'clsx';
import styles from './Button.module.scss';
const x = styles./*|*/
`,
    });
    const result = handleCompletion(completionCursor(outsideWorkspace), clsxMakeDeps());
    expect(result).toBeNull();
  });

  it("returns null when no clsx/classnames import exists", () => {
    const noClsxWorkspace = workspace({
      [SOURCE_PATH]: `
import styles from './Button.module.scss';
const el = someFunc(styles./*|*/
`,
    });
    const result = handleCompletion(completionCursor(noClsxWorkspace), clsxMakeDeps());
    expect(result).toBeNull();
  });

  it("returns null when classMap is empty", () => {
    const result = handleCompletion(
      completionCursor(CLSX_COMPLETION_WORKSPACE),
      clsxMakeDeps({ selectorMapForPath: () => new Map() }),
    );
    expect(result).toBeNull();
  });

  it("returns null for files whose analyzed entry has no relevant bindings", () => {
    const plainPath = "/fake/ws/src/Plain.tsx";
    const plainUri = "file:///fake/ws/src/Plain.tsx";
    const plainWorkspace = workspace({
      [plainPath]: `
import React from 'react';
const el = <div className="foo/*|*/">
`,
    });
    const result = handleCompletion(
      completionCursor(plainWorkspace, "cursor", plainPath, plainUri),
      clsxMakeDeps(),
    );
    expect(result).toBeNull();
  });
});
