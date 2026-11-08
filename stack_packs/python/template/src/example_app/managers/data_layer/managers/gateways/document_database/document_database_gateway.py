from typing import final

from pydantic import ValidationError

from example_app.managers.common.formatters.validation_error_formatter import ValidationErrorFormatter
from example_app.managers.data_layer.managers.gateways.document_database.clients.document import Document
from example_app.managers.data_layer.managers.gateways.document_database.clients.in_memory_document_database_client import (  # noqa: E501
    InMemoryDocumentDatabaseClient,
)
from example_app.managers.data_layer.managers.gateways.document_database.mappers.audit_event_mapper import (
    AuditEventMapper,
)
from example_app.managers.data_layer.managers.gateways.document_database.query_builders.audit_event_query_builder import (  # noqa: E501
    AuditEventQueryBuilder,
)
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.audit_event_document import (
    AuditEventDocument,
)
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.document_find_query import (
    DocumentFindQuery,
)
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.document_insert_query import (
    DocumentInsertQuery,
)
from example_app.managers.data_layer.managers.gateways.document_database.validators.validator_audit_event_document import (  # noqa: E501
    ValidatorAuditEventDocument,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_audit_event_document_exception import InvalidAuditEventDocumentException
from example_app.managers.models.audit_event import AuditEvent


@final
class DocumentDatabaseGateway:
    def __init__(self, document_database_client: InMemoryDocumentDatabaseClient) -> None:
        self._document_database_client: InMemoryDocumentDatabaseClient = document_database_client

    async def get_audit_events(self, customer_id: str) -> list[AuditEvent]:
        context: ContextualData = ContextualData().with_value("customer_id", customer_id)
        document_find_query: DocumentFindQuery = AuditEventQueryBuilder.build_get_audit_events_query(customer_id)

        documents: list[Document] = await self._document_database_client.find(
            document_find_query.collection_name,
            document_find_query.filter_criteria,
            document_find_query.sort_field,
        )
        return [DocumentDatabaseGateway._to_audit_event(document, context) for document in documents]

    async def save_audit_event(self, audit_event: AuditEvent) -> None:
        document_insert_query: DocumentInsertQuery = AuditEventQueryBuilder.build_save_audit_event_query(audit_event)
        await self._document_database_client.insert_one(
            document_insert_query.collection_name, document_insert_query.document
        )

    async def close(self) -> None:
        await self._document_database_client.close()

    @staticmethod
    def _to_audit_event(document: Document, context: ContextualData) -> AuditEvent:
        try:
            audit_event_document: AuditEventDocument = AuditEventDocument.model_validate(document)
        except ValidationError as error:
            raise InvalidAuditEventDocumentException(
                "Audit event document from the document database audit_events collection could not be deserialized",
                context.with_value("errors", ValidationErrorFormatter.format(error, "document")),
            ) from error

        ValidatorAuditEventDocument.validate_audit_event_document(audit_event_document)
        return AuditEventMapper.to_audit_event(audit_event_document)
