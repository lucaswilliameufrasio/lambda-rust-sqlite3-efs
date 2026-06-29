CREATE TABLE users_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users_old (name, email, created_at, updated_at)
    SELECT name, email, created_at, updated_at FROM users;

DROP TABLE users;

ALTER TABLE users_old RENAME TO users;
