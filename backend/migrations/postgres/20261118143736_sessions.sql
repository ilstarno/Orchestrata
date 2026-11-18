-- Create "app_sessions" table
CREATE TABLE "public"."app_sessions" (
  "session_digest" bytea NOT NULL,
  "principal_id" uuid NOT NULL,
  "expires_at_unix_ms" bigint NOT NULL,
  PRIMARY KEY ("session_digest"),
  CONSTRAINT "app_sessions_expires_at_unix_ms_check" CHECK (expires_at_unix_ms > 0),
  CONSTRAINT "app_sessions_session_digest_check" CHECK (octet_length(session_digest) = 32)
);
-- Create index "app_sessions_expiry" to table: "app_sessions"
CREATE INDEX "app_sessions_expiry" ON "public"."app_sessions" ("expires_at_unix_ms");
