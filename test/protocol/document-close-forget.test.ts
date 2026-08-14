import { afterEach, describe, expect, it } from "vitest";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import type { FileTask } from "../../server/engine-core-ts/src/core/indexing/indexer-worker";

const BUTTON_SCSS = `.btn { color: red; }
`;

const APP_TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('btn')}>hi</div>;
}
`;

function emptyWorkspaceFileSupplier(): AsyncIterable<FileTask> {
  return {
    [Symbol.asyncIterator](): AsyncIterator<FileTask> {
      return {
        next: () => Promise.resolve({ done: true, value: undefined as never }),
      };
    },
  };
}

describe("document close forgets semantic reference contributions", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("closing a TSX file drops its semantic reference sites so SCSS unused-check re-flags the selector", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptyWorkspaceFileSupplier,
    });
    await client.initialize({
      capabilities: {
        workspace: {
          codeLens: { refreshSupport: true },
          didChangeWatchedFiles: { dynamicRegistration: true },
          workspaceFolders: true,
        },
      },
    });
    client.initialized();
    await client.waitForDynamicWatcherRegistration();

    const SCSS_URI = "file:///fake/workspace/src/Button.module.scss";
    const TSX_URI = "file:///fake/workspace/src/App.tsx";

    client.didOpen({
      textDocument: { uri: TSX_URI, languageId: "typescriptreact", version: 1, text: APP_TSX },
    });
    await client.waitForDiagnostics(TSX_URI);

    client.didOpen({
      textDocument: { uri: SCSS_URI, languageId: "scss", version: 1, text: BUTTON_SCSS },
    });
    const scssWhileTsxOpen = await client.waitForDiagnostics(SCSS_URI);
    // TSX is open and references `.btn` — the unused check should
    // NOT flag it.
    expect(scssWhileTsxOpen.find((d) => d.message.includes("'.btn'"))).toBeUndefined();

    // Close the TSX file. The close handler must drop the workspace
    // semantic contribution the TSX buffer owned — otherwise
    // a subsequent unused-selector check on the SCSS file would
    // keep seeing the closed cx() site and treat `.btn` as referenced.
    client.didClose({ textDocument: { uri: TSX_URI } });

    // Touch the SCSS buffer to force a reschedule (onDidChangeContent
    // fires with the same text, triggering scheduleScss).
    client.didChange({
      textDocument: { uri: SCSS_URI, version: 2 },
      contentChanges: [{ text: BUTTON_SCSS }],
    });
    const scssAfterClose = await client.waitForDiagnostics(SCSS_URI);
    expect(scssAfterClose.find((d) => d.message.includes("'.btn'"))).toBeDefined();
  });
});
