from dataclasses import dataclass

from example_app.managers.data_layer.managers.gateways.document_database.clients.document import Document


@dataclass(frozen=True, slots=True)
class DocumentInsertQuery:
    collection_name: str
    document: Document
