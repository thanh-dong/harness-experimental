//! Shadow-mode event log (US-028a).
//!
//! Every durable CLI mutation appends one event to a per-writer, append-only
//! JSONL file under `.harness/events/`, alongside the authoritative SQLite
//! write. Shadow guarantee: an append failure warns on stderr and never fails
//! the command — `harness.db` remains the write of record until US-028b.

use std::collections::hash_map::RandomState;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

pub const EVENT_SCHEMA_VERSION: i64 = 1;
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

// (last ms, last 80-bit randomness as two words) for same-ms monotonicity.
// Process-global so row ids (US-028b) and event ids share one monotonic
// stream per writer process.
static LAST_ULID: Mutex<Option<(u64, u64, u16)>> = Mutex::new(None);

/// Mint a ULID: 48-bit ms timestamp + 80-bit randomness, monotonic within
/// this process for same-millisecond calls.
pub fn mint_ulid() -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let mut guard = LAST_ULID.lock().expect("ulid lock");
    let (ms, hi, lo) = match *guard {
        Some((last_ms, last_hi, last_lo)) if now_ms <= last_ms => {
            let (lo, carry) = last_lo.overflowing_add(1);
            (last_ms, last_hi.wrapping_add(u64::from(carry)), lo)
        }
        _ => {
            let (hi, lo) = random_words();
            (now_ms, hi, lo)
        }
    };
    *guard = Some((ms, hi, lo));
    encode_ulid(ms, hi, lo)
}

#[derive(Debug)]
pub struct EventLog {
    events_dir: PathBuf,
    writer: String,
}

impl EventLog {
    pub fn new(repo_root: &Path) -> Self {
        let writer = resolve_writer(repo_root);
        Self::with_writer(repo_root.join(".harness/events"), writer)
    }

    pub fn with_writer(events_dir: PathBuf, writer: String) -> Self {
        Self { events_dir, writer }
    }

    pub fn writer(&self) -> &str {
        &self.writer
    }

    /// This writer's log file name inside the events dir.
    pub fn file_name(&self) -> String {
        format!("{}.jsonl", self.writer)
    }

    pub fn own_file_path(&self) -> PathBuf {
        self.events_dir.join(self.file_name())
    }

    /// Append one raw line (newline added) to this writer's file, fsync'd.
    pub fn append_line(&self, line: &str) -> std::io::Result<()> {
        fs::create_dir_all(&self.events_dir)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.own_file_path())?;
        let mut owned = line.to_owned();
        owned.push('\n');
        file.write_all(owned.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }

    /// Serialize an event to its canonical log line (sorted keys).
    /// `observed` is the writer's per-writer consumed-count snapshot at
    /// append time — the causal-audit signal (DKR-4); optional for
    /// compatibility with US-028a shadow events and genesis events.
    pub fn event_line(
        event_id: &str,
        writer: &str,
        recorded_at: &str,
        op: &str,
        payload: &Value,
        observed: Option<&Value>,
    ) -> String {
        let mut event = json!({
            "event_id": event_id,
            "writer": writer,
            "recorded_at": recorded_at,
            "op": op,
            "schema": EVENT_SCHEMA_VERSION,
            "payload": payload,
        });
        if let Some(observed) = observed {
            event["observed"] = observed.clone();
        }
        serde_json::to_string(&event).expect("event serializes")
    }

    /// Append one event (US-028b: failures are the caller's to handle — the
    /// event log is the write of record). Production writes go through the
    /// repository's append_and_apply; tests use this to simulate teammates.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn emit(&self, op: &str, payload: Value) -> std::io::Result<String> {
        let event_id = mint_ulid();
        let line = Self::event_line(
            &event_id,
            &self.writer,
            &rfc3339_utc_now(),
            op,
            &payload,
            None,
        );
        self.append_line(&line)?;
        Ok(event_id)
    }
}

/// Deterministic ULID for genesis events (US-028b): time part from the row's
/// original timestamp, "random" part hashed from (table, id). Re-running
/// migration on the same input reproduces byte-identical events, and this can
/// never collide with live ULIDs minted by real writers at the same ms except
/// with fnv-collision probability.
pub fn genesis_ulid(unix_ms: u64, table: &str, row_id: &str) -> String {
    let hi = fnv1a64(format!("{table}:{row_id}").as_bytes());
    let lo = (fnv1a64(format!("{row_id}:{table}").as_bytes()) & 0xFFFF) as u16;
    encode_ulid(unix_ms, hi, lo)
}

