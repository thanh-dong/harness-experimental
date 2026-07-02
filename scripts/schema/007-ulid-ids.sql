-- Harness v0 schema - migration 007 (US-028b cutover, stage 1)
-- Integer auto-increment row ids become TEXT so writers can mint ULIDs that
-- never collide across parallel clones (decision 0008; parent spec US-028).
-- Existing integer ids survive as their decimal strings, which is what lets
-- migrate-to-events preserve original ids in payload.id losslessly.
-- Also adds the cutover cache tables: event watermarks, the event-backed
-- flag, and the causal LWW audit surface. These are cache artifacts —
-- rebuilt from the log, never logged, never part of durable dumps.

PRAGMA foreign_keys = OFF;

CREATE TABLE intake_new (
    id            TEXT NOT NULL PRIMARY KEY,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    input_type    TEXT    NOT NULL
                         CHECK(input_type IN (
                           'new_spec','spec_slice','change_request',
                           'new_initiative','maintenance','harness_improvement'
                         )),
    summary       TEXT    NOT NULL,
    risk_lane     TEXT    NOT NULL
                         CHECK(risk_lane IN ('tiny','normal','high_risk')),
    risk_flags    TEXT,
    affected_docs TEXT,
    story_id      TEXT,
    notes         TEXT
);
INSERT INTO intake_new
    SELECT CAST(id AS TEXT), created_at, input_type, summary, risk_lane,
           risk_flags, affected_docs, story_id, notes
    FROM intake;
DROP TABLE intake;
ALTER TABLE intake_new RENAME TO intake;

CREATE TABLE backlog_new (
    id                    TEXT NOT NULL PRIMARY KEY,
    created_at            TEXT    NOT NULL DEFAULT (datetime('now')),
    title                 TEXT    NOT NULL,
    discovered_while      TEXT,
    current_pain          TEXT,
    suggested_improvement TEXT,
    risk                  TEXT    CHECK(risk IN ('tiny','normal','high_risk')),
    status                TEXT    NOT NULL DEFAULT 'proposed'
                          CHECK(status IN (
                            'proposed','accepted','implemented','rejected'
                          )),
    predicted_impact      TEXT,
    actual_outcome        TEXT,
    implemented_at        TEXT,
    notes                 TEXT
);
INSERT INTO backlog_new
    SELECT CAST(id AS TEXT), created_at, title, discovered_while, current_pain,
           suggested_improvement, risk, status, predicted_impact,
           actual_outcome, implemented_at, notes
    FROM backlog;
DROP TABLE backlog;
ALTER TABLE backlog_new RENAME TO backlog;

CREATE TABLE trace_new (
    id              TEXT NOT NULL PRIMARY KEY,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    task_summary    TEXT    NOT NULL,
    intake_id       TEXT    REFERENCES intake(id),
    story_id        TEXT    REFERENCES story(id),
    agent           TEXT,
    actions_taken   TEXT,
    files_read      TEXT,
    files_changed   TEXT,
    decisions_made  TEXT,
    errors          TEXT,
    outcome         TEXT
                    CHECK(outcome IN (
                      'completed','blocked','partial','failed'
                    )),
    duration_seconds INTEGER,
    token_estimate   INTEGER,
    harness_friction TEXT,
    notes            TEXT
);
INSERT INTO trace_new
    SELECT CAST(id AS TEXT), created_at, task_summary, CAST(intake_id AS TEXT),
           story_id, agent, actions_taken, files_read, files_changed,
           decisions_made, errors, outcome, duration_seconds, token_estimate,
           harness_friction, notes
    FROM trace;
DROP TABLE trace;
ALTER TABLE trace_new RENAME TO trace;

CREATE TABLE intervention_new (
    id          TEXT NOT NULL PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    trace_id    TEXT REFERENCES trace(id),
    story_id    TEXT,
    type        TEXT NOT NULL CHECK(type IN ('correction','override','escalation','approval')),
    description TEXT NOT NULL,
    source      TEXT NOT NULL CHECK(source IN ('human','reviewer','ci','agent')),
    impact      TEXT
);
INSERT INTO intervention_new
    SELECT CAST(id AS TEXT), created_at, CAST(trace_id AS TEXT), story_id,
           type, description, source, impact
    FROM intervention;
DROP TABLE intervention;
ALTER TABLE intervention_new RENAME TO intervention;

CREATE TABLE story_signal_new (
    id          TEXT NOT NULL PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    story_id    TEXT REFERENCES story(id),
    trace_id    TEXT REFERENCES trace(id),
    type        TEXT NOT NULL CHECK(type IN (
                  'design_decision','deviation','tradeoff','open_question'
                )),
    summary     TEXT NOT NULL,
    component   TEXT,
    notes       TEXT
);
INSERT INTO story_signal_new
    SELECT CAST(id AS TEXT), created_at, story_id, CAST(trace_id AS TEXT),
           type, summary, component, notes
    FROM story_signal;
DROP TABLE story_signal;
ALTER TABLE story_signal_new RENAME TO story_signal;

PRAGMA foreign_keys = ON;

-- Cutover cache state (US-028b stage 3/4). Cache artifacts only.
CREATE TABLE cache_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE event_watermark (
    file           TEXT PRIMARY KEY,   -- file name inside .harness/events/
    consumed_count INTEGER NOT NULL,
    file_size      INTEGER NOT NULL,
    file_mtime_ns  INTEGER NOT NULL,
    content_hash   TEXT    NOT NULL    -- fnv1a64 of consumed bytes
);

CREATE TABLE field_last_update (
    story_id   TEXT NOT NULL,
    field      TEXT NOT NULL,
    event_id   TEXT NOT NULL,
    writer     TEXT NOT NULL,
    writer_seq INTEGER NOT NULL,       -- 1-based line number in writer file
    PRIMARY KEY (story_id, field)
);

CREATE TABLE lww_audit (
    loser_event_id  TEXT NOT NULL,
    winner_event_id TEXT NOT NULL,
    story_id        TEXT NOT NULL,
    field           TEXT NOT NULL,
    loser_writer    TEXT NOT NULL,
    winner_writer   TEXT NOT NULL,
    PRIMARY KEY (loser_event_id, field)
);

INSERT INTO schema_version (version) VALUES (7);
