from dataclasses import dataclass


# A domain model: frozen, so it never changes once made, and no component can change one another holds.
@dataclass(frozen=True, slots=True)
class Customer:
    id: str
    name: str
    email: str
