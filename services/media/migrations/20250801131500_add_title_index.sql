-- Add generated column and index for title to optimize search queries

ALTER TABLE media_items
ADD COLUMN IF NOT EXISTS title TEXT
GENERATED ALWAYS AS (metadata->>'title') STORED;

CREATE INDEX IF NOT EXISTS idx_media_items_title ON media_items(title);

-- Optionally, add a generated column and index for genre if needed
-- ALTER TABLE media_items
-- ADD COLUMN IF NOT EXISTS genre TEXT
-- GENERATED ALWAYS AS (metadata->>'genre') STORED;
-- CREATE INDEX IF NOT EXISTS idx_media_items_genre ON media_items(genre);

-- Testing:
-- 1. Use EXPLAIN ANALYZE on common queries (by title, type, genre) to verify index effectiveness.
-- 2. Conduct an end-to-end integration test: add a file to S3, verify the pipeline creates the correct DB entry, thumbnail, and logs.