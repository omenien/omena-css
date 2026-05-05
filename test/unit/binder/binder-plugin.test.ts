import { describe, expect, it } from "vitest";
import ts from "typescript";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import {
  composeBinderPluginsV0,
  cssModulesClassnamesBinderPluginV0,
} from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/binder/binder-builder";

const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake", {});

function parse(source: string, filePath = "/fake/src/Card.tsx"): ts.SourceFile {
  return ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
}

describe("cssModulesClassnamesBinderPluginV0", () => {
  it("declares the built-in binder plugin boundary without exposing an external ABI", () => {
    expect(cssModulesClassnamesBinderPluginV0).toMatchObject({
      id: "css-modules-classnames-bind",
      version: "0",
      stability: "builtIn",
      domains: ["css-modules"],
    });
    expect(cssModulesClassnamesBinderPluginV0.importTargets).toEqual([
      "*.module.css",
      "*.module.scss",
      "*.module.less",
    ]);
    expect(cssModulesClassnamesBinderPluginV0.utilityTargets).toEqual([
      "classnames/bind",
      "classnames",
      "clsx",
      "clsx/lite",
    ]);
  });

  it("ports current CSS Modules, cx, style access, and class util facts behind one plugin", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import clsx from 'clsx';
      import styles from './Card.module.scss';
      const cx = classNames.bind(styles);
      const tone = 'primary';
      const el = <div className={clsx(cx('card', \`tone-\${tone}\`), styles.icon)} />;
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = cssModulesClassnamesBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.pluginId).toBe("css-modules-classnames-bind");
    expect(result.stylesBindings.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Card.module.scss",
    });
    expect(result.cxBindings).toMatchObject([
      {
        cxVarName: "cx",
        stylesVarName: "styles",
        scssModulePath: "/fake/src/Card.module.scss",
      },
    ]);
    expect(result.classUtilNames).toEqual(["clsx"]);
    expect(result.classExpressions.map((entry) => entry.kind)).toEqual([
      "literal",
      "template",
      "styleAccess",
    ]);
    expect(result.domainClassReferences).toEqual([]);
  });

  it("tracks Tailwind/Uno utility classes without pretending to own a CSS Module source", () => {
    const sourceFile = parse(`
      import clsx from 'clsx';
      export function Card({ active }: { active: boolean }) {
        return (
          <section className={clsx("flex gap-2", active && "bg-blue-500", \`tone-\${active}\`)}>
            <div className="text-sm hover:underline" />
          </section>
        );
      }
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = tailwindUnoUtilityBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.size).toBe(0);
    expect(result.classExpressions).toEqual([]);
    expect(result.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "flex", domain: "utility-css" },
      { matchKind: "literal", className: "gap-2", domain: "utility-css" },
      { matchKind: "literal", className: "bg-blue-500", domain: "utility-css" },
      { matchKind: "templatePrefix", staticPrefix: "tone-" },
      { matchKind: "literal", className: "text-sm", origin: "jsxClassAttribute" },
      { matchKind: "literal", className: "hover:underline", origin: "jsxClassAttribute" },
    ]);
  });

  it("composes CSS Modules and utility-domain plugins into one BinderPluginV0 runtime slot", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import clsx from 'clsx';
      import styles from './Card.module.scss';
      const cx = classNames.bind(styles);
      const el = <div className={clsx(cx('card'), "flex")} />;
    `);
    const sourceBinder = buildSourceBinder(sourceFile);
    const plugin = composeBinderPluginsV0([
      cssModulesClassnamesBinderPluginV0,
      tailwindUnoUtilityBinderPluginV0,
    ]);

    const result = plugin.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Card.module.scss",
    });
    expect(result.classExpressions).toMatchObject([{ kind: "literal", className: "card" }]);
    expect(result.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "flex", domain: "utility-css" },
    ]);
  });
});
