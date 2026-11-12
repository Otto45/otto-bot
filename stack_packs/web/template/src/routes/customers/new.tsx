import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import type { SubmitEvent, ReactNode } from "react";

import { Button } from "#src/components/button.tsx";
import { ErrorMessage } from "#src/components/error_message.tsx";
import { TextField } from "#src/components/text_field.tsx";
import { ErrorMessageFormatter } from "#src/formatters/error_message_formatter.ts";
import { saveCustomer } from "#src/server_functions/save_customer.ts";

export const Route = createFileRoute("/customers/new")({
  component: NewCustomerPage,
});

// A form: what the person types is kept in the component's state, sent to a server function when they submit it, and
// what went wrong (the domain's own message, e.g. a blank name) is shown beside it.
function NewCustomerPage(): ReactNode {
  const navigate = useNavigate();
  const [customerId, setCustomerId] = useState<string>("");
  const [name, setName] = useState<string>("");
  const [email, setEmail] = useState<string>("");
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | undefined>(undefined);

  async function save(): Promise<void> {
    setIsSaving(true);
    setErrorMessage(undefined);
    try {
      await saveCustomer({ data: { customerId, name, email } });
      await navigate({ to: "/customers/$customerId", params: { customerId } });
    } catch (error: unknown) {
      setErrorMessage(ErrorMessageFormatter.format(error));
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <section className="flex flex-col gap-6">
      <h1 className="text-2xl font-semibold">New customer</h1>
      <form
        className="flex max-w-md flex-col gap-4"
        onSubmit={(event: SubmitEvent<HTMLFormElement>): void => {
          event.preventDefault();
          void save();
        }}
      >
        <TextField label="Customer number" value={customerId} onChange={setCustomerId} required />
        <TextField label="Name" value={name} onChange={setName} autoComplete="name" />
        <TextField label="Email" type="email" value={email} onChange={setEmail} autoComplete="email" />
        {errorMessage === undefined ? null : <ErrorMessage message={errorMessage} />}
        <div>
          <Button type="submit" disabled={isSaving}>
            {isSaving ? "Saving…" : "Save customer"}
          </Button>
        </div>
      </form>
    </section>
  );
}
