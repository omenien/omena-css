export const site = {
  name: "Omena",
  origin: "https://omena.dev",
  description:
    "Semantic CSS tooling for evidence-aware diagnostics, transformations, modules, and editor workflows.",
  repository: "https://github.com/omenien/omena-css",
  docsBaseUrl: "/docs",
  googleTagManagerId: "GTM-P76BX94H",
} as const;

const viteBasePath = import.meta.env.BASE_URL ?? "/";

export const deploymentBasePath = viteBasePath === "/" ? "" : viteBasePath.replace(/\/$/u, "");

export function withBasePath(pathname: string): string {
  if (!pathname.startsWith("/")) return pathname;
  return `${deploymentBasePath}${pathname}`;
}
