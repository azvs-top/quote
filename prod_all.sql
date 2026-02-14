DROP SCHEMA IF EXISTS quote CASCADE;
CREATE SCHEMA quote;

------------------------------------------------------------------------------------------------------------------------
---------------   Function f_dict
------------------------------------------------------------------------------------------------------------------------

CREATE TABLE quote.dict_type
(
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    dict_key  TEXT    NOT NULL,
    dict_name JSONB   NOT NULL,
    active    BOOLEAN NOT NULL DEFAULT true,
    creator   TEXT    NOT NULL DEFAULT 'user',
    remark    TEXT
);

CREATE UNIQUE INDEX uk_dict_type_key ON quote.dict_type (dict_key);

ALTER TABLE quote.dict_type
    ADD CONSTRAINT ck_dict_type_creator CHECK (creator IN ('system', 'user'));

CREATE OR REPLACE FUNCTION quote.trg_ban_dict_type_update()
    RETURNS trigger AS
$$
BEGIN
    IF OLD.creator = 'system' THEN
        RAISE EXCEPTION
            'dict_type "%" is system-managed and cannot be modified',
            OLD.dict_key;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_before_dict_type_update
    BEFORE UPDATE
    ON quote.dict_type
    FOR EACH ROW
EXECUTE FUNCTION quote.trg_ban_dict_type_update();

CREATE OR REPLACE FUNCTION quote.trg_ban_dict_type_delete()
    RETURNS trigger AS
$$
BEGIN
    IF OLD.creator = 'system' THEN
        RAISE EXCEPTION
            'dict_type "%" is system-managed and cannot be deleted',
            OLD.dict_key;
    END IF;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_before_dict_type_delete
    BEFORE DELETE
    ON quote.dict_type
    FOR EACH ROW
EXECUTE FUNCTION quote.trg_ban_dict_type_delete();

CREATE TABLE quote.dict_data
(
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    dict_type_id BIGINT  NOT NULL
        REFERENCES quote.dict_type (id) ON DELETE RESTRICT,
    dict_key     TEXT    NOT NULL,
    dict_value   JSONB   NOT NULL,
    is_default   BOOLEAN NOT NULL DEFAULT false,
    active       BOOLEAN NOT NULL DEFAULT true,
    creator      TEXT    NOT NULL DEFAULT 'user',
    remark       TEXT
);

CREATE UNIQUE INDEX uk_dict_data_type_key ON quote.dict_data (dict_type_id, dict_key);

CREATE UNIQUE INDEX uk_dict_data_default ON quote.dict_data (dict_type_id) WHERE is_default = true;

ALTER TABLE quote.dict_data
    ADD CONSTRAINT ck_dict_data_creator CHECK (creator IN ('system', 'user'));

CREATE OR REPLACE FUNCTION quote.trg_ban_dict_data_update()
    RETURNS trigger AS
$$
DECLARE
    type_creator TEXT;
BEGIN
    SELECT creator
    INTO type_creator
    FROM quote.dict_type
    WHERE id = OLD.dict_type_id;

    IF type_creator = 'system' AND OLD.creator = 'system' THEN
        RAISE EXCEPTION
            'dict_data "%" of system dict_type cannot be modified',
            OLD.dict_key;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_before_dict_data_update
    BEFORE UPDATE
    ON quote.dict_data
    FOR EACH ROW
EXECUTE FUNCTION quote.trg_ban_dict_data_update();

CREATE OR REPLACE FUNCTION quote.trg_ban_dict_data_delete()
    RETURNS trigger AS
$$
DECLARE
    type_creator TEXT;
BEGIN
    SELECT creator
    INTO type_creator
    FROM quote.dict_type
    WHERE id = OLD.dict_type_id;

    IF type_creator = 'system' AND OLD.creator = 'system' THEN
        RAISE EXCEPTION
            'dict_data "%" of system dict_type cannot be deleted',
            OLD.dict_key;
    END IF;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_before_dict_data_delete
    BEFORE DELETE
    ON quote.dict_data
    FOR EACH ROW
