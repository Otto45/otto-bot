import { useId } from "react";
import type { ChangeEvent, ReactNode } from "react";

export interface TextFieldProps {
  readonly label: string;
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly type?: "text" | "email";
  readonly required?: boolean;
  readonly autoComplete?: string;
}

// A labelled text input: the label names it for screen readers too.
export function TextField({
  label,
  value,
  onChange,
  type = "text",
  required = false,
  autoComplete = "off",
}: TextFieldProps): ReactNode {
  const inputId: string = useId();
  return (
    <div className="flex flex-col gap-1.5">
      <label htmlFor={inputId} className="font-medium">
        {label}
      </label>
      <input
        id={inputId}
        type={type}
        value={value}
        required={required}
        autoComplete={autoComplete}
        onChange={(event: ChangeEvent<HTMLInputElement>): void => {
          onChange(event.target.value);
        }}
        className="rounded-control border border-border bg-surface px-3 py-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
      />
    </div>
  );
}
