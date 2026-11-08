from dataclasses import dataclass
from datetime import datetime


@dataclass(frozen=True, slots=True)
class AuditEvent:
    id: str
    customer_id: str
    description: str
    occurred_on: datetime
