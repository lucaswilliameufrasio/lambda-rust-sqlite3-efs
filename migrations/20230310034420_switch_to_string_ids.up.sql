CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users_new (id, name, email, created_at, updated_at)
    SELECT CAST(id AS TEXT), name, email, created_at, updated_at FROM users;

DROP TABLE users;

ALTER TABLE users_new RENAME TO users;
