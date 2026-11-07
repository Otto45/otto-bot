use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

/// A document as the driver stores and returns it: raw fields and values.
pub type Document = BTreeMap<String, String>;

type Collections = HashMap<String, Vec<Document>>;

// Like a database server, the store outlives any one client: every client in the process that connects with the
// same connection string sees the same collections.
static DATABASES: LazyLock<Mutex<HashMap<String, Arc<Mutex<Collections>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Stand-in for a document database driver client (e.g. mongodb) so the template runs with no infrastructure.
/// Documents are stored and returned raw, the way a driver returns them, and closing a client leaves them in place.
/// Replace it with the real client rather than copying it into production code. Its methods are `async` because the
/// driver's are: code written against it doesn't change when the real client replaces it.
#[derive(Clone)]
pub struct InMemoryDocumentDatabaseClient {
    collections: Arc<Mutex<Collections>>,
}

impl InMemoryDocumentDatabaseClient {
    pub fn new(connection_string: &str) -> Self {
        let mut databases: MutexGuard<'_, HashMap<String, Arc<Mutex<Collections>>>> =
            DATABASES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let collections: Arc<Mutex<Collections>> = databases.entry(connection_string.to_string()).or_default().clone();
        InMemoryDocumentDatabaseClient { collections }
    }

    pub async fn find(&self, collection_name: &str, filter_criteria: &Document, sort_field: &str) -> Vec<Document> {
        let collections: MutexGuard<'_, Collections> = self.lock_collections();
        let mut documents: Vec<Document> = collections
            .get(collection_name)
            .into_iter()
            .flatten()
            .filter(|document: &&Document| InMemoryDocumentDatabaseClient::matches(document, filter_criteria))
            .cloned()
            .collect();

        documents.sort_by(|left: &Document, right: &Document| {
            left.get(sort_field)
                .map_or("", String::as_str)
                .cmp(right.get(sort_field).map_or("", String::as_str))
        });
        documents
    }

    pub async fn insert_one(&self, collection_name: &str, document: Document) {
        self.lock_collections()
            .entry(collection_name.to_string())
            .or_default()
            .push(document);
    }

    pub async fn delete_many(&self, collection_name: &str, filter_criteria: &Document) {
        if let Some(documents) = self.lock_collections().get_mut(collection_name) {
            documents.retain(|document: &Document| !InMemoryDocumentDatabaseClient::matches(document, filter_criteria));
        }
    }

    pub async fn close(&self) {
        // Nothing to disconnect from; the documents stay in the store, as they would on a server.
    }

    fn lock_collections(&self) -> MutexGuard<'_, Collections> {
        self.collections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn matches(document: &Document, filter_criteria: &Document) -> bool {
        filter_criteria
            .iter()
            .all(|(field, value): (&String, &String)| document.get(field) == Some(value))
    }
}
