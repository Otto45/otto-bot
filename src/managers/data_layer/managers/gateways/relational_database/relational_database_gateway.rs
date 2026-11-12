use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgArguments, PgConnection, PgPool, PgRow};
use sqlx::{Connection, FromRow, Postgres, Transaction};

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::common::validators::validator_session_id::ValidatorSessionId;
use crate::managers::data_layer::managers::gateways::relational_database::guards::advisory_lock_guard::AdvisoryLockGuard;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::advisory_lock_mapper::AdvisoryLockMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::conversation_item_mapper::ConversationItemMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::implementation_brief_count_mapper::ImplementationBriefCountMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::review_baseline_mapper::ReviewBaselineMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::run_record_mapper::RunRecordMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::session_mapper::SessionMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::session_project_maps_mapper::SessionProjectMapsMapper;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::transcript_entry_mapper::TranscriptEntryMapper;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::advisory_lock_query_builder::AdvisoryLockQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::conversation_item_query_builder::ConversationItemQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::implementation_brief_query_builder::ImplementationBriefQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::project_map_query_builder::ProjectMapQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::review_baseline_query_builder::ReviewBaselineQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::run_record_query_builder::RunRecordQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::session_query_builder::SessionQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::transcript_entry_query_builder::TranscriptEntryQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::usage_record_query_builder::UsageRecordQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::advisory_lock_row::AdvisoryLockRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::conversation_item_row::ConversationItemRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::implementation_brief_count_row::ImplementationBriefCountRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::review_baseline_row::ReviewBaselineRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::run_record_row::RunRecordRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_project_maps_row::SessionProjectMapsRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_row::SessionRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{SqlParameter, SqlQuery};
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::transcript_entry_row::TranscriptEntryRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_advisory_lock_row::ValidatorAdvisoryLockRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_conversation_item_row::ValidatorConversationItemRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_implementation_brief_count_row::ValidatorImplementationBriefCountRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_review_baseline_row::ValidatorReviewBaselineRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_run_record_row::ValidatorRunRecordRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_session_project_maps_row::ValidatorSessionProjectMapsRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_session_row::ValidatorSessionRow;
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_transcript_entry_row::ValidatorTranscriptEntryRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::run_record::RunRecord;
use crate::managers::models::session::Session;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::workspace_file::WorkspaceFile;
use crate::managers::runtime::held_lock::HeldLock;

type BoundQuery<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;

// How often a session's lock held elsewhere is tried again while it is waited for, as the lock store's file locks are.
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(250);

/// Keeps the sessions and their locks in Postgres, when otto-bot is given a database: the session store's records,
/// one table per file of a session's folder, only ever inserted, never updated; and the locks that keep runs apart,
/// as Postgres advisory locks, so processes on different machines sharing one database are kept apart as processes on
/// one machine are by the lock store's file locks.
pub struct RelationalDatabaseGateway {
    // The ServiceLocator builds the pool without connecting (its methods are synchronous); the pool opens its first
    // connection when the gateway first sends a statement. The schema is not the gateway's: it is migrated outside
    // otto-bot (migrations/, applied by dbmate).
    pool: PgPool,
}

impl RelationalDatabaseGateway {
    pub fn new(pool: PgPool) -> Self {
        RelationalDatabaseGateway { pool }
    }

    pub async fn save_session(&self, session: &Session) -> ApplicationResult<()> {
        self.execute(
            &SessionQueryBuilder::build_save_session_query(session),
            "saving a session",
            &ContextualData::new().with("session_id", session.id.as_str()),
        )
        .await
    }

