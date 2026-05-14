-- Draft preview tokens. NULL when the post is published (no need —
-- public can already read it). Populated when status='draft' so the
-- author can share a private preview link.
--
-- Tokens are 32-char hex (128 bits) generated at draft time and rotated
-- whenever a post transitions back to draft. Indexed UNIQUE because the
-- public read path queries by slug+token in a single round trip.

ALTER TABLE posts ADD COLUMN preview_token TEXT;
CREATE UNIQUE INDEX posts_preview_token_idx
    ON posts (preview_token)
    WHERE preview_token IS NOT NULL;
