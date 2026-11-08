from typing import final


@final
class ValidatedValue:
    @staticmethod
    def expect[T](value: T | None, validator_name: str) -> T:
        """A value a validator has checked is there (a mapper's or a settings loader's): it raises only if the validator
        is wrong, which is a bug, so it is an unexpected exception rather than an ApplicationBaseException. Not an
        assert, which `python -O` removes."""
        if value is None:
            raise AssertionError(f"validated by {validator_name}")
        return value
