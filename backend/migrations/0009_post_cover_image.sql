-- Add optional cover image URL to posts.
-- Stored as a plain URL string (relative /uploads/... or absolute).
-- NULL means no cover image — existing posts are unaffected.
ALTER TABLE posts ADD COLUMN cover_image TEXT;
