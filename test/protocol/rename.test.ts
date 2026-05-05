import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import {
  textDocumentPositionFixture,
  textDocumentRenameFixture,
  workspace,
  type CmeWorkspace,
} from "../../packages/vitest-cme/src";

const APP_URI = "file:///fake/workspace/src/App.tsx";
const BUTTON_SCSS_URI = "file:///fake/workspace/src/Button.module.scss";
const THEME_SCSS_URI = "file:///fake/workspace/src/theme.module.scss";
const OUT_OF_DOCUMENT_POSITION = { line: 99, character: 0 };

const BUTTON_SCSS_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `.in/*|*/dicator { color: red; }
.active { color: blue; }
`,
});

const BUTTON_SCSS = BUTTON_SCSS_WORKSPACE.file(BUTTON_SCSS_URI).content;

const APP_WORKSPACE = workspace({
  [APP_URI]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('ind/*|*/icator')}>hi</div>;
}
`,
});

const APP_TSX = APP_WORKSPACE.file(APP_URI).content;

const SASS_SYMBOL_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
.button {
  color: $/*at:variable*/gap;
  margin: $gap;
  @include ra/*at:mixin*/ised();
  border-color: t/*at:function*/one($gap);
}
`,
});

const SASS_SYMBOL_SCSS = SASS_SYMBOL_WORKSPACE.file(BUTTON_SCSS_URI).content;

const CUSTOM_PROPERTY_WORKSPACE = workspace({
  [THEME_SCSS_URI]: `:root {
  --brand: red;
}
`,
  [BUTTON_SCSS_URI]: `@use "./theme.module.scss";
.button {
  color: var(--/*at:customProperty*/brand);
  border-color: var(--brand);
}
`,
});

const THEME_SCSS = CUSTOM_PROPERTY_WORKSPACE.file(THEME_SCSS_URI).content;
const CUSTOM_PROPERTY_SCSS = CUSTOM_PROPERTY_WORKSPACE.file(BUTTON_SCSS_URI).content;

function fixturePositionParams(source: CmeWorkspace, filePath: string, markerName?: string) {
  return textDocumentPositionFixture({
    workspace: source,
    documentUri: filePath,
    filePath,
    markerName,
  });
}

function fixtureRenameParams(
  source: CmeWorkspace,
  filePath: string,
  newName: string,
  markerName?: string,
) {
  return textDocumentRenameFixture({
    workspace: source,
    documentUri: filePath,
    filePath,
    markerName,
    newName,
  });
}

