-- Optional agent + notes on lot reservation (agents_json node id).
ALTER TABLE public.lots
  ADD COLUMN IF NOT EXISTS reserve_notes TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS reserve_agent_id TEXT NULL;
