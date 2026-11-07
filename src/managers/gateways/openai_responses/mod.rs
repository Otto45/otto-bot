// Only the gateway is public: its request builders, resource models, validators and mappers are private to this
// module, so the compiler keeps resource models from leaving their gateway.
mod mappers;
pub mod openai_responses_gateway;
mod request_builders;
mod resource_models;
mod validators;
