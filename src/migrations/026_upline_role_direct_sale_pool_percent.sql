-- NULL = not configured; the app falls back to treating 100% of the shared Agent
-- pool as this role's "Direct buyer" income on a sale they closed personally
-- (today's existing behavior, unchanged until an admin sets this explicitly).
ALTER TABLE public.upline_role_types
    ADD COLUMN IF NOT EXISTS direct_sale_pool_percent DOUBLE PRECISION;
