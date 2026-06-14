import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

const require = createRequire(import.meta.url);
const repoRoot = process.cwd();
const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "acme-postcss-consumer-"));
const srcRoot = path.join(tempRoot, "src");
const distRoot = path.join(tempRoot, "dist");
const stylePath = path.join(srcRoot, "App.module.scss");
const intermediatePath = path.join(srcRoot, "Intermediate.module.scss");
const pluginPath = path.join(repoRoot, "packages/postcss-plugin/index.cjs");
const postcssPath = require.resolve("postcss");
const postcssScssPath = require.resolve("postcss-scss");

try {
  fs.mkdirSync(srcRoot, { recursive: true });
  fs.writeFileSync(
    path.join(tempRoot, "package.json"),
    JSON.stringify(
      {
        name: "acme-postcss-consumer",
        private: true,
        scripts: {
          build: "NODE_ENV=production node ./build.mjs",
        },
      },
      null,
      2,
    ),
  );
  fs.writeFileSync(
    stylePath,
    "$brand: blue;\n.button {\n  color: $brand;\n}\n/* prod remove */\n",
    "utf8",
  );
  fs.writeFileSync(
    path.join(tempRoot, "build.mjs"),
    [
      `import fs from "node:fs";`,
      `import path from "node:path";`,
      `import { createRequire } from "node:module";`,
      `const require = createRequire(import.meta.url);`,
      `const postcss = require(${JSON.stringify(postcssPath)});`,
      `const scssSyntax = require(${JSON.stringify(postcssScssPath)});`,
      `const { omenaPostcss } = require(${JSON.stringify(pluginPath)});`,
      `const root = ${JSON.stringify(tempRoot)};`,
      `const src = ${JSON.stringify(stylePath)};`,
      `const intermediate = ${JSON.stringify(intermediatePath)};`,
      `const dist = path.join(root, "dist");`,
      `fs.mkdirSync(dist, { recursive: true });`,
      `const upstream = await postcss([]).process(fs.readFileSync(src, "utf8"), { from: src, to: intermediate, syntax: scssSyntax, map: { inline: false, annotation: false } });`,
      `const result = await postcss([omenaPostcss({ passes: ["scss-module-evaluate", "comment-strip", "whitespace-strip"], cwd: root, configFile: false })]).process(upstream.css, { from: intermediate, to: path.join(dist, "app.css"), syntax: scssSyntax, map: { prev: upstream.map.toJSON(), inline: false, annotation: false } });`,
      `fs.writeFileSync(path.join(dist, "app.css"), result.css);`,
      `fs.writeFileSync(path.join(dist, "app.css.map"), result.map.toString());`,
      `fs.writeFileSync(path.join(dist, "omena-postcss-summary.json"), JSON.stringify({ packageName: "acme-postcss-consumer", messages: result.messages }, null, 2));`,
    ].join("\n"),
  );

  const build = spawnSync(process.execPath, ["build.mjs"], {
    cwd: tempRoot,
    encoding: "utf8",
    env: { ...process.env, NODE_ENV: "production" },
  });
  assert.equal(
    build.status,
    0,
    `non-omena PostCSS consumer build failed\nstdout:\n${build.stdout}\nstderr:\n${build.stderr}`,
  );

  const packageJson = JSON.parse(fs.readFileSync(path.join(tempRoot, "package.json"), "utf8"));
  assert.equal(
    packageJson.name.includes("omena"),
    false,
    "fixture must remain a non-omena consumer",
  );
  const outputCss = fs.readFileSync(path.join(distRoot, "app.css"), "utf8");
  assert.equal(
    outputCss.includes("prod remove"),
    false,
    "production build should run Omena passes",
  );
  assert.equal(
    outputCss.includes("$brand"),
    false,
    "production build should evaluate SCSS variables",
  );
  assert.equal(
    outputCss.includes(".button"),
    true,
    "production build should retain CSS module selector",
  );
  const outputMap = JSON.parse(fs.readFileSync(path.join(distRoot, "app.css.map"), "utf8"));
  assert.equal(outputMap.version, 3, "production build should emit Source Map V3");
  assert.ok(
    outputMap.sources.some((source) => source.endsWith("App.module.scss")),
    `production build source map should retain source provenance, got ${JSON.stringify(
      outputMap.sources,
    )}`,
  );
  const summary = JSON.parse(
    fs.readFileSync(path.join(distRoot, "omena-postcss-summary.json"), "utf8"),
  );
  assert.equal(summary.packageName, "acme-postcss-consumer");
  assert.ok(
    summary.messages.some((message) => message.type === "omena-css"),
    "production build should record an Omena PostCSS message",
  );
  assert.ok(
    summary.messages.some(
      (message) => message.type === "omena-css" && message.upstreamMapApplied === true,
    ),
    "production build should record upstream source-map composition",
  );
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
