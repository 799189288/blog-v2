-- Register additional audit action codes added by later features (tag CRUD,
-- dict CRUD). Using ON CONFLICT DO NOTHING so this migration is safe if
-- you already manually inserted these.

INSERT INTO dict_items (type_id, code, label_zh, label_en, sort)
SELECT t.id, v.code, v.label_zh, v.label_en, v.sort
FROM dict_types t
JOIN (VALUES
    ('tag.create',         '创建标签',       'Create tag',         70),
    ('tag.update',         '编辑标签',       'Update tag',         80),
    ('tag.delete',         '删除标签',       'Delete tag',         90),
    ('dict.type.create',   '创建字典类型',   'Create dict type',  200),
    ('dict.type.update',   '编辑字典类型',   'Update dict type',  210),
    ('dict.type.delete',   '删除字典类型',   'Delete dict type',  220),
    ('dict.item.create',   '创建字典项',     'Create dict item',  230),
    ('dict.item.update',   '编辑字典项',     'Update dict item',  240),
    ('dict.item.delete',   '删除字典项',     'Delete dict item',  250)
) AS v(code, label_zh, label_en, sort) ON TRUE
WHERE t.code = 'audit.action'
ON CONFLICT (type_id, code) DO NOTHING;
