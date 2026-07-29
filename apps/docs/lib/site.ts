export const site = {
  name: "Omena",
  description:
    "Semantic CSS tooling for evidence-aware diagnostics, transformations, modules, and editor workflows.",
  repository: "https://github.com/omenien/omena-css",
  docsBaseUrl: "/docs",
} as const;

export const deploymentBasePath = process.env.NEXT_PUBLIC_OMENA_DOCS_BASE_PATH ?? "";

export function withBasePath(pathname: string): string {
  if (!pathname.startsWith("/")) return pathname;
  return `${deploymentBasePath}${pathname}`;
}
