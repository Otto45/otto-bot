from pydantic import BaseModel

from example_app.managers.gateways.json_placeholder.resource_models.address_resource import AddressResource
from example_app.managers.gateways.json_placeholder.resource_models.company_resource import CompanyResource


class UserResource(BaseModel):
    id: int | None = None
    name: str | None = None
    username: str | None = None
    email: str | None = None
    phone: str | None = None
    website: str | None = None
    address: AddressResource | None = None
    company: CompanyResource | None = None
