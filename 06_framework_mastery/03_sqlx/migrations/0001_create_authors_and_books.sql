-- Migration 1: the first version of the schema.
-- Money is stored as whole pence (INTEGER), never as a floating-point number.
CREATE TABLE authors (
    id   INTEGER PRIMARY KEY,
    name TEXT    NOT NULL UNIQUE
);

CREATE TABLE books (
    id          INTEGER PRIMARY KEY,
    isbn        TEXT    NOT NULL UNIQUE CHECK (length(isbn) = 13),
    title       TEXT    NOT NULL CHECK (title <> ''),
    author_id   INTEGER NOT NULL REFERENCES authors (id),
    price_pence INTEGER NOT NULL CHECK (price_pence >= 0),
    stock       INTEGER NOT NULL DEFAULT 0 CHECK (stock >= 0)
);