/// 48-bit ms timestamp + 80-bit randomness, Crockford base32, 26 chars.
fn encode_ulid(ms: u64, rand_hi: u64, rand_lo: u16) -> String {
    // Assemble the 128-bit value: 48-bit time | 64-bit hi | 16-bit lo.
    let value = (u128::from(ms & 0xFFFF_FFFF_FFFF) << 80)
        | (u128::from(rand_hi) << 16)
        | u128::from(rand_lo);
    let mut out = [0u8; 26];
    for (index, slot) in out.iter_mut().enumerate() {
        let shift = 125 - (5 * index as u32);
        *slot = CROCKFORD[((value >> shift) & 0x1F) as usize];
    }
    String::from_utf8(out.to_vec()).expect("crockford is ascii")
}

fn random_words() -> (u64, u16) {
    // std-only entropy: RandomState is seeded from the OS per process; mixing
    // the current nanos keeps successive calls distinct. Uniqueness — not
    // unpredictability — is what event ids need (see implementation notes).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or(0);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u32(nanos);
    hasher.write_u32(std::process::id());
    let hi = hasher.finish();
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(hi);
    let lo = (hasher.finish() & 0xFFFF) as u16;
    (hi, lo)
}

/// FNV-1a 64 — deterministic across processes and machines, unlike SipHash.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    fnv1a64_continue(0xcbf2_9ce4_8422_2325, bytes)
}

/// Streaming FNV-1a: continue a running hash with more bytes. Lets watermark
/// hashes advance per appended line without re-reading the whole file.
pub fn fnv1a64_continue(previous: u64, bytes: &[u8]) -> u64 {
    let mut hash = previous;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Writer identity (decision 0008 Q3): `HARNESS_WRITER` override, else
/// email-hash short form plus a per-clone disambiguator stored untracked in
/// the git dir, so one human on two machines never shares a writer file.
fn resolve_writer(repo_root: &Path) -> String {
    if let Ok(writer) = std::env::var("HARNESS_WRITER") {
        let sanitized = sanitize_writer(&writer);
        if !sanitized.is_empty() {
            return sanitized;
        }
    }

    let email = git_user_email(repo_root).unwrap_or_else(|| "unknown@local".to_owned());
    let email_hash = format!("{:016x}", fnv1a64(email.trim().to_lowercase().as_bytes()));
    let prefix = &email_hash[..8];

    match clone_disambiguator(repo_root) {
        Some(suffix) => format!("{prefix}-{suffix}"),
        None => prefix.to_owned(),
    }
}

fn sanitize_writer(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(64)
        .collect()
}

fn git_user_email(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "user.email"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let email = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

fn clone_disambiguator(repo_root: &Path) -> Option<String> {
    let git_dir = resolve_git_dir(repo_root)?;
    let marker = git_dir.join("harness-writer");
    if let Ok(existing) = fs::read_to_string(&marker) {
        let existing = sanitize_writer(existing.trim());
        if !existing.is_empty() {
            return Some(existing);
        }
    }
    let (hi, lo) = random_words();
    let suffix = format!("{:04x}", (hi ^ u64::from(lo)) & 0xFFFF);
    fs::write(&marker, &suffix).ok()?;
    Some(suffix)
}

fn resolve_git_dir(repo_root: &Path) -> Option<PathBuf> {
    let dot_git = repo_root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    // Worktrees: .git is a file "gitdir: <path>".
    if dot_git.is_file() {
        let content = fs::read_to_string(&dot_git).ok()?;
        let target = content.strip_prefix("gitdir:")?.trim();
        let path = PathBuf::from(target);
        let resolved = if path.is_absolute() {
            path
        } else {
            repo_root.join(path)
        };
        if resolved.is_dir() {
            return Some(resolved);
        }
    }
    None
}

pub fn rfc3339_utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    rfc3339_from_unix(seconds as i64)
}

/// Civil-from-days (Howard Hinnant's algorithm); std has no calendar.
pub fn rfc3339_from_unix(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let secs_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    )
}

