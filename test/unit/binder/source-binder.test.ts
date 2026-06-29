import { describe, expect, it } from "vitest";
import ts from "typescript";
import {
  getDeclById,
  resolveIdentifierAtOffset,
} from "../../../server/engine-core-ts/src/core/binder/source-binder";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/source-frontend/ts-source-binder-oracle";

function parseMarked(
  source: string,
  filePath = "/fake/src/App.tsx",
): {
  sourceFile: ts.SourceFile;
  offset: number;
} {
  const marker = "__MARK__";
  const offset = source.indexOf(marker);
  if (offset === -1) {
    throw new Error("missing __MARK__ marker");
  }
  const text = source.replace(marker, "");
  return {
    sourceFile: ts.createSourceFile(
      filePath,
      text,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    ),
    offset,
  };
}

describe("buildSourceBinder", () => {
  it("resolves an imported identifier from the file scope", () => {
    const { sourceFile, offset } = parseMarked(`
      import styles from "./App.module.scss";
      const view = __MARK__styles.button;
    `);
    const binder = buildSourceBinder(sourceFile);
    const resolution = resolveIdentifierAtOffset(binder, "styles", offset);
    const decl = resolution ? getDeclById(binder, resolution.declId) : null;
    expect(decl).toMatchObject({
      kind: "import",
      name: "styles",
      importPath: "./App.module.scss",
    });
  });

  it("prefers the innermost shadowing local over an outer declaration", () => {
    const { sourceFile, offset } = parseMarked(`
      const size = "outer";
      function view() {
        if (true) {
          const size = "inner";
          return __MARK__size;
        }
        return size;
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const resolution = resolveIdentifierAtOffset(binder, "size", offset);
    const decl = resolution ? getDeclById(binder, resolution.declId) : null;
    expect(decl?.kind).toBe("localVar");
    expect(sourceFile.text.slice(decl!.span.start, decl!.span.end)).toBe("size");
    expect(resolution?.depth).toBe(0);
  });

  it("resolves a parameter before outer locals when inside the function body", () => {
    const { sourceFile, offset } = parseMarked(`
      const variant = "outer";
      function view(variant: string) {
        return __MARK__variant;
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const resolution = resolveIdentifierAtOffset(binder, "variant", offset);
    const decl = resolution ? getDeclById(binder, resolution.declId) : null;
    expect(decl?.kind).toBe("parameter");
    expect(resolution?.depth).toBe(1);
  });

  it("does not leak sibling block declarations across blocks", () => {
    const { sourceFile, offset } = parseMarked(`
      function view() {
        if (true) {
          const tone = "warm";
        }
        {
          return __MARK__tone;
        }
      }
    `);
    const binder = buildSourceBinder(sourceFile);
    const resolution = resolveIdentifierAtOffset(binder, "tone", offset);
    expect(resolution).toBeNull();
  });
});
