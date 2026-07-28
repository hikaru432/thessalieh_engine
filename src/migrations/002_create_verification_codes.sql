DROP TABLE IF EXISTS public.verification_codes;

CREATE TABLE public.verification_codes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(25) NOT NULL,
    email         VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    code          VARCHAR(6) NOT NULL,
    expires_at    BIGINT NOT NULL,
    created_at    BIGINT NOT NULL
);

CREATE INDEX idx_verification_codes_email ON public.verification_codes(email);
