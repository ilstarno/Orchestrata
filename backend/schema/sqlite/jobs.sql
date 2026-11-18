CREATE TABLE jobs (
    job_id TEXT NOT NULL PRIMARY KEY,
    owner_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'cancelled')),
    lease_id TEXT,
    lease_expires_ms INTEGER,
    result_ref TEXT,
    UNIQUE (owner_id, idempotency_key),
    CHECK (
        (status = 'running' AND lease_id IS NOT NULL AND lease_expires_ms IS NOT NULL
         AND lease_expires_ms > 0 AND result_ref IS NULL)
        OR (status = 'completed' AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NOT NULL)
        OR (status IN ('pending', 'cancelled') AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NULL)
    )
) STRICT;

CREATE INDEX jobs_claimable ON jobs (job_id) WHERE status IN ('pending', 'running');
