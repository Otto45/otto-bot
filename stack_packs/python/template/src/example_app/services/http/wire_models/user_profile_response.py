from pydantic import BaseModel


class UserProfileResponse(BaseModel):
    id: int
    name: str
    username: str
    email: str
    city: str
    company_name: str
