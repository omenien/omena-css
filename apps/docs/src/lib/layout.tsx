import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { site } from "./site";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: site.name,
    },
    githubUrl: site.repository,
  };
}
