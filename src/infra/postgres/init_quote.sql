DROP SCHEMA IF EXISTS quote CASCADE;
CREATE SCHEMA IF NOT EXISTS quote;

CREATE TABLE IF NOT EXISTS quote.quote
(
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    inline     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    external   JSONB       NOT NULL DEFAULT '{}'::jsonb,
    markdown   JSONB       NOT NULL DEFAULT '{}'::jsonb,
    image      JSONB       NOT NULL DEFAULT '[]'::jsonb,
    remark     TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT quote_inline_is_object CHECK (jsonb_typeof(inline) = 'object'),
    CONSTRAINT quote_external_is_object CHECK (jsonb_typeof(external) = 'object'),
    CONSTRAINT quote_markdown_is_object CHECK (jsonb_typeof(markdown) = 'object'),
    CONSTRAINT quote_image_is_array CHECK (jsonb_typeof(image) = 'array'),

    CONSTRAINT quote_has_content CHECK (
        inline <> '{}'::jsonb
            OR external <> '{}'::jsonb
            OR markdown <> '{}'::jsonb
            OR jsonb_array_length(image) > 0
        )
);

CREATE INDEX IF NOT EXISTS idx_quote_created_at ON quote.quote (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_quote_updated_at ON quote.quote (updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_quote_inline_gin ON quote.quote USING GIN (inline);
CREATE INDEX IF NOT EXISTS idx_quote_external_gin ON quote.quote USING GIN (external);
CREATE INDEX IF NOT EXISTS idx_quote_markdown_gin ON quote.quote USING GIN (markdown);
CREATE INDEX IF NOT EXISTS idx_quote_image_gin ON quote.quote USING GIN (image);

CREATE OR REPLACE FUNCTION quote.set_updated_at()
    RETURNS TRIGGER
    LANGUAGE plpgsql
AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_quote_set_updated_at ON quote.quote;
CREATE TRIGGER trg_quote_set_updated_at
    BEFORE UPDATE
    ON quote.quote
    FOR EACH ROW
EXECUTE FUNCTION quote.set_updated_at();