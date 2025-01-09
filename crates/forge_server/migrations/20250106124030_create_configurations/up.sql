CREATE TABLE IF NOT EXISTS configuration_table (
    id TEXT NOT NULL PRIMARY KEY,
    data TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);