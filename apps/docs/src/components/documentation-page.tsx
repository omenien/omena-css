import { notFound } from "@tanstack/react-router";
import { createServerFn } from "@tanstack/react-start";
import browserCollections from "collections/browser";
import { useFumadocsLoader } from "fumadocs-core/source/client";
import { staticFunctionMiddleware } from "@tanstack/start-static-server-functions";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  EditOnGitHub,
  useDocsPage,
} from "fumadocs-ui/layouts/docs/page";
import { Suspense, type ComponentProps } from "react";
import { createDocumentationLink, getMDXComponents } from "@/components/mdx";
import { baseOptions } from "@/lib/layout";
import { site } from "@/lib/site";
import { source } from "@/lib/source";

export interface DocumentationPageData {
  path: string;
  pageTree: Awaited<ReturnType<typeof source.serializePageTree>>;
  title: string;
  description: string | undefined;
  canonicalUrl: string;
}

export function createDocumentationHead(data: DocumentationPageData | undefined) {
  if (!data) return { meta: [] };

  const title = `${data.title} | Omena`;

  return {
    meta: [
      {
        title,
      },
      {
        name: "description",
        content: data.description,
      },
      {
        property: "og:type",
        content: "article",
      },
      {
        property: "og:title",
        content: title,
      },
      {
        property: "og:description",
        content: data.description,
      },
      {
        property: "og:url",
        content: data.canonicalUrl,
      },
      {
        name: "twitter:title",
        content: title,
      },
      {
        name: "twitter:description",
        content: data.description,
      },
      {
        "script:ld+json": createDocumentationStructuredData(data),
      },
    ],
    links: [
      {
        rel: "canonical",
        href: data.canonicalUrl,
      },
    ],
  };
}

export const loadDocumentationPage = createServerFn({
  method: "GET",
})
  .validator((slugs: string[]) => slugs)
  .middleware([staticFunctionMiddleware])
  .handler(async ({ data: slugs }): Promise<DocumentationPageData> => {
    const page = source.getPage(slugs);
    if (!page) throw notFound();

    return {
      path: page.path,
      pageTree: await source.serializePageTree(source.getPageTree()),
      title: page.data.title,
      description: page.data.description,
      canonicalUrl: documentationCanonicalUrl(page.path),
    };
  });

export async function preloadDocumentationPage(data: DocumentationPageData) {
  await clientLoader.preload(data.path);
  return data;
}

const clientLoader = browserCollections.docs.createClientLoader({
  component({ toc, frontmatter, default: MDX }, { path }: { path: string }) {
    const sourceDescription =
      frontmatter.sourceOfTruth === "generated"
        ? "Generated from product code."
        : frontmatter.sourceOfTruth === "hybrid"
          ? "Authored guidance with generated contracts."
          : undefined;

    return (
      <DocsPage
        toc={toc.filter((item) => item.depth > 1)}
        full={frontmatter.full}
        slots={{ container: DocumentationMain }}
      >
        {frontmatter.status !== "stable" ? (
          <p className="mb-2 border-s-2 border-fd-warning ps-3 text-sm text-fd-muted-foreground">
            This page documents{" "}
            {frontmatter.status === "experimental" ? "an experimental" : `a ${frontmatter.status}`}{" "}
            surface. Its contract may still change.
          </p>
        ) : null}
        <DocsTitle>{frontmatter.title}</DocsTitle>
        <DocsDescription>{frontmatter.description}</DocsDescription>
        <div className="mb-8 flex flex-wrap items-center gap-x-3 gap-y-2 border-b border-fd-border pb-6">
          <EditOnGitHub
            href={`${site.repository}/edit/master/docs/${path}`}
            aria-label={`Edit ${frontmatter.title} on GitHub`}
          />
          {sourceDescription ? (
            <span className="text-xs text-fd-muted-foreground">{sourceDescription}</span>
          ) : null}
        </div>
        <DocsBody>
          <MDX components={getMDXComponents({ a: createDocumentationLink(path) })} />
        </DocsBody>
      </DocsPage>
    );
  },
});

export function DocumentationPage({ data }: { data: DocumentationPageData }) {
  const { pageTree, path } = useFumadocsLoader(data);

  return (
    <DocsLayout {...baseOptions()} tree={pageTree}>
      <Suspense>{clientLoader.useContent(path, { path })}</Suspense>
    </DocsLayout>
  );
}

function DocumentationMain({ children, className, ...props }: ComponentProps<"article">) {
  const { full } = useDocsPage();
  const widthClass = full ? "max-w-[1168px]" : "max-w-[900px]";

  return (
    <main
      {...props}
      id="main-content"
      data-full={full}
      className={`flex w-full ${widthClass} mx-auto flex-col [grid-area:main] gap-4 px-4 py-6 md:px-6 md:pt-8 xl:px-8 xl:pt-14 ${className ?? ""}`}
    >
      <article className="contents">{children}</article>
    </main>
  );
}

function documentationCanonicalUrl(sourcePath: string): string {
  const slug = sourcePath.replace(/\.(?:md|mdx)$/u, "");
  const pathname = slug === "index" ? "/" : `/docs/${slug}/`;
  return new URL(pathname, site.origin).href;
}

function createDocumentationStructuredData(data: DocumentationPageData) {
  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "WebSite",
        "@id": `${site.origin}/#website`,
        name: "Omena Documentation",
        url: `${site.origin}/`,
        description: site.description,
      },
      {
        "@type": "SoftwareApplication",
        "@id": `${site.origin}/#software`,
        name: site.name,
        applicationCategory: "DeveloperApplication",
        operatingSystem: "Cross-platform",
        url: `${site.origin}/`,
        sameAs: site.repository,
      },
      {
        "@type": "TechArticle",
        headline: data.title,
        description: data.description,
        url: data.canonicalUrl,
        mainEntityOfPage: data.canonicalUrl,
        isPartOf: {
          "@id": `${site.origin}/#website`,
        },
        about: {
          "@id": `${site.origin}/#software`,
        },
      },
    ],
  };
}
