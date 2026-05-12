-- Dictionary system: typed key/value lookup with bilingual labels.
-- Used to render system enums (audit actions, post/comment status, user roles)
-- and any business taxonomies the admin needs to manage at runtime.

CREATE TABLE dict_types (
    id         BIGSERIAL PRIMARY KEY,
    code       TEXT UNIQUE NOT NULL,
    name_zh    TEXT NOT NULL,
    name_en    TEXT NOT NULL,
    -- System types are seeded here and protected from deletion in admin UI.
    -- The flag does NOT prevent label edits — that's a UI policy.
    is_system  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dict_items (
    id         BIGSERIAL PRIMARY KEY,
    type_id    BIGINT NOT NULL REFERENCES dict_types(id) ON DELETE CASCADE,
    code       TEXT NOT NULL,
    label_zh   TEXT NOT NULL,
    label_en   TEXT NOT NULL,
    sort       INTEGER NOT NULL DEFAULT 0,
    enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (type_id, code)
);

CREATE INDEX dict_items_type_sort_idx ON dict_items (type_id, sort, id);

-- Seed system dictionaries. Adding a new system enum requires:
--   1. The new code added here
--   2. Backend code that writes/reads it
INSERT INTO dict_types (code, name_zh, name_en, is_system) VALUES
    ('audit.action',   '操作日志动作', 'Audit action',    TRUE),
    ('post.status',    '文章状态',     'Post status',     TRUE),
    ('comment.status', '评论状态',     'Comment status',  TRUE),
    ('user.role',      '用户角色',     'User role',       TRUE);

INSERT INTO dict_items (type_id, code, label_zh, label_en, sort) VALUES
    -- audit.action
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'login',                '登录',             'Login',                10),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'post.create',          '创建文章',         'Create post',          20),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'post.update',          '编辑文章',         'Update post',          30),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'post.delete',          '删除文章',         'Delete post',          40),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'comment.set_status',   '修改评论状态',     'Set comment status',   50),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'comment.delete',       '删除评论',         'Delete comment',       60),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'tag.create',           '创建标签',         'Create tag',           70),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'tag.update',           '编辑标签',         'Update tag',           80),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'tag.delete',           '删除标签',         'Delete tag',           90),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'user.create',          '创建用户',         'Create user',         100),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'user.reset_password',  '重置密码',         'Reset password',      110),
    ((SELECT id FROM dict_types WHERE code='audit.action'), 'user.delete',          '删除用户',         'Delete user',         120),
    -- post.status
    ((SELECT id FROM dict_types WHERE code='post.status'),  'draft',                '草稿',             'Draft',                10),
    ((SELECT id FROM dict_types WHERE code='post.status'),  'published',            '已发布',           'Published',            20),
    -- comment.status
    ((SELECT id FROM dict_types WHERE code='comment.status'), 'pending',            '待审',             'Pending',              10),
    ((SELECT id FROM dict_types WHERE code='comment.status'), 'approved',           '已通过',           'Approved',             20),
    ((SELECT id FROM dict_types WHERE code='comment.status'), 'spam',               '垃圾',             'Spam',                 30),
    -- user.role
    ((SELECT id FROM dict_types WHERE code='user.role'),    'admin',                '管理员',           'Admin',                10);
