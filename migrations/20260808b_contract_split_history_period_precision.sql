-- The commission release cycle is biweekly (1-15 releases on the 16th; 16-30/31
-- releases on the 1st of the next month), not monthly — a split change made anytime
-- in the 16-30 window must take effect starting with THAT month's second release, not
-- get deferred all the way to periods dated the following calendar month. Rename the
-- column to make clear it can hold the 1st OR the 16th of a month, not just the 1st.

ALTER TABLE public.contract_split_history
    RENAME COLUMN effective_month TO effective_period_start;
