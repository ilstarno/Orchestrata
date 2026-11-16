CREATE TABLE solver_jobs (
    id UUID PRIMARY KEY,
    status VARCHAR(24) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    CONSTRAINT solver_jobs_status_valid CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    )
);

CREATE INDEX solver_jobs_status_created
    ON solver_jobs (status, created_at);
