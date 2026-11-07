use std::path::Path;

use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    CheckResource, DetectResource, StackPackManifestResource,
};
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_check::StackPackCheck;

pub struct StackPackMapper;

impl StackPackMapper {
    /// Expects a manifest that has already passed ValidatorStackPackManifestResource: each `expect` states a checked
    /// fact, and fails only if the validator is wrong.
    pub fn to_stack_pack(manifest: StackPackManifestResource, pack_directory: &Path) -> StackPack {
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
        }
    }
}
