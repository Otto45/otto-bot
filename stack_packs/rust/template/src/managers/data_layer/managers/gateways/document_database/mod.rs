// The gateway and the client it drives are public (the ServiceLocator builds the client); its queries, documents,
// validators and mappers are private to this module, so the compiler keeps them from leaving their gateway.
pub mod clients;
pub mod document_database_gateway;
mod mappers;
mod query_builders;
mod resource_models;
mod validators;
