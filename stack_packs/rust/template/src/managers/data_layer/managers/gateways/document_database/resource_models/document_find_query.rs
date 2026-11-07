use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFindQuery {
    pub collection_name: &'static str,
    pub filter_criteria: Document,
    pub sort_field: &'static str,
}
