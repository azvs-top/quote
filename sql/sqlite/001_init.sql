-- 先删除触发器与表，便于重复初始化
DROP TRIGGER IF EXISTS trg_quote_set_updated_at;
DROP TABLE IF EXISTS quote;

-- 创建表
CREATE TABLE quote
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    inline     TEXT     NOT NULL DEFAULT '{}', -- JSON 多语言（object）
    external   TEXT     NOT NULL DEFAULT '{}', -- JSON 多语言（object）
    markdown   TEXT     NOT NULL DEFAULT '{}', -- JSON 多语言（object）
    image      TEXT     NOT NULL DEFAULT '[]', -- JSON 数组（array）
    remark     TEXT,
    created_at DATETIME NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    updated_at DATETIME NOT NULL DEFAULT (CURRENT_TIMESTAMP),

    -- JSON 完整性约束
    CONSTRAINT quote_inline_is_object
        CHECK (json_valid(inline) AND json_type(inline) = 'object'),
    CONSTRAINT quote_external_is_object
        CHECK (json_valid(external) AND json_type(external) = 'object'),
    CONSTRAINT quote_markdown_is_object
        CHECK (json_valid(markdown) AND json_type(markdown) = 'object'),
    CONSTRAINT quote_image_is_array
        CHECK (json_valid(image) AND json_type(image) = 'array'),

    -- 业务约束：至少一种内容非空
    CONSTRAINT quote_has_content
        CHECK (
            inline <> '{}'
            OR external <> '{}'
            OR markdown <> '{}'
            OR json_array_length(image) > 0
        )
);

-- 创建触发器：更新记录时自动更新时间戳
-- 使用 AFTER UPDATE + WHEN，避免无条件二次更新导致的递归风险。
CREATE TRIGGER trg_quote_set_updated_at
    AFTER UPDATE
    ON quote
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE quote
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- 时间索引
CREATE INDEX idx_quote_created_at ON quote (created_at DESC);
CREATE INDEX idx_quote_updated_at ON quote (updated_at DESC);

-- 常用语言 key 的表达式索引（按需扩展）
CREATE INDEX idx_quote_inline_en ON quote (json_extract(inline, '$.en'));
CREATE INDEX idx_quote_inline_zh ON quote (json_extract(inline, '$.zh'));
