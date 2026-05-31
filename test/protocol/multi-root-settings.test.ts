import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import { textDocumentPositionParams, workspace } from "../../packages/vitest-cme/src";

const ROOT_A_URI = "file:///fake/workspace-a";
const ROOT_B_URI = "file:///fake/workspace-b";
const APP_A_URI = `${ROOT_A_URI}/src/Button.tsx`;
const APP_B_URI = `${ROOT_B_URI}/src/Button.tsx`;
const itNonWindows = process.platform === "win32" ? it.skip : it;

const CLSX_WORKSPACE = workspace({
  [APP_A_URI]: `import clsx from 'clsx';
import styles from './Button.module.scss';
export function Button() {
  return <div className={clsx(styles./*|*/fooBar)}>hi</div>;
}
`,
  [APP_B_URI]: `import clsx from 'clsx';
import styles from './Button.module.scss';
export function Button() {
  return <div className={clsx(styles./*|*/fooBar)}>hi</div>;
}
`,
});

const CLSX_TSX = CLSX_WORKSPACE.file(APP_A_URI).content;

const SCSS = `.foo-bar { color: red; }\n`;

describe("multi-root resource-scoped settings", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  itNonWindows("applies classnameTransform per workspace folder", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? SCSS : null),
      typeResolver: new FakeTypeResolver(),
    });
    client.setScopedConfiguration("omena", ROOT_A_URI, {
      scss: { classnameTransform: "camelCase" },
    });
    client.setScopedConfiguration("omena", ROOT_B_URI, {
      scss: { classnameTransform: "asIs" },
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
        text: CLSX_TSX,
      },
    });
    client.didOpen({
      textDocument: {
        uri: APP_B_URI,
        languageId: "typescriptreact",
        version: 1,
        text: CLSX_TSX,
      },
    });

    expect(await client.waitForDiagnostics(APP_A_URI)).toEqual([]);
    expect(await client.waitForDiagnostics(APP_B_URI)).toEqual([]);

    const definitionA = await client.definition(
      textDocumentPositionParams({
        workspace: CLSX_WORKSPACE,
        documentUri: APP_A_URI,
        filePath: APP_A_URI,
      }),
    );
    expect(definitionA).not.toBeNull();
    expect((definitionA as Array<{ targetUri: string }>)[0]!.targetUri).toBe(
      `${ROOT_A_URI}/src/Button.module.scss`,
    );

    const definitionB = await client.definition(
      textDocumentPositionParams({
        workspace: CLSX_WORKSPACE,
        documentUri: APP_B_URI,
        filePath: APP_B_URI,
      }),
    );
    expect(definitionB).toBeNull();
  });
});
