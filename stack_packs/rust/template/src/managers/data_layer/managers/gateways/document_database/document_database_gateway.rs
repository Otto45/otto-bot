use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::{
    Document, InMemoryDocumentDatabaseClient,
};
use crate::managers::data_layer::managers::gateways::document_database::mappers::audit_event_mapper::AuditEventMapper;
use crate::managers::data_layer::managers::gateways::document_database::query_builders::audit_event_query_builder::AuditEventQueryBuilder;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::audit_event_document::AuditEventDocument;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::document_find_query::DocumentFindQuery;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::document_insert_query::DocumentInsertQuery;
use crate::managers::data_layer::managers::gateways::document_database::validators::validator_audit_event_document::ValidatorAuditEventDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::document_database_gateway_exception::DocumentDatabaseGatewayException;
use crate::managers::models::audit_event::AuditEvent;

pub struct DocumentDatabaseGateway {
    document_database_client: InMemoryDocumentDatabaseClient,
}

impl DocumentDatabaseGateway {
    pub fn new(document_database_client: InMemoryDocumentDatabaseClient) -> Self {
        DocumentDatabaseGateway {
            document_database_client,
        }
    }

    pub async fn get_audit_events(&self, customer_id: &str) -> ApplicationResult<Vec<AuditEvent>> {
        let context: ContextualData = ContextualData::new().with("customer_id", customer_id);
        let document_find_query: DocumentFindQuery = AuditEventQueryBuilder::build_get_audit_events_query(customer_id);

        let documents: Vec<Document> = self
            .document_database_client
            .find(
                document_find_query.collection_name,
                &document_find_query.filter_criteria,
                document_find_query.sort_field,
            )
            .await;

        documents
            .into_iter()
            .map(|document: Document| DocumentDatabaseGateway::to_audit_event(document, &context))
            .collect()
    }

    pub async fn save_audit_event(&self, audit_event: &AuditEvent) -> ApplicationResult<()> {
        let document_insert_query: DocumentInsertQuery =
            AuditEventQueryBuilder::build_save_audit_event_query(audit_event);
        self.document_database_client
            .insert_one(document_insert_query.collection_name, document_insert_query.document)
            .await;
        Ok(())
    }

    pub async fn close(&self) {
        self.document_database_client.close().await;
    }

    fn to_audit_event(document: Document, context: &ContextualData) -> ApplicationResult<AuditEvent> {
        let document_value: serde_json::Value = serde_json::Value::Object(
            document
                .into_iter()
                .map(|(field, value): (String, String)| (field, value.into()))
                .collect(),
        );

        let audit_event_document: AuditEventDocument = serde_path_to_error::deserialize(document_value).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                let errors: String = ValidationErrorFormatter::format(&exception, "document");
                ApplicationBaseException::new(
                    DocumentDatabaseGatewayException::InvalidAuditEventDocument,
                    "Audit event document from the document database audit_events collection could not be deserialized",
                    context.clone().with("errors", errors),
                )
                .with_source(exception)
            },
        )?;

        ValidatorAuditEventDocument::validate_audit_event_document(&audit_event_document)?;
        Ok(AuditEventMapper::to_audit_event(audit_event_document))
    }
}
