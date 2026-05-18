-- Replace the 'simple' tsvector full-text search with pg_trgm trigram
-- substring search. This fixes Chinese content being unsearchable because
-- the 'simple' dictionary treats the entire CJK run as one token.
--
-- Trade-off: no linguistic ranking, but ILIKE '%q%' works for any language.

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Drop the old GIN index on the generated tsvector column.
DROP INDEX IF EXISTS posts_search_idx;

-- Remove the generated search_vector column.
ALTER TABLE posts DROP COLUMN IF EXISTS search_vector;

-- Trigram GIN indexes on the three searchable fields.
-- GIN + gin_trgm_ops makes ILIKE fast even on large text columns.
CREATE INDEX posts_trgm_title_idx   ON posts USING GIN (title        gin_trgm_ops);
CREATE INDEX posts_trgm_excerpt_idx ON posts USING GIN (excerpt      gin_trgm_ops);
CREATE INDEX posts_trgm_content_idx ON posts USING GIN (content_md   gin_trgm_ops);
