import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

export interface PageLayoutProps {
  readonly children: ReactNode;
}

// What every page sits in: the app's name, its navigation, and the page itself.
export function PageLayout({ children }: PageLayoutProps): ReactNode {
  return (
    <div className="min-h-screen">
      <header className="border-b border-border bg-surface">
        <nav aria-label="Main" className="mx-auto flex max-w-3xl flex-wrap items-center gap-6 px-6 py-4">
          <Link to="/" className="text-lg font-semibold">
            Example App
          </Link>
          <Link
            to="/customers/new"
            className="text-muted-foreground hover:text-foreground"
            activeProps={{ className: "text-foreground font-medium" }}
          >
            New customer
          </Link>
          <Link
            to="/users/$userId"
            params={{ userId: "1" }}
            className="text-muted-foreground hover:text-foreground"
            activeProps={{ className: "text-foreground font-medium" }}
          >
            Look up a user
          </Link>
        </nav>
      </header>
      <main className="mx-auto max-w-3xl px-6 py-10">{children}</main>
    </div>
  );
}
