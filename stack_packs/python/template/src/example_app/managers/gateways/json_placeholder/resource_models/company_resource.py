from pydantic import BaseModel


class CompanyResource(BaseModel):
    name: str | None = None
