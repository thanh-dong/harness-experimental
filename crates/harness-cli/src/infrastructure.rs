use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use rusqlite::{params, types::ValueRef, Connection, OptionalExtension};
use serde_json::Value as JsonValue;
use thiserror::Error;

use crate::events::{
    fnv1a64, fnv1a64_continue, genesis_ulid, mint_ulid, rfc3339_from_unix, rfc3339_utc_now,
    unix_from_sqlite_datetime, EventLog,
};
use serde_json::json;

use crate::application::{
    BacklogAddInput, BacklogCloseInput, BrownfieldImportResult, DecisionAddInput,
    DecisionUpdateInput, DecisionVerifyResult, HarnessContext, InitResult, IntakeInput,
    InterventionAddInput, InterventionFilter, MigrateResult, QueryTable, StoryAddInput,
    StorySignalAddInput, StorySignalFilter, StoryUpdateInput, StoryVerifyResult, ToolRegisterInput,
    TraceInput,
};
use crate::domain::{
    compiled_tool_registry, normalize_token, score_context, score_trace, validate_tool_description,
    AuditFinding, AuditResult, BacklogFilter, BacklogRecord, ContextScoreResult,
    ContextScoreSource, CsvList, DecisionRecord, FrictionRecord, HarnessStats, ImprovementProposal,
    IntakeRecord, InterventionRecord, RiskLane, StoryMatrixRecord, StorySignalRecord,
    StoryVerifyAllItem, StoryVerifyAllResult, StoryVerifyStatus, ToolArgSpec, ToolEntry,
    TraceRecord, TraceScoreResult, TraceScoreSource,
};

pub type Result<T> = std::result::Result<T, HarnessInfraError>;

/// Highest schema version this binary can read and write. `migrate` refuses
/// newer migration files instead of applying SQL it cannot operate against
/// (a stale binary that migrates forward bricks its own write path).
pub const SUPPORTED_SCHEMA_VERSION: i64 = 8;

#[derive(Debug, Error)]
pub enum HarnessInfraError {
    #[error("database not found at {0}. Run: harness init")]
    MissingDatabase(String),
    #[error("schema migration v{0} is newer than this binary supports (v{1}). Update harness-cli before migrating.")]
    UnsupportedSchemaVersion(i64, i64),
    #[error("schema file missing: {0}")]
    MissingSchema(String),
    #[error("brownfield import: missing {0}")]
    MissingBrownfieldPath(String),
    #[error("decision {0} has no verify_command. Configure one with: harness-cli decision add --id {0} --title <title> --verify \"<command>\"")]
    MissingDecisionVerifyCommand(String),
    #[error("story {0} has no verify_command. Configure one with: harness-cli story update --id {0} --verify \"<command>\"")]
    MissingStoryVerifyCommand(String),
    #[error("story update: story '{0}' not found")]
    StoryNotFound(String),
    #[error("tool register: tool '{0}' already exists with command '{1}'")]
    ToolAlreadyExists(String, String),
    #[error("tool remove: tool '{0}' not found")]
    ToolNotFound(String),
    #[error("tool register: command '{0}' was not found. Re-run with --force to register anyway.")]
    ToolCommandNotFound(String),
    #[error("{0}")]
    ToolValidation(#[from] crate::domain::ToolValidationError),
    #[error("{0} id '{1}' not found")]
    RowIdNotFound(String, String),
    #[error("ambiguous {0} id prefix '{1}': matches {2}")]
    AmbiguousRowId(String, String, String),
    #[error("no traces found")]
    NoTraces,
    #[error("story update: nothing to update")]
    EmptyStoryUpdate,
    #[error("decision update: decision '{0}' not found")]
    DecisionNotFound(String),
    #[error("decision update: nothing to update")]
    EmptyDecisionUpdate,
    #[error("rebuild: corrupt event log {0}: {1}")]
    CorruptEventLog(String, String),
    #[error("migrate-to-events verification failed: {0}")]
    MigrationVerifyFailed(String),
    #[error("this database has rows but is not event-backed. Run: harness-cli migrate-to-events")]
    NotEventBacked,
    #[error("import brownfield is migration-only (decision 0008 Q4): it may only seed a database that is not yet event-backed")]
    BrownfieldOnEventBacked,
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Outcome of one `tool check` scan. The CLI reports these facts; the agent
/// applies policy (skip / degrade / use) based on `status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCheckResult {
    pub name: String,
    pub kind: String,
    pub capability: Option<String>,
    pub status: String,
    pub detail: String,
}

pub trait HarnessRepository {
    fn init(&self) -> Result<InitResult>;
    fn migrate(&self) -> Result<MigrateResult>;
    fn info(&self) -> Result<InfoReport>;
    fn import_brownfield(&self) -> Result<BrownfieldImportResult>;
    fn record_intake(&self, input: IntakeInput) -> Result<String>;
    fn add_story(&self, input: StoryAddInput) -> Result<()>;
    fn update_story(&self, input: StoryUpdateInput) -> Result<()>;
    fn verify_story(&self, id: &str) -> Result<StoryVerifyResult>;
    fn verify_all_stories(&self) -> Result<StoryVerifyAllResult>;
    fn add_decision(&self, input: DecisionAddInput) -> Result<()>;
    fn update_decision(&self, input: DecisionUpdateInput) -> Result<()>;
    fn verify_decision(&self, id: &str) -> Result<DecisionVerifyResult>;
    fn add_backlog(&self, input: BacklogAddInput) -> Result<String>;
    fn close_backlog(&self, input: BacklogCloseInput) -> Result<()>;
    fn register_tool(&self, input: ToolRegisterInput) -> Result<()>;
    fn remove_tool(&self, name: &str) -> Result<()>;
    fn check_tools(&self, name: Option<String>) -> Result<Vec<ToolCheckResult>>;
    fn add_intervention(&self, input: InterventionAddInput) -> Result<String>;
    fn record_trace(&self, input: TraceInput) -> Result<String>;
    fn score_trace(&self, id: Option<String>) -> Result<TraceScoreResult>;
    fn score_context(&self, id: &str) -> Result<ContextScoreResult>;
    fn story_verify_status(&self, id: &str) -> Result<StoryVerifyStatus>;
    fn query_matrix(&self) -> Result<Vec<StoryMatrixRecord>>;
    fn query_backlog(&self, filter: BacklogFilter) -> Result<Vec<BacklogRecord>>;
    fn query_decisions(&self) -> Result<Vec<DecisionRecord>>;
    fn query_intakes(&self) -> Result<Vec<IntakeRecord>>;
    fn query_traces(&self) -> Result<Vec<TraceRecord>>;
    fn query_friction(&self) -> Result<Vec<FrictionRecord>>;
    fn query_tools(
        &self,
        responsibility: Option<String>,
        capability: Option<String>,
    ) -> Result<Vec<ToolEntry>>;
    fn query_interventions(&self, filter: InterventionFilter) -> Result<Vec<InterventionRecord>>;
    fn add_story_signal(&self, input: StorySignalAddInput) -> Result<String>;
    fn query_story_signals(&self, filter: StorySignalFilter) -> Result<Vec<StorySignalRecord>>;
    fn query_stats(&self) -> Result<HarnessStats>;
    fn audit(&self) -> Result<AuditResult>;
    fn propose(&self, commit: bool) -> Result<Vec<ImprovementProposal>>;
    fn query_sql(&self, sql: &str) -> Result<QueryTable>;
}

/// Read-only snapshot of version and state for `harness-cli info` (US-036).
/// Built without mutating the cache or replaying the log, so it can report a
/// cache that is behind the log rather than silently healing it.
#[derive(Debug, PartialEq, Eq)]
pub struct InfoReport {
    /// Version of this CLI binary (Cargo package version).
    pub cli_version: String,
    /// Highest schema version this binary can read and write.
    pub supported_schema_version: i64,
    /// Highest migration version present on disk (what `migrate` targets).
    pub available_schema_version: i64,
    /// Event-log line format version this binary writes.
    pub event_format_version: i64,
    /// Whether a harness database exists at `db_path`.
    pub initialized: bool,
    /// Absolute path where the database is (or would be).
    pub db_path: String,
    /// Highest applied migration recorded in the database, or 0 when absent.
    pub applied_schema_version: i64,
    /// Every applied migration version, ascending (empty when uninitialized).
    pub applied_migrations: Vec<i64>,
    /// Whether the cache is marked event-backed (US-028b cutover).
    pub event_backed: bool,
    /// Number of `.jsonl` writer files in `.harness/events/`.
    pub event_files: usize,
    /// Applied schema is behind the migrations on disk: run `migrate`.
    pub schema_behind_cli: bool,
    /// The cache has unconsumed events in the log: a read command replays them.
    pub cache_behind_log: bool,
}

#[derive(Debug)]
pub struct SqliteHarnessRepository {
    repo_root: PathBuf,
    db_path: PathBuf,
    schema_dir: PathBuf,
    // US-028b: the event log is the write of record; the repository owns the
    // writer identity and appends before the cache commit.
    events: EventLog,
}

impl SqliteHarnessRepository {
    pub fn new(repo_root: PathBuf, db_path: PathBuf, schema_dir: PathBuf) -> Self {
        let events = EventLog::new(&repo_root);
        Self {
            repo_root,
            db_path,
            schema_dir,
            events,
        }
    }

    fn events_dir(&self) -> PathBuf {
        self.repo_root.join(".harness/events")
    }

