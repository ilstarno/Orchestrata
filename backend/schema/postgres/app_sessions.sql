CREATE TABLE IF NOT EXISTS app_sessions (
    session_digest BYTEA PRIMARY KEY CHECK (octet_length(session_digest) = 32),
    principal_id UUID NOT NULL,
    expires_at_unix_ms BIGINT NOT NULL CHECK (expires_at_unix_ms > 0)
);

CREATE INDEX IF NOT EXISTS app_sessions_expiry ON app_sessions (expires_at_unix_ms);
