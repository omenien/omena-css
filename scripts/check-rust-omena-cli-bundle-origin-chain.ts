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

interface BundleSplitManifestImport {
  readonly importSource: string;
  readonly resolvedStylePath: string;
  readonly fileName: string;
}

interface BundleSplitManifestOutput {
  readonly sourcePath: string;
  readonly fileName: string;
  readonly isEntry: boolean;
  readonly splitBoundary: string;
  readonly sourceMapFile: string | null;
  readonly imports: readonly BundleSplitManifestImport[];
}

interface BundleSplitManifest {
  readonly schemaVersion: number;
  readonly product: string;
  readonly entryStylePath: string;
  readonly entryFile: string;
  readonly outputCount: number;
  readonly outputs: readonly BundleSplitManifestOutput[];
}

const splitManifestFileName = "omena.bundle-split.manifest.json";
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
  const appScssPath = join(workspace, "app.scss");
  const appSourceMapPath = join(workspace, "app.css.map");
  const adminPath = join(workspace, "admin.css");
  const tokensPath = join(themeDir, "tokens.css");
  const basePath = join(themeDir, "base.css");
  const contextPath = join(workspace, "context.json");
  const appAssetPath = join(assetDir, "app.svg");
  const tokenAssetPath = join(tokenAssetDir, "token.svg");
  const appSource =
    '@import "./theme/tokens.css"; .app { color: green; background: url("./assets/app.svg"); } .deadApp { color: red; }';
  const appScssSource = "$brand: green;\n.app { color: $brand; }\n";
  const adminSource =
    '@import "./theme/tokens.css"; .admin { color: green; } .deadAdmin { color: red; }';
  const tokensSource =
    '@import "./base.css"; .token { color: blue; background-image: url("./icons/token.svg"); } .deadToken { color: red; }';
  const baseSource = ".base { color: red; } .deadBase { color: gray; }";
  const appSourceMap = {
    version: 3,
    file: "app.css",
    sources: [appScssPath],
    sourcesContent: [appScssSource],
    names: [],
    mappings: "AAAA",
  };

  writeFileSync(appPath, appSource);
  writeFileSync(appScssPath, appScssSource);
  writeFileSync(appSourceMapPath, JSON.stringify(appSourceMap, null, 2));
  writeFileSync(adminPath, adminSource);
  writeFileSync(tokensPath, tokensSource);
  writeFileSync(basePath, baseSource);
  writeFileSync(
    contextPath,
    JSON.stringify({ reachableClassNames: ["app", "admin", "token", "base"] }),
  );
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
      adminPath,
      "--source",
      tokensPath,
      "--source",
      basePath,
      "--source-map",
      "--input-source-map",
      `${appPath}=${appSourceMapPath}`,
      "--split-out-dir",
      splitDir,
      "--bundle-entry",
      adminPath,
      "--tree-shake",
      "--context-json",
      contextPath,
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
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitManifestEmission"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitBoundaryManifest"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitEntryConfig"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitSharedChunkEmission"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitTreeShakeEmission"));
  assert.ok(summary.readySurfaces.includes("bundleCodeSplitSourceMapEmission"));
  assert.ok(summary.readySurfaces.includes("bundleUpstreamSourceMapComposition"));
  assert.ok(!summary.execution.outputCss.includes("@import"));
  assert.ok(summary.execution.outputCss.includes(".base { color: red;"));
  assert.ok(summary.execution.outputCss.includes(".token { color: blue;"));
  assert.ok(summary.execution.outputCss.includes(".app { color: green;"));
  assert.ok(!summary.execution.outputCss.includes(".deadApp"));
  assert.ok(!summary.execution.outputCss.includes(".deadToken"));
  assert.ok(!summary.execution.outputCss.includes(".deadBase"));
  assert.ok(summary.execution.outputCss.includes(`url("${appAssetPath}")`));
  assert.ok(summary.execution.outputCss.includes(`url("${tokenAssetPath}")`));
  assert.ok(!summary.execution.outputCss.includes("./assets/app.svg"));
  assert.ok(!summary.execution.outputCss.includes("./icons/token.svg"));
  assert.ok(sourceMap.mappings.length > 0);
  assert.ok(sourceMap.x_omenaSegmentCount >= 3);
  assert.ok(sourceMap.x_omenaPassIds.includes("import-inline"));

  assertSourceContent(sourceMap, appScssPath, appScssSource);
  assertSourceContent(sourceMap, tokensPath, tokensSource);
  assertSourceContent(sourceMap, basePath, baseSource);
  assertSplitOutputs(splitDir, appScssSource, adminSource, tokensSource, baseSource);

  console.log(
    [
      "validated omena-cli bundle origin chain:",
      `sources=${sourceMap.sources.length}`,
      `segments=${sourceMap.x_omenaSegmentCount}`,
      "ready=bundleSourceMapOriginChain+bundleAssetUrlRewrite+bundleCodeSplitEmission+bundleCodeSplitManifestEmission+bundleCodeSplitBoundaryManifest+bundleCodeSplitEntryConfig+bundleCodeSplitSharedChunkEmission+bundleCodeSplitTreeShakeEmission+bundleCodeSplitSourceMapEmission+bundleUpstreamSourceMapComposition",
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
  adminSource: string,
  tokensSource: string,
  baseSource: string,
): void {
  const files = readdirSync(splitDir);
  const cssFiles = files.filter((file) => file.endsWith(".css"));
  const mapFiles = files.filter((file) => file.endsWith(".css.map"));
  const manifest = JSON.parse(
    readFileSync(join(splitDir, splitManifestFileName), "utf8"),
  ) as BundleSplitManifest;
  assert.equal(
    cssFiles.length,
    4,
    `expected two entries plus two shared imported split files: ${files.join(",")}`,
  );
  assert.equal(
    mapFiles.length,
    4,
    `expected source-map sidecars for every split CSS file: ${files.join(",")}`,
  );
  const outputs = cssFiles.map((file) => readFileSync(join(splitDir, file), "utf8"));
  const appOutput = outputs.find((output) => output.includes(".app { color: green;"));
  const tokensOutput = outputs.find((output) => output.includes(".token { color: blue;"));
  const baseOutput = outputs.find((output) => output.includes(".base { color: red;"));
  assert.ok(appOutput, "split outputs should include the entry CSS file");
  assert.ok(tokensOutput, "split outputs should include the imported token CSS file");
  assert.ok(baseOutput, "split outputs should include the transitive base CSS file");
  assert.ok(!appOutput.includes(".deadApp"));
  assert.ok(!tokensOutput.includes(".deadToken"));
  assert.ok(!baseOutput.includes(".deadBase"));
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
  assert.equal(manifest.product, "omena-cli.bundle-code-split-manifest");
  assert.equal(manifest.schemaVersion, 0);
  assert.equal(manifest.outputCount, 4);
  assert.equal(manifest.outputs.length, 4);
  assert.ok(cssFiles.includes(manifest.entryFile), "split manifest entry file should be emitted");
  const manifestFiles = new Set(manifest.outputs.map((output) => output.fileName));
  assert.deepEqual(
    [...manifestFiles].sort(),
    [...cssFiles].sort(),
    "split manifest should describe every emitted CSS split file",
  );
  assert.ok(
    manifest.outputs.every((output) => !("sources" in output) && !("sourcesContent" in output)),
    "split manifest should not duplicate composed original sources; sidecar .map files are the origin authority",
  );
  const appManifest = manifest.outputs.find((output) => output.sourcePath.endsWith("/app.css"));
  const adminManifest = manifest.outputs.find((output) => output.sourcePath.endsWith("/admin.css"));
  const tokensManifest = manifest.outputs.find((output) =>
    output.sourcePath.endsWith("/theme/tokens.css"),
  );
  const baseManifest = manifest.outputs.find((output) =>
    output.sourcePath.endsWith("/theme/base.css"),
  );
  assert.ok(appManifest, "split manifest should describe the entry file");
  assert.ok(adminManifest, "split manifest should describe the configured entry file");
  assert.ok(tokensManifest, "split manifest should describe the token import file");
  assert.ok(baseManifest, "split manifest should describe the base transitive import file");
  assert.equal(appManifest.isEntry, true);
  assert.equal(adminManifest.isEntry, true);
  assert.equal(tokensManifest.isEntry, false);
  assert.equal(baseManifest.isEntry, false);
  assert.equal(appManifest.splitBoundary, "entry");
  assert.equal(adminManifest.splitBoundary, "entryConfig");
  assert.equal(tokensManifest.splitBoundary, "shared");
  assert.equal(baseManifest.splitBoundary, "shared");
  assert.equal(appManifest.sourceMapFile, `${appManifest.fileName}.map`);
  assert.equal(adminManifest.sourceMapFile, `${adminManifest.fileName}.map`);
  assert.equal(tokensManifest.sourceMapFile, `${tokensManifest.fileName}.map`);
  assert.equal(baseManifest.sourceMapFile, `${baseManifest.fileName}.map`);
  assert.equal(appManifest.imports.length, 1);
  assert.equal(appManifest.imports[0].importSource, "./theme/tokens.css");
  assert.equal(appManifest.imports[0].fileName, tokensManifest.fileName);
  assert.equal(appManifest.imports[0].resolvedStylePath, tokensManifest.sourcePath);
  assert.equal(adminManifest.imports.length, 1);
  assert.equal(adminManifest.imports[0].importSource, "./theme/tokens.css");
  assert.equal(adminManifest.imports[0].fileName, tokensManifest.fileName);
  assert.equal(adminManifest.imports[0].resolvedStylePath, tokensManifest.sourcePath);
  assert.equal(tokensManifest.imports.length, 1);
  assert.equal(tokensManifest.imports[0].importSource, "./base.css");
  assert.equal(tokensManifest.imports[0].fileName, baseManifest.fileName);
  assert.equal(tokensManifest.imports[0].resolvedStylePath, baseManifest.sourcePath);
  assert.equal(baseManifest.imports.length, 0);
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
      [appSource, adminSource, tokensSource, baseSource].includes(splitMap.sourcesContent[0]),
      `split map should preserve original source content for ${cssFile}`,
    );
  }
}
