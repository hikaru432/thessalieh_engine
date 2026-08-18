-- Per split-change row: how future commission amounts are recomputed after a
-- mid-stream split update. Existing rows default to catch_up (preserves live
-- projects already changed before this column existed).

ALTER TABLE public.contract_split_history
    ADD COLUMN IF NOT EXISTS rebalance_strategy TEXT NOT NULL DEFAULT 'catch_up'
    CHECK (rebalance_strategy IN ('even_split', 'catch_up'));
