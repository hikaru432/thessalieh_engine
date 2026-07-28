-- Fix #3: Per-email brute-force protection on /auth/verify
ALTER TABLE public.verification_codes
    ADD COLUMN IF NOT EXISTS failed_attempts INT NOT NULL DEFAULT 0;

-- Fix #4: Access tokens expire
ALTER TABLE public.access_tokens
    ADD COLUMN IF NOT EXISTS expires_at BIGINT;

CREATE INDEX IF NOT EXISTS idx_access_tokens_expires_at ON public.access_tokens(expires_at);
