/* oxlint-disable import/no-default-export -- Next.js discovers nested layouts through a default export. */
import type { ReactNode } from "react";
import { GlassLayout } from "fumadocs-ui/layouts/glass";
import { source } from "@/lib/source";
import { baseOptions } from "@/lib/layout";

export default function DocsLayout({ children }: { children: ReactNode }) {
  return (
    <GlassLayout tree={source.getPageTree()} {...baseOptions()}>
      {children}
    </GlassLayout>
  );
}
