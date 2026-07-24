ALTER TABLE public.company_settings
  ADD COLUMN IF NOT EXISTS agent_commission_split_months INTEGER NOT NULL DEFAULT 15;
