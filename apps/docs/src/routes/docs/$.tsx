import { createFileRoute } from "@tanstack/react-router";
import {
  DocumentationPage,
  loadDocumentationPage,
  preloadDocumentationPage,
} from "@/components/documentation-page";

export const Route = createFileRoute("/docs/$")({
  component: RouteComponent,
  loader: async ({ params }) => {
    const slugs = params._splat?.split("/").filter(Boolean) ?? [];
    const data = await loadDocumentationPage({ data: slugs });
    return preloadDocumentationPage(data);
  },
  head: ({ loaderData }) => ({
    meta: loaderData
      ? [
          {
            title: `${loaderData.title} | Omena`,
          },
          {
            name: "description",
            content: loaderData.description,
          },
        ]
      : [],
  }),
});

function RouteComponent() {
  return <DocumentationPage data={Route.useLoaderData()} />;
}
