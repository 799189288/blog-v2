-- Per-post view counter. Incremented by the public GET /api/posts/:slug
-- endpoint. Default 0 covers existing rows.

ALTER TABLE posts ADD COLUMN views BIGINT NOT NULL DEFAULT 0;
