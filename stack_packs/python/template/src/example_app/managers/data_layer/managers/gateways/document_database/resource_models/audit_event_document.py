from datetime import datetime

from pydantic import BaseModel


# A document as the database stores it: every field optional, so that any document deserializes and the validator can
# say what is wrong with it.
class AuditEventDocument(BaseModel):
    event_id: str | None = None
    customer_id: str | None = None
    description: str | None = None
    occurred_on: datetime | None = None
