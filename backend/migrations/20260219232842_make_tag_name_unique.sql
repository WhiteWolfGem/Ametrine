-- Add migration script here
ALTER TABLE tag_stats ADD CONSTRAINT unique_tag_name UNIQUE (tag_name);
