from pydantic import BaseModel


# A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say what is
# wrong with it.
class CustomerRow(BaseModel):
    customer_id: str | None = None
    full_name: str | None = None
    email_address: str | None = None
