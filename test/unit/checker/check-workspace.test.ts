import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { checkWorkspace } from "../../../server/engine-host-node/src/checker-host";

const tempDirs: string[] = [];

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});

describe("checkWorkspace", () => {
  it("reports source and style findings from one semantic pass", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "const ok = cx('button');",
        "const bad = cx('missing');",
        "",
      ].join("\n"),
      "src/Button.module.scss": [".button {}", ".unused {}"].join("\n"),
    });

    const result = await checkWorkspace({ workspaceRoot });

    expect(result.sourceFiles).toEqual([path.join(workspaceRoot, "src/App.tsx")]);
    expect(result.styleFiles).toEqual([path.join(workspaceRoot, "src/Button.module.scss")]);
    expect(result.findings).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          filePath: path.join(workspaceRoot, "src/App.tsx"),
          finding: expect.objectContaining({
            category: "source",
            code: "missing-static-class",
            className: "missing",
          }),
        }),
        expect.objectContaining({
          filePath: path.join(workspaceRoot, "src/Button.module.scss"),
          finding: expect.objectContaining({
            category: "style",
            code: "unused-selector",
            canonicalName: "unused",
          }),
        }),
      ]),
    );
    expect(result.summary).toMatchObject({
      warnings: 1,
      hints: 1,
      total: 2,
    });
  });

  it("honors composes resolution during style checks", async () => {
    const workspaceRoot = makeWorkspace({
      "src/Button.module.scss": ".button { composes: base from './Base.module.scss'; }",
      "src/Base.module.scss": ".other {}",
    });

    const result = await checkWorkspace({ workspaceRoot });

    expect(result.findings).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          filePath: path.join(workspaceRoot, "src/Button.module.scss"),
          finding: expect.objectContaining({
            category: "style",
            code: "missing-composed-selector",
            className: "base",
          }),
        }),
      ]),
    );
  });

  it("uses literal Vite aliases during checker workspace analysis", async () => {
    const workspaceRoot = makeWorkspace({
      "vite.config.ts": [
        "export default {",
        "  resolve: { alias: { '@styles': './src/styles' } },",
        "};",
        "",
      ].join("\n"),
      "src/App.tsx": [
        "import styles from '@styles/Button.module.scss';",
        "const ok = styles.button;",
        "",
      ].join("\n"),
      "src/styles/Button.module.scss": ".button {}",
    });

    const result = await checkWorkspace({ workspaceRoot });

    expect(result.findings).toEqual([]);
  });
});

function makeWorkspace(files: Readonly<Record<string, string>>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-workspace-"));
  tempDirs.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const filePath = path.join(root, relativePath);
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, content, "utf8");
  }
  return root;
}
