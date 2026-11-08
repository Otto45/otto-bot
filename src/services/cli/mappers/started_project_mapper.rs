use crate::managers::models::started_project::StartedProject;

// What the terminal shows for an app `otto new` started.
pub struct StartedProjectMapper;

impl StartedProjectMapper {
    pub fn to_output(started_project: &StartedProject) -> String {
        let formatted: &str = if started_project.formatted {
            ""
        } else {
            "\n(its formatters didn't all run: the first run of the checks will say why)"
        };
        format!(
            "started {} ({} app, {} files from its template) in {}{formatted}\n",
            started_project.app_name.title,
            started_project.stack_name,
            started_project.file_count,
            started_project.workspace_directory.display()
        )
    }
}
