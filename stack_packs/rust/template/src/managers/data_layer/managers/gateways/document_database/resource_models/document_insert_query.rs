use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentInsertQuery {
    pub collection_name: &'static str,
    pub document: Document,
}