EXECUTE FUNCTION quote.trg_ban_dict_data_delete();

CREATE OR REPLACE FUNCTION quote.f_dict(langs text[] DEFAULT ARRAY ['en', 'zh'])
    RETURNS TABLE
            (
                type_id      bigint,
                type_key     text,
                type_name    text,
                type_active  boolean,
                type_creator text,
                type_remark  text,

                item_id      bigint,
                item_key     text,
                item_value   text,
                is_default   boolean,
                item_active  boolean,
                item_creator text,
                item_remark  text
            )
    LANGUAGE sql
    STABLE
AS
$$
SELECT t.id       AS type_id,
       t.dict_key AS type_key,
       CASE
           WHEN jsonb_typeof(t.dict_name) = 'string'
               THEN t.dict_name #>> '{}'
           ELSE (SELECT t.dict_name ->> lang
                 FROM unnest(langs || ARRAY ['en','zh']) AS lang
                 WHERE t.dict_name ? lang
                 LIMIT 1)
           END    AS type_name,
       t.active   AS type_active,
       t.creator  AS type_creator,
       t.remark   AS type_remark,

       d.id       AS item_id,
       d.dict_key AS item_key,

       CASE
           WHEN jsonb_typeof(d.dict_value) = 'string'
               THEN d.dict_value #>> '{}'
           ELSE (SELECT d.dict_value ->> lang
                 FROM unnest(langs || ARRAY ['en','zh']) AS lang
                 WHERE d.dict_value ? lang
                 LIMIT 1)
           END    AS item_value,
       d.is_default,
       d.active   AS item_active,
       d.creator  AS item_creator,
       d.remark   AS item_remark
FROM quote.dict_data d
         JOIN quote.dict_type t ON t.id = d.dict_type_id;
$$;

INSERT INTO quote.dict_type (id, dict_key, dict_name, creator, remark)
    OVERRIDING SYSTEM VALUE
VALUES (1, 'status', '{
  "zh": "状态",
  "en": "Status"
}', 'system', 'The state of the dictionary.'),
       (2, 'lang', '{
         "zh": "语言",
         "en": "Language"
       }', 'system', 'The languages supported by quote.'),
       (3, 'storage', '{
         "zh": "Quote存储",
         "en": "Quote storage"
       }', 'system', 'Various storage methods of Quotes.');

INSERT INTO quote.dict_data(id, dict_type_id, dict_key, dict_value, creator, is_default)
    OVERRIDING SYSTEM VALUE
VALUES (1, 1, 'enable', '{
  "zh": "启用",
  "en": "Enable"
}', 'system', true),
       (2, 1, 'disable', '{
         "zh": "停用",
         "en": "Disable"
       }', 'system', false);

INSERT INTO quote.dict_data(id, dict_type_id, dict_key, dict_value, creator, is_default, active)
    OVERRIDING SYSTEM VALUE
VALUES (11, 2, 'zh', to_jsonb('中文'::text), 'system', true, true),
       (12, 2, 'en', to_jsonb('English'::text), 'system', false, true),
       (13, 2, 'ja', to_jsonb('日本語'::text), 'user', false, true),
       (14, 2, 'ko', to_jsonb('한국어'::text), 'user', false, false),
       (15, 2, 'fr', to_jsonb('Français'::text), 'user', false, false),
       (16, 2, 'de', to_jsonb('Deutsch'::text), 'user', false, false),
       (17, 2, 'es', to_jsonb('Español'::text), 'user', false, false),
       (18, 2, 'ru', to_jsonb('Русский'::text), 'user', false, false);

INSERT INTO quote.dict_data(id, dict_type_id, dict_key, dict_value, creator, is_default, remark)
    OVERRIDING SYSTEM VALUE
