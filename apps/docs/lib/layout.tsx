import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { BookOpenText, Code2, Play } from "lucide-react";
import { site } from "./site";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="omena-wordmark">
          <span aria-hidden="true" className="omena-wordmark-mark">
            O
          </span>
          {site.name}
        </span>
      ),
    },
    githubUrl: site.repository,
    links: [
      {
        text: "Guides",
        url: "/docs/getting-started",
        icon: <BookOpenText />,
        active: "nested-url",
      },
      {
        text: "Playground",
        url: "/docs/playground",
        icon: <Play />,
      },
      {
        type: "icon",
        label: "GitHub",
        text: "Source",
        url: site.repository,
        icon: <Code2 />,
        external: true,
      },
    ],
  };
}
