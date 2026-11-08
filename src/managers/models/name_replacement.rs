/// A name in a stack pack's template and what starting an app replaces it with: a text holding placeholders for the
/// app's name ({app_name_kebab}, {app_name_snake}, {app_name_pascal}, {app_name_upper}, {app_name_title}).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameReplacement {
    pub name: String,
    pub replacement: String,
}