VALUES (101, 3, 'inline', '{
  "zh": "内嵌文本",
  "en": "Inline text"
}', 'system', true, 'The text is short and multiple languages. The data is stored in the "content".'),
       (102, 3, 'external', '{
         "zh": "外部文本",
         "en": "External text"
       }', 'system', false,
        'The text is quite long. The "content" stores the necessary information for external references.'),
       (103, 3, 'markdown', '{
         "zh": "样式文本",
         "en": "Markdown"
       }', 'system', false, 'The text is stored in Markdown format.'),
       (104, 3, 'image', '{
         "zh": "图片",
         "en": "Image"
       }', 'system', false, 'The text is stored in image format, or related image.'),
       (105, 3, 'audio', '{
         "zh": "音频",
         "en": "Audio"
       }', 'system', false, 'The text is stored in audio format, or related audio.');

SELECT setval(pg_get_serial_sequence('quote.dict_type', 'id'), (SELECT MAX(id) FROM quote.dict_type));
SELECT setval(pg_get_serial_sequence('quote.dict_data', 'id'), 1000);

------------------------------------------------------------------------------------------------------------------------
---------------   Table quote
------------------------------------------------------------------------------------------------------------------------

CREATE TABLE quote.quote
(
    id      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    content JSONB   NOT NULL,
    active  BOOLEAN NOT NULL DEFAULT true,
    remark  TEXT
);

CREATE INDEX idx_quote_content_gin ON quote.quote USING GIN (content);

CREATE OR REPLACE FUNCTION quote.trg_check_quote_content_storage()
    RETURNS trigger AS
$$
DECLARE
    invalid_key TEXT;
BEGIN
    -- content 必须是 JSON object
    IF jsonb_typeof(NEW.content) <> 'object' THEN
        RAISE EXCEPTION
            'quote.content must be a JSON object';
    END IF;

    -- 顶层 key 必须全部属于 storage 字典
    SELECT key
    INTO invalid_key
    FROM jsonb_object_keys(NEW.content) AS key
    WHERE key NOT IN (SELECT d.dict_key
                      FROM quote.dict_data d
                               JOIN quote.dict_type t ON t.id = d.dict_type_id
                      WHERE t.dict_key = 'storage'
                        AND d.active = true)
    LIMIT 1;

    IF invalid_key IS NOT NULL THEN
        RAISE EXCEPTION
            'Invalid storage key "%" in quote.content',
            invalid_key;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_before_quote_content_check
    BEFORE INSERT OR UPDATE
    ON quote.quote
    FOR EACH ROW
EXECUTE FUNCTION quote.trg_check_quote_content_storage();

------------------------------------------------------------------------------------------------------------------------
---------------   Data
------------------------------------------------------------------------------------------------------------------------

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "Life is a soup. And I''m a fork.",
    "zh": "世事如羹，我为箸。"
  }
}'::jsonb, '世界如此丰富，我却格格不入。');

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "From sadness comes joy. as dead flowers bring blooms.",
    "zh": "悲伤生喜悦，如枯花绽新蕾。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "The sand moves by little and little, but it moves all the time.",
    "zh": "缓缓而行，时时不停。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "I love three things in this world. Sun, moon and you. Sun for morning, moon for night, and you forever.",
    "zh": "浮世万千，吾爱有三。日，月与卿。日为朝，月为暮，卿为朝朝暮暮。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "Wrinkles should merely indicate where smiles have been.",
    "zh": "皱纹只是微笑留下的痕迹。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "Things do not change, We change.",
    "zh": "万物并没有改变，变的是我们。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "Nurture passes nature.",
    "zh": "教养胜过天性。",
    "ja": "育みは天性に優る。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "I am because you are.",
    "zh": "我因你而存在。"
  }
}'::jsonb, NULL);

INSERT INTO quote.quote(content, remark)
VALUES ('{
  "inline": {
    "en": "You complete me.",
    "zh": "你使我完整。"
  }
}'::jsonb, NULL);