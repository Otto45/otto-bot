from pydantic import BaseModel


# Request models belong to the framework: FastAPI deserializes each request into them and answers a 422 itself when one
# doesn't fit, so the routes only ever map well-formed requests into domain models. A request model says what shape a
# request has (its fields and their types) and no more: whether the values are acceptable is the domain container's
# decision (here CustomerProcessor's), so that a caller who sends a blank name is answered with the domain's own
# message.
class SaveCustomerRequest(BaseModel):
    name: str
    email: str
