/// The RelationalDatabaseGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalDatabaseGatewayException {
    /// A statement Postgres refused, or one that couldn't reach it (a refused connection, a pool that timed out).
    PostgresQuery,
    InvalidPostgresSessionRow,
    InvalidPostgresTranscriptEntryRow,
    InvalidPostgresConversationItemRow,
    InvalidPostgresSessionProjectMapsRow,
    InvalidPostgresImplementationBriefCountRow,
    InvalidPostgresAdvisoryLockRow,
    InvalidPostgresRunRecordRow,
    InvalidPostgresReviewBaselineRow,
}

impl RelationalDatabaseGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            RelationalDatabaseGatewayException::PostgresQuery => "PostgresQueryException",
            RelationalDatabaseGatewayException::InvalidPostgresSessionRow => "InvalidPostgresSessionRowException",
            RelationalDatabaseGatewayException::InvalidPostgresTranscriptEntryRow => {
                "InvalidPostgresTranscriptEntryRowException"
            }
            RelationalDatabaseGatewayException::InvalidPostgresConversationItemRow => {
                "InvalidPostgresConversationItemRowException"
            }
            RelationalDatabaseGatewayException::InvalidPostgresSessionProjectMapsRow => {
                "InvalidPostgresSessionProjectMapsRowException"
            }
            RelationalDatabaseGatewayException::InvalidPostgresImplementationBriefCountRow => {
                "InvalidPostgresImplementationBriefCountRowException"
            }
            RelationalDatabaseGatewayException::InvalidPostgresAdvisoryLockRow => {
                "InvalidPostgresAdvisoryLockRowException"
            }
            RelationalDatabaseGatewayException::InvalidPostgresRunRecordRow => "InvalidPostgresRunRecordRowException",
            RelationalDatabaseGatewayException::InvalidPostgresReviewBaselineRow => {
                "InvalidPostgresReviewBaselineRowException"
            }
        }
    }
}
