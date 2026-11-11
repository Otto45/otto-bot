use std::fmt::Display;

// Turns the error that deserializing something from outside the process failed with into one string, naming it by
// where in the input it is. Errors about the input as a whole have no location, so they are named after root_name
// (e.g. "body", "row", "document").
//
// There is a function per kind of input: JSON (an HTTP body, a document) fails with serde's error, located by
// serde_path_to_error, and a database row with the driver's. Both stop at the first error.
pub struct ValidationErrorFormatter;

impl ValidationErrorFormatter {
    pub fn format(error: &serde_path_to_error::Error<serde_json::Error>, root_name: &str) -> String {
        // An empty path prints as ".".
        let location: String = error.path().to_string();
        if location == "." {
            return ValidationErrorFormatter::format_whole(error.inner(), root_name);
        }
        format!("{location}: {}", error.inner())
    }

    pub fn format_row(error: &sqlx::Error, root_name: &str) -> String {
        match error {
            // The driver quotes the column's name.
            sqlx::Error::ColumnDecode { index, source } => format!("{}: {source}", index.trim_matches('"')),
            sqlx::Error::ColumnNotFound(column_name) => format!("{column_name}: column not found"),
            other => ValidationErrorFormatter::format_whole(other, root_name),
        }
    }

    pub fn format_whole(error: &dyn Display, root_name: &str) -> String {
        format!("{root_name}: {error}")
    }
}
