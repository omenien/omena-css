import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import { textDocumentPositionParams, workspace } from "../../packages/vitest-cme/src";
const ROOT_A_URI = "file:///fake/workspace-a";
const ROOT_B_URI = "file:///fake/workspace-b";
const APP_A_URI = `${ROOT_A_URI}/src/App.tsx`;
const APP_B_URI = `${ROOT_B_URI}/src/App.tsx`;
const itNonWindows = process.platform === "win32" ? it.skip : it;

const APP_WORKSPACE = workspace({
  [APP_A_URI]: `import classNames from 'classnames/bind';
import styles from '@styles/Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('/*|*/button')}>ok</div>;
}
`,
  [APP_B_URI]: `import classNames from 'classnames/bind';
import styles from '@styles/Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('/*|*/button')}>ok</div>;
}
`,
});

const APP_TSX = APP_WORKSPACE.file(APP_A_URI).content;

function readStyleFile(path: string): string | null {
  if (path === "/fake/workspace-a/src/styles-a/Button.module.scss") {
    return ".button { color: red; }\n";
  }
  if (path === "/fake/workspace-b/src/styles-b/Button.module.scss") {
    return ".button { color: blue; }\n";
  }
  return null;
}

describe("multi-root pathAlias", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  itNonWindows("resolves native pathAlias per workspace folder", async () => {
    client = createInProcessServer({
      readStyleFile,
      typeResolver: new FakeTypeResolver(),
    });
    client.setScopedConfiguration("omena", ROOT_A_URI, {
      pathAlias: { "@styles": "src/styles-a" },
    });
    client.setScopedConfiguration("omena", ROOT_B_URI, {
      pathAlias: { "@styles": "src/styles-b" },
    });
    await client.initialize({
      rootUri: ROOT_A_URI,
      workspaceFolders: [
        { uri: ROOT_A_URI, name: "a" },
        { uri: ROOT_B_URI, name: "b" },
      ],
    });
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: APP_A_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });
    client.didOpen({
      textDocument: {
        uri: APP_B_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });

    expect(await client.waitForDiagnostics(APP_A_URI)).toEqual([]);
    expect(await client.waitForDiagnostics(APP_B_URI)).toEqual([]);
    const definitionA = await client.definition(
      textDocumentPositionParams({
        workspace: APP_WORKSPACE,
        documentUri: APP_A_URI,
        filePath: APP_A_URI,
      }),
    );
    expect(definitionA).not.toBeNull();
    expect((definitionA as Array<{ targetUri: string }>)[0]!.targetUri).toBe(
      `${ROOT_A_URI}/src/styles-a/Button.module.scss`,
    );

    const definitionB = await client.definition(
      textDocumentPositionParams({
        workspace: APP_WORKSPACE,
        documentUri: APP_B_URI,
        filePath: APP_B_URI,
      }),
    );
    expect(definitionB).not.toBeNull();
    expect((definitionB as Array<{ targetUri: string }>)[0]!.targetUri).toBe(
      `${ROOT_B_URI}/src/styles-b/Button.module.scss`,
    );
  });
});
