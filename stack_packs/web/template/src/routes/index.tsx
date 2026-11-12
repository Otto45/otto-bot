import { Link, createFileRoute } from "@tanstack/react-router";
import type { ReactNode } from "react";

// A page is a file route: its path (the file's, under src/routes/), what it loads (a loader, which calls server
// functions only, since it runs on the server and in the browser alike), and the component that shows it.
export const Route = createFileRoute("/")({
  component: HomePage,
});

function HomePage(): ReactNode {
  return (
    <section className="flex flex-col gap-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-semibold">Welcome</h1>
        <p className="text-muted-foreground">Save your customers, greet them, and see everything that happened.</p>
      </div>
      <ul className="grid gap-4 sm:grid-cols-2">
        <li>
          <Link
            to="/customers/new"
            className="flex h-full flex-col gap-1 rounded-panel border border-border bg-surface p-5 hover:border-primary"
          >
            <span className="font-semibold">Add a customer</span>
            <span className="text-muted-foreground">Save someone's name and email.</span>
          </Link>
        </li>
        <li>
          <Link
            to="/users/$userId"
            params={{ userId: "1" }}
            className="flex h-full flex-col gap-1 rounded-panel border border-border bg-surface p-5 hover:border-primary"
          >
            <span className="font-semibold">Look up a user</span>
            <span className="text-muted-foreground">See a user's profile, and add them as a customer.</span>
          </Link>
        </li>
      </ul>
    </section>
  );
}
