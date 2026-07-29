import react from "@vitejs/plugin-react";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import mdx from "fumadocs-mdx/vite";
import { readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";

const deploymentBasePath = process.env.OMENA_DOCS_BASE_PATH ?? "";
const viteBase = deploymentBasePath ? `${deploymentBasePath}/` : "/";
const applicationRoot = path.dirname(fileURLToPath(import.meta.url));
const documentationRoot = path.resolve(applicationRoot, "../../docs");
const documentationPages = collectDocumentationPages(documentationRoot);

export default defineConfig({
  base: viteBase,
  server: {
    port: 3000,
  },
  plugins: [
    mdx(),
    tailwindcss(),
    tanstackStart({
      spa: {
        enabled: true,
      },
      prerender: {
        enabled: true,
        crawlLinks: true,
        failOnError: true,
      },
      pages: [
        prerenderPage("/"),
        ...documentationPages.map(prerenderPage),
        prerenderPage("/api/search"),
      ],
    }),
    react(),
    nitro(),
  ],
  resolve: {
    tsconfigPaths: true,
    alias: {
      tslib: "tslib/tslib.es6.js",
    },
  },
});

function withDeploymentBase(route: string): string {
  if (!deploymentBasePath) return route;
  return route === "/" ? `${deploymentBasePath}/` : `${deploymentBasePath}${route}`;
}

function prerenderPage(route: string) {
  return {
    path: withDeploymentBase(route),
    prerender: {
      outputPath: route,
    },
  };
}

function collectDocumentationPages(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) return collectDocumentationPages(absolutePath);
      if (!/\.(?:md|mdx)$/u.test(entry.name)) return [];

      const relativePath = path.relative(documentationRoot, absolutePath).replaceAll(path.sep, "/");
      const slug = relativePath.replace(/\.(?:md|mdx)$/u, "");
      return [slug === "index" ? "/docs" : `/docs/${slug}`];
    })
    .toSorted();
}
