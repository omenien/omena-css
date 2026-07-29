import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";
import type { ComponentProps } from "react";
import { site } from "@/lib/site";
import { SupportMatrix } from "./docs-primitives";
import { WasmPlayground } from "./wasm-playground";

export function createDocumentationLink(sourcePath: string) {
  return function DocumentationLink({ href = "", ...props }: ComponentProps<"a">) {
    const normalizedHref = normalizeDocumentationHref(sourcePath, href);
    const Link = defaultMdxComponents.a;
    return <Link href={normalizedHref} {...props} />;
  };
}

function normalizeDocumentationHref(sourcePath: string, href: string): string {
  if (/^(?:[a-z]+:|#|\/)/iu.test(href)) return href;

  const [pathname, fragment] = href.split("#", 2);
  const segments = sourcePath.split("/").slice(0, -1);
  let outsideDocumentationRoot = false;

  for (const segment of pathname.split("/")) {
    if (!segment || segment === ".") continue;
    if (segment === "..") {
      if (segments.length === 0) outsideDocumentationRoot = true;
      else segments.pop();
      continue;
    }
    segments.push(segment);
  }

  if (outsideDocumentationRoot) {
    const repositoryPath = segments.join("/");
    return `${site.repository}/blob/master/${repositoryPath}${fragment ? `#${fragment}` : ""}`;
  }

  const documentationPath = segments.join("/").replace(/\.(?:md|mdx)$/iu, "");
  const normalizedPath = `/docs/${documentationPath}`.replace(/\/$/u, "");

  return fragment ? `${normalizedPath}#${fragment}` : normalizedPath;
}

export function getMDXComponents(components?: MDXComponents) {
  return {
    ...defaultMdxComponents,
    h1: () => null,
    SupportMatrix,
    WasmPlayground,
    ...components,
  } satisfies MDXComponents;
}

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
