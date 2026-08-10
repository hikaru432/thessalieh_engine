-- Admin-entered override for a buyer row's commission amount in a given period
-- (used by the Promo commission view — a promo sale's automatic %-computed amount
-- can be manually adjusted; any shortfall carries forward onto the next period,
-- computed client-side from this stored override). NULL = no override, use the
-- automatic computed amount unchanged.
ALTER TABLE public.commission_row_meta
    ADD COLUMN IF NOT EXISTS override_amount DOUBLE PRECISION;
