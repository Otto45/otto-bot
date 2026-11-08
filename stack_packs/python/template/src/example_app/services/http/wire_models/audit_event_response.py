from datetime import datetime

from pydantic import BaseModel


class AuditEventResponse(BaseModel):
    id: str
    customer_id: str
    description: str
    occurred_on: datetime
