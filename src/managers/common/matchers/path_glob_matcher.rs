use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;

// Matches paths relative to a workspace (forward slashes) against a stack pack's globs (e.g. "src/**/*.rs").
pub struct PathGlobMatcher;

impl PathGlobMatcher {
    pub fn build(globs: &[String]) -> ApplicationResult<GlobSet> {
        let mut glob_set_builder: GlobSetBuilder = GlobSetBuilder::new();
        for glob in globs {
            glob_set_builder.add(
                Glob::new(glob)
                    .map_err(|exception: globset::Error| PathGlobMatcher::to_invalid_stack_pack(glob, &exception))?,
            );
        }
        glob_set_builder
            .build()
            .map_err(|exception: globset::Error| PathGlobMatcher::to_invalid_stack_pack(&globs.join(", "), &exception))
    }

    /// The paths any of the globs match, in the order given.
    pub fn select(paths: &[String], globs: &[String]) -> ApplicationResult<Vec<String>> {
        let glob_set: GlobSet = PathGlobMatcher::build(globs)?;
        Ok(paths
            .iter()
            .filter(|path: &&String| glob_set.is_match(path.as_str()))
            .cloned()
            .collect())
    }

    fn to_invalid_stack_pack(glob: &str, exception: &globset::Error) -> ApplicationBaseException {
        ApplicationBaseException::new(
            ExceptionKind::InvalidStackPack,
            "A glob of the workspace's stack pack is not valid",
            ContextualData::new()
                .with("glob", glob)
                .with("errors", exception.to_string()),
        )
    }
}
