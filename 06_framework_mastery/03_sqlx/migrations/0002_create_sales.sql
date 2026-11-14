-- Migration 2: record every sale. A book that has been sold can't be deleted
-- any more, because the foreign key below would point at nothing.
CREATE TABLE sales (
    id          INTEGER PRIMARY KEY,
    book_id     INTEGER NOT NULL REFERENCES books (id),
    quantity    INTEGER NOT NULL CHECK (quantity > 0),
    total_pence INTEGER NOT NULL CHECK (total_pence >= 0),
    sold_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX sales_by_book ON sales (book_id);
