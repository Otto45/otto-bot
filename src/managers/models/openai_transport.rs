/// How requests reach the OpenAI Responses API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiTransport {
    /// One request per model turn, each carrying the whole conversation.
    Http,
    /// A persistent connection per lane: a turn that continues the lane's last response sends only the new items.
    WebSocket,
}

impl OpenAiTransport {
    pub fn from_name(name: &str) -> Option<OpenAiTransport> {
        match name {
            "http" => Some(OpenAiTransport::Http),
            "websocket" => Some(OpenAiTransport::WebSocket),
            _ => None,
        }
    }
}
