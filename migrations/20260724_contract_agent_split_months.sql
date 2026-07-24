ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS agent_commission_split_months INTEGER NOT NULL DEFAULT 36;

UPDATE public.contracts
SET agent_commission_split_months = COALESCE(
  (SELECT agent_commission_split_months FROM public.company_settings LIMIT 1),
  36
);
