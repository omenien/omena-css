import { describe, expect, it, vi } from "vitest";
import {
  CodeActionKind,
  DiagnosticSeverity,
  type CodeActionParams,
  type Diagnostic,
} from "vscode-languageserver-protocol/node";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handleCodeAction } from "../../../server/lsp-server/src/providers/code-actions";
import {
  scenario,
  textDocumentRangeFixture,
  workspace,
  type CmeWorkspace,
} from "../../../packages/vitest-cme/src";
import { makeBaseDeps } from "../../_fixtures/test-helpers";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";

const SOURCE_PATH = "src/Button.tsx";
const SOURCE_URI = "file:///fake/src/Button.tsx";
const STYLE_PATH = "src/Button.module.scss";
const STYLE_URI = "file:///fake/src/Button.module.scss";
const BASE_STYLE_PATH = "/fake/src/Base.module.scss";
const DEFAULT_WORKSPACE = workspace({
  [SOURCE_PATH]: "const cls = cx('/*<missing>*/missing/*</missing>*/');\n",
});
const DEFAULT_DIAGNOSTIC_RANGE = DEFAULT_WORKSPACE.range("missing", SOURCE_PATH).range;
const ZERO_RANGE: Diagnostic["range"] = {
  start: { line: 0, character: 0 },
  end: { line: 0, character: 0 },
};

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  return makeBaseDeps({
    selectorMapForPath: () => new Map(),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

function diagnostic(
  suggestion: string | undefined,
  message = "foo",
  range: Diagnostic["range"] = DEFAULT_DIAGNOSTIC_RANGE,
): Diagnostic {
  const className = /Class '\.([^']+)'/.exec(message)?.[1] ?? "generated";
  return {
    range,
    severity: DiagnosticSeverity.Warning,
    source: "css-module-explainer",
    message,
    data:
      suggestion === undefined
        ? undefined
        : {
            suggestion,
            createSelector: {
              uri: STYLE_URI,
              range: ZERO_RANGE,
              newText: `\n\n.${className} {\n}\n`,
            },
          },
  };
}

function makeParams(diagnostics: Diagnostic[]): CodeActionParams {
  return {
    textDocument: { uri: SOURCE_URI },
    range: diagnostics[0]?.range ?? ZERO_RANGE,
    context: { diagnostics, triggerKind: 1 },
  };
}

function makeMarkedParams(
  fixture: CmeWorkspace,
  rangeName: string,
  diagnostics: Diagnostic[],
): CodeActionParams {
  return {
    ...textDocumentRangeFixture({
      workspace: fixture,
      documentUri: SOURCE_URI,
      filePath: SOURCE_PATH,
      rangeName,
    }),
    context: { diagnostics, triggerKind: 1 },
  };
}

