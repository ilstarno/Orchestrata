-- Create "column_mappings" table
CREATE TABLE `column_mappings` (
  `mapping_id` text NOT NULL,
  `owner_id` text NOT NULL,
  `mapping_json` text NOT NULL,
  PRIMARY KEY (`mapping_id`),
  CHECK (length(CAST(mapping_json AS BLOB)) BETWEEN 1 AND 65536)
) STRICT;

PRAGMA user_version = 4;
