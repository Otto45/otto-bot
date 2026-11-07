use std::collections::{HashMap, HashSet};

use globset::GlobSet;

use crate::managers::common::matchers::path_glob_matcher::PathGlobMatcher;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::stack_pack_layout::StackPackLayout;
use crate::managers::models::work_package::WorkPackage;
use crate::managers::models::work_plan::WorkPlan;

// The lane of the orchestrator, which no package may be named after.
const ORCHESTRATOR_LANE_NAME: &str = "orchestrator";
// More packages than this don't build faster: the machine's builds and the orchestrator's integration are the limit.
const MAX_PACKAGES: usize = 12;

// Checks an orchestrator's work plan without a model, so the workers it starts can build at the same time without stepping on
// each other: every package owns files no other does, all of them where a worker's files may be, and none of them
// the wiring or contracts the orchestrator writes. Every problem is reported at once, so the orchestrator fixes them in one go.
pub struct WorkPlanProcessor;

impl WorkPlanProcessor {
    /// `created_paths` are the files the plan's components are scaffolded into; `component_errors` what is wrong with
    /// the components themselves (a kind the stack doesn't have, a file that exists), found by scaffolding them on
    /// paper. A rejected plan is an InvalidWorkPlan, whose message the orchestrator is told.
    pub fn check(
        &self,
        work_plan: &WorkPlan,
        stack_pack_layout: &StackPackLayout,
        created_paths: &[String],
        component_errors: Vec<String>,
    ) -> ApplicationResult<()> {
        let orchestrator_glob_set: GlobSet = PathGlobMatcher::build(&stack_pack_layout.orchestrator_globs)?;
        let worker_glob_set: GlobSet = PathGlobMatcher::build(&stack_pack_layout.worker_globs)?;
        let is_workers: &dyn Fn(&str) -> bool =
            &|path: &str| worker_glob_set.is_match(path) && !orchestrator_glob_set.is_match(path);

        let mut errors: Vec<String> = Vec::new();
        if work_plan.summary.trim().is_empty() {
            errors.push("summary is empty".to_string());
        }
        if work_plan.packages.len() < 2 {
            errors.push(
                "a plan needs at least two packages: a change one agent can build, build yourself instead".to_string(),
            );
        }
        if work_plan.packages.len() > MAX_PACKAGES {
            errors.push(format!("a plan has at most {MAX_PACKAGES} packages"));
        }
        errors.extend(WorkPlanProcessor::check_components(&work_plan.components));
        errors.extend(component_errors);

        // Each path's owner, so a path owned twice is reported once, with both owners.
        let mut owners: HashMap<&str, &str> = HashMap::new();
        let mut package_names: HashSet<&str> = HashSet::new();
        for work_package in &work_plan.packages {
            errors.extend(WorkPlanProcessor::check_package_fields(work_package));
            if !package_names.insert(work_package.name.as_str()) {
                errors.push(format!("two packages are named {}", work_package.name));
            }
            for owned_path in &work_package.owned_paths {
                if !WorkPlanProcessor::is_normal_path(owned_path) {
                    errors.push(format!(
                        "{owned_path} (in {}) is not a plain path relative to the workspace",
                        work_package.name
                    ));
                } else if orchestrator_glob_set.is_match(owned_path) {
                    errors.push(format!(
                        "{owned_path} (in {}) is wiring or a contract, which you write before the workers start, so no \
                         package may own it",
                        work_package.name
                    ));
                } else if !worker_glob_set.is_match(owned_path) {
                    errors.push(format!(
                        "{owned_path} (in {}) is not where a worker's files go; those are {}",
                        work_package.name,
                        stack_pack_layout.worker_globs.join(", ")
                    ));
                }
                match owners.get(owned_path.as_str()) {
                    Some(owner) if *owner != work_package.name => errors.push(format!(
                        "{owned_path} is owned by both {owner} and {}; a file has one owner",
                        work_package.name
                    )),
                    Some(_) => {}
                    None => {
                        owners.insert(owned_path.as_str(), work_package.name.as_str());
                    }
                }
            }
        }

        // A component's file a worker could own but none does would be left as its scaffold made it.
        let mut reported_paths: HashSet<&str> = HashSet::new();
        for created_path in created_paths {
            if is_workers(created_path)
                && !owners.contains_key(created_path.as_str())
                && reported_paths.insert(created_path.as_str())
            {
                errors.push(format!(
                    "{created_path} is created by a component of the plan, but no package owns it to write its logic"
                ));
            }
        }

        if errors.is_empty() {
            return Ok(());
        }
        Err(ApplicationBaseException::new(
            AgentRunEngineException::InvalidWorkPlan,
            format!(
                "The plan was not accepted:\n- {}\nFix all of these and submit the plan again.",
                errors.join("\n- ")
            ),
            ContextualData::new().with("error_count", i64::try_from(errors.len()).unwrap_or(i64::MAX)),
        ))
    }

    fn check_components(planned_components: &[PlannedComponent]) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        let mut seen: HashSet<(&str, &str, Option<&str>)> = HashSet::new();
        for planned_component in planned_components {
            let key: (&str, &str, Option<&str>) = (
                planned_component.kind.as_str(),
                planned_component.name.as_str(),
                planned_component.within.as_deref(),
            );
            if !seen.insert(key) {
                errors.push(format!(
                    "the {} {} is listed twice",
                    planned_component.kind, planned_component.name
                ));
            }
        }
        errors
    }

    fn check_package_fields(work_package: &WorkPackage) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        let is_snake_case: bool = work_package
            .name
            .chars()
            .next()
            .is_some_and(|character: char| character.is_ascii_lowercase())
            && work_package.name.chars().all(|character: char| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            });
        if !is_snake_case {
            errors.push(format!("the package name {:?} is not snake_case", work_package.name));
        } else if work_package.name == ORCHESTRATOR_LANE_NAME {
            errors.push(format!("no package may be named {ORCHESTRATOR_LANE_NAME}"));
        }
        if work_package.brief.trim().is_empty() {
            errors.push(format!("{} has no brief", work_package.name));
        }
        if work_package.owned_paths.is_empty() {
            errors.push(format!("{} owns no files", work_package.name));
        }
        errors
    }

    // Relative, with forward slashes and no `.` or `..` parts, as the workspace names its files.
    fn is_normal_path(path: &str) -> bool {
        !path.is_empty()
            && !path.starts_with('/')
            && !path.contains('\\')
            && path
                .split('/')
                .all(|segment: &str| !segment.is_empty() && segment != "." && segment != "..")
    }
}
