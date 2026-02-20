-- Create the sites table
CREATE TABLE IF NOT EXISTS sites (
    id SERIAL PRIMARY KEY,
    domain TEXT NOT NULL UNIQUE,
    site_mask_bit INTEGER NOT NULL UNIQUE,
    requires_auth BOOLEAN DEFAULT FALSE
);

-- Add visibility to tag_stats
CREATE TABLE IF NOT EXISTS tag_stats (
    tag_name TEXT PRIMARY KEY,
    tag_uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
    use_count INTEGER DEFAULT 0,
    selected_count INTEGER DEFAULT 0,
    visibility_mask INTEGER DEFAULT 1 NOT NULL 
);

-- Update posts table
ALTER TABLE posts ADD COLUMN visibility_mask INTEGER DEFAULT 1 NOT NULL;

-- Make existing posts visible on bit 1 by default
UPDATE posts SET visibility_mask = 1 WHERE visibility_mask IS NULL;
