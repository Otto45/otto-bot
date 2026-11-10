use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StartedAppResponse {
    /// The app's kebab-case name, which addresses it from now on (e.g. orders-app).
    pub name: String,
    /// The app's name as a title (e.g. Orders App).
    pub title: String,
    pub stack: String,
    /// How many files it started with, from its stack's template.
    pub file_count: usize,
    /// Whether the stack's formatters ran over it.
    pub formatted: bool,
}
