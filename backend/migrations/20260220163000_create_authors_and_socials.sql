-- Create authors table
CREATE TABLE IF NOT EXISTS authors (
    id SERIAL PRIMARY KEY,
    uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
    name TEXT NOT NULL,
    bio TEXT,
    signing_email TEXT -- GPG email for verification
);

-- Create author_socials table
CREATE TABLE IF NOT EXISTS author_socials (
    id SERIAL PRIMARY KEY,
    author_uuid UUID NOT NULL REFERENCES authors(uuid) ON DELETE CASCADE,
    platform TEXT NOT NULL, -- e.g., 'github', 'x', 'bluesky', 'instagram'
    handle TEXT NOT NULL,
    url TEXT,
    visibility_mask INTEGER DEFAULT 1 NOT NULL -- Controls which sites this social appears on
);

-- Add author_uuid to posts
ALTER TABLE posts ADD COLUMN author_uuid UUID REFERENCES authors(uuid);
