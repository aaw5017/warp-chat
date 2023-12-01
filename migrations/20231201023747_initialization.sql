CREATE TABLE users (
    id INTEGER NOT NULL,
    email TEXT NOT NULL,
    handle TEXT NOT NULL,
    hashed_password TEXT NOT NULL
);

CREATE TABLE sessions (
    id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    csrf_token TEXT NOT NULL,
    created_at INTEGER
);