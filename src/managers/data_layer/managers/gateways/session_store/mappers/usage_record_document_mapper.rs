use chrono::SecondsFormat;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::usage_record_document::UsageRecordDocument;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::usage_record::UsageRecord;

pub struct UsageRecordDocumentMapper;

impl UsageRecordDocumentMapper {
    pub fn to_usage_record_document(usage_record: &UsageRecord) -> UsageRecordDocument {
        UsageRecordDocument {
            session_id: Some(usage_record.session_id.clone()),
            response_id: Some(usage_record.response_id.clone()),
            model: Some(usage_record.model.clone()),
            requested_service_tier: Some(usage_record.requested_service_tier.name().to_string()),
            served_service_tier: usage_record
                .served_service_tier
                .map(|service_tier: ServiceTier| service_tier.name().to_string()),
            input_tokens: Some(usage_record.usage.input_tokens),
            cached_input_tokens: Some(usage_record.usage.cached_input_tokens),
            cache_write_tokens: Some(usage_record.usage.cache_write_tokens),
            output_tokens: Some(usage_record.usage.output_tokens),
            reasoning_tokens: Some(usage_record.usage.reasoning_tokens),
            recorded_on: Some(usage_record.recorded_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}
