import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import postcss from "postcss";
import scssSyntax from "postcss-scss";
import { afterEach, describe, expect, it } from "vitest";

type OmenaPostcssExports = {
  readonly omenaPostcss: (options?: Record<string, unknown>) => postcss.AcceptedPlugin;
};

type BuildSource = {
  readonly stylePath: string;
  readonly styleSource: string;
};

const require = createRequire(import.meta.url);
const { omenaPostcss } =
  require("../../../packages/postcss-plugin/index.cjs") as OmenaPostcssExports;
const packageRequire = createRequire(path.join(process.cwd(), "packages/postcss-plugin/index.cjs"));
const { SourceMapGenerator } = packageRequire("source-map-js") as {
  readonly SourceMapGenerator: new (options: { readonly file: string }) => {
    setSourceContent(filePath: string, source: string): void;
    addMapping(mapping: {
      readonly generated: { readonly line: number; readonly column: number };
      readonly original: { readonly line: number; readonly column: number };
      readonly source: string;
    }): void;
    toJSON(): unknown;
  };
};

const tempRoots: string[] = [];

afterEach(() => {
  for (const root of tempRoots.splice(0)) {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

describe("@omena/postcss-plugin", () => {
  it("runs the Once hook through the shared adapter and replaces the PostCSS root", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-postcss-unit-"));
    tempRoots.push(root);
    const stylePath = path.join(root, "Button.module.css");
    const source = ".button {\n  color: red;\n}\n/* strip */\n";
    fs.writeFileSync(stylePath, source, "utf8");
    const calls: unknown[][] = [];
    const engine = {
      buildStyleSourcesWithContextJson: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          execution: {
            outputCss: ".button{color:blue}",
            executedPassIds: ["comment-strip", "whitespace-strip"],
          },
          sourceMapV3: identitySourceMap(stylePath, source),
          readySurfaces: ["sourceMapV3Serializer"],
        });
      },
    };

    const result = await postcss([
      omenaPostcss({
        engine,
        passes: ["comment-strip", "whitespace-strip"],
        cwd: root,
        configFile: false,
      }),
    ]).process(source, {
      from: stylePath,
      to: path.join(root, "dist", "Button.module.css"),
      map: { inline: false, annotation: false },
    });

    expect(result.css).toBe(".button{color:blue}");
    expect(calls).toHaveLength(1);
    const [targetPath, sourcesJson, passIds] = calls[0]!;
    expect(targetPath).toBe(fs.realpathSync.native(stylePath));
    expect(JSON.parse(sourcesJson as string)).toEqual([
      {
        stylePath: fs.realpathSync.native(stylePath),
        styleSource: source,
      },
    ]);
    expect(passIds).toEqual(["comment-strip", "whitespace-strip"]);
    expect(result.root.nodes).toHaveLength(1);
    expect(result.root.first?.toString()).toBe(".button{color:blue}");
    const message = result.messages.find((entry) => entry.type === "omena-css");
    expect(message).toMatchObject({
      type: "omena-css",
      file: fs.realpathSync.native(stylePath),
      upstreamMapApplied: false,
    });
    expect(
      result.map?.toJSON().sources.some((mapSource) => mapSource.endsWith("Button.module.css")),
    ).toBe(true);
  });

  it("composes a previous SCSS source map into the Omena output map", async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "omena-postcss-map-unit-"));
    tempRoots.push(root);
    const upstreamPath = path.join(root, "UpstreamSource.module.scss");
    const intermediatePath = path.join(root, "Intermediate.module.scss");
    const upstreamSource = "$brand: green;\n.upstream { color: $brand; }\n";
    fs.writeFileSync(upstreamPath, upstreamSource, "utf8");

    const upstreamResult = await postcss([]).process(upstreamSource, {
      from: upstreamPath,
      to: intermediatePath,
      syntax: scssSyntax,
      map: { inline: false, annotation: false },
    });
    const intermediateSource = upstreamResult.css;
    const engine = {
      buildStyleSourcesWithContextJson: (_targetPath: string, sourcesJson: string) => {
        const [source] = JSON.parse(sourcesJson) as BuildSource[];
        return JSON.stringify({
          execution: {
            outputCss: ".upstream{color:green}",
            executedPassIds: ["scss-module-evaluate"],
          },
          sourceMapV3: identitySourceMap(source.stylePath, source.styleSource),
          readySurfaces: ["sourceMapV3Serializer"],
        });
      },
    };

    const result = await postcss([
      omenaPostcss({
        engine,
        passes: ["scss-module-evaluate"],
        cwd: root,
        configFile: false,
      }),
    ]).process(intermediateSource, {
      from: intermediatePath,
      to: path.join(root, "dist", "Composed.module.css"),
      syntax: scssSyntax,
      map: {
        prev: upstreamResult.map.toJSON(),
        inline: false,
        annotation: false,
      },
    });

    const message = result.messages.find((entry) => entry.type === "omena-css");
    expect(message).toMatchObject({
      type: "omena-css",
      upstreamMapApplied: true,
    });
    const map = result.map?.toJSON();
    expect(map?.sources.some((source) => source.endsWith("UpstreamSource.module.scss"))).toBe(true);
    expect(map?.sources.some((source) => source.endsWith("Intermediate.module.scss"))).toBe(false);
    expect(result.css).toBe(".upstream{color:green}");
  });
});

function identitySourceMap(filePath: string, source: string) {
  const generator = new SourceMapGenerator({ file: path.basename(filePath) });
  generator.setSourceContent(filePath, source);
  const lines = source.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    generator.addMapping({
      generated: { line: index + 1, column: 0 },
      original: { line: index + 1, column: 0 },
      source: filePath,
    });
  }
  return generator.toJSON();
}
