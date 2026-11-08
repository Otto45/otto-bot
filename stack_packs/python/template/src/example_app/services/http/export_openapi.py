import json
import sys
from pathlib import Path
from typing import Final

from example_app.services.http.http_service import HttpService

# The service's contract: every route, with the wire models it takes and answers with, as the OpenAPI document FastAPI
# builds from the routers. It is kept in the project so that a change to the API shows up as a change to this file, and
# so that whatever calls the API can check itself against it.
_CONTRACT_PATH: Final[Path] = Path("openapi.json")


# Writes the contract (`uv run export-openapi`), or with `--check` fails when the file is out of date.
def main() -> int:
    # Sorted keys, so the file only changes when the contract does.
    contract: str = json.dumps(HttpService.get_contract(), indent=2, sort_keys=True, ensure_ascii=False) + "\n"

    if "--check" not in sys.argv[1:]:
        try:
            _CONTRACT_PATH.write_text(contract, encoding="utf-8")
        except OSError as error:
            print(f"{_CONTRACT_PATH} could not be written: {error}", file=sys.stderr)
            return 1
        return 0

    try:
        current_contract: str | None = _CONTRACT_PATH.read_text(encoding="utf-8")
    except OSError:
        current_contract: str | None = None
    if current_contract != contract:
        print(f"{_CONTRACT_PATH} is out of date: run `uv run export-openapi`", file=sys.stderr)
        return 1
    return 0