    fn list_event_files(&self) -> Result<Vec<PathBuf>> {
        let events_dir = self.events_dir();
        let mut files = Vec::new();
        if events_dir.is_dir() {
            for entry in fs::read_dir(&events_dir)? {
                let path = entry?.path();
                if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                    files.push(path);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    fn ensure_event_backed(connection: &Connection) -> Result<()> {
        if Self::cache_meta_get(connection, "event_backed")?.as_deref() == Some("true") {
            Ok(())
        } else {
            Err(HarnessInfraError::NotEventBacked)
        }
    }

    /// The cutover write path: apply the event to the cache inside a
    /// transaction (SQLite constraints validate it before it can reach the
    /// log), append it to this writer's file (fsync), advance the watermark,
    /// then commit. A crash between append and commit leaves the cache behind
    /// the log — healed by watermark replay on the next command. The cache is
    /// only ever written from events, so cache and log cannot drift.
    fn append_and_apply(
        &self,
        connection: &Connection,
        op: &str,
        payload: JsonValue,
    ) -> Result<()> {
        Self::ensure_event_backed(connection)?;
        let tx = connection.unchecked_transaction()?;
        let file_name = self.events.file_name();
        let previous: Option<(i64, i64, String)> = tx
            .query_row(
                "SELECT consumed_count, file_size, content_hash FROM event_watermark WHERE file=?1;",
                params![file_name],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let (prev_count, prev_size, prev_hash) = match previous {
            Some((count, size, hash)) => (
                count,
                size,
                u64::from_str_radix(&hash, 16).unwrap_or(0xcbf2_9ce4_8422_2325),
            ),
            None => (0, 0, 0xcbf2_9ce4_8422_2325),
        };

        // Causal-audit signal (DKR-4): what this writer had consumed of every
        // writer's file when it appended.
        let mut observed = serde_json::Map::new();
        {
            let mut statement = tx.prepare("SELECT file, consumed_count FROM event_watermark;")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for (file, count) in collect_rows(rows)? {
                let writer = file.trim_end_matches(".jsonl").to_owned();
                observed.insert(writer, json!(count));
            }
        }

        let event = LogEvent {
            event_id: mint_ulid(),
            writer: self.events.writer().to_owned(),
            recorded_at: rfc3339_utc_now(),
            op: op.to_owned(),
            payload,
            observed: Some(JsonValue::Object(observed)),
            writer_seq: prev_count + 1,
        };
        apply_event(&tx, &event)?;

        let line = EventLog::event_line(
            &event.event_id,
            &event.writer,
            &event.recorded_at,
            &event.op,
            &event.payload,
            event.observed.as_ref(),
        );
        self.events.append_line(&line)?;
        let mut appended = line.into_bytes();
        appended.push(b'\n');
        let new_hash = fnv1a64_continue(prev_hash, &appended);
        let mtime_ns = file_mtime_ns(&self.events.own_file_path())?;
        Self::write_watermark(
            &tx,
            &file_name,
            prev_count + 1,
            prev_size + appended.len() as i64,
            mtime_ns,
            &format!("{new_hash:016x}"),
        )?;
        tx.commit()?;

        if matches!(
            event.op.as_str(),
            "story.add"
                | "story.update"
                | "story.verify_result"
                | "backlog.add"
                | "backlog.close"
                | "decision.add"
                | "decision.update"
                | "decision.verify_result"
        ) {
            self.regenerate_views(connection)?;
        }
        Ok(())
    }

    /// Record watermarks for every log file as fully consumed.
    fn write_watermarks_for_all_files(&self, connection: &Connection) -> Result<()> {
        connection.execute("DELETE FROM event_watermark;", [])?;
        for path in self.list_event_files()? {
            let bytes = fs::read(&path)?;
            let count = bytes
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .count() as i64;
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            Self::write_watermark(
                connection,
                &name,
                count,
                bytes.len() as i64,
                file_mtime_ns(&path)?,
                &format!("{:016x}", fnv1a64(&bytes)),
            )?;
        }
        Ok(())
    }

    /// Full rebuild of the primary cache from the log, marking it
    /// event-backed with fresh watermarks.
    fn rebuild_primary_cache(&self) -> Result<Connection> {
        let events = self.read_event_log()?;
        let connection = self.build_cache_from_events(&events, &self.db_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Self::cache_meta_set(&connection, "event_backed", "true")?;
        self.write_watermarks_for_all_files(&connection)?;
        self.regenerate_views(&connection)?;
        Ok(connection)
    }

    /// Every command goes through here (US-028b): compare per-file watermarks
    /// (stat short-circuit), incrementally replay new events (e.g. after
    /// `git pull`), rebuild from genesis when the log shrank or diverged, and
    /// auto-rebuild a missing cache (fresh clone).
    fn open_fresh(&self) -> Result<Connection> {
        if !self.db_path.exists() {
            if !self.list_event_files()?.is_empty() {
                return self.rebuild_primary_cache();
            }
            return Err(HarnessInfraError::MissingDatabase(
                self.db_path.display().to_string(),
            ));
        }

        let connection = self.open_existing()?;
        if Self::cache_meta_get(&connection, "event_backed")?.as_deref() != Some("true") {
            // Legacy cache: reads work as before; mutations are guarded by
            // ensure_event_backed and point at migrate-to-events.
            return Ok(connection);
        }

        let mut new_events: Vec<LogEvent> = Vec::new();
        let mut watermark_updates: Vec<(String, i64, i64, i64, String)> = Vec::new();
        let mut seen_files: Vec<String> = Vec::new();
        let mut need_rebuild = false;

        for path in self.list_event_files()? {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            seen_files.push(name.clone());
            let metadata = fs::metadata(&path)?;
            let size = metadata.len() as i64;
            let mtime_ns = file_mtime_ns(&path)?;
            let watermark: Option<(i64, i64, i64, String)> = connection
                .query_row(
                    "SELECT consumed_count, file_size, file_mtime_ns, content_hash
                     FROM event_watermark WHERE file=?1;",
                    params![name],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .optional()?;

            match watermark {
                Some((count, wm_size, wm_mtime, wm_hash)) => {
                    if wm_size == size && wm_mtime == mtime_ns {
                        continue; // stat short-circuit (DKR-2)
                    }
                    let bytes = fs::read(&path)?;
                    let consumed = wm_size.max(0) as usize;
                    if bytes.len() >= consumed
                        && format!("{:016x}", fnv1a64(&bytes[..consumed])) == wm_hash
                    {
                        // Incremental: only the appended suffix is new.
                        let display = path.display().to_string();
                        for (offset, line) in String::from_utf8_lossy(&bytes[consumed..])
                            .lines()
                            .enumerate()
                        {
                            if line.trim().is_empty() {
                                continue;
                            }
                            new_events.push(parse_event_line(
                                line,
                                &display,
                                count as usize + offset,
                            )?);
                        }
                        watermark_updates.push((
                            name,
                            count
                                + bytes[consumed..]
                                    .split(|byte| *byte == b'\n')
                                    .filter(|line| !line.is_empty())
                                    .count() as i64,
                            bytes.len() as i64,
                            mtime_ns,
                            format!("{:016x}", fnv1a64(&bytes)),
                        ));
                    } else {
                        // Shrank or diverged: the log is the truth — rebuild.
                        need_rebuild = true;
                        break;
                    }
                }
                None => {
                    // A new writer file (e.g. first pull from a teammate).
                    let bytes = fs::read(&path)?;
                    let display = path.display().to_string();
                    for (index, line) in String::from_utf8_lossy(&bytes).lines().enumerate() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        new_events.push(parse_event_line(line, &display, index)?);
                    }
                    watermark_updates.push((
                        name,
                        bytes
                            .split(|byte| *byte == b'\n')
                            .filter(|line| !line.is_empty())
                            .count() as i64,
                        bytes.len() as i64,
                        mtime_ns,
                        format!("{:016x}", fnv1a64(&bytes)),
                    ));
                }
            }
        }

        // A watermarked file that vanished means applied events no longer
        // exist in the log: rebuild from what remains.
        if !need_rebuild {
            let mut statement = connection.prepare("SELECT file FROM event_watermark;")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            for file in collect_rows(rows)? {
                if !seen_files.contains(&file) {
                    need_rebuild = true;
                    break;
                }
            }
        }

        if need_rebuild {
            drop(connection);
            return self.rebuild_primary_cache();
        }
        if new_events.is_empty() {
            return Ok(connection);
        }

        new_events.sort_by(|left, right| {
            (left.event_id.as_str(), left.writer.as_str())
                .cmp(&(right.event_id.as_str(), right.writer.as_str()))
        });
        // Replay tolerates cross-writer references arriving in log order.
        connection.pragma_update(None, "foreign_keys", "OFF")?;
        let tx = connection.unchecked_transaction()?;
        for event in &new_events {
            apply_event(&tx, event)?;
        }
        for (file, count, size, mtime_ns, hash) in &watermark_updates {
            Self::write_watermark(&tx, file, *count, *size, *mtime_ns, hash)?;
        }
        tx.commit()?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        self.regenerate_views(&connection)?;
        Ok(connection)
    }

    fn open_existing(&self) -> Result<Connection> {
        if !self.db_path.exists() {
            return Err(HarnessInfraError::MissingDatabase(
                self.db_path.display().to_string(),
            ));
        }

        let connection = Connection::open(&self.db_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }

    fn open_or_create(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }

    fn schema_version(connection: &Connection) -> Result<i64> {
        let version = connection
            .query_row(
                "SELECT COALESCE(MAX(version),0) FROM schema_version;",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(version)
    }

    /// Every applied migration version, ascending. Empty when the
    /// `schema_version` table is absent (uninitialized/legacy cache).
    fn applied_migrations(connection: &Connection) -> Result<Vec<i64>> {
        let table_exists: Option<String> = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version';",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if table_exists.is_none() {
            return Ok(Vec::new());
        }
        let mut statement =
            connection.prepare("SELECT version FROM schema_version ORDER BY version;")?;
        let rows = statement.query_map([], |row| row.get::<_, i64>(0))?;
        collect_rows(rows)
    }

    /// Whether the log holds events the cache has not consumed — the same
    /// comparison `open_fresh` uses to decide on replay, but read-only: it
    /// only reports pending work and never mutates the cache or watermarks.
    fn cache_behind_log(&self, connection: &Connection) -> Result<bool> {
        let mut seen_files: Vec<String> = Vec::new();
        for path in self.list_event_files()? {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            seen_files.push(name.clone());
            let metadata = fs::metadata(&path)?;
            let size = metadata.len() as i64;
            let mtime_ns = file_mtime_ns(&path)?;
            let watermark: Option<(i64, i64)> = connection
                .query_row(
                    "SELECT file_size, file_mtime_ns FROM event_watermark WHERE file=?1;",
                    params![name],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            match watermark {
                Some((wm_size, wm_mtime)) => {
                    if wm_size != size || wm_mtime != mtime_ns {
                        return Ok(true);
                    }
                }
                // A log file with no watermark row is entirely unconsumed.
                None => return Ok(true),
            }
        }
        // A watermarked file that no longer exists means the cache holds
        // events the log has dropped: a read command rebuilds from the log.
        let mut statement = connection.prepare("SELECT file FROM event_watermark;")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        for file in collect_rows(rows)? {
            if !seen_files.contains(&file) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn apply_schema_v1(&self, connection: &Connection) -> Result<()> {
        let schema_path = self.schema_dir.join("001-init.sql");
        if !schema_path.exists() {
            return Err(HarnessInfraError::MissingSchema(
                schema_path.display().to_string(),
            ));
        }

        let schema = fs::read_to_string(schema_path)?;
        connection.execute_batch(&schema)?;
        Ok(())
    }

    fn apply_pending_migrations(
        &self,
        connection: &Connection,
        current_version: i64,
    ) -> Result<Vec<i64>> {
        let mut applied = Vec::new();
        for (version, path) in self.migration_files()? {
            if version > SUPPORTED_SCHEMA_VERSION {
                return Err(HarnessInfraError::UnsupportedSchemaVersion(
                    version,
                    SUPPORTED_SCHEMA_VERSION,
                ));
            }
            if version > current_version {
                let sql = fs::read_to_string(path)?;
                connection.execute_batch(&sql)?;
                applied.push(version);
            }
        }
        Ok(applied)
    }

    fn migration_files(&self) -> Result<Vec<(i64, PathBuf)>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("sql") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let Some(prefix) = file_name.split('-').next() else {
                continue;
            };
            let Ok(version) = prefix.trim_start_matches('0').parse::<i64>() else {
                continue;
            };
            files.push((version, path));
        }
        files.sort_by_key(|(version, _)| *version);
        Ok(files)
    }

    /// Resolve a row id given exactly or as an unambiguous prefix (US-028b:
    /// ULID ids keep `--id <x>` ergonomics via prefixes; legacy numeric ids
    /// match exactly first).
    fn resolve_row_id(connection: &Connection, table: &str, given: &str) -> Result<String> {
        let exact: Option<String> = connection
            .query_row(
                &format!("SELECT id FROM {table} WHERE id=?1;"),
                params![given],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = exact {
            return Ok(id);
        }
        let mut statement = connection.prepare(&format!(
            "SELECT id FROM {table} WHERE id LIKE ?1 || '%' ORDER BY id LIMIT 11;"
        ))?;
        let rows = statement.query_map(params![given], |row| row.get::<_, String>(0))?;
        let matches = collect_rows(rows)?;
        match matches.len() {
            0 => Err(HarnessInfraError::RowIdNotFound(
                table.to_owned(),
                given.to_owned(),
            )),
            1 => Ok(matches.into_iter().next().expect("one match")),
            _ => Err(HarnessInfraError::AmbiguousRowId(
                table.to_owned(),
                given.to_owned(),
                matches.join(", "),
            )),
        }
    }

    fn import_matrix(&self, connection: &Connection) -> Result<usize> {
        let matrix_path = self.repo_root.join("docs/TEST_MATRIX.md");
        if !matrix_path.exists() {
            return Err(HarnessInfraError::MissingBrownfieldPath(
                matrix_path.display().to_string(),
            ));
        }

        let content = fs::read_to_string(matrix_path)?;
        let mut story_count = 0;
        let mut columns: Option<MatrixColumns> = None;

        for line in content.lines() {
            if !line.trim_start().starts_with('|') {
                continue;
            }

            let fields = markdown_table_fields(line);
            if fields.len() < 2 {
                continue;
            }

            if columns.is_none() {
                let candidate = MatrixColumns::from_header(&fields);
                if candidate.story.is_some() && candidate.status.is_some() {
                    columns = Some(candidate);
                }
                continue;
            }

            let columns = columns.as_ref().expect("matrix columns discovered");
            let id = field_at(&fields, columns.story).unwrap_or_default();
            let token = normalize_token(&id);
            if matches!(
                token.as_str(),
                "" | "story" | "tbd" | "todo" | "example" | "examples"
            ) || id.chars().all(|character| character == '-')
            {
                continue;
            }

            let mut title = field_at(&fields, columns.contract).unwrap_or_else(|| id.clone());
            if title.is_empty() {
                title = id.clone();
            }

            let status =
                normalize_story_status(&field_at(&fields, columns.status).unwrap_or_default());
            let unit = proof_from_cell(&field_at(&fields, columns.unit).unwrap_or_default());
            let integration =
                proof_from_cell(&field_at(&fields, columns.integration).unwrap_or_default());
            let e2e = proof_from_cell(&field_at(&fields, columns.e2e).unwrap_or_default());
            let platform =
                proof_from_cell(&field_at(&fields, columns.platform).unwrap_or_default());
            let evidence = columns
                .evidence
                .and_then(|index| evidence_from_fields(&fields, index));

            connection.execute(
                "INSERT INTO story (
                    id, title, risk_lane, contract_doc, status,
                    unit_proof, integration_proof, e2e_proof, platform_proof,
                    evidence, notes
                 ) VALUES (?1, ?2, 'high_risk', ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                    'Imported from docs/TEST_MATRIX.md by harness import brownfield.'
                 )
                 ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    contract_doc=excluded.contract_doc,
                    status=excluded.status,
                    unit_proof=excluded.unit_proof,
                    integration_proof=excluded.integration_proof,
                    e2e_proof=excluded.e2e_proof,
                    platform_proof=excluded.platform_proof,
                    evidence=excluded.evidence,
                    notes=excluded.notes;",
                params![
                    id,
                    title,
                    field_at(&fields, columns.contract),
                    status,
                    unit,
                    integration,
                    e2e,
                    platform,
                    evidence,
                ],
            )?;
            story_count += 1;
        }

        Ok(story_count)
    }

    fn import_decisions(&self, connection: &Connection) -> Result<usize> {
        let decisions_dir = self.repo_root.join("docs/decisions");
        if !decisions_dir.is_dir() {
            return Err(HarnessInfraError::MissingBrownfieldPath(
                decisions_dir.display().to_string(),
            ));
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&decisions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if is_decision_file_name(file_name) {
                files.push(path);
            }
        }
        files.sort();

        let mut decision_count = 0;
        for path in files {
            let content = fs::read_to_string(&path)?;
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned();
            let title = content
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("# "))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(&stem)
                .to_owned();
            let status =
                normalize_decision_status(&markdown_section_first_value(&content, "Status"));
            let doc_path = format!(
                "docs/decisions/{}",
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
            );

            connection.execute(
                "INSERT INTO decision (id, title, status, doc_path, notes)
                 VALUES (?1, ?2, ?3, ?4,
                    'Imported from docs/decisions by harness import brownfield.'
                 )
                 ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    status=excluded.status,
                    doc_path=excluded.doc_path,
                    notes=excluded.notes;",
                params![stem, title, status, doc_path],
            )?;
            decision_count += 1;
        }

        Ok(decision_count)
    }

    fn import_backlog(&self, connection: &Connection) -> Result<usize> {
        let backlog_path = self.repo_root.join("docs/HARNESS_BACKLOG.md");
        if !backlog_path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(backlog_path)?;
        let items = backlog_items(&content);
        let mut imported = 0;
        for item in items {
            if item.title.is_empty() || item.title == "Short name." {
                continue;
            }

            let risk = if item.risk.is_empty() {
                None
            } else {
                RiskLane::from_str(&item.risk)
                    .ok()
                    .map(|value| value.as_db_value().to_owned())
            };
            let status = normalize_backlog_status(&item.status);
            let discovered = empty_to_none(item.discovered_while);
            let pain = empty_to_none(item.current_pain);
            let suggestion = empty_to_none(item.suggested_improvement);

            connection.execute(
                "INSERT INTO backlog (
                    id, title, discovered_while, current_pain, suggested_improvement,
                    risk, status, notes
                 )
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                    'Imported from docs/HARNESS_BACKLOG.md by harness import brownfield.'
                 WHERE NOT EXISTS (
                    SELECT 1 FROM backlog WHERE title=?2
                 );",
                params![
                    mint_ulid(),
                    item.title,
                    discovered,
                    pain,
                    suggestion,
                    risk,
                    status
                ],
            )?;
            imported += 1;
        }

        Ok(imported)
    }
}

impl HarnessRepository for SqliteHarnessRepository {
    fn init(&self) -> Result<InitResult> {
        if self.db_path.exists() {
            let connection = self.open_existing()?;
            let current = Self::schema_version(&connection).unwrap_or(0);
            if current == 0 {
                self.apply_schema_v1(&connection)?;
                self.apply_pending_migrations(&connection, 1)?;
                return Ok(InitResult::MigratedExisting {
                    db_path: self.db_path.clone(),
                });
            }

            return Ok(InitResult::Existing {
                db_path: self.db_path.clone(),
                version: current,
            });
        }

        let connection = self.open_or_create()?;
        self.apply_schema_v1(&connection)?;
        self.apply_pending_migrations(&connection, 1)?;
        // A fresh database is event-backed from genesis: its (empty) log is
        // the source of truth from the first write. Materialize the events
        // dir (with a .gitkeep, since git cannot track an empty dir) so the
        // SETUP.md `git add .harness/events` step works before any write.
        let events_dir = self.events_dir();
        fs::create_dir_all(&events_dir)?;
        let gitkeep = events_dir.join(".gitkeep");
        if !gitkeep.exists() {
            fs::write(&gitkeep, "")?;
        }
        Self::cache_meta_set(&connection, "event_backed", "true")?;
        Ok(InitResult::Created {
            db_path: self.db_path.clone(),
        })
    }

    fn migrate(&self) -> Result<MigrateResult> {
        let connection = self.open_existing()?;
        let current_version = Self::schema_version(&connection).unwrap_or(0);
        let applied = self.apply_pending_migrations(&connection, current_version)?;

        Ok(MigrateResult {
            current_version,
            applied,
        })
    }

    fn info(&self) -> Result<InfoReport> {
        let available_schema_version = self
            .migration_files()?
            .iter()
            .map(|(version, _)| *version)
            .max()
            .unwrap_or(0);
        let event_files = self.list_event_files()?.len();

        let mut report = InfoReport {
            cli_version: env!("CARGO_PKG_VERSION").to_owned(),
            supported_schema_version: SUPPORTED_SCHEMA_VERSION,
            available_schema_version,
            event_format_version: crate::events::EVENT_SCHEMA_VERSION,
            initialized: self.db_path.exists(),
            db_path: self.db_path.display().to_string(),
            applied_schema_version: 0,
            applied_migrations: Vec::new(),
            event_backed: false,
            event_files,
            schema_behind_cli: false,
            // A fresh clone with events but no cache is behind the log until
            // the next command rebuilds the cache.
            cache_behind_log: false,
        };

        if !report.initialized {
            report.cache_behind_log = event_files > 0;
            return Ok(report);
        }

        // Read-only open: never open_fresh here, which would replay the log
        // and destroy the cache-behind-log signal this command exists to
        // report.
        let connection = self.open_existing()?;
        report.applied_migrations = Self::applied_migrations(&connection)?;
        report.applied_schema_version =
            report.applied_migrations.iter().copied().max().unwrap_or(0);
        report.event_backed =
            Self::cache_meta_get(&connection, "event_backed")?.as_deref() == Some("true");
        report.schema_behind_cli = report.applied_schema_version < available_schema_version;
        report.cache_behind_log = report.event_backed && self.cache_behind_log(&connection)?;
        Ok(report)
    }

    fn import_brownfield(&self) -> Result<BrownfieldImportResult> {
        let connection = self.open_fresh()?;
        // Post-cutover, brownfield import is migration-only (decision 0008
        // Q4). On a pristine event-backed cache (fresh init, empty log) it
        // still works as the one-time seed: import via the legacy path, then
        // re-run the genesis migration so every imported row becomes a
        // proven event. Anything else must not bypass the log.
        let event_backed =
            Self::cache_meta_get(&connection, "event_backed")?.as_deref() == Some("true");
        if event_backed {
            let durable_rows: i64 = connection.query_row(
                "SELECT (SELECT COUNT(*) FROM intake) + (SELECT COUNT(*) FROM story)
                      + (SELECT COUNT(*) FROM decision) + (SELECT COUNT(*) FROM backlog)
                      + (SELECT COUNT(*) FROM trace) + (SELECT COUNT(*) FROM tool)
                      + (SELECT COUNT(*) FROM intervention) + (SELECT COUNT(*) FROM story_signal);",
                [],
                |row| row.get(0),
            )?;
            if durable_rows > 0 || !self.list_event_files()?.is_empty() {
                return Err(HarnessInfraError::BrownfieldOnEventBacked);
            }
            Self::cache_meta_set(&connection, "event_backed", "false")?;
        }

        let stories = self.import_matrix(&connection)?;
        let decisions = self.import_decisions(&connection)?;
        let backlog_items = self.import_backlog(&connection)?;
        drop(connection);

        if event_backed {
            self.migrate_to_events()?;
        }

        Ok(BrownfieldImportResult {
            stories,
            decisions,
            backlog_items,
        })
    }

    fn record_intake(&self, input: IntakeInput) -> Result<String> {
        let connection = self.open_fresh()?;
        let id = mint_ulid();
        let payload = json!({
            "id": id,
            "input_type": input.input_type.as_db_value(),
            "summary": input.summary,
            "risk_lane": input.risk_lane.as_db_value(),
            "risk_flags": csv_payload(&input.risk_flags),
            "affected_docs": csv_payload(&input.affected_docs),
            "story_id": input.story_id,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "intake.record", payload)?;
        Ok(id)
    }

    fn add_story(&self, input: StoryAddInput) -> Result<()> {
        let connection = self.open_fresh()?;
        let payload = json!({
            "id": input.id,
            "title": input.title,
            "risk_lane": input.risk_lane.as_db_value(),
            "contract_doc": input.contract_doc,
            "verify_command": input.verify_command,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "story.add", payload)
    }

    fn update_story(&self, input: StoryUpdateInput) -> Result<()> {
        if input.status.is_none()
            && input.evidence.is_none()
            && input.unit.is_none()
            && input.integration.is_none()
            && input.e2e.is_none()
            && input.platform.is_none()
            && input.verify_command.is_none()
        {
            return Err(HarnessInfraError::EmptyStoryUpdate);
        }

        let connection = self.open_fresh()?;
        let exists: Option<String> = connection
            .query_row(
                "SELECT id FROM story WHERE id=?1;",
                params![input.id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(HarnessInfraError::StoryNotFound(input.id));
        }
        let mut payload = serde_json::Map::new();
        payload.insert("id".to_owned(), json!(input.id));
        if let Some(status) = &input.status {
            payload.insert("status".to_owned(), json!(status));
        }
        if let Some(evidence) = &input.evidence {
            payload.insert("evidence".to_owned(), json!(evidence));
        }
        if let Some(flag) = &input.unit {
            payload.insert("unit_proof".to_owned(), json!(flag.0));
        }
        if let Some(flag) = &input.integration {
            payload.insert("integration_proof".to_owned(), json!(flag.0));
        }
        if let Some(flag) = &input.e2e {
            payload.insert("e2e_proof".to_owned(), json!(flag.0));
        }
        if let Some(flag) = &input.platform {
            payload.insert("platform_proof".to_owned(), json!(flag.0));
        }
        if let Some(verify_command) = &input.verify_command {
            payload.insert("verify_command".to_owned(), json!(verify_command));
        }
        self.append_and_apply(&connection, "story.update", JsonValue::Object(payload))
    }

    fn verify_story(&self, id: &str) -> Result<StoryVerifyResult> {
        let connection = self.open_fresh()?;
        let verify_command = connection
            .query_row(
                "SELECT verify_command FROM story WHERE id=?1;",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HarnessInfraError::MissingStoryVerifyCommand(id.to_owned()))?;

        let unreviewed = unreviewed_diagrams(&self.repo_root, id);
        if !unreviewed.is_empty() {
            self.append_and_apply(
                &connection,
                "story.verify_result",
                json!({"id": id, "result": "fail"}),
            )?;
            return Ok(StoryVerifyResult {
                command: verify_command,
                stdout: String::new(),
                stderr: diagram_gate_message(id, &unreviewed),
                result: "fail".to_owned(),
            });
        }

        let (shell, flag) = verifier_shell();
        let output = Command::new(shell)
            .arg(flag)
            .arg(&verify_command)
            .current_dir(&self.repo_root)
            .output()?;
        let result = if output.status.success() {
            "pass"
        } else {
            "fail"
        }
        .to_owned();
        self.append_and_apply(
            &connection,
            "story.verify_result",
            json!({"id": id, "result": result}),
        )?;

        Ok(StoryVerifyResult {
            command: verify_command,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            result,
        })
    }

    fn verify_all_stories(&self) -> Result<StoryVerifyAllResult> {
        let connection = self.open_fresh()?;
        let mut statement =
            connection.prepare("SELECT id, title, verify_command FROM story ORDER BY id;")?;
        let story_rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let stories = collect_rows(story_rows)?;
        let mut items = Vec::new();

        for (id, title, verify_command) in stories {
            let Some(command) = verify_command.filter(|value| !value.trim().is_empty()) else {
                items.push(StoryVerifyAllItem {
                    id,
                    title,
                    command: None,
                    result: "skipped".to_owned(),
                    stdout: String::new(),
                    stderr: String::new(),
                });
                continue;
            };

            let unreviewed = unreviewed_diagrams(&self.repo_root, &id);
            if !unreviewed.is_empty() {
                self.append_and_apply(
                    &connection,
                    "story.verify_result",
                    json!({"id": id, "result": "fail"}),
                )?;
                let stderr = diagram_gate_message(&id, &unreviewed);
                items.push(StoryVerifyAllItem {
                    id,
                    title,
                    command: Some(command),
                    result: "fail".to_owned(),
                    stdout: String::new(),
                    stderr,
                });
                continue;
            }

            let (shell, flag) = verifier_shell();
            let output = Command::new(shell)
                .arg(flag)
                .arg(&command)
                .current_dir(&self.repo_root)
                .output()?;
            let result = if output.status.success() {
                "pass"
            } else {
                "fail"
            }
            .to_owned();
            self.append_and_apply(
                &connection,
                "story.verify_result",
                json!({"id": id, "result": result}),
            )?;
            items.push(StoryVerifyAllItem {
                id,
                title,
                command: Some(command),
                result,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(StoryVerifyAllResult { items })
    }

    fn add_decision(&self, input: DecisionAddInput) -> Result<()> {
        let connection = self.open_fresh()?;
        let payload = json!({
            "id": input.id,
            "title": input.title,
            "status": input.status,
            "doc_path": input.doc_path,
            "verify_command": input.verify_command,
            "predicted_impact": input.predicted_impact,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "decision.add", payload)
    }

    fn update_decision(&self, input: DecisionUpdateInput) -> Result<()> {
        if input.title.is_none()
            && input.status.is_none()
            && input.doc_path.is_none()
            && input.verify_command.is_none()
            && input.predicted_impact.is_none()
            && input.notes.is_none()
        {
            return Err(HarnessInfraError::EmptyDecisionUpdate);
        }

        let connection = self.open_fresh()?;
        let exists: Option<String> = connection
            .query_row(
                "SELECT id FROM decision WHERE id=?1;",
                params![input.id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(HarnessInfraError::DecisionNotFound(input.id));
        }
        let mut payload = serde_json::Map::new();
        payload.insert("id".to_owned(), json!(input.id));
        if let Some(title) = &input.title {
            payload.insert("title".to_owned(), json!(title));
        }
        if let Some(status) = &input.status {
            payload.insert("status".to_owned(), json!(status));
        }
        if let Some(doc_path) = &input.doc_path {
            payload.insert("doc_path".to_owned(), json!(doc_path));
        }
        if let Some(verify_command) = &input.verify_command {
            payload.insert("verify_command".to_owned(), json!(verify_command));
        }
        if let Some(predicted_impact) = &input.predicted_impact {
            payload.insert("predicted_impact".to_owned(), json!(predicted_impact));
        }
        if let Some(notes) = &input.notes {
            payload.insert("notes".to_owned(), json!(notes));
        }
        self.append_and_apply(&connection, "decision.update", JsonValue::Object(payload))
    }

    fn verify_decision(&self, id: &str) -> Result<DecisionVerifyResult> {
        let connection = self.open_fresh()?;
        let verify_command = connection
            .query_row(
                "SELECT verify_command FROM decision WHERE id=?1;",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HarnessInfraError::MissingDecisionVerifyCommand(id.to_owned()))?;

        let (shell, flag) = verifier_shell();
        let status = Command::new(shell)
            .arg(flag)
            .arg(&verify_command)
            .current_dir(&self.repo_root)
            .status()?;
        let result = if status.success() { "pass" } else { "fail" }.to_owned();
        self.append_and_apply(
            &connection,
            "decision.verify_result",
            json!({"id": id, "result": result}),
        )?;

        Ok(DecisionVerifyResult {
            command: verify_command,
            result,
        })
    }

    fn add_backlog(&self, input: BacklogAddInput) -> Result<String> {
        let connection = self.open_fresh()?;
        let id = mint_ulid();
        let payload = json!({
            "id": id,
            "title": input.title,
            "discovered_while": input.discovered_while,
            "current_pain": input.current_pain,
            "suggested_improvement": input.suggestion,
            "risk": input.risk.map(|value| value.as_db_value().to_owned()),
            "predicted_impact": input.predicted_impact,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "backlog.add", payload)?;
        Ok(id)
    }

    fn close_backlog(&self, input: BacklogCloseInput) -> Result<()> {
        let connection = self.open_fresh()?;
        let id = Self::resolve_row_id(&connection, "backlog", &input.id)?;
        let payload = json!({
            "id": id,
            "status": input.status,
            "actual_outcome": input.actual_outcome,
        });
        self.append_and_apply(&connection, "backlog.close", payload)
    }

    fn register_tool(&self, input: ToolRegisterInput) -> Result<()> {
        validate_tool_description(&input.description)?;
        // Only exec-probed kinds are PATH-checked at register time. mcp/skill/http
        // are not on PATH by nature, so registering intent always succeeds; their
        // presence is resolved later by `tool check` via scan_target.
        let exec_probed = matches!(input.kind.as_str(), "cli" | "binary");
        if exec_probed && !input.force && !command_available(&self.repo_root, &input.command) {
            return Err(HarnessInfraError::ToolCommandNotFound(input.command));
        }

        let connection = self.open_fresh()?;
        let existing = connection
            .query_row(
                "SELECT command FROM tool WHERE name=?1;",
                params![input.name],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(command) = existing {
            return Err(HarnessInfraError::ToolAlreadyExists(input.name, command));
        }

        // Machine-local scan state (status, checked_at) is never logged.
        let payload = json!({
            "name": input.name,
            "command": input.command,
            "description": input.description,
            "args": tool_args_json(&input.args)
                .and_then(|text| serde_json::from_str::<JsonValue>(&text).ok()),
            "responsibility": input.responsibility,
            "kind": input.kind,
            "capability": input.capability,
            "scan_target": input.scan_target,
        });
        self.append_and_apply(&connection, "tool.register", payload)
    }

    fn remove_tool(&self, name: &str) -> Result<()> {
        let connection = self.open_fresh()?;
        let exists: Option<String> = connection
            .query_row(
                "SELECT name FROM tool WHERE name=?1;",
                params![name],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(HarnessInfraError::ToolNotFound(name.to_owned()));
        }
        self.append_and_apply(&connection, "tool.remove", json!({"name": name}))
    }

    fn check_tools(&self, name: Option<String>) -> Result<Vec<ToolCheckResult>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT name, kind, command, scan_target, capability FROM tool
             WHERE (?1 IS NULL OR name = ?1)
             ORDER BY name;",
        )?;
        let rows = statement.query_map(params![name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        let tools = collect_rows(rows)?;

        let mut results = Vec::with_capacity(tools.len());
        for (name, kind, command, scan_target, capability) in tools {
            let (status, detail) =
                scan_tool_status(&self.repo_root, &kind, &command, scan_target.as_deref());
            connection.execute(
                "UPDATE tool SET status=?1, checked_at=datetime('now') WHERE name=?2;",
                params![status, name],
            )?;
            results.push(ToolCheckResult {
                name,
                kind,
                capability,
                status: status.to_owned(),
                detail,
            });
        }
        Ok(results)
    }

    fn add_intervention(&self, input: InterventionAddInput) -> Result<String> {
        let connection = self.open_fresh()?;
        let trace_id = input
            .trace_id
            .as_deref()
            .map(|given| Self::resolve_row_id(&connection, "trace", given))
            .transpose()?;
        let id = mint_ulid();
        let payload = json!({
            "id": id,
            "trace_id": trace_id,
            "story_id": input.story_id,
            "type": input.intervention_type,
            "description": input.description,
            "source": input.source,
            "impact": input.impact,
        });
        self.append_and_apply(&connection, "intervention.add", payload)?;
        Ok(id)
    }

    fn add_story_signal(&self, input: StorySignalAddInput) -> Result<String> {
        let connection = self.open_fresh()?;
        let trace_id = input
            .trace_id
            .as_deref()
            .map(|given| Self::resolve_row_id(&connection, "trace", given))
            .transpose()?;
        let id = mint_ulid();
        let payload = json!({
            "id": id,
            "story_id": input.story_id,
            "trace_id": trace_id,
            "type": input.signal_type,
            "summary": input.summary,
            "component": input.component,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "signal.add", payload)?;
        Ok(id)
    }

    fn record_trace(&self, input: TraceInput) -> Result<String> {
        let connection = self.open_fresh()?;
        let intake_id = input
            .intake_id
            .as_deref()
            .map(|given| Self::resolve_row_id(&connection, "intake", given))
            .transpose()?;
        let id = mint_ulid();
        let payload = json!({
            "id": id,
            "task_summary": input.task_summary,
            "intake_id": intake_id,
            "story_id": input.story_id,
            "agent": input.agent,
            "actions_taken": csv_payload(&input.actions),
            "files_read": csv_payload(&input.files_read),
            "files_changed": csv_payload(&input.files_changed),
            "decisions_made": csv_payload(&input.decisions),
            "errors": csv_payload(&input.errors),
            "outcome": input.outcome,
            "duration_seconds": input.duration_seconds,
            "token_estimate": input.token_estimate,
            "harness_friction": input.friction,
            "notes": input.notes,
        });
        self.append_and_apply(&connection, "trace.record", payload)?;
        Ok(id)
    }

    fn score_trace(&self, id: Option<String>) -> Result<TraceScoreResult> {
        let connection = self.open_fresh()?;
        let id = id
            .as_deref()
            .map(|given| Self::resolve_row_id(&connection, "trace", given))
            .transpose()?;
        let sql = match id {
            Some(_) => {
                "SELECT
                    trace.id,
                    trace.task_summary,
                    trace.intake_id,
                    intake.risk_lane,
                    trace.agent,
                    trace.actions_taken,
                    trace.files_read,
                    trace.files_changed,
                    trace.decisions_made,
                    trace.errors,
                    trace.outcome,
                    trace.duration_seconds,
                    trace.token_estimate,
                    trace.harness_friction,
                    trace.notes
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 WHERE trace.id = ?1"
            }
            None => {
                "SELECT
                    trace.id,
                    trace.task_summary,
                    trace.intake_id,
                    intake.risk_lane,
                    trace.agent,
                    trace.actions_taken,
                    trace.files_read,
                    trace.files_changed,
                    trace.decisions_made,
                    trace.errors,
                    trace.outcome,
                    trace.duration_seconds,
                    trace.token_estimate,
                    trace.harness_friction,
                    trace.notes
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 ORDER BY trace.created_at DESC, trace.id DESC
                 LIMIT 1"
            }
        };

        let source = if let Some(id) = id {
            connection
                .query_row(sql, params![id], trace_score_source_from_row)
                .optional()?
                .ok_or_else(|| HarnessInfraError::RowIdNotFound("trace".to_owned(), id))?
        } else {
            connection
                .query_row(sql, [], trace_score_source_from_row)
                .optional()?
                .ok_or(HarnessInfraError::NoTraces)?
        };

        Ok(score_trace(source))
    }

    fn score_context(&self, id: &str) -> Result<ContextScoreResult> {
        let connection = self.open_fresh()?;
        let id = Self::resolve_row_id(&connection, "trace", id)?;
        let source = connection
            .query_row(
                "SELECT
                    trace.id,
                    intake.risk_lane,
                    trace.story_id,
                    trace.files_read,
                    trace.files_changed,
                    trace.outcome
                 FROM trace
                 LEFT JOIN intake ON intake.id = trace.intake_id
                 WHERE trace.id=?1;",
                params![id],
                |row| {
                    Ok(ContextScoreSource {
                        id: row.get(0)?,
                        risk_lane: row.get(1)?,
                        story_id: row.get(2)?,
                        files_read: row.get(3)?,
                        files_changed: row.get(4)?,
                        outcome: row.get(5)?,
                    })
                },
            )
            .optional()?
            .ok_or(HarnessInfraError::RowIdNotFound("trace".to_owned(), id))?;

        Ok(score_context(source))
    }

    fn story_verify_status(&self, id: &str) -> Result<StoryVerifyStatus> {
        let connection = self.open_fresh()?;
        connection
            .query_row(
                "SELECT id, verify_command, last_verified_result FROM story WHERE id=?1;",
                params![id],
                |row| {
                    Ok(StoryVerifyStatus {
                        id: row.get(0)?,
                        verify_command: row.get(1)?,
                        last_verified_result: row.get(2)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HarnessInfraError::StoryNotFound(id.to_owned()))
    }

    fn query_matrix(&self) -> Result<Vec<StoryMatrixRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, title, status, unit_proof, integration_proof, e2e_proof, platform_proof, evidence
             FROM story ORDER BY id;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(StoryMatrixRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                unit: row.get(3)?,
                integration: row.get(4)?,
                e2e: row.get(5)?,
                platform: row.get(6)?,
                evidence: row.get(7)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_backlog(&self, filter: BacklogFilter) -> Result<Vec<BacklogRecord>> {
        let connection = self.open_fresh()?;
        let where_clause = match filter {
            BacklogFilter::All => "",
            BacklogFilter::Open => "WHERE status IN ('proposed', 'accepted')",
            BacklogFilter::Closed => "WHERE status IN ('implemented', 'rejected')",
        };
        let sql = format!(
            "SELECT id, title, status, risk, predicted_impact, actual_outcome
             FROM backlog {where_clause} ORDER BY status, id;"
        );
        let mut statement = connection.prepare(&sql)?;

        let rows = statement.query_map([], |row| {
            Ok(BacklogRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                risk: row.get(3)?,
                predicted_impact: row.get(4)?,
                actual_outcome: row.get(5)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_decisions(&self) -> Result<Vec<DecisionRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, title, status, last_verified_at, last_verified_result
             FROM decision ORDER BY id;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(DecisionRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                last_verified_at: row.get(3)?,
                last_verified_result: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_intakes(&self) -> Result<Vec<IntakeRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, input_type, risk_lane, summary
             FROM intake ORDER BY created_at DESC, id DESC LIMIT 20;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(IntakeRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                input_type: row.get(2)?,
                risk_lane: row.get(3)?,
                summary: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_traces(&self) -> Result<Vec<TraceRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, outcome, task_summary, harness_friction
             FROM trace ORDER BY created_at DESC, id DESC LIMIT 20;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(TraceRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                outcome: row.get(2)?,
                task_summary: row.get(3)?,
                harness_friction: row.get(4)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_friction(&self) -> Result<Vec<FrictionRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT
                trace.id,
                trace.created_at,
                intake.risk_lane,
                intake.input_type,
                trace.task_summary,
                trace.harness_friction
             FROM trace
             LEFT JOIN intake ON intake.id = trace.intake_id
             WHERE trace.harness_friction IS NOT NULL
             ORDER BY trace.created_at DESC, trace.id DESC;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(FrictionRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                risk_lane: row.get(2)?,
                input_type: row.get(3)?,
                task_summary: row.get(4)?,
                harness_friction: row.get(5)?,
            })
        })?;

        collect_rows(rows)
    }

    fn query_tools(
        &self,
        responsibility: Option<String>,
        capability: Option<String>,
    ) -> Result<Vec<ToolEntry>> {
        let connection = self.open_fresh()?;
        let mut tools = compiled_tool_registry();
        let mut statement = connection.prepare(
            "SELECT provider, name, command, description, args, responsibility, since,
                    kind, capability, scan_target, status, checked_at
             FROM tool ORDER BY name;",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ToolEntry {
                provider: row.get(0)?,
                name: row.get(1)?,
                command: row.get(2)?,
                description: row.get(3)?,
                args: parse_stored_tool_args(row.get::<_, Option<String>>(4)?.as_deref()),
                responsibility: row.get(5)?,
                source: "registered".to_owned(),
                since: row.get(6)?,
                kind: row.get(7)?,
                capability: row.get(8)?,
                scan_target: row.get(9)?,
                status: row.get(10)?,
                checked_at: row.get(11)?,
            })
        })?;
        tools.extend(collect_rows(rows)?);
        if let Some(responsibility) = responsibility {
            let normalized = normalize_token(&responsibility);
            tools.retain(|tool| normalize_token(&tool.responsibility) == normalized);
        }
        if let Some(capability) = capability {
            let normalized = normalize_token(&capability);
            tools.retain(|tool| {
                tool.capability
                    .as_deref()
                    .is_some_and(|value| normalize_token(value) == normalized)
            });
        }
        Ok(tools)
    }

    fn query_interventions(&self, filter: InterventionFilter) -> Result<Vec<InterventionRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, trace_id, story_id, type, description, source, impact
             FROM intervention
             WHERE (?1 IS NULL OR trace_id = ?1)
               AND (?2 IS NULL OR story_id = ?2)
               AND (?3 IS NULL OR type = ?3)
             ORDER BY created_at DESC, id DESC;",
        )?;
        let rows = statement.query_map(
            params![filter.trace_id, filter.story_id, filter.intervention_type],
            |row| {
                Ok(InterventionRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    trace_id: row.get(2)?,
                    story_id: row.get(3)?,
                    intervention_type: row.get(4)?,
                    description: row.get(5)?,
                    source: row.get(6)?,
                    impact: row.get(7)?,
                })
            },
        )?;
        collect_rows(rows)
    }

    fn query_story_signals(&self, filter: StorySignalFilter) -> Result<Vec<StorySignalRecord>> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(
            "SELECT id, created_at, story_id, trace_id, type, summary, component, notes
             FROM story_signal
             WHERE (?1 IS NULL OR story_id = ?1)
               AND (?2 IS NULL OR type = ?2)
             ORDER BY created_at DESC, id DESC;",
        )?;
        let rows = statement.query_map(params![filter.story_id, filter.signal_type], |row| {
            Ok(StorySignalRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                story_id: row.get(2)?,
                trace_id: row.get(3)?,
                signal_type: row.get(4)?,
                summary: row.get(5)?,
                component: row.get(6)?,
                notes: row.get(7)?,
            })
        })?;
        collect_rows(rows)
    }

    fn query_stats(&self) -> Result<HarnessStats> {
        let connection = self.open_fresh()?;
        connection
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM intake) AS intakes,
                    (SELECT COUNT(*) FROM story) AS stories,
                    (SELECT COUNT(*) FROM decision) AS decisions,
                    (SELECT COUNT(*) FROM backlog) AS backlog_items,
                    (SELECT COUNT(*) FROM trace) AS traces;",
                [],
                |row| {
                    Ok(HarnessStats {
                        intakes: row.get(0)?,
                        stories: row.get(1)?,
                        decisions: row.get(2)?,
                        backlog_items: row.get(3)?,
                        traces: row.get(4)?,
                    })
                },
            )
            .map_err(HarnessInfraError::from)
    }

    fn audit(&self) -> Result<AuditResult> {
        let connection = self.open_fresh()?;
        let mut result = AuditResult {
            orphaned_stories: audit_findings(
                &connection,
                "SELECT story.id, story.title
                 FROM story
                 LEFT JOIN trace ON trace.story_id = story.id
                 WHERE story.status IN ('planned','in_progress') AND trace.id IS NULL
                 ORDER BY story.id;",
            )?,
            unverified_stories: audit_findings(
                &connection,
                "SELECT id, title FROM story
                 WHERE verify_command IS NOT NULL
                   AND TRIM(verify_command) <> ''
                   AND last_verified_result IS NULL
                 ORDER BY id;",
            )?,
            unverified_decisions: audit_findings(
                &connection,
                "SELECT id, title FROM decision
                 WHERE verify_command IS NOT NULL
                   AND TRIM(verify_command) <> ''
                   AND last_verified_result IS NULL
                 ORDER BY id;",
            )?,
            backlog_without_outcomes: audit_findings(
                &connection,
                "SELECT CAST(id AS TEXT), title FROM backlog
                 WHERE predicted_impact IS NOT NULL
                   AND actual_outcome IS NULL
                   AND status='implemented'
                 ORDER BY id;",
            )?,
            stale_stories: audit_findings(
                &connection,
                "SELECT story.id, story.title
                 FROM story
                 JOIN trace ON trace.story_id = story.id
                 WHERE story.status <> 'implemented'
                 GROUP BY story.id, story.title
                 HAVING julianday('now') - julianday(MAX(trace.created_at)) > 30
                 ORDER BY story.id;",
            )?,
            broken_tools: Vec::new(),
            concurrent_lww_updates: audit_findings(
                &connection,
                "SELECT story_id || '.' || field,
                        'concurrent update: ' || loser_event_id || ' (' || loser_writer ||
                        ') lost to ' || winner_event_id || ' (' || winner_writer || ')'
                 FROM lww_audit ORDER BY loser_event_id, field;",
            )?,
        };

        let mut statement =
            connection.prepare("SELECT name, command, kind, status FROM tool ORDER BY name;")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        for (name, command, kind, status) in collect_rows(rows)? {
            // Exec-probed kinds are checked live against PATH. Scanned kinds
            // (mcp/skill/http) are only "broken" once a scan has positively
            // found them missing; an un-scanned `unknown` is not drift.
            let broken = match kind.as_str() {
                "cli" | "binary" => !command_available(&self.repo_root, &command),
                _ => status == "missing",
            };
            if broken {
                result.broken_tools.push(AuditFinding {
                    id: name,
                    title: command,
                });
            }
        }
        Ok(result)
    }

    fn propose(&self, commit: bool) -> Result<Vec<ImprovementProposal>> {
        let connection = self.open_fresh()?;
        let audit = self.audit()?;
        let mut proposals = Vec::new();

        for (text, count) in repeated_friction(&connection)? {
            proposals.push(ImprovementProposal {
                title: format!("Reduce repeated friction: {}", short_title(&text)),
                component: "Failure attribution".to_owned(),
                evidence: format!("{count} traces recorded similar friction: {text}"),
                predicted_impact: "Fewer repeated harness friction entries for similar tasks.".to_owned(),
                risk: "normal".to_owned(),
                suggested_action: "Update the relevant Harness docs, templates, or CLI guidance for this friction pattern.".to_owned(),
                validation_plan: "Review the next five related traces and compare friction frequency.".to_owned(),
                confidence: confidence_for_count(count),
                committed_backlog_id: None,
            });
        }

        for (key, count) in repeated_interventions(&connection)? {
            proposals.push(ImprovementProposal {
                title: format!("Address repeated intervention: {}", short_title(&key)),
                component: "Intervention recording".to_owned(),
                evidence: format!("{count} interventions share the pattern: {key}"),
                predicted_impact: "Fewer repeated human or review interventions for the same issue.".to_owned(),
                risk: "normal".to_owned(),
                suggested_action: "Clarify the relevant operating rule or validation gate that would have caught this earlier.".to_owned(),
                validation_plan: "Future interventions of this type should decrease after the rule change.".to_owned(),
                confidence: confidence_for_count(count),
                committed_backlog_id: None,
            });
        }

        for (signal_type, summary, count) in repeated_story_signals(&connection)? {
            proposals.push(ImprovementProposal {
                title: format!(
                    "Recurring {} signal: {}",
                    signal_type,
                    short_title(&summary)
                ),
                component: "Task specification".to_owned(),
                evidence: format!(
                    "{count} stories recorded a similar {signal_type} signal: {summary}"
                ),
                predicted_impact: "Closing the spec, plan, or template gap behind this recurring signal.".to_owned(),
                risk: "normal".to_owned(),
                suggested_action: "Update the plan/intake template, story template, or docs so this design point is settled up front.".to_owned(),
                validation_plan: "The signal should stop recurring in stories opened after the fix.".to_owned(),
                confidence: confidence_for_count(count),
                committed_backlog_id: None,
            });
        }

        for (category, count) in [
            (
                "orphaned planned or in-progress stories",
                audit.orphaned_stories.len(),
            ),
            ("unverified story commands", audit.unverified_stories.len()),
            (
                "unverified decision commands",
                audit.unverified_decisions.len(),
            ),
            (
                "implemented backlog items without outcomes",
                audit.backlog_without_outcomes.len(),
            ),
            ("stale unfinished stories", audit.stale_stories.len()),
            ("broken registered tools", audit.broken_tools.len()),
        ] {
            if count > 0 {
                proposals.push(ImprovementProposal {
                    title: format!("Clean up {category}"),
                    component: "Entropy auditing".to_owned(),
                    evidence: format!("Audit found {count} {category}."),
                    predicted_impact: "Lower entropy score and stronger completion evidence.".to_owned(),
                    risk: "tiny".to_owned(),
                    suggested_action: "Resolve the listed audit findings or record why they are intentionally retained.".to_owned(),
                    validation_plan: "Run harness-cli audit and confirm the category count decreases.".to_owned(),
                    confidence: "low".to_owned(),
                    committed_backlog_id: None,
                });
            }
        }

        if commit {
            for proposal in &mut proposals {
                let id = mint_ulid();
                let payload = json!({
                    "id": id,
                    "title": proposal.title,
                    "discovered_while": "harness-cli propose",
                    "current_pain": proposal.evidence,
                    "suggested_improvement": proposal.suggested_action,
                    "risk": normalize_token(&proposal.risk),
                    "predicted_impact": proposal.predicted_impact,
                    "notes": format!(
                        "component: {}; confidence: {}; validation: {}",
                        proposal.component, proposal.confidence, proposal.validation_plan
                    ),
                });
                self.append_and_apply(&connection, "backlog.add", payload)?;
                proposal.committed_backlog_id = Some(id);
            }
        }

        Ok(proposals)
    }

    fn query_sql(&self, sql: &str) -> Result<QueryTable> {
        let connection = self.open_fresh()?;
        let mut statement = connection.prepare(sql)?;
        let headers = statement
            .column_names()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let column_count = statement.column_count();
        let rows = statement.query_map([], |row| {
            let mut values = Vec::new();
            for index in 0..column_count {
                values.push(sql_value_to_string(row.get_ref(index)?));
            }
            Ok(values)
        })?;

        Ok(QueryTable {
            headers,
            rows: collect_rows(rows)?,
        })
    }
}

impl From<HarnessContext> for SqliteHarnessRepository {
    fn from(context: HarnessContext) -> Self {
        Self::new(context.repo_root, context.db_path, context.schema_dir)
    }
}

#[derive(Debug)]
struct MatrixColumns {
    story: Option<usize>,
    contract: Option<usize>,
    unit: Option<usize>,
    integration: Option<usize>,
    e2e: Option<usize>,
    platform: Option<usize>,
    status: Option<usize>,
    evidence: Option<usize>,
}

#[derive(Debug, Default)]
struct BacklogMarkdownItem {
    title: String,
    discovered_while: String,
    current_pain: String,
    suggested_improvement: String,
    risk: String,
    status: String,
}

impl MatrixColumns {
    fn from_header(fields: &[String]) -> Self {
        let mut columns = Self {
            story: None,
            contract: None,
            unit: None,
            integration: None,
            e2e: None,
            platform: None,
            status: None,
            evidence: None,
        };

        for (index, field) in fields.iter().enumerate() {
            match normalize_token(field).as_str() {
                "story" => columns.story = Some(index),
                "contract" => columns.contract = Some(index),
                "unit" => columns.unit = Some(index),
                "integration" => columns.integration = Some(index),
                "e2e" => columns.e2e = Some(index),
                "platform" => columns.platform = Some(index),
                "status" => columns.status = Some(index),
                "evidence" => columns.evidence = Some(index),
                _ => {}
            }
        }

        columns
    }
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(HarnessInfraError::from)
}

fn trace_score_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TraceScoreSource> {
    Ok(TraceScoreSource {
        id: row.get(0)?,
        task_summary: row.get(1)?,
        intake_id: row.get(2)?,
        risk_lane: row.get(3)?,
        agent: row.get(4)?,
        actions_taken: row.get(5)?,
        files_read: row.get(6)?,
        files_changed: row.get(7)?,
        decisions_made: row.get(8)?,
        errors: row.get(9)?,
        outcome: row.get(10)?,
        duration_seconds: row.get(11)?,
        token_estimate: row.get(12)?,
        harness_friction: row.get(13)?,
        notes: row.get(14)?,
    })
}

fn markdown_table_fields(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
    trimmed
        .split('|')
        .map(|field| field.trim().to_owned())
        .collect()
}

fn field_at(fields: &[String], index: Option<usize>) -> Option<String> {
    index
        .and_then(|value| fields.get(value))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn evidence_from_fields(fields: &[String], start_index: usize) -> Option<String> {
    fields
        .get(start_index..)
        .map(|values| values.join(" | "))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn proof_from_cell(value: &str) -> i64 {
    match normalize_token(value).as_str() {
        ""
        | "no"
        | "none"
        | "n_a"
        | "na"
        | "planned"
        | "pending"
        | "blocked"
        | "not_attempted"
        | "not_operator_reviewed" => 0,
        token
            if token.starts_with("no_")
                || token.starts_with("pending")
                || token.starts_with("blocked")
                || token.contains("pending")
                || token.contains("blocked")
                || token.contains("not_attempted")
                || token.contains("not_operator_reviewed") =>
        {
            0
        }
        _ => 1,
    }
}

fn normalize_story_status(value: &str) -> String {
    match normalize_token(value).as_str() {
        "planned" => "planned",
        "in_progress" => "in_progress",
        "implemented" => "implemented",
        "changed" => "changed",
        "retired" => "retired",
        _ => "planned",
    }
    .to_owned()
}

fn normalize_decision_status(value: &str) -> String {
    let token = normalize_token(value);
    match token.as_str() {
        "proposed" => "proposed",
        "accepted" => "accepted",
        "superseded" => "superseded",
        "rejected" => "rejected",
        token if token.starts_with("superseded_") => "superseded",
        _ => "accepted",
    }
    .to_owned()
}

fn normalize_backlog_status(value: &str) -> String {
    match normalize_token(value).as_str() {
        "proposed" => "proposed",
        "accepted" => "accepted",
        "implemented" => "implemented",
        "rejected" => "rejected",
        _ => "proposed",
    }
    .to_owned()
}

fn markdown_section_first_value(content: &str, heading: &str) -> String {
    let target = format!("## {heading}");
    let mut found = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if found && !trimmed.is_empty() {
            return trimmed.to_owned();
        }
        if trimmed == target {
            found = true;
        }
    }
    String::new()
}

fn backlog_items(content: &str) -> Vec<BacklogMarkdownItem> {
    let mut in_items = false;
    let mut current_heading = String::new();
    let mut current = BacklogMarkdownItem::default();
    let mut items = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Items" {
            in_items = true;
            current_heading.clear();
            continue;
        }
        if !in_items {
            continue;
        }

        if let Some(heading) = trimmed.strip_prefix("### ") {
            let normalized = normalize_token(heading);
            if normalized == "title" && !current.title.is_empty() {
                items.push(current);
                current = BacklogMarkdownItem::default();
            }
            current_heading = normalized;
            continue;
        }

        if trimmed.is_empty() || current_heading.is_empty() {
            continue;
        }

        let target = match current_heading.as_str() {
            "title" => &mut current.title,
            "discovered_while" => &mut current.discovered_while,
            "current_pain" => &mut current.current_pain,
            "suggested_improvement" => &mut current.suggested_improvement,
            "risk" => &mut current.risk,
            "status" => &mut current.status,
            _ => continue,
        };
        if target.is_empty() {
            *target = trimmed.to_owned();
        }
    }

    if !current.title.is_empty() {
        items.push(current);
    }
    items
}

fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn command_available(repo_root: &Path, command: &str) -> bool {
    let first = command.split_whitespace().next().unwrap_or(command);
    if first.is_empty() {
        return false;
    }
    let candidate = Path::new(first);
    if candidate.is_absolute() {
        return candidate.exists();
    }
    if first.contains('/') || first.contains('\\') {
        return repo_root.join(first).exists();
    }
    env::var_os("PATH")
        .is_some_and(|path| env::split_paths(&path).any(|dir| dir.join(first).exists()))
}

/// Kind-aware presence probe. Returns `(status, detail)` where status is one of
/// `present` / `missing` / `unknown`. It never fails: an absent extension is a
/// fact to report, not an error to raise.
fn scan_tool_status(
    repo_root: &Path,
    kind: &str,
    command: &str,
    scan_target: Option<&str>,
) -> (&'static str, String) {
    match kind {
        "cli" | "binary" => {
            if command_available(repo_root, command) {
                ("present", command.to_owned())
            } else {
                ("missing", command.to_owned())
            }
        }
        "mcp" | "skill" => match scan_target.map(str::trim).filter(|t| !t.is_empty()) {
            Some(target) => {
                if scan_target_resolves(repo_root, target) {
                    ("present", target.to_owned())
                } else {
                    ("missing", target.to_owned())
                }
            }
            None => (
                "unknown",
                "no scan target; agent confirms availability".to_owned(),
            ),
        },
        "http" => match scan_target.map(str::trim).filter(|t| !t.is_empty()) {
            Some(target) => {
                if http_reachable(target) || scan_target_resolves(repo_root, target) {
                    ("present", target.to_owned())
                } else {
                    ("missing", target.to_owned())
                }
            }
            None => ("unknown", "no scan target".to_owned()),
        },
        _ => ("unknown", String::new()),
    }
}

/// Resolve a declarative scan target as a filesystem path: `~` expands to HOME,
/// absolute paths are tested directly, relative paths are tested against the
/// repo root.
fn scan_target_resolves(repo_root: &Path, target: &str) -> bool {
    let expanded = expand_home(target);
    let path = Path::new(&expanded);
    if path.is_absolute() {
        path.exists()
    } else {
        repo_root.join(&expanded).exists()
    }
}

fn expand_home(target: &str) -> String {
    if let Some(rest) = target.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return format!("{}/{}", home.to_string_lossy(), rest);
        }
    }
    target.to_owned()
}

/// Best-effort TCP reachability for `http`/`https` scan targets. Any failure
/// (parse, DNS, timeout, refused) is reported as not reachable rather than an
/// error, so a down endpoint degrades the capability instead of breaking intake.
fn http_reachable(target: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let (default_port, rest) = if let Some(rest) = target.strip_prefix("https://") {
        (443u16, rest)
    } else if let Some(rest) = target.strip_prefix("http://") {
        (80u16, rest)
    } else {
        return false;
    };

    let authority = rest.split('/').next().unwrap_or("");
    if authority.is_empty() {
        return false;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().unwrap_or(default_port)),
        None => (authority, default_port),
    };

    let Ok(addresses) = (host, port).to_socket_addrs() else {
        return false;
    };
    addresses
        .into_iter()
        .any(|address| TcpStream::connect_timeout(&address, Duration::from_secs(2)).is_ok())
}

pub(crate) fn tool_args_json(args: &[ToolArgSpec]) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    Some(format!(
        "[{}]",
        args.iter()
            .map(|arg| {
                format!(
                    "{{\"name\":\"{}\",\"type\":\"{}\",\"required\":{},\"help\":\"{}\"}}",
                    escape_json(&arg.name),
                    escape_json(&arg.arg_type),
                    arg.required,
                    escape_json(arg.help.as_deref().unwrap_or(""))
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn parse_stored_tool_args(value: Option<&str>) -> Vec<ToolArgSpec> {
    let Some(value) = value else {
        return Vec::new();
    };
    if !value.contains("\"name\"") {
        return Vec::new();
    }
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split("},{")
        .filter_map(|raw| {
            let item = raw.trim_matches('{').trim_matches('}');
            let name = json_object_value(item, "name")?;
            let arg_type = json_object_value(item, "type").unwrap_or_else(|| "string".to_owned());
            let required = json_object_value(item, "required")
                .map(|value| value == "true")
                .unwrap_or(false);
            let help = json_object_value(item, "help").filter(|value| !value.is_empty());
            Some(ToolArgSpec {
                name,
                arg_type,
                required,
                help,
            })
        })
        .collect()
}

fn json_object_value(raw: &str, key: &str) -> Option<String> {
    let target = format!("\"{key}\":");
    let start = raw.find(&target)? + target.len();
    let rest = &raw[start..];
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        Some(rest[..end].to_owned())
    } else {
        Some(rest.split(',').next().unwrap_or_default().trim().to_owned())
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn audit_findings(connection: &Connection, sql: &str) -> Result<Vec<AuditFinding>> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok(AuditFinding {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;
    collect_rows(rows)
}

fn repeated_friction(connection: &Connection) -> Result<Vec<(String, usize)>> {
    let mut statement = connection.prepare(
        "SELECT harness_friction FROM trace
         WHERE harness_friction IS NOT NULL
           AND TRIM(harness_friction) <> ''
           AND LOWER(TRIM(harness_friction)) <> 'none';",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let values = collect_rows(rows)?;
    Ok(repeated_values(values))
}

fn repeated_interventions(connection: &Connection) -> Result<Vec<(String, usize)>> {
    let mut statement = connection.prepare(
        "SELECT type || ': ' || description FROM intervention
         WHERE TRIM(description) <> '';",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let values = collect_rows(rows)?;
    Ok(repeated_values(values))
}

fn repeated_story_signals(connection: &Connection) -> Result<Vec<(String, String, usize)>> {
    let mut statement = connection.prepare(
        "SELECT type, summary, COALESCE(story_id, '') FROM story_signal
         WHERE TRIM(summary) <> '';",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let pairs = collect_rows(rows)?;
    // Recurrence means the same signal across distinct stories; duplicate
    // signals on one story are noise, not a spec gap. Signals without a
    // story id cannot be deduplicated, so each counts as its own occurrence.
    let mut grouped: Vec<(String, String, String, Vec<String>)> = Vec::new();
    for (index, (signal_type, summary, story_id)) in pairs.into_iter().enumerate() {
        let story_id = if story_id.is_empty() {
            format!("?unattributed-{index}")
        } else {
            story_id
        };
        let key = format!("{}|{}", signal_type, normalize_token(&summary));
        if let Some(existing) = grouped.iter_mut().find(|item| item.0 == key) {
            if !existing.3.contains(&story_id) {
                existing.3.push(story_id);
            }
        } else {
            grouped.push((key, signal_type, summary, vec![story_id]));
        }
    }
    let grouped: Vec<(String, String, String, usize)> = grouped
        .into_iter()
        .map(|(key, signal_type, summary, stories)| (key, signal_type, summary, stories.len()))
        .collect();
    Ok(grouped
        .into_iter()
        .filter(|(_, _, _, count)| *count >= 2)
        .map(|(_, signal_type, summary, count)| (signal_type, summary, count))
        .collect())
}

fn repeated_values(values: Vec<String>) -> Vec<(String, usize)> {
    let mut grouped: Vec<(String, String, usize)> = Vec::new();
    for value in values {
        let key = normalize_token(&value);
        if let Some(existing) = grouped.iter_mut().find(|item| item.0 == key) {
            existing.2 += 1;
        } else {
            grouped.push((key, value, 1));
        }
    }
    grouped
        .into_iter()
        .filter(|(_, _, count)| *count >= 2)
        .map(|(_, value, count)| (value, count))
        .collect()
}

fn confidence_for_count(count: usize) -> String {
    if count >= 3 {
        "high".to_owned()
    } else {
        "medium".to_owned()
    }
}

fn short_title(value: &str) -> String {
    let words = value
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ");
    if words.len() > 72 {
        format!("{}...", &words[..69])
    } else {
        words
    }
}

/// Change diagrams (`docs/DIAGRAMS.md`) under `docs/**/diagrams/D<n>-*.md`
/// whose `Story:` header names `story_id` and whose `Status:` is not
/// `reviewed`. Returned as `"<relative path> (<status>)"`, sorted. A story
/// with such a diagram fails `story verify` before its command runs: a drawn
/// but unreviewed (or stale) diagram is an unmet done gate, not a warning.
pub fn unreviewed_diagrams(repo_root: &Path, story_id: &str) -> Vec<String> {
    fn walk(dir: &Path, story_id: &str, out: &mut Vec<(PathBuf, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, story_id, out);
                continue;
            }
            let in_diagrams_dir = path
                .parent()
                .and_then(|parent| parent.file_name())
                .is_some_and(|name| name == "diagrams");
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let is_diagram = name.len() > 3
                && name.starts_with('D')
                && name.as_bytes()[1].is_ascii_digit()
                && name.as_bytes()[2] == b'-'
                && name.ends_with(".md");
            if !(in_diagrams_dir && is_diagram) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let field = |key: &str| {
                text.lines()
                    .find_map(|line| line.strip_prefix(key))
                    .map(|value| value.trim().to_owned())
            };
            if field("Story:").as_deref() != Some(story_id) {
                continue;
            }
            let status = field("Status:").unwrap_or_else(|| "missing".to_owned());
            if status != "reviewed" {
                out.push((path, status));
            }
        }
    }

    let mut found = Vec::new();
    walk(&repo_root.join("docs"), story_id, &mut found);
    found.sort();
    found
        .into_iter()
        .map(|(path, status)| {
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            format!("{relative} ({status})")
        })
        .collect()
}

fn diagram_gate_message(story_id: &str, unreviewed: &[String]) -> String {
    let mut message =
        format!("story {story_id} has change diagrams that are not reviewed (docs/DIAGRAMS.md):\n");
    for item in unreviewed {
        message.push_str("  - ");
        message.push_str(item);
        message.push('\n');
    }
    message.push_str(
        "Review each (Status: reviewed + harness-cli intervention add --type review) before verifying.\n",
    );
    message
}

fn verifier_shell() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

fn is_decision_file_name(file_name: &str) -> bool {
    let Some((prefix, _)) = file_name.split_once('-') else {
        return false;
    };
    prefix.len() == 4 && prefix.chars().all(|character| character.is_ascii_digit())
}

fn sql_value_to_string(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(value) => format!("<{} bytes>", value.len()),
    }
}

// ---------------------------------------------------------------------------
// Shadow-mode rebuild (US-028a): deterministic replay of .harness/events/.
// ---------------------------------------------------------------------------

/// The eight durable tables and the column each is canonically ordered by.
const DURABLE_TABLES: [(&str, &str); 8] = [
    ("intake", "id"),
    ("story", "id"),
    ("decision", "id"),
    ("backlog", "id"),
    ("trace", "id"),
    ("tool", "name"),
    ("intervention", "id"),
    ("story_signal", "id"),
];

#[derive(Debug, PartialEq, Eq)]
pub struct RebuildResult {
    pub db_path: PathBuf,
    pub events_consumed: usize,
    pub table_counts: Vec<(String, i64)>,
    pub dump_hash: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrateToEventsResult {
    pub already_event_backed: bool,
    pub events_written: usize,
    pub table_counts: Vec<(String, i64)>,
    pub backup_db: Option<PathBuf>,
    pub archived_log_files: usize,
}

struct LogEvent {
    event_id: String,
    writer: String,
    recorded_at: String,
    op: String,
    payload: JsonValue,
    /// Per-writer consumed counts this writer had seen at append time
    /// (causal-audit signal, DKR-4). Absent on shadow/genesis events.
    observed: Option<JsonValue>,
    /// 1-based position of this event within its writer's file.
    writer_seq: i64,
}

impl SqliteHarnessRepository {
    pub(crate) fn cache_meta_get(connection: &Connection, key: &str) -> Result<Option<String>> {
        // cache_meta arrives with schema 007; older caches simply have no flag.
        let table_exists: Option<String> = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='cache_meta';",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if table_exists.is_none() {
            return Ok(None);
        }
        Ok(connection
            .query_row(
                "SELECT value FROM cache_meta WHERE key=?1;",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn cache_meta_set(connection: &Connection, key: &str, value: &str) -> Result<()> {
        connection.execute(
            "INSERT INTO cache_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value;",
            params![key, value],
        )?;
        Ok(())
    }

    fn write_watermark(
        connection: &Connection,
        file_name: &str,
        consumed_count: i64,
        file_size: i64,
        mtime_ns: i64,
        content_hash: &str,
    ) -> Result<()> {
        connection.execute(
            "INSERT INTO event_watermark (file, consumed_count, file_size, file_mtime_ns, content_hash)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(file) DO UPDATE SET
                consumed_count=excluded.consumed_count,
                file_size=excluded.file_size,
                file_mtime_ns=excluded.file_mtime_ns,
                content_hash=excluded.content_hash;",
            params![file_name, consumed_count, file_size, mtime_ns, content_hash],
        )?;
        Ok(())
    }

    /// One genesis event per durable row: writer `migration`, deterministic
    /// event ids, original timestamps, original ids preserved in payload.id
    /// (the three DKR-3 clauses).
    fn synthesize_genesis(&self, connection: &Connection) -> Result<Vec<LogEvent>> {
        let mut events = Vec::new();
        for (table, op) in [
            ("intake", "intake.record"),
            ("story", "story.add"),
            ("decision", "decision.add"),
            ("backlog", "backlog.add"),
            ("tool", "tool.register"),
            ("trace", "trace.record"),
            ("intervention", "intervention.add"),
            ("signal", "signal.add"),
        ] {
            let sql_table = if table == "signal" {
                "story_signal"
            } else {
                table
            };
            let mut statement =
                connection.prepare(&format!("SELECT * FROM {sql_table} ORDER BY rowid;"))?;
            let column_names: Vec<String> = statement
                .column_names()
                .iter()
                .map(|name| name.to_string())
                .collect();
            let mut rows = statement.query([])?;
            while let Some(row) = rows.next()? {
                let mut payload = serde_json::Map::new();
                let mut created_at: Option<String> = None;
                let mut row_key: Option<String> = None;
                for (index, name) in column_names.iter().enumerate() {
                    let value = match row.get_ref(index)? {
                        ValueRef::Null => JsonValue::Null,
                        ValueRef::Integer(value) => JsonValue::from(value),
                        ValueRef::Real(value) => JsonValue::from(value),
                        ValueRef::Text(value) => {
                            JsonValue::String(String::from_utf8_lossy(value).into_owned())
                        }
                        ValueRef::Blob(_) => {
                            return Err(HarnessInfraError::MigrationVerifyFailed(format!(
                                "{sql_table} has a BLOB column ({name}); genesis does not support blobs"
                            )))
                        }
                    };
                    match name.as_str() {
                        "created_at" => {
                            created_at = value.as_str().map(str::to_owned);
                        }
                        // Machine-local scan state is never logged.
                        "status" if sql_table == "tool" => {}
                        "checked_at" if sql_table == "tool" => {}
                        "id" | "name" => {
                            row_key = Some(match &value {
                                JsonValue::String(text) => text.clone(),
                                other => other.to_string(),
                            });
                            payload.insert(name.clone(), value);
                        }
                        _ => {
                            payload.insert(name.clone(), value);
                        }
                    }
                }
                let row_key = row_key.ok_or_else(|| {
                    HarnessInfraError::MigrationVerifyFailed(format!(
                        "{sql_table} row without id/name key"
                    ))
                })?;
                let created_at = created_at.ok_or_else(|| {
                    HarnessInfraError::MigrationVerifyFailed(format!(
                        "{sql_table} row {row_key} has no created_at"
                    ))
                })?;
                let unix = unix_from_sqlite_datetime(&created_at).ok_or_else(|| {
                    HarnessInfraError::MigrationVerifyFailed(format!(
                        "{sql_table} row {row_key}: unparseable created_at '{created_at}'"
                    ))
                })?;
                events.push(LogEvent {
                    event_id: genesis_ulid((unix.max(0) as u64) * 1000, sql_table, &row_key),
                    writer: "migration".to_owned(),
                    recorded_at: rfc3339_from_unix(unix),
                    op: op.to_owned(),
                    payload: JsonValue::Object(payload),
                    observed: None,
                    writer_seq: 0, // assigned when serialized in sorted order
                });
            }
        }
        events.sort_by(|left, right| {
            (left.event_id.as_str(), left.writer.as_str())
                .cmp(&(right.event_id.as_str(), right.writer.as_str()))
        });
        Ok(events)
    }

    /// US-028b: read the DB, synthesize genesis events, PROVE equality
    /// against the live DB, and only then install the log and demote the DB.
    /// Old DB backed up, never deleted; pre-cutover shadow logs archived
    /// (their content is already captured by genesis).
    pub fn migrate_to_events(&self) -> Result<MigrateToEventsResult> {
        let connection = self.open_existing()?;
        if Self::cache_meta_get(&connection, "event_backed")?.as_deref() == Some("true") {
            return Ok(MigrateToEventsResult {
                already_event_backed: true,
                events_written: 0,
                table_counts: Vec::new(),
                backup_db: None,
                archived_log_files: 0,
            });
        }

        let schema_version = Self::schema_version(&connection).unwrap_or(0);
        let expected = self
            .migration_files()?
            .last()
            .map(|(version, _)| *version)
            .unwrap_or(0);
        if schema_version < expected {
            return Err(HarnessInfraError::MigrationVerifyFailed(format!(
                "database schema is v{schema_version}, migrations go to v{expected} — run `harness-cli migrate` first"
            )));
        }
        self.guard_unimported_matrix_rows(&connection)?;
        let events = self.synthesize_genesis(&connection)?;

        // Prove before installing: rebuild from the candidate events and
        // compare per-table counts + content hashes against the live DB.
        let verify_path = self.repo_root.join(".harness/tmp-migration-verify.db");
        let rebuilt = self.build_cache_from_events(&events, &verify_path)?;
        let mut table_counts = Vec::with_capacity(DURABLE_TABLES.len());
        for (table, order_column) in DURABLE_TABLES {
            let excluded: &[&str] = if table == "tool" {
                &["status", "checked_at"]
            } else {
                &[]
            };
            let (live_count, live_dump) = dump_table(&connection, table, order_column, excluded)?;
            let (rebuilt_count, rebuilt_dump) =
                dump_table(&rebuilt, table, order_column, excluded)?;
            if live_count != rebuilt_count {
                return Err(HarnessInfraError::MigrationVerifyFailed(format!(
                    "{table}: row count {rebuilt_count} (rebuilt) != {live_count} (live)"
                )));
            }
            if fnv1a64(live_dump.as_bytes()) != fnv1a64(rebuilt_dump.as_bytes()) {
                return Err(HarnessInfraError::MigrationVerifyFailed(format!(
                    "{table}: content hash mismatch between live DB and genesis rebuild"
                )));
            }
            table_counts.push((table.to_owned(), live_count));
        }
        drop(rebuilt);
        let _ = fs::remove_file(&verify_path);

        // Install.
        let backup_dir = self.repo_root.join(".harness/backup");
        fs::create_dir_all(&backup_dir)?;
        let backup_db = backup_dir.join("harness.db.pre-migration");
        fs::copy(&self.db_path, &backup_db)?;

        let events_dir = self.repo_root.join(".harness/events");
        fs::create_dir_all(&events_dir)?;
        let archive_dir = backup_dir.join("pre-migration-events");
        let mut archived = 0usize;
        for entry in fs::read_dir(&events_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                fs::create_dir_all(&archive_dir)?;
                let file_name = path.file_name().expect("jsonl file name").to_owned();
                fs::rename(&path, archive_dir.join(file_name))?;
                archived += 1;
            }
        }

        let log_path = events_dir.join("migration.jsonl");
        let mut serialized = String::new();
        for event in &events {
            serialized.push_str(&EventLog::event_line(
                &event.event_id,
                &event.writer,
                &event.recorded_at,
                &event.op,
                &event.payload,
                None,
            ));
            serialized.push('\n');
        }
        fs::write(&log_path, &serialized)?;

        let mtime_ns = file_mtime_ns(&log_path)?;
        Self::write_watermark(
            &connection,
            "migration.jsonl",
            events.len() as i64,
            serialized.len() as i64,
            mtime_ns,
            &format!("{:016x}", fnv1a64(serialized.as_bytes())),
        )?;
        Self::cache_meta_set(&connection, "event_backed", "true")?;

        Ok(MigrateToEventsResult {
            already_event_backed: false,
            events_written: events.len(),
            table_counts,
            backup_db: Some(backup_db),
            archived_log_files: archived,
        })
    }

    /// US-028b generated views: human-readable markdown is a projection of
    /// the cache, never hand-edited. Output is idempotent (no timestamps,
    /// stable ordering) so PR diffs carry only real state changes — that is
    /// the pr_reviewability wall's surface.
    fn regenerate_views(&self, connection: &Connection) -> Result<()> {
        const MARKER: &str = "<!-- generated by harness-cli — do not hand-edit -->";

        // TEST_MATRIX.md
        let mut matrix = format!(
            "{MARKER}\n# Test Matrix\n\nGenerated view of the story table; the event log is the source of truth.\nUpdate proof with `harness-cli story update`, never by editing this file.\n\n| Story | Contract | Unit | Integration | E2E | Platform | Status | Evidence |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n"
        );
        let mut statement = connection.prepare(
            "SELECT id, COALESCE(contract_doc, title), unit_proof, integration_proof,
                    e2e_proof, platform_proof, status, COALESCE(evidence, '')
             FROM story ORDER BY id;",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;
        let proof = |value: i64| if value == 1 { "yes" } else { "no" };
        for (id, contract, unit, integration, e2e, platform, status, evidence) in
            collect_rows(rows)?
        {
            let evidence = evidence.replace('|', "\\|").replace('\n', " ");
            matrix.push_str(&format!(
                "| {id} | {contract} | {} | {} | {} | {} | {status} | {evidence} |\n",
                proof(unit),
                proof(integration),
                proof(e2e),
                proof(platform),
            ));
        }
        write_if_changed(&self.repo_root.join("docs/TEST_MATRIX.md"), &matrix)?;

        // HARNESS_BACKLOG.md
        let mut backlog = format!(
            "{MARKER}\n# Harness Backlog\n\nGenerated view of the backlog table; the event log is the source of truth.\nAdd items with `harness-cli backlog add`, close with `harness-cli backlog close`.\n\n| Id | Title | Risk | Status | Predicted impact | Actual outcome |\n| --- | --- | --- | --- | --- | --- |\n"
        );
        let mut statement = connection.prepare(
            "SELECT id, title, COALESCE(risk, ''), status,
                    COALESCE(predicted_impact, ''), COALESCE(actual_outcome, '')
             FROM backlog ORDER BY id;",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        for (id, title, risk, status, predicted, actual) in collect_rows(rows)? {
            let clean = |text: String| text.replace('|', "\\|").replace('\n', " ");
            backlog.push_str(&format!(
                "| {id} | {} | {risk} | {status} | {} | {} |\n",
                clean(title),
                clean(predicted),
                clean(actual),
            ));
        }
        write_if_changed(&self.repo_root.join("docs/HARNESS_BACKLOG.md"), &backlog)?;

        // Decision index
        let mut index = format!(
            "{MARKER}\n# Decisions\n\nGenerated index of the decision table; records live in this directory.\nAdd decisions with `harness-cli decision add` (doc from `docs/templates/decision.md`).\n\n| Id | Title | Status | Doc |\n| --- | --- | --- | --- |\n"
        );
        let mut statement = connection.prepare(
            "SELECT id, title, status, COALESCE(doc_path, '') FROM decision ORDER BY id;",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        for (id, title, status, doc) in collect_rows(rows)? {
            index.push_str(&format!("| {id} | {title} | {status} | {doc} |\n"));
        }
        write_if_changed(&self.repo_root.join("docs/decisions/README.md"), &index)?;
        Ok(())
    }

    /// Migration guard (parent spec + US-028b design): markdown story rows
    /// that never reached the database would be silently erased by the first
    /// view regeneration — refuse and point at import brownfield.
    fn guard_unimported_matrix_rows(&self, connection: &Connection) -> Result<()> {
        let matrix_path = self.repo_root.join("docs/TEST_MATRIX.md");
        if !matrix_path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&matrix_path)?;
        if content.starts_with("<!-- generated by harness-cli") {
            return Ok(());
        }
        let mut missing = Vec::new();
        let mut header_seen = false;
        for line in content.lines() {
            if !line.trim_start().starts_with('|') {
                continue;
            }
            let fields = markdown_table_fields(line);
            if fields.len() < 2 {
                continue;
            }
            if !header_seen {
                let candidate = MatrixColumns::from_header(&fields);
                if candidate.story.is_some() && candidate.status.is_some() {
                    header_seen = true;
                }
                continue;
            }
            let id = field_at(&fields, Some(0)).unwrap_or_default();
            let token = normalize_token(&id);
            if matches!(
                token.as_str(),
                "" | "story" | "status" | "meaning" | "tbd" | "todo" | "example" | "examples"
            ) || id.chars().all(|character| character == '-')
            {
                continue;
            }
            let exists: Option<String> = connection
                .query_row("SELECT id FROM story WHERE id=?1;", params![id], |row| {
                    row.get(0)
                })
                .optional()?;
            if exists.is_none() {
                missing.push(id);
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(HarnessInfraError::MigrationVerifyFailed(format!(
                "docs/TEST_MATRIX.md has stories missing from the database ({}) — run `harness-cli import brownfield` before migrating, or they will vanish from the generated view",
                missing.join(", ")
            )))
        }
    }

    /// Replay events into a fresh cache database at `db_path`.
    fn build_cache_from_events(&self, events: &[LogEvent], db_path: &Path) -> Result<Connection> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if db_path.exists() {
            fs::remove_file(db_path)?;
        }

        let connection = Connection::open(db_path)?;
        self.apply_schema_v1(&connection)?;
        self.apply_pending_migrations(&connection, 1)?;
        // A log may reference rows that predate it (shadow phase) and replay
        // order across writers is (event_id, writer), so referential
        // integrity is the log's contract, not SQLite FK enforcement.
        connection.pragma_update(None, "foreign_keys", "OFF")?;

        for event in events {
            apply_event(&connection, event)?;
        }
        Ok(connection)
    }

    /// Full deterministic replay from genesis into a fresh cache. Two rebuilds
    /// of the same log must produce identical dump hashes — every timestamp
    /// comes from the event, never from the wall clock.
    pub fn rebuild(&self, output: Option<PathBuf>) -> Result<RebuildResult> {
        let events = self.read_event_log()?;
        let db_path = output.unwrap_or_else(|| self.repo_root.join(".harness/shadow.db"));
        let connection = self.build_cache_from_events(&events, &db_path)?;

        let mut table_counts = Vec::with_capacity(DURABLE_TABLES.len());
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (table, order_column) in DURABLE_TABLES {
            let (count, table_dump) = dump_table(&connection, table, order_column, &[])?;
            table_counts.push((table.to_owned(), count));
            hash ^= crate::events::fnv1a64(table_dump.as_bytes());
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }

        Ok(RebuildResult {
            db_path,
            events_consumed: events.len(),
            table_counts,
            dump_hash: format!("{hash:016x}"),
        })
    }

    fn read_event_log(&self) -> Result<Vec<LogEvent>> {
        let events_dir = self.repo_root.join(".harness/events");
        let mut events = Vec::new();
        if !events_dir.is_dir() {
            return Ok(events);
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&events_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }
        files.sort();

        for path in files {
            let display = path.display().to_string();
            let content = fs::read_to_string(&path)?;
            for (index, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                events.push(parse_event_line(line, &display, index)?);
            }
        }

        // Total replay order per the US-028 spec: (event_id ULID, writer).
        events.sort_by(|left, right| {
            (left.event_id.as_str(), left.writer.as_str())
                .cmp(&(right.event_id.as_str(), right.writer.as_str()))
        });
        Ok(events)
    }
}

fn parse_event_line(line: &str, file: &str, line_index: usize) -> Result<LogEvent> {
    let value: JsonValue = serde_json::from_str(line).map_err(|error| {
        HarnessInfraError::CorruptEventLog(format!("{file}:{}", line_index + 1), error.to_string())
    })?;
    Ok(LogEvent {
        event_id: require_str(&value, "event_id", file, line_index)?,
        writer: require_str(&value, "writer", file, line_index)?,
        recorded_at: require_str(&value, "recorded_at", file, line_index)?,
        op: require_str(&value, "op", file, line_index)?,
        payload: value.get("payload").cloned().unwrap_or(JsonValue::Null),
        observed: value.get("observed").cloned(),
        writer_seq: (line_index + 1) as i64,
    })
}

fn require_str(value: &JsonValue, key: &str, file: &str, line_index: usize) -> Result<String> {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            HarnessInfraError::CorruptEventLog(
                format!("{file}:{}", line_index + 1),
                format!("missing or non-string field '{key}'"),
            )
        })
}

fn p_str(payload: &JsonValue, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_owned)
}

fn p_i64(payload: &JsonValue, key: &str) -> Option<i64> {
    payload.get(key).and_then(JsonValue::as_i64)
}

/// Row ids: ULID strings post-cutover, integers in legacy shadow events and
/// genesis payloads. Both land in TEXT id columns.
fn p_id(payload: &JsonValue, key: &str) -> Option<String> {
    match payload.get(key) {
        Some(JsonValue::String(value)) => Some(value.clone()),
        Some(JsonValue::Number(value)) => Some(value.to_string()),
        _ => None,
    }
}

/// JSON-array columns (risk_flags, trace lists, tool args) are stored as the
/// array's JSON text, exactly as the live write path stores them. Live events
/// carry them as arrays (serialized back to text); genesis events carry the
/// original column text verbatim as a string, so migration round-trips
/// byte-exactly regardless of the original formatting.
fn p_json_text(payload: &JsonValue, key: &str) -> Option<String> {
    match payload.get(key) {
        None | Some(JsonValue::Null) => None,
        Some(JsonValue::String(value)) => Some(value.clone()),
        Some(value) => Some(value.to_string()),
    }
}

/// Event `recorded_at` (RFC3339, second precision) to the `datetime('now')`
/// format the live schema writes, so rebuilt rows look native.
fn event_timestamp(recorded_at: &str) -> String {
    recorded_at
        .replace('T', " ")
        .trim_end_matches('Z')
        .to_owned()
}

/// Per-field last-writer-wins with the causal audit (DKR-4, US-028b).
///
/// A field only takes an incoming value when the incoming event is later in
/// the `(event_id, writer)` total order than the field's last update — which
/// makes incremental replay converge to the same state as a full rebuild
/// regardless of arrival order. A cross-writer update is CONCURRENT when the
/// incoming writer had not observed the last update's position in its
/// writer's file; concurrency is recorded in `lww_audit` (never a wall-clock
/// window — any window W silently misses skew > W).
fn apply_story_update(connection: &Connection, event: &LogEvent) -> Result<()> {
    const FIELDS: [&str; 7] = [
        "status",
        "evidence",
        "unit_proof",
        "integration_proof",
        "e2e_proof",
        "platform_proof",
        "verify_command",
    ];
    let payload = &event.payload;
    let Some(story_id) = p_str(payload, "id") else {
        return Ok(());
    };

    for field in FIELDS {
        if payload.get(field).is_none() {
            continue;
        }
        let last: Option<(String, String, i64)> = connection
            .query_row(
                "SELECT event_id, writer, writer_seq FROM field_last_update
                 WHERE story_id=?1 AND field=?2;",
                params![story_id, field],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let mut apply_field = true;
        if let Some((last_event, last_writer, last_seq)) = &last {
            let incoming_key = (event.event_id.as_str(), event.writer.as_str());
            let last_key = (last_event.as_str(), last_writer.as_str());
            if *last_writer != event.writer {
                let observed_count = event
                    .observed
                    .as_ref()
                    .and_then(|map| map.get(last_writer))
                    .and_then(JsonValue::as_i64)
                    .unwrap_or(0);
                if observed_count < *last_seq {
                    // Causally concurrent: record it, deterministically keyed
                    // so replay and incremental apply agree.
                    let (loser_event, loser_writer, winner_event, winner_writer) =
                        if incoming_key > last_key {
                            (
                                last_event.as_str(),
                                last_writer.as_str(),
                                event.event_id.as_str(),
                                event.writer.as_str(),
                            )
                        } else {
                            (
                                event.event_id.as_str(),
                                event.writer.as_str(),
                                last_event.as_str(),
                                last_writer.as_str(),
                            )
                        };
                    connection.execute(
                        "INSERT OR IGNORE INTO lww_audit
                            (loser_event_id, winner_event_id, story_id, field,
                             loser_writer, winner_writer)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                        params![
                            loser_event,
                            winner_event,
                            story_id,
                            field,
                            loser_writer,
                            winner_writer
                        ],
                    )?;
                }
            }
            if incoming_key <= last_key {
                apply_field = false;
            }
        }

        if apply_field {
            if let Some(number) = p_i64(payload, field) {
                connection.execute(
                    &format!("UPDATE story SET {field}=?1 WHERE id=?2;"),
                    params![number, story_id],
                )?;
            } else if let Some(text) = p_str(payload, field) {
                connection.execute(
                    &format!("UPDATE story SET {field}=?1 WHERE id=?2;"),
                    params![text, story_id],
                )?;
            }
            connection.execute(
                "INSERT INTO field_last_update (story_id, field, event_id, writer, writer_seq)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(story_id, field) DO UPDATE SET
                    event_id=excluded.event_id,
                    writer=excluded.writer,
                    writer_seq=excluded.writer_seq;",
                params![
                    story_id,
                    field,
                    event.event_id,
                    event.writer,
                    event.writer_seq
                ],
            )?;
        }
    }
    Ok(())
}

fn apply_event(connection: &Connection, event: &LogEvent) -> Result<()> {
    let payload = &event.payload;
    let created_at = event_timestamp(&event.recorded_at);
    match event.op.as_str() {
        "intake.record" => {
            connection.execute(
                "INSERT INTO intake (
                    id, created_at, input_type, summary, risk_lane,
                    risk_flags, affected_docs, story_id, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
                params![
                    p_id(payload, "id"),
                    created_at,
                    p_str(payload, "input_type"),
                    p_str(payload, "summary"),
                    p_str(payload, "risk_lane"),
                    p_json_text(payload, "risk_flags"),
                    p_json_text(payload, "affected_docs"),
                    p_str(payload, "story_id"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        "story.add" => {
            // Live events carry the add-form fields; genesis events carry the
            // full row (status, proofs, verify results) — defaults cover the
            // difference so one op serves both.
            connection.execute(
                "INSERT INTO story (
                    id, title, created_at, risk_lane, contract_doc, status,
                    unit_proof, integration_proof, e2e_proof, platform_proof,
                    evidence, verify_command, last_verified_at, last_verified_result, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, COALESCE(?6, 'planned'),
                    COALESCE(?7, 0), COALESCE(?8, 0), COALESCE(?9, 0), COALESCE(?10, 0),
                    ?11, ?12, ?13, ?14, ?15);",
                params![
                    p_str(payload, "id"),
                    p_str(payload, "title"),
                    created_at,
                    p_str(payload, "risk_lane"),
                    p_str(payload, "contract_doc"),
                    p_str(payload, "status"),
                    p_i64(payload, "unit_proof"),
                    p_i64(payload, "integration_proof"),
                    p_i64(payload, "e2e_proof"),
                    p_i64(payload, "platform_proof"),
                    p_str(payload, "evidence"),
                    p_str(payload, "verify_command"),
                    p_str(payload, "last_verified_at"),
                    p_str(payload, "last_verified_result"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        "story.update" => {
            apply_story_update(connection, event)?;
        }
        "story.verify_result" => {
            connection.execute(
                "UPDATE story SET last_verified_at=?1, last_verified_result=?2 WHERE id=?3;",
                params![created_at, p_str(payload, "result"), p_str(payload, "id")],
            )?;
        }
        "decision.add" => {
            connection.execute(
                "INSERT INTO decision (
                    id, title, created_at, status, doc_path, verify_command,
                    last_verified_at, last_verified_result, predicted_impact,
                    actual_outcome, notes
                 ) VALUES (?1, ?2, ?3, COALESCE(?4, 'proposed'), ?5, ?6, ?7, ?8, ?9, ?10, ?11);",
                params![
                    p_str(payload, "id"),
                    p_str(payload, "title"),
                    created_at,
                    p_str(payload, "status"),
                    p_str(payload, "doc_path"),
                    p_str(payload, "verify_command"),
                    p_str(payload, "last_verified_at"),
                    p_str(payload, "last_verified_result"),
                    p_str(payload, "predicted_impact"),
                    p_str(payload, "actual_outcome"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        "decision.update" => {
            const FIELDS: [&str; 6] = [
                "title",
                "status",
                "doc_path",
                "verify_command",
                "predicted_impact",
                "notes",
            ];
            if let Some(id) = p_str(payload, "id") {
                for field in FIELDS {
                    if let Some(value) = p_str(payload, field) {
                        connection.execute(
                            &format!("UPDATE decision SET {field}=?1 WHERE id=?2;"),
                            params![value, id],
                        )?;
                    }
                }
            }
        }
        "decision.verify_result" => {
            connection.execute(
                "UPDATE decision SET last_verified_at=?1, last_verified_result=?2 WHERE id=?3;",
                params![created_at, p_str(payload, "result"), p_str(payload, "id")],
            )?;
        }
        "backlog.add" => {
            connection.execute(
                "INSERT INTO backlog (
                    id, created_at, title, discovered_while, current_pain,
                    suggested_improvement, risk, status, predicted_impact,
                    actual_outcome, implemented_at, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, COALESCE(?8, 'proposed'), ?9, ?10, ?11, ?12);",
                params![
                    p_id(payload, "id"),
                    created_at,
                    p_str(payload, "title"),
                    p_str(payload, "discovered_while"),
                    p_str(payload, "current_pain"),
                    p_str(payload, "suggested_improvement"),
                    p_str(payload, "risk"),
                    p_str(payload, "status"),
                    p_str(payload, "predicted_impact"),
                    p_str(payload, "actual_outcome"),
                    p_str(payload, "implemented_at"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        "backlog.close" => {
            connection.execute(
                "UPDATE backlog SET status=?1, actual_outcome=?2, implemented_at=?3 WHERE id=?4;",
                params![
                    p_str(payload, "status"),
                    p_str(payload, "actual_outcome"),
                    created_at,
                    p_id(payload, "id"),
                ],
            )?;
        }
        "tool.register" => {
            // Scan state is machine-local and never logged: status starts
            // 'unknown', checked_at NULL, exactly like a fresh registration.
            connection.execute(
                "INSERT INTO tool (
                    name, created_at, provider, command, description, args,
                    responsibility, since, kind, capability, scan_target, status
                 ) VALUES (?1, ?2, COALESCE(?3, 'custom'), ?4, ?5, ?6, ?7,
                    COALESCE(?8, 'registered'), ?9, ?10, ?11, 'unknown');",
                params![
                    p_str(payload, "name"),
                    created_at,
                    p_str(payload, "provider"),
                    p_str(payload, "command"),
                    p_str(payload, "description"),
                    p_json_text(payload, "args"),
                    p_str(payload, "responsibility"),
                    p_str(payload, "since"),
                    p_str(payload, "kind"),
                    p_str(payload, "capability"),
                    p_str(payload, "scan_target"),
                ],
            )?;
        }
        "tool.remove" => {
            connection.execute(
                "DELETE FROM tool WHERE name=?1;",
                params![p_str(payload, "name")],
            )?;
        }
        "intervention.add" => {
            connection.execute(
                "INSERT INTO intervention (
                    id, created_at, trace_id, story_id, type, description, source, impact
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                params![
                    p_id(payload, "id"),
                    created_at,
                    p_id(payload, "trace_id"),
                    p_str(payload, "story_id"),
                    p_str(payload, "type"),
                    p_str(payload, "description"),
                    p_str(payload, "source"),
                    p_str(payload, "impact"),
                ],
            )?;
        }
        "signal.add" => {
            connection.execute(
                "INSERT INTO story_signal (
                    id, created_at, story_id, trace_id, type, summary, component, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                params![
                    p_id(payload, "id"),
                    created_at,
                    p_str(payload, "story_id"),
                    p_id(payload, "trace_id"),
                    p_str(payload, "type"),
                    p_str(payload, "summary"),
                    p_str(payload, "component"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        "trace.record" => {
            connection.execute(
                "INSERT INTO trace (
                    id, created_at, task_summary, intake_id, story_id, agent,
                    actions_taken, files_read, files_changed, decisions_made, errors,
                    outcome, duration_seconds, token_estimate, harness_friction, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16);",
                params![
                    p_id(payload, "id"),
                    created_at,
                    p_str(payload, "task_summary"),
                    p_id(payload, "intake_id"),
                    p_str(payload, "story_id"),
                    p_str(payload, "agent"),
                    p_json_text(payload, "actions_taken"),
                    p_json_text(payload, "files_read"),
                    p_json_text(payload, "files_changed"),
                    p_json_text(payload, "decisions_made"),
                    p_json_text(payload, "errors"),
                    p_str(payload, "outcome"),
                    p_i64(payload, "duration_seconds"),
                    p_i64(payload, "token_estimate"),
                    p_str(payload, "harness_friction"),
                    p_str(payload, "notes"),
                ],
            )?;
        }
        unknown => {
            // schema-field upcasters arrive with US-028b; in shadow phase an
            // unknown op means a corrupt or newer-format log — refuse.
            return Err(HarnessInfraError::CorruptEventLog(
                event.event_id.clone(),
                format!("unknown op '{unknown}'"),
            ));
        }
    }
    Ok(())
}

/// A `CsvList` renders as JSON array text for the database; event payloads
/// carry the same array parsed back to a JSON value.
fn csv_payload(list: &CsvList) -> JsonValue {
    list.as_json_text()
        .and_then(|text| serde_json::from_str::<JsonValue>(&text).ok())
        .unwrap_or(JsonValue::Null)
}

fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(());
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn file_mtime_ns(path: &Path) -> Result<i64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos() as i64)
        .unwrap_or(0);
    Ok(mtime)
}

/// Canonical, order-stable serialization of one table for the dump hash.
fn dump_table(
    connection: &Connection,
    table: &str,
    order_column: &str,
    excluded: &[&str],
) -> Result<(i64, String)> {
    let mut statement =
        connection.prepare(&format!("SELECT * FROM {table} ORDER BY {order_column};"))?;
    let column_count = statement.column_count();
    let included: Vec<usize> = (0..column_count)
        .filter(|index| !excluded.contains(&statement.column_name(*index).unwrap_or_default()))
        .collect();
    let mut rows = statement.query([])?;
    let mut dump = String::from(table);
    let mut count = 0i64;
    while let Some(row) = rows.next()? {
        count += 1;
        dump.push('\u{1e}');
        for (position, index) in included.iter().copied().enumerate() {
            if position > 0 {
                dump.push('\u{1f}');
            }
            match row.get_ref(index)? {
                ValueRef::Null => dump.push('\u{2205}'),
                ValueRef::Integer(value) => dump.push_str(&format!("i:{value}")),
                ValueRef::Real(value) => dump.push_str(&format!("r:{value}")),
                ValueRef::Text(value) => {
                    dump.push_str("t:");
                    dump.push_str(&String::from_utf8_lossy(value));
                }
                ValueRef::Blob(value) => {
                    dump.push_str("b:");
                    for byte in value {
                        dump.push_str(&format!("{byte:02x}"));
                    }
                }
            }
        }
    }
    Ok((count, dump))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::application::{
        BacklogAddInput, BacklogCloseInput, DecisionAddInput, IntakeInput, InterventionAddInput,
        InterventionFilter, StoryAddInput, StorySignalAddInput, StorySignalFilter,
        StoryUpdateInput, ToolRegisterInput, TraceInput,
    };
    use crate::domain::{BacklogFilter, BoolFlag, CsvList, InputType, RiskLane, TraceQualityTier};

    fn real_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    /// Isolated repo root (the repository now owns an event log rooted there —
    /// tests must NEVER write into the real repo's .harness/events) with the
    /// real schema dir.
    fn test_repository() -> (TempDir, SqliteHarnessRepository) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repository = SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("harness.db"),
            real_repo_root().join("scripts/schema"),
        );
        (temp_dir, repository)
    }

    fn write_diagram(repo_root: &Path, rel: &str, story: &str, status: &str) {
        let path = repo_root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "# D3 Sequence — {story}\n\nStory: {story}\nKind: sequence\nSource: hand\nScope: x\nStatus: {status}\nReviewed-by: -\nReviewed-at: -\n\n```mermaid\nsequenceDiagram\n```\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn unreviewed_diagrams_finds_only_matching_story_and_non_reviewed_status() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        write_diagram(root, "docs/stories/a/diagrams/D3-flow.md", "US-1", "draft");
        write_diagram(root, "docs/stories/a/diagrams/D4-state.md", "US-1", "stale");
        write_diagram(
            root,
            "docs/stories/a/diagrams/D5-data.md",
            "US-1",
            "reviewed",
        );
        write_diagram(root, "docs/stories/b/diagrams/D3-flow.md", "US-2", "draft");
        write_diagram(
            root,
            "docs/stories/a/notes/D3-not-a-diagram.md",
            "US-1",
            "draft",
        );
        write_diagram(
            root,
            "docs/templates/diagrams/D3-sequence.md",
            "US-XXX",
            "draft",
        );

        assert_eq!(
            unreviewed_diagrams(root, "US-1"),
            vec![
                "docs/stories/a/diagrams/D3-flow.md (draft)".to_owned(),
                "docs/stories/a/diagrams/D4-state.md (stale)".to_owned(),
            ]
        );
        assert!(unreviewed_diagrams(root, "US-3").is_empty());
        assert!(unreviewed_diagrams(root, "US-XXX").len() == 1);
    }

    #[test]
    fn story_verify_fails_on_unreviewed_diagram_without_running_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_root).unwrap();
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            real_repo_root().join("scripts/schema"),
        );
        repository.init().unwrap();
        let marker = repo_root.join("ran.txt");
        let verify_command = if cfg!(windows) {
            "echo ran > ran.txt".to_owned()
        } else {
            "touch ran.txt".to_owned()
        };
        repository
            .add_story(StoryAddInput {
                id: "US-D".to_owned(),
                title: "Diagram gated".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some(verify_command),
                notes: None,
            })
            .unwrap();
        write_diagram(
            &repo_root,
            "docs/stories/d/diagrams/D3-flow.md",
            "US-D",
            "draft",
        );

        let gated = repository.verify_story("US-D").unwrap();
        assert_eq!(gated.result, "fail");
        assert!(gated
            .stderr
            .contains("docs/stories/d/diagrams/D3-flow.md (draft)"));
        assert!(
            !marker.exists(),
            "verify_command must not run while a diagram is unreviewed"
        );
        assert_eq!(
            repository
                .story_verify_status("US-D")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("fail")
        );
        let all = repository.verify_all_stories().unwrap();
        assert_eq!(all.items[0].result, "fail");
        assert!(all.items[0].stderr.contains("not reviewed"));

        write_diagram(
            &repo_root,
            "docs/stories/d/diagrams/D3-flow.md",
            "US-D",
            "reviewed",
        );
        let passed = repository.verify_story("US-D").unwrap();
        assert_eq!(passed.result, "pass");
        assert!(marker.exists());
    }

    fn story_columns(connection: &Connection) -> Vec<String> {
        let mut statement = connection.prepare("PRAGMA table_info(story);").unwrap();
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap();
        rows.collect::<std::result::Result<Vec<_>, _>>().unwrap()
    }

    /// Isolated repo root with the real schema dir, driven through
    /// `HarnessService` so the full event-backed write path runs.
    fn events_test_service() -> (TempDir, crate::application::HarnessService) {
        let temp_dir = tempfile::tempdir().unwrap();
        let service = crate::application::HarnessService::new(crate::application::HarnessContext {
            repo_root: temp_dir.path().to_path_buf(),
            db_path: temp_dir.path().join("harness.db"),
            schema_dir: real_repo_root().join("scripts/schema"),
        });
        service.init().unwrap();
        (temp_dir, service)
    }

    fn events_seed_all_ops(service: &crate::application::HarnessService) {
        let intake_id = service
            .record_intake(IntakeInput {
                input_type: InputType::from_str("maintenance").unwrap(),
                summary: "shadow mode seed".to_owned(),
                risk_lane: RiskLane::from_str("tiny").unwrap(),
                risk_flags: CsvList::from_optional(Some("weak_proof".to_owned())),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: Some("seed".to_owned()),
            })
            .unwrap();
        service
            .add_story(StoryAddInput {
                id: "US-1".to_owned(),
                title: "shadow story".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: Some("docs/x.md".to_owned()),
                verify_command: None,
                notes: None,
            })
            .unwrap();
        service
            .update_story(StoryUpdateInput {
                id: "US-1".to_owned(),
                status: Some("implemented".to_owned()),
                evidence: Some("unit \"quoted\" evidence".to_owned()),
                unit: Some(BoolFlag(1)),
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();
        service
            .add_decision(DecisionAddInput {
                id: "0001".to_owned(),
                title: "shadow decision".to_owned(),
                status: "accepted".to_owned(),
                doc_path: None,
                verify_command: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        let backlog_id = service
            .add_backlog(BacklogAddInput {
                title: "shadow backlog".to_owned(),
                discovered_while: Some("testing".to_owned()),
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::from_str("tiny").unwrap()),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        service
            .close_backlog(BacklogCloseInput {
                id: backlog_id,
                status: "implemented".to_owned(),
                actual_outcome: Some("done".to_owned()),
            })
            .unwrap();
        service
            .register_tool(ToolRegisterInput {
                name: "shadow-tool".to_owned(),
                command: "skill:shadow".to_owned(),
                description: "Shadow tool for rebuild test".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: false,
                kind: "skill".to_owned(),
                capability: Some("impact-analysis".to_owned()),
                scan_target: Some(".shadow".to_owned()),
            })
            .unwrap();
        let trace_id = service
            .record_trace(TraceInput {
                task_summary: "shadow trace".to_owned(),
                intake_id: Some(intake_id.clone()),
                story_id: Some("US-1".to_owned()),
                agent: Some("test".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: Some(5),
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(Some("a,b".to_owned())),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        service
            .add_intervention(InterventionAddInput {
                trace_id: Some(trace_id.clone()),
                story_id: Some("US-1".to_owned()),
                intervention_type: "correction".to_owned(),
                description: "shadow correction".to_owned(),
                source: "human".to_owned(),
                impact: None,
            })
            .unwrap();
        service
            .add_story_signal(StorySignalAddInput {
                story_id: Some("US-1".to_owned()),
                trace_id: Some(trace_id),
                signal_type: "design_decision".to_owned(),
                summary: "shadow signal".to_owned(),
                component: None,
                notes: None,
            })
            .unwrap();
    }

    #[test]
    fn events_shadow_writes_one_event_per_durable_mutation() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        let events_dir = temp_dir.path().join(".harness/events");
        let mut files: Vec<_> = fs::read_dir(&events_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 1, "one writer, one file");
        let content = fs::read_to_string(&files[0]).unwrap();
        let ops: Vec<String> = content
            .lines()
            .map(|line| {
                serde_json::from_str::<JsonValue>(line).unwrap()["op"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(
            ops,
            vec![
                "intake.record",
                "story.add",
                "story.update",
                "decision.add",
                "backlog.add",
                "backlog.close",
                "tool.register",
                "trace.record",
                "intervention.add",
                "signal.add",
            ]
        );
    }

    #[test]
    fn events_rebuild_is_deterministic_and_reproduces_writes() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        let first = service
            .rebuild(Some(temp_dir.path().join("rebuild-1.db")))
            .unwrap();
        let second = service
            .rebuild(Some(temp_dir.path().join("rebuild-2.db")))
            .unwrap();

        // The determinism proof: two independent rebuilds, identical dumps.
        assert_eq!(first.dump_hash, second.dump_hash);
        assert_eq!(first.events_consumed, 10);
        let counts: Vec<(&str, i64)> = first
            .table_counts
            .iter()
            .map(|(table, count)| (table.as_str(), *count))
            .collect();
        assert_eq!(
            counts,
            vec![
                ("intake", 1),
                ("story", 1),
                ("decision", 1),
                ("backlog", 1),
                ("trace", 1),
                ("tool", 1),
                ("intervention", 1),
                ("story_signal", 1),
            ]
        );

        // Replay reproduced the mutation, not just the insert.
        let connection = Connection::open(&first.db_path).unwrap();
        let (status, unit_proof, evidence): (String, i64, String) = connection
            .query_row(
                "SELECT status, unit_proof, evidence FROM story WHERE id='US-1';",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "implemented");
        assert_eq!(unit_proof, 1);
        assert_eq!(evidence, "unit \"quoted\" evidence");
        let backlog_status: String = connection
            .query_row("SELECT status FROM backlog;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(backlog_status, "implemented");
        // Machine-local scan state was never logged: rebuilt tool is pristine.
        let (tool_status, checked_at): (String, Option<String>) = connection
            .query_row("SELECT status, checked_at FROM tool;", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(tool_status, "unknown");
        assert_eq!(checked_at, None);
    }

    #[test]
    fn events_rebuild_refuses_corrupt_log_line() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        let events_dir = temp_dir.path().join(".harness/events");
        let file = fs::read_dir(&events_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .unwrap();
        let mut content = fs::read_to_string(&file).unwrap();
        content.push_str("{not json\n");
        fs::write(&file, content).unwrap();

        let error = service
            .rebuild(Some(temp_dir.path().join("rebuild.db")))
            .unwrap_err();
        assert!(matches!(error, HarnessInfraError::CorruptEventLog(..)));
    }

    #[test]
    fn cutover_migration_007_preserves_rows_and_converts_ids_to_text() {
        let (_temp_dir, repository) = test_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO intake (input_type, summary, risk_lane)
                 VALUES ('maintenance', 'pre-007 row', 'tiny');",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO backlog (title, status) VALUES ('pre-007 backlog', 'proposed');",
                [],
            )
            .unwrap();
        drop(connection);

        repository.migrate().unwrap();

        let connection = repository.open_existing().unwrap();
        let (intake_id, summary): (String, String) = connection
            .query_row("SELECT id, summary FROM intake;", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(intake_id, "1");
        assert_eq!(summary, "pre-007 row");
        let backlog_id: String = connection
            .query_row("SELECT id FROM backlog;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(backlog_id, "1");
    }

    #[test]
    fn cutover_row_ids_are_ulids_and_prefixes_resolve() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let first = repository
            .record_trace(TraceInput {
                task_summary: "first trace for prefix resolution".to_owned(),
                intake_id: None,
                story_id: None,
                agent: None,
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        let second = repository
            .record_trace(TraceInput {
                task_summary: "second trace for prefix resolution".to_owned(),
                intake_id: None,
                story_id: None,
                agent: None,
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        assert_eq!(first.len(), 26);
        assert_ne!(first, second);

        let connection = repository.open_existing().unwrap();
        // Exact id resolves to itself.
        assert_eq!(
            SqliteHarnessRepository::resolve_row_id(&connection, "trace", &first).unwrap(),
            first
        );
        // A shared prefix (ULID time part) is ambiguous.
        let shared: String = first.chars().take(1).collect();
        assert!(matches!(
            SqliteHarnessRepository::resolve_row_id(&connection, "trace", &shared),
            Err(HarnessInfraError::AmbiguousRowId(..))
        ));
        // An unknown id is not found.
        assert!(matches!(
            SqliteHarnessRepository::resolve_row_id(&connection, "trace", "ZZZZZZ"),
            Err(HarnessInfraError::RowIdNotFound(..))
        ));
        // Legacy numeric ids resolve exactly.
        connection
            .execute(
                "INSERT INTO trace (id, task_summary) VALUES ('1', 'legacy row');",
                [],
            )
            .unwrap();
        assert_eq!(
            SqliteHarnessRepository::resolve_row_id(&connection, "trace", "1").unwrap(),
            "1"
        );
        // An unambiguous long prefix of a ULID resolves.
        let unique_prefix: String = first.chars().take(25).collect();
        let resolved =
            SqliteHarnessRepository::resolve_row_id(&connection, "trace", &unique_prefix);
        if let Ok(resolved) = resolved {
            assert_eq!(resolved, first);
        }
    }

    #[test]
    fn cutover_migrate_to_events_proves_equality_and_is_idempotent() {
        // A legacy database: rows exist but predate the event log (the
        // upgraded-v6 situation migrate-to-events exists for).
        let (temp_dir, repository) = test_repository();
        repository.init().unwrap();
        {
            let connection = repository.open_existing().unwrap();
            SqliteHarnessRepository::cache_meta_set(&connection, "event_backed", "false").unwrap();
            connection
                .execute_batch(
                    r#"
                    INSERT INTO intake (id, created_at, input_type, summary, risk_lane, risk_flags)
                    VALUES ('1', '2026-06-16 07:13:57', 'maintenance', 'legacy intake', 'tiny', '["weak_proof"]');
                    INSERT INTO story (id, title, created_at, risk_lane, status, unit_proof, evidence, verify_command, last_verified_at, last_verified_result)
                    VALUES ('US-L1', 'legacy "quoted" story', '2026-06-16 07:14:00', 'normal', 'implemented', 1, 'evidence text', 'true', '2026-06-17 01:00:00', 'pass');
                    INSERT INTO decision (id, title, created_at, status, doc_path)
                    VALUES ('0004', 'legacy decision', '2026-06-16 07:15:00', 'accepted', 'docs/decisions/0004.md');
                    INSERT INTO backlog (id, created_at, title, status, actual_outcome, implemented_at)
                    VALUES ('1', '2026-06-16 07:16:00', 'legacy backlog', 'implemented', 'done', '2026-06-18 02:00:00');
                    INSERT INTO tool (name, created_at, command, description, responsibility, kind, capability, scan_target, status, checked_at)
                    VALUES ('legacy-tool', '2026-06-16 07:17:00', 'skill:x', 'Legacy tool', 'Verification', 'skill', 'impact-analysis', '.x', 'present', '2026-07-01 00:00:00');
                    INSERT INTO trace (id, created_at, task_summary, intake_id, story_id, outcome, actions_taken)
                    VALUES ('1', '2026-06-16 07:18:00', 'legacy trace', '1', 'US-L1', 'completed', '["a","b"]');
                    INSERT INTO intervention (id, created_at, trace_id, type, description, source)
                    VALUES ('1', '2026-06-16 07:19:00', '1', 'correction', 'legacy correction', 'human');
                    INSERT INTO story_signal (id, created_at, story_id, trace_id, type, summary)
                    VALUES ('1', '2026-06-16 07:20:00', 'US-L1', '1', 'deviation', 'legacy signal');
                    "#,
                )
                .unwrap();
        }

        let service = crate::application::HarnessService::new(crate::application::HarnessContext {
            repo_root: temp_dir.path().to_path_buf(),
            db_path: temp_dir.path().join("harness.db"),
            schema_dir: real_repo_root().join("scripts/schema"),
        });

        // Pre-migration, mutations refuse: the log is not yet the truth.
        let refused = service
            .add_backlog(BacklogAddInput {
                title: "must refuse".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap_err();
        assert!(matches!(refused, HarnessInfraError::NotEventBacked));

        let result = service.migrate_to_events().unwrap();
        assert!(!result.already_event_backed);
        assert_eq!(result.events_written, 8); // one genesis event per legacy row
        assert!(result.backup_db.as_ref().unwrap().exists());

        let events_dir = temp_dir.path().join(".harness/events");
        let log = fs::read_to_string(events_dir.join("migration.jsonl")).unwrap();
        assert_eq!(log.lines().count(), 8);
        let first: JsonValue = serde_json::from_str(log.lines().next().unwrap()).unwrap();
        assert_eq!(first["writer"], "migration");

        // Rebuild from ONLY the migration log reproduces every durable row,
        // with machine-local tool scan state reset (never logged).
        let rebuild = service
            .rebuild(Some(temp_dir.path().join("verify.db")))
            .unwrap();
        assert_eq!(rebuild.events_consumed, 8);
        assert!(rebuild.table_counts.iter().all(|(_, count)| *count == 1));
        let rebuilt = Connection::open(temp_dir.path().join("verify.db")).unwrap();
        let (evidence, verified): (String, String) = rebuilt
            .query_row(
                "SELECT evidence, last_verified_result FROM story WHERE id='US-L1';",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(evidence, "evidence text");
        assert_eq!(verified, "pass");
        let tool_status: String = rebuilt
            .query_row("SELECT status FROM tool;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tool_status, "unknown");

        // Idempotent: second run is a no-op.
        let second = service.migrate_to_events().unwrap();
        assert!(second.already_event_backed);
        assert_eq!(
            fs::read_to_string(events_dir.join("migration.jsonl")).unwrap(),
            log
        );

        // Post-migration, the cutover write path works: mutations append to
        // this writer's log and land in the cache.
        let new_id = service
            .add_backlog(BacklogAddInput {
                title: "post-cutover item".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        assert_eq!(new_id.len(), 26);
        let backlog = service.query_backlog(BacklogFilter::All).unwrap();
        assert_eq!(backlog.len(), 2);
    }

    #[test]
    fn cutover_incremental_replay_applies_pulled_events_before_queries() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        // Simulate `git pull`: a teammate's writer file appears with a
        // story.update this cache has never applied.
        let teammate = EventLog::with_writer(
            temp_dir.path().join(".harness/events"),
            "teammate1".to_owned(),
        );
        teammate
            .emit(
                "story.update",
                json!({"id": "US-1", "status": "changed", "evidence": "teammate evidence"}),
            )
            .unwrap();

        // The very next query sees the teammate's write (watermark replay).
        let matrix = service.query_matrix().unwrap();
        let row = matrix.iter().find(|record| record.id == "US-1").unwrap();
        assert_eq!(row.status, "changed");
        assert_eq!(row.evidence.as_deref(), Some("teammate evidence"));

        // And the file grows: appending MORE events to the same teammate
        // file exercises the stat+prefix-hash incremental path.
        teammate
            .emit("story.update", json!({"id": "US-1", "status": "retired"}))
            .unwrap();
        let matrix = service.query_matrix().unwrap();
        let row = matrix.iter().find(|record| record.id == "US-1").unwrap();
        assert_eq!(row.status, "retired");
    }

    #[test]
    fn cutover_fresh_clone_rebuilds_cache_from_log() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        // A fresh clone has the log but no cache.
        fs::remove_file(temp_dir.path().join("harness.db")).unwrap();
        let fresh = crate::application::HarnessService::new(crate::application::HarnessContext {
            repo_root: temp_dir.path().to_path_buf(),
            db_path: temp_dir.path().join("harness.db"),
            schema_dir: real_repo_root().join("scripts/schema"),
        });
        let matrix = fresh.query_matrix().unwrap();
        assert_eq!(matrix.len(), 1);
        assert_eq!(matrix[0].id, "US-1");
        assert_eq!(matrix[0].status, "implemented");
        // And the rebuilt cache accepts new writes immediately.
        let id = fresh
            .add_backlog(BacklogAddInput {
                title: "post-rebuild write".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn cutover_mutation_fails_when_log_is_unwritable() {
        let (temp_dir, service) = events_test_service();
        // Make the events dir unwritable by replacing it with a file.
        let events_dir = temp_dir.path().join(".harness/events");
        if events_dir.exists() {
            fs::remove_dir_all(&events_dir).unwrap();
        }
        fs::create_dir_all(temp_dir.path().join(".harness")).unwrap();
        fs::write(&events_dir, "not a directory").unwrap();

        // US-028a's shadow guarantee is flipped: the log is the write of
        // record, so a failed append fails the command...
        let error = service
            .add_story(StoryAddInput {
                id: "US-X".to_owned(),
                title: "must fail".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap_err();
        assert!(matches!(error, HarnessInfraError::Io(_)));

        // ...and the cache transaction rolled back with it: no drift.
        fs::remove_file(&events_dir).unwrap();
        let matrix = service.query_matrix().unwrap();
        assert!(matrix.iter().all(|record| record.id != "US-X"));
    }

    fn own_writer_name(events_dir: &Path) -> String {
        fs::read_dir(events_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_owned)
            })
            .next()
            .expect("one writer file")
    }

    #[test]
    fn cutover_causal_audit_flags_concurrent_updates_and_lww_guard_holds() {
        let (temp_dir, service) = events_test_service();
        service
            .add_story(StoryAddInput {
                id: "US-1".to_owned(),
                title: "audited story".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        service
            .update_story(StoryUpdateInput {
                id: "US-1".to_owned(),
                status: Some("in_progress".to_owned()),
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();

        // A teammate updated the SAME field without ever observing this
        // writer's update (concurrent), with a LATER event id: teammate wins,
        // and the collision is audited.
        let events_dir = temp_dir.path().join(".harness/events");
        let teammate = EventLog::with_writer(events_dir.clone(), "teammate1".to_owned());
        let line = EventLog::event_line(
            "7ZZZZZZZZZZZZZZZZZZZZZZZZ1",
            "teammate1",
            "2026-07-02T08:00:00Z",
            "story.update",
            &json!({"id": "US-1", "status": "changed"}),
            None,
        );
        teammate.append_line(&line).unwrap();

        let matrix = service.query_matrix().unwrap();
        assert_eq!(matrix[0].status, "changed");
        let audit = service.audit().unwrap();
        assert_eq!(audit.concurrent_lww_updates.len(), 1);
        assert!(audit.concurrent_lww_updates[0].id.contains("US-1.status"));

        // A LATE-ARRIVING OLDER concurrent update must not overwrite (the
        // LWW guard is what makes incremental replay match a full rebuild)
        // but is still audited as a loser.
        let older = EventLog::event_line(
            "0000000000000000000000000A",
            "teammate2",
            "2026-07-02T00:00:00Z",
            "story.update",
            &json!({"id": "US-1", "status": "retired"}),
            None,
        );
        EventLog::with_writer(events_dir.clone(), "teammate2".to_owned())
            .append_line(&older)
            .unwrap();

        let matrix = service.query_matrix().unwrap();
        assert_eq!(matrix[0].status, "changed", "older event must not win");
        let audit = service.audit().unwrap();
        assert_eq!(audit.concurrent_lww_updates.len(), 2);

        // Determinism: a full rebuild from the merged log converges on the
        // same final state the incremental path produced.
        let rebuild_path = temp_dir.path().join("rebuild.db");
        service.rebuild(Some(rebuild_path.clone())).unwrap();
        let rebuilt = Connection::open(&rebuild_path).unwrap();
        let status: String = rebuilt
            .query_row("SELECT status FROM story WHERE id='US-1';", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "changed");
    }

    #[test]
    fn cutover_observed_sequential_updates_are_not_audited() {
        let (temp_dir, service) = events_test_service();
        service
            .add_story(StoryAddInput {
                id: "US-1".to_owned(),
                title: "sequential story".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        service
            .update_story(StoryUpdateInput {
                id: "US-1".to_owned(),
                status: Some("in_progress".to_owned()),
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();

        // The teammate HAD pulled and consumed this writer's file before
        // editing (happens-before): same-field update, but not concurrent.
        let events_dir = temp_dir.path().join(".harness/events");
        let own_writer = own_writer_name(&events_dir);
        let line = EventLog::event_line(
            "7ZZZZZZZZZZZZZZZZZZZZZZZZ2",
            "teammate1",
            "2026-07-02T08:00:00Z",
            "story.update",
            &json!({"id": "US-1", "status": "implemented"}),
            Some(&json!({ own_writer: 100 })),
        );
        EventLog::with_writer(events_dir, "teammate1".to_owned())
            .append_line(&line)
            .unwrap();

        let matrix = service.query_matrix().unwrap();
        assert_eq!(matrix[0].status, "implemented");
        let audit = service.audit().unwrap();
        assert!(audit.concurrent_lww_updates.is_empty());
    }

    #[test]
    fn cutover_generated_views_are_stable_projections_of_the_cache() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);

        let matrix_path = temp_dir.path().join("docs/TEST_MATRIX.md");
        let backlog_path = temp_dir.path().join("docs/HARNESS_BACKLOG.md");
        let index_path = temp_dir.path().join("docs/decisions/README.md");
        for path in [&matrix_path, &backlog_path, &index_path] {
            let content = fs::read_to_string(path).unwrap();
            assert!(content.starts_with("<!-- generated by harness-cli"));
        }
        let matrix = fs::read_to_string(&matrix_path).unwrap();
        assert!(matrix.contains("| US-1 |"));
        assert!(matrix.contains("| implemented |"));
        let backlog = fs::read_to_string(&backlog_path).unwrap();
        assert!(backlog.contains("shadow backlog"));
        assert!(backlog.contains("| implemented |"));
        let index = fs::read_to_string(&index_path).unwrap();
        assert!(index.contains("| 0001 | shadow decision | accepted |"));

        // Idempotent: a pure read regenerates nothing (byte-identical).
        let before = fs::read_to_string(&matrix_path).unwrap();
        service.query_matrix().unwrap();
        assert_eq!(fs::read_to_string(&matrix_path).unwrap(), before);
    }

    #[test]
    fn cutover_migration_guard_refuses_unimported_matrix_rows() {
        let (temp_dir, repository) = test_repository();
        repository.init().unwrap();
        {
            let connection = repository.open_existing().unwrap();
            SqliteHarnessRepository::cache_meta_set(&connection, "event_backed", "false").unwrap();
            connection
                .execute(
                    "INSERT INTO story (id, title, created_at, risk_lane)
                     VALUES ('US-A', 'known story', '2026-06-01 00:00:00', 'normal');",
                    [],
                )
                .unwrap();
        }
        fs::create_dir_all(temp_dir.path().join("docs")).unwrap();
        fs::write(
            temp_dir.path().join("docs/TEST_MATRIX.md"),
            r#"# Test Matrix

| Story | Contract | Unit | Integration | E2E | Platform | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US-A | docs/a.md | yes | no | no | no | implemented | |
| US-GHOST | docs/ghost.md | no | no | no | no | planned | |
"#,
        )
        .unwrap();

        let error = repository.migrate_to_events().unwrap_err();
        match error {
            HarnessInfraError::MigrationVerifyFailed(detail) => {
                assert!(detail.contains("US-GHOST"));
                assert!(!detail.contains("US-A,"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    fn git(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn clone_service(root: &Path) -> crate::application::HarnessService {
        crate::application::HarnessService::new(crate::application::HarnessContext {
            repo_root: root.to_path_buf(),
            db_path: root.join("harness.db"),
            schema_dir: real_repo_root().join("scripts/schema"),
        })
    }

    #[test]
    fn cutover_two_writer_git_merge_has_zero_conflicts() {
        // THE merge_conflict_count == 0 anti-goal read (frame 0008): two
        // clones, same human email (decision 0008 Q3 scenario), disjoint
        // writes on diverging histories, merged with zero conflicts, rebuild
        // on the merge contains every record from both writers.
        let temp = tempfile::tempdir().unwrap();
        let origin = temp.path().join("origin.git");
        fs::create_dir_all(&origin).unwrap();
        git(&origin, &["init", "--bare", "."]);

        let clone_a = temp.path().join("a");
        git(temp.path(), &["clone", origin.to_str().unwrap(), "a"]);
        for (key, value) in [("user.email", "dev@example.com"), ("user.name", "Dev")] {
            git(&clone_a, &["config", key, value]);
        }
        // Seed: clone A initializes and pushes the (empty) event-backed base.
        fs::write(
            clone_a.join(".gitignore"),
            "harness.db*\n.harness/shadow.db\n",
        )
        .unwrap();
        let service_a = clone_service(&clone_a);
        service_a.init().unwrap();
        service_a
            .add_story(StoryAddInput {
                id: "US-A".to_owned(),
                title: "story from clone A".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        git(&clone_a, &["add", "-A"]);
        git(&clone_a, &["commit", "-m", "clone A state"]);
        git(&clone_a, &["push", "origin", "HEAD:main"]);

        // Clone B: same email, fresh clone (no harness.db — auto-rebuild).
        let clone_b = temp.path().join("b");
        git(
            temp.path(),
            &["clone", "--branch", "main", origin.to_str().unwrap(), "b"],
        );
        for (key, value) in [("user.email", "dev@example.com"), ("user.name", "Dev")] {
            git(&clone_b, &["config", key, value]);
        }
        let service_b = clone_service(&clone_b);
        let matrix_b = service_b.query_matrix().unwrap();
        assert_eq!(matrix_b.len(), 1, "fresh clone sees A's story via rebuild");

        // Diverge: A and B each write without seeing the other.
        service_a
            .add_backlog(BacklogAddInput {
                title: "backlog from A".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        git(&clone_a, &["add", "-A"]);
        git(&clone_a, &["commit", "-m", "A adds backlog"]);
        git(&clone_a, &["push", "origin", "HEAD:main"]);

        service_b
            .add_story(StoryAddInput {
                id: "US-B".to_owned(),
                title: "story from clone B".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        git(&clone_b, &["add", "-A"]);
        git(&clone_b, &["commit", "-m", "B adds story"]);

        // Same email, two clones: the per-clone disambiguator must have kept
        // their writer files distinct (decision 0008 Q3).
        let files_b: Vec<String> = fs::read_dir(clone_b.join(".harness/events"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".jsonl"))
            .collect();
        assert_eq!(files_b.len(), 2, "A's file + B's own file: {files_b:?}");

        // Merge A's push into B: zero conflicts by construction.
        git(&clone_b, &["fetch", "origin"]);
        let merge = Command::new("git")
            .args(["merge", "--no-edit", "origin/main"])
            .current_dir(&clone_b)
            .output()
            .unwrap();
        assert!(
            merge.status.success(),
            "merge_conflict_count != 0: {}",
            String::from_utf8_lossy(&merge.stdout)
        );

        // Rebuild on the merge result contains every record from both.
        let matrix = service_b.query_matrix().unwrap();
        let ids: Vec<&str> = matrix.iter().map(|row| row.id.as_str()).collect();
        assert!(ids.contains(&"US-A") && ids.contains(&"US-B"));
        let backlog = service_b.query_backlog(BacklogFilter::All).unwrap();
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].title, "backlog from A");
        let rebuild = service_b.rebuild(Some(clone_b.join("verify.db"))).unwrap();
        assert_eq!(
            rebuild
                .table_counts
                .iter()
                .map(|(_, count)| count)
                .sum::<i64>(),
            3
        );
    }

    /// write_latency_ms <= 100 p95 wall read (frame 0008). Ignored in the
    /// default suite; run with: cargo test -p harness-cli bench -- --ignored --nocapture
    #[test]
    #[ignore = "benchmark: run explicitly for the latency wall read"]
    fn cutover_bench_write_latency_p95_under_wall_at_scale() {
        for log_size in [1_000usize, 10_000, 100_000] {
            let temp = tempfile::tempdir().unwrap();
            let events_dir = temp.path().join(".harness/events");
            fs::create_dir_all(&events_dir).unwrap();
            let mut lines = String::new();
            for index in 0..log_size {
                lines.push_str(&EventLog::event_line(
                    &mint_ulid(),
                    "bench",
                    "2026-07-02T00:00:00Z",
                    "intake.record",
                    &json!({
                        "id": format!("bench-{index}"),
                        "input_type": "maintenance",
                        "summary": format!("bench intake row number {index} with some realistic text"),
                        "risk_lane": "tiny",
                    }),
                    None,
                ));
                lines.push('\n');
            }
            fs::write(events_dir.join("bench.jsonl"), lines).unwrap();

            let service = clone_service(temp.path());
            // First touch pays the full rebuild (fresh-clone path).
            let rebuild_start = std::time::Instant::now();
            service.query_stats().unwrap();
            let rebuild_ms = rebuild_start.elapsed().as_secs_f64() * 1_000.0;

            let mut samples = Vec::with_capacity(50);
            for index in 0..50 {
                let start = std::time::Instant::now();
                service
                    .add_backlog(BacklogAddInput {
                        title: format!("bench mutation {index}"),
                        discovered_while: None,
                        current_pain: None,
                        suggestion: None,
                        risk: None,
                        predicted_impact: None,
                        notes: None,
                    })
                    .unwrap();
                samples.push(start.elapsed().as_secs_f64() * 1_000.0);
            }
            samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p50 = samples[samples.len() / 2];
            let p95 = samples[(samples.len() as f64 * 0.95) as usize];
            println!(
                "log={log_size}: rebuild {rebuild_ms:.1}ms, mutation p50={p50:.2}ms p95={p95:.2}ms"
            );
            assert!(
                p95 < 100.0,
                "write_latency_ms p95 {p95:.2} breaches the 100ms wall at {log_size} events"
            );
        }
    }

    #[test]
    fn cutover_genesis_generation_is_deterministic() {
        let (temp_dir, service) = events_test_service();
        events_seed_all_ops(&service);
        // Two independent synthesize passes over the same DB must be
        // byte-identical (deterministic event ids + sorted key serialization).
        let schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("scripts/schema");
        let repository = SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("harness.db"),
            schema_dir,
        );
        let connection = repository.open_existing().unwrap();
        let first = repository.synthesize_genesis(&connection).unwrap();
        let second = repository.synthesize_genesis(&connection).unwrap();
        let render = |events: &[LogEvent]| -> String {
            events
                .iter()
                .map(|event| {
                    format!(
                        "{}|{}|{}|{}|{}",
                        event.event_id, event.writer, event.recorded_at, event.op, event.payload
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(render(&first), render(&second));
    }

    #[test]
    fn events_rebuild_on_empty_log_yields_empty_tables() {
        let (temp_dir, service) = events_test_service();
        let result = service
            .rebuild(Some(temp_dir.path().join("rebuild.db")))
            .unwrap();
        assert_eq!(result.events_consumed, 0);
        assert!(result.table_counts.iter().all(|(_, count)| *count == 0));
    }

    #[test]
    fn init_creates_database_and_schema() {
        let (_temp_dir, repository) = test_repository();

        let result = repository.init().unwrap();

        assert!(matches!(result, InitResult::Created { .. }));
        assert_eq!(repository.query_stats().unwrap().intakes, 0);
        let connection = repository.open_existing().unwrap();
        let schema_version = SqliteHarnessRepository::schema_version(&connection).unwrap();
        assert_eq!(schema_version, SUPPORTED_SCHEMA_VERSION);
        let story_columns = story_columns(&connection);
        assert!(story_columns.contains(&"verify_command".to_owned()));
        assert!(story_columns.contains(&"last_verified_at".to_owned()));
        assert!(story_columns.contains(&"last_verified_result".to_owned()));
    }

    #[test]
    fn migrate_applies_story_verify_columns_to_existing_database() {
        let (_temp_dir, repository) = test_repository();
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        drop(connection);

        let result = repository.migrate().unwrap();

        assert_eq!(result.current_version, 1);
        assert_eq!(result.applied, vec![2, 3, 4, 5, 6, 7, 8]);
        let connection = repository.open_existing().unwrap();
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            SUPPORTED_SCHEMA_VERSION
        );
        let story_columns = story_columns(&connection);
        assert!(story_columns.contains(&"verify_command".to_owned()));
        assert!(story_columns.contains(&"last_verified_at".to_owned()));
        assert!(story_columns.contains(&"last_verified_result".to_owned()));
    }

    #[test]
    fn migration_005_backfills_kind_from_command_prefix() {
        let (_temp_dir, repository) = test_repository();
        let schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("scripts/schema");

        // Build a pre-kind (v4) database: v1 base plus migrations 002-004 only.
        let connection = repository.open_or_create().unwrap();
        repository.apply_schema_v1(&connection).unwrap();
        for file in [
            "002-story-verify.sql",
            "003-tool-registry.sql",
            "004-intervention.sql",
        ] {
            let sql = std::fs::read_to_string(schema_dir.join(file)).unwrap();
            connection.execute_batch(&sql).unwrap();
        }
        assert_eq!(
            SqliteHarnessRepository::schema_version(&connection).unwrap(),
            4
        );

        // Insert tools the old way (no kind column existed yet).
        for (name, command) in [
            ("mcp-example", "mcp:example-server"),
            ("skill-example", "skill:example-skill"),
            ("cli-example", "./deploy.sh"),
        ] {
            connection
                .execute(
                    "INSERT INTO tool (name, command, description, responsibility)
                     VALUES (?1, ?2, 'pre-kind registered tool example', 'Verification');",
                    params![name, command],
                )
                .unwrap();
        }
        drop(connection);

        // Upgrade: migration 005 must infer kind from the command prefix.
        // (Migration 006 adds story_signal and rides along in the same upgrade.)
        assert_eq!(repository.migrate().unwrap().applied, vec![5, 6, 7, 8]);
        let connection = repository.open_existing().unwrap();
        let kind_of = |name: &str| -> String {
            connection
                .query_row(
                    "SELECT kind FROM tool WHERE name=?1;",
                    params![name],
                    |row| row.get::<_, String>(0),
                )
                .unwrap()
        };
        assert_eq!(kind_of("mcp-example"), "mcp");
        assert_eq!(kind_of("skill-example"), "skill");
        assert_eq!(kind_of("cli-example"), "cli");
    }

    #[test]
    fn records_and_queries_intake() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        let id = repository
            .record_intake(IntakeInput {
                input_type: InputType::HarnessImprovement,
                summary: "Port one CLI slice".to_owned(),
                risk_lane: RiskLane::HighRisk,
                risk_flags: CsvList::from_optional(Some("public contracts".to_owned())),
                affected_docs: CsvList::from_optional(None),
                story_id: Some("US-002".to_owned()),
                notes: None,
            })
            .unwrap();

        let intakes = repository.query_intakes().unwrap();
        assert_eq!(id.len(), 26);
        assert_eq!(intakes[0].id, id);
        assert_eq!(intakes[0].summary, "Port one CLI slice");
        assert_eq!(intakes[0].input_type, "harness_improvement");
        assert_eq!(intakes[0].risk_lane, "high_risk");

        let connection = repository.open_existing().unwrap();
        let missing_lists_are_null: (bool, bool) = connection
            .query_row(
                "SELECT risk_flags IS NULL, affected_docs IS NULL FROM intake WHERE id=?1;",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(missing_lists_are_null, (false, true));
    }

    #[test]
    fn decision_verify_runs_from_repo_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_root).unwrap();
        let schema_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
            .join("scripts/schema");
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            schema_root,
        );
        repository.init().unwrap();

        let pwd_output = repo_root.join("verify-pwd.txt");
        let verify_command = if cfg!(windows) {
            "cd > verify-pwd.txt".to_owned()
        } else {
            "pwd > verify-pwd.txt".to_owned()
        };
        repository
            .add_decision(DecisionAddInput {
                id: "0001-test".to_owned(),
                title: "Verify from root".to_owned(),
                status: "accepted".to_owned(),
                doc_path: None,
                verify_command: Some(verify_command),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();

        let result = repository.verify_decision("0001-test").unwrap();

        assert_eq!(result.result, "pass");
        assert_eq!(
            fs::canonicalize(fs::read_to_string(pwd_output).unwrap().trim()).unwrap(),
            fs::canonicalize(repo_root).unwrap()
        );
    }

    #[test]
    fn decision_update_edits_fields_and_survives_rebuild() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .add_decision(DecisionAddInput {
                id: "0002-update".to_owned(),
                title: "Original".to_owned(),
                status: "accepted".to_owned(),
                doc_path: None,
                verify_command: Some("prose expectations, not runnable".to_owned()),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();

        // Nothing-to-update guard and missing-id guard.
        assert!(matches!(
            repository.update_decision(DecisionUpdateInput {
                id: "0002-update".to_owned(),
                title: None,
                status: None,
                doc_path: None,
                verify_command: None,
                predicted_impact: None,
                notes: None,
            }),
            Err(HarnessInfraError::EmptyDecisionUpdate)
        ));
        assert!(matches!(
            repository.update_decision(DecisionUpdateInput {
                id: "no-such".to_owned(),
                title: None,
                status: None,
                doc_path: None,
                verify_command: Some(String::new()),
                predicted_impact: None,
                notes: None,
            }),
            Err(HarnessInfraError::DecisionNotFound(_))
        ));

        repository
            .update_decision(DecisionUpdateInput {
                id: "0002-update".to_owned(),
                title: None,
                status: Some("superseded".to_owned()),
                doc_path: None,
                verify_command: Some(String::new()),
                predicted_impact: None,
                notes: Some("expectations moved to notes".to_owned()),
            })
            .unwrap();

        let read_row = |repository: &SqliteHarnessRepository| -> (String, String, String, String) {
            let connection = repository.open_existing().unwrap();
            connection
                .query_row(
                    "SELECT title, status, COALESCE(verify_command,'<null>'), COALESCE(notes,'')
                     FROM decision WHERE id='0002-update';",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .unwrap()
        };
        let live = read_row(&repository);
        assert_eq!(
            live,
            (
                "Original".to_owned(),
                "superseded".to_owned(),
                String::new(),
                "expectations moved to notes".to_owned()
            )
        );

        // The decision.update event must replay identically on rebuild.
        repository.rebuild(None).unwrap();
        assert_eq!(read_row(&repository), live);
    }

    #[test]
    fn story_add_update_and_verify_status_store_verify_command() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .add_story(StoryAddInput {
                id: "US-VERIFY".to_owned(),
                title: "Verify command story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("echo ok".to_owned()),
                notes: None,
            })
            .unwrap();
        assert_eq!(
            repository
                .story_verify_status("US-VERIFY")
                .unwrap()
                .verify_command
                .as_deref(),
            Some("echo ok")
        );

        repository
            .update_story(StoryUpdateInput {
                id: "US-VERIFY".to_owned(),
                status: None,
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: Some("npm test".to_owned()),
            })
            .unwrap();

        assert_eq!(
            repository
                .story_verify_status("US-VERIFY")
                .unwrap()
                .verify_command
                .as_deref(),
            Some("npm test")
        );
    }

    #[test]
    fn story_verify_records_pass_fail_and_missing_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_root).unwrap();
        let schema_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
            .join("scripts/schema");
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            schema_root,
        );
        repository.init().unwrap();

        let pwd_output = repo_root.join("story-verify-pwd.txt");
        let verify_command = if cfg!(windows) {
            "cd > story-verify-pwd.txt".to_owned()
        } else {
            "pwd > story-verify-pwd.txt".to_owned()
        };
        repository
            .add_story(StoryAddInput {
                id: "US-PASS".to_owned(),
                title: "Passing story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some(verify_command),
                notes: None,
            })
            .unwrap();
        let pass = repository.verify_story("US-PASS").unwrap();
        assert_eq!(pass.result, "pass");
        assert_eq!(
            fs::canonicalize(fs::read_to_string(pwd_output).unwrap().trim()).unwrap(),
            fs::canonicalize(repo_root).unwrap()
        );
        assert_eq!(
            repository
                .story_verify_status("US-PASS")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("pass")
        );

        repository
            .add_story(StoryAddInput {
                id: "US-FAIL".to_owned(),
                title: "Failing story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("exit 1".to_owned()),
                notes: None,
            })
            .unwrap();
        let fail = repository.verify_story("US-FAIL").unwrap();
        assert_eq!(fail.result, "fail");
        assert_eq!(
            repository
                .story_verify_status("US-FAIL")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("fail")
        );

        repository
            .add_story(StoryAddInput {
                id: "US-MISSING".to_owned(),
                title: "Missing command story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        assert!(matches!(
            repository.verify_story("US-MISSING"),
            Err(HarnessInfraError::MissingStoryVerifyCommand(id)) if id == "US-MISSING"
        ));
    }

    #[test]
    fn story_verify_all_reports_pass_fail_and_skipped() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        for (id, command) in [
            ("US-PASS", Some("exit 0")),
            ("US-FAIL", Some("exit 1")),
            ("US-SKIP", None),
        ] {
            repository
                .add_story(StoryAddInput {
                    id: id.to_owned(),
                    title: id.to_owned(),
                    risk_lane: RiskLane::Normal,
                    contract_doc: None,
                    verify_command: command.map(str::to_owned),
                    notes: None,
                })
                .unwrap();
        }

        let result = repository.verify_all_stories().unwrap();

        assert_eq!(result.passed(), 1);
        assert_eq!(result.failed(), 1);
        assert_eq!(result.skipped(), 1);
        assert_eq!(
            repository
                .story_verify_status("US-PASS")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("pass")
        );
        assert_eq!(
            repository
                .story_verify_status("US-FAIL")
                .unwrap()
                .last_verified_result
                .as_deref(),
            Some("fail")
        );
    }

    #[test]
    fn tool_registry_register_query_and_remove_work() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .register_tool(ToolRegisterInput {
                name: "deploy-check".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Verify deploy health before release".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: Some("deploy-verification".to_owned()),
                scan_target: None,
            })
            .unwrap();
        assert!(matches!(
            repository.register_tool(ToolRegisterInput {
                name: "deploy-check".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Verify deploy health before release".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: Some("deploy-verification".to_owned()),
                scan_target: None,
            }),
            Err(HarnessInfraError::ToolAlreadyExists(_, _))
        ));

        let verification_tools = repository
            .query_tools(Some("Verification".to_owned()), None)
            .unwrap();
        assert!(verification_tools
            .iter()
            .any(|tool| tool.name == "deploy-check" && tool.source == "registered"));

        // Capability lookup returns the registered provider.
        let by_capability = repository
            .query_tools(None, Some("deploy-verification".to_owned()))
            .unwrap();
        assert!(by_capability.iter().any(|tool| tool.name == "deploy-check"));

        repository.remove_tool("deploy-check").unwrap();
        assert!(!repository
            .query_tools(None, None)
            .unwrap()
            .iter()
            .any(|tool| tool.name == "deploy-check"));
    }

    #[test]
    fn tool_check_scans_and_persists_status_per_kind() {
        let (temp_dir, repository) = test_repository();
        repository.init().unwrap();

        // Absolute scan targets keep the test hermetic: test_repository's
        // repo_root points at the real project, so relative targets would
        // resolve against the checkout rather than the temp dir.
        let present_target = temp_dir.path().join("skill-present");
        std::fs::create_dir_all(&present_target).unwrap();
        let missing_target = temp_dir.path().join("mcp-missing");

        // An mcp tool whose scan target does not exist -> missing.
        repository
            .register_tool(ToolRegisterInput {
                name: "mcp-example".to_owned(),
                command: "mcp:example-server".to_owned(),
                description: "Example MCP-backed provider".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: false,
                kind: "mcp".to_owned(),
                capability: Some("impact-analysis".to_owned()),
                scan_target: Some(missing_target.to_string_lossy().into_owned()),
            })
            .unwrap();

        // A skill tool whose scan target exists -> present.
        repository
            .register_tool(ToolRegisterInput {
                name: "skill-example".to_owned(),
                command: "skill:example-skill".to_owned(),
                description: "Example skill-backed provider".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: false,
                kind: "skill".to_owned(),
                capability: Some("impact-analysis".to_owned()),
                scan_target: Some(present_target.to_string_lossy().into_owned()),
            })
            .unwrap();

        let results = repository.check_tools(None).unwrap();
        let mcp_tool = results.iter().find(|r| r.name == "mcp-example").unwrap();
        let skill_tool = results.iter().find(|r| r.name == "skill-example").unwrap();
        assert_eq!(mcp_tool.status, "missing");
        assert_eq!(skill_tool.status, "present");

        // Status is persisted, not just returned.
        let stored = repository
            .query_tools(None, Some("impact-analysis".to_owned()))
            .unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored
            .iter()
            .all(|tool| tool.checked_at.as_deref().is_some_and(|v| !v.is_empty())));
        assert_eq!(
            stored
                .iter()
                .find(|t| t.name == "skill-example")
                .unwrap()
                .status,
            "present"
        );
    }

    #[test]
    fn interventions_can_be_added_and_filtered() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        repository
            .add_story(StoryAddInput {
                id: "US-I".to_owned(),
                title: "Intervention story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        let trace_id = repository
            .record_trace(TraceInput {
                task_summary: "Trace for intervention".to_owned(),
                intake_id: None,
                story_id: Some("US-I".to_owned()),
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("none".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .add_intervention(InterventionAddInput {
                trace_id: Some(trace_id.clone()),
                story_id: Some("US-I".to_owned()),
                intervention_type: "correction".to_owned(),
                description: "Use error handling instead of unwrap".to_owned(),
                source: "human".to_owned(),
                impact: Some("Reduced panic risk".to_owned()),
            })
            .unwrap();

        assert_eq!(
            repository
                .query_interventions(InterventionFilter {
                    trace_id: Some(trace_id.clone()),
                    story_id: None,
                    intervention_type: None,
                })
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            repository
                .query_interventions(InterventionFilter {
                    trace_id: None,
                    story_id: Some("US-I".to_owned()),
                    intervention_type: Some("override".to_owned()),
                })
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn audit_detects_drift_and_propose_can_commit_backlog_items() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        repository
            .add_story(StoryAddInput {
                id: "US-AUDIT".to_owned(),
                title: "Audit story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: Some("exit 0".to_owned()),
                notes: None,
            })
            .unwrap();
        repository
            .update_story(StoryUpdateInput {
                id: "US-AUDIT".to_owned(),
                status: Some("in_progress".to_owned()),
                evidence: None,
                unit: None,
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();
        repository
            .add_backlog(BacklogAddInput {
                title: "Implemented without outcome".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Tiny),
                predicted_impact: Some("Expected improvement".to_owned()),
                notes: None,
            })
            .unwrap();
        let backlog_id = repository
            .query_backlog(BacklogFilter::All)
            .unwrap()
            .last()
            .expect("backlog row")
            .id
            .clone();
        repository
            .close_backlog(BacklogCloseInput {
                id: backlog_id,
                status: "implemented".to_owned(),
                actual_outcome: None,
            })
            .unwrap();
        repository
            .register_tool(ToolRegisterInput {
                name: "missing-tool".to_owned(),
                command: "definitely-missing-tool".to_owned(),
                description: "Missing command for audit coverage".to_owned(),
                responsibility: "Verification".to_owned(),
                args: Vec::new(),
                force: true,
                kind: "cli".to_owned(),
                capability: None,
                scan_target: None,
            })
            .unwrap();
        for _ in 0..2 {
            repository
                .record_trace(TraceInput {
                    task_summary: "Repeated friction trace".to_owned(),
                    intake_id: None,
                    story_id: None,
                    agent: Some("codex".to_owned()),
                    outcome: Some("completed".to_owned()),
                    duration_seconds: None,
                    token_estimate: None,
                    friction: Some("Context rules missed schema decision".to_owned()),
                    notes: None,
                    actions: CsvList::from_optional(Some("read".to_owned())),
                    files_read: CsvList::from_optional(Some("docs/HARNESS.md".to_owned())),
                    files_changed: CsvList::from_optional(Some(
                        "scripts/schema/003-tool-registry.sql".to_owned(),
                    )),
                    decisions: CsvList::from_optional(None),
                    errors: CsvList::from_optional(None),
                })
                .unwrap();
        }

        let audit = repository.audit().unwrap();
        assert_eq!(audit.orphaned_stories.len(), 1);
        assert_eq!(audit.unverified_stories.len(), 1);
        assert_eq!(audit.backlog_without_outcomes.len(), 1);
        assert_eq!(audit.broken_tools.len(), 1);
        assert!(audit.entropy_score() > 0);

        let proposals = repository.propose(true).unwrap();
        assert!(proposals.iter().any(|proposal| proposal
            .evidence
            .contains("Context rules missed schema decision")));
        assert!(proposals
            .iter()
            .all(|proposal| proposal.committed_backlog_id.is_some()));
        assert!(repository.query_backlog(BacklogFilter::Open).unwrap().len() >= 1);
    }

    #[test]
    fn propose_ignores_duplicate_signals_on_one_story() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        for id in ["US-1", "US-2"] {
            repository
                .add_story(StoryAddInput {
                    id: id.to_owned(),
                    title: format!("story {id}"),
                    risk_lane: RiskLane::Normal,
                    contract_doc: None,
                    verify_command: None,
                    notes: None,
                })
                .unwrap();
        }

        for _ in 0..2 {
            repository
                .add_story_signal(StorySignalAddInput {
                    story_id: Some("US-1".to_owned()),
                    trace_id: None,
                    signal_type: "deviation".to_owned(),
                    summary: "spec silent on empty name".to_owned(),
                    component: None,
                    notes: None,
                })
                .unwrap();
        }

        // Same story twice is noise, not recurrence.
        assert!(!repository
            .propose(false)
            .unwrap()
            .iter()
            .any(|proposal| proposal.title.starts_with("Recurring deviation")));

        repository
            .add_story_signal(StorySignalAddInput {
                story_id: Some("US-2".to_owned()),
                trace_id: None,
                signal_type: "deviation".to_owned(),
                summary: "spec silent on empty name".to_owned(),
                component: None,
                notes: None,
            })
            .unwrap();

        // A second distinct story crosses the threshold.
        assert!(repository
            .propose(false)
            .unwrap()
            .iter()
            .any(|proposal| proposal.title.starts_with("Recurring deviation")));
    }

    #[test]
    fn propose_mines_recurring_story_signals_only_when_they_repeat() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        let add_signal = |summary: &str| {
            repository
                .add_story_signal(StorySignalAddInput {
                    story_id: None,
                    trace_id: None,
                    signal_type: "deviation".to_owned(),
                    summary: summary.to_owned(),
                    component: None,
                    notes: None,
                })
                .unwrap();
        };

        // A single, non-recurring signal must not produce a proposal.
        add_signal("plan omitted the migration step");
        assert!(!repository
            .propose(false)
            .unwrap()
            .iter()
            .any(|proposal| proposal.title.starts_with("Recurring deviation")));

        // A second matching signal crosses the >= 2 recurrence threshold.
        add_signal("plan omitted the migration step");
        let proposals = repository.propose(false).unwrap();
        let recurring = proposals
            .iter()
            .find(|proposal| proposal.title.starts_with("Recurring deviation"))
            .expect("recurring story signal should yield a proposal");
        assert_eq!(recurring.component, "Task specification");
        assert!(recurring.evidence.contains("2 stories"));
        assert_eq!(recurring.confidence, "medium");

        // Both signals are queryable, and the type filter works.
        assert_eq!(
            repository
                .query_story_signals(StorySignalFilter {
                    story_id: None,
                    signal_type: Some("deviation".to_owned()),
                })
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn story_backlog_trace_and_queries_work() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        repository
            .add_story(StoryAddInput {
                id: "US-T".to_owned(),
                title: "Test story".to_owned(),
                risk_lane: RiskLane::Normal,
                contract_doc: None,
                verify_command: None,
                notes: None,
            })
            .unwrap();
        repository
            .update_story(StoryUpdateInput {
                id: "US-T".to_owned(),
                status: Some("implemented".to_owned()),
                evidence: Some("unit test".to_owned()),
                unit: Some(BoolFlag(1)),
                integration: None,
                e2e: None,
                platform: None,
                verify_command: None,
            })
            .unwrap();
        assert_eq!(repository.query_matrix().unwrap()[0].unit, 1);

        let backlog_id = repository
            .add_backlog(BacklogAddInput {
                title: "Improve CLI".to_owned(),
                discovered_while: None,
                current_pain: Some("manual SQL".to_owned()),
                suggestion: None,
                risk: Some(RiskLane::HighRisk),
                predicted_impact: None,
                notes: None,
            })
            .unwrap();
        repository
            .close_backlog(BacklogCloseInput {
                id: backlog_id,
                status: "implemented".to_owned(),
                actual_outcome: Some("done".to_owned()),
            })
            .unwrap();
        assert_eq!(
            repository.query_backlog(BacklogFilter::All).unwrap()[0]
                .actual_outcome
                .as_deref(),
            Some("done")
        );

        let trace_id = repository
            .record_trace(TraceInput {
                task_summary: "Test trace".to_owned(),
                intake_id: None,
                story_id: Some("US-T".to_owned()),
                agent: Some("test".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("none".to_owned()),
                notes: None,
                actions: CsvList::from_optional(Some("one,two".to_owned())),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        assert_eq!(trace_id.len(), 26);
        assert_eq!(
            repository.query_traces().unwrap()[0].task_summary,
            "Test trace"
        );
        assert_eq!(
            repository.query_friction().unwrap()[0].harness_friction,
            "none"
        );
    }

    #[test]
    fn friction_query_includes_intake_context_and_filters_null_friction() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let intake_id = repository
            .record_intake(IntakeInput {
                input_type: InputType::ChangeRequest,
                summary: "Friction query context".to_owned(),
                risk_lane: RiskLane::Normal,
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Trace without friction".to_owned(),
                intake_id: Some(intake_id.clone()),
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Trace with linked friction".to_owned(),
                intake_id: Some(intake_id.clone()),
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("Linked friction".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Trace with unlinked friction".to_owned(),
                intake_id: None,
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("Unlinked friction".to_owned()),
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();

        let friction = repository.query_friction().unwrap();

        assert_eq!(friction.len(), 2);
        assert_eq!(friction[0].risk_lane, None);
        assert_eq!(friction[0].input_type, None);
        assert_eq!(friction[1].risk_lane.as_deref(), Some("normal"));
        assert_eq!(friction[1].input_type.as_deref(), Some("change_request"));
    }

    #[test]
    fn import_brownfield_seeds_markdown_state_idempotently() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_root = temp_dir.path().join("repo");
        fs::create_dir_all(repo_root.join("docs/decisions")).unwrap();
        fs::write(
            repo_root.join("docs/TEST_MATRIX.md"),
            r#"# Test Matrix

| Story | Contract | Unit | Integration | E2E | Platform | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US-010 | docs/product/tasks.md | yes | pending | no | mac smoke | implemented | cargo test |
"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("docs/decisions/0007-test-decision.md"),
            r#"# Test Decision

## Status

Accepted
"#,
        )
        .unwrap();
        fs::write(
            repo_root.join("docs/HARNESS_BACKLOG.md"),
            r#"# Harness Backlog

## Items

### Title

Import existing docs

### Discovered While

Testing brownfield import

### Current Pain

Existing Harness v0 repos have markdown truth.

### Suggested Improvement

Seed the durable database.

### Risk

normal

### Status

accepted

### Title

Keep installer checksum

### Discovered While

Testing release install

### Current Pain

Downloads need verification.

### Suggested Improvement

Verify sha256 files.

### Risk

high-risk

### Status

implemented
"#,
        )
        .unwrap();

        let source_repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let repository = SqliteHarnessRepository::new(
            repo_root.clone(),
            temp_dir.path().join("harness.db"),
            source_repo_root.join("scripts/schema"),
        );
        repository.init().unwrap();

        let first = repository.import_brownfield().unwrap();

        assert_eq!(
            first,
            BrownfieldImportResult {
                stories: 1,
                decisions: 1,
                backlog_items: 2,
            }
        );
        // Decision 0008 Q4: brownfield is a one-time seed. The seed itself
        // ran migrate-to-events, so a re-run refuses instead of re-importing.
        assert!(matches!(
            repository.import_brownfield().unwrap_err(),
            HarnessInfraError::BrownfieldOnEventBacked
        ));

        let matrix = repository.query_matrix().unwrap();
        assert_eq!(matrix[0].id, "US-010");
        assert_eq!(matrix[0].title, "docs/product/tasks.md");
        assert_eq!(matrix[0].status, "implemented");
        assert_eq!(matrix[0].unit, 1);
        assert_eq!(matrix[0].integration, 0);
        assert_eq!(matrix[0].platform, 1);

        let decisions = repository.query_decisions().unwrap();
        assert_eq!(decisions[0].id, "0007-test-decision");
        assert_eq!(decisions[0].status, "accepted");

        let backlog = repository.query_backlog(BacklogFilter::All).unwrap();
        assert_eq!(backlog.len(), 2);
        assert!(backlog
            .iter()
            .any(|item| item.title == "Import existing docs"
                && item.status == "accepted"
                && item.risk.as_deref() == Some("normal")));
        assert!(backlog
            .iter()
            .any(|item| item.title == "Keep installer checksum"
                && item.status == "implemented"
                && item.risk.as_deref() == Some("high_risk")));
    }

    #[test]
    fn filters_open_and_closed_backlog_items() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();

        let proposed_id = repository
            .add_backlog(BacklogAddInput {
                title: "Proposed item".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Tiny),
                predicted_impact: Some("Should improve trace review.".to_owned()),
                notes: None,
            })
            .unwrap();
        let implemented_id = repository
            .add_backlog(BacklogAddInput {
                title: "Implemented item".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: Some(RiskLane::Normal),
                predicted_impact: Some("Should reduce missing proof.".to_owned()),
                notes: None,
            })
            .unwrap();
        repository
            .close_backlog(BacklogCloseInput {
                id: implemented_id.clone(),
                status: "implemented".to_owned(),
                actual_outcome: Some("Proof gaps were found earlier.".to_owned()),
            })
            .unwrap();

        let all = repository.query_backlog(BacklogFilter::All).unwrap();
        let open = repository.query_backlog(BacklogFilter::Open).unwrap();
        let closed = repository.query_backlog(BacklogFilter::Closed).unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, proposed_id);
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].id, implemented_id);
        assert_eq!(
            closed[0].actual_outcome.as_deref(),
            Some("Proof gaps were found earlier.")
        );
    }

    #[test]
    fn scores_latest_and_specific_trace_with_lane_lookup() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let intake_id = repository
            .record_intake(IntakeInput {
                input_type: InputType::HarnessImprovement,
                summary: "High risk trace quality test".to_owned(),
                risk_lane: RiskLane::HighRisk,
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();
        let first_trace = repository
            .record_trace(TraceInput {
                task_summary: "Minimal trace test".to_owned(),
                intake_id: None,
                story_id: None,
                agent: None,
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: None,
                notes: None,
                actions: CsvList::from_optional(None),
                files_read: CsvList::from_optional(None),
                files_changed: CsvList::from_optional(None),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();
        repository
            .record_trace(TraceInput {
                task_summary: "Standard trace linked to high risk intake".to_owned(),
                intake_id: Some(intake_id),
                story_id: None,
                agent: Some("codex".to_owned()),
                outcome: Some("completed".to_owned()),
                duration_seconds: None,
                token_estimate: None,
                friction: Some("none".to_owned()),
                notes: None,
                actions: CsvList::from_optional(Some("read,patched".to_owned())),
                files_read: CsvList::from_optional(Some("PHASE3.md".to_owned())),
                files_changed: CsvList::from_optional(Some(
                    "crates/harness-cli/src/domain.rs".to_owned(),
                )),
                decisions: CsvList::from_optional(None),
                errors: CsvList::from_optional(None),
            })
            .unwrap();

        let latest = repository.score_trace(None).unwrap();
        assert_eq!(latest.achieved, TraceQualityTier::Standard);
        assert_eq!(latest.required, Some(TraceQualityTier::Detailed));
        assert!(!latest.meets_requirement);
        assert!(latest
            .missing_detailed
            .iter()
            .any(|field| field.starts_with("decisions_made")));

        let specific = repository.score_trace(Some(first_trace.clone())).unwrap();
        assert_eq!(specific.trace_id, first_trace);
        assert_eq!(specific.achieved, TraceQualityTier::Minimal);
        assert_eq!(specific.required, None);
        assert!(specific.meets_requirement);
    }

    #[test]
    fn info_reports_absence_on_uninitialized_repo() {
        // Acceptance (US-036): reports absence rather than erroring, exit 0.
        let (_temp_dir, repository) = test_repository();
        let report = repository.info().unwrap();

        assert!(!report.initialized);
        assert!(!report.cli_version.is_empty());
        assert_eq!(report.supported_schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(report.available_schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(
            report.event_format_version,
            crate::events::EVENT_SCHEMA_VERSION
        );
        assert_eq!(report.applied_schema_version, 0);
        assert!(report.applied_migrations.is_empty());
        assert!(!report.event_backed);
        assert_eq!(report.event_files, 0);
        assert!(!report.schema_behind_cli);
        assert!(!report.cache_behind_log);
    }

    #[test]
    fn info_reports_initialized_state() {
        let (_temp_dir, repository) = test_repository();
        repository.init().unwrap();
        let report = repository.info().unwrap();

        assert!(report.initialized);
        assert_eq!(report.applied_schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(
            report.applied_migrations,
            (1..=SUPPORTED_SCHEMA_VERSION).collect::<Vec<_>>()
        );
        assert!(report.event_backed);
        assert!(!report.schema_behind_cli);
        assert!(!report.cache_behind_log);
    }

    #[test]
    fn info_flags_schema_behind_cli() {
        // Init against a truncated schema dir (migrations 001..007 only), then
        // read info through a repository that sees the full schema on disk:
        // the applied schema is behind the CLI and needs `migrate`.
        let temp_dir = tempfile::tempdir().unwrap();
        let partial_schema = temp_dir.path().join("partial-schema");
        fs::create_dir_all(&partial_schema).unwrap();
        for (version, path) in SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("harness.db"),
            real_repo_root().join("scripts/schema"),
        )
        .migration_files()
        .unwrap()
        {
            if version <= SUPPORTED_SCHEMA_VERSION - 1 {
                let name = path.file_name().unwrap();
                fs::copy(&path, partial_schema.join(name)).unwrap();
            }
        }

        let db_path = temp_dir.path().join("harness.db");
        let partial = SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            db_path.clone(),
            partial_schema,
        );
        partial.init().unwrap();

        let full = SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            db_path,
            real_repo_root().join("scripts/schema"),
        );
        let report = full.info().unwrap();

        assert!(report.initialized);
        assert_eq!(report.applied_schema_version, SUPPORTED_SCHEMA_VERSION - 1);
        assert_eq!(report.available_schema_version, SUPPORTED_SCHEMA_VERSION);
        assert!(report.schema_behind_cli);
        assert!(!report.cache_behind_log);
    }

    #[test]
    fn info_flags_cache_behind_log() {
        // Simulate the crash window: an event appended to the log whose
        // watermark the cache never advanced. A read command would replay it;
        // info reports it without mutating anything.
        let (temp_dir, service) = events_test_service();
        service
            .record_intake(IntakeInput {
                input_type: InputType::from_str("maintenance").unwrap(),
                summary: "seed".to_owned(),
                risk_lane: RiskLane::from_str("tiny").unwrap(),
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();

        let events_dir = temp_dir.path().join(".harness/events");
        let writer_file = events_dir.join(format!("{}.jsonl", own_writer_name(&events_dir)));
        let mut existing = fs::read_to_string(&writer_file).unwrap();
        existing.push_str("{\"event_id\":\"z\",\"writer\":\"x\",\"recorded_at\":\"t\",\"op\":\"intake.record\",\"schema\":1,\"payload\":{}}\n");
        fs::write(&writer_file, existing).unwrap();

        let repository = SqliteHarnessRepository::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("harness.db"),
            real_repo_root().join("scripts/schema"),
        );
        let report = repository.info().unwrap();

        assert!(report.initialized);
        assert!(report.event_backed);
        assert!(report.cache_behind_log);
        assert!(!report.schema_behind_cli);
    }
}
