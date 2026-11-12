import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

// What a page shows when what it asked for isn't there (the router's default): no such page, or no such record.
export function NotFoundPage(): ReactNode {
  return (
    <section className="flex flex-col items-start gap-4">
      <h1 className="text-2xl font-semibold">We couldn't find that</h1>
      <p className="text-muted-foreground">It may have been moved, or the link may be wrong.</p>
      <Link to="/" className="font-medium text-primary hover:underline">
        Go to the home page
      </Link>
    </section>
  );
}
