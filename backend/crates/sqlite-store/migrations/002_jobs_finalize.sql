DROP TABLE jobs;
ALTER TABLE jobs_next RENAME TO jobs;

CREATE INDEX jobs_claimable ON jobs (tenant_id, job_id) WHERE status IN ('pending', 'running');
