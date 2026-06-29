import { describe, expect, it } from "vitest";
import ts from "typescript";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/source-frontend/ts-source-binder-oracle";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import { scanCxImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import { resolveCxBindings } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";

const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake", {});

function parse(source: string, filePath = "/fake/src/Button.tsx"): ts.SourceFile {
  return ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
}

describe("resolveCxBindings", () => {
  it("drops bindings when classnames/bind is shadowed by a local parameter", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      function render(classNames: { bind(value: unknown): unknown }) {
        const cx = classNames.bind(styles);
        return cx;
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const { bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );

    expect(bindings).toHaveLength(1);
    expect(resolveCxBindings(bindings, binder)).toEqual([]);
  });

  it("drops bindings when the imported styles name is shadowed locally", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      function render(styles: Record<string, string>) {
        const cx = classNames.bind(styles);
        return cx;
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const { bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );

    expect(bindings).toHaveLength(1);
    expect(resolveCxBindings(bindings, binder)).toEqual([]);
  });

  it("keeps a valid imported classnames/styles binding", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
    `);
    const binder = buildSourceBinder(sourceFile);
    const { bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );

    expect(resolveCxBindings(bindings, binder)).toMatchObject([
      {
        cxVarName: "cx",
        stylesVarName: "styles",
        classNamesImportName: "classNames",
        scssModulePath: "/fake/src/Button.module.scss",
      },
    ]);
  });
});
