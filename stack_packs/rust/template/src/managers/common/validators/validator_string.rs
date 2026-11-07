// A stateless helper shared by every validator: a struct with associated functions only, never built and never
// registered in the ServiceLocator.
pub struct ValidatorString;

impl ValidatorString {
    pub fn validate(property_name: &str, property_value: Option<&str>) -> Option<String> {
        match property_value {
            None => Some(format!("{property_name} is missing")),
            Some("") => Some(format!("{property_name} is empty")),
            Some(value) if value.trim().is_empty() => Some(format!("{property_name} is whitespace")),
            Some(_) => None,
        }
    }
}
