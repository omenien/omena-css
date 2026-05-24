import { afterEach, describe, expect, it } from "vitest";
import {
  CodeActionKind,
  DiagnosticSeverity,
  type CodeActionParams,
  type Diagnostic,
  type Range,
} from "vscode-languageserver-protocol/node";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";

const BUTTON_TSX_URI = "file:///fake/workspace/src/Button.tsx";
const BUTTON_SCSS_URI = "file:///fake/workspace/src/Button.module.scss";
const ZERO_RANGE: Range = {
  start: { line: 0, character: 0 },
  end: { line: 0, character: 0 },
};

const BUTTON_TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button() {
  return <div className={cx('indicaror')}>hi</div>;
}
`;

const BUTTON_SCSS = `
.indicator { color: red; }
`;

function codeActionParams(
  documentUri: string,
  diagnostics: Diagnostic[],
  range: Range = diagnostics[0]?.range ?? ZERO_RANGE,
): CodeActionParams {
  return {
    textDocument: { uri: documentUri },
    range,
    context: {
      diagnostics,
      triggerKind: 1,
    },
  };
}

describe("code-action protocol", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("advertises codeActionProvider with quickfix kind", async () => {
    client = createInProcessServer();
    const result = await client.initialize();
    expect(result.capabilities.codeActionProvider).toEqual({
      codeActionKinds: ["quickfix", "refactor.extract", "refactor.inline"],
      resolveProvider: false,
    });
  });

  it("returns replace and create actions for a missing-selector diagnostic", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: BUTTON_TSX,
      },
    });
    // Wait for diagnostics → the typo 'indicaror' should produce
    // one Warning with `data.suggestion: "indicator"`.
    const diagnostics = await client.waitForDiagnostics(BUTTON_TSX_URI);
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.data).toMatchObject({
      suggestion: "indicator",
      createSelector: {
        uri: BUTTON_SCSS_URI,
      },
    });

    // Now ask for code actions at the diagnostic range.
    const actions = await client.codeAction(codeActionParams(BUTTON_TSX_URI, diagnostics));
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(2);
    const action = actions![0] as {
      title: string;
      kind: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(action.title).toBe("Replace with 'indicator'");
    expect(action.kind).toBe(CodeActionKind.QuickFix);
    const edits = action.edit?.changes?.[BUTTON_TSX_URI];
    expect(edits).toHaveLength(1);
    expect(edits![0]!.newText).toBe("indicator");

    const createAction = actions![1] as {
      title: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(createAction.title).toBe("Add '.indicaror' to Button.module.scss");
    const styleEdits = createAction.edit?.changes?.[BUTTON_SCSS_URI];
    expect(styleEdits).toHaveLength(1);
    expect(styleEdits![0]!.newText).toBe("\n\n.indicaror {\n}\n");
  });

  it("returns null when the context contains only suggestion-less diagnostics", async () => {
    client = createInProcessServer();
    await client.initialize();
    const actions = await client.codeAction(
      codeActionParams(
        "file:///never/opened.tsx",
        [
          {
            range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } },
            severity: DiagnosticSeverity.Warning,
            source: "css-module-explainer",
            message: "whatever",
          },
        ],
        ZERO_RANGE,
      ),
    );
    expect(actions).toBeNull();
  });

  it("returns a create-file action for a missing-module diagnostic", async () => {
    const MISSING_MODULE_TSX = `import styles from './Missing.module.scss';
export const Button = () => <div className={styles.root}>hi</div>;
`;
    client = createInProcessServer({
      fileExists: () => false,
      readStyleFile: () => null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: MISSING_MODULE_TSX,
      },
    });
    let diagnostics = await client.waitForDiagnostics(BUTTON_TSX_URI);
    if (diagnostics.length === 0) {
      diagnostics = await client.waitForDiagnostics(BUTTON_TSX_URI);
    }
    expect(diagnostics).toHaveLength(1);

    const actions = await client.codeAction(codeActionParams(BUTTON_TSX_URI, diagnostics));
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      kind: string;
      edit?: { documentChanges?: Array<{ kind: string; uri: string }> };
    };
    expect(action.title).toBe("Create Missing.module.scss");
    expect(action.kind).toBe(CodeActionKind.QuickFix);
    expect(action.edit?.documentChanges).toEqual([
      {
        kind: "create",
        uri: "file:///fake/workspace/src/Missing.module.scss",
        options: { overwrite: false, ignoreIfExists: true },
      },
    ]);
  });

  it("returns sibling module creation actions without diagnostics for an unstyled TSX file", async () => {
    client = createInProcessServer({
      fileExists: () => false,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    const actions = await client.codeAction(codeActionParams(BUTTON_TSX_URI, [], ZERO_RANGE));
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(3);
    expect(actions?.map((action) => ("title" in action ? action.title : null))).toEqual([
      "Create Button.module.scss",
      "Create Button.module.css",
      "Create Button.module.less",
    ]);
  });

  it("returns a create-file action for an unresolved composes module diagnostic", async () => {
    const COMPOSING_SCSS = `
