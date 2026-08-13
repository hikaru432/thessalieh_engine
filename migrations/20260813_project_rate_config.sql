-- Per-project TCP allocation split. Replaces the global, fixed-role commission_rates
-- table as the source of truth for real commission math: each project now has its own
-- dynamic list of allocation categories (label + percent, admin add/rename/remove/reorder)
-- and its own per-upline-role percent within the "agent pool" category. commission_rates
-- and upline_role_types.base_commission_percent stay in place unchanged, but only serve
-- as default/seed values for brand new projects from here on.
-- Safe to re-run: creates tables if missing, backfills only rows that don't exist yet.

CREATE TABLE IF NOT EXISTS public.project_rate_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    label VARCHAR(100) NOT NULL,
    percent DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (percent >= 0 AND percent <= 100),
    is_agent_pool BOOLEAN NOT NULL DEFAULT false,
    sort_order INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (project_id, label)
);

-- Exactly one is_agent_pool row per project — this is the bucket that gets further
-- split among upline roles (project_upline_role_rates below), with the remainder being
-- the shared pool individual downline agents draw their sharePercent from.
CREATE UNIQUE INDEX IF NOT EXISTS project_rate_categories_one_agent_pool
    ON public.project_rate_categories (project_id) WHERE is_agent_pool;

CREATE TABLE IF NOT EXISTS public.project_upline_role_rates (
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    upline_role_type_slug TEXT NOT NULL REFERENCES public.upline_role_types(slug) ON DELETE CASCADE,
    percent DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (percent >= 0 AND percent <= 100),
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (project_id, upline_role_type_slug)
);

-- Backfill every existing project from today's global commission_rates values, so
-- nothing changes for existing projects until an admin edits their Rate Config.
INSERT INTO public.project_rate_categories
    (project_id, label, percent, is_agent_pool, sort_order, created_at, updated_at)
SELECT p.id, fixed.role, COALESCE(r.commission_rate, fixed.default_percent), false, fixed.sort_order,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
FROM public.projects p
CROSS JOIN (VALUES
    ('Legal Counsel', 5.0, 0),
    ('Land Owner', 40.0, 1),
    ('Hypomone', 25.0, 2),
    ('Project Dev & Processing', 10.0, 3)
) AS fixed(role, default_percent, sort_order)
LEFT JOIN public.commission_rates r ON r.role = fixed.role
WHERE NOT EXISTS (
    SELECT 1 FROM public.project_rate_categories c
     WHERE c.project_id = p.id AND c.label = fixed.role
);

INSERT INTO public.project_rate_categories
    (project_id, label, percent, is_agent_pool, sort_order, created_at, updated_at)
SELECT p.id, 'Agent Commission',
       COALESCE(
           (SELECT SUM(commission_rate) FROM public.commission_rates
             WHERE role IN ('Lead Broker', 'Titling Officer', 'Agent')),
           20
       ),
       true, 99, extract(epoch from now())::bigint, extract(epoch from now())::bigint
FROM public.projects p
WHERE NOT EXISTS (
    SELECT 1 FROM public.project_rate_categories c
     WHERE c.project_id = p.id AND c.is_agent_pool
);

INSERT INTO public.project_upline_role_rates
    (project_id, upline_role_type_slug, percent, created_at, updated_at)
SELECT p.id, u.slug, u.base_commission_percent,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
FROM public.projects p
CROSS JOIN public.upline_role_types u
WHERE NOT EXISTS (
    SELECT 1 FROM public.project_upline_role_rates x
     WHERE x.project_id = p.id AND x.upline_role_type_slug = u.slug
);
