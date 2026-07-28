CREATE TABLE IF NOT EXISTS public.access_tokens (
    token          TEXT PRIMARY KEY,
    created_at     BIGINT NOT NULL,
    reserved_email VARCHAR(255),
    reserved_at    BIGINT,
    redeemed_by    UUID,
    redeemed_at    BIGINT,
    revoked_at     BIGINT
);

CREATE INDEX IF NOT EXISTS idx_access_tokens_reserved_email ON public.access_tokens(reserved_email);
CREATE INDEX IF NOT EXISTS idx_access_tokens_redeemed_by ON public.access_tokens(redeemed_by);

ALTER TABLE public.verification_codes
    ADD COLUMN IF NOT EXISTS access_token TEXT;

CREATE INDEX IF NOT EXISTS idx_verification_codes_access_token ON public.verification_codes(access_token);
