import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const targets = ["packages/css-build-adapter/index.cjs", "packages/vite-plugin/index.cjs"] as const;
const sources = targets.map((relativePath) => ({
  relativePath,
  source: fs.readFileSync(path.join(repoRoot, relativePath), "utf8"),
}));

for (const { relativePath, source } of sources) {
  assert.ok(
    !source.includes("extractCssModuleClassMap"),
    `${relativePath} must not restore CSS-text class-map extraction`,
  );
  assert.ok(
    !source.includes(".matchAll("),
    `${relativePath} must not derive class maps by scanning emitted CSS`,
  );
}

const adapter = sources.find((entry) => entry.relativePath.includes("css-build-adapter"))!.source;
const vite = sources.find((entry) => entry.relativePath.includes("vite-plugin"))!.source;
assert.ok(adapter.includes("resolveCssModuleInterface(engine"));
assert.ok(adapter.includes("classMap: moduleInterface.classMap"));
assert.ok(vite.includes("classMap: output.classMap"));

process.stdout.write(
  `${JSON.stringify({
    schemaVersion: "0",
    product: "js-bundler-host.no-regex-classmap",
    scannedFiles: targets.length,
    semanticTransportAnchors: 3,
  })}\n`,
);
