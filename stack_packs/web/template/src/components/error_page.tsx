import { useRouter } from "@tanstack/react-router";
import type { ErrorComponentProps } from "@tanstack/react-router";
import type { ReactNode } from "react";

import { Button } from "#src/components/button.tsx";
import { ErrorMessage } from "#src/components/error_message.tsx";
import { ErrorMessageFormatter } from "#src/formatters/error_message_formatter.ts";

// What a page shows when its loader fails (the router's default): what the server function said, and a way to try again.
export function ErrorPage({ error }: ErrorComponentProps): ReactNode {
  const router = useRouter();
  return (
    <section className="flex flex-col items-start gap-4">
      <h1 className="text-2xl font-semibold">This page couldn't be shown</h1>
      <ErrorMessage message={ErrorMessageFormatter.format(error)} />
      <Button
        variant="secondary"
        onClick={(): void => {
          void router.invalidate();
        }}
      >
        Try again
      </Button>
    </section>
  );
}
