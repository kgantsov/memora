CREATE KEYSPACE memora WITH replication = { 'class': 'NetworkTopologyStrategy',
'replication_factor': 1 };
CREATE TABLE IF NOT EXISTS memora.files (
    user_id Uuid,
    id Uuid,
    name Text,
    directory Text,
    file_type Text,
    status Text,
    created_at Timestamp,
    modified_at Timestamp,
    PRIMARY KEY (user_id, id)
);
CREATE MATERIALIZED VIEW IF NOT EXISTS memora.files_by_directory AS
SELECT user_id,
    id,
    name,
    directory,
    file_type,
    status,
    created_at,
    modified_at
FROM memora.files
WHERE directory IS NOT NULL
    AND user_id IS NOT NULL
    AND id IS NOT NULL PRIMARY KEY (user_id, directory, id) WITH CLUSTERING
ORDER BY (directory ASC, id DESC);
CREATE TABLE IF NOT EXISTS memora.users (
    id Uuid,
    email Text,
    password_hash Text,
    first_name Text,
    last_name Text,
    file_type Text,
    status Text,
    created_at Timestamp,
    modified_at Timestamp,
    PRIMARY KEY ((id))
);
CREATE MATERIALIZED VIEW IF NOT EXISTS memora.users_by_email AS
SELECT id,
    email,
    password_hash,
    first_name,
    last_name,
    status,
    created_at,
    modified_at
FROM memora.users
WHERE email IS NOT NULL
    AND id IS NOT NULL PRIMARY KEY ((email), id);