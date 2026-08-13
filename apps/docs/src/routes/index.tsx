import { createFileRoute } from "@tanstack/react-router";
import {
  createDocumentationHead,
  DocumentationPage,
  loadDocumentationPage,
  preloadDocumentationPage,
} from "@/components/documentation-page";

export const Route = createFileRoute("/")({
  component: RouteComponent,
  loader: async () => {
    const data = await loadDocumentationPage({ data: [] });
    return preloadDocumentationPage(data);
  },
  head: ({ loaderData }) => createDocumentationHead(loaderData),
});

function RouteComponent() {
  return <DocumentationPage data={Route.useLoaderData()} />;
}
