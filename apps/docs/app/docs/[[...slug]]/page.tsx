/* oxlint-disable import/no-default-export -- Next.js discovers route pages through a default export. */
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { createRelativeLink } from "fumadocs-ui/mdx";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  EditOnGitHub,
} from "fumadocs-ui/layouts/glass/page";
import { getMDXComponents } from "@/components/mdx";
import { source } from "@/lib/source";
import { site } from "@/lib/site";

type PageProperties = {
  params: Promise<{ slug?: string[] }>;
};

export default async function Page({ params }: PageProperties) {
  const { slug } = await params;
  const page = source.getPage(slug);
  if (!page) notFound();

  const MDX = page.data.body;

  return (
    <DocsPage toc={page.data.toc.filter((item) => item.depth > 1)} full={page.data.full}>
      <div className="mb-5 flex flex-wrap gap-2 text-xs font-semibold uppercase tracking-[0.13em]">
        <span className="omena-meta-pill">{page.data.kind}</span>
        <span className="omena-meta-pill" data-status={page.data.status}>
          {page.data.status}
        </span>
        <span className="omena-meta-pill">{page.data.sourceOfTruth}</span>
      </div>
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      <div className="mb-8 flex items-center gap-3 border-b border-[var(--omena-line)] pb-6">
        <EditOnGitHub
          href={`${site.repository}/edit/master/docs/${page.path}`}
          aria-label={`Edit ${page.data.title} on GitHub`}
        />
        <span className="text-xs text-[var(--omena-muted)]">Owned by {page.data.owner}</span>
      </div>
      <DocsBody>
        <MDX components={getMDXComponents({ a: createRelativeLink(source, page) })} />
      </DocsBody>
    </DocsPage>
  );
}

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata({ params }: PageProperties): Promise<Metadata> {
  const { slug } = await params;
  const page = source.getPage(slug);
  if (!page) notFound();

  return {
    title: page.data.title,
    description: page.data.description,
  };
}
