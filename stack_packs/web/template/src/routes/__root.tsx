import { HeadContent, Scripts, createRootRoute } from "@tanstack/react-router";
import type { ReactNode } from "react";

import { PageLayout } from "#src/components/page_layout.tsx";
import appCss from "#src/styles/app.css?url";

// The root route: the HTML document every page is rendered in (the shell renders even when a page fails), its head,
// and the layout around each page.
export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Example App" },
      { name: "description", content: "Saves customers, greets them, and looks up user profiles." },
    ],
    links: [
      { rel: "stylesheet", href: appCss },
      { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
    ],
  }),
  shellComponent: RootDocument,
});

interface RootDocumentProps {
  readonly children: ReactNode;
}

function RootDocument({ children }: RootDocumentProps): ReactNode {
  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body>
        <PageLayout>{children}</PageLayout>
        <Scripts />
      </body>
    </html>
  );
}
