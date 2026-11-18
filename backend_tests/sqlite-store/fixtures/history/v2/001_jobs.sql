CREATE TABLE jobs (
    tenant_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'cancelled')),
    PRIMARY KEY (tenant_id, job_id),
    UNIQUE (tenant_id, idempotency_key)
) STRICT;
