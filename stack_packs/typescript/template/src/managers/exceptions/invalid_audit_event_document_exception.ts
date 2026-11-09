import { DocumentDatabaseGatewayException } from "#src/managers/exceptions/document_database_gateway_exception.ts";

/** An audit_events document doesn't have what an AuditEvent needs. */
export class InvalidAuditEventDocumentException extends DocumentDatabaseGatewayException {}
