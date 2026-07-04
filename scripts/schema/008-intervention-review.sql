-- 008: allow the 'review' intervention type.
-- FEATURE_INTAKE.md and IMPROVEMENT_PROTOCOL.md record the high-risk
-- independent check with `intervention add --type review`; the CHECK
-- constraint predated that rule. SQLite cannot alter a CHECK, so rebuild.

CREATE TABLE intervention_new (
    id          TEXT NOT NULL PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    trace_id    TEXT REFERENCES trace(id),
    story_id    TEXT,
    type        TEXT NOT NULL CHECK(type IN ('correction','override','escalation','approval','review')),
    description TEXT NOT NULL,
    source      TEXT NOT NULL CHECK(source IN ('human','reviewer','ci','agent')),
    impact      TEXT
);
INSERT INTO intervention_new
    SELECT id, created_at, trace_id, story_id, type, description, source, impact
    FROM intervention;
DROP TABLE intervention;
ALTER TABLE intervention_new RENAME TO intervention;

INSERT INTO schema_version (version) VALUES (8);