.button {
  composes: base from './Base.module.scss';
  color: red;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) =>
        filePath.endsWith("Button.module.scss") ? COMPOSING_SCSS : null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: COMPOSING_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(BUTTON_SCSS_URI);
    const missingModule = diagnostics.find((diagnostic) =>
      diagnostic.message.includes("Cannot resolve composed CSS Module './Base.module.scss'."),
    );
    expect(missingModule).toBeDefined();

    const actions = await client.codeAction(
      codeActionParams(BUTTON_SCSS_URI, diagnostics, missingModule!.range),
    );
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      kind: string;
      edit?: { documentChanges?: Array<{ kind: string; uri: string }> };
    };
    expect(action.title).toBe("Create Base.module.scss");
    expect(action.kind).toBe(CodeActionKind.QuickFix);
    expect(action.edit?.documentChanges).toEqual([
      {
        kind: "create",
        uri: "file:///fake/workspace/src/Base.module.scss",
        options: { overwrite: false, ignoreIfExists: true },
      },
    ]);
  });

  it("returns an add-selector action for a missing selector in a composed module", async () => {
    const COMPOSING_SCSS = `
.button {
  composes: base from './Base.module.scss';
  color: red;
}
`;
    const BASE_SCSS = `
.other {
  color: blue;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) => {
        if (filePath.endsWith("Button.module.scss")) return COMPOSING_SCSS;
        if (filePath.endsWith("Base.module.scss")) return BASE_SCSS;
        return null;
      },
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: COMPOSING_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(BUTTON_SCSS_URI);
    const missingSelector = diagnostics.find((diagnostic) =>
      diagnostic.message.includes(
        "Selector '.base' not found in composed module './Base.module.scss'.",
      ),
    );
    expect(missingSelector).toBeDefined();

    const actions = await client.codeAction(
      codeActionParams(BUTTON_SCSS_URI, diagnostics, missingSelector!.range),
    );
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(action.title).toBe("Add '.base' to Base.module.scss");
    expect(action.edit?.changes?.["file:///fake/workspace/src/Base.module.scss"]).toEqual([
      {
        range: {
          start: { line: 3, character: 1 },
          end: { line: 3, character: 1 },
        },
        newText: "\n\n.base {\n}\n",
      },
    ]);
  });

  it("returns an add-value action for a missing imported @value binding", async () => {
    const VALUE_SCSS = `
@value primary, secondary as accent from './tokens.module.scss';

.button {
  color: accent;
}
`;
    const TOKENS_SCSS = `@value primary: #ff3355;`;
    client = createInProcessServer({
      readStyleFile: (filePath) => {
        if (filePath.endsWith("Button.module.scss")) return VALUE_SCSS;
        if (filePath.endsWith("tokens.module.scss")) return TOKENS_SCSS;
        return null;
      },
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: VALUE_SCSS,
      },
    });
    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/tokens.module.scss",
        languageId: "scss",
        version: 1,
        text: TOKENS_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(BUTTON_SCSS_URI);
    const missingValue = diagnostics.find((diagnostic) =>
      diagnostic.message.includes(
        "@value 'secondary' not found in './tokens.module.scss' for local binding 'accent'.",
      ),
    );
    expect(missingValue).toBeDefined();

    const actions = await client.codeAction(
      codeActionParams(BUTTON_SCSS_URI, diagnostics, missingValue!.range),
    );
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(action.title).toBe("Add '@value secondary' to tokens.module.scss");
    expect(action.edit?.changes?.["file:///fake/workspace/src/tokens.module.scss"]).toEqual([
      {
        range: {
          start: { line: 0, character: 24 },
          end: { line: 0, character: 24 },
        },
        newText: "\n@value secondary: ;",
      },
    ]);
  });

  it("returns an add-keyframes action for a missing animation target", async () => {
    const KEYFRAMES_SCSS = `
.button {
  animation: fade 200ms ease-in;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) =>
        filePath.endsWith("Button.module.scss") ? KEYFRAMES_SCSS : null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: KEYFRAMES_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(BUTTON_SCSS_URI);
    const missingKeyframes = diagnostics.find((diagnostic) =>
      diagnostic.message.includes("@keyframes 'fade' not found in this file."),
    );
    expect(missingKeyframes).toBeDefined();

    const actions = await client.codeAction(
      codeActionParams(BUTTON_SCSS_URI, diagnostics, missingKeyframes!.range),
    );
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(action.title).toBe("Add '@keyframes fade' to Button.module.scss");
    expect(action.edit?.changes?.[BUTTON_SCSS_URI]).toEqual([
      {
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 0 },
        },
        newText: "@keyframes fade {\n}\n\n",
      },
    ]);
  });

  it("returns an add-Sass-symbol action for an unresolved Sass variable", async () => {
    const SASS_SCSS = `
.button {
  color: $missing;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) => (filePath.endsWith("Button.module.scss") ? SASS_SCSS : null),
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: SASS_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(BUTTON_SCSS_URI);
    const missingVariable = diagnostics.find((diagnostic) =>
      diagnostic.message.includes("Sass variable '$missing' not found"),
    );
    expect(missingVariable).toBeDefined();

    const actions = await client.codeAction(
      codeActionParams(BUTTON_SCSS_URI, diagnostics, missingVariable!.range),
    );
    expect(actions).not.toBeNull();
    expect(actions).toHaveLength(1);
    const action = actions![0] as {
      title: string;
      edit?: { changes?: Record<string, Array<{ newText: string }>> };
    };
    expect(action.title).toBe("Add '$missing' to Button.module.scss");
    expect(action.edit?.changes?.[BUTTON_SCSS_URI]).toEqual([
      {
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 0 },
        },
        newText: "$missing: ;\n\n",
      },
    ]);
  });
});
