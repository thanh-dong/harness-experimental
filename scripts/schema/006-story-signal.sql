-- Harness v0 schema - migration 006
-- Story signals: the mineable subset of implementation-notes.html.
-- Distinct from interventions (external corrections) and trace friction
-- (per-run friction). These are the agent's own design narrative, typed by the
-- four implementation-note categories so `propose` can mine recurrences.

CREATE TABLE story_signal (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    story_id    TEXT REFERENCES story(id),
    trace_id    INTEGER REFERENCES trace(id),
    type        TEXT NOT NULL CHECK(type IN (
                  'design_decision','deviation','tradeoff','open_question'
                )),
    summary     TEXT NOT NULL,
    component   TEXT,
    notes       TEXT
);

INSERT INTO schema_version (version) VALUES (6);
