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
const STATUS_TSX_URI = "file:///fake/workspace/src/Status.tsx";
const STATE_CHIP_TSX_URI = "file:///fake/workspace/src/StateChip.tsx";
const BUTTON_CHIP_TSX_URI = "file:///fake/workspace/src/ButtonChip.tsx";

const BUTTON_TSX_WORKSPACE = workspace({
  [BUTTON_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button() {
  return <div className={cx('indic/*|*/ator')}>hi</div>;
}
`,
});

const BUTTON_TSX = BUTTON_TSX_WORKSPACE.file(BUTTON_TSX_URI).content;

const BUTTON_SCSS_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `
.in/*|*/dicator {
  color: red;
  font-size: 14px;
}
`,
});

const BUTTON_SCSS = BUTTON_SCSS_WORKSPACE.file(BUTTON_SCSS_URI).content;

const KEYFRAMES_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `@keyframes fa/*|*/de {
  from { opacity: 0; }
  to { opacity: 1; }
}

.box {
  animation: fade 1s linear;
}

.pulse {
  animation-name: fade;
}
`,
});

const KEYFRAMES_SCSS = KEYFRAMES_WORKSPACE.file(BUTTON_SCSS_URI).content;

const VALUE_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `@value pr/*|*/imary: #ff3355;

.button {
  color: primary;
}
`,
});

const VALUE_SCSS = VALUE_WORKSPACE.file(BUTTON_SCSS_URI).content;

const IMPORTED_VALUE_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `@value primary from "./tokens.module.scss";

.button {
  color: p/*|*/rimary;
}
`,
});

const IMPORTED_VALUE_SCSS = IMPORTED_VALUE_WORKSPACE.file(BUTTON_SCSS_URI).content;

const TOKENS_SCSS = `@value primary: #ff3355;`;

const SASS_SYMBOL_WORKSPACE = workspace({
  [BUTTON_SCSS_URI]: `$gap: 1rem;
.button {
  color: $/*|*/gap;
}
`,
});

const SASS_SYMBOL_SCSS = SASS_SYMBOL_WORKSPACE.file(BUTTON_SCSS_URI).content;

const FUNCTION_DYNAMIC_WORKSPACE = workspace({
  [STATUS_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './Status.module.scss';
const cx = classNames.bind(styles);
type Status = 'idle' | 'busy' | 'error';
function resolveStatusClass(status: Status): string {
  switch (status) {
    case 'idle':
      return 'state-idle';
    case 'busy':
      return 'state-busy';
    case 'error':
      return 'state-error';
    default:
      return 'state-idle';
  }
}
export function StatusChip(status: Status) {
  const derivedStatusClass = resolveStatusClass(status);
  return <div className={cx('chip', deri/*|*/vedStatusClass)}>hi</div>;
}
`,
});

const FUNCTION_DYNAMIC_TSX = FUNCTION_DYNAMIC_WORKSPACE.file(STATUS_TSX_URI).content;

const FUNCTION_DYNAMIC_SCSS = `
.chip { color: slategray; }
.state-idle { color: teal; }
.state-busy { color: orange; }
.state-error { color: crimson; }
`;

const SUFFIX_DYNAMIC_WORKSPACE = workspace({
  [STATE_CHIP_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './StateChip.module.scss';
const cx = classNames.bind(styles);
export function StateChip(variant: string) {
  const derivedChipClass = variant + '-chip';
  return <div className={cx('chip', derive/*|*/dChipClass)}>hi</div>;
}
`,
});

const SUFFIX_DYNAMIC_TSX = SUFFIX_DYNAMIC_WORKSPACE.file(STATE_CHIP_TSX_URI).content;

const SUFFIX_DYNAMIC_SCSS = `
.chip { color: slategray; }
.idle-chip { color: teal; }
.busy-chip { color: orange; }
.error-chip { color: crimson; }
`;

const PREFIX_SUFFIX_DYNAMIC_WORKSPACE = workspace({
  [BUTTON_CHIP_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './ButtonChip.module.scss';
const cx = classNames.bind(styles);
export function ButtonChip(variant: string) {
  const derivedChipClass = 'btn-' + variant + '-chip';
  return <div className={cx('chip', derive/*|*/dChipClass)}>hi</div>;
}
`,
});

const PREFIX_SUFFIX_DYNAMIC_TSX = PREFIX_SUFFIX_DYNAMIC_WORKSPACE.file(BUTTON_CHIP_TSX_URI).content;

const PREFIX_SUFFIX_DYNAMIC_SCSS = `
.chip { color: slategray; }
.btn-idle-chip { color: teal; }
.btn-busy-chip { color: orange; }
.btn-error-chip { color: crimson; }
`;

function fixturePositionParams(
  source: CmeWorkspace,
  filePath: string,
): {
  readonly textDocument: { readonly uri: string };
  readonly position: { readonly line: number; readonly character: number };
} {
  return textDocumentPositionParams({
    workspace: source,
    documentUri: filePath,
    filePath,
  });
}

describe("hover protocol / clsx", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  const CLSX_WORKSPACE = workspace({
    [BUTTON_TSX_URI]: `import clsx from 'clsx';
import styles from './Button.module.scss';
export function Button() {
  return <div className={clsx(styles.indic/*|*/ator)}>hi</div>;
}
`,
  });

  const CLSX_TSX = CLSX_WORKSPACE.file(BUTTON_TSX_URI).content;

  const CLSX_SCSS = `
.indicator {
  color: red;
  font-size: 14px;
}
`;

  it("returns a markdown Hover for styles.indicator inside clsx()", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? CLSX_SCSS : null),
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
    // Line 3: "  return <div className={clsx(styles.indicator)}>hi</div>;"
    // "indicator" starts at character 38 (after "styles.")
    const hover = await client.hover(fixturePositionParams(CLSX_WORKSPACE, BUTTON_TSX_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.indicator`");
    expect(value).toContain("color: red;");
  });

  it("keeps real style hover when a template literal contains a resolvable fake import", async () => {
    const poisonedWorkspace = workspace({
      [BUTTON_TSX_URI]: `const fake = \`import styles from './AFake.module.scss';\`;
import styles from './Button.module.scss';
export const value = styles.indic/*|*/ator;
`,
    });
    client = createInProcessServer({
      readStyleFile: (filePath) =>
        filePath.endsWith("Button.module.scss") ? CLSX_SCSS : ".indicator { color: hotpink; }",
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: poisonedWorkspace.file(BUTTON_TSX_URI).content,
      },
    });

    const hover = await client.hover(fixturePositionParams(poisonedWorkspace, BUTTON_TSX_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.indicator`");
    expect(value).toContain("color: red;");
    expect(value).not.toContain("hotpink");
  });
});

describe("hover protocol", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("returns a markdown Hover for cx('indicator')", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? BUTTON_SCSS : null),
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
    const hover = await client.hover(fixturePositionParams(BUTTON_TSX_WORKSPACE, BUTTON_TSX_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.indicator`");
    expect(value).toContain("color: red;");
    expect(value).toContain("font-size: 14px;");
  });

  it("returns a hover for same-file helper derived class candidates", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Status.module.scss") ? FUNCTION_DYNAMIC_SCSS : null),
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: STATUS_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: FUNCTION_DYNAMIC_TSX,
      },
    });
    const hover = await client.hover(
      fixturePositionParams(FUNCTION_DYNAMIC_WORKSPACE, STATUS_TSX_URI),
    );
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("state-idle");
    expect(value).toContain("state-busy");
    expect(value).toContain("state-error");
  });

  it("returns a hover for suffix-constrained derived class candidates", async () => {
    client = createInProcessServer({
      readStyleFile: (path) =>
        path.endsWith("StateChip.module.scss") ? SUFFIX_DYNAMIC_SCSS : null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: STATE_CHIP_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: SUFFIX_DYNAMIC_TSX,
      },
    });
    const hover = await client.hover(
      fixturePositionParams(SUFFIX_DYNAMIC_WORKSPACE, STATE_CHIP_TSX_URI),
    );
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("idle-chip");
    expect(value).toContain("busy-chip");
    expect(value).toContain("error-chip");
    expect(value).toContain("Value domain: suffix `-chip`.");
    expect(value).toContain(
      "Value certainty reason: known suffix preserved while prepending an unknown prefix.",
    );
  });

  it("returns a hover for prefix-suffix constrained derived class candidates", async () => {
    client = createInProcessServer({
      readStyleFile: (path) =>
        path.endsWith("ButtonChip.module.scss") ? PREFIX_SUFFIX_DYNAMIC_SCSS : null,
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_CHIP_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: PREFIX_SUFFIX_DYNAMIC_TSX,
      },
    });
    const hover = await client.hover(
      fixturePositionParams(PREFIX_SUFFIX_DYNAMIC_WORKSPACE, BUTTON_CHIP_TSX_URI),
    );
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("btn-idle-chip");
    expect(value).toContain("btn-busy-chip");
    expect(value).toContain("btn-error-chip");
    expect(value).toContain("Value domain: prefix `btn-` + suffix `-chip`.");
    expect(value).toContain(
      "Value certainty reason: known prefix and suffix were preserved across concatenation.",
    );
  });

  it("returns a selector hover for SCSS declarations", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? BUTTON_SCSS : null),
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
    client.didOpen({
      textDocument: {
        uri: BUTTON_SCSS_URI,
        languageId: "scss",
        version: 1,
        text: BUTTON_SCSS,
      },
    });
    await client.waitForDiagnostics(BUTTON_TSX_URI);

    const hover = await client.hover(fixturePositionParams(BUTTON_SCSS_WORKSPACE, BUTTON_SCSS_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.indicator`");
    expect(value).toContain("References: 1 total.");
    expect(value).toContain("color: red;");
  });

  it("returns a selector hover for a cross-file composes token", async () => {
    const tsxUri = "file:///fake/workspace/src/Base.tsx";
    const buttonScssUri = "file:///fake/workspace/src/Button.module.scss";
    const baseScssUri = "file:///fake/workspace/src/Base.module.scss";
    const baseTsx = `import classNames from 'classnames/bind';
import styles from './Base.module.scss';
const cx = classNames.bind(styles);
export function Base() {
  return <div className={cx('base')}>hi</div>;
}
`;
    const buttonScssWorkspace = workspace({
      [buttonScssUri]: `
.button {
  composes: b/*|*/ase from './Base.module.scss';
  color: red;
}
`,
    });
    const buttonScss = buttonScssWorkspace.file(buttonScssUri).content;
    const baseScss = `
.base {
  color: blue;
}
`;
    client = createInProcessServer({
      readStyleFile: (path) => {
        if (path.endsWith("Button.module.scss")) return buttonScss;
        if (path.endsWith("Base.module.scss")) return baseScss;
        return null;
      },
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: tsxUri,
        languageId: "typescriptreact",
        version: 1,
        text: baseTsx,
      },
    });
    client.didOpen({
      textDocument: {
        uri: buttonScssUri,
        languageId: "scss",
        version: 1,
        text: buttonScss,
      },
    });
    client.didOpen({
      textDocument: {
        uri: baseScssUri,
        languageId: "scss",
        version: 1,
        text: baseScss,
      },
    });
    await client.waitForDiagnostics(tsxUri);

    const hover = await client.hover(fixturePositionParams(buttonScssWorkspace, buttonScssUri));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.base`");
    expect(value).toContain("Referenced via `composes` from `.button`");
    expect(value).toContain("color: blue;");
  });

  it("returns null on unknown class", async () => {
    client = createInProcessServer({
      readStyleFile: () => ".other { color: red; }",
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
    const hover = await client.hover(fixturePositionParams(BUTTON_TSX_WORKSPACE, BUTTON_TSX_URI));
    expect(hover).toBeNull();
  });

  it("returns a keyframes hover for @keyframes declarations", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? KEYFRAMES_SCSS : null),
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

    const hover = await client.hover(fixturePositionParams(KEYFRAMES_WORKSPACE, BUTTON_SCSS_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`@keyframes fade`");
    expect(value).toContain("2 animation references");
  });

  it("returns a value hover for @value declarations", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? VALUE_SCSS : null),
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

    const hover = await client.hover(fixturePositionParams(VALUE_WORKSPACE, BUTTON_SCSS_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`@value primary`");
    expect(value).toContain("1 value reference");
  });

  it("returns a value hover for imported @value usages", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => {
        if (path.endsWith("Button.module.scss")) return IMPORTED_VALUE_SCSS;
        if (path.endsWith("tokens.module.scss")) return TOKENS_SCSS;
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
        text: IMPORTED_VALUE_SCSS,
      },
    });

    const hover = await client.hover(
      fixturePositionParams(IMPORTED_VALUE_WORKSPACE, BUTTON_SCSS_URI),
    );
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`@value primary`");
    expect(value).toContain("imported from `./tokens.module.scss` as `primary`");
  });

  it("returns a Sass symbol hover for same-file references", async () => {
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? SASS_SYMBOL_SCSS : null),
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

    const hover = await client.hover(fixturePositionParams(SASS_SYMBOL_WORKSPACE, BUTTON_SCSS_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`$gap`");
    expect(value).toContain("1 Sass symbol reference");
  });

  it("includes dynamic explanation for a flow-resolved symbol ref", async () => {
    const flowWorkspace = workspace({
      [BUTTON_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button(enabled: boolean) {
  const size = enabled ? 'indicator' : 'active';
  return <div className={cx(s/*|*/ize)}>hi</div>;
}
`,
    });
    const FLOW_TSX = flowWorkspace.file(BUTTON_TSX_URI).content;
    client = createInProcessServer({
      readStyleFile: (path) => (path.endsWith("Button.module.scss") ? BUTTON_SCSS : null),
      typeResolver: new FakeTypeResolver(),
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: {
        uri: BUTTON_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: FLOW_TSX,
      },
    });
    const hover = await client.hover(fixturePositionParams(flowWorkspace, BUTTON_TSX_URI));
    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("Resolved from `size` via branched local flow analysis.");
    expect(value).toContain("Value certainty: inferred.");
    expect(value).toContain("Value certainty shape: bounded finite (2).");
    expect(value).toContain(
      "Value certainty reason: analysis preserved multiple finite candidate values.",
    );
    expect(value).toContain("Selector certainty: inferred.");
    expect(value).toContain("Candidates: `active`, `indicator`.");
  });
});
