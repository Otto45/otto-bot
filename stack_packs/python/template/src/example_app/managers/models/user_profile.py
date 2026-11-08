from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class UserProfile:
    id: int
    name: str
    username: str
    email: str
    city: str
    company_name: str
