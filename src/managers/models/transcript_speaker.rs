/// Who said something in a session's conversation: the person, or otto (its orchestrator).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptSpeaker {
    Person,
    Assistant,
}

impl TranscriptSpeaker {
    pub fn from_name(name: &str) -> Option<TranscriptSpeaker> {
        match name {
            "person" => Some(TranscriptSpeaker::Person),
            "assistant" => Some(TranscriptSpeaker::Assistant),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TranscriptSpeaker::Person => "person",
            TranscriptSpeaker::Assistant => "assistant",
        }
    }
}
