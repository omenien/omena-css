import { expect, test } from "../_fixtures/protocol";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";

const SCSS_URI = "file:///fake/workspace/src/Button.module.scss";

test("codeLens protocol shows reference counts for SCSS selectors used from TSX", async ({
  makeClient,
}) => {
  const tsxUri = "file:///fake/workspace/src/Button.tsx";
  const tsx = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button() {
  return <div className={cx('indicator')}>hi</div>;
}
`;
  const scss = `.indicator { color: red; }\n.active { color: blue; }\n`;

  const client = makeClient({
    readStyleFile: (path) => (path.endsWith("Button.module.scss") ? scss : null),
    typeResolver: new FakeTypeResolver(),
  });

  await client.initialize();
  client.initialized();
  client.didOpen({
    textDocument: {
      uri: tsxUri,
      languageId: "typescriptreact",
      version: 1,
      text: tsx,
    },
  });
  await client.waitForDiagnostics(tsxUri);

  const result = await client.codeLens({
    textDocument: { uri: SCSS_URI },
  });

  expect(result).not.toBeNull();
  expect(result).toHaveLength(1);
  expect(result![0]!.command?.title).toBe("1 reference");
  expect(result![0]!.command?.command).toBe("editor.action.showReferences");
});

test("codeLens protocol deduplicates alias views under camelCase classnameTransform", async ({
  makeClient,
}) => {
  const tsxUri = "file:///fake/workspace/src/StyleAccessButton.tsx";
  const tsx = `import styles from './Button.module.scss';
export function Button() {
  return <div className={styles.btnPrimary}>hi</div>;
}
`;
  const scss = `.btn-primary { color: red; }\n`;

  const client = makeClient({
    readStyleFile: (path) => (path.endsWith("Button.module.scss") ? scss : null),
    typeResolver: new FakeTypeResolver(),
  });

  client.setConfiguration("omena", {
    scss: { classnameTransform: "camelCase" },
  });

  await client.initialize();
  client.initialized();
  client.didOpen({
    textDocument: {
      uri: tsxUri,
      languageId: "typescriptreact",
      version: 1,
      text: tsx,
    },
  });
  await client.waitForDiagnostics(tsxUri);

  const result = await client.codeLens({
    textDocument: { uri: SCSS_URI },
  });

  expect(result).not.toBeNull();
  expect(result).toHaveLength(1);
  expect(result![0]!.command?.title).toBe("1 reference");
});
