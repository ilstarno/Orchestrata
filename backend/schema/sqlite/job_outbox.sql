CREATE TABLE job_outbox (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,
    job_id TEXT NOT NULL REFERENCES jobs (job_id),
    kind TEXT NOT NULL CHECK (kind IN ('job_submitted', 'job_cancelled', 'job_completed')),
    published INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1))
) STRICT;

CREATE INDEX job_outbox_pending ON job_outbox (sequence) WHERE published = 0;
