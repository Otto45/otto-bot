import type { Document } from "#src/managers/data_layer/managers/gateways/document_database/clients/document.ts";

// Stand-in for a document database driver client (e.g. mongodb's MongoClient) so the template runs with no
// infrastructure. Documents are stored and returned raw, the way a driver returns them, and closing a client leaves them
// in place. Replace it with the real client rather than copying it into production code. Its methods are async because
// the driver's are: code written against it doesn't change when the real client replaces it.
export class InMemoryDocumentDatabaseClient {
  // Like a database server, the store outlives any one client: every client in the process that connects with the same
  // connection string sees the same collections.
  private static readonly databases: Map<string, Map<string, Document[]>> = new Map<string, Map<string, Document[]>>();

  private readonly collections: Map<string, Document[]>;

  public constructor(connectionString: string) {
    const collections: Map<string, Document[]> =
      InMemoryDocumentDatabaseClient.databases.get(connectionString) ?? new Map<string, Document[]>();
    InMemoryDocumentDatabaseClient.databases.set(connectionString, collections);
    this.collections = collections;
  }

  public find(collectionName: string, filterCriteria: Document, sortField: string): Promise<Document[]> {
    const documents: Document[] = (this.collections.get(collectionName) ?? [])
      .filter((document: Document): boolean => InMemoryDocumentDatabaseClient.matches(document, filterCriteria))
      .map((document: Document): Document => ({ ...document }));
    documents.sort((left: Document, right: Document): number =>
      (left[sortField] ?? "").localeCompare(right[sortField] ?? ""),
    );
    return Promise.resolve(documents);
  }

  public insertOne(collectionName: string, document: Document): Promise<void> {
    const documents: Document[] = this.collections.get(collectionName) ?? [];
    documents.push({ ...document });
    this.collections.set(collectionName, documents);
    return Promise.resolve();
  }

  public deleteMany(collectionName: string, filterCriteria: Document): Promise<void> {
    const documents: Document[] = (this.collections.get(collectionName) ?? []).filter(
      (document: Document): boolean => !InMemoryDocumentDatabaseClient.matches(document, filterCriteria),
    );
    this.collections.set(collectionName, documents);
    return Promise.resolve();
  }

  public close(): Promise<void> {
    // Nothing to disconnect from; the documents stay in the store, as they would on a server.
    return Promise.resolve();
  }

  private static matches(document: Document, filterCriteria: Document): boolean {
    return Object.entries(filterCriteria).every(
      ([field, value]: [string, string]): boolean => document[field] === value,
    );
  }
}
