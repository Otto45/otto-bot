import { createFileRoute, notFound, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import type { ReactNode } from "react";

import { Button } from "#src/components/button.tsx";
import { ErrorMessage } from "#src/components/error_message.tsx";
import { ErrorMessageFormatter } from "#src/formatters/error_message_formatter.ts";
import { getUserProfile } from "#src/server_functions/get_user_profile.ts";
import { onboardCustomer } from "#src/server_functions/onboard_customer.ts";
import type { CustomerResponse } from "#src/wire_models/customer_response.ts";
import type { UserProfileResponse } from "#src/wire_models/user_profile_response.ts";

export const Route = createFileRoute("/users/$userId")({
  // A path's parameters are text: one that can't be a user's id is a page that isn't there.
  loader: ({ params }) => {
    const userId: number = Number(params.userId);
    if (!Number.isSafeInteger(userId)) {
      throw notFound();
    }
    return getUserProfile({ data: { userId } });
  },
  component: UserProfilePage,
});

function UserProfilePage(): ReactNode {
  const userProfile: UserProfileResponse = Route.useLoaderData();
  const navigate = useNavigate();
  const [errorMessage, setErrorMessage] = useState<string | undefined>(undefined);
  const [isAdding, setIsAdding] = useState<boolean>(false);

  async function addAsCustomer(): Promise<void> {
    setIsAdding(true);
    setErrorMessage(undefined);
    try {
      const customer: CustomerResponse = await onboardCustomer({ data: { userId: userProfile.id } });
      await navigate({ to: "/customers/$customerId", params: { customerId: customer.id } });
    } catch (error: unknown) {
      setErrorMessage(ErrorMessageFormatter.format(error));
    } finally {
      setIsAdding(false);
    }
  }

  return (
    <section className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold">{userProfile.name}</h1>
        <p className="text-muted-foreground">@{userProfile.username}</p>
      </div>
      <dl className="grid max-w-md grid-cols-[auto_1fr] gap-x-6 gap-y-2">
        <dt className="text-muted-foreground">Email</dt>
        <dd>{userProfile.email}</dd>
        <dt className="text-muted-foreground">City</dt>
        <dd>{userProfile.city}</dd>
        <dt className="text-muted-foreground">Works at</dt>
        <dd>{userProfile.companyName}</dd>
      </dl>
      {errorMessage === undefined ? null : <ErrorMessage message={errorMessage} />}
      <div>
        <Button
          disabled={isAdding}
          onClick={(): void => {
            void addAsCustomer();
          }}
        >
          {isAdding ? "Adding…" : "Add as a customer"}
        </Button>
      </div>
    </section>
  );
}
