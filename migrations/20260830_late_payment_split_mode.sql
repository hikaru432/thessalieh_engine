-- Per split-change row: when a catch-up installment is paid in a biweek governed by
-- a newer split, whether to re-rate it at the new split (adopt_new_split) or keep
-- the due-period genesis rate (keep_due_period_split). Existing rows default to
-- legacy behavior.

ALTER TABLE public.contract_split_history
    ADD COLUMN IF NOT EXISTS late_payment_split_mode TEXT NOT NULL DEFAULT 'keep_due_period_split'
    CHECK (late_payment_split_mode IN ('adopt_new_split', 'keep_due_period_split'));
