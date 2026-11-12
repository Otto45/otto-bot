import type { z } from "zod";

import { ValidationErrorFormatter } from "#src/domain_container/managers/common/formatters/validation_error_formatter.ts";
import type { Document } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/clients/document.ts";
import type { InMemoryDocumentDatabaseClient } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/clients/in_memory_document_database_client.ts";
import { AuditEventMapper } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/mappers/audit_event_mapper.ts";
import { AuditEventQueryBuilder } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/query_builders/audit_event_query_builder.ts";
import { AuditEventDocumentSchema } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/resource_models/audit_event_document.ts";
import type { AuditEventDocument } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/resource_models/audit_event_document.ts";
import type { DocumentFindQuery } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/resource_models/document_find_query.ts";
import type { DocumentInsertQuery } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/resource_models/document_insert_query.ts";
import { ValidatorAuditEventDocument } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/validators/validator_audit_event_document.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidAuditEventDocumentException } from "#src/domain_container/managers/exceptions/invalid_audit_event_document_exception.ts";
import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";

export class DocumentDatabaseGateway {
  private readonly documentDatabaseClient: InMemoryDocumentDatabaseClient;

  public constructor(documentDatabaseClient: InMemoryDocumentDatabaseClient) {
    this.documentDatabaseClient = documentDatabaseClient;
  }

  public async getAuditEvents(customerId: string): Promise<AuditEvent[]> {
    const context: ContextualData = new ContextualData().withValue("customer_id", customerId);
    const documentFindQuery: DocumentFindQuery = AuditEventQueryBuilder.buildGetAuditEventsQuery(customerId);

    const documents: Document[] = await this.documentDatabaseClient.find(
      documentFindQuery.collectionName,
      documentFindQuery.filterCriteria,
      documentFindQuery.sortField,
    );
    return documents.map((document: Document): AuditEvent => DocumentDatabaseGateway.toAuditEvent(document, context));
  }

  public async saveAuditEvent(auditEvent: AuditEvent): Promise<void> {
    const documentInsertQuery: DocumentInsertQuery = AuditEventQueryBuilder.buildSaveAuditEventQuery(auditEvent);
    await this.documentDatabaseClient.insertOne(documentInsertQuery.collectionName, documentInsertQuery.document);
  }

  public async close(): Promise<void> {
    await this.documentDatabaseClient.close();
  }

  private static toAuditEvent(document: Document, context: ContextualData): AuditEvent {
    const parsed: z.ZodSafeParseResult<AuditEventDocument> = AuditEventDocumentSchema.safeParse(document);
    if (!parsed.success) {
      throw new InvalidAuditEventDocumentException(
        "Audit event document from the document database audit_events collection could not be deserialized",
        context.withValue("errors", ValidationErrorFormatter.format(parsed.error, "document")),
      );
    }
    ValidatorAuditEventDocument.validateAuditEventDocument(parsed.data);
    return AuditEventMapper.toAuditEvent(parsed.data);
  }
}
