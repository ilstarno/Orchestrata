CREATE TABLE jobs_next (
    tenant_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'cancelled')),
    lease_id TEXT,
    lease_expires_ms INTEGER,
    result_ref TEXT,
    PRIMARY KEY (tenant_id, job_id),
    UNIQUE (tenant_id, idempotency_key),
    CHECK (
        (status = 'running' AND lease_id IS NOT NULL AND lease_expires_ms IS NOT NULL
         AND lease_expires_ms > 0 AND result_ref IS NULL)
        OR (status = 'completed' AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NOT NULL)
        OR (status IN ('pending', 'cancelled') AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NULL)
    )
) STRICT;

INSERT INTO jobs_next (tenant_id, job_id, idempotency_key, input_ref, status)
SELECT tenant_id, job_id, idempotency_key, input_ref, status FROM jobs;
