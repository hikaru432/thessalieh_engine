CREATE TABLE IF NOT EXISTS public.password_reset_codes (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email            VARCHAR(255) NOT NULL,
    code             VARCHAR(6) NOT NULL,
    expires_at       BIGINT NOT NULL,
    created_at       BIGINT NOT NULL,
    failed_attempts  INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_password_reset_codes_email ON public.password_reset_codes(email);
