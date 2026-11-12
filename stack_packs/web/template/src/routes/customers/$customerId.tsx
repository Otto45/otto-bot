import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useState } from "react";
import type { ReactNode } from "react";

import { Button } from "#src/components/button.tsx";
import { ErrorMessage } from "#src/components/error_message.tsx";
import { DateTimeFormatter } from "#src/formatters/date_time_formatter.ts";
import { ErrorMessageFormatter } from "#src/formatters/error_message_formatter.ts";
import { getCustomerAuditEvents } from "#src/server_functions/get_customer_audit_events.ts";
import { greetCustomer } from "#src/server_functions/greet_customer.ts";
import type { AuditEventListResponse } from "#src/wire_models/audit_event_list_response.ts";
import type { AuditEventResponse } from "#src/wire_models/audit_event_response.ts";
import type { GreetingResponse } from "#src/wire_models/greeting_response.ts";

export const Route = createFileRoute("/customers/$customerId")({
  loader: ({ params }) => getCustomerAuditEvents({ data: { customerId: params.customerId } }),
  component: CustomerPage,
});

// What the page loaded, and a write the person asks for: once it is done, the page loads again (router.invalidate), so
// it shows what the write changed.
function CustomerPage(): ReactNode {
  const { customerId } = Route.useParams();
  const auditEventList: AuditEventListResponse = Route.useLoaderData();
  const router = useRouter();
  const [greetingMessage, setGreetingMessage] = useState<string | undefined>(undefined);
  const [errorMessage, setErrorMessage] = useState<string | undefined>(undefined);
  const [isGreeting, setIsGreeting] = useState<boolean>(false);

  async function greet(): Promise<void> {
    setIsGreeting(true);
    setErrorMessage(undefined);
    try {
      const greeting: GreetingResponse = await greetCustomer({ data: { customerId } });
      setGreetingMessage(greeting.message);
      await router.invalidate();
    } catch (error: unknown) {
      setErrorMessage(ErrorMessageFormatter.format(error));
    } finally {
      setIsGreeting(false);
    }
  }

  return (
    <section className="flex flex-col gap-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <h1 className="text-2xl font-semibold">Customer {customerId}</h1>
        <Button
          disabled={isGreeting}
          onClick={(): void => {
            void greet();
          }}
        >
          {isGreeting ? "Greeting…" : "Greet them"}
        </Button>
      </div>
      {greetingMessage === undefined ? null : (
        <p role="status" className="rounded-panel bg-muted px-4 py-3 text-lg">
          {greetingMessage}
        </p>
      )}
      {errorMessage === undefined ? null : <ErrorMessage message={errorMessage} />}
      <div className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold">What happened</h2>
        {auditEventList.auditEvents.length === 0 ? (
          <p className="text-muted-foreground">Nothing yet.</p>
        ) : (
          <ol className="flex flex-col divide-y divide-border rounded-panel border border-border bg-surface">
            {auditEventList.auditEvents.map((auditEvent: AuditEventResponse): ReactNode => (
              <li key={auditEvent.id} className="flex flex-wrap justify-between gap-2 px-4 py-3">
                <span>{auditEvent.description}</span>
                <time dateTime={auditEvent.occurredOn} className="text-muted-foreground">
                  {DateTimeFormatter.format(auditEvent.occurredOn)}
                </time>
              </li>
            ))}
          </ol>
        )}
      </div>
    </section>
  );
}
