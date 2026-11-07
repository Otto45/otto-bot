use chrono::SecondsFormat;

use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::Document;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::document_find_query::DocumentFindQuery;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::document_insert_query::DocumentInsertQuery;
use crate::managers::models::audit_event::AuditEvent;

const AUDIT_EVENTS_COLLECTION_NAME: &str = "audit_events";

pub struct AuditEventQueryBuilder;

impl AuditEventQueryBuilder {
    pub fn build_get_audit_events_query(customer_id: &str) -> DocumentFindQuery {
        DocumentFindQuery {
            collection_name: AUDIT_EVENTS_COLLECTION_NAME,
            filter_criteria: Document::from([("customer_id".to_string(), customer_id.to_string())]),
            sort_field: "occurred_on",
        }
    }

    pub fn build_save_audit_event_query(audit_event: &AuditEvent) -> DocumentInsertQuery {
        DocumentInsertQuery {
            collection_name: AUDIT_EVENTS_COLLECTION_NAME,
            document: Document::from([
                ("event_id".to_string(), audit_event.id.clone()),
                ("customer_id".to_string(), audit_event.customer_id.clone()),
                ("description".to_string(), audit_event.description.clone()),
                (
                    "occurred_on".to_string(),
                    audit_event.occurred_on.to_rfc3339_opts(SecondsFormat::AutoSi, false),
                ),
            ]),
        }
    }
}
