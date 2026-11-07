use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    CheckResource, DetectResource, LintResource, LintRuleResource, ProjectMapResource, StackPackManifestResource,
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

        // Mapping and linting parse source files, which takes the pack's grammar.
        if (manifest.project_map.is_some() || manifest.lint.is_some())
            && let Some(error) = ValidatorString::validate("language", manifest.language.as_deref())
        {
            errors.push(error);
        }
        if let Some(project_map) = manifest.project_map.as_ref() {
            errors.extend(ValidatorStackPackManifestResource::validate_project_map_resource(
                project_map,
            ));
        }
        if let Some(lint) = manifest.lint.as_ref() {
            errors.extend(ValidatorStackPackManifestResource::validate_lint_resource(lint));
        }

        if let Some(scaffold) = manifest.scaffold.as_ref() {
            if let Some(error) = ValidatorString::validate("scaffold.kinds", scaffold.kinds.as_deref()) {
                errors.push(error);
            }
            if scaffold.format.is_none() {
                errors.push("scaffold.format is missing".to_string());
            }
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
        let mut errors: Vec<String> = Vec::new();
        if let Some(error) = ValidatorString::validate("project_map.query", project_map.query.as_deref()) {
            errors.push(error);
        }
        errors.extend(ValidatorStackPackManifestResource::validate_globs(
            "project_map.sources",
            project_map.sources.as_ref(),
        ));
        errors
    }

    fn validate_lint_resource(lint: &LintResource) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(error) = ValidatorString::validate("lint.imports_query", lint.imports_query.as_deref()) {
            errors.push(error);
        }
        errors.extend(ValidatorStackPackManifestResource::validate_globs(
            "lint.sources",
            lint.sources.as_ref(),
        ));
        for (index, lint_rule) in lint.rules.iter().flatten().enumerate() {
            errors.extend(ValidatorStackPackManifestResource::validate_lint_rule_resource(
                lint_rule, index,
            ));
        }
        errors
    }

    fn validate_lint_rule_resource(lint_rule: &LintRuleResource, index: usize) -> Vec<String> {
        let property_path = |property_name: &str| -> String { format!("lint.rules[{index}].{property_name}") };
        let mut errors: Vec<String> = [
            ("name", lint_rule.name.as_deref()),
            ("message", lint_rule.message.as_deref()),
            ("kind", lint_rule.kind.as_deref()),
        ]
        .into_iter()
        .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
            ValidatorString::validate(&property_path(property_name), property_value)
        })
        .collect();

        match lint_rule.kind.as_deref() {
            Some("imports") => {
                if let Some(error) = ValidatorString::validate(&property_path("files"), lint_rule.files.as_deref()) {
                    errors.push(error);
                }
                match lint_rule.forbid.as_ref() {
                    Some(forbid) if !forbid.is_empty() => {}
                    _ => errors.push(format!("{} is missing or empty", property_path("forbid"))),
                }
            }
            Some("query") | Some("named_after_file") => {
                if let Some(error) = ValidatorString::validate(&property_path("query"), lint_rule.query.as_deref()) {
                    errors.push(error);
                }
            }
            Some(_) => errors.push(format!(
                "{} is not one of imports, query, named_after_file",
                property_path("kind")
            )),
            None => {}
        }
        errors
    }

    fn validate_globs(property_name: &str, globs: Option<&Vec<String>>) -> Vec<String> {
        match globs {
            None => vec![format!("{property_name} is missing")],
            Some(globs) if globs.is_empty() => vec![format!("{property_name} is empty")],
            Some(_) => Vec::new(),
        }
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
