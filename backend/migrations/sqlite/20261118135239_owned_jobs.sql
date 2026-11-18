-- Create "job_outbox" table
CREATE TABLE `job_outbox` (
  `sequence` integer NULL PRIMARY KEY AUTOINCREMENT,
  `event_id` text NOT NULL,
  `job_id` text NOT NULL,
  `kind` text NOT NULL,
  `published` integer NOT NULL DEFAULT 0,
  CONSTRAINT `0` FOREIGN KEY (`job_id`) REFERENCES `jobs` (`job_id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CHECK (kind IN ('job_submitted', 'job_cancelled', 'job_completed')),
  CHECK (published IN (0, 1))
) STRICT;
-- Create index "job_outbox_event_id" to table: "job_outbox"
CREATE UNIQUE INDEX `job_outbox_event_id` ON `job_outbox` (`event_id`);
-- Create index "job_outbox_pending" to table: "job_outbox"
CREATE INDEX `job_outbox_pending` ON `job_outbox` (`sequence`) WHERE published = 0;
-- Create "jobs" table
CREATE TABLE `jobs` (
  `job_id` text NOT NULL,
  `owner_id` text NOT NULL,
  `idempotency_key` text NOT NULL,
  `input_ref` text NOT NULL,
  `status` text NOT NULL,
  `lease_id` text NULL,
  `lease_expires_ms` integer NULL,
  `result_ref` text NULL,
  PRIMARY KEY (`job_id`),
  CHECK (status IN ('pending', 'running', 'completed', 'cancelled')),
  CHECK (
        (status = 'running' AND lease_id IS NOT NULL AND lease_expires_ms IS NOT NULL
         AND lease_expires_ms > 0 AND result_ref IS NULL)
        OR (status = 'completed' AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NOT NULL)
        OR (status IN ('pending', 'cancelled') AND lease_id IS NULL AND lease_expires_ms IS NULL
            AND result_ref IS NULL)
    )
) STRICT;
-- Create index "jobs_owner_id_idempotency_key" to table: "jobs"
CREATE UNIQUE INDEX `jobs_owner_id_idempotency_key` ON `jobs` (`owner_id`, `idempotency_key`);
-- Create index "jobs_claimable" to table: "jobs"
CREATE INDEX `jobs_claimable` ON `jobs` (`job_id`) WHERE status IN ('pending', 'running');

-- Identify the owned-store format in this fresh database.
PRAGMA application_id = 1330791240;
PRAGMA user_version = 3;
