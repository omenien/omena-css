import { describe, it, expect } from "vitest";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import ts from "typescript";
import type { StyleImport } from "@omena/shared";
import { scanCxImports } from "../../../server/engine-core-ts/src/core/cx/binding-detector";
import {
  AliasResolver,
  loadWorkspaceTsconfigPathAliases,
} from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";

const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake", {});

// Thin adapters over `scanCxImports` that project its single result
// back to the two outputs these tests pin individually. Keeps every
// behavioural assertion intact while the production code uses the
// consolidated single-walk API.
function detectCxBindings(
  src: ts.SourceFile,
  filePath: string,
  aliasResolver: AliasResolver,
  fileExists: (p: string) => boolean = () => true,
): readonly CxBinding[] {
  return scanCxImports(src, filePath, fileExists, aliasResolver).bindings;
}

function collectStyleImports(
  src: ts.SourceFile,
  filePath: string,
  fileExists: (p: string) => boolean,
  aliasResolver: AliasResolver,
): ReadonlyMap<string, StyleImport> {
  return scanCxImports(src, filePath, fileExists, aliasResolver).stylesBindings;
}

function parse(source: string, filePath = "/fake/src/Button.tsx"): ts.SourceFile {
  return ts.createSourceFile(
    filePath,
    source,
    ts.ScriptTarget.Latest,
    /*setParentNodes*/ true,
    ts.ScriptKind.TSX,
  );
}

describe("detectCxBindings / standard pattern", () => {
  it("detects a plain top-level binding", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
    expect(bindings[0]).toMatchObject({
      cxVarName: "cx",
      stylesVarName: "styles",
      classNamesImportName: "classNames",
    });
  });

  it("resolves relative scss paths against the filePath directory", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings[0]!.scssModulePath).toBe("/fake/src/Button.module.scss");
  });

  it("resolves ../ paths correctly", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from '../styles/Button.module.scss';
      const cx = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/components/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings[0]!.scssModulePath).toBe("/fake/src/styles/Button.module.scss");
  });
});

describe("detectCxBindings / free variable names", () => {
  it("honors a non-'cx' variable name", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const classes = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings[0]!.cxVarName).toBe("classes");
  });
});

describe("detectCxBindings / aliased classnames import", () => {
  it("detects 'cn.bind(styles)' when classnames/bind is imported as 'cn'", () => {
    const src = parse(`
      import cn from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = cn.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
    expect(bindings[0]!.classNamesImportName).toBe("cn");
  });

  it("ignores a .bind() call from an unrelated import", () => {
    const src = parse(`
      import cn from 'some-other-lib';
      import styles from './Button.module.scss';
      const cx = cn.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(0);
  });

  it("unwraps transparent expressions around the bind target and styles argument", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = (classNames as typeof classNames).bind((styles satisfies typeof styles));
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
    expect(bindings[0]).toMatchObject({
      cxVarName: "cx",
      stylesVarName: "styles",
      classNamesImportName: "classNames",
      scssModulePath: "/fake/src/Button.module.scss",
    });
  });
});

describe("detectCxBindings / free styles name", () => {
  it("honors a non-'styles' variable name", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import btnStyles from './Button.module.scss';
      const cx = classNames.bind(btnStyles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings[0]!.stylesVarName).toBe("btnStyles");
  });
});

describe("detectCxBindings / multiple bindings per file", () => {
  it("detects two bindings to different style imports", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import btnStyles from './Button.module.scss';
      import formStyles from './Form.module.scss';
      const cxBtn = classNames.bind(btnStyles);
      const cxForm = classNames.bind(formStyles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(2);
    const names = bindings.map((b) => b.cxVarName).toSorted();
    expect(names).toEqual(["cxBtn", "cxForm"]);
  });
});

describe("detectCxBindings / function-scoped binding", () => {
  it("detects a binding declared inside a function body", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';

      function Button() {
        const cx = classNames.bind(styles);
        return null;
      }
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
    const b = bindings[0]!;
    expect(b.cxVarName).toBe("cx");
    expect(b.bindingRange.start.line).toBeGreaterThan(0);
    expect(b.bindingRange.end.line).toBeGreaterThanOrEqual(b.bindingRange.start.line);
  });

  it("gives a top-level binding the file scope", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const cx = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    const b = bindings[0]!;
    expect(b.bindingRange.start.line).toBe(3);
    expect(b.bindingRange.end.line).toBe(3);
  });
});

