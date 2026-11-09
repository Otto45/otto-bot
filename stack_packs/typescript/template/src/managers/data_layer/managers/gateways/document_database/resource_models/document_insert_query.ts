import type { Document } from "#src/managers/data_layer/managers/gateways/document_database/clients/document.ts";

export interface DocumentInsertQuery {
  readonly collectionName: string;
  readonly document: Document;
}
