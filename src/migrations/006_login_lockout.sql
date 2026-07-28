-- Per-email login brute-force lockout
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS failed_login_attempts INT    NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS lockout_until         BIGINT NOT NULL DEFAULT 0;
