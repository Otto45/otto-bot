import { DocumentDatabaseGatewayException } from "#src/domain_container/managers/exceptions/document_database_gateway_exception.ts";

/** An audit_events document doesn't have what an AuditEvent needs. */
export class InvalidAuditEventDocumentException extends DocumentDatabaseGatewayException {}
