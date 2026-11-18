CREATE TABLE jobs (
    tenant_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'cancelled')),
    PRIMARY KEY (tenant_id, job_id),
    UNIQUE (tenant_id, idempotency_key)
) STRICT;

CREATE TABLE job_outbox (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE,
    job_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('job_submitted', 'job_cancelled')),
    published INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1)),
    FOREIGN KEY (tenant_id, job_id) REFERENCES jobs (tenant_id, job_id)
) STRICT;

CREATE INDEX job_outbox_pending ON job_outbox (tenant_id, sequence) WHERE published = 0;
