// Only the gateway is public: its resource models, validators and mappers are private to this module, so the
// compiler keeps resource models from leaving their gateway.
pub mod json_placeholder_gateway;
mod mappers;
mod resource_models;
mod validators;
