-- Add migration script here
CREATE TABLE rust_test (
    uuid UUID PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP not null,
    hash TEXT not null
);

