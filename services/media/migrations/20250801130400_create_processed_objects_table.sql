-- Create table to track processed S3 objects
CREATE TABLE processed_s3_objects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    s3_key VARCHAR(500) NOT NULL UNIQUE,
    etag VARCHAR(255) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_processed_s3_objects_s3_key ON processed_s3_objects(s3_key);
CREATE INDEX idx_processed_s3_objects_etag ON processed_s3_objects(etag);
CREATE INDEX idx_processed_s3_objects_processed_at ON processed_s3_objects(processed_at);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_processed_s3_objects_updated_at BEFORE UPDATE ON processed_s3_objects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();