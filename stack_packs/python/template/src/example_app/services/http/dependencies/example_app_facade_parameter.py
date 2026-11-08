from typing import Annotated

from fastapi import Depends

from example_app.example_app_facade import ExampleAppFacade
from example_app.services.http.dependencies.example_app_facade_dependency import ExampleAppFacadeDependency

# How a route asks for the facade: `example_app_facade: ExampleAppFacadeParameter`.
type ExampleAppFacadeParameter = Annotated[ExampleAppFacade, Depends(ExampleAppFacadeDependency.get)]
