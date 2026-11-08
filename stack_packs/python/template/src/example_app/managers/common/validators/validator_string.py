from typing import final


# A stateless helper shared by every validator: a class with static methods only, never built and never registered in
# the ServiceLocator.
@final
class ValidatorString:
    @staticmethod
    def validate(property_name: str, property_value: str | None) -> str | None:
        if property_value is None:
            return f"{property_name} is missing"
        if property_value == "":
            return f"{property_name} is empty"
        if property_value.strip() == "":
            return f"{property_name} is whitespace"
        return None
