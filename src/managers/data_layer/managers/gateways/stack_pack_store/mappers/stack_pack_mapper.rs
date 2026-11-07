use std::path::Path;

use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    CheckResource, DetectResource, ProjectMapResource, StackPackManifestResource,
};
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_check::StackPackCheck;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;

pub struct StackPackMapper;

impl StackPackMapper {
    /// Expects a manifest that has already passed ValidatorStackPackManifestResource: each `expect` states a checked
    /// fact, and fails only if the validator is wrong. `project_map_query` is the text of the manifest's
    /// project_map.query file, read by the gateway.
    pub fn to_stack_pack(
        manifest: StackPackManifestResource,
        pack_directory: &Path,
        project_map_query: Option<String>,
    ) -> StackPack {
        let detect: DetectResource = manifest
            .detect
            .expect("validated by ValidatorStackPackManifestResource");

        StackPack {
            name: manifest.name.expect("validated by ValidatorStackPackManifestResource"),
            detect_files: detect.files.expect("validated by ValidatorStackPackManifestResource"),
            rules_path: pack_directory.join(manifest.rules.expect("validated by ValidatorStackPackManifestResource")),
            checks: manifest
                .checks
                .unwrap_or_default()
                .into_iter()
                .map(|check: CheckResource| StackPackCheck {
                    name: check.name.expect("validated by ValidatorStackPackManifestResource"),
                    command: check.command.expect("validated by ValidatorStackPackManifestResource"),
                })
                .collect(),
            project_map: manifest.project_map.zip(project_map_query).map(
                |(project_map, query): (ProjectMapResource, String)| StackPackProjectMap {
                    language: project_map
                        .language
                        .expect("validated by ValidatorStackPackManifestResource"),
                    source_globs: project_map
                        .sources
                        .expect("validated by ValidatorStackPackManifestResource"),
                    query,
                },
            ),
        }
    }
}
