-- Admin-configurable upline (org-chart root) role types — replaces the hardcoded
-- Lead Broker / Titling Officer pair with a table so new root roles (each with their
-- own base commission % and self-service portal path) can be added without code changes.
-- Safe to re-run: creates table if missing, seeds only when absent.

CREATE TABLE IF NOT EXISTS public.upline_role_types (
    slug TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    base_commission_percent DOUBLE PRECISION NOT NULL DEFAULT 0,
    portal_path TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS upline_role_types_label_unique
    ON public.upline_role_types (label);

CREATE UNIQUE INDEX IF NOT EXISTS upline_role_types_portal_path_unique
    ON public.upline_role_types (portal_path);

INSERT INTO public.upline_role_types
    (slug, label, base_commission_percent, portal_path, sort_order, created_at, updated_at)
SELECT 'lead-broker', 'Lead Broker', 5, 'leadbroker', 0,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.upline_role_types WHERE slug = 'lead-broker');

INSERT INTO public.upline_role_types
    (slug, label, base_commission_percent, portal_path, sort_order, created_at, updated_at)
SELECT 'titling-officer', 'Titling Officer', 3, 'titlingofficer', 1,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.upline_role_types WHERE slug = 'titling-officer');

-- users.role now also accepts any upline_role_types.label. Postgres CHECK constraints
-- can't reference another table, so drop the fixed CHECK entirely and validate the
-- allowed set at the application layer instead (same pattern already used for
-- roster.role and commission_rates.role, which have never had a DB-level CHECK).
ALTER TABLE public.users DROP CONSTRAINT IF EXISTS users_role_check;
