-- Per-project upline role commission behavior (baseline + direct-sale pool split).
-- Global upline_role_types values remain the seed/default for brand-new projects.

ALTER TABLE public.project_upline_role_rates
    ADD COLUMN IF NOT EXISTS has_baseline BOOLEAN,
    ADD COLUMN IF NOT EXISTS direct_sale_pool_percent DOUBLE PRECISION;

UPDATE public.project_upline_role_rates r
   SET has_baseline = u.has_baseline,
       direct_sale_pool_percent = u.direct_sale_pool_percent
  FROM public.upline_role_types u
 WHERE r.upline_role_type_slug = u.slug
   AND r.has_baseline IS NULL;