describe("detectCxBindings / negative cases", () => {
  it("returns [] when there is no classnames/bind import", () => {
    const src = parse(`
      import styles from './Button.module.scss';
      const cx = {};
    `);
    expect(detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER)).toEqual([]);
  });

  it("returns [] when there is no CSS module import", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      const cx = classNames.bind({});
    `);
    expect(detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER)).toEqual([]);
  });

  it("ignores a .bind() call on a different object", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import styles from './Button.module.scss';
      const fn = console.log.bind(console);
      const cx = classNames.bind(styles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
  });
});

describe("detectCxBindings / single-walk consolidation (4.2.c)", () => {
  // Golden-equivalence fixture for the 4.2.c refactor. Previously
  // `collectImports` walked `sourceFile.statements` twice (once via
  // `collectStyleImports`, once for classnames/bind). The consolidated
  // version walks once. This test fixes the expected CxBinding[] output
  // for a file containing BOTH import kinds so any regression in the
  // combined walk (order, lost entries, misclassified specifiers) fails
  // here.
  it("produces the expected CxBinding[] for a fixture with both classnames/bind and .module.scss imports", () => {
    const src = parse(`
      import classNames from 'classnames/bind';
      import btnStyles from './Button.module.scss';
      import formStyles from './Form.module.css';
      const cxBtn = classNames.bind(btnStyles);
      const cxForm = classNames.bind(formStyles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    // Same walk must still discover both bindings with correct
    // classNamesImportName AND correct scssModulePath.
    expect(bindings).toHaveLength(2);
    const byName = new Map(bindings.map((b) => [b.cxVarName, b]));
    expect(byName.get("cxBtn")).toMatchObject({
      cxVarName: "cxBtn",
      stylesVarName: "btnStyles",
      classNamesImportName: "classNames",
      scssModulePath: "/fake/src/Button.module.scss",
    });
    expect(byName.get("cxForm")).toMatchObject({
      cxVarName: "cxForm",
      stylesVarName: "formStyles",
      classNamesImportName: "classNames",
      scssModulePath: "/fake/src/Form.module.css",
    });
  });

  it("handles interleaved classnames/bind and style imports in a single walk", () => {
    // Intentional ordering: style import, classnames/bind, another style
    // import. The single-pass walk must not depend on encounter order.
    const src = parse(`
      import btnStyles from './Button.module.scss';
      import cn from 'classnames/bind';
      import formStyles from './Form.module.scss';
      const cx = cn.bind(formStyles);
    `);
    const bindings = detectCxBindings(src, "/fake/src/Button.tsx", EMPTY_ALIAS_RESOLVER);
    expect(bindings).toHaveLength(1);
    expect(bindings[0]).toMatchObject({
      cxVarName: "cx",
      stylesVarName: "formStyles",
      classNamesImportName: "cn",
      scssModulePath: "/fake/src/Form.module.scss",
    });
  });
});

