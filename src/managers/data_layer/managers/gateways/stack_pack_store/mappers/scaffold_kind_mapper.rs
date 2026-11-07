use std::collections::HashMap;

use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::scaffold_kind_resource::{
    ScaffoldFileResource, ScaffoldInsertionResource, ScaffoldKindResource,
};
use crate::managers::models::scaffold_file::ScaffoldFile;
use crate::managers::models::scaffold_insertion::ScaffoldInsertion;
use crate::managers::models::scaffold_kind::ScaffoldKind;

pub struct ScaffoldKindMapper;

impl ScaffoldKindMapper {
    /// Expects a kind that has already passed ValidatorScaffoldKindResource. `templates` holds the text of every
    /// template file the kind names, by the path it names it with, read by the gateway.
    pub fn to_scaffold_kind(
        scaffold_kind: ScaffoldKindResource,
        kind_name: &str,
        templates: &HashMap<String, String>,
    ) -> ScaffoldKind {
        ScaffoldKind {
            name: kind_name.to_string(),
            description: scaffold_kind
                .description
                .expect("validated by ValidatorScaffoldKindResource"),
            within_path: scaffold_kind.within,
            files: scaffold_kind
                .files
                .expect("validated by ValidatorScaffoldKindResource")
                .into_iter()
                .map(|file: ScaffoldFileResource| {
                    let template_path: String = file.template.expect("validated by ValidatorScaffoldKindResource");
                    ScaffoldFile {
                        path: file.path.expect("validated by ValidatorScaffoldKindResource"),
                        content: templates
                            .get(&template_path)
                            .cloned()
                            .expect("read by StackPackStoreGateway for every template path"),
                    }
                })
                .collect(),
            insertions: scaffold_kind
                .insertions
                .unwrap_or_default()
                .into_iter()
                .map(|insertion: ScaffoldInsertionResource| ScaffoldInsertion {
                    path: insertion.path.expect("validated by ValidatorScaffoldKindResource"),
                    before: insertion.before,
                    text: insertion.text.expect("validated by ValidatorScaffoldKindResource"),
                })
                .collect(),
        }
    }
}
