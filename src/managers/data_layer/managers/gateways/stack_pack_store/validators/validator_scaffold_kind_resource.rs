use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::scaffold_kind_resource::{
    ScaffoldFileResource, ScaffoldInsertionResource, ScaffoldKindResource,
};
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::stack_pack_store_gateway_exception::StackPackStoreGatewayException;

pub struct ValidatorScaffoldKindResource;

impl ValidatorScaffoldKindResource {
    pub fn validate_scaffold_kind_resource(
        scaffold_kind: &ScaffoldKindResource,
        kind_name: &str,
    ) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        if let Some(error) = ValidatorString::validate("description", scaffold_kind.description.as_deref()) {
            errors.push(error);
        }
        match scaffold_kind.files.as_ref() {
            Some(files) if !files.is_empty() => {
                for (index, file) in files.iter().enumerate() {
                    let file: &ScaffoldFileResource = file;
                    for (property_name, property_value) in
                        [("path", file.path.as_deref()), ("template", file.template.as_deref())]
                    {
                        if let Some(error) =
                            ValidatorString::validate(&format!("files[{index}].{property_name}"), property_value)
                        {
                            errors.push(error);
                        }
                    }
                }
            }
            _ => errors.push("files is missing or empty".to_string()),
        }
        for (index, insertion) in scaffold_kind.insertions.iter().flatten().enumerate() {
            let insertion: &ScaffoldInsertionResource = insertion;
            if let Some(error) =
                ValidatorString::validate(&format!("insertions[{index}].path"), insertion.path.as_deref())
            {
                errors.push(error);
            }
            if insertion.text.as_deref().is_none_or(str::is_empty) {
                errors.push(format!("insertions[{index}].text is missing or empty"));
            }
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                StackPackStoreGatewayException::InvalidStackPackManifest,
                "A stack pack's scaffold kind is invalid",
                ContextualData::new()
                    .with("scaffold_kind", kind_name)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
