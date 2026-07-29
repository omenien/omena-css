import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const docsRoot = path.join(repoRoot, "docs");
const outputRoot = path.join(repoRoot, "apps/docs/.output/public");
const sourcePages = collect(docsRoot, /\.(?:md|mdx)$/u);
const expectedDocumentationOutputs = sourcePages
  .map((sourcePath) => documentationOutputForSource(sourcePath))
  .toSorted();
const actualDocumentationOutputs = collect(path.join(outputRoot, "docs"), /(?:^|\/)index\.html$/u)
  .map((outputPath) => path.relative(outputRoot, outputPath).replaceAll(path.sep, "/"))
  .toSorted();

assert.deepEqual(
  actualDocumentationOutputs,
  expectedDocumentationOutputs,
  "the prerendered documentation set must exactly match the Markdown source set",
);

for (const relativePath of expectedDocumentationOutputs) {
  const absolutePath = path.join(outputRoot, relativePath);
  assert.ok(statSync(absolutePath).size > 500, `${relativePath} is unexpectedly empty`);
  assertDocumentHtml(readFileSync(absolutePath, "utf8"), relativePath);
}

const rootIndexPath = path.join(outputRoot, "index.html");
const fallbackPath = path.join(outputRoot, "404.html");
const searchIndexPath = path.join(outputRoot, "api/search");
for (const [label, absolutePath] of [
  ["root documentation index", rootIndexPath],
  ["SPA fallback", fallbackPath],
  ["static search index", searchIndexPath],
]) {
  assert.ok(existsSync(absolutePath), `${label} is missing`);
  assert.ok(statSync(absolutePath).size > 100, `${label} is unexpectedly empty`);
}

const home = readFileSync(rootIndexPath, "utf8");
const documentationHome = readFileSync(path.join(outputRoot, "docs/index.html"), "utf8");
const playground = readFileSync(path.join(outputRoot, "docs/playground/index.html"), "utf8");
const gettingStarted = readFileSync(
  path.join(outputRoot, "docs/getting-started/index.html"),
  "utf8",
);
assertDocumentHtml(home, "index.html");
assert.ok(home.includes("Omena documentation"), "the root route is still an empty SPA shell");
assert.ok(
  documentationHome.includes("/docs/playground"),
  "the documentation index does not link to the playground",
);
assert.ok(
  documentationHome.includes("Currently published Omena product surfaces"),
  "the documentation index is missing the publication support matrix",
);
assert.ok(
  !documentationHome.includes("CSS tools should show"),
  "the removed marketing landing page is still present",
);
assert.ok(playground.includes("Runs locally in your browser"), "WASM playground did not render");
assert.ok(playground.includes("No server analysis"), "playground privacy boundary is missing");
for (const workflowHeading of [
  "Follow references across files",
  "Build from workspace semantics",
  "Plan output for a browser target",
]) {
  assert.ok(
    playground.includes(workflowHeading),
    `playground workflow is missing: ${workflowHeading}`,
  );
}
assert.ok(gettingStarted.includes("Install The CLI"), "getting-started content did not render");

const assetFiles = collect(path.join(outputRoot, "assets"), /./u);
assert.ok(assetFiles.length > 0, "TanStack Start static assets are missing");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.site-smoke",
      sourcePageCount: sourcePages.length,
      prerenderedDocumentationPageCount: actualDocumentationOutputs.length,
      rootDocumentPrerendered: true,
      mainLandmarkCount: actualDocumentationOutputs.length + 1,
      skipNavigationCount: actualDocumentationOutputs.length + 1,
      assetFileCount: assetFiles.length,
      missingPageCount: 0,
      orphanPageCount: 0,
      playgroundWorkflowCount: 3,
    },
    null,
    2,
  )}\n`,
);

function assertDocumentHtml(html, relativePath) {
  assert.match(html, /<main\b[^>]*\bid="main-content"/u, `${relativePath} is missing main`);
  assert.match(html, /<article\b/u, `${relativePath} is missing article semantics`);
  assert.match(html, /<h1\b/u, `${relativePath} is missing its page heading`);
  assert.match(
    html,
    /<a\b[^>]*class="docs-skip-link"[^>]*href="#main-content"/u,
    `${relativePath} is missing skip navigation`,
  );
}

function documentationOutputForSource(sourcePath) {
  const relativePath = path.relative(docsRoot, sourcePath).replaceAll(path.sep, "/");
  const slug = relativePath.replace(/\.(?:md|mdx)$/u, "");
  return slug === "index" ? "docs/index.html" : `docs/${slug}/index.html`;
}

function collect(directory, filePattern) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) return collect(absolutePath, filePattern);
    return filePattern.test(absolutePath.replaceAll(path.sep, "/")) ? [absolutePath] : [];
  });
}
