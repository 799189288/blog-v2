-- Audit log table. One row per privileged action by an admin.
-- The username is denormalized so log rows survive user deletion.

CREATE TABLE audit_logs (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT REFERENCES users(id) ON DELETE SET NULL,
    username    TEXT NOT NULL,
    action      TEXT NOT NULL,
    target_type TEXT,
    target_id   BIGINT,
    detail      JSONB,
    ip          INET,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_logs_created_idx ON audit_logs (created_at DESC);
CREATE INDEX audit_logs_user_idx    ON audit_logs (user_id, created_at DESC);
CREATE INDEX audit_logs_action_idx  ON audit_logs (action, created_at DESC);
