-- Word count and reading time, computed from the post's markdown body.
-- The backfill uses a coarse approximation that's good enough for old
-- rows; new/edited posts get the precise count from `markdown::word_count`.
ALTER TABLE posts ADD COLUMN word_count        INT NOT NULL DEFAULT 0;
ALTER TABLE posts ADD COLUMN reading_time_min  INT NOT NULL DEFAULT 0;

-- Rough backfill: count Han/Hangul/Kana characters 1-for-1 plus
-- whitespace-delimited ASCII tokens. This is intentionally simple — the
-- real word_count() will overwrite these on next save.
UPDATE posts SET
    word_count = GREATEST(1,
        (SELECT COUNT(*) FROM regexp_matches(content_md, '[一-鿿가-힯぀-ヿ]', 'g'))
        + array_length(regexp_split_to_array(coalesce(regexp_replace(content_md, '[一-鿿가-힯぀-ヿ]', '', 'g'), ''), '\s+'), 1)
    ),
    reading_time_min = GREATEST(1,
        ((SELECT COUNT(*) FROM regexp_matches(content_md, '[一-鿿가-힯぀-ヿ]', 'g'))
         + array_length(regexp_split_to_array(coalesce(regexp_replace(content_md, '[一-鿿가-힯぀-ヿ]', '', 'g'), ''), '\s+'), 1))
        / 300
    );
