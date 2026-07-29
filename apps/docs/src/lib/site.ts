export const site = {
  name: "Omena",
  description:
    "Semantic CSS tooling for evidence-aware diagnostics, transformations, modules, and editor workflows.",
  repository: "https://github.com/omenien/omena-css",
  docsBaseUrl: "/docs",
} as const;

const viteBasePath = import.meta.env.BASE_URL ?? "/";

export const deploymentBasePath = viteBasePath === "/" ? "" : viteBasePath.replace(/\/$/u, "");

export function withBasePath(pathname: string): string {
  if (!pathname.startsWith("/")) return pathname;
  return `${deploymentBasePath}${pathname}`;
}
