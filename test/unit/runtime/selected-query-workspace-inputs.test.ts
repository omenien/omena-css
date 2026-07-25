import { describe, expect, it } from "vitest";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { collectSelectedQueryWorkspaceInputs } from "../../../server/engine-host-node/src/selected-query-workspace-inputs";

describe("collectSelectedQueryWorkspaceInputs", () => {
  it("collects recursive CSS Modules and Sass inputs with resolver provenance", () => {
    const targetPath = "/workspace/src/Button.module.scss";
    const targetSource = `
@use "$shared/theme";
@forward "./mixins";
@value primary from "./tokens.module.css";
.button { composes: base from "./Base.module.scss"; }
`;
    const sources = new Map<string, string>([
      [targetPath, targetSource],
      ["/workspace/src/Base.module.scss", ".base { color: red; }"],
      ["/workspace/src/tokens.module.css", "@value primary: red;"],
      ["/workspace/src/shared/_theme.scss", "$tone: red;"],
      ["/workspace/src/_mixins.scss", "@mixin focus { outline: none; }"],
      ["/workspace/package.json", '{"name":"fixture"}'],
    ]);
    const openBaseSource = ".base { color: blue; }";
    const styleDocumentForPath = (filePath: string) => {
      const source =
        filePath === "/workspace/src/Base.module.scss"
          ? openBaseSource
          : (sources.get(filePath) ?? null);
      return source === null ? null : parseStyleDocument(source, filePath);
    };

    const result = collectSelectedQueryWorkspaceInputs(
      [
        {
          stylePath: targetPath,
          styleSource: targetSource,
          styleDocument: parseStyleDocument(targetSource, targetPath),
        },
      ],
      {
        aliasResolver: new AliasResolver("/workspace", { $shared: "src/shared" }),
        buildStyleDocument: (filePath, content) => parseStyleDocument(content, filePath),
        readOpenDocumentText: (filePath) =>
          filePath === "/workspace/src/Base.module.scss" ? openBaseSource : null,
        readStyleFile: (filePath) => {
          if (filePath.endsWith("package.json")) {
            throw new Error("package manifests must not use the style-file reader");
          }
          return sources.get(filePath) ?? null;
        },
        readWorkspaceFile: (filePath) => sources.get(filePath) ?? null,
        styleDocumentForPath,
        workspaceRoot: "/workspace",
      },
      targetPath,
    );

    expect(result.styles.map((style) => style.stylePath).toSorted()).toEqual([
      "/workspace/src/Base.module.scss",
      "/workspace/src/Button.module.scss",
      "/workspace/src/_mixins.scss",
      "/workspace/src/shared/_theme.scss",
      "/workspace/src/tokens.module.css",
    ]);
    expect(
      result.styles.find((style) => style.stylePath.endsWith("Base.module.scss"))?.styleSource,
    ).toBe(openBaseSource);
    expect(result.packageManifests).toEqual([
      {
        packageJsonPath: "/workspace/package.json",
        packageJsonSource: '{"name":"fixture"}',
      },
    ]);
    expect(result.resolutionInputs).toMatchObject({
      packageManifests: result.packageManifests,
      bundlerPathMappings: [
        {
          pattern: "$shared",
          targetPath: "/workspace/src/shared",
        },
      ],
    });
  });
});
