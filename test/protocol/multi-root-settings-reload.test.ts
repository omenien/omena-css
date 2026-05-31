import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";

const ROOT_A_URI = "file:///fake/workspace-a";
const ROOT_B_URI = "file:///fake/workspace-b";
const itNonWindows = process.platform === "win32" ? it.skip : it;

const BUTTON_SCSS = `.btn-primary { color: red; }
.orphan { color: blue; }
`;

const APP_TSX = `import styles from './Button.module.scss';
export function App() {
  return <div className={styles.btnPrimary}>hi</div>;
}
`;

describe("multi-root settings reload", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  itNonWindows(
    "reschedules only the affected workspace root on classnameTransform change",
    async () => {
      client = createInProcessServer({
        readStyleFile: () => BUTTON_SCSS,
        typeResolver: new FakeTypeResolver(),
      });
      client.setScopedConfiguration("omena", ROOT_A_URI, {
        scss: { classnameTransform: "asIs" },
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

      const tsxAUri = `${ROOT_A_URI}/src/App.tsx`;
      const scssAUri = `${ROOT_A_URI}/src/Button.module.scss`;
      const tsxBUri = `${ROOT_B_URI}/src/App.tsx`;
      const scssBUri = `${ROOT_B_URI}/src/Button.module.scss`;

      client.didOpen({
        textDocument: { uri: tsxAUri, languageId: "typescriptreact", version: 1, text: APP_TSX },
      });
      client.didOpen({
        textDocument: { uri: tsxBUri, languageId: "typescriptreact", version: 1, text: APP_TSX },
      });
      await client.waitForDiagnostics(tsxAUri);
      await client.waitForDiagnostics(tsxBUri);

      client.didOpen({
        textDocument: { uri: scssAUri, languageId: "scss", version: 1, text: BUTTON_SCSS },
      });
      client.didOpen({
        textDocument: { uri: scssBUri, languageId: "scss", version: 1, text: BUTTON_SCSS },
      });

      const initialA = await client.waitForDiagnostics(scssAUri);
      const initialB = await client.waitForDiagnostics(scssBUri);
      expect(initialA.find((d) => d.message.includes("'.btn-primary'"))).toBeDefined();
      expect(initialB.find((d) => d.message.includes("'.btn-primary'"))).toBeDefined();

      client.setScopedConfiguration("omena", ROOT_A_URI, {
        scss: { classnameTransform: "camelCase" },
      });
      client.didChangeConfiguration();

      const updatedA = await client.waitForDiagnostics(scssAUri);
      expect(updatedA.find((d) => d.message.includes("'.orphan'"))).toBeDefined();
      expect(updatedA.find((d) => d.message.includes("'.btn-primary'"))).toBeUndefined();

      await expect(client.waitForDiagnostics(scssBUri, 200)).rejects.toThrow(/timed out/u);
    },
  );
});
