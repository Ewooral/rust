-- Migration 2: book reviews, each written by one user. Deleting a user
-- deletes their reviews too (ON DELETE CASCADE).
CREATE TABLE reviews (
    id         INTEGER PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    book_title TEXT    NOT NULL CHECK (length(book_title) BETWEEN 1 AND 200),
    rating     INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    body       TEXT    NOT NULL DEFAULT '',
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX reviews_by_user ON reviews (user_id);
CREATE INDEX reviews_by_newest ON reviews (created_at DESC, id DESC);