describe("handleCodeAction", () => {
  it("returns replace and create actions for a diagnostic with a suggestion", async () => {
    const ws = workspace({
      [SOURCE_PATH]: "const cls = cx('/*<missing>*/indicaror/*</missing>*/');/*|*/",
    });
    const d = diagnostic(
      "indicator",
      "Class '.indicaror' not found. Did you mean 'indicator'?",
      ws.range("missing", SOURCE_PATH).range,
    );
    const spec = scenario({
      name: "code action quick fix",
      workspace: ws,
      actions: {
        codeAction: ({ workspace: fixture }) =>
          handleCodeAction(makeMarkedParams(fixture, "missing", [d]), makeDeps()),
      },
    });

    const result = await spec.codeAction();
    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    const action = result![0]!;
    expect(action.title).toBe("Replace with 'indicator'");
    expect(action.kind).toBe(CodeActionKind.QuickFix);
    expect(action.isPreferred).toBe(true);
    expect(action.diagnostics).toEqual([d]);
    const edits = action.edit?.changes?.[SOURCE_URI];
    expect(edits).toHaveLength(1);
    expect(edits![0]!.newText).toBe("indicator");
    expect(edits![0]!.range).toEqual(d.range);

    const createAction = result![1]!;
    expect(createAction.title).toBe("Add '.indicaror' to Button.module.scss");
    const createEdits = createAction.edit?.changes?.[STYLE_URI];
    expect(createEdits).toHaveLength(1);
    expect(createEdits![0]!.newText).toBe("\n\n.indicaror {\n}\n");
  });

  it("returns null when no diagnostic carries a suggestion", () => {
    const result = handleCodeAction(
      makeParams([
        {
          range: DEFAULT_DIAGNOSTIC_RANGE,
          severity: DiagnosticSeverity.Warning,
          source: "css-module-explainer",
          message: "whatever",
        },
      ]),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("skips diagnostics with non-string or empty suggestion payloads", () => {
    const withBadShape: Diagnostic = {
      ...diagnostic("keep"),
      data: { suggestion: 123 }, // wrong type
    };
    const empty = { ...diagnostic(""), data: { suggestion: "" } };
    const good = diagnostic("real-one");
    const result = handleCodeAction(makeParams([withBadShape, empty, good]), makeDeps());
    expect(result).toHaveLength(2);
    expect(result![0]!.title).toBe("Replace with 'real-one'");
  });

  it("returns a create-selector quick fix even when there is no typo suggestion", () => {
    const d: Diagnostic = {
      range: DEFAULT_DIAGNOSTIC_RANGE,
      severity: DiagnosticSeverity.Warning,
      source: "css-module-explainer",
      message: "Class '.missing' not found in Button.module.scss.",
      data: {
        createSelector: {
          uri: STYLE_URI,
          range: ZERO_RANGE,
          newText: "\n\n.missing {\n}\n",
        },
      },
    };
    const result = handleCodeAction(makeParams([d]), makeDeps());
    expect(result).toHaveLength(1);
    expect(result![0]!.title).toBe("Add '.missing' to Button.module.scss");
  });

  it("returns a create-module quick fix for a missing-module diagnostic", () => {
    const moduleWorkspace = workspace({
      [SOURCE_PATH]: "import styles from /*<module>*/'./Button.module.scss'/*</module>*/;\n",
    });
    const d: Diagnostic = {
      range: moduleWorkspace.range("module", SOURCE_PATH).range,
      severity: DiagnosticSeverity.Warning,
      source: "css-module-explainer",
      message: "Cannot resolve CSS Module './Button.module.scss'. The file does not exist.",
      code: "missing-module",
      data: {
        createModuleFile: {
          uri: STYLE_URI,
        },
      },
    };
    const result = handleCodeAction(makeParams([d]), makeDeps());
    expect(result).toHaveLength(1);
    expect(result![0]!.title).toBe("Create Button.module.scss");
    expect(result![0]!.kind).toBe(CodeActionKind.QuickFix);
    expect(result![0]!.isPreferred).toBe(true);
    expect(result![0]!.edit?.documentChanges).toEqual([
      {
        kind: "create",
        uri: STYLE_URI,
        options: { overwrite: false, ignoreIfExists: true },
      },
    ]);
  });

  it("returns a create-custom-property quick fix for style diagnostics", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: ".button { color: /*<token>*/var(--missing)/*</token>*/; }\n",
    });
    const d: Diagnostic = {
      range: styleWorkspace.range("token", STYLE_PATH).range,
      severity: DiagnosticSeverity.Warning,
      source: "css-module-explainer",
      message: "CSS custom property '--missing' not found in indexed style tokens.",
      data: {
        createCustomProperty: {
          uri: STYLE_URI,
          range: ZERO_RANGE,
          newText: "\n\n:root {\n  --missing: ;\n}\n",
        },
      },
    };
    const result = handleCodeAction(makeParams([d]), makeDeps());
    expect(result).toHaveLength(1);
    expect(result![0]!.title).toBe("Add '--missing' to Button.module.scss");
    expect(result![0]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: ZERO_RANGE,
        newText: "\n\n:root {\n  --missing: ;\n}\n",
      },
    ]);
  });

  it("returns a create-Sass-symbol quick fix for style diagnostics", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: "/*<variable>*/$missing/*</variable>*/: ;\n",
    });
    const d: Diagnostic = {
      range: styleWorkspace.range("variable", STYLE_PATH).range,
      severity: DiagnosticSeverity.Warning,
      source: "css-module-explainer",
      message: "Sass variable '$missing' not found in this file.",
      data: {
        createSassSymbol: {
          uri: STYLE_URI,
          range: ZERO_RANGE,
          newText: "$missing: ;\n\n",
        },
      },
    };
    const result = handleCodeAction(makeParams([d]), makeDeps());
    expect(result).toHaveLength(1);
    expect(result![0]!.title).toBe("Add '$missing' to Button.module.scss");
    expect(result![0]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: ZERO_RANGE,
        newText: "$missing: ;\n\n",
      },
    ]);
  });

  it("returns extract refactors for selected style values", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: ".button { color: /*<value>*/#fff/*</value>*/; }\n",
    });
    const params: CodeActionParams = {
      ...textDocumentRangeFixture({
        workspace: styleWorkspace,
        documentUri: STYLE_URI,
        filePath: STYLE_PATH,
        rangeName: "value",
      }),
      context: { diagnostics: [], triggerKind: 1 },
    };
    const result = handleCodeAction(params, makeDeps(), styleWorkspace.file(STYLE_PATH).content);

    expect(result).toHaveLength(2);
    expect(result![0]!.kind).toBe(CodeActionKind.RefactorExtract);
    expect(result![0]!.title).toBe("Extract CSS custom property '--extracted-color'");
    expect(result![0]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: ZERO_RANGE,
        newText: ":root {\n  --extracted-color: #fff;\n}\n\n",
      },
      {
        range: styleWorkspace.range("value", STYLE_PATH).range,
        newText: "var(--extracted-color)",
      },
    ]);
    expect(result![1]!.kind).toBe(CodeActionKind.RefactorExtract);
    expect(result![1]!.title).toBe("Extract @value 'extractedColor'");
    expect(result![1]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: ZERO_RANGE,
        newText: "@value extractedColor: #fff;\n\n",
      },
      {
        range: styleWorkspace.range("value", STYLE_PATH).range,
        newText: "extractedColor",
      },
    ]);
  });

  it("returns an inline composed class refactor for same-file composes tokens", () => {
    const styleWorkspace = workspace({
      [STYLE_PATH]: `.base { color: red; padding: 4px; }
.button {
  composes: /*<compose>*/base/*</compose>*/;
  background: blue;
}
`,
    });
    const params: CodeActionParams = {
      ...textDocumentRangeFixture({
        workspace: styleWorkspace,
        documentUri: STYLE_URI,
        filePath: STYLE_PATH,
        rangeName: "compose",
      }),
      context: { diagnostics: [], triggerKind: 1 },
    };
    const result = handleCodeAction(params, makeDeps(), styleWorkspace.file(STYLE_PATH).content);

    expect(result).toHaveLength(1);
    expect(result![0]!.kind).toBe(CodeActionKind.RefactorInline);
    expect(result![0]!.title).toBe("Inline composed class 'base'");
    expect(result![0]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: {
          start: { line: 2, character: 2 },
          end: { line: 2, character: 17 },
        },
        newText: "color: red;\n  padding: 4px;",
      },
    ]);
  });

  it("returns an inline composed class refactor for cross-file composes tokens", () => {
    const baseScss = `.base { color: red; padding: 4px; }\n`;
    const styleWorkspace = workspace({
      [STYLE_PATH]: `.button {
  composes: /*<compose>*/base/*</compose>*/ from "./Base.module.scss";
  background: blue;
}
`,
    });
    const params: CodeActionParams = {
      ...textDocumentRangeFixture({
        workspace: styleWorkspace,
        documentUri: STYLE_URI,
        filePath: STYLE_PATH,
        rangeName: "compose",
      }),
      context: { diagnostics: [], triggerKind: 1 },
    };
    const result = handleCodeAction(
      params,
      makeDeps({
        styleDocumentForPath: (filePath) =>
          filePath === BASE_STYLE_PATH ? parseStyleDocument(baseScss, filePath) : null,
      }),
      styleWorkspace.file(STYLE_PATH).content,
    );

    expect(result).toHaveLength(1);
    expect(result![0]!.kind).toBe(CodeActionKind.RefactorInline);
    expect(result![0]!.title).toBe("Inline composed class 'base' from Base.module.scss");
    expect(result![0]!.edit?.changes?.[STYLE_URI]).toEqual([
      {
        range: {
          start: { line: 1, character: 2 },
          end: { line: 1, character: 43 },
        },
        newText: "color: red;\n  padding: 4px;",
      },
    ]);
  });

  it("inlines transitive composed class declarations in dependency order", () => {
    const baseScss = `.reset { box-sizing: border-box; }
.base { composes: reset; color: red; }
`;
    const styleWorkspace = workspace({
      [STYLE_PATH]: `.button {
  composes: /*<compose>*/base/*</compose>*/ from "./Base.module.scss";
}
`,
    });
    const params: CodeActionParams = {
      ...textDocumentRangeFixture({
        workspace: styleWorkspace,
        documentUri: STYLE_URI,
        filePath: STYLE_PATH,
        rangeName: "compose",
      }),
      context: { diagnostics: [], triggerKind: 1 },
    };
    const result = handleCodeAction(
      params,
      makeDeps({
        styleDocumentForPath: (filePath) =>
          filePath === BASE_STYLE_PATH ? parseStyleDocument(baseScss, filePath) : null,
      }),
      styleWorkspace.file(STYLE_PATH).content,
    );

    expect(result).toHaveLength(1);
    expect(result![0]!.edit?.changes?.[STYLE_URI]?.[0]?.newText).toBe(
      "box-sizing: border-box;\n  color: red;",
    );
  });

  it("returns sibling module creation actions for a TSX file without an existing sibling module", () => {
    const result = handleCodeAction(makeParams([]), makeDeps({ fileExists: () => false }));
    expect(result).toHaveLength(3);
    expect(result?.map((action) => action.title)).toEqual([
      "Create Button.module.scss",
      "Create Button.module.css",
      "Create Button.module.less",
    ]);
    expect(result?.every((action) => action.kind === CodeActionKind.QuickFix)).toBe(true);
    expect(result?.[0]?.edit?.documentChanges).toEqual([
      {
        kind: "create",
        uri: STYLE_URI,
        options: { overwrite: false, ignoreIfExists: true },
      },
    ]);
  });

  it("does not return sibling module creation actions when a sibling module already exists", () => {
    const result = handleCodeAction(
      makeParams([]),
      makeDeps({
        fileExists: (path) => path.endsWith("Button.module.scss"),
      }),
    );
    expect(result).toBeNull();
  });

  it("logs and returns null on exception", () => {
    const logError = vi.fn();
    // Poison the diagnostics iterable so for-of throws.
    const poisonedParams = {
      textDocument: { uri: SOURCE_URI },
      range: DEFAULT_DIAGNOSTIC_RANGE,
      context: {
        diagnostics: new Proxy([diagnostic("x")], {
          get(target, prop) {
            if (prop === Symbol.iterator) throw new Error("boom");
            return Reflect.get(target, prop);
          },
        }) as Diagnostic[],
        triggerKind: 1,
      },
    } satisfies CodeActionParams;
    const result = handleCodeAction(poisonedParams, makeDeps({ logError }));
    expect(result).toBeNull();
    expect(logError).toHaveBeenCalledWith("codeAction handler failed", expect.any(Error));
  });
});
