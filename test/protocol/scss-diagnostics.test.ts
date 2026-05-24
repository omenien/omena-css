import { afterEach, describe, expect, it } from "vitest";
import { DiagnosticTag } from "vscode-languageserver-protocol/node";
import {
  createInProcessServer,
  emptySupplier,
  type LspTestClient,
} from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";

const BUTTON_SCSS = `
.indicator { color: red; }
.active { color: blue; }
.unused { opacity: 0.5; }
`;

// A TSX file that references "indicator" and "active" but not "unused".
const APP_TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('indicator', 'active')}>hi</div>;
}
`;

describe("SCSS unused selector diagnostics protocol", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("publishes Unnecessary hint for selectors with zero references after indexing completes", async () => {
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    // Open the TSX file first so the semantic reference index is populated.
    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/App.tsx",
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });
    // Consume TSX diagnostics (clean file, no unknown classes).
    await client.waitForDiagnostics("file:///fake/workspace/src/App.tsx");

    // Now open the SCSS file -- expect diagnostics for "unused".
    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: BUTTON_SCSS,
      },
    });
    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    // "unused" has zero references.
    const unusedDiag = diagnostics.find((d) => d.message.includes("'.unused'"));
    expect(unusedDiag).toBeDefined();
    expect(unusedDiag!.tags).toContain(DiagnosticTag.Unnecessary);
  });

  // SCSS buffer-first read for unused diagnostics. `classMapForPath`
  // must read from the open-document buffer first and fall back to
  // disk only when no buffer exists; otherwise classes added to an
  // unsaved buffer would be reported as unused until the user saves.
  it("unused diagnostics use buffered SCSS content, NOT disk content", async () => {
    // Disk content has only `.a`; buffer content has `.a` and `.b`.
    // TSX references `styles.b`. When `classMapForPath` reads disk
    // it returns `{a}` and TSX validation emits "Class '.b' not
    // found"; when it reads the buffer it returns `{a, b}` and the
    // diagnostic is suppressed. This test pins the buffer-first
    // read path by asserting the diagnostic never fires.
    const DISK_SCSS = ".a {}\n";
    const BUFFER_SCSS = ".a {}\n.b {}\n";
    const TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx('b')}>hi</div>;
}
`;
    client = createInProcessServer({
      readStyleFile: () => DISK_SCSS,
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    // Open SCSS FIRST so its buffer is in `documents` before
    // the TSX analysis runs classMapForPath on the SCSS path.
    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: BUFFER_SCSS,
      },
    });

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/App.tsx",
        languageId: "typescriptreact",
        version: 1,
        text: TSX,
      },
    });

    const tsxDiagnostics = await client.waitForDiagnostics("file:///fake/workspace/src/App.tsx");
    const notFoundDiag = tsxDiagnostics.find((d) => d.message.includes("Class '.b' not found"));
    expect(notFoundDiag).toBeUndefined();
  });

  it("keeps unused diagnostics when dynamic refs are resolved by local flow", async () => {
    const FLOW_TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function App(enabled: boolean) {
  const state = enabled ? 'indicator' : 'active';
  return <div className={cx(state)}>hi</div>;
}
`;
    client = createInProcessServer({
      readStyleFile: () => BUTTON_SCSS,
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/App.tsx",
        languageId: "typescriptreact",
        version: 1,
        text: FLOW_TSX,
      },
    });
    await client.waitForDiagnostics("file:///fake/workspace/src/App.tsx");

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: BUTTON_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    const unusedDiag = diagnostics.find((d) => d.message.includes("'.unused'"));
    expect(unusedDiag).toBeDefined();
  });

  it("reports an unresolved cross-file composes module", async () => {
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
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: COMPOSING_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) =>
        d.message.includes("Cannot resolve composed CSS Module './Base.module.scss'."),
      ),
    ).toBeDefined();
  });

  it("reports a missing selector in a composed module", async () => {
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
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: COMPOSING_SCSS,
      },
    });
    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Base.module.scss",
        languageId: "scss",
        version: 1,
        text: BASE_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) =>
        d.message.includes("Selector '.base' not found in composed module './Base.module.scss'."),
      ),
    ).toBeDefined();
  });

  it("reports an unresolved imported @value module", async () => {
    const VALUE_SCSS = `
@value primary from './tokens.module.scss';

.button {
  color: primary;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) => (filePath.endsWith("Button.module.scss") ? VALUE_SCSS : null),
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: VALUE_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) =>
        d.message.includes("Cannot resolve imported @value module './tokens.module.scss'."),
      ),
    ).toBeDefined();
  });

  it("reports a missing imported @value in an existing module", async () => {
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
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
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

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) =>
        d.message.includes(
          "@value 'secondary' not found in './tokens.module.scss' for local binding 'accent'.",
        ),
      ),
    ).toBeDefined();
  });

  it("reports missing @keyframes declarations for animation tokens", async () => {
    const KEYFRAMES_SCSS = `
.button {
  animation: fade 200ms ease-in;
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) =>
        filePath.endsWith("Button.module.scss") ? KEYFRAMES_SCSS : null,
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: KEYFRAMES_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) => d.message.includes("@keyframes 'fade' not found in this file.")),
    ).toBeDefined();
  });

  it("does not report Sass symbols resolved through wildcard module imports", async () => {
    const SASS_SCSS = `@use "./tokens.module" as *;

.button {
  color: $gap;
  @include raised();
  border-color: tone($gap);
}
`;
    const TOKENS_SCSS = `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`;
    client = createInProcessServer({
      readStyleFile: (filePath) => {
        if (filePath.endsWith("Button.module.scss")) return SASS_SCSS;
        if (filePath.endsWith("tokens.module.scss")) return TOKENS_SCSS;
        return null;
      },
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: SASS_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) => d.message.includes("Sass variable '$gap' not found in this file.")),
    ).toBeUndefined();
    expect(
      diagnostics.find((d) =>
        d.message.includes("Sass mixin '@mixin raised' not found in this file."),
      ),
    ).toBeUndefined();
    expect(
      diagnostics.find((d) => d.message.includes("Sass function 'tone()' not found in this file.")),
    ).toBeUndefined();
  });

  it("reports unresolved Sass symbols from the SCSS buffer", async () => {
    const SASS_SCSS = `
.button {
  color: $missing;
  @include absent();
}
`;
    client = createInProcessServer({
      readStyleFile: (filePath) => (filePath.endsWith("Button.module.scss") ? SASS_SCSS : null),
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: "file:///fake/workspace/src/Button.module.scss",
        languageId: "scss",
        version: 1,
        text: SASS_SCSS,
      },
    });

    const diagnostics = await client.waitForDiagnostics(
      "file:///fake/workspace/src/Button.module.scss",
    );
    expect(
      diagnostics.find((d) => d.message.includes("Sass variable '$missing' not found")),
    ).toBeDefined();
    expect(
      diagnostics.find((d) => d.message.includes("Sass mixin '@mixin absent' not found")),
    ).toBeDefined();
  });
});
