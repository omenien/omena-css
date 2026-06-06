import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface SourceMapV3 {
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
  mkdirSync(themeDir, { recursive: true });
  mkdirSync(assetDir, { recursive: true });
  mkdirSync(tokenAssetDir, { recursive: true });

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

  console.log(
    [
      "validated omena-cli bundle origin chain:",
      `sources=${sourceMap.sources.length}`,
      `segments=${sourceMap.x_omenaSegmentCount}`,
      "ready=bundleSourceMapOriginChain+bundleAssetUrlRewrite",
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
