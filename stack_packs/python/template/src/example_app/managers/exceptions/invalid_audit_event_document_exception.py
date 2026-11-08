from typing import final

from example_app.managers.exceptions.document_database_gateway_exception import DocumentDatabaseGatewayException


@final
class InvalidAuditEventDocumentException(DocumentDatabaseGatewayException):
    """An audit_events document doesn't have what an AuditEvent needs."""
