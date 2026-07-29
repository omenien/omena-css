/// <reference types="vite/client" />

import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { RootProvider } from "fumadocs-ui/provider/tanstack";
import type { ReactNode } from "react";
import SearchDialog from "@/components/search";
import { site } from "@/lib/site";
import appCss from "@/styles/app.css?url";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        title: "Omena Documentation",
      },
      {
        name: "description",
        content: site.description,
      },
    ],
    links: [{ rel: "stylesheet", href: appCss }],
  }),
  component: RootComponent,
});

function RootComponent() {
  return (
    <RootDocument>
      <Outlet />
    </RootDocument>
  );
}

function RootDocument({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body className="flex min-h-screen flex-col">
        <a className="docs-skip-link" href="#main-content">
          Skip to content
        </a>
        <RootProvider
          search={{ SearchDialog }}
          theme={{
            defaultTheme: "light",
            enableSystem: false,
            storageKey: "omena-documentation-theme",
          }}
        >
          {children}
        </RootProvider>
        <Scripts />
      </body>
    </html>
  );
}
