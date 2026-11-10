use std::process::ExitCode;

fn main() -> ExitCode {
    otto_bot::services::http::export_openapi::main()
}
