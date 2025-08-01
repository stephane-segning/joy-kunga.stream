-- Update media_items table to include metadata and thumbnail URL
ALTER TABLE media_items
ADD COLUMN IF NOT EXISTS duration DOUBLE PRECISION,
ADD COLUMN IF NOT EXISTS width INTEGER,
ADD COLUMN IF NOT EXISTS height INTEGER,
ADD COLUMN IF NOT EXISTS video_codec VARCHAR(50),
ADD COLUMN IF NOT EXISTS audio_codec VARCHAR(50),
ADD COLUMN IF NOT EXISTS format VARCHAR(50),
ADD COLUMN IF NOT EXISTS bitrate BIGINT,
ADD COLUMN IF NOT EXISTS sample_rate INTEGER,
ADD COLUMN IF NOT EXISTS channels INTEGER,
ADD COLUMN IF NOT EXISTS thumbnail_url VARCHAR(500);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_media_items_duration ON media_items(duration);
CREATE INDEX IF NOT EXISTS idx_media_items_width ON media_items(width);
CREATE INDEX IF NOT EXISTS idx_media_items_height ON media_items(height);
CREATE INDEX IF NOT EXISTS idx_media_items_video_codec ON media_items(video_codec);
CREATE INDEX IF NOT EXISTS idx_media_items_audio_codec ON media_items(audio_codec);
CREATE INDEX IF NOT EXISTS idx_media_items_format ON media_items(format);
CREATE INDEX IF NOT EXISTS idx_media_items_thumbnail_url ON media_items(thumbnail_url);