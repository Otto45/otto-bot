from dataclasses import dataclass

from example_app.managers.data_layer.managers.gateways.document_database.clients.document import Document


@dataclass(frozen=True, slots=True)
class DocumentFindQuery:
    collection_name: str
    filter_criteria: Document
    sort_field: str
