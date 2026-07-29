import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outputRoot = path.join(repoRoot, "apps/docs/out");
const expectedPages = [
  "index.html",
  "404.html",
  "docs/index.html",
  "docs/getting-started/index.html",
  "docs/playground/index.html",
  "docs/reference/README/index.html",
  "api/search",
];

for (const relativePath of expectedPages) {
  const absolutePath = path.join(outputRoot, relativePath);
  assert.ok(existsSync(absolutePath), `static documentation output is missing ${relativePath}`);
  assert.ok(statSync(absolutePath).size > 100, `${relativePath} is unexpectedly empty`);
}

const home = readFileSync(path.join(outputRoot, "index.html"), "utf8");
const playground = readFileSync(path.join(outputRoot, "docs/playground/index.html"), "utf8");
const gettingStarted = readFileSync(
  path.join(outputRoot, "docs/getting-started/index.html"),
  "utf8",
);
assert.ok(home.includes("CSS tools should show"), "home page hero is missing");
assert.ok(home.includes("/docs/playground"), "home page does not link to the playground");
assert.ok(playground.includes("Runs locally in your browser"), "WASM playground did not render");
assert.ok(playground.includes("No server analysis"), "playground privacy boundary is missing");
assert.ok(gettingStarted.includes("Install The CLI"), "getting-started content did not render");

const htmlFiles = collect(outputRoot, ".html");
const assetFiles = collect(path.join(outputRoot, "_next"), "");
assert.ok(htmlFiles.length >= 20, "static export produced too few documentation pages");
assert.ok(assetFiles.length > 0, "Next.js static assets are missing");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.site-smoke",
      htmlPageCount: htmlFiles.length,
      assetFileCount: assetFiles.length,
      expectedPageCount: expectedPages.length,
      missingPageCount: 0,
      homeLinksPlayground: true,
      playgroundPrivacyBoundary: true,
    },
    null,
    2,
  )}\n`,
);

function collect(directory, extension) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) return collect(absolutePath, extension);
    return !extension || entry.name.endsWith(extension) ? [absolutePath] : [];
  });
}
