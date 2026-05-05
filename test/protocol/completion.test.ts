import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import {
  textDocumentPositionParams,
  workspace,
  type CmeWorkspace,
} from "../../packages/vitest-cme/src";

const BUTTON_TSX_URI = "file:///fake/workspace/src/Button.tsx";
const BUTTON_SCSS_URI = "file:///fake/workspace/src/Button.module.scss";

const BUTTON_TSX_WORKSPACE = workspace({
  [BUTTON_TSX_URI]: `impor/*at:outside*/t classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button() {
  return <div className={cx('/*|*/
}
`,
});

const BUTTON_TSX = BUTTON_TSX_WORKSPACE.file(BUTTON_TSX_URI).content;

const BUTTON_SCSS = `
.indicator { color: red; }
.active { color: blue; }
.disabled { color: gray; }
`;

const SASS_SYMBOL_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
.button {
  color: $/*at:variable*/;
  @include ra/*at:mixin*/;
  border-color: to/*at:function*/;
}
`,
});

const SASS_SYMBOL_SCSS = SASS_SYMBOL_WORKSPACE.file(BUTTON_SCSS_URI).content;

const INVALID_SASS_SYMBOL_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
.button {
  color: $/*at:variable*/
  @include ra/*at:mixin*/
  border-color: to/*at:function*/
}
`,
});

const INVALID_SASS_SYMBOL_SCSS = INVALID_SASS_SYMBOL_WORKSPACE.file(BUTTON_SCSS_URI).content;

const WILDCARD_SASS_SYMBOL_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `@use "./tokens.module" as *;

.button {
  color: $/*at:variable*/;
  @include ra/*at:mixin*/;
  border-color: to/*at:function*/;
}
`,
});

const WILDCARD_SASS_SYMBOL_SCSS = WILDCARD_SASS_SYMBOL_WORKSPACE.file(BUTTON_SCSS_URI).content;

const WILDCARD_TOKENS_SCSS = `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`;

function completionParams(
  source: CmeWorkspace,
  filePath: string,
  markerName?: string,
): {
  readonly textDocument: { readonly uri: string };
  readonly position: { readonly line: number; readonly character: number };
} {
  return textDocumentPositionParams({
    workspace: source,
    documentUri: filePath,
    filePath,
    ...(markerName === undefined ? {} : { markerName }),
  });
}

describe("completion protocol / clsx", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  const CLSX_TSX_WORKSPACE = workspace({
    [BUTTON_TSX_URI]: `import clsx from 'clsx';
import styles from './Button.module.scss';
export function Button() {
  return <div className={clsx(styles./*|*/
}
`,
  });

  const CLSX_TSX = CLSX_TSX_WORKSPACE.file(BUTTON_TSX_URI).content;

  const CLSX_SCSS = `
.indicator { color: red; }
.active { color: blue; }
.disabled { color: gray; }
`;

  it("returns the SCSS class list inside clsx(styles.|)", async () => {
    client = createInProcessServer({
      readStyleFile: () => CLSX_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: CLSX_TSX,
      },
    });
    // Cursor after "styles." on line 3 (dot is at 36, cursor after dot is 37)
    const result = await client.completion(completionParams(CLSX_TSX_WORKSPACE, BUTTON_TSX_URI));
    expect(result).not.toBeNull();
    const items = Array.isArray(result) ? result : result!.items;
    expect(items.length).toBe(3);
    const labels = items.map((i) => i.label).toSorted();
    expect(labels).toEqual(["active", "disabled", "indicator"]);
  });

  it("returns direct style completions when outside clsx() call", async () => {
    const outsideWorkspace = workspace({
      [BUTTON_TSX_URI]: `import clsx from 'clsx';
import styles from './Button.module.scss';
const x = styles./*|*/
`,
    });
    const OUTSIDE_TSX = outsideWorkspace.file(BUTTON_TSX_URI).content;
    client = createInProcessServer({
      readStyleFile: () => CLSX_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: OUTSIDE_TSX,
      },
    });
    const result = await client.completion(completionParams(outsideWorkspace, BUTTON_TSX_URI));
    expect(result).not.toBeNull();
    const items = Array.isArray(result) ? result : result!.items;
    expect(items.length).toBe(3);
    const labels = items.map((i) => i.label).toSorted();
    expect(labels).toEqual(["active", "disabled", "indicator"]);
  });
});

