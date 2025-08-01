-- Add search indices to media_items table
-- Note: We're assuming there might be a title column in the metadata JSON field
-- For better search performance, we might want to extract frequently queried fields
-- from the metadata JSON into separate columns in the future

-- Create indices on existing columns that are likely to be queried
CREATE INDEX IF NOT EXISTS idx_media_items_type_status ON media_items(type, status);
CREATE INDEX IF NOT EXISTS idx_media_items_user_id_status ON media_items(user_id, status);
CREATE INDEX IF NOT EXISTS idx_media_items_created_at ON media_items(created_at DESC);

-- Create a GIN index on the metadata JSONB column for efficient JSON queries
CREATE INDEX IF NOT EXISTS idx_media_items_metadata_gin ON media_items USING GIN(metadata);

-- If we need to search by title, we could add a generated column and index it
-- ALTER TABLE media_items ADD COLUMN IF NOT EXISTS title TEXT 
-- GENERATED ALWAYS AS (metadata->>'title') STORED;
-- CREATE INDEX IF NOT EXISTS idx_media_items_title ON media_items(title);