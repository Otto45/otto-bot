use std::process::ExitCode;

fn main() -> ExitCode {
    example_app::services::http::export_openapi::main()
}
