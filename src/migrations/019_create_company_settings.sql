-- Singleton table (id is always 1) holding the org-wide company defaults
-- shown/edited on the admin Settings page.
CREATE TABLE IF NOT EXISTS public.company_settings (
    id             SMALLINT PRIMARY KEY DEFAULT 1,
    company_name   VARCHAR(255) NOT NULL DEFAULT '',
    office_address TEXT NOT NULL DEFAULT '',
    currency       VARCHAR(10) NOT NULL DEFAULT 'PHP',
    timezone       VARCHAR(64) NOT NULL DEFAULT 'Asia/Manila',
    updated_at     BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    CONSTRAINT company_settings_singleton CHECK (id = 1)
);

INSERT INTO public.company_settings (id)
VALUES (1)
ON CONFLICT (id) DO NOTHING;
