/// What a person asks a session to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRequest {
    pub text: String,
}