describe("rename protocol", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("rename from SCSS selector updates all cx() string literals", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    // Open TSX file to populate semantic reference sites.
    client.didOpen({
      textDocument: {
        uri: APP_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });
    await client.waitForDiagnostics(APP_URI);

    // Prepare rename on .indicator in SCSS.
    const prep = await client.prepareRename(
      fixturePositionParams(BUTTON_SCSS_WORKSPACE, BUTTON_SCSS_URI),
    );
    expect(prep).not.toBeNull();
    expect(prep!.placeholder).toBe("indicator");

    // Execute rename.
    const edit = await client.rename(
      fixtureRenameParams(BUTTON_SCSS_WORKSPACE, BUTTON_SCSS_URI, "status"),
    );
    expect(edit).not.toBeNull();
    const changes = edit!.changes!;
    // SCSS edit
    expect(changes[BUTTON_SCSS_URI]).toHaveLength(1);
    // TSX edit
    expect(changes[APP_URI]).toHaveLength(1);
    expect(changes[APP_URI]![0]!.newText).toBe("status");
  });

  it("rename from cx('indicator') in TSX updates SCSS selector", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: APP_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });
    await client.waitForDiagnostics(APP_URI);

    // Prepare rename on 'indicator' inside cx('indicator').
    // Line 4: return <div className={cx('indicator')}>hi</div>;
    // 'indicator' starts at character 28 (after cx(')
    const prep = await client.prepareRename(fixturePositionParams(APP_WORKSPACE, APP_URI));
    expect(prep).not.toBeNull();
    expect(prep!.placeholder).toBe("indicator");

    const edit = await client.rename(fixtureRenameParams(APP_WORKSPACE, APP_URI, "status"));
    expect(edit).not.toBeNull();
    const changes = edit!.changes!;
    // SCSS edit
    expect(changes[BUTTON_SCSS_URI]).toHaveLength(1);
    expect(changes[BUTTON_SCSS_URI]![0]!.newText).toBe("status");
    // TSX edit
    expect(changes[APP_URI]).toHaveLength(1);
  });

  it("prepareRename returns null for a non-renameable position", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    const prep = await client.prepareRename({
      textDocument: { uri: BUTTON_SCSS_URI },
      position: OUT_OF_DOCUMENT_POSITION,
    });
    expect(prep).toBeNull();
  });

  it("rename from a Sass variable reference updates same-file declaration and references", async () => {
    client = createInProcessServer({
      readStyleFile: () => SASS_SYMBOL_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: SASS_SYMBOL_SCSS,
      },
    });

    const prep = await client.prepareRename(
      fixturePositionParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "variable"),
    );
    expect(prep).not.toBeNull();
    expect(prep!.placeholder).toBe("$gap");

    const edit = await client.rename(
      fixtureRenameParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "space", "variable"),
    );
    expect(edit).not.toBeNull();
    const scssEdits = edit!.changes![BUTTON_SCSS_URI]!;
    expect(scssEdits).toHaveLength(4);
    expect(scssEdits.map((textEdit) => textEdit.newText)).toEqual([
      "$space",
      "$space",
      "$space",
      "$space",
    ]);
    expect(scssEdits.map((textEdit) => textEdit.range.start.line)).toEqual([0, 4, 5, 7]);
  });

  it("rename from a Sass mixin include updates same-file declaration and include", async () => {
    client = createInProcessServer({
      readStyleFile: () => SASS_SYMBOL_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: SASS_SYMBOL_SCSS,
      },
    });

    const edit = await client.rename(
      fixtureRenameParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "elevated", "mixin"),
    );
    expect(edit).not.toBeNull();
    const scssEdits = edit!.changes![BUTTON_SCSS_URI]!;
    expect(scssEdits).toHaveLength(2);
    expect(scssEdits.map((textEdit) => textEdit.newText)).toEqual(["elevated", "elevated"]);
    expect(scssEdits.map((textEdit) => textEdit.range.start.line)).toEqual([1, 6]);
  });

  it("rename from a Sass function call updates same-file declaration and call", async () => {
    client = createInProcessServer({
      readStyleFile: () => SASS_SYMBOL_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: SASS_SYMBOL_SCSS,
      },
    });

    const edit = await client.rename(
      fixtureRenameParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "theme-tone", "function"),
    );
    expect(edit).not.toBeNull();
    const scssEdits = edit!.changes![BUTTON_SCSS_URI]!;
    expect(scssEdits).toHaveLength(2);
    expect(scssEdits.map((textEdit) => textEdit.newText)).toEqual(["theme-tone", "theme-tone"]);
    expect(scssEdits.map((textEdit) => textEdit.range.start.line)).toEqual([2, 7]);
  });

  it("rename from a CSS custom property reference updates workspace declarations and refs", async () => {
    client = createInProcessServer({
      readStyleFile: (filePath) =>
        filePath.endsWith("theme.module.scss")
          ? THEME_SCSS
          : filePath.endsWith("Button.module.scss")
            ? CUSTOM_PROPERTY_SCSS
            : null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: THEME_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: THEME_SCSS,
      },
    });
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: CUSTOM_PROPERTY_SCSS,
      },
    });

    const prep = await client.prepareRename(
      fixturePositionParams(CUSTOM_PROPERTY_WORKSPACE, BUTTON_SCSS_URI, "customProperty"),
    );
    expect(prep).not.toBeNull();
    expect(prep!.placeholder).toBe("--brand");

    const edit = await client.rename(
      fixtureRenameParams(
        CUSTOM_PROPERTY_WORKSPACE,
        BUTTON_SCSS_URI,
        "--surface",
        "customProperty",
      ),
    );
    expect(edit).not.toBeNull();
    const changes = edit!.changes!;
    expect(changes[THEME_SCSS_URI]).toEqual([
      {
        range: {
          start: { line: 1, character: 2 },
          end: { line: 1, character: 9 },
        },
        newText: "--surface",
      },
    ]);
    expect(changes[BUTTON_SCSS_URI]?.map((textEdit) => textEdit.newText)).toEqual([
      "--surface",
      "--surface",
    ]);
  });

  it("prepareRename rejects dynamic source expressions with a message", async () => {
    const dynamicWorkspace = workspace({
      [APP_URI]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = 'indicator';
export function App() {
  return <div className={cx(s/*|*/ize)}>hi</div>;
}
`,
    });
    const dynamicTsx = dynamicWorkspace.file(APP_URI).content;
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: APP_URI,
        languageId: "typescriptreact",
        version: 1,
        text: dynamicTsx,
      },
    });
    await client.waitForDiagnostics(APP_URI);

    await expect(
      client.prepareRename({
        ...fixturePositionParams(dynamicWorkspace, APP_URI),
      }),
    ).rejects.toMatchObject({
      message: "Dynamic class expressions cannot be renamed safely.",
    });
  });

  // End-to-end BEM suffix rename across SCSS + TSX.
  it("rename &-nested BEM suffix rewrites only the suffix in SCSS and the full class in TSX", async () => {
    const bemScssWorkspace = workspace({
      [BUTTON_SCSS_URI]: `.button {
  padding: 8px;
  /*|*/&--primary {
    color: white;
  }
}
`,
    });
    const BEM_SCSS = bemScssWorkspace.file(BUTTON_SCSS_URI).content;
    const BEM_TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('button--primary')}>hi</div>;
}
`;
    client = createInProcessServer({
      readStyleFile: () => BEM_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: APP_URI,
        languageId: "typescriptreact",
        version: 1,
        text: BEM_TSX,
      },
    });
    await client.waitForDiagnostics(APP_URI);

    // Cursor on the `&` of `&--primary` at line 2, column 2.
    const cursorParams = fixturePositionParams(bemScssWorkspace, BUTTON_SCSS_URI);

    // prepareRename: range covers exactly `&--primary` (10 chars),
    // placeholder is the resolved class name.
    const prep = await client.prepareRename(cursorParams);
    expect(prep).not.toBeNull();
    expect(prep!.placeholder).toBe("button--primary");
    expect(prep!.range.start).toEqual({ line: 2, character: 2 });
    expect(prep!.range.end).toEqual({ line: 2, character: 12 });

    // rename: SCSS edit is only `--primary → --tiny` (9 chars).
    // TSX edit is the full `button--primary → button--tiny`.
    const edit = await client.rename(
      fixtureRenameParams(bemScssWorkspace, BUTTON_SCSS_URI, "button--tiny"),
    );
    expect(edit).not.toBeNull();
    const changes = edit!.changes!;

    const scssEdits = changes[BUTTON_SCSS_URI]!;
    expect(scssEdits).toHaveLength(1);
    expect(scssEdits[0]!.newText).toBe("--tiny");
    expect(scssEdits[0]!.range.start).toEqual({ line: 2, character: 3 });
    expect(scssEdits[0]!.range.end).toEqual({ line: 2, character: 12 });

    const tsxEdits = changes[APP_URI]!;
    expect(tsxEdits).toHaveLength(1);
    expect(tsxEdits[0]!.newText).toBe("button--tiny");
  });
});
