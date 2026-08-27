-- Seeded system role for Marketing roster assignment: a root upline with no company-wide
-- baseline (earns purely from their own tree). Safe to re-run.

INSERT INTO public.upline_role_types
    (slug, label, base_commission_percent, portal_path, sort_order, has_baseline, created_at, updated_at)
SELECT 'normal-upline', 'Normal Upline', 0, 'normalupline', 2, false,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.upline_role_types WHERE slug = 'normal-upline');

INSERT INTO public.project_upline_role_rates
    (project_id, upline_role_type_slug, percent, has_baseline, direct_sale_pool_percent, created_at, updated_at)
SELECT p.id, 'normal-upline', 0, false, NULL,
       extract(epoch from now())::bigint, extract(epoch from now())::bigint
  FROM public.projects p
 WHERE NOT EXISTS (
    SELECT 1 FROM public.project_upline_role_rates r
     WHERE r.project_id = p.id AND r.upline_role_type_slug = 'normal-upline'
 );