describe("completion protocol", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("returns the SCSS class list inside an open cx('", async () => {
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
    // Cursor just after cx(' on line 4
    const result = await client.completion(completionParams(BUTTON_TSX_WORKSPACE, BUTTON_TSX_URI));
    expect(result).not.toBeNull();
    const items = Array.isArray(result) ? result : result!.items;
    expect(items.length).toBe(3);
    const labels = items.map((i) => i.label).toSorted();
    expect(labels).toEqual(["active", "disabled", "indicator"]);
  });

  it("returns null when outside any cx call", async () => {
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
    const result = await client.completion(
      completionParams(BUTTON_TSX_WORKSPACE, BUTTON_TSX_URI, "outside"),
    );
    expect(result).toBeNull();
  });

  it("returns Sass symbol completions inside SCSS files", async () => {
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

    const variableResult = await client.completion(
      completionParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "variable"),
    );
    const variableItems = Array.isArray(variableResult) ? variableResult : variableResult!.items;
    expect(variableItems.map((item) => item.label)).toEqual(["$gap"]);
    expect(variableItems[0]!.textEdit).toMatchObject({
      newText: "$gap",
      range: {
        start: { line: 4, character: 9 },
        end: { line: 4, character: 10 },
      },
    });

    const mixinResult = await client.completion(
      completionParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "mixin"),
    );
    const mixinItems = Array.isArray(mixinResult) ? mixinResult : mixinResult!.items;
    expect(mixinItems.map((item) => item.label)).toEqual(["raised"]);

    const functionResult = await client.completion(
      completionParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "function"),
    );
    const functionItems = Array.isArray(functionResult) ? functionResult : functionResult!.items;
    expect(functionItems.map((item) => item.label)).toEqual(["tone"]);
  });

  it("returns wildcard Sass module completions inside SCSS files", async () => {
    client = createInProcessServer({
      readStyleFile: (filePath) => {
        if (filePath.endsWith("Button.module.scss")) return WILDCARD_SASS_SYMBOL_SCSS;
        if (filePath.endsWith("tokens.module.scss")) return WILDCARD_TOKENS_SCSS;
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
        text: WILDCARD_SASS_SYMBOL_SCSS,
      },
    });

    const variableResult = await client.completion(
      completionParams(WILDCARD_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "variable"),
    );
    const variableItems = Array.isArray(variableResult) ? variableResult : variableResult!.items;
    expect(variableItems.map((item) => item.label)).toEqual(["$gap"]);

    const mixinResult = await client.completion(
      completionParams(WILDCARD_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "mixin"),
    );
    const mixinItems = Array.isArray(mixinResult) ? mixinResult : mixinResult!.items;
    expect(mixinItems.map((item) => item.label)).toEqual(["raised"]);

    const functionResult = await client.completion(
      completionParams(WILDCARD_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "function"),
    );
    const functionItems = Array.isArray(functionResult) ? functionResult : functionResult!.items;
    expect(functionItems.map((item) => item.label)).toEqual(["tone"]);
  });

  it("returns Sass symbol completions while the SCSS buffer is mid-edit invalid", async () => {
    client = createInProcessServer({
      readStyleFile: () => INVALID_SASS_SYMBOL_SCSS,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: INVALID_SASS_SYMBOL_SCSS,
      },
    });

    const variableResult = await client.completion(
      completionParams(INVALID_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "variable"),
    );
    const variableItems = Array.isArray(variableResult) ? variableResult : variableResult!.items;
    expect(variableItems.map((item) => item.label)).toEqual(["$gap"]);

    const mixinResult = await client.completion(
      completionParams(INVALID_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "mixin"),
    );
    const mixinItems = Array.isArray(mixinResult) ? mixinResult : mixinResult!.items;
    expect(mixinItems.map((item) => item.label)).toEqual(["raised"]);

    const functionResult = await client.completion(
      completionParams(INVALID_SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI, "function"),
    );
    const functionItems = Array.isArray(functionResult) ? functionResult : functionResult!.items;
    expect(functionItems.map((item) => item.label)).toEqual(["tone"]);
  });
});
