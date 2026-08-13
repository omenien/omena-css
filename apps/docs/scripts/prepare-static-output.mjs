import { copyFileSync, existsSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const canonicalOrigin = "https://omena.dev";
const applicationRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const publicOutput = path.join(applicationRoot, ".output/public");
const shellPath = path.join(publicOutput, "_shell.html");
const rootIndexPath = path.join(publicOutput, "index.html");
const documentationIndexPath = path.join(publicOutput, "docs/index.html");

if (!existsSync(shellPath)) {
  throw new Error("TanStack Start did not emit the SPA shell");
}

if (!existsSync(documentationIndexPath)) {
  throw new Error("TanStack Start did not prerender the documentation index");
}

if (!existsSync(rootIndexPath)) {
  throw new Error("TanStack Start did not prerender the root documentation route");
}

copyFileSync(shellPath, path.join(publicOutput, "404.html"));

const publishedUrls = collectPublishedUrls(publicOutput);
writeFileSync(path.join(publicOutput, "sitemap.xml"), createSitemap(publishedUrls));
writeFileSync(
  path.join(publicOutput, "robots.txt"),
  `User-agent: *\nAllow: /\n\nSitemap: ${canonicalOrigin}/sitemap.xml\n`,
);

function collectPublishedUrls(directory) {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) return collectPublishedUrls(absolutePath);
      if (entry.name !== "index.html") return [];

      const relativePath = path.relative(publicOutput, absolutePath).replaceAll(path.sep, "/");
      if (relativePath === "docs/index.html") return [];

      const pathname =
        relativePath === "index.html" ? "/" : `/${relativePath.replace(/index\.html$/u, "")}`;
      return [new URL(pathname, canonicalOrigin).href];
    })
    .toSorted();
}

function createSitemap(urls) {
  const entries = urls.map((url) => `  <url><loc>${escapeXml(url)}</loc></url>`).join("\n");
  return `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${entries}\n</urlset>\n`;
}

function escapeXml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}
