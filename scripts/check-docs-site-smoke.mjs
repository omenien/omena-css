import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const docsRoot = path.join(repoRoot, "docs");
const outputRoot = path.join(repoRoot, "apps/docs/.output/public");
const deploymentBasePath = normalizeBasePath(process.env.OMENA_DOCS_BASE_PATH ?? "");
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
const robotsPath = path.join(outputRoot, "robots.txt");
const sitemapPath = path.join(outputRoot, "sitemap.xml");
const llmsPath = path.join(outputRoot, "llms.txt");
for (const [label, absolutePath, minimumSize] of [
  ["root documentation index", rootIndexPath, 100],
  ["SPA fallback", fallbackPath, 100],
  ["static search index", searchIndexPath, 100],
  ["robots directives", robotsPath, 20],
  ["XML sitemap", sitemapPath, 100],
  ["LLM discovery summary", llmsPath, 100],
]) {
  assert.ok(existsSync(absolutePath), `${label} is missing`);
  assert.ok(statSync(absolutePath).size > minimumSize, `${label} is unexpectedly empty`);
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
  home.includes('<link rel="canonical" href="https://omena.dev/"/>'),
  "the root route is missing its canonical URL",
);
assert.ok(home.includes("GTM-P76BX94H"), "the root route is missing Google Tag Manager");
assert.ok(
  home.includes("https://www.googletagmanager.com/ns.html?id=GTM-P76BX94H"),
  "the root route is missing the Google Tag Manager noscript fallback",
);
assert.ok(
  documentationHome.includes('<link rel="canonical" href="https://omena.dev/"/>'),
  "the duplicate documentation index does not point to the canonical root URL",
);
assert.ok(
  !home.includes('"/docs/$":'),
  "the root document contains hydration state for the catch-all documentation route",
);
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
assert.ok(
  gettingStarted.includes('<link rel="canonical" href="https://omena.dev/docs/getting-started/"/>'),
  "getting-started is missing its canonical URL",
);

const robots = readFileSync(robotsPath, "utf8");
assert.ok(robots.includes("User-agent: *"), "robots.txt does not address all crawlers");
assert.ok(robots.includes("Allow: /"), "robots.txt does not allow the documentation site");
assert.ok(
  robots.includes("Sitemap: https://omena.dev/sitemap.xml"),
  "robots.txt does not advertise the sitemap",
);

const sitemap = readFileSync(sitemapPath, "utf8");
const sitemapUrls = [...sitemap.matchAll(/<loc>([^<]+)<\/loc>/gu)].map((match) => match[1]);
assert.equal(
  sitemapUrls.length,
  expectedDocumentationOutputs.length,
  "the sitemap must contain one canonical URL per documentation source",
);
assert.ok(sitemapUrls.includes("https://omena.dev/"), "the sitemap is missing the canonical root");
assert.ok(
  !sitemapUrls.includes("https://omena.dev/docs/"),
  "the sitemap includes the duplicate documentation index",
);

const assetFiles = collect(path.join(outputRoot, "assets"), /./u);
assert.ok(assetFiles.length > 0, "TanStack Start static assets are missing");
const staticFunctionCacheFiles = collect(
  path.join(outputRoot, "__tsr/staticServerFnCache"),
  /\.json$/u,
);
assert.ok(staticFunctionCacheFiles.length > 0, "static server function cache is missing");

const staticFunctionClientAssets = assetFiles.filter((assetPath) => {
  if (!assetPath.endsWith(".js")) return false;
  return readFileSync(assetPath, "utf8").includes("/__tsr/staticServerFnCache/");
});
assert.ok(
  staticFunctionClientAssets.length > 0,
  "client assets do not contain the static server function cache route",
);

if (deploymentBasePath) {
  for (const assetPath of staticFunctionClientAssets) {
    const clientSource = readFileSync(assetPath, "utf8");
    const cacheRouteIndex = clientSource.indexOf("/__tsr/staticServerFnCache/");
    const cacheRoutingContext = clientSource.slice(
      Math.max(0, cacheRouteIndex - 500),
      cacheRouteIndex + 500,
    );
    assert.ok(
      cacheRoutingContext.includes(deploymentBasePath.replace(/^\//u, "")),
      `${path.relative(outputRoot, assetPath)} does not apply the deployment base path to the static server function cache`,
    );
  }

  assert.equal(
    collect(path.join(outputRoot, deploymentBasePath, "__tsr/staticServerFnCache"), /\.json$/u)
      .length,
    0,
    "static server function cache was written into a duplicated deployment base directory",
  );
}

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
      staticFunctionCacheFileCount: staticFunctionCacheFiles.length,
      staticFunctionClientAssetCount: staticFunctionClientAssets.length,
      missingPageCount: 0,
      orphanPageCount: 0,
      playgroundWorkflowCount: 3,
      sitemapUrlCount: sitemapUrls.length,
      robotsAllowsCrawling: true,
      structuredDataPageCount: actualDocumentationOutputs.length + 1,
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
  assert.match(
    html,
    /<meta\b[^>]*property="og:title"/u,
    `${relativePath} is missing its Open Graph title`,
  );
  assert.match(
    html,
    /<script\b[^>]*type="application\/ld\+json"/u,
    `${relativePath} is missing JSON-LD structured data`,
  );
}

function documentationOutputForSource(sourcePath) {
  const relativePath = path.relative(docsRoot, sourcePath).replaceAll(path.sep, "/");
  const slug = relativePath.replace(/\.(?:md|mdx)$/u, "");
  return slug === "index" ? "docs/index.html" : `docs/${slug}/index.html`;
}

function normalizeBasePath(basePath) {
  if (!basePath || basePath === "/") return "";
  return `/${basePath.replace(/^\/+|\/+$/gu, "")}`;
}

function collect(directory, filePattern) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) return collect(absolutePath, filePattern);
    return filePattern.test(absolutePath.replaceAll(path.sep, "/")) ? [absolutePath] : [];
  });
}
