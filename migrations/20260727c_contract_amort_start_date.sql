-- Optional per-contract override for when the recurring monthly amortization schedule
-- begins, independent of approval_at (e.g. approved in July but first installment
-- deliberately starts in September). NULL means "use approval_at", the existing behavior.

ALTER TABLE public.contracts ADD COLUMN IF NOT EXISTS amort_start_date DATE;
