import type { Document } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/clients/document.ts";

export interface DocumentFindQuery {
  readonly collectionName: string;
  readonly filterCriteria: Document;
  readonly sortField: string;
}
