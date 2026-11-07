// Only the gateway and its clients (which the ServiceLocator's seams build) are public: its request builders, resource
// models, validators and mappers are private to this module, so the compiler keeps resource models from leaving it.
pub mod clients;
mod mappers;
pub mod openai_responses_gateway;
mod request_builders;
mod resource_models;
mod validators;
