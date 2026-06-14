import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import postcss from "postcss";
import scssSyntax from "postcss-scss";

const require = createRequire(import.meta.url);
const { omenaPostcss } = require("../packages/postcss-plugin/index.cjs");

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-postcss-plugin-"));
const stylePath = path.join(tempRoot, "App.module.css");

try {
  const pluginSource = fs.readFileSync(
    path.join(process.cwd(), "packages/postcss-plugin/index.cjs"),
    "utf8",
  );
  const adapterSource = fs.readFileSync(
    path.join(process.cwd(), "packages/css-build-adapter/index.cjs"),
    "utf8",
  );
  assert.equal(
    pluginSource.includes("handleHotUpdate") || pluginSource.includes("createServer"),
    false,
    "PostCSS plugin must not carry Vite HMR/server hooks",
  );
  assert.equal(
    `${pluginSource}\n${adapterSource}`.includes("execFileSync") ||
      `${pluginSource}\n${adapterSource}`.includes("cargo run"),
    false,
    "PostCSS plugin path must not use CLI/cargo fallback",
  );
  assert.equal(
    adapterSource.includes("buildStyleSourcesWithContextJson"),
    true,
    "PostCSS plugin path should reuse the in-process native build adapter",
  );

  fs.writeFileSync(stylePath, ".root {\n  color: red;\n}\n/* remove me */\n", "utf8");
  const canonicalStylePath = fs.realpathSync.native(stylePath);
  const result = await postcss([
    omenaPostcss({
      passes: ["comment-strip", "whitespace-strip"],
      cwd: tempRoot,
      configFile: false,
    }),
  ]).process(fs.readFileSync(stylePath, "utf8"), {
    from: stylePath,
    to: path.join(tempRoot, "dist", "App.module.css"),
    map: {
      inline: false,
      annotation: false,
    },
  });

  assert.equal(result.css.includes("remove me"), false, "comment-strip pass should run");
  assert.equal(result.css.includes(".root"), true, "selector should be preserved");
  const omenaMessage = result.messages.find((message) => message.type === "omena-css");
  assert.ok(omenaMessage, "PostCSS result should expose an Omena build message");
  assert.equal(
    omenaMessage.sourceMapSources.includes(canonicalStylePath),
    true,
    `Omena build message should retain provenance source, got ${JSON.stringify(
      omenaMessage.sourceMapSources,
    )}`,
  );
  const sourceMap = result.map?.toJSON();
  assert.equal(sourceMap?.version, 3, "PostCSS result should emit Source Map V3");
  assert.ok(
    sourceMap.sources.some((source) => source.endsWith("App.module.css")),
    `PostCSS result source map should include the original module path, got ${JSON.stringify(
      sourceMap.sources,
    )}`,
  );

  const scssPath = path.join(tempRoot, "Tokens.module.scss");
  fs.writeFileSync(scssPath, "$brand: blue;\n.token { color: $brand; }\n", "utf8");
  const scssResult = await postcss([
    omenaPostcss({
      passes: ["scss-module-evaluate", "comment-strip"],
      cwd: tempRoot,
      configFile: false,
    }),
  ]).process(fs.readFileSync(scssPath, "utf8"), {
    from: scssPath,
    to: path.join(tempRoot, "dist", "Tokens.module.css"),
    syntax: scssSyntax,
    map: {
      inline: false,
      annotation: false,
    },
  });
  assert.equal(scssResult.css.includes("$brand"), false, "SCSS module variables should evaluate");
  assert.equal(
    scssResult.css.includes("blue"),
    true,
    "SCSS module output should keep evaluated value",
  );
  assert.ok(
    scssResult.messages.some((message) => message.type === "omena-css"),
    "SCSS PostCSS run should expose an Omena build message",
  );

  const upstreamSourcePath = path.join(tempRoot, "UpstreamSource.module.scss");
  const intermediatePath = path.join(tempRoot, "Intermediate.module.scss");
  fs.writeFileSync(upstreamSourcePath, "$brand: green;\n.upstream { color: $brand; }\n", "utf8");
  const upstreamResult = await postcss([]).process(fs.readFileSync(upstreamSourcePath, "utf8"), {
    from: upstreamSourcePath,
    to: intermediatePath,
    syntax: scssSyntax,
    map: {
      inline: false,
      annotation: false,
    },
  });
  const composedResult = await postcss([
    omenaPostcss({
      passes: ["scss-module-evaluate"],
      cwd: tempRoot,
      configFile: false,
    }),
  ]).process(upstreamResult.css, {
    from: intermediatePath,
    to: path.join(tempRoot, "dist", "Composed.module.css"),
    syntax: scssSyntax,
    map: {
      prev: upstreamResult.map.toJSON(),
      inline: false,
      annotation: false,
    },
  });
  const composedMessage = composedResult.messages.find((message) => message.type === "omena-css");
  assert.equal(
    composedMessage?.upstreamMapApplied,
    true,
    `PostCSS previous source map should compose into Omena map, got ${JSON.stringify(
      composedMessage,
    )}`,
  );
  const composedMap = composedResult.map?.toJSON();
  assert.ok(
    composedMap.sources.some((source) => source.endsWith("UpstreamSource.module.scss")),
    `composed map should point back to the upstream source, got ${JSON.stringify(
      composedMap.sources,
    )}`,
  );
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
