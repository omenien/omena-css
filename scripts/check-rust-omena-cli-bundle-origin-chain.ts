import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface SourceMapV3 {
  readonly file: string;
  readonly sources: readonly string[];
  readonly sourcesContent: readonly string[];
  readonly mappings: string;
  readonly x_omenaSegmentCount: number;
  readonly x_omenaPassIds: readonly string[];
}

interface ConsumerBuildSummary {
  readonly execution: {
    readonly outputCss: string;
  };
  readonly sourceMapV3?: SourceMapV3;
  readonly readySurfaces: readonly string[];
}

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-bundle-origin-"));

try {
  const themeDir = join(workspace, "theme");
  const assetDir = join(workspace, "assets");
  const tokenAssetDir = join(themeDir, "icons");
  const splitDir = join(workspace, "split");
  mkdirSync(themeDir, { recursive: true });
  mkdirSync(assetDir, { recursive: true });
  mkdirSync(tokenAssetDir, { recursive: true });
  mkdirSync(splitDir, { recursive: true });

  const appPath = join(workspace, "app.css");
  const tokensPath = join(themeDir, "tokens.css");
  const basePath = join(themeDir, "base.css");
  const appAssetPath = join(assetDir, "app.svg");
  const tokenAssetPath = join(tokenAssetDir, "token.svg");
  const appSource =
    '@import "./theme/tokens.css"; .app { color: green; background: url("./assets/app.svg"); }';
  const tokensSource =
    '@import "./base.css"; .token { color: blue; background-image: url("./icons/token.svg"); }';
  const baseSource = ".base { color: red; }";

  writeFileSync(appPath, appSource);
  writeFileSync(tokensPath, tokensSource);
  writeFileSync(basePath, baseSource);
  writeFileSync(appAssetPath, "<svg />");
  writeFileSync(tokenAssetPath, "<svg />");

  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--bin",
      "omena-cli",
      "--",
      "build",
      appPath,
      "--bundle",
      "--source",
      tokensPath,
      "--source",
      basePath,
      "--source-map",
      "--split-out-dir",
      splitDir,
      "--json",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );

  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `omena build --bundle --source-map failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );

  const summary = JSON.parse(result.stdout) as ConsumerBuildSummary;
  const sourceMap = summary.sourceMapV3;
  assert.ok(sourceMap, "bundle build should include Source Map V3 output");
  assert.ok(summary.readySurfaces.includes("bundleBuildMode"));
  assert.ok(summary.readySurfaces.includes("bundleAssetUrlRewrite"));
  assert.ok(summary.readySurfaces.includes("sourceMapV3Serializer"));
  assert.ok(summary.readySurfaces.includes("bundleSourceMapOriginChain"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitEmission"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitSourceMapEmission"));
  assert.ok(!summary.execution.outputCss.includes("@import"));
  assert.ok(summary.execution.outputCss.includes(baseSource));
  assert.ok(summary.execution.outputCss.includes(".token { color: blue;"));
  assert.ok(summary.execution.outputCss.includes(".app { color: green;"));
  assert.ok(summary.execution.outputCss.includes(`url("${appAssetPath}")`));
  assert.ok(summary.execution.outputCss.includes(`url("${tokenAssetPath}")`));
  assert.ok(!summary.execution.outputCss.includes("./assets/app.svg"));
  assert.ok(!summary.execution.outputCss.includes("./icons/token.svg"));
  assert.ok(sourceMap.mappings.length > 0);
  assert.ok(sourceMap.x_omenaSegmentCount >= 3);
  assert.ok(sourceMap.x_omenaPassIds.includes("import-inline"));

  assertSourceContent(sourceMap, appPath, appSource);
  assertSourceContent(sourceMap, tokensPath, tokensSource);
  assertSourceContent(sourceMap, basePath, baseSource);
  assertSplitOutputs(splitDir, appSource, tokensSource, baseSource);

  console.log(
    [
      "validated omena-cli bundle origin chain:",
      `sources=${sourceMap.sources.length}`,
      `segments=${sourceMap.x_omenaSegmentCount}`,
      "ready=bundleSourceMapOriginChain+bundleAssetUrlRewrite+bundleCodeSplitEmission+bundleCodeSplitSourceMapEmission",
    ].join(" "),
  );
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function assertSourceContent(sourceMap: SourceMapV3, sourcePath: string, source: string): void {
  const sourceIndex = sourceMap.sources.indexOf(sourcePath);
  assert.notEqual(sourceIndex, -1, `source map should include ${sourcePath}`);
  assert.equal(
    sourceMap.sourcesContent[sourceIndex],
    source,
    `source map should preserve sourcesContent for ${sourcePath}`,
  );
}

function assertSplitOutputs(
  splitDir: string,
  appSource: string,
  tokensSource: string,
  baseSource: string,
): void {
  const files = readdirSync(splitDir);
  const cssFiles = files.filter((file) => file.endsWith(".css"));
  const mapFiles = files.filter((file) => file.endsWith(".css.map"));
  assert.equal(
    cssFiles.length,
    3,
    `expected entry plus two imported split files: ${files.join(",")}`,
  );
  assert.equal(
    mapFiles.length,
    3,
    `expected source-map sidecars for every split CSS file: ${files.join(",")}`,
  );
  const outputs = cssFiles.map((file) => readFileSync(join(splitDir, file), "utf8"));
  const appOutput = outputs.find((output) => output.includes(".app { color: green;"));
  const tokensOutput = outputs.find((output) => output.includes(".token { color: blue;"));
  const baseOutput = outputs.find((output) => output.includes(baseSource));
  assert.ok(appOutput, "split outputs should include the entry CSS file");
  assert.ok(tokensOutput, "split outputs should include the imported token CSS file");
  assert.ok(baseOutput, "split outputs should include the transitive base CSS file");
  assert.notEqual(appOutput, appSource, "entry split output should rewrite its import specifier");
  assert.notEqual(
    tokensOutput,
    tokensSource,
    "imported split output should rewrite its transitive import specifier",
  );
  assert.ok(!appOutput.includes("./theme/tokens.css"));
  assert.ok(!tokensOutput.includes("./base.css"));
  assert.ok(appOutput.includes('@import "'));
  assert.ok(tokensOutput.includes('@import "'));
  for (const cssFile of cssFiles) {
    const output = readFileSync(join(splitDir, cssFile), "utf8");
    const sourceMapFile = `${cssFile}.map`;
    assert.ok(
      output.includes(`sourceMappingURL=${sourceMapFile}`),
      `split CSS should point at its map sidecar: ${cssFile}`,
    );
    assert.ok(
      mapFiles.includes(sourceMapFile),
      `split source map sidecar should exist for ${cssFile}: ${files.join(",")}`,
    );
    const splitMap = JSON.parse(readFileSync(join(splitDir, sourceMapFile), "utf8")) as SourceMapV3;
    assert.equal(splitMap.file, cssFile);
    assert.equal(splitMap.sources.length, 1);
    assert.equal(splitMap.sourcesContent.length, 1);
    assert.ok(splitMap.mappings.length > 0);
    assert.ok(splitMap.x_omenaPassIds.includes("code-split-emission"));
    assert.ok(
      [appSource, tokensSource, baseSource].includes(splitMap.sourcesContent[0]),
      `split map should preserve original source content for ${cssFile}`,
    );
  }
}
