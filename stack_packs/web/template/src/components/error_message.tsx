import type { ReactNode } from "react";

export interface ErrorMessageProps {
  readonly message: string;
}

// What went wrong, announced to screen readers as it appears.
export function ErrorMessage({ message }: ErrorMessageProps): ReactNode {
  return (
    <p role="alert" className="rounded-control border border-danger bg-danger-muted px-4 py-3 text-danger">
      {message}
    </p>
  );
}
