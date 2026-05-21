-- post_tags has a PK on (post_id, tag_id) but no index on tag_id alone.
-- Queries that look up posts by tag (reverse join) do a full scan of the
-- PK index. A dedicated index on tag_id makes those lookups O(log n).
CREATE INDEX post_tags_tag_id_idx ON post_tags (tag_id);

-- comments(parent_id) is used in self-joins when building threaded comment
-- trees. Without an index every parent lookup scans the whole table.
CREATE INDEX comments_parent_id_idx ON comments (parent_id);
