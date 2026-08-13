/// <reference types="vite/client" />

import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { RootProvider } from "fumadocs-ui/provider/tanstack";
import type { ReactNode } from "react";
import { OmenaSearchDialog } from "@/components/search";
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
      {
        name: "application-name",
        content: "Omena Documentation",
      },
      {
        name: "robots",
        content: "index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1",
      },
      {
        property: "og:site_name",
        content: "Omena Documentation",
      },
      {
        property: "og:locale",
        content: "en_US",
      },
      {
        name: "twitter:card",
        content: "summary",
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
        <script dangerouslySetInnerHTML={{ __html: googleTagManagerBootstrap }} />
        <HeadContent />
      </head>
      <body className="flex min-h-screen flex-col">
        <noscript>
          <iframe
            src={`https://www.googletagmanager.com/ns.html?id=${site.googleTagManagerId}`}
            height="0"
            width="0"
            style={{ display: "none", visibility: "hidden" }}
            title="Google Tag Manager"
          />
        </noscript>
        <a className="docs-skip-link" href="#main-content">
          Skip to content
        </a>
        <RootProvider
          search={{ SearchDialog: OmenaSearchDialog }}
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

const googleTagManagerBootstrap = `(function(w,d,s,l,i){w[l]=w[l]||[];w[l].push({'gtm.start':
new Date().getTime(),event:'gtm.js'});var f=d.getElementsByTagName(s)[0],
j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src=
'https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);
})(window,document,'script','dataLayer','${site.googleTagManagerId}');`;
