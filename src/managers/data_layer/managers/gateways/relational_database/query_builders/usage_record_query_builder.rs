use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::usage_record::UsageRecord;

pub struct UsageRecordQueryBuilder;

impl UsageRecordQueryBuilder {
    pub fn build_append_usage_record_query(usage_record: &UsageRecord) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "INSERT INTO usage_records ",
                "(session_id, lane_name, role, response_id, model, requested_service_tier, served_service_tier, ",
                "input_tokens, cached_input_tokens, cache_write_tokens, output_tokens, reasoning_tokens, recorded_on) ",
                "VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            ),
            parameters: vec![
                SqlParameter::Text(usage_record.session_id.clone()),
                SqlParameter::Text(usage_record.lane_name.clone()),
                SqlParameter::Text(usage_record.role.name().to_string()),
                SqlParameter::Text(usage_record.response_id.clone()),
                SqlParameter::Text(usage_record.model.clone()),
                SqlParameter::Text(usage_record.requested_service_tier.name().to_string()),
                SqlParameter::OptionalText(
                    usage_record
                        .served_service_tier
                        .map(|service_tier: ServiceTier| service_tier.name().to_string()),
                ),
                SqlParameter::BigInt(usage_record.usage.input_tokens),
                SqlParameter::BigInt(usage_record.usage.cached_input_tokens),
                SqlParameter::BigInt(usage_record.usage.cache_write_tokens),
                SqlParameter::BigInt(usage_record.usage.output_tokens),
                SqlParameter::BigInt(usage_record.usage.reasoning_tokens),
                SqlParameter::Timestamp(usage_record.recorded_on),
            ],
        }
    }
}
