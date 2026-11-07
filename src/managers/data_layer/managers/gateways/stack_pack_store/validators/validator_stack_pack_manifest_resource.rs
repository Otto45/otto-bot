use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    CheckResource, DetectResource, ProjectMapResource, StackPackManifestResource,
};
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::stack_pack_store_gateway_exception::StackPackStoreGatewayException;

pub struct ValidatorStackPackManifestResource;

impl ValidatorStackPackManifestResource {
    pub fn validate_stack_pack_manifest_resource(
        manifest: &StackPackManifestResource,
        pack_directory_name: &str,
    ) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        for (property_name, property_value) in
            [("name", manifest.name.as_deref()), ("rules", manifest.rules.as_deref())]
        {
            if let Some(error) = ValidatorString::validate(property_name, property_value) {
                errors.push(error);
            }
        }

        match manifest
            .detect
            .as_ref()
            .and_then(|detect: &DetectResource| detect.files.as_ref())
        {
            None => errors.push("detect.files is missing".to_string()),
            Some(files) if files.is_empty() => errors.push("detect.files is empty".to_string()),
            Some(_) => {}
        }

        if let Some(project_map) = manifest.project_map.as_ref() {
            errors.extend(ValidatorStackPackManifestResource::validate_project_map_resource(
                project_map,
            ));
        }

        for (index, check) in manifest.checks.iter().flatten().enumerate() {
            errors.extend(ValidatorStackPackManifestResource::validate_check_resource(
                check, index,
            ));
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                StackPackStoreGatewayException::InvalidStackPackManifest,
                "A stack pack's manifest is invalid",
                ContextualData::new()
                    .with("stack_pack", pack_directory_name)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }

    fn validate_project_map_resource(project_map: &ProjectMapResource) -> Vec<String> {
        let mut errors: Vec<String> = [
            ("project_map.language", project_map.language.as_deref()),
            ("project_map.query", project_map.query.as_deref()),
        ]
        .into_iter()
        .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
            ValidatorString::validate(property_name, property_value)
        })
        .collect();
        match project_map.sources.as_ref() {
            None => errors.push("project_map.sources is missing".to_string()),
            Some(sources) if sources.is_empty() => errors.push("project_map.sources is empty".to_string()),
            Some(_) => {}
        }
        errors
    }

    fn validate_check_resource(check: &CheckResource, index: usize) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(error) = ValidatorString::validate(&format!("checks[{index}].name"), check.name.as_deref()) {
            errors.push(error);
        }
        // An empty command keeps a check's place without running anything (e.g. tests, until there are some).
        if check.command.is_none() {
            errors.push(format!("checks[{index}].command is missing"));
        }
        errors
    }
}
