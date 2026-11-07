use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::stack_pack_store::mappers::stack_pack_mapper::StackPackMapper;
use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    LintRuleResource, ProjectMapResource, StackPackManifestResource,
};
use crate::managers::data_layer::managers::gateways::stack_pack_store::validators::validator_stack_pack_manifest_resource::ValidatorStackPackManifestResource;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::stack_pack_store_gateway_exception::StackPackStoreGatewayException;
use crate::managers::models::stack_pack::StackPack;

const MANIFEST_FILE_NAME: &str = "pack.toml";

/// The stack packs: one folder each, named after its stack, holding a pack.toml. Read-only.
pub struct StackPackStoreGateway {
    stack_packs_directory: PathBuf,
}

impl StackPackStoreGateway {
    pub fn new(stack_packs_directory: PathBuf) -> Self {
        StackPackStoreGateway { stack_packs_directory }
    }

    /// Every pack, in the order of their folders' names, so which pack a workspace matches first never changes.
    pub async fn get_stack_packs(&self) -> ApplicationResult<Vec<StackPack>> {
        let context: ContextualData = ContextualData::new().with(
            "stack_packs_directory",
            self.stack_packs_directory.to_string_lossy().into_owned(),
        );

        let mut pack_directories: Vec<PathBuf> = Vec::new();
        let mut entries: tokio::fs::ReadDir =
            tokio::fs::read_dir(&self.stack_packs_directory)
                .await
                .map_err(|exception: std::io::Error| {
                    StackPackStoreGateway::to_io_exception(exception, "listing the stack packs", &context)
                })?;
        while let Some(entry) = entries.next_entry().await.map_err(|exception: std::io::Error| {
            StackPackStoreGateway::to_io_exception(exception, "listing the stack packs", &context)
        })? {
            let pack_directory: PathBuf = entry.path();
            if tokio::fs::try_exists(pack_directory.join(MANIFEST_FILE_NAME))
                .await
                .unwrap_or(false)
            {
                pack_directories.push(pack_directory);
            }
        }
        pack_directories.sort();

        let mut stack_packs: Vec<StackPack> = Vec::new();
        for pack_directory in pack_directories {
            stack_packs.push(self.get_stack_pack(&pack_directory).await?);
        }
        Ok(stack_packs)
    }

    /// The architecture's rules for the pack's apps.
    pub async fn get_rules(&self, stack_pack: &StackPack) -> ApplicationResult<String> {
        tokio::fs::read_to_string(&stack_pack.rules_path)
            .await
            .map_err(|exception: std::io::Error| {
                StackPackStoreGateway::to_io_exception(
                    exception,
                    "reading a stack pack's rules",
                    &ContextualData::new().with("stack_pack", stack_pack.name.as_str()),
                )
            })
    }

    /// Holds nothing open, so there is nothing to close.
    pub fn close(&self) {}

    async fn get_stack_pack(&self, pack_directory: &Path) -> ApplicationResult<StackPack> {
        let pack_directory_name: String = pack_directory
            .file_name()
            .map(|file_name: &std::ffi::OsStr| file_name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let context: ContextualData = ContextualData::new().with("stack_pack", pack_directory_name.as_str());

        let content: String = tokio::fs::read_to_string(pack_directory.join(MANIFEST_FILE_NAME))
            .await
            .map_err(|exception: std::io::Error| {
                StackPackStoreGateway::to_io_exception(exception, "reading a stack pack's manifest", &context)
            })?;
        let manifest: StackPackManifestResource = toml::from_str(&content).map_err(|exception: toml::de::Error| {
            ApplicationBaseException::new(
                StackPackStoreGatewayException::InvalidStackPackManifest,
                "A stack pack's manifest could not be deserialized",
                context
                    .clone()
                    .with("errors", ValidationErrorFormatter::format_whole(&exception, "manifest")),
            )
        })?;
        ValidatorStackPackManifestResource::validate_stack_pack_manifest_resource(&manifest, &pack_directory_name)?;

        let mut queries: HashMap<String, String> = HashMap::new();
        for query_path in StackPackStoreGateway::get_query_paths(&manifest) {
            let query: String = tokio::fs::read_to_string(pack_directory.join(&query_path))
                .await
                .map_err(|exception: std::io::Error| {
                    StackPackStoreGateway::to_io_exception(
                        exception,
                        "reading a stack pack's query",
                        &context.clone().with("query_path", query_path.as_str()),
                    )
                })?;
            queries.insert(query_path, query);
        }
        Ok(StackPackMapper::to_stack_pack(manifest, pack_directory, &queries))
    }

    // Every query file the manifest names.
    fn get_query_paths(manifest: &StackPackManifestResource) -> Vec<String> {
        let mut query_paths: Vec<String> = Vec::new();
        query_paths.extend(
            manifest
                .project_map
                .as_ref()
                .and_then(|project_map: &ProjectMapResource| project_map.query.clone()),
        );
        if let Some(lint) = manifest.lint.as_ref() {
            query_paths.extend(lint.imports_query.clone());
            query_paths.extend(
                lint.rules
                    .iter()
                    .flatten()
                    .filter_map(|lint_rule: &LintRuleResource| lint_rule.query.clone()),
            );
        }
        query_paths
    }

    fn to_io_exception(
        exception: std::io::Error,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        ApplicationBaseException::new(
            StackPackStoreGatewayException::StackPackStoreIo,
            format!("The stack pack store failed while {operation_description}"),
            context.clone().with("error_type", format!("{:?}", exception.kind())),
        )
        .with_source(exception)
    }
}
