from pydantic import BaseModel


# Resource models: the JSON body deserialized as it is, with every field optional so that anything the API sends
# deserializes and the validator can say what is wrong with it. Fields the API sends that the model doesn't name are
# ignored.
class AddressResource(BaseModel):
    street: str | None = None
    suite: str | None = None
    city: str | None = None
    zipcode: str | None = None