describe("collectStyleImports", () => {
  it("collects a default import of a .module.scss file", () => {
    const src = parse(`
      import styles from './Button.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(1);
    expect(result.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Button.module.scss",
    });
  });

  it("collects a namespace import of a .module.scss file", () => {
    const src = parse(`
      import * as styles from './Button.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(1);
    expect(result.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Button.module.scss",
    });
  });

  it("collects multiple style module imports", () => {
    const src = parse(`
      import btnStyles from './Button.module.scss';
      import formStyles from './Form.module.css';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(2);
    expect(result.get("btnStyles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Button.module.scss",
    });
    expect(result.get("formStyles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Form.module.css",
    });
  });

  it("handles .module.less extensions", () => {
    const src = parse(`
      import styles from './Button.module.less';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(1);
    expect(result.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Button.module.less",
    });
  });

  it("resolves ../ paths correctly", () => {
    const src = parse(`
      import styles from '../styles/Button.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/components/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/styles/Button.module.scss",
    });
  });

  it("ignores named imports (no default or namespace)", () => {
    const src = parse(`
      import { something } from './Button.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(0);
  });

  it("ignores non-style-module imports", () => {
    const src = parse(`
      import React from 'react';
      import clsx from 'clsx';
      import styles from './Button.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(1);
    expect(result.has("React")).toBe(false);
    expect(result.has("clsx")).toBe(false);
  });

  it("returns an empty map when there are no style module imports", () => {
    const src = parse(`
      import React from 'react';
      import clsx from 'clsx';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(0);
  });

  // ── missing-module detection via fileExists DI ──

  it("emits missing variant when fileExists returns false", () => {
    const src = parse(`import s from './foo.module.scss';`);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => false,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(1);
    const entry = result.get("s");
    expect(entry?.kind).toBe("missing");
    if (entry?.kind === "missing") {
      expect(entry.specifier).toBe("./foo.module.scss");
      expect(entry.absolutePath).toBe("/fake/src/foo.module.scss");
      // `import s from '` is 15 chars — `.` lands at 0-based LSP char 15.
      expect(entry.range.start.line).toBe(0);
      expect(entry.range.start.character).toBe(15);
      // start 15 + `./foo.module.scss`.length 17 = exclusive end 32.
      expect(entry.range.end.character).toBe(32);
    }
  });

  it("emits resolved variant when fileExists returns true", () => {
    const src = parse(`import s from './foo.module.scss';`);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.get("s")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/foo.module.scss",
    });
  });

  it("calls fileExists at least once per unique import path", () => {
    const src = parse(`
      import a from './a.module.scss';
      import b from './b.module.scss';
    `);
    const calls: string[] = [];
    const spy = (p: string): boolean => {
      calls.push(p);
      return true;
    };
    collectStyleImports(src, "/fake/src/Button.tsx", spy, EMPTY_ALIAS_RESOLVER);
    expect(calls).toContain("/fake/src/a.module.scss");
    expect(calls).toContain("/fake/src/b.module.scss");
    expect(calls.length).toBeGreaterThanOrEqual(2);
  });

  it("emits a missing and a resolved variant for mixed fixture", () => {
    const src = parse(`
      import good from './good.module.scss';
      import bad from './bad.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      (p) => p.endsWith("good.module.scss"),
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.size).toBe(2);
    expect(result.get("good")?.kind).toBe("resolved");
    expect(result.get("bad")?.kind).toBe("missing");
  });

  it("does not call fileExists for non-style imports", () => {
    const src = parse(`import data from './data.json';`);
    let called = false;
    collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => {
        called = true;
        return true;
      },
      EMPTY_ALIAS_RESOLVER,
    );
    expect(called).toBe(false);
  });

  it("classnames/bind import is handled separately and never appears in stylesBindings", () => {
    const src = parse(`
      import cn from 'classnames/bind';
      import s from './x.module.scss';
    `);
    const result = collectStyleImports(
      src,
      "/fake/src/Button.tsx",
      () => true,
      EMPTY_ALIAS_RESOLVER,
    );
    expect(result.has("cn")).toBe(false);
    expect(result.get("s")?.kind).toBe("resolved");
  });

  // ── alias-aware import resolution ──

  it("resolves aliased imports via AliasResolver", () => {
    const src = parse(`import s from '@styles/button.module.scss';`);
    const resolver = new AliasResolver("/fake", { "@styles": "src/styles" });
    const result = collectStyleImports(src, "/fake/src/Button.tsx", () => true, resolver);
    expect(result.size).toBe(1);
    expect(result.get("s")?.kind).toBe("resolved");
    if (result.get("s")?.kind === "resolved") {
      // AliasResolver resolves @styles/button.module.scss against workspace root.
      const entry = result.get("s")!;
      expect(entry.absolutePath).toMatch(/fake\/src\/styles\/button\.module\.scss$/);
    }
  });

  it("resolves tsconfig wildcard aliases via AliasResolver", () => {
    const src = parse(`import s from '$components/button.module.scss';`);
    const resolver = new AliasResolver(
      "/fake",
      {},
      {
        basePath: "/fake/src",
        paths: {
          "$components/*": ["components/*"],
        },
      },
    );
    const result = collectStyleImports(src, "/fake/src/Button.tsx", () => true, resolver);
    expect(result.size).toBe(1);
    expect(result.get("s")?.kind).toBe("resolved");
    if (result.get("s")?.kind === "resolved") {
      expect(result.get("s")!.absolutePath).toBe("/fake/src/components/button.module.scss");
    }
  });

  it("resolves package-level tsconfig aliases from the source file path", () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "cme-monorepo-bindings-"));
    const packageRoot = path.join(workspace, "packages/core");
    const sourcePath = path.join(packageRoot, "src/App.tsx");
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(
      path.join(packageRoot, "tsconfig.json"),
      JSON.stringify(
        {
          compilerOptions: {
            baseUrl: ".",
            paths: {
              "$components/*": ["src/components/*"],
            },
          },
        },
        null,
        2,
      ),
    );
    const src = parse(`import s from '$components/button.module.scss';`, sourcePath);
    const resolver = new AliasResolver(workspace, {}, loadWorkspaceTsconfigPathAliases(workspace));

    const result = collectStyleImports(src, sourcePath, () => true, resolver);

    expect(result.get("s")?.kind).toBe("resolved");
    if (result.get("s")?.kind === "resolved") {
      expect(result.get("s")!.absolutePath).toBe(
        path.join(packageRoot, "src/components/button.module.scss"),
      );
    }
  });

  it("drops aliased imports with no matching alias (fallthrough)", () => {
    const src = parse(`import s from '@nope/button.module.scss';`);
    const resolver = new AliasResolver("/fake", { "@styles": "src/styles" });
    const result = collectStyleImports(src, "/fake/src/Button.tsx", () => true, resolver);
    expect(result.size).toBe(0);
  });

  it("relative specifier wins over alias (order-independent)", () => {
    const src = parse(`import s from './Button.module.scss';`);
    const resolver = new AliasResolver("/fake", { ".": "/wrong" });
    const result = collectStyleImports(src, "/fake/src/Button.tsx", () => true, resolver);
    expect(result.get("s")?.kind).toBe("resolved");
    if (result.get("s")?.kind === "resolved") {
      expect(result.get("s")!.absolutePath).toBe("/fake/src/Button.module.scss");
    }
  });
});
