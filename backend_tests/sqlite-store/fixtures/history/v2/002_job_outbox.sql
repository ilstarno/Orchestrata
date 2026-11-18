CREATE TABLE job_outbox_next (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE,
    job_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('job_submitted', 'job_cancelled', 'job_completed')),
    published INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1)),
    FOREIGN KEY (tenant_id, job_id) REFERENCES jobs_next (tenant_id, job_id)
) STRICT;

INSERT INTO job_outbox_next (sequence, tenant_id, event_id, job_id, kind, published)
SELECT sequence, tenant_id, event_id, job_id, kind, published FROM job_outbox;

DROP TABLE job_outbox;
ALTER TABLE job_outbox_next RENAME TO job_outbox;

CREATE INDEX job_outbox_pending ON job_outbox (tenant_id, sequence) WHERE published = 0;
