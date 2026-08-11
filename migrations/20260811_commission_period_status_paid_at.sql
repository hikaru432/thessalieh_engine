-- Persists the date a period actually transitioned to "paid", separate from
-- updated_at (which resets on ANY re-save of the row, including an unrelated
-- correction) — see upsert_commission_status, which preserves this once set and
-- only refreshes it the first time a period becomes paid.

ALTER TABLE public.commission_period_status
    ADD COLUMN IF NOT EXISTS paid_at date;
