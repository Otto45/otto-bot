// The longest a session's id may be: the ids made are 32 characters.
const MAX_SESSION_ID_CHARS: usize = 64;

// A stateless helper shared by the stores that keep sessions: whether a text could be a session's id, a word of
// lowercase letters and digits as the ids made are. One that couldn't names no session, so it is never used to name
// a folder, a file or a row.
pub struct ValidatorSessionId;

impl ValidatorSessionId {
    pub fn is_session_id(session_id: &str) -> bool {
        !session_id.is_empty()
            && session_id.len() <= MAX_SESSION_ID_CHARS
            && session_id
                .chars()
                .all(|character: char| character.is_ascii_lowercase() || character.is_ascii_digit())
    }
}
