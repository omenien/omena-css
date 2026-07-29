/* oxlint-disable import/no-default-export -- Next.js discovers root layouts through a default export. */
import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Provider } from "@/components/provider";
import { site } from "@/lib/site";
// Next.js loads global styles from the root layout boundary.
// oxlint-disable-next-line import/no-unassigned-import
import "./global.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://omenien.github.io/omena-css"),
  title: {
    default: "Omena Documentation",
    template: "%s | Omena",
  },
  description: site.description,
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}