/// Inverse of `rfc3339_from_unix` for SQLite `datetime('now')` strings
/// ("YYYY-MM-DD HH:MM:SS", UTC). Returns unix seconds.
pub fn unix_from_sqlite_datetime(value: &str) -> Option<i64> {
    let value = value.trim();
    let (date, time) = value.split_once(' ').or_else(|| value.split_once('T'))?;
    let mut date_parts = date.splitn(3, '-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;
    let time = time.trim_end_matches('Z');
    let mut time_parts = time.splitn(3, ':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: i64 = time_parts.next()?.split('.').next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ulid_is_26_crockford_chars() {
        let ulid = mint_ulid();
        assert_eq!(ulid.len(), 26);
        assert!(ulid.bytes().all(|byte| CROCKFORD.contains(&byte)));
    }

    #[test]
    fn ulids_are_monotonic_within_a_process() {
        let mut previous = mint_ulid();
        for _ in 0..1_000 {
            let next = mint_ulid();
            assert!(next > previous, "{next} !> {previous}");
            previous = next;
        }
    }

    #[test]
    fn ulid_orders_by_timestamp_across_milliseconds() {
        let earlier = encode_ulid(1_000, u64::MAX, u16::MAX);
        let later = encode_ulid(1_001, 0, 0);
        assert!(later > earlier);
    }

    #[test]
    fn sqlite_datetime_roundtrips_through_unix() {
        let unix = unix_from_sqlite_datetime("2026-07-02 04:00:00").unwrap();
        assert_eq!(rfc3339_from_unix(unix), "2026-07-02T04:00:00Z");
        assert_eq!(unix_from_sqlite_datetime("1970-01-01 00:00:00"), Some(0));
    }

    #[test]
    fn genesis_ulids_are_deterministic_and_time_ordered() {
        let a1 = genesis_ulid(1_000, "intake", "1");
        let a2 = genesis_ulid(1_000, "intake", "1");
        let b = genesis_ulid(1_000, "intake", "2");
        let later = genesis_ulid(2_000, "intake", "1");
        assert_eq!(a1, a2);
        assert_ne!(a1, b);
        assert!(later > a1);
        assert_eq!(a1.len(), 26);
    }

    #[test]
    fn rfc3339_matches_known_instant() {
        // 2026-07-02T04:00:00Z
        assert_eq!(rfc3339_from_unix(1_782_964_800), "2026-07-02T04:00:00Z");
        assert_eq!(rfc3339_from_unix(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn emit_appends_parseable_event_line() {
        let temp = tempfile::tempdir().unwrap();
        let log = EventLog::with_writer(temp.path().join("events"), "testwriter".into());
        log.emit("story.add", json!({"id": "US-1", "title": "t"}))
            .unwrap();
        log.emit(
            "story.update",
            json!({"id": "US-1", "status": "implemented"}),
        )
        .unwrap();

        let content = fs::read_to_string(temp.path().join("events/testwriter.jsonl")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        let event: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(event["op"], "story.add");
        assert_eq!(event["writer"], "testwriter");
        assert_eq!(event["schema"], 1);
        assert_eq!(event["payload"]["id"], "US-1");
        assert_eq!(event["event_id"].as_str().unwrap().len(), 26);
        assert!(event["recorded_at"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn emit_fails_on_unwritable_dir() {
        // A file where the events dir should be makes create_dir_all fail —
        // post-cutover the caller must surface this, not swallow it.
        let temp = tempfile::tempdir().unwrap();
        let blocker = temp.path().join("events");
        fs::write(&blocker, "not a directory").unwrap();
        let log = EventLog::with_writer(blocker, "testwriter".into());
        assert!(log.emit("story.add", json!({"id": "US-1"})).is_err());
    }

    #[test]
    fn fnv_continue_matches_whole_input() {
        let whole = fnv1a64(b"hello world");
        let split = fnv1a64_continue(fnv1a64(b"hello "), b"world");
        assert_eq!(whole, split);
    }

    #[test]
    fn harness_writer_override_wins_and_is_sanitized() {
        std::env::set_var("HARNESS_WRITER", "agent/one!x");
        let writer = resolve_writer(Path::new("/nonexistent"));
        std::env::remove_var("HARNESS_WRITER");
        assert_eq!(writer, "agentonex");
    }

    #[test]
    fn fnv1a64_is_stable() {
        // Pinned so writer prefixes never drift across releases.
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(format!("{:016x}", fnv1a64(b"a@b.c")), "92cc491485c90b75");
    }
}
