import type { ReactNode } from "react";

export interface ButtonProps {
  readonly children: ReactNode;
  readonly type?: "button" | "submit";
  readonly variant?: "primary" | "secondary";
  readonly disabled?: boolean;
  readonly onClick?: () => void;
}

const VARIANT_CLASS_NAMES: Record<NonNullable<ButtonProps["variant"]>, string> = {
  primary: "bg-primary text-primary-foreground hover:bg-primary-hover",
  secondary: "border border-border bg-surface text-foreground hover:bg-muted",
};

// A button, styled by what it is for: the page's main action (primary) or any other (secondary).
export function Button({
  children,
  type = "button",
  variant = "primary",
  disabled = false,
  onClick,
}: ButtonProps): ReactNode {
  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      className={`inline-flex items-center justify-center rounded-control px-4 py-2 font-medium focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:cursor-not-allowed disabled:opacity-60 ${VARIANT_CLASS_NAMES[variant]}`}
    >
      {children}
    </button>
  );
}
