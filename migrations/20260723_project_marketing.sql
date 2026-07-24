ALTER TABLE public.projects
  ADD COLUMN IF NOT EXISTS lead_broker_roster_id UUID REFERENCES public.roster(id),
  ADD COLUMN IF NOT EXISTS titling_officer_roster_id UUID REFERENCES public.roster(id),
  ADD COLUMN IF NOT EXISTS agent_commission_split_months INTEGER NOT NULL DEFAULT 15;

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Legal Counsel', 5, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Legal Counsel');

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Land Owner', 40, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Land Owner');

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Hypomone', 25, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Hypomone');

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Project Dev & Processing', 10, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Project Dev & Processing');
