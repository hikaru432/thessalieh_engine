-- Ties each commission_period_status row to a specific buyer/contract row (row_key),
-- not just the subject agent + period, so status/partial-payment can be set per buyer.

ALTER TABLE public.commission_period_status
    ADD COLUMN IF NOT EXISTS row_key TEXT NOT NULL DEFAULT '';

ALTER TABLE public.commission_period_status
    DROP CONSTRAINT IF EXISTS commission_period_status_unique;

ALTER TABLE public.commission_period_status
    ADD CONSTRAINT commission_period_status_unique
        UNIQUE (project_id, subject_agent_id, row_key, period_start);
