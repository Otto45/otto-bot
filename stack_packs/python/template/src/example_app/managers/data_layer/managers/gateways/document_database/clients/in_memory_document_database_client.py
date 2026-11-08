from typing import ClassVar, final

from example_app.managers.data_layer.managers.gateways.document_database.clients.document import Document


# Stand-in for a document database driver client (e.g. pymongo's AsyncMongoClient) so the template runs with no
# infrastructure. Documents are stored and returned raw, the way a driver returns them, and closing a client leaves them
# in place. Replace it with the real client rather than copying it into production code. Its methods are async because
# the driver's are: code written against it doesn't change when the real client replaces it.
@final
class InMemoryDocumentDatabaseClient:
    # Like a database server, the store outlives any one client: every client in the process that connects with the
    # same connection string sees the same collections.
    _databases: ClassVar[dict[str, dict[str, list[Document]]]] = {}

    def __init__(self, connection_string: str) -> None:
        self._collections: dict[str, list[Document]] = InMemoryDocumentDatabaseClient._databases.setdefault(
            connection_string, {}
        )

    async def find(self, collection_name: str, filter_criteria: Document, sort_field: str) -> list[Document]:
        documents: list[Document] = [
            dict(document)
            for document in self._collections.get(collection_name, [])
            if InMemoryDocumentDatabaseClient._matches(document, filter_criteria)
        ]
        documents.sort(key=lambda document: document.get(sort_field, ""))
        return documents

    async def insert_one(self, collection_name: str, document: Document) -> None:
        self._collections.setdefault(collection_name, []).append(dict(document))

    async def delete_many(self, collection_name: str, filter_criteria: Document) -> None:
        self._collections[collection_name] = [
            document
            for document in self._collections.get(collection_name, [])
            if not InMemoryDocumentDatabaseClient._matches(document, filter_criteria)
        ]

    async def close(self) -> None:
        # Nothing to disconnect from; the documents stay in the store, as they would on a server.
        pass

    @staticmethod
    def _matches(document: Document, filter_criteria: Document) -> bool:
        return all(document.get(field) == value for field, value in filter_criteria.items())
