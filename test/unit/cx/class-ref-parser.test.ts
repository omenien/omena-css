import { describe, expect, it } from "vitest";
import ts from "typescript";
import { scanCxImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import { parseClassExpressions } from "../../../server/engine-core-ts/src/core/cx/class-ref-parser";
import { resolveCxBindings } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import {
  buildSourceBinder,
  resolveIdentifierAtOffset,
} from "../../../server/engine-core-ts/src/core/binder/binder-builder";

const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake", {});

function parse(source: string, filePath = "/fake/src/Button.tsx"): ts.SourceFile {
  return ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
}

describe("parseClassExpressions", () => {
  it("parses cx() calls when the bind initializer is wrapped in transparent expressions", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = (classNames as typeof classNames).bind((styles));
      const el = cx('button');
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings, bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(
      sourceFile,
      resolveCxBindings(bindings, binder),
      stylesBindings,
      binder,
    );

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "literal",
      className: "button",
      scssModulePath: "/fake/src/Button.module.scss",
    });
  });

  it("unwraps transparent expressions around symbol arguments", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
      const size = 'lg' as const;
      const el = cx((size as string));
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings, bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(
      sourceFile,
      resolveCxBindings(bindings, binder),
      stylesBindings,
      binder,
    );

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "symbolRef",
      rawReference: "size",
      rootName: "size",
      pathSegments: [],
    });
    const resolution = resolveIdentifierAtOffset(
      binder,
      "size",
      sourceFile.text.indexOf("size as string"),
    );
    expect(expressions[0]).toMatchObject({
      rootBindingDeclId: resolution?.declId,
    });
  });

  it("emits template expressions for string concatenation with a static prefix", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
      const variant = 'primary';
      const el = cx('btn-' + variant);
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings, bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(
      sourceFile,
      resolveCxBindings(bindings, binder),
      stylesBindings,
      binder,
    );

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "template",
      rawTemplate: "'btn-' + variant",
      staticPrefix: "btn-",
    });
  });

  it("emits template expressions for computed object keys", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
      const tone = 'info';
      const el = cx({ [\`tone-\${tone}\`]: tone });
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings, bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(
      sourceFile,
      resolveCxBindings(bindings, binder),
      stylesBindings,
      binder,
    );

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "template",
      rawTemplate: "`tone-${tone}`",
      staticPrefix: "tone-",
    });
  });

  it("emits symbol references for computed object key identifiers", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
      const className = 'button';
      const el = cx({ [className]: true });
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings, bindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(
      sourceFile,
      resolveCxBindings(bindings, binder),
      stylesBindings,
      binder,
    );

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "symbolRef",
      rawReference: "className",
      rootName: "className",
      pathSegments: [],
    });
  });

  it("records binder-linked styleAccess expressions for imported styles", () => {
    const sourceFile = parse(`
      import styles from './Button.module.scss';
      const el = <div className={styles.button} />;
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(sourceFile, [], stylesBindings, binder);

    expect(expressions).toHaveLength(1);
    expect(expressions[0]).toMatchObject({
      kind: "styleAccess",
      className: "button",
      bindingDeclId: "decl:0",
    });
  });

  it("does not emit styleAccess when a local binding shadows the imported styles name", () => {
    const sourceFile = parse(`
      import styles from './Button.module.scss';
      function render(styles: { button: string }) {
        return <div className={styles.button} />;
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const { stylesBindings } = scanCxImports(
      sourceFile,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    const expressions = parseClassExpressions(sourceFile, [], stylesBindings, binder);

    expect(expressions).toEqual([]);
  });
});
