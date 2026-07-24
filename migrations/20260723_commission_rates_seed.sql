-- Commission rate baselines (agent pool + TCP allocation).
-- Safe to re-run: creates table/index if missing, inserts defaults only when absent.

CREATE TABLE IF NOT EXISTS public.commission_rates (
    role TEXT PRIMARY KEY,
    commission_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0
);

-- Support upsert if an older schema used a surrogate id instead of role PK.
CREATE UNIQUE INDEX IF NOT EXISTS commission_rates_role_unique
    ON public.commission_rates (role);

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Lead Broker', 5, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Lead Broker');

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Titling Officer', 3, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Titling Officer');

INSERT INTO public.commission_rates (role, commission_rate, updated_at)
SELECT 'Agent', 12, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_rates WHERE role = 'Agent');

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
