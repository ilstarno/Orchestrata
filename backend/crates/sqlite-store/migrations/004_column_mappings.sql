CREATE TABLE column_mappings (
    mapping_id TEXT NOT NULL PRIMARY KEY,
    owner_id TEXT NOT NULL,
    mapping_json TEXT NOT NULL CHECK (length(CAST(mapping_json AS BLOB)) BETWEEN 1 AND 65536)
) STRICT;
