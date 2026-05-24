import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  createInProcessServer,
  emptySupplier,
  type LspTestClient,
} from "./_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";
import {
  textDocumentPositionParams,
  workspace,
  type CmeWorkspace,
} from "../../packages/vitest-cme/src";

const COMPONENT_SCSS_URI = "file:///fake/workspace/src/components/ItemList.module.scss";
const COMPONENT_TSX_URI = "file:///fake/workspace/src/components/ItemList.tsx";

const PACKAGE_VARIABLES_SCSS = `$gray200: #eeeeef;
$gray600: #767678;
`;

const PACKAGE_TYPOGRAPHY_SCSS = `@mixin typography14 {}`;

const USE_ALIAS_WORKSPACE = workspace({
  [COMPONENT_SCSS_URI]: `@use "$/scss/utils" as *;

.font14 { @include ds_typography14; }
.gray600 { color: $ds_gray600; }
.item { position: relative; }
`,
  [COMPONENT_TSX_URI]: `import classNames from 'classnames/bind';
import styles from './ItemList.module.scss';
const cx = classNames.bind(styles);
export function ItemList() {
  return <li className={cx('it/*|*/em')}>item</li>;
}
`,
});

const IMPORT_ALIAS_WORKSPACE = workspace({
  [COMPONENT_SCSS_URI]: `@import "$shared/utils";

.article {
  border: 1px solid $ds_g/*at:gray200*/ray200;
  color: $ds_g/*at:gray600*/ray600;
}
`,
});

function writeTsconfig(workspacePath: string, paths: Record<string, readonly string[]>): void {
  fs.writeFileSync(
    path.join(workspacePath, "tsconfig.json"),
    JSON.stringify({ compilerOptions: { baseUrl: ".", paths } }, null, 2),
  );
}

function readWorkspaceStyleFile(
  workspacePath: string,
  files: Readonly<Record<string, string>>,
): (filePath: string) => string | null {
  const workspacePrefix = workspacePath.replaceAll("\\", "/") + "/";
  return (filePath) => {
    const normalized = filePath.replaceAll("\\", "/");
    const relative = normalized.startsWith(workspacePrefix)
      ? normalized.slice(workspacePrefix.length)
      : normalized;
    return files[relative] ?? null;
  };
}

function positionParams(
  source: CmeWorkspace,
  documentUri: string,
  markerName?: string,
): {
  readonly textDocument: { readonly uri: string };
  readonly position: { readonly line: number; readonly character: number };
} {
  return textDocumentPositionParams({
    workspace: source,
    documentUri,
    filePath: documentUri,
    ...(markerName === undefined ? {} : { markerName }),
  });
}

describe("SCSS path alias token protocol integration", () => {
  let client: LspTestClient | null = null;
  let workspacePath: string | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
    if (workspacePath) {
      fs.rmSync(workspacePath, { recursive: true, force: true });
      workspacePath = null;
    }
  });

  it("keeps @use path-alias modules indexed and resolves forwarded package tokens", async () => {
    workspacePath = fs.mkdtempSync(path.join(os.tmpdir(), "cme-scss-use-alias-"));
    writeTsconfig(workspacePath, { "$/*": ["./*"] });
    const scss = USE_ALIAS_WORKSPACE.file(COMPONENT_SCSS_URI).content;
    const tsx = USE_ALIAS_WORKSPACE.file(COMPONENT_TSX_URI).content;

    client = createInProcessServer({
      workspacePath,
      readStyleFile: readWorkspaceStyleFile(workspacePath, {
        "src/components/ItemList.module.scss": scss,
        "scss/_utils.scss": `@forward "@my-design-system/foundation/scss/variables" as ds_*;
@forward "@my-design-system/foundation/scss/typography" as ds_*;
`,
        "node_modules/@my-design-system/foundation/scss/_variables.scss": PACKAGE_VARIABLES_SCSS,
        "node_modules/@my-design-system/foundation/scss/_typography.scss": PACKAGE_TYPOGRAPHY_SCSS,
      }),
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: { uri: COMPONENT_SCSS_URI, languageId: "scss", version: 1, text: scss },
    });
    client.didOpen({
      textDocument: {
        uri: COMPONENT_TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: tsx,
      },
    });

    const scssDiagnostics = await client.waitForDiagnostics(COMPONENT_SCSS_URI);
    expect(scssDiagnostics.filter((diagnostic) => diagnostic.message.includes("Sass "))).toEqual(
      [],
    );

    const hover = await client.hover(positionParams(USE_ALIAS_WORKSPACE, COMPONENT_TSX_URI));
    expect((hover!.contents as { value: string }).value).toContain("`.item`");
  });

  it("resolves legacy @import path-alias forwards to package token definitions", async () => {
    workspacePath = fs.mkdtempSync(path.join(os.tmpdir(), "cme-scss-import-alias-"));
    writeTsconfig(workspacePath, { "$shared/*": ["shared/*"] });
    const scss = IMPORT_ALIAS_WORKSPACE.file(COMPONENT_SCSS_URI).content;

    client = createInProcessServer({
      workspacePath,
      readStyleFile: readWorkspaceStyleFile(workspacePath, {
        "src/components/ItemList.module.scss": scss,
        "shared/_utils.scss": `@forward "@my-design-system/foundation/scss/variables" as ds_*;`,
        "node_modules/@my-design-system/foundation/scss/_variables.scss": PACKAGE_VARIABLES_SCSS,
      }),
      typeResolver: new FakeTypeResolver(),
      fileSupplier: emptySupplier,
    });
    await client.initialize();
    client.initialized();
    client.didOpen({
      textDocument: { uri: COMPONENT_SCSS_URI, languageId: "scss", version: 1, text: scss },
    });

    const diagnostics = await client.waitForDiagnostics(COMPONENT_SCSS_URI);
    expect(diagnostics.filter((diagnostic) => diagnostic.code === "missingSassSymbol")).toEqual([]);
    expect(
      diagnostics.find((diagnostic) => diagnostic.code === "deprecatedSassImport"),
    ).toMatchObject({
      message: "Sass @import is deprecated; prefer @use or @forward.",
    });

    const definition = await client.definition(
      positionParams(IMPORT_ALIAS_WORKSPACE, COMPONENT_SCSS_URI, "gray200"),
    );
    expect(definition).toEqual([
      expect.objectContaining({
        targetUri:
          "file:///fake/workspace/node_modules/@my-design-system/foundation/scss/_variables.scss",
      }),
    ]);
  });
});
