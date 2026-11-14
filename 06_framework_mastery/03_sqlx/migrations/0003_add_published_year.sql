-- Migration 3: the schema grows. Never edit a migration that has already run
-- somewhere; add a new one instead. Existing rows get NULL for the new column.
ALTER TABLE books ADD COLUMN published_year INTEGER;
