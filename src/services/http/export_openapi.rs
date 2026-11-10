use std::process::ExitCode;

use crate::services::http::http_service::HttpService;

// The service's contract: every route, with the wire models it takes and answers with, as an OpenAPI document utoipa
// builds from the routers. It is kept in the project so that a change to the API shows up as a change to this file,
// and so that whatever calls the API can check itself against it.
const CONTRACT_PATH: &str = "openapi.json";

/// Writes the contract (`cargo run --bin export-openapi`), or with `--check` fails when the file is out of date.
pub fn main() -> ExitCode {
    let contract_value: serde_json::Value =
        serde_json::to_value(HttpService::get_contract()).expect("the contract serializes to JSON");
    // serde_json keeps an object's keys sorted, so the file only changes when the contract does.
    let contract: String =
        serde_json::to_string_pretty(&contract_value).expect("the contract serializes to JSON") + "\n";

    if !std::env::args().skip(1).any(|argument: String| argument == "--check") {
        if let Err(error) = std::fs::write(CONTRACT_PATH, contract) {
            eprintln!("{CONTRACT_PATH} could not be written: {error}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    if std::fs::read_to_string(CONTRACT_PATH).ok().as_deref() != Some(contract.as_str()) {
        eprintln!("{CONTRACT_PATH} is out of date: run `cargo run --bin export-openapi`");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