    /// The session with the id; None if there is none, or the id couldn't be one (so nothing a client made up is
    /// sent to the database).
    pub async fn get_session(&self, session_id: &str) -> ApplicationResult<Option<Session>> {
        if !ValidatorSessionId::is_session_id(session_id) {
            return Ok(None);
        }
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let sql_query: SqlQuery = SessionQueryBuilder::build_get_session_query(session_id);
        let Some(row) = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, &sql_query, "reading a session", &context)
            })?
        else {
            return Ok(None);
        };
        RelationalDatabaseGateway::to_session(&row, &context).map(Some)
    }

    /// Every session kept, in no order. A session that can't be read is passed over, with a warning: one damaged
    /// session doesn't hide the others.
    pub async fn get_sessions(&self) -> ApplicationResult<Vec<Session>> {
        let context: ContextualData = ContextualData::new();
        let rows: Vec<PgRow> = self
            .fetch_all(
                &SessionQueryBuilder::build_get_sessions_query(),
                "listing the sessions",
                &context,
            )
            .await?;
        let mut sessions: Vec<Session> = Vec::new();
        for row in &rows {
            match RelationalDatabaseGateway::to_session(row, &context) {
                Ok(session) => sessions.push(session),
                Err(exception) => ApplicationLogger::log_exception(&exception, LogLevel::Warning),
            }
        }
        Ok(sessions)
    }

    /// Keeps something the person or the orchestrator said, after what was said before it.
    pub async fn append_transcript_entry(
        &self,
        session_id: &str,
        transcript_entry: &TranscriptEntry,
    ) -> ApplicationResult<()> {
        self.execute(
            &TranscriptEntryQueryBuilder::build_append_transcript_entry_query(session_id, transcript_entry),
            "appending to a session's transcript",
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// What the person and the orchestrator said in the session, oldest first; nothing before its first request.
    pub async fn get_transcript(&self, session_id: &str) -> ApplicationResult<Vec<TranscriptEntry>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let rows: Vec<PgRow> = self
            .fetch_all(
                &TranscriptEntryQueryBuilder::build_get_transcript_query(session_id),
                "reading a session's transcript",
                &context,
            )
            .await?;
        let mut transcript_entries: Vec<TranscriptEntry> = Vec::new();
        for row in &rows {
            let transcript_entry_row: TranscriptEntryRow = RelationalDatabaseGateway::from_row(
                row,
                RelationalDatabaseGatewayException::InvalidPostgresTranscriptEntryRow,
                "Transcript entry row from the Postgres transcript_entries table could not be deserialized",
                &context,
            )?;
            ValidatorTranscriptEntryRow::validate_transcript_entry_row(&transcript_entry_row, session_id)?;
            transcript_entries.push(TranscriptEntryMapper::to_transcript_entry(transcript_entry_row));
        }
        Ok(transcript_entries)
    }

    /// The lane's conversation as its next request sends it: from its last compaction on (everything before it is in
    /// the compaction), or all of it; none for a lane that hasn't started.
    pub async fn get_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        let context: ContextualData = ContextualData::new()
            .with("session_id", session_id)
            .with("lane_name", lane_name);
        let rows: Vec<PgRow> = self
            .fetch_all(
                &ConversationItemQueryBuilder::build_get_conversation_items_query(session_id, lane_name),
                "reading a lane's conversation",
                &context,
            )
            .await?;
        let mut conversation_items: Vec<ConversationItem> = Vec::new();
        for row in &rows {
            let conversation_item_row: ConversationItemRow = RelationalDatabaseGateway::from_row(
                row,
                RelationalDatabaseGatewayException::InvalidPostgresConversationItemRow,
                "Conversation item row from the Postgres conversation_items table could not be deserialized",
                &context,
            )?;
            ValidatorConversationItemRow::validate_conversation_item_row(
                &conversation_item_row,
                session_id,
                lane_name,
            )?;
            conversation_items.push(ConversationItemMapper::to_conversation_item(conversation_item_row));
        }
        Ok(conversation_items)
    }

    /// Appends the items in one transaction, so a run's items land together (a compaction and the role's
    /// configuration update after it, say), or none of them do.
    pub async fn append_conversation_items(
        &self,
        session_id: &str,
        lane_name: &str,
        conversation_items: &[ConversationItem],
    ) -> ApplicationResult<()> {
        let context: ContextualData = ContextualData::new()
            .with("session_id", session_id)
            .with("lane_name", lane_name);
        let sql_queries: Vec<SqlQuery> = conversation_items
            .iter()
            .map(|conversation_item: &ConversationItem| {
                ConversationItemQueryBuilder::build_append_conversation_item_query(
                    session_id,
                    lane_name,
                    &ConversationItemMapper::to_conversation_item_row(conversation_item),
                )
            })
            .collect();
        let Some(first_query) = sql_queries.first() else {
            return Ok(());
        };

        let mut transaction: Transaction<'static, Postgres> =
            self.pool.begin().await.map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(
                    exception,
                    first_query,
                    "starting to append to a lane's conversation",
                    &context,
                )
            })?;
        for sql_query in &sql_queries {
            RelationalDatabaseGateway::bind(sql_query)
                .execute(&mut *transaction)
                .await
                .map_err(|exception: sqlx::Error| {
                    RelationalDatabaseGateway::to_query_exception(
                        exception,
                        sql_query,
                        "appending to a lane's conversation",
                        &context,
                    )
                })?;
        }
        transaction.commit().await.map_err(|exception: sqlx::Error| {
            RelationalDatabaseGateway::to_query_exception(
                exception,
                first_query,
                "committing what was appended to a lane's conversation",
                &context,
            )
        })
    }

    pub async fn append_usage_record(&self, usage_record: &UsageRecord) -> ApplicationResult<()> {
        self.execute(
            &UsageRecordQueryBuilder::build_append_usage_record_query(usage_record),
            "appending to the usage ledger",
            &ContextualData::new().with("session_id", usage_record.session_id.as_str()),
        )
        .await
    }

    /// The project maps the session keeps; none before its first run.
    pub async fn get_session_project_maps(&self, session_id: &str) -> ApplicationResult<SessionProjectMaps> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let sql_query: SqlQuery = ProjectMapQueryBuilder::build_get_session_project_maps_query(session_id);
        let row: PgRow = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(
                    exception,
                    &sql_query,
                    "reading a session's project maps",
                    &context,
                )
            })?;
        let session_project_maps_row: SessionProjectMapsRow = RelationalDatabaseGateway::from_row(
            &row,
            RelationalDatabaseGatewayException::InvalidPostgresSessionProjectMapsRow,
            "Project maps row from the Postgres project_maps table could not be deserialized",
            &context,
        )?;
        ValidatorSessionProjectMapsRow::validate_session_project_maps_row(&session_project_maps_row, session_id)?;
        Ok(SessionProjectMapsMapper::to_session_project_maps(
            session_project_maps_row,
        ))
    }

    pub async fn append_project_map(
        &self,
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
    ) -> ApplicationResult<()> {
        self.execute(
            &ProjectMapQueryBuilder::build_append_project_map_query(
                session_id,
                project_map,
                project_map_role,
                Utc::now(),
            ),
            "keeping a session's project map",
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// Keeps a brief the harness accepted, after those before it.
    pub async fn append_implementation_brief(
        &self,
        session_id: &str,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<()> {
        self.execute(
            &ImplementationBriefQueryBuilder::build_append_implementation_brief_query(
                session_id,
                implementation_brief,
                Utc::now(),
            ),
            "keeping a session's brief",
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// How many briefs the session has accepted; what they say isn't read.
    pub async fn count_implementation_briefs(&self, session_id: &str) -> ApplicationResult<usize> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let sql_query: SqlQuery = ImplementationBriefQueryBuilder::build_count_implementation_briefs_query(session_id);
        let row: PgRow = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(
                    exception,
                    &sql_query,
                    "counting a session's briefs",
                    &context,
                )
            })?;
        let implementation_brief_count_row: ImplementationBriefCountRow = RelationalDatabaseGateway::from_row(
            &row,
            RelationalDatabaseGatewayException::InvalidPostgresImplementationBriefCountRow,
            "Brief count row from the Postgres implementation_briefs table could not be deserialized",
            &context,
        )?;
        ValidatorImplementationBriefCountRow::validate_implementation_brief_count_row(
            &implementation_brief_count_row,
            session_id,
        )?;
        Ok(ImplementationBriefCountMapper::to_implementation_brief_count(
            implementation_brief_count_row,
        ))
    }

    /// Keeps a step one of the session's runs got to, after those before it.
    pub async fn append_run_record(&self, session_id: &str, run_record: &RunRecord) -> ApplicationResult<()> {
        self.execute(
            &RunRecordQueryBuilder::build_append_run_record_query(session_id, run_record),
            "keeping a session's run record",
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// The steps the session's runs got to, oldest first; none before its first run.
    pub async fn get_run_records(&self, session_id: &str) -> ApplicationResult<Vec<RunRecord>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let rows: Vec<PgRow> = self
            .fetch_all(
                &RunRecordQueryBuilder::build_get_run_records_query(session_id),
                "reading a session's run records",
                &context,
            )
            .await?;
        let mut run_records: Vec<RunRecord> = Vec::new();
        for row in &rows {
            let run_record_row: RunRecordRow = RelationalDatabaseGateway::from_row(
                row,
                RelationalDatabaseGatewayException::InvalidPostgresRunRecordRow,
                "Run record row from the Postgres run_records table could not be deserialized",
                &context,
            )?;
            ValidatorRunRecordRow::validate_run_record_row(&run_record_row, session_id)?;
            run_records.push(RunRecordMapper::to_run_record(run_record_row));
        }
        Ok(run_records)
    }

    /// Keeps what the session's next review is made against: `files`, the workspace's text files as they were when
    /// everything in it had been reviewed; or None, once everything the session built has been reviewed.
    pub async fn append_review_baseline(
        &self,
        session_id: &str,
        files: Option<&[WorkspaceFile]>,
    ) -> ApplicationResult<()> {
        self.execute(
            &ReviewBaselineQueryBuilder::build_append_review_baseline_query(
                session_id,
                files.map(ReviewBaselineMapper::to_files_text),
                Utc::now(),
            ),
            "keeping a session's review baseline",
            &ContextualData::new().with("session_id", session_id),
        )
        .await
    }

    /// What the session's next review is made against, as last kept; None when everything it built has been
    /// reviewed, or it has built nothing yet.
    pub async fn get_review_baseline(&self, session_id: &str) -> ApplicationResult<Option<Vec<WorkspaceFile>>> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let sql_query: SqlQuery = ReviewBaselineQueryBuilder::build_get_review_baseline_query(session_id);
        let Some(row) = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(
                    exception,
                    &sql_query,
                    "reading a session's review baseline",
                    &context,
                )
            })?
        else {
            return Ok(None);
        };
        let review_baseline_row: ReviewBaselineRow = RelationalDatabaseGateway::from_row(
            &row,
            RelationalDatabaseGatewayException::InvalidPostgresReviewBaselineRow,
            "Review baseline row from the Postgres review_baselines table could not be deserialized",
            &context,
        )?;
        ValidatorReviewBaselineRow::validate_review_baseline_row(&review_baseline_row, session_id)?;
        Ok(ReviewBaselineMapper::to_review_baseline(review_baseline_row))
    }

    /// The app's lock, if no run (in this process or another, on this machine or another) holds it: None when one
    /// does. `workspace_directory` is the app's resolved folder, so every name for it is one lock.
    pub async fn try_lock_app(&self, workspace_directory: &Path) -> ApplicationResult<Option<HeldLock>> {
        let lock_key: i64 = AdvisoryLockQueryBuilder::get_app_lock_key(workspace_directory);
        self.try_lock(lock_key, &ContextualData::new().with("lock_key", lock_key))
            .await
    }

    /// The session's lock, once nothing else holds it: a run of the session in another process (still compacting its
    /// conversation, say) is waited for, and so is a cancelled run's task until it has stopped. Waiting can be cancelled
    /// with the run waiting.
    pub async fn lock_session(&self, session_id: &str) -> ApplicationResult<HeldLock> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        let lock_key: i64 = AdvisoryLockQueryBuilder::get_session_lock_key(session_id);
        // One connection of its own asks until it has the lock: dropped while it waits (the run waiting was
        // cancelled), it is closed, and takes with it a lock Postgres had just granted.
        let mut connection: PgConnection = self.detach_connection(&context).await?;
        let mut waited: bool = false;
        loop {
            if RelationalDatabaseGateway::try_lock_on(&mut connection, lock_key, &context).await? {
                if waited {
                    ApplicationLogger::info("The session's lock is free again; its run goes on", context);
                }
                return Ok(HeldLock::new(Box::new(AdvisoryLockGuard::new(connection, lock_key))));
            }
            if !waited {
                ApplicationLogger::info(
                    "The session's lock is held elsewhere (its last run may still be compacting); its run waits",
                    context.clone(),
                );
                waited = true;
            }
            tokio::time::sleep(LOCK_RETRY_INTERVAL).await;
        }
    }

    /// Closes the pool's connections. A pool that never connected has none, and closing it does not connect. A held
    /// lock's connection is no longer the pool's, so its lock stays held until whoever holds it lets go.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Binds the query's parameters, in order, to its statement.
    pub(super) fn bind(sql_query: &SqlQuery) -> BoundQuery<'_> {
        let mut query: BoundQuery<'_> = sqlx::query(sql_query.statement);
        for parameter in &sql_query.parameters {
            query = match parameter {
                SqlParameter::Text(value) => query.bind(value.as_str()),
                SqlParameter::OptionalText(value) => query.bind(value.as_deref()),
                SqlParameter::TextArray(value) => query.bind(value.as_slice()),
                SqlParameter::OptionalTextArray(value) => query.bind(value.as_deref()),
                SqlParameter::BigInt(value) => query.bind(*value),
                SqlParameter::OptionalBoolean(value) => query.bind(*value),
                SqlParameter::Timestamp(value) => query.bind(*value),
            };
        }
        query
    }

    pub(super) fn to_query_exception(
        driver_error: sqlx::Error,
        sql_query: &SqlQuery,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        // Parameter values stay out of the context: they hold what the person said and the code the agent read.
        let mut exception_context: ContextualData = context
            .clone()
            .with(
                "error_type",
                RelationalDatabaseGateway::describe_error_type(&driver_error),
            )
            .with("postgres_error_message", driver_error.to_string())
            .with("sql_statement", sql_query.statement);

        // Only set when the error comes from Postgres itself (e.g. not for a refused connection): its SQLSTATE code.
        if let Some(postgres_error_code) = driver_error
            .as_database_error()
            .and_then(|database_error: &dyn sqlx::error::DatabaseError| database_error.code())
        {
            exception_context.insert("postgres_error_code", postgres_error_code.into_owned());
        }

        ApplicationBaseException::new(
            RelationalDatabaseGatewayException::PostgresQuery,
            format!("Postgres query failed while {operation_description}"),
            exception_context,
        )
        .with_source(driver_error)
    }

    // Takes the lock on a connection of its own, taken out of the pool before it asks, so that the lock can never go
    // back to the pool with it: whatever happens next (the lock is refused, or whoever asked is cancelled before the
    // answer), a connection that isn't kept is closed, and Postgres frees what it held.
    async fn try_lock(&self, lock_key: i64, context: &ContextualData) -> ApplicationResult<Option<HeldLock>> {
        let mut connection: PgConnection = self.detach_connection(context).await?;
        if RelationalDatabaseGateway::try_lock_on(&mut connection, lock_key, context).await? {
            return Ok(Some(HeldLock::new(Box::new(AdvisoryLockGuard::new(
                connection, lock_key,
            )))));
        }
        connection.close().await.ok();
        Ok(None)
    }

    async fn detach_connection(&self, context: &ContextualData) -> ApplicationResult<PgConnection> {
        let sql_query: SqlQuery = AdvisoryLockQueryBuilder::build_try_lock_query(0);
        let connection: PoolConnection<Postgres> = self.pool.acquire().await.map_err(|exception: sqlx::Error| {
            RelationalDatabaseGateway::to_query_exception(exception, &sql_query, "taking a lock", context)
        })?;
        Ok(connection.detach())
    }

    // Whether the connection now holds the lock.
    async fn try_lock_on(
        connection: &mut PgConnection,
        lock_key: i64,
        context: &ContextualData,
    ) -> ApplicationResult<bool> {
        let sql_query: SqlQuery = AdvisoryLockQueryBuilder::build_try_lock_query(lock_key);
        let row: PgRow = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_one(&mut *connection)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, &sql_query, "taking a lock", context)
            })?;
        let advisory_lock_row: AdvisoryLockRow = RelationalDatabaseGateway::from_row(
            &row,
            RelationalDatabaseGatewayException::InvalidPostgresAdvisoryLockRow,
            "Advisory lock row from Postgres could not be deserialized",
            context,
        )?;
        ValidatorAdvisoryLockRow::validate_advisory_lock_row(&advisory_lock_row)?;
        Ok(AdvisoryLockMapper::to_locked(advisory_lock_row))
    }

    async fn execute(
        &self,
        sql_query: &SqlQuery,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<()> {
        RelationalDatabaseGateway::bind(sql_query)
            .execute(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, sql_query, operation_description, context)
            })?;
        Ok(())
    }

    async fn fetch_all(
        &self,
        sql_query: &SqlQuery,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<Vec<PgRow>> {
        RelationalDatabaseGateway::bind(sql_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, sql_query, operation_description, context)
            })
    }

    fn to_session(row: &PgRow, context: &ContextualData) -> ApplicationResult<Session> {
        let session_row: SessionRow = RelationalDatabaseGateway::from_row(
            row,
            RelationalDatabaseGatewayException::InvalidPostgresSessionRow,
            "Session row from the Postgres sessions table could not be deserialized",
            context,
        )?;
        ValidatorSessionRow::validate_session_row(&session_row)?;
        Ok(SessionMapper::to_session(session_row))
    }

    fn from_row<T>(
        row: &PgRow,
        exception_kind: RelationalDatabaseGatewayException,
        message: &str,
        context: &ContextualData,
    ) -> ApplicationResult<T>
    where
        T: for<'row> FromRow<'row, PgRow>,
    {
        T::from_row(row).map_err(|exception: sqlx::Error| {
            let errors: String = ValidationErrorFormatter::format_row(&exception, "row");
            ApplicationBaseException::new(exception_kind, message, context.clone().with("errors", errors))
                .with_source(exception)
        })
    }

    // sqlx has one error type; what kind of failure it was is which of its variants it is.
    fn describe_error_type(driver_error: &sqlx::Error) -> &'static str {
        match driver_error {
            sqlx::Error::Configuration(_) => "ConfigurationError",
            sqlx::Error::Database(_) => "DatabaseError",
            sqlx::Error::Io(_) => "IoError",
            sqlx::Error::Tls(_) => "TlsError",
            sqlx::Error::Protocol(_) => "ProtocolError",
            sqlx::Error::PoolTimedOut => "PoolTimedOutError",
            sqlx::Error::PoolClosed => "PoolClosedError",
            sqlx::Error::WorkerCrashed => "WorkerCrashedError",
            _ => "Error",
        }
    }
}
